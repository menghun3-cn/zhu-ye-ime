#requires -Version 5.1
<#
.SYNOPSIS
卸载竹叶输入法 TSF 服务并清理注册表与 DLL。

.DESCRIPTION
1. 删除 HKLM TIP 与 CLSID 注册树（不存在时静默跳过）
2. 校验注册残留
3. 删除安装目录中的 DLL；目录为空时一并移除
4. 输出完成状态

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
$targetDll = Join-Path $InstallDir $TsfIdentity['DllName']
$targetDictionary = Join-Path $InstallDir $TsfIdentity['DictionaryFileName']

Remove-TsfRegistration
$dummyPath = Join-Path (Get-TsfInstallDir) $TsfIdentity['DllName']
if (Test-TsfRegistration -DllPath $dummyPath) {
    throw '注册表清理失败，仍有 TSF 或 CLSID 残留。'
}

if (Test-Path -LiteralPath $targetDll -PathType Leaf) {
    try {
        Remove-Item -LiteralPath $targetDll -Force
    } catch {
        throw "DLL 正被进程占用，请关闭输入法相关应用后重试: $targetDll"
    }
}

foreach ($dictionaryFile in @($targetDictionary, (Join-Path $InstallDir 'seed.zyct'))) {
    if (Test-Path -LiteralPath $dictionaryFile -PathType Leaf) {
        try {
            Remove-Item -LiteralPath $dictionaryFile -Force
        } catch {
            throw "词典文件正被占用，请关闭输入法相关应用后重试: $dictionaryFile"
        }
    }
}

if (Test-Path -LiteralPath $InstallDir -PathType Container) {
    $remaining = @(Get-ChildItem -LiteralPath $InstallDir -Force -ErrorAction SilentlyContinue)
    if ($remaining.Count -eq 0) {
        Remove-Item -LiteralPath $InstallDir -Force
    }
}

Write-Host '卸载完成：TSF 注册、安装 DLL 与注册目录均已清理。'
