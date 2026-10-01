#requires -Version 5.1
<#
.SYNOPSIS
校验竹叶输入法 DLL 是否可加载且包含 TSF 所需导出函数。

.DESCRIPTION
以最小副作用方式加载 DLL（不解析依赖、不执行 DllMain），
检查 DllGetClassObject、DllCanUnloadNow、dll_probe 三个导出符号。
不修改注册表或文件系统。

注意：`paired_core_version` 是 crate 内的普通 pub fn（无 #[no_mangle]），
不是 DLL 导出，不在此校验范围内。

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

# 先做 PE 头静态校验：防止把非 PE 文件（文本/损坏文件）当作 DLL 注册。
# 实测 LoadLibraryEx(DONT_RESOLVE_DLL_REFERENCES) 对部分无效文件不报错，
# 因此不能只依赖加载结果。
$bytes = [System.IO.File]::ReadAllBytes($fullPath)
if ($bytes.Length -lt 0x40 -or $bytes[0] -ne 0x4D -or $bytes[1] -ne 0x5A) {
    throw "不是有效的 PE 文件（缺少 MZ 头）: $fullPath"
}
$peOffset = [System.BitConverter]::ToInt32($bytes, 0x3C)
if ($peOffset -lt 0 -or $peOffset + 4 -gt $bytes.Length -or
    $bytes[$peOffset] -ne 0x50 -or $bytes[$peOffset + 1] -ne 0x45 -or
    $bytes[$peOffset + 2] -ne 0 -or $bytes[$peOffset + 3] -ne 0) {
    throw "不是有效的 PE 文件（缺少 PE 签名）: $fullPath"
}
Write-Host "PE 头校验通过: $fullPath"

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
