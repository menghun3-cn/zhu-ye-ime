#requires -Version 5.1
<#
.SYNOPSIS
竹叶输入法 TSF 身份常量与注册表工具。

.DESCRIPTION
统一维护 TIP CLSID、语言 Profile GUID、键盘类别 GUID、默认安装目录与 DLL 名称，
并为安装/卸载脚本提供创建、删除和校验 HKLM TSF 注册的辅助函数。

.NOTES
常量必须与 crates/zhu-ye-ime/src/tsf.rs 中 CLSID_ZHU_YE_TIP、PROFILE_GUID_ZHU_YE 保持一致。
本文件只读共享，不含任何写注册表副作用。
#>
Set-StrictMode -Version Latest

$script:TsfIdentity = [ordered]@{
    DisplayName            = '竹叶输入法'
    TipClsid               = '{E54D6682-8650-40E7-A9EE-6FD1137849AE}'
    ProfileGuid            = '{6315FE74-92C3-439B-8CDF-FDB6E43EDAF1}'
    KeyboardCategoryGuid   = '{34745C63-B2F0-4784-8B67-5E12C8701A31}'
    LanguageIdHex          = '0x00000804'
    DllName                = 'zhu-ye-ime.dll'
}

function Get-TsfInstallDir {
    return (Join-Path ${env:ProgramFiles} 'ai-zhu-ye-ime\tsf')
}

function Get-TsfTipRegistryPath {
    return "SOFTWARE\Microsoft\CTF\TIP\$($script:TsfIdentity['TipClsid'])"
}

function Get-TsfClsidRegistryPath {
    return "SOFTWARE\Classes\CLSID\$($script:TsfIdentity['TipClsid'])"
}

function Set-TsfRegistryValue {
    param(
        [Parameter(Mandatory)][string]$Path,
        [AllowNull()][string]$Name,
        [Parameter(Mandatory)][object]$Value,
        [Microsoft.Win32.RegistryValueKind]$Kind = [Microsoft.Win32.RegistryValueKind]::String
    )

    $key = [Microsoft.Win32.Registry]::LocalMachine.CreateSubKey($Path)
    if ($null -eq $key) {
        throw "无法创建或打开注册表键: HKLM\$Path"
    }
    try {
        $key.SetValue($Name, $Value, $Kind)
    } finally {
        $key.Dispose()
    }
}

function New-TsfRegistration {
    param([Parameter(Mandatory)][string]$DllPath)

    $tipPath = Get-TsfTipRegistryPath
    $clsidPath = Get-TsfClsidRegistryPath
    $tip = $script:TsfIdentity
    $categoryPath = "$tipPath\Category\Category\$($tip['KeyboardCategoryGuid'])\$($tip['TipClsid'])"
    $itemPath = "$tipPath\Category\Item\$($tip['TipClsid'])\$($tip['KeyboardCategoryGuid'])"
    $profilePath = "$tipPath\LanguageProfile\$($tip['LanguageIdHex'])\$($tip['ProfileGuid'])"
    $inprocPath = "$clsidPath\InProcServer32"

    # 先清理旧注册再重建，保证重复安装幂等且不残留历史路径。
    Remove-TsfRegistration

    foreach ($path in @($tipPath, $categoryPath, $itemPath, $profilePath, $clsidPath, $inprocPath)) {
        $null = [Microsoft.Win32.Registry]::LocalMachine.CreateSubKey($path)
    }

    Set-TsfRegistryValue -Path $itemPath -Name 'Description' -Value $tip['DisplayName']

    Set-TsfRegistryValue -Path $profilePath -Name 'Description' -Value $tip['DisplayName']
    Set-TsfRegistryValue -Path $profilePath -Name 'Display Description' -Value $tip['DisplayName']
    Set-TsfRegistryValue -Path $profilePath -Name 'Enable' -Value 1 -Kind ([Microsoft.Win32.RegistryValueKind]::DWord)
    Set-TsfRegistryValue -Path $profilePath -Name 'IconFile' -Value $DllPath
    Set-TsfRegistryValue -Path $profilePath -Name 'IconIndex' -Value 0 -Kind ([Microsoft.Win32.RegistryValueKind]::DWord)

    Set-TsfRegistryValue -Path $clsidPath -Name $null -Value $tip['DisplayName']
    Set-TsfRegistryValue -Path $inprocPath -Name $null -Value $DllPath
    Set-TsfRegistryValue -Path $inprocPath -Name 'ThreadingModel' -Value 'Apartment'
}

function Remove-TsfRegistration {
    foreach ($path in @((Get-TsfTipRegistryPath), (Get-TsfClsidRegistryPath))) {
        $existing = [Microsoft.Win32.Registry]::LocalMachine.OpenSubKey($path, $false)
        if ($null -ne $existing) {
            $existing.Dispose()
            [Microsoft.Win32.Registry]::LocalMachine.DeleteSubKeyTree($path)
        }
    }
}

function Test-TsfRegistration {
    param([Parameter(Mandatory)][string]$DllPath)

    $tipPath = Get-TsfTipRegistryPath
    $clsidPath = Get-TsfClsidRegistryPath
    $profilePath = "$tipPath\LanguageProfile\$($script:TsfIdentity['LanguageIdHex'])\$($script:TsfIdentity['ProfileGuid'])"
    $inprocPath = "$clsidPath\InProcServer32"

    $profileKey = [Microsoft.Win32.Registry]::LocalMachine.OpenSubKey($profilePath, $false)
    if ($null -eq $profileKey) { return $false }
    try {
        if ($profileKey.GetValue('Enable', -1) -cne 1) { return $false }
    } finally {
        $profileKey.Dispose()
    }

    $inprocKey = [Microsoft.Win32.Registry]::LocalMachine.OpenSubKey($inprocPath, $false)
    if ($null -eq $inprocKey) { return $false }
    try {
        $defaultValue = $inprocKey.GetValue($null)
        $threading = $inprocKey.GetValue('ThreadingModel')
        return ($defaultValue -eq $DllPath -and $threading -eq 'Apartment')
    } finally {
        $inprocKey.Dispose()
    }
}
