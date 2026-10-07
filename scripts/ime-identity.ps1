#requires -Version 5.1
<#
.SYNOPSIS
竹叶输入法 TSF 身份常量与安装/卸载共享工具。

.DESCRIPTION
统一维护 TIP CLSID、语言 Profile GUID、键盘类别 GUID、默认安装目录与 DLL 名称，
为安装/卸载脚本提供创建、删除和校验 HKLM TSF 注册的辅助函数，以及
版本化 DLL 升级所需的 InProcServer32 读写与 MoveFileEx 延迟清理工具。

.NOTES
常量必须与 crates/zhu-ye-ime/src/tsf.rs 中 CLSID_ZHU_YE_TIP、PROFILE_GUID_ZHU_YE 保持一致；
DictionaryFileName / EnWordbookFileName 与 crates/zhu-ye-core/src/identity.rs 的
DICTIONARY_FILE_NAME / EN_WORDBOOK_FILE_NAME 双份维护（D-42 / T-085）。
本文件只读共享，dot-source 加载本身不含任何副作用。
#>
Set-StrictMode -Version Latest

$script:TsfIdentity = [ordered]@{
    DisplayName            = '竹叶输入法'
    TipClsid               = '{E54D6682-8650-40E7-A9EE-6FD1137849AE}'
    ProfileGuid            = '{6315FE74-92C3-439B-8CDF-FDB6E43EDAF1}'
    ProfileGuidEn          = '{EF42481A-233D-4035-A80A-7F416BE0E6CA}'
    DisplayNameEn          = '竹叶输入法（英文）'
    KeyboardCategoryGuid   = '{34745C63-B2F0-4784-8B67-5E12C8701A31}'
    LanguageIdHex          = '0x00000804'
    DllName                = 'zhu-ye-ime.dll'
    DictionaryFileName     = 'dictionary.zyct'
    EnWordbookFileName     = 'en.zyen'
}

function Get-TsfInstallDir {
    return (Join-Path ${env:ProgramFiles} 'zhu-ye-ime\tsf')
}

function Get-TsfDictionaryPath {
    return (Join-Path (Get-TsfInstallDir) $script:TsfIdentity['DictionaryFileName'])
}

function Get-TsfEnWordbookPath {
    return (Join-Path (Get-TsfInstallDir) $script:TsfIdentity['EnWordbookFileName'])
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

    # 英文态语言档案（T-112 后续批四：任务栏"未激活状态"图标）：同 TIP 树、
    # 同 0x0804 段第二个语言档案，Win+Space 可切换"竹叶中文⇄竹叶英文"，
    # 任务栏指示器随档案显示竹/英图标；引擎按激活档案装配起始模式
    # （tsf.rs start_mode_for_profile）。IconFile 指向独立 ying.ico 文件
    # （部署位 tsf\ying.ico，与 install.ps1 分发动作保持一致）。
    $profileEnPath = "$tipPath\LanguageProfile\$($tip['LanguageIdHex'])\$($tip['ProfileGuidEn'])"
    $null = [Microsoft.Win32.Registry]::LocalMachine.CreateSubKey($profileEnPath)
    Set-TsfRegistryValue -Path $profileEnPath -Name 'Description' -Value $tip['DisplayNameEn']
    Set-TsfRegistryValue -Path $profileEnPath -Name 'Display Description' -Value $tip['DisplayNameEn']
    Set-TsfRegistryValue -Path $profileEnPath -Name 'Enable' -Value 1 -Kind ([Microsoft.Win32.RegistryValueKind]::DWord)
    Set-TsfRegistryValue -Path $profileEnPath -Name 'IconFile' -Value (Join-Path (Get-TsfInstallDir) 'ying.ico')
    Set-TsfRegistryValue -Path $profileEnPath -Name 'IconIndex' -Value 0 -Kind ([Microsoft.Win32.RegistryValueKind]::DWord)

    # 设置 GUI 入口（T-115 后续）：TIP 键 EnableConfiguration=1 让系统"按键
    # 选项/键盘选项"页对该输入法显示配置入口，点击经 ITfFnConfigure::Show
    # （T-114 已实现）拉起设置窗口。
    Set-TsfRegistryValue -Path $tipPath -Name 'EnableConfiguration' -Value 1 -Kind ([Microsoft.Win32.RegistryValueKind]::DWord)

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

    # 英文态档案（T-112 后续批四）：Enable=1 且 IconFile 指向存在的 ying.ico。
    $profileEnPath = "$tipPath\LanguageProfile\$($script:TsfIdentity['LanguageIdHex'])\$($script:TsfIdentity['ProfileGuidEn'])"
    $profileEnKey = [Microsoft.Win32.Registry]::LocalMachine.OpenSubKey($profileEnPath, $false)
    if ($null -eq $profileEnKey) { return $false }
    try {
        if ($profileEnKey.GetValue('Enable', -1) -cne 1) { return $false }
        if (-not (Test-Path -LiteralPath ([string]$profileEnKey.GetValue('IconFile')))) { return $false }
    } finally {
        $profileEnKey.Dispose()
    }

    $tipKey = [Microsoft.Win32.Registry]::LocalMachine.OpenSubKey($tipPath, $false)
    if ($null -eq $tipKey) { return $false }
    try {
        if ($tipKey.GetValue('EnableConfiguration', -1) -cne 1) { return $false }
    } finally {
        $tipKey.Dispose()
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

function Get-TsfInprocServerDefault {
    <#
    .SYNOPSIS
    读取当前注册的 InProcServer32 (默认) DLL 路径；未注册时返回 $null。
    #>
    $inprocPath = "SOFTWARE\Classes\CLSID\$($script:TsfIdentity['TipClsid'])\InProcServer32"
    $key = [Microsoft.Win32.Registry]::LocalMachine.OpenSubKey($inprocPath, $false)
    if ($null -eq $key) { return $null }
    try {
        return $key.GetValue($null)
    } finally {
        $key.Dispose()
    }
}

function Set-TsfInprocServerDefault {
    <#
    .SYNOPSIS
    仅改写 InProcServer32 (默认) DLL 路径，不触碰 TSF 注册的其余键（用于升级失败回滚）。
    #>
    param(
        [Parameter(Mandatory)][string]$DllPath
    )
    Set-TsfRegistryValue -Path "SOFTWARE\Classes\CLSID\$($script:TsfIdentity['TipClsid'])\InProcServer32" -Name $null -Value $DllPath
}

Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class ZhuYeMoveFile {
    [DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static extern bool MoveFileEx(string lpExistingFileName, string lpNewFileName, uint dwFlags);
}
'@

function Add-TsfDelayedCleanup {
    <#
    .SYNOPSIS
    清理被占用、无法立即删除的旧版本 DLL（无占用时立即删除，占用时登记
    MoveFileEx 重启后迁移清理），实现无锁升级/卸载。

    .DESCRIPTION
    平台限制：在本项目目标 Windows Server 上实测，
    MoveFileEx(MOVEFILE_DELAY_UNTIL_REBOOT) 的"删除"操作（lpNewFileName 为
    NULL 或空串）返回 ERROR_PATH_NOT_FOUND，而"改名"操作可用；因此占用文件
    的延迟清理退化为改名到 "<原名>.zy-del"，由下一次安装/卸载清扫该残留
    （改名后的文件不再被任何进程映射，可立即删除）。
    文件在磁盘上不存在视为已清理；标记失败仅输出警告，不抛错。
    #>
    param(
        [Parameter(Mandatory)][string[]]$Paths
    )
    foreach ($path in $Paths) {
        if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { continue }
        try {
            Remove-Item -LiteralPath $path -Force -ErrorAction Stop
            Write-Host "  已删除: $path"
            continue
        } catch {
            # 被进程占用，改走重启后迁移清理。
        }
        $sweepTarget = "$path.zy-del"
        try {
            # 清扫上一次重启遗留的同名 .zy-del（目标不存在时重启改名才会成功）。
            Remove-Item -LiteralPath $sweepTarget -Force -ErrorAction Stop
        } catch {
        }
        if ([ZhuYeMoveFile]::MoveFileEx($path, $sweepTarget, 5)) {
            Write-Host "  已登记重启后迁移清理: $path -> $sweepTarget"
        } else {
            $errorCode = [System.Runtime.InteropServices.Marshal]::GetLastWin32Error()
            Write-Warning "延迟清理标记失败（Win32 $errorCode）: $path"
        }
    }
}
