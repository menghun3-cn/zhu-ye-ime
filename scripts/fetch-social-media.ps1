# 网络语数据源获取脚本（T-087，D-52）：下载 social-media-chinese-words 并合并为单文件
# 用法： .\scripts\fetch-social-media.ps1 [-Force]
#   -Force     已存在且哈希一致也强制重下（用于刷新源内容后重锁）
# 产出：
#   data/cache/social-media-chinese-words.txt  7 个分类 txt 合并（行格式「词 词频」，
#                                              每分类前插 "# CATEGORY: <名>" 行，UTF-8 无 BOM）
#   data/cache/social-license.txt              LICENSE（MIT，随上游保留）
#   data/cache/social-readme.txt               README.md（来源记录）
# 锁：data/pins/social-media-zh.json 的 sha256/size 必须一致，否则失败并保留旧缓存；
#     缓存位于 data/cache（gitignored），不清洗不进发行物
# 兼容：Windows PowerShell 5.1+；脚本文件使用 UTF-8 BOM

param(
    [switch]$Force
)

$ErrorActionPreference = 'Stop'
try { [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor [System.Net.SecurityProtocolType]::Tls12 } catch { }

$root = (Get-Location).Path
$pinPath = Join-Path $root 'data/pins/social-media-zh.json'
$cacheDir = Join-Path $root 'data/cache'
$outFile = Join-Path $cacheDir 'social-media-chinese-words.txt'
$outLicense = Join-Path $cacheDir 'social-license.txt'
$outReadme = Join-Path $cacheDir 'social-readme.txt'
$base = 'https://raw.githubusercontent.com/jilelab/social-media-chinese-words/main/'
$files = @('动物', '美食', '生活', '时尚', '数码', '音乐', '影视', '娱乐')

if (-not (Test-Path $pinPath)) { throw "缺少 pin：$pinPath（先运行 fetch-sources.ps1 -WritePins 或按现有 pin 手工补）" }
$pin = Get-Content $pinPath -Raw -Encoding UTF8 | ConvertFrom-Json

function Get-Sha256([string]$Path) { (Get-FileHash -Path $Path -Algorithm SHA256).Hash }

# 1) 逐分类下载到临时目录
$tmpDir = Join-Path $cacheDir '.social-tmp'
New-Item -ItemType Directory -Force -Path $tmpDir | Out-Null
$parts = @()
foreach ($name in $files) {
    $part = Join-Path $tmpDir ($name + '.txt')
    $parts += $part
    $url = $base + [uri]::EscapeDataString($name + '.txt')
    if ((Test-Path $part) -and -not $Force) { continue }
    $attempt = 0
    while ($attempt -lt 3) {
        try {
            Write-Host ("  下载 {0}.txt" -f $name)
            Invoke-WebRequest -Uri $url -OutFile $part -UseBasicParsing -TimeoutSec 300
            break
        } catch {
            $attempt++
            if ($attempt -ge 3) { throw "下载失败：$url —— $($_.Exception.Message)" }
            Write-Host ("    第 {0} 次失败，2 秒后重试：{1}" -f $attempt, $_.Exception.Message)
            Start-Sleep -Seconds 2
        }
    }
}

# 2) 合并（UTF-8 无 BOM；每分类前插 CATEGORY 注释；跳过空行）
$sb = New-Object System.Text.StringBuilder
foreach ($name in $files) {
    $part = Join-Path $tmpDir ($name + '.txt')
    [void]$sb.AppendLine(('# CATEGORY: ' + $name))
    foreach ($line in [System.IO.File]::ReadAllLines($part, [System.Text.Encoding]::UTF8)) {
        if ($line.Trim().Length -gt 0) { [void]$sb.AppendLine($line) }
    }
}
$merged = Join-Path $cacheDir '.social-merged.tmp'
[System.IO.File]::WriteAllText($merged, $sb.ToString(), (New-Object System.Text.UTF8Encoding $false))

# 3) 校验哈希与大小（失败即中止并保留旧缓存）
$h = Get-Sha256 $merged
$size = (Get-Item $merged).Length
$expected = [string]$pin.sha256
if ($h -ne $expected -and $h -ne $expected.ToLowerInvariant()) {
    Remove-Item $merged -Force -ErrorAction SilentlyContinue
    throw "合并文件哈希漂移：期望 $expected，实际 $h（上游内容变化，人工审查 pin 后再锁）"
}
if ($size -ne [int64]$pin.size) {
    Remove-Item $merged -Force -ErrorAction SilentlyContinue
    throw "合并文件大小漂移：期望 $($pin.size)，实际 $size（人工审查 pin）"
}

# 4) 落盘 + 附 LICENSE/README（哈希一并记录，保留来源与许可义务）
Move-Item $merged $outFile -Force
Invoke-WebRequest -Uri ($base + 'LICENSE') -OutFile $outLicense -UseBasicParsing -TimeoutSec 120
Invoke-WebRequest -Uri ($base + 'README.md') -OutFile $outReadme -UseBasicParsing -TimeoutSec 120
Remove-Item $tmpDir -Recurse -Force -ErrorAction SilentlyContinue

Write-Host ("social-media-chinese-words 就绪：{0:N0} 字节（SHA-256 {1}）" -f $size, $h)
Write-Host '下一步：zhu-ye-dict social-clean（清洗出高频子集 → data/slang/social-words.tsv）→ build-slang'
