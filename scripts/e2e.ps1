#requires -Version 5.1
<#
.SYNOPSIS
运行主机侧端到端回归检查器 host-e2e。

.DESCRIPTION
先确保 data/artifacts/seed.zyct 存在（缺失时用 zhu-ye-dict build 生成），
构建 host-e2e 检查器并针对种子词典执行核心输入闭环回归；若存在
data/artifacts/real.zyct，再执行真实词典 smoke。默认 debug 构建，
加 -Release 使用 release 构建正式验收；-SkipBuild 跳过 Rust 构建。

.EXAMPLE
.\scripts\e2e.ps1

.EXAMPLE
.\scripts\e2e.ps1 -Release
#>
[CmdletBinding()]
param(
    [switch]$Release,
    [switch]$SkipBuild,
    [string]$RealDictionaryPath
)

$ErrorActionPreference = 'Stop'
$repoRoot = Split-Path -Parent $PSScriptRoot
$seedPath = Join-Path $repoRoot 'data\artifacts\seed.zyct'
$configDir = if ($Release) { 'release' } else { 'debug' }
$binPath = Join-Path $repoRoot "target\$configDir\host-e2e.exe"
if ([string]::IsNullOrWhiteSpace($RealDictionaryPath)) {
    $RealDictionaryPath = Join-Path $repoRoot 'data\artifacts\real.zyct'
} else {
    $RealDictionaryPath = [System.IO.Path]::GetFullPath($RealDictionaryPath)
}

Push-Location $repoRoot
try {
    if (-not (Test-Path -LiteralPath $seedPath -PathType Leaf)) {
        Write-Host '未找到种子词典，先生成 v2 词典产物...'
        & cargo run -q -p zhu-ye-dict -- build
        if ($LASTEXITCODE -ne 0) {
            throw 'zhu-ye-dict build 失败，无法执行主机侧回归'
        }
    }

    if (-not $SkipBuild) {
        $buildArgs = @('build', '-q', '-p', 'zhu-ye-ime', '--bin', 'host-e2e')
        if ($Release) {
            $buildArgs += '--release'
        }
        & cargo @buildArgs
        if ($LASTEXITCODE -ne 0) {
            throw 'host-e2e 构建失败'
        }
    } elseif (-not (Test-Path -LiteralPath $binPath -PathType Leaf)) {
        throw "未找到 $binPath，请去掉 -SkipBuild 或先构建"
    }

    Write-Host "主机端到端回归（种子词典：$seedPath）"
    & $binPath $seedPath
    if ($LASTEXITCODE -ne 0) {
        throw '种子词典核心输入闭环回归未通过'
    }

    if (Test-Path -LiteralPath $RealDictionaryPath -PathType Leaf) {
        Write-Host "真实词典 smoke（$RealDictionaryPath）"
        & $binPath --real-smoke $RealDictionaryPath
        if ($LASTEXITCODE -ne 0) {
            throw '真实词典 smoke 未通过'
        }
        Write-Host "M7 输入体验优化断言组（$RealDictionaryPath）"
        & $binPath --m7 $RealDictionaryPath
        if ($LASTEXITCODE -ne 0) {
            throw 'M7 输入体验优化断言组未通过'
        }
    } else {
        Write-Host '未发现真实词典 data\artifacts\real.zyct，跳过真实数据 smoke 与 M7 断言（seed 回归已通过）'
    }
} finally {
    Pop-Location
}

Write-Host '端到端回归验收通过'
