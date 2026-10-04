# CC-CEDICT 解压（发布 CI 用，T-095）
#
# fetch-sources.ps1 仅下载并哈希锁定压缩包 data/raw/cedict.ts.gz；构建链
# （build-base / build-pack it|med / build-slang）直接读解压明文
# data/raw/cedict_ts.u8。明文是压缩包的确定性派生（gz 已锁定 = 可信），
# 本脚本在明文缺失时幂等解压，已存在则跳过（保留既有本地开发锚）。
#
# 用法： .\scripts\unpack-cedict.ps1 [-Root <仓库根>]
# 兼容： Windows PowerShell 5.1+；脚本文件使用 UTF-8 BOM
[CmdletBinding()]
param(
    [string]$Root
)
$ErrorActionPreference = 'Stop'

if (-not $Root) { $Root = Split-Path -Parent $PSScriptRoot }
$Root = [System.IO.Path]::GetFullPath($Root)
$gz = Join-Path $Root 'data\raw\cedict.ts.gz'
$out = Join-Path $Root 'data\raw\cedict_ts.u8'

if (-not (Test-Path -LiteralPath $gz -PathType Leaf)) {
    throw "缺少压缩包：$gz（先运行 .\scripts\fetch-sources.ps1）"
}
if (Test-Path -LiteralPath $out -PathType Leaf) {
    Write-Host "明文已存在，跳过解压（保留现有锚）：$out"
    exit 0
}

Write-Host "解压 $gz → $out ..."
Add-Type -AssemblyName System.IO.Compression.FileSystem
$gzStream = [System.IO.Compression.GZipStream]::new(
    [System.IO.File]::OpenRead($gz),
    [System.IO.Compression.CompressionMode]::Decompress)
try {
    $dir = Split-Path -Parent $out
    if ($dir -and -not (Test-Path -LiteralPath $dir)) {
        New-Item -ItemType Directory -Force -Path $dir | Out-Null
    }
    $fs = [System.IO.File]::Create($out)
    try {
        $gzStream.CopyTo($fs)
    } finally {
        $fs.Dispose()
    }
} finally {
    $gzStream.Dispose()
}
$size = (Get-Item -LiteralPath $out).Length
$hash = (Get-FileHash -LiteralPath $out -Algorithm SHA256).Hash
Write-Host ("明文就绪：{0:N0} 字节 / SHA-256 {1}" -f $size, $hash)
