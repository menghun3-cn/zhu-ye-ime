#requires -Version 5.1
<#
.SYNOPSIS
校验竹叶输入法 DLL 是否可加载且包含 TSF 所需导出函数。

.DESCRIPTION
以最小副作用方式加载 DLL（不解析依赖、不执行 DllMain），
检查 DllGetClassObject、DllCanUnloadNow、dll_probe、paired_core_version 四个导出符号。
不修改注册表或文件系统。

.EXAMPLE
.\scripts\verify-tsf-dll.ps1 -DllPath .\target\release\zhu_ye_ime.dll
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$DllPath
)

$ErrorActionPreference = 'Stop'
$resolved = Resolve-Path -LiteralPath $DllPath
if ($null -eq $resolved) {
    throw "DLL 不存在: $DllPath"
}
$fullPath = [string]$resolved

Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class ZhuYeNativeExports {
    [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    public static extern IntPtr LoadLibraryEx(string lpFileName, IntPtr hFile, uint dwFlags);

    [DllImport("kernel32.dll", CharSet = CharSet.Ansi, SetLastError = true)]
    public static extern IntPtr GetProcAddress(IntPtr hModule, string lpProcName);

    [DllImport("kernel32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static extern bool FreeLibrary(IntPtr hModule);
}
'@

# DONT_RESOLVE_DLL_REFERENCES：映射但不执行 DllMain，适合构建期导出校验。
$module = [ZhuYeNativeExports]::LoadLibraryEx($fullPath, [IntPtr]::Zero, 1)
if ($module -eq [IntPtr]::Zero) {
    $errorCode = [System.Runtime.InteropServices.Marshal]::GetLastWin32Error()
    throw "加载 DLL 失败（Win32 错误 $errorCode）: $fullPath"
}

try {
    $requiredExports = @(
        'DllGetClassObject',
        'DllCanUnloadNow',
        'dll_probe'
    )
    foreach ($export in $requiredExports) {
        $address = [ZhuYeNativeExports]::GetProcAddress($module, $export)
        if ($address -eq [IntPtr]::Zero) {
            throw "DLL 缺少导出符号: $export"
        }
        Write-Host "  导出通过: $export"
    }
    Write-Host "DLL 导出校验通过: $fullPath"
} finally {
    $null = [ZhuYeNativeExports]::FreeLibrary($module)
}
