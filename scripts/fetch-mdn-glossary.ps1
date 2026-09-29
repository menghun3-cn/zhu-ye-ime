# MDN zh-cn 术语表正文抓取（M6-P，T-045）
# 用法： .\scripts\fetch-mdn-glossary.ps1 [-Out data/cache/mdn-glossary-zh-pages.txt]
# 输入：data/pins/mdn-glossary-zh.snapshot.json 的 slug 列表 + data/pins/mdn-glossary-zh-pages.json 的锁定 commit
# 输出：单一打包文件，按 slug 字典序拼接，每页前加 `### <slug>` 分隔行；缺页（该 commit 下 404）记为 `### <slug> MISSING`
# 输出确定：同 commit 同 slug 列表必得同字节（UTF-8 无 BOM、LF 换行），哈希由 fetch-sources.ps1 锁定
# 兼容：Windows PowerShell 5.1+；脚本文件使用 UTF-8 BOM

param(
    [string]$Out = 'data/cache/mdn-glossary-zh-pages.txt',
    [string]$PinPath = 'data/pins/mdn-glossary-zh-pages.json',
    [string]$SnapshotPath = 'data/pins/mdn-glossary-zh.snapshot.json'
)

$ErrorActionPreference = 'Stop'
try { [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor [System.Net.SecurityProtocolType]::Tls12 } catch { }

$root = (Get-Location).Path
$pin = Get-Content -Path (Join-Path $root $PinPath) -Raw -Encoding UTF8 | ConvertFrom-Json
$snapshot = Get-Content -Path (Join-Path $root $SnapshotPath) -Raw -Encoding UTF8 | ConvertFrom-Json
if (-not $pin.commit) { throw "pin 缺 commit 字段：$PinPath" }

$slugs = @($snapshot.slugs | Sort-Object -CaseSensitive)
$utf8 = New-Object System.Text.UTF8Encoding($false)
$client = New-Object System.Net.WebClient
$builder = New-Object System.Text.StringBuilder
$missing = 0
$index = 0

foreach ($slug in $slugs) {
    $index++
    $url = $pin.url_template.Replace('{commit}', $pin.commit).Replace('{slug}', $slug)
    $text = $null
    $attempt = 0
    while ($attempt -lt 3) {
        try {
            $text = $utf8.GetString($client.DownloadData($url))
            break
        } catch {
            $status = $null
            if ($_.Exception.InnerException -and $_.Exception.InnerException.Response) {
                $status = [int]$_.Exception.InnerException.Response.StatusCode
            }
            if ($status -eq 404) { break }
            $attempt++
            if ($attempt -ge 3) { throw "抓取失败（重试 3 次）：$url —— $($_.Exception.Message)" }
            Start-Sleep -Seconds 2
        }
    }
    if ($null -eq $text) {
        $missing++
        [void]$builder.Append("### $slug MISSING`n")
    } else {
        $body = $text.Replace("`r`n", "`n").TrimEnd("`n")
        [void]$builder.Append("### $slug`n").Append($body).Append("`n")
    }
    if ($index % 50 -eq 0) { Write-Host ("  已抓取 {0}/{1}" -f $index, $slugs.Count) }
}

$target = Join-Path $root $Out
New-Item -ItemType Directory -Force -Path (Split-Path $target) | Out-Null
[System.IO.File]::WriteAllText($target, $builder.ToString(), $utf8)
$hash = (Get-FileHash -Path $target -Algorithm SHA256).Hash
Write-Host ("MDN 术语表打包完成：{0} 个 slug，缺页 {1}，{2:N0} 字节，SHA-256 {3}" -f $slugs.Count, $missing, (Get-Item $target).Length, $hash)
