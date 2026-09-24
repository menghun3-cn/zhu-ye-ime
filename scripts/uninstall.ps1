#requires -Version 5.1
<#
.SYNOPSIS
卸载竹叶输入法 TSF 服务并清理注册表与全部版本化 DLL。

.DESCRIPTION
1. 删除 HKLM TIP 与 CLSID 注册树（不存在时静默跳过）
2. 校验注册残留
3. 删除注册指向的 DLL 与安装目录中全部 zhu-ye-ime*.dll（含历史版本与固定名）；
   被进程占用的文件改为 MoveFileEx 重启后延迟清理，不再中止卸载
4. 删除安装目录中的词典（占用时同样延迟清理）；目录为空时一并移除
5. 输出完成状态

重复执行安全；需要管理员权限。

.EXAMPLE
.\scripts\uninstall.ps1

.EXAMPLE
.\scripts\uninstall.ps1 -InstallDir C:\Program Files\ai-zhu-ye-ime\tsf
#>
[CmdletBinding()]
param(
    [string]$InstallDir
)

$ErrorActionPreference = 'Stop'
$repoRoot = Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'ime-identity.ps1')

function Test-Admin {
    $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = [Security.Principal.WindowsPrincipal]::new($identity)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

if (-not (Test-Admin)) {
    throw '卸载 TSF 服务需要管理员权限，请以管理员身份重新运行。'
}

if (-not $InstallDir) {
    $InstallDir = Get-TsfInstallDir
}
$InstallDir = [System.IO.Path]::GetFullPath($InstallDir)
$targetDictionary = Join-Path $InstallDir $TsfIdentity['DictionaryFileName']

$activeDll = Get-TsfInprocServerDefault
if ($activeDll) {
    Write-Host "卸载前注册指向: $activeDll"
}

Remove-TsfRegistration
$dummyPath = Join-Path (Get-TsfInstallDir) $TsfIdentity['DllName']
if (Test-TsfRegistration -DllPath $dummyPath) {
    throw '注册表清理失败，仍有 TSF 或 CLSID 残留。'
}

# 收集要删除的 DLL：注册指向的 + 安装目录中全部 zhu-ye-ime*.dll（历史版本与固定名）
# 及 .zy-del 迁移残留（改名后不再被映射，可直接删除）。
$dllPaths = @()
if ($activeDll -and (Test-Path -LiteralPath $activeDll -PathType Leaf)) {
    $dllPaths += $activeDll
}
$dllPaths += @(Get-ChildItem -LiteralPath $InstallDir -File -ErrorAction SilentlyContinue |
    Where-Object { $_.Name -like 'zhu-ye-ime*.dll*' } |
    ForEach-Object { $_.FullName })

foreach ($dll in ($dllPaths | Select-Object -Unique)) {
    try {
        Remove-Item -LiteralPath $dll -Force -ErrorAction Stop
        Write-Host "  已删除: $dll"
    } catch {
        Write-Warning "DLL 正被进程占用，已改为重启后延迟清理: $dll"
        Add-TsfDelayedCleanup -Paths $dll
    }
}

foreach ($dictionaryFile in @($targetDictionary, (Join-Path $InstallDir 'seed.zyct'))) {
    if (Test-Path -LiteralPath $dictionaryFile -PathType Leaf) {
        try {
            Remove-Item -LiteralPath $dictionaryFile -Force -ErrorAction Stop
        } catch {
            Write-Warning "词典文件正被占用，已改为重启后延迟清理: $dictionaryFile"
            Add-TsfDelayedCleanup -Paths $dictionaryFile
        }
    }
}

if (Test-Path -LiteralPath $InstallDir -PathType Container) {
    $remaining = @(Get-ChildItem -LiteralPath $InstallDir -Force -ErrorAction SilentlyContinue)
    if ($remaining.Count -eq 0) {
        Remove-Item -LiteralPath $InstallDir -Force
    }
}

Write-Host '卸载完成：TSF 注册与全部版本化 DLL、词典均已清理。'
