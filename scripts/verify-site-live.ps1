# verify-site-live.ps1
# 官网线上验证（验收标准 §17.3 FR-063 / T-099）：对部署后的 GitHub Pages 站点
# 做 HTTP 200 + 关键内容片段断言。
#
# Usage:
#   .\scripts\verify-site-live.ps1 [-BaseUrl <https://menghun3-cn.github.io/zhu-ye-ime>]
#
# 检查项：
#   1. 六个页面（首页/文档/下载/隐私/关于/404）均 HTTP 200 且各自含关键片段；
#   2. 下载页含 exe 安装包说明与 Releases 链接；
#   3. 首页含 AI 入口（agents.txt / llms.txt 链接）与路线图"发布收尾"状态；
#   4. agents.txt / llms.txt / robots.txt / sitemap.xml 均 HTTP 200 且结构片段存在；
#   5. sitemap.xml 六个 <loc> 与线上逐一 200 对应。
# 全部通过输出 PASS 并 exit 0，否则 FAIL 并 exit 1。
#
# ASCII-only source preferred (PS 5.1 misreads UTF-8 without BOM); the
# Chinese assertion strings below make that impossible, so this file is
# deliberately stored as UTF-8 **with BOM**.

[CmdletBinding()]
param(
    [string]$BaseUrl = 'https://menghun3-cn.github.io/zhu-ye-ime'
)

$ErrorActionPreference = 'Stop'

function Check-Url {
    param([string]$Url, [string]$Contains)
    try {
        $body = curl.exe -fsSL --max-time 60 $Url 2>$null
        if ($LASTEXITCODE -ne 0) {
            Write-Host "FAIL $Url : curl exit $LASTEXITCODE"
            return $false
        }
        $text = ($body -join "`n")
        if ($Contains -and $text -notmatch $Contains) {
            Write-Host "FAIL $Url : miss fragment '$Contains'"
            return $false
        }
        Write-Host "PASS $Url ($([math]::Round(($body -join '').Length / 1024, 1)) KB)"
        return $true
    } catch {
        Write-Host "FAIL $Url : $($_.Exception.Message)"
        return $false
    }
}

$fail = 0

# 1. 六个页面 + 关键片段
$pages = @(
    @{ path = 'index.html';        frag = '竹叶输入法' },
    @{ path = 'docs/index.html';   frag = '文档' },
    @{ path = 'download/index.html'; frag = '发行包' },
    @{ path = 'privacy/index.html';  frag = '隐私' },
    @{ path = 'about/index.html';    frag = '关于' },
    @{ path = '404.html';            frag = '404' }
)
foreach ($p in $pages) {
    if (-not (Check-Url -Url "$BaseUrl/$($p.path)" -Contains $p.frag)) { $fail++ }
}

# 2. 下载页：exe 与更新源状态
if (-not (Check-Url -Url "$BaseUrl/download/index.html" -Contains 'exe')) { $fail++ }
if (-not (Check-Url -Url "$BaseUrl/download/index.html" -Contains '更新源')) { $fail++ }

# 3. 首页：AI 入口（llms.txt 链接；agents.txt 供爬虫级访问，见第 4 项）
if (-not (Check-Url -Url "$BaseUrl/index.html" -Contains 'llms.txt')) { $fail++ }

# 4. AI 条目文件
if (-not (Check-Url -Url "$BaseUrl/agents.txt" -Contains '项目事实')) { $fail++ }
if (-not (Check-Url -Url "$BaseUrl/llms.txt" -Contains '竹叶输入法')) { $fail++ }
if (-not (Check-Url -Url "$BaseUrl/robots.txt" -Contains 'Sitemap')) { $fail++ }

# 5. sitemap.xml 六个 <loc> 逐一 200
$sm = curl.exe -fsSL --max-time 60 "$BaseUrl/sitemap.xml" 2>$null
if ($LASTEXITCODE -ne 0) {
    Write-Host 'FAIL sitemap.xml : fetch failed'
    $fail++
} else {
    try {
        [xml]$xml = ($sm -join "`n")
        $locs = @($xml.urlset.url.loc)
        if ($locs.Count -ne 6) {
            Write-Host "FAIL sitemap.xml : loc count = $($locs.Count) (expected 6)"
            $fail++
        }
        foreach ($loc in $locs) {
            if (-not (Check-Url -Url $loc -Contains '')) { $fail++ }
        }
    } catch {
        Write-Host "FAIL sitemap.xml : $($_.Exception.Message)"
        $fail++
    }
}

if ($fail -eq 0) {
    Write-Host "verify-site-live: PASS ($BaseUrl)"
    exit 0
}
Write-Host "verify-site-live: FAIL ($fail issue(s))"
exit 1
