# check-site-links.ps1
# Internal-link checker for site/ (github pages static site).
#
# Usage:
#   .\scripts\check-site-links.ps1
#
# Scans every *.html under site/, extracts href/src attributes, resolves
# site-internal relative references against the referring file, and verifies
# the target exists (a trailing "/" requires an index.html inside). In
# addition it validates:
#   - markdown links in llms.txt / agents.txt (resolved from the site root)
#   - <loc> URLs in sitemap.xml (path portion resolved from the site root;
#     external scheme/host is expected and skipped)
# External URLs (http/https/mailto) are listed but not fetched. Exits 1 when
# any internal link is broken or entry files reference a missing file.
#
# ASCII-only source: must run under Windows PowerShell 5.1 as well, where a
# UTF-8 file without BOM is misread when it contains non-ASCII bytes.

$ErrorActionPreference = 'Stop'

$siteRoot = Join-Path $PSScriptRoot '..\site'
if (-not (Test-Path $siteRoot)) {
    Write-Error "site root not found: $siteRoot"
    exit 2
}

$entries = @()

# 1) HTML files: href/src attributes.
$htmlFiles = Get-ChildItem -Path $siteRoot -Recurse -Filter '*.html' -File
foreach ($file in $htmlFiles) {
    $content = Get-Content -LiteralPath $file.FullName -Raw -Encoding UTF8
    $refDir = Split-Path $file.FullName -Parent
    $matches = [regex]::Matches($content, '(?:href|src)="([^"]+)"')
    foreach ($m in $matches) {
        $entries += [pscustomobject]@{ Url = $m.Groups[1].Value; RefDir = $refDir }
    }
}

# 2) Markdown entry files (llms.txt / agents.txt): [text](url) links.
foreach ($mdName in @('llms.txt', 'agents.txt')) {
    $mdPath = Join-Path $siteRoot $mdName
    if (-not (Test-Path $mdPath)) {
        Write-Host "MISSING entry file: $mdName"
        $entries += [pscustomobject]@{ Url = $mdName; RefDir = $siteRoot }
        continue
    }
    $content = Get-Content -LiteralPath $mdPath -Raw -Encoding UTF8
    $matches = [regex]::Matches($content, '\]\(([^)]+)\)')
    foreach ($m in $matches) {
        $entries += [pscustomobject]@{ Url = $m.Groups[1].Value; RefDir = $siteRoot }
    }
}

# 3) sitemap.xml: <loc> URLs.
$sitemapPath = Join-Path $siteRoot 'sitemap.xml'
if (Test-Path $sitemapPath) {
    $content = Get-Content -LiteralPath $sitemapPath -Raw -Encoding UTF8
    $matches = [regex]::Matches($content, '<loc>([^<]+)</loc>')
    foreach ($m in $matches) {
        $url = $m.Groups[1].Value
        # Strip scheme + authority + the Pages project prefix (the repo name),
        # keeping the remainder as a site-relative reference. A site root URL
        # maps to index.html.
        $pathOnly = $url -replace '^[a-z]+://[^/]+/[^/]*/?', ''
        if ($pathOnly -eq '') {
            $pathOnly = 'index.html'
        }
        $entries += [pscustomobject]@{ Url = $pathOnly; RefDir = $siteRoot }
    }
}

$unique = $entries | Sort-Object RefDir, Url -Unique
$broken = @()
$external = @()
$total = 0

foreach ($l in $unique) {
    $url = $l.Url
    if ($url -match '^(https?://|mailto:|tel:|data:|#)') {
        if ($url -match '^(https?://|mailto:|tel:)') {
            $external += $url
        }
        continue
    }
    $total++
    $clean = $url -replace '#.*$', ''
    if ($clean -eq '') { continue }

    $candidate = Join-Path $l.RefDir $clean
    $full = [System.IO.Path]::GetFullPath($candidate)

    $ok = $false
    if ($clean -match '/$') {
        $ok = Test-Path -LiteralPath (Join-Path $full 'index.html')
    } else {
        $ok = Test-Path -LiteralPath $full
    }

    if (-not $ok) {
        $broken += [pscustomobject]@{
            Url  = $url
            From = $l.RefDir
        }
    }
}

Write-Host '== site link check =='
Write-Host "html files : $($htmlFiles.Count)"
Write-Host "internal   : $total"
Write-Host "external   : $($external.Count)"
foreach ($e in ($external | Sort-Object -Unique)) {
    Write-Host "  ext  $e"
}

if ($broken.Count -eq 0) {
    Write-Host 'RESULT: PASS (no broken internal links)'
    exit 0
}

Write-Host "RESULT: FAIL ($($broken.Count) broken link(s))"
foreach ($b in $broken) {
    Write-Host "  BROKEN  $($b.Url)  (referenced under $($b.From))"
}
exit 1
