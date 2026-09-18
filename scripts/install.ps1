#requires -Version 5.1
<#
.SYNOPSIS
构建并安装竹叶输入法 TSF 服务。

.DESCRIPTION
1. 以 release 模式构建 zhu-ye-ime（可跳过）
2. 校验 DLL 导出
3. 复制 DLL 到 Program Files 安装目录
4. 注册 HKLM TSF TIP/Category/LanguageProfile/CLSID 键
5. 校验注册结果

重复执行安全；每次安装会先清理旧注册再写入，保证无重复路径残留。
需要管理员权限。

.EXAMPLE
.\scripts\install.ps1

.EXAMPLE
.\scripts\install.ps1 -SkipBuild
#>
[CmdletBinding()]
param(
    [string]$InstallDir,
    [switch]$SkipBuild
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
    throw '安装 TSF 服务需要管理员权限，请以管理员身份重新运行。'
}

if (-not $InstallDir) {
    $InstallDir = Get-TsfInstallDir
}
$InstallDir = [System.IO.Path]::GetFullPath($InstallDir)
$targetDll = Join-Path $InstallDir $TsfIdentity['DllName']

if (-not $SkipBuild) {
    Write-Host '开始构建 release DLL...'
    Push-Location $repoRoot
    try {
        & cargo build -p zhu-ye-ime --release
        if ($LASTEXITCODE -ne 0) {
            throw 'cargo build 失败，安装中止。'
        }
    } finally {
        Pop-Location
    }
}

$sourceDll = Join-Path $repoRoot 'target\release\zhu_ye_ime.dll'
if (-not (Test-Path -LiteralPath $sourceDll -PathType Leaf)) {
    throw "未找到构建产物，请先构建或移除 -SkipBuild: $sourceDll"
}
$sourceDll = (Resolve-Path -LiteralPath $sourceDll).Path

$null = New-Item -ItemType Directory -Path $InstallDir -Force
if (Test-Path -LiteralPath $targetDll -PathType Leaf) {
    try {
        Remove-Item -LiteralPath $targetDll -Force
    } catch {
        throw "安装目录中的 DLL 正被进程占用，请关闭输入法相关应用后重试: $targetDll"
    }
}
Copy-Item -LiteralPath $sourceDll -Destination $targetDll -Force

& (Join-Path $PSScriptRoot 'verify-tsf-dll.ps1') -DllPath $targetDll
if ($LASTEXITCODE -ne 0) {
    throw 'DLL 导出校验失败，安装中止。'
}

New-TsfRegistration -DllPath $targetDll
if (-not (Test-TsfRegistration -DllPath $targetDll)) {
    throw 'TSF 注册结果校验失败，请检查注册表权限或使用 uninstall.ps1 回滚。'
}

Write-Host "安装完成: $targetDll"
Write-Host '输入法已在系统中注册；可在 设置 -> 时间和语言 -> 语言和区域 中切换到竹叶输入法。'
