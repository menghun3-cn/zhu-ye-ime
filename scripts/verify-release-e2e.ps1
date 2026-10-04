<#
.SYNOPSIS
发布资产端到端验证（T-094 / P-01 发布分发闭环）。

对 `assemble-release.ps1` 产出的发布目录跑完整更新链路（不走真实网络，
用 curl 的 file:// 协议验证"拉取 manifest → 内置公钥验签 → 下载分发包 →
内容哈希/大小校验 → staging + rename 原子落地 → 幂等"）以及两个负面用例
（篡改落地包自愈、篡改 manifest 被拒）。

前置：
  1. `assemble-release.ps1` 已产出发布目录（含签名 manifest）；
  2. 已用注入发布公钥的更新器构建产物（ZHU_YE_RELEASE_PUBLIC_KEY 构建）；
  3. 本机存在 `target\release\zhu-ye-updater.exe`（未注入公钥的产物会在
     "签名校验"步骤报"未内置发布公钥"并拒绝——那也是通过：说明未配公钥
     时更新器不降级）。

用法：
  .\scripts\verify-release-e2e.ps1 [AssetsDir] [-KeepAppData]

  AssetsDir  发布资产目录（默认 target\release-assets\<当前工作区版本>）
  -KeepAppData  保留临时 APPDATA（第二次运行可验证增量：已装包跳过下载）
#>
[CmdletBinding()]
param(
    [string]$AssetsDir,
    [switch]$KeepAppData
)
$ErrorActionPreference = 'Stop'

$repoRoot = Split-Path -Parent $PSScriptRoot
$updater = Join-Path $repoRoot 'target\release\zhu-ye-updater.exe'
if (-not (Test-Path -LiteralPath $updater -PathType Leaf)) {
    throw "未找到更新器产物：$updater（先执行 assemble-release.ps1）"
}

if (-not $AssetsDir) {
    $match = Select-String -LiteralPath (Join-Path $repoRoot 'Cargo.toml') -Pattern '^version = "([^"]+)"' |
        Select-Object -First 1
    $version = $match.Matches[0].Groups[1].Value
    $AssetsDir = Join-Path $repoRoot "target\release-assets\$version\packs"
}
$packsDir = if (Test-Path -LiteralPath (Join-Path $AssetsDir 'packs') -PathType Container) {
    Join-Path $AssetsDir 'packs'
} else {
    $AssetsDir
}
$manifestPath = Join-Path $packsDir 'manifest.json'
if (-not (Test-Path -LiteralPath $manifestPath -PathType Leaf)) {
    throw "发布资产缺失：$manifestPath"
}

# 隔离 APPDATA（干净环境；-KeepAppData 保留下一次增量验证）。
$appData = Join-Path $repoRoot 'target\tmp-e2e-appdata'
if (-not $KeepAppData -and (Test-Path -LiteralPath $appData)) {
    Remove-Item -LiteralPath $appData -Recurse -Force
}
$null = New-Item -ItemType Directory -Path (Join-Path $appData 'ai-zhu-ye-ime') -Force
$env:APPDATA = (Resolve-Path $appData).Path
$configPath = Join-Path $appData 'ai-zhu-ye-ime\config.json'
Set-Content -LiteralPath $configPath -Value '{"online_update": true}' -Encoding utf8

$manifestUrl = 'file:///' + $packsDir.Replace('\', '/') + '/manifest.json'
$env:ZHU_YE_MANIFEST_URL = $manifestUrl

function Assert-Step([string]$name, [string[]]$expectLines, [int]$exit) {
    Write-Host "[OK] $name" -ForegroundColor Green
}

$failures = @()

Write-Host '== 1/4 apply：下载并落地全部可分发包 =='
$applyOut = & $updater apply 2>&1
$applyExit = $LASTEXITCODE
$applyText = $applyOut -join "`n"
if ($applyExit -eq 0 -and $applyText -match '已应用') {
    Assert-Step 'apply 成功'
} elseif ($applyExit -eq 0 -and $applyText -match '均已是最新') {
    Assert-Step 'apply 幂等（已是最新）'
} else {
    $failures += "apply 失败（exit=$applyExit）：`n$applyText"
}

Write-Host '== 2/4 落地包与 manifest 哈希/大小一致 =='
$manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
$installedOk = $true
foreach ($pack in $manifest.packs) {
    $file = Join-Path $appData "ai-zhu-ye-ime\packs\$($pack.file)"
    if (-not (Test-Path -LiteralPath $file -PathType Leaf)) { $installedOk = $false; continue }
    $len = (Get-Item -LiteralPath $file).Length
    $hash = (Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($len -ne [int64]$pack.size -or $hash -ne $pack.sha256) { $installedOk = $false }
}
if ($installedOk) {
    Assert-Step "落地 $($manifest.packs.Count) 包与 manifest 一致"
} else {
    $failures += '落地包哈希/大小与 manifest 不一致'
}

Write-Host '== 3/4 篡改落地包：应被检测并自愈 =='
$first = $manifest.packs[0]
$packFile = Join-Path $appData "ai-zhu-ye-ime\packs\$($first.file)"
$bytes = [System.IO.File]::ReadAllBytes($packFile)
$bytes[0] = $bytes[0] -bxor 0xFF
[System.IO.File]::WriteAllBytes($packFile, $bytes)
$tamperOut = & $updater apply 2>&1
$tamperText = $tamperOut -join "`n"
$hashAfter = (Get-FileHash -LiteralPath $packFile -Algorithm SHA256).Hash.ToLowerInvariant()
if ($hashAfter -eq $first.sha256) {
    Assert-Step '篡改包被重新拉取恢复'
} else {
    $failures += "篡改包未被恢复（哈希仍不一致）：`n$tamperText"
}

Write-Host '== 4/4 篡改 manifest：内置公钥验签拒绝 =='
$origText = Get-Content -LiteralPath $manifestPath -Raw
$tampered = $origText -replace '"published_at": "[^"]+"', '"published_at": "1999-01-01"'
Set-Content -LiteralPath $manifestPath -Value $tampered -Encoding utf8
$sigOut = & $updater check 2>&1
$sigExit = $LASTEXITCODE
Set-Content -LiteralPath $manifestPath -Value $origText -Encoding utf8
if ($sigExit -ne 0) {
    Assert-Step '篡改 manifest 被拒绝'
} else {
    $failures += "篡改 manifest 未被拒绝：`n$($sigOut -join "`n")"
}

Write-Host ''
if ($failures.Count -eq 0) {
    Write-Host '端到端验证通过：签名链 + 内容哈希 + 原子落地 + 篡改拒绝全部符合预期。' -ForegroundColor Green
    exit 0
} else {
    Write-Host '端到端验证失败：' -ForegroundColor Red
    $failures | ForEach-Object { Write-Host "  - $_" -ForegroundColor Red }
    exit 1
}
