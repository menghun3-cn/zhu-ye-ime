#requires -Version 5.1
<#
.SYNOPSIS
构建英文词表 en.zyen（T-085）并做性能验收。

.DESCRIPTION
1. 核对 ECDICT 缓存（data/cache/ecdict-full.csv）SHA-256 与 data/pins/ecdict.json 锁定一致，
   缺失或漂移时提示先运行 .\scripts\fetch-sources.ps1；
2. cargo run --release en-build 消费 ECDICT + FrequencyWords + CC-CEDICT 英文侧 +
   补丁（en-capitals.tsv / en-exclude.tsv）产出 data/artifacts/en.zyen；
3. cargo run --release en-bench 实测：加载+校验 ≤50ms、10 万次前缀查询命中中位数 ≤0.5ms（500us）、
   产物 ≤20MB（含大写原形后预算约 19.5MB）。

.EXAMPLE
.\scripts\build-en-wordbook.ps1

.EXAMPLE
.\scripts\build-en-wordbook.ps1 -SkipBench
#>
[CmdletBinding()]
param(
    [switch]$SkipBench
)

$ErrorActionPreference = 'Stop'
$repoRoot = Split-Path -Parent $PSScriptRoot
Push-Location $repoRoot
try {
    $pin = Get-Content -Raw -Encoding UTF8 (Join-Path $repoRoot 'data\pins\ecdict.json') | ConvertFrom-Json
    $cache = Join-Path $repoRoot (Join-Path 'data\cache' $pin.cache_file)
    if (-not (Test-Path -LiteralPath $cache -PathType Leaf)) {
        throw "缺少 ECDICT 缓存 $cache —— 请先运行 .\scripts\fetch-sources.ps1"
    }
    $actual = (Get-FileHash -LiteralPath $cache -Algorithm SHA256).Hash
    if ($actual -ne $pin.sha256) {
        throw "ECDICT 缓存哈希漂移：期望 $($pin.sha256)，实际 $actual（源内容变化或缓存被改，人工审查 pin 后重锁）"
    }
    Write-Host "ECDICT 缓存锁定一致：$($pin.sha256.Substring(0, 12))…"

    Write-Host '构建英文词表 en.zyen（release）...'
    & cargo run -q --release -p zhu-ye-dict -- en-build
    if ($LASTEXITCODE -ne 0) { throw 'en-build 失败' }

    if ($SkipBench) { return }

    Write-Host '英文词表性能实测（加载 + 20 万次前缀查询）...'
    $output = & cargo run -q --release -p zhu-ye-dict -- en-bench 'data\artifacts\en.zyen' 200000 2>&1 | Out-String
    if ($LASTEXITCODE -ne 0) { throw 'en-bench 失败' }
    $output | Write-Host

    $metrics = @{}
    foreach ($entry in ($output -split "`r?`n")) {
        if ($entry -match '^指标:\s*(.+)$') {
            $kv = $Matches[1] -split '=' 
            if ($kv.Count -eq 2 -and $kv[0] -match '^[A-Za-z_]+$') {
                $metrics[$kv[0]] = [double]$kv[1]
            }
        }
    }
    if (-not $metrics.ContainsKey('en_load_ms') -or -not $metrics.ContainsKey('en_query_us')) {
        throw 'en-bench 输出缺少 en_load_ms/en_query_us 指标行，无法验收'
    }

    $failed = @()
    if ($metrics['en_load_ms'] -gt 50) { $failed += "en_load_ms $($metrics['en_load_ms'])ms > 50ms" }
    if ($metrics['en_query_us'] -gt 500) { $failed += "en_query_us $($metrics['en_query_us'])us > 500us" }
    $size = (Get-Item (Join-Path $repoRoot 'data\artifacts\en.zyen')).Length
    if ($size -gt 20MB) { $failed += "产物 $size 字节 > 20MB" }
    '{0,-14} {1,10}  {2}' -f 'en_load_ms', "$($metrics['en_load_ms']) ms", $(if ($metrics['en_load_ms'] -le 50) { '通过' } else { '超限' }) | Write-Host
    '{0,-14} {1,10}  {2}' -f 'en_query_us', "$($metrics['en_query_us']) us", $(if ($metrics['en_query_us'] -le 500) { '通过' } else { '超限' }) | Write-Host
    '{0,-14} {1,10}  {2}' -f '产物大小', "$([math]::Round($size / 1MB, 2)) MB", $(if ($size -le 20MB) { '通过' } else { '超限' }) | Write-Host

    if ($failed.Count -gt 0) { throw "英文词表性能验收未通过：$($failed -join '；')" }
    Write-Host '英文词表性能验收通过（加载 ≤50ms、查询 ≤0.5ms、产物 ≤20MB）'
}
finally {
    Pop-Location
}
