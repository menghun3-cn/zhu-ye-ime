#requires -Version 5.1
<#
.SYNOPSIS
生成发行包：包含 TSF 服务、设置窗口、更新器、基础词典与三个领域包（T-078）。

.DESCRIPTION
发行包布局（可在无源码树与无 Rust 工具链的机器上直接安装）：

    zhu-ye-ime-<version>-test/
      bin/zhu_ye_ime.dll             TSF 服务 DLL
      bin/zhu-ye-settings.exe        设置窗口
      bin/zhu-ye-updater.exe         词典更新器（唯一联网组件）
      bin/dictionary.zyct            基础词典（data/artifacts/base.zyct，随安装只读）
      bin/en.zyen                    英文词表（T-085，data/artifacts/en.zyen，随安装只读）
      packs/it.zyct med.zyct slang.zyct  预置领域包（D-46，可离线验收）
      scripts/ime-identity.ps1 install.ps1 uninstall.ps1 verify-tsf-dll.ps1 verify-tsf-identity.ps1
      docs/licenses.md 数据清单.md
      README-测试.txt

打包步骤：
1. 若缺失 release 产物，先构建 zhu-ye-ime / zhu-ye-settings / zhu-ye-updater / zhu-ye-tray
2. 复制二进制、基础词典与领域包到 target/portable 暂存目录
3. 生成 zip；干净机解压后以管理员运行 scripts/install.ps1（无需 -SkipBuild，
   发行包模式不构建）

.EXAMPLE
.\scripts\package-portable.ps1

.EXAMPLE
.\scripts\package-portable.ps1 -Version 0.1.0
#>
[CmdletBinding()]
param(
    [string]$Version
)

$ErrorActionPreference = 'Stop'
$repoRoot = Split-Path -Parent $PSScriptRoot

if (-not $Version) {
    $match = Select-String -LiteralPath (Join-Path $repoRoot 'Cargo.toml') -Pattern '^version = "([^"]+)"' |
        Select-Object -First 1
    if ($null -eq $match) {
        throw '无法从根 Cargo.toml 读取 version。'
    }
    $Version = $match.Matches[0].Groups[1].Value
}

$needed = @(
    (Join-Path $repoRoot 'target\release\zhu_ye_ime.dll'),
    (Join-Path $repoRoot 'target\release\zhu-ye-settings.exe'),
    (Join-Path $repoRoot 'target\release\zhu-ye-updater.exe'),
    (Join-Path $repoRoot 'target\release\zhu-ye-tray.exe')
)
if (-not ($needed | Where-Object { Test-Path -LiteralPath $_ -PathType Leaf })) {
    Write-Host '未找到 release 产物，先执行构建...'
    Push-Location $repoRoot
    try {
        & cargo build --release -p zhu-ye-ime -p zhu-ye-settings -p zhu-ye-updater -p zhu-ye-tray
        if ($LASTEXITCODE -ne 0) {
            throw 'cargo build 失败，无法生成发行包。'
        }
    } finally {
        Pop-Location
    }
}
foreach ($artifact in $needed) {
    if (-not (Test-Path -LiteralPath $artifact -PathType Leaf)) {
        throw "release 产物缺失：$artifact"
    }
}

$portableRoot = Join-Path $repoRoot 'target\portable'
$packageName = "zhu-ye-ime-$Version-test"
$staging = Join-Path $portableRoot $packageName
$zipPath = Join-Path $portableRoot "$packageName.zip"

if (Test-Path -LiteralPath $staging) {
    Remove-Item -LiteralPath $staging -Recurse -Force
}
if (Test-Path -LiteralPath $zipPath) {
    Remove-Item -LiteralPath $zipPath -Force
}

# 公共 staging（zip 与 Inno exe 安装包共用载荷，同源同哈希，T-097）
# 子脚本 throw 会冒泡终止（父 ErrorActionPreference=Stop）；不检查 LASTEXITCODE
& (Join-Path $PSScriptRoot 'stage-package.ps1') -Version $Version -StagingDir $staging

Compress-Archive -Path (Join-Path $staging '*') -DestinationPath $zipPath -Force
$zipInfo = Get-Item -LiteralPath $zipPath
Remove-Item -LiteralPath $staging -Recurse -Force

Write-Host "发行包已生成: $zipPath（$([math]::Round($zipInfo.Length / 1MB, 2)) MB）"
Write-Host '在干净机解压后以管理员执行：.\scripts\install.ps1'
