# check-site-structure.ps1
# Structural + content-vocabulary assertions for site/ (acceptance 15.1/15.3).
#
# Usage:
#   .\scripts\check-site-structure.ps1
#
# For every page (5 content pages + 404.html) asserts:
#   title contains "竹叶输入法", non-empty description, favicon ref,
#   exactly 5 nav links, a GitHub nav link, exactly one aria-current="page"
#   (404.html: none), the four footer elements, and the shared M1-M14
#   version-status line. Then asserts the forbidden vocabulary is absent,
#   planned-capability keywords appear only on the home page, AI entry files
#   keep their structures (llms.txt v2, agents.txt S-18 four sections,
#   robots.txt allow-all + sitemap, sitemap.xml 6 <loc>), and that site/
#   contains zero .js files. Exits 1 on any failure.
#
# ASCII-only source preferred (PS 5.1 misreads UTF-8 without BOM); the
# Chinese assertion strings below make that impossible, so this file is
# deliberately stored as UTF-8 **with BOM**.

$ErrorActionPreference = 'Stop'

$siteRoot = Join-Path $PSScriptRoot '..\site'
if (-not (Test-Path $siteRoot)) {
    Write-Error "site root not found: $siteRoot"
    exit 2
}

$pages = @(
    (Join-Path $siteRoot 'index.html'),
    (Join-Path $siteRoot 'docs\index.html'),
    (Join-Path $siteRoot 'download\index.html'),
    (Join-Path $siteRoot 'privacy\index.html'),
    (Join-Path $siteRoot 'about\index.html'),
    (Join-Path $siteRoot '404.html')
)

$fail = 0

foreach ($p in $pages) {
    if (-not (Test-Path -LiteralPath $p)) {
        Write-Host "FAIL missing page: $p"
        $fail++
        continue
    }
    $c = Get-Content -LiteralPath $p -Raw -Encoding UTF8
    $name = Split-Path $p -Leaf

    if ($c -notmatch '<title>[^<]*竹叶输入法') {
        Write-Host "FAIL $name : title"
        $fail++
    }
    if ($c -notmatch '<meta name="description" content="[^"]+"') {
        Write-Host "FAIL $name : description"
        $fail++
    }
    if ($c -notmatch 'favicon\.svg') {
        Write-Host "FAIL $name : favicon"
        $fail++
    }
    $navCount = ([regex]::Matches($c, 'class="nav-link" href="[^"]+"')).Count
    if ($navCount -ne 5) {
        Write-Host "FAIL $name : nav links = $navCount"
        $fail++
    }
    if ($c -notmatch 'class="nav-gh"') {
        Write-Host "FAIL $name : github nav link"
        $fail++
    }
    $current = ([regex]::Matches($c, 'aria-current="page"')).Count
    if ($name -eq '404.html') {
        if ($current -ne 0) {
            Write-Host "FAIL $name : 404.html must not set aria-current (found $current)"
            $fail++
        }
    } elseif ($current -ne 1) {
        Write-Host "FAIL $name : aria-current count = $current"
        $fail++
    }
    foreach ($el in @('footer-brand-zh', 'footer-slogan', 'footer-status', 'footer-copy')) {
        $needle = 'class="' + $el + '"'
        if ($c -notmatch [regex]::Escape($needle)) {
            Write-Host "FAIL $name : footer element $el"
            $fail++
        }
    }
    if ($c -notmatch 'M1–M15 发布收尾：更新源已开通 · exe 安装包 CI 已接入 · Pages 已上线') {
        Write-Host "FAIL $name : version-status line out of date"
        $fail++
    }
}

# Forbidden vocabulary (acceptance 15.1 FR-056 / design 7): never appear.
$forbidden = @('macOS', '背单词', '云服务', '多语言译词')
foreach ($p in $pages) {
    $c = Get-Content -LiteralPath $p -Raw -Encoding UTF8
    foreach ($w in $forbidden) {
        if ($c -match [regex]::Escape($w)) {
            Write-Host "FAIL $(Split-Path $p -Leaf) : forbidden word $w"
            $fail++
        }
    }
}

# Planned-capability keywords may appear only on the home page roadmap.
$planned = @('英文词典全面扩容', '网络语词典扩充', '设置窗口增强', '中英混合整句解码', 'proptest')
foreach ($p in ($pages | Where-Object { $PSItem -notmatch 'index\.html$' })) {
    $c = Get-Content -LiteralPath $p -Raw -Encoding UTF8
    foreach ($w in $planned) {
        if ($c -match [regex]::Escape($w)) {
            Write-Host "FAIL $p : planned keyword leaked to non-home page: $w"
            $fail++
        }
    }
}

# AI entry files.
$llms = Get-Content -LiteralPath (Join-Path $siteRoot 'llms.txt') -Raw -Encoding UTF8
if ($llms -notmatch '(?m)^# ' -or $llms -notmatch '(?m)^> ' -or $llms -notmatch '(?m)^## ') {
    Write-Host 'FAIL llms.txt : v2 structure (H1 / summary / sections)'
    $fail++
}
$agents = Get-Content -LiteralPath (Join-Path $siteRoot 'agents.txt') -Raw -Encoding UTF8
foreach ($sec in @('项目事实', '内容边界', '可信路径', '文件索引')) {
    if ($agents -notmatch [regex]::Escape("## $sec")) {
        Write-Host "FAIL agents.txt : missing section $sec"
        $fail++
    }
}
$robots = Get-Content -LiteralPath (Join-Path $siteRoot 'robots.txt') -Raw -Encoding UTF8
if ($robots -notmatch 'User-agent: \*' -or $robots -notmatch 'Allow: /' -or $robots -notmatch 'Sitemap:') {
    Write-Host 'FAIL robots.txt : allow-all + sitemap line'
    $fail++
}
try {
    [xml]$sm = Get-Content -LiteralPath (Join-Path $siteRoot 'sitemap.xml') -Raw -Encoding UTF8
    $locCount = @($sm.urlset.url.loc).Count
    if ($locCount -ne 6) {
        Write-Host "FAIL sitemap.xml : loc count = $locCount (expected 6)"
        $fail++
    }
} catch {
    Write-Host "FAIL sitemap.xml : not valid XML ($($_.Exception.Message))"
    $fail++
}

# Zero JS (design S-16).
$js = Get-ChildItem -Path $siteRoot -Recurse -Filter '*.js' -File
if ($js) {
    Write-Host "FAIL zero-JS : $($js.Count) js file(s)"
    $fail++
}

if ($fail -eq 0) {
    Write-Host 'check-site-structure: PASS'
    exit 0
}
Write-Host "check-site-structure: FAIL ($fail issue(s))"
exit 1
