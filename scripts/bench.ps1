#requires -Version 5.1
<#
.SYNOPSIS
运行 zhu-ye-cli bench 并校验性能指标阈值。

.DESCRIPTION
先确保 data/artifacts/seed.zyct 存在（缺失时用 zhu-ye-dict build 生成），
再执行 bench，解析“指标:”行并按每操作微秒阈值验收（默认 1000us）。
可加 -Release 使用 release 构建做正式验收，用 ZYDT_BENCH_MAX_US 覆盖阈值。

.EXAMPLE
.\scripts\bench.ps1

.EXAMPLE
.\scripts\bench.ps1 -Release -MaxUsPerOp 200
#>
[CmdletBinding()]
param(
    [switch]$Release,
    [double]$MaxUsPerOp = 1000
)

$ErrorActionPreference = 'Stop'
$repoRoot = Split-Path -Parent $PSScriptRoot
$dictPath = Join-Path $repoRoot 'data\artifacts\seed.zyct'

Push-Location $repoRoot
try {
    if (-not (Test-Path -LiteralPath $dictPath -PathType Leaf)) {
        Write-Host '未找到词典产物，先生成 v2 种子词典...'
        & cargo run -q -p zhu-ye-dict -- build
        if ($LASTEXITCODE -ne 0) {
            throw 'zhu-ye-dict build 失败，无法执行基准验收'
        }
    }

    $run = @('run', '-q') + $(if ($Release) { @('--release') } else { @() }) + @('-p', 'zhu-ye-cli', '--', 'bench')
    $output = & cargo @run 2>&1 | Out-String
    if ($LASTEXITCODE -ne 0) {
        throw 'zhu-ye-cli bench 执行失败'
    }
    $output | Write-Host
} finally {
    Pop-Location
}

$line = ($output -split "`r?`n" | Where-Object { $_ -like '指标:*' } | Select-Object -First 1)
if (-not $line) {
    throw 'bench 输出缺少“指标:”行，无法验收'
}

$failed = @()
foreach ($entry in ($line -replace '^指标:\s*', '' -split '\s+')) {
    if ($entry -notmatch '^([A-Za-z_]+)=([0-9.]+)$') {
        continue
    }
    $name = $Matches[1]
    $value = [double]$Matches[2]
    $ok = $value -le $MaxUsPerOp
    '{0,-12} {1,10:0.000} us/次  {2}' -f $name, $value, $(if ($ok) { '通过' } else { '超限' }) | Write-Host
    if (-not $ok) {
        $failed += $name
    }
}

if ($failed.Count -gt 0) {
    throw "性能验收未通过: $($failed -join ', ')，阈值 $MaxUsPerOp us/次"
}
Write-Host "性能验收通过：bench 全部指标 <= $MaxUsPerOp us/次"
