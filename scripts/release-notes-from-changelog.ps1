# 从 CHANGELOG.md 提取指定版本段的发布说明（发布 CI 用，T-095 修订）
#
# 用法： .\scripts\release-notes-from-changelog.ps1 -Version <X.Y.Z> [-NotesFile <路径>]
# 行为： 定位 `## [<Version>] - <date>` 标题，取其正文（含 `### 新增` 等小节，
#        不含后续版本段）；找不到版本段时抛错（CI fail-fast）。
#        指定 -NotesFile 时以 UTF-8 无 BOM 写入（供 `gh release create --notes-file`）。
# 兼容： Windows PowerShell 5.1+；脚本文件使用 UTF-8 BOM
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$Version,
    [string]$NotesFile
)
$ErrorActionPreference = 'Stop'

$root = Split-Path -Parent $PSScriptRoot
$chgPath = Join-Path $root 'CHANGELOG.md'
if (-not (Test-Path -LiteralPath $chgPath -PathType Leaf)) {
    Write-Error "缺少 CHANGELOG.md：$chgPath"
    exit 1
}
$chg = Get-Content -LiteralPath $chgPath -Raw -Encoding UTF8

# 标题行 `## [0.1.2-alpha] - 2026-10-04`（独立匹配标题行，避免组合正则的行尾陷阱）
$hdr = [regex]::Match($chg, "(?m)^## \[$([regex]::Escape($Version))\] - .*$")
if (-not $hdr.Success) {
    Write-Error "CHANGELOG 中未找到版本段：$Version"
    exit 1
}
$tail = $chg.Substring($hdr.Index + $hdr.Length)
$nxt = [regex]::Match($tail, "(?m)^## ")
$len = if ($nxt.Success) { $nxt.Index } else { $tail.Length }
$body = $tail.Substring(0, $len).Trim()

if ($NotesFile) {
    $outDir = Split-Path -Parent $NotesFile
    if ($outDir -and -not (Test-Path -LiteralPath $outDir)) {
        New-Item -ItemType Directory -Force -Path $outDir | Out-Null
    }
    $UTF8NoBOM = New-Object System.Text.UTF8Encoding($false)
    [System.IO.File]::WriteAllText(([System.IO.Path]::GetFullPath($NotesFile)), $body, $UTF8NoBOM)
    Write-Host ("发布说明 {0} 字符 → {1}" -f $body.Length, $NotesFile)
} else {
    Write-Output $body
}
# 显式成功退出码：避免调用方读取 $LASTEXITCODE 时沿用上个 native 命令的残留值
# （发布 CI 曾因此误判「生成发布说明失败」，见 T-097 正式发布 v0.1.2 run 37290578197）
exit 0
