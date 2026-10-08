<#
.SYNOPSIS
组装发布资产（T-094 / 第十期 P-01 发布分发闭环）：在发布目录生成可分发包 +
签名 manifest + 安装 zip，并输出上传核对清单（SHA256SUMS.txt）。

流程（在发布环境执行；私钥只在发布环境，绝不入库）：
  1. 首次发布先运行 `zhu-ye-dict keygen` 生成密钥对；
  2. 私钥写入发布环境（如系统密钥库/受保护文件），公钥配置到构建环境：
       $env:ZHU_YE_RELEASE_SECRET_KEY = '<32 字节私钥 hex>'
       $env:ZHU_YE_RELEASE_PUBLIC_KEY = '<32 字节公钥 hex>'
  3. 运行本脚本：
       .\scripts\assemble-release.ps1
  4. 按输出的 SHA256SUMS.txt 上传 GitHub Releases 资产（zip + 各包 + manifest.json）。

产物布局（release-assets/<version>/）：
  zhu-ye-ime-<version>.zip     安装/便携包（含 base.zyct、en.zyen、DLL、两个 exe；命名见 T-108）
  packs/it.zyct med.zyct slang.zyct   可分发的领域包（DISTRIBUTABLE_PACK_IDS）
  manifest.json                 签名 manifest（schema 1，ed25519）
  SHA256SUMS.txt                上传核对清单

注意：
  - 只分发包进 manifest（updater 的 DISTRIBUTABLE_PACK_IDS = it/med/slang）；
    base.zyct 随安装包只读交付、en.zyen 是引擎资产，均不进更新源；
  - ZHU_YE_RELEASE_PUBLIC_KEY 参与编译（option_env!），变更公钥后必须先
    重编 zhu-ye-updater（本脚本对 updater 强制 clean 重建，避免增量缓存旧公钥）；
  - 私钥丢失无恢复手段：重新 keygen 并随下一次引擎发布轮换信任锚。
#>
[CmdletBinding()]
param(
    # 发布版本（默认读根 Cargo.toml workspace.package.version）。
    [string]$Version,
    # 发布资产输出目录（相对仓库根；默认 target/release-assets，不入 git）。
    [string]$OutDir = 'target/release-assets',
    # 跳过 release 构建（仅使用现有 target\release 产物；不推荐，见公钥注意）。
    [switch]$SkipBuild,
    # 不签名 manifest（dry-run 验证用；发布必须签名）。
    [switch]$SkipSign
)

$ErrorActionPreference = 'Stop'
$repoRoot = Split-Path -Parent $PSScriptRoot
$dictExe = Join-Path $repoRoot 'target\release\zhu-ye-dict.exe'

# ---- 1. 版本 ----
if (-not $Version) {
    $match = Select-String -LiteralPath (Join-Path $repoRoot 'Cargo.toml') -Pattern '^version = "([^"]+)"' |
        Select-Object -First 1
    if ($null -eq $match) {
        throw '无法从根 Cargo.toml 读取 version。'
    }
    $Version = $match.Matches[0].Groups[1].Value
}
$packVersion = (Get-Date -Format 'yyyy-MM-dd')   # 词典包版本独立于引擎版本（P-06）
$minEngine = '0.1.1'                              # 首个含 updater/可分发包的引擎（M6-U，T-051）

# ---- 2. 发布公钥检查 ----
if (-not $env:ZHU_YE_RELEASE_PUBLIC_KEY) {
    throw ('缺少 ZHU_YE_RELEASE_PUBLIC_KEY（发布公钥）。首次发布请先运行 `zhu-ye-dict keygen`，' +
        '并把公钥配置到发布构建环境（公钥可入库，私钥 ZHU_YE_RELEASE_SECRET_KEY 不入库）。')
}
if (-not $SkipSign -and -not $env:ZHU_YE_RELEASE_SECRET_KEY) {
    throw '缺少 ZHU_YE_RELEASE_SECRET_KEY（发布私钥）。私钥只应存在于发布环境，绝不写入仓库。'
}

# ---- 3. 构建 release 产物（注入公钥） ----
if (-not $SkipBuild) {
    Push-Location $repoRoot
    try {
        # 强制重编 updater：增量缓存无法感知 option_env! 的公钥变化。
        & cargo clean -p zhu-ye-updater 2>$null
        & cargo build --release -p zhu-ye-dict -p zhu-ye-ime -p zhu-ye-settings -p zhu-ye-updater -p zhu-ye-tray
        if ($LASTEXITCODE -ne 0) {
            throw 'cargo build 失败，无法生成发布产物。'
        }
    } finally {
        Pop-Location
    }
}
foreach ($artifact in @($dictExe, (Join-Path $repoRoot 'target\release\zhu_ye_ime.dll'),
        (Join-Path $repoRoot 'target\release\zhu-ye-settings.exe'),
        (Join-Path $repoRoot 'target\release\zhu-ye-updater.exe'),
        (Join-Path $repoRoot 'target\release\zhu-ye-tray.exe'))) {
    if (-not (Test-Path -LiteralPath $artifact -PathType Leaf)) {
        throw "release 产物缺失：$artifact（请勿使用 -SkipBuild）"
    }
}

# ---- 4. 组装发布目录 ----
$relRoot = Join-Path $repoRoot $OutDir
$relDir = Join-Path $relRoot $Version
foreach ($dir in @($relRoot, $relDir, (Join-Path $relDir 'packs'))) {
    if (-not (Test-Path -LiteralPath $dir -PathType Container)) {
        $null = New-Item -ItemType Directory -Path $dir -Force
    }
}

$packsDir = Join-Path $relDir 'packs'
$artifacts = Join-Path $repoRoot 'data\artifacts'
foreach ($pack in @('it', 'med', 'slang')) {
    $src = Join-Path $artifacts "$pack.zyct"
    if (-not (Test-Path -LiteralPath $src -PathType Leaf)) {
        throw "领域包缺失：$src（先运行 zhu-ye-dict build-pack $pack）"
    }
    Copy-Item -LiteralPath $src -Destination (Join-Path $packsDir "$pack.zyct") -Force
}

# ---- 5. manifest：生成 → 签名 → 复核 ----
& $dictExe build-manifest $packsDir --version $packVersion --min-engine $minEngine
if ($LASTEXITCODE -ne 0) { throw 'build-manifest 失败。' }
$manifestPath = Join-Path $packsDir 'manifest.json'
if (-not $SkipSign) {
    & $dictExe sign-manifest $manifestPath
    if ($LASTEXITCODE -ne 0) { throw 'sign-manifest 失败。' }
}
# 签名/哈希复核：逐包内容哈希与大小必须与 manifest 一致（发布检查清单 7.4）。
& $dictExe verify-manifest $manifestPath
if ($LASTEXITCODE -ne 0) { throw 'verify-manifest 失败（内容哈希与 manifest 不一致）。' }

# ---- 6. 安装/便携 zip（复用 package-portable，产物已构建则自动跳过构建） ----
& (Join-Path $repoRoot 'scripts\package-portable.ps1') -Version $Version
if ($LASTEXITCODE -ne 0) { throw 'package-portable 失败。' }
$portableZip = Join-Path $repoRoot "target\portable\zhu-ye-ime-$Version-test.zip"
if (-not (Test-Path -LiteralPath $portableZip -PathType Leaf)) {
    throw "便携包未生成：$portableZip"
}
$zipDest = Join-Path $relDir "zhu-ye-ime-$Version.zip"
Copy-Item -LiteralPath $portableZip -Destination $zipDest -Force
# 载荷完整性守卫：zip 必须含三个 exe（设置/更新器/托盘，T-078 + 批六）；缺一即中止，
# 避免把缺组件的发布包上传出去。
Add-Type -AssemblyName System.IO.Compression.FileSystem
$zipRead = [System.IO.Compression.ZipFile]::OpenRead((Resolve-Path $zipDest))
try {
    $zipNames = @($zipRead.Entries | ForEach-Object { $_.FullName })
    foreach ($entry in @('bin/zhu-ye-tray.exe', 'bin/zhu-ye-settings.exe', 'bin/zhu-ye-updater.exe', 'bin/zhu_ye_ime.dll')) {
        if ($zipNames -notcontains $entry) {
            throw "发布 zip 缺少组件 $entry —— 发行包组装有缺口，中止发布。"
        }
    }
} finally {
    $zipRead.Dispose()
}

# ---- 7. 上传核对清单 ----
$sums = Join-Path $relDir 'SHA256SUMS.txt'
Get-ChildItem -LiteralPath $relDir -Recurse -File | Where-Object {
    $_.Name -ne 'SHA256SUMS.txt'
} | ForEach-Object {
    $hash = (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
    $relative = $_.FullName.Substring($relDir.Length + 1)
    '{0}  {1}  {2}' -f $hash, $_.Length, $relative
} | Sort-Object | Set-Content -LiteralPath $sums -Encoding utf8

Write-Host ''
Write-Host '发布资产已就绪：'
Write-Host "  目录: $relDir"
Write-Host "  包版本: $packVersion  最低引擎: $minEngine"
Write-Host "  签名: $(if ($SkipSign) { '未签名（dry-run）' } else { 'ed25519 已签名' })"
Write-Host "  上传: gh release create v$Version --title 'v$Version' --notes '发布说明（见 CHANGELOG）'"
$sumsRel = (Join-Path $relDir 'SHA256SUMS.txt').Replace('\', '/')
Write-Host "        后 gh release upload v$Version $sumsRel"
Write-Host '        并按 SHA256SUMS.txt 上传 zip、packs/*.zyct 与 manifest.json'
Write-Host '  验证: ZHU_YE_MANIFEST_URL 指向实际 releases 地址后运行 updater check/apply'
