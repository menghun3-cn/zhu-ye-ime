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
    KeyboardCategoryGuid   = '{34745C63-B2F0-4784-8B67-5E12C8701A31}'
    LanguageIdHex          = '0x00000804'
    DllName                = 'zhu-ye-ime.dll'
    DictionaryFileName     = 'dictionary.zyct'
    EnWordbookFileName     = 'en.zyen'
    # T-175 固定名壳布局：宿主永远加载 <InstallDir>\zhu-ye-ime.dll，注册表指针
    # 恒定指向它；各版本实现归档在 <InstallDir>\versions\<sha8>\ 下。
    # 固定名与上面的 DllName 同名（连字符），与项目标识符 zhu-ye-ime 一致；
    # 下划线形式（zhu_ye_ime_*.dll）是更早期的历史布局，只在清理旧残留时识别。
    StableDllName          = 'zhu-ye-ime.dll'
    VersionsDirName        = 'versions'
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
        # 已经带 .zy-del 后缀的（上一轮改名遗留）不再追加后缀，否则每轮清理
        # 都会叠一层（实测出现过 .dll.zy-del.zy-del.zy-del.zy-del）。
        if ($path -like '*.zy-del*') {
            $sweepTarget = $path
        } else {
            $sweepTarget = "$path.zy-del"
        }
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

# ---------------------------------------------------------------------------
# T-175 固定名壳部署：宿主永远加载同一个 DLL 文件
# ---------------------------------------------------------------------------
#
# 旧布局（版本化文件名 + 切注册表指针）的问题：注册表指针每次升级都变，
# 已打开的宿主仍持旧版句柄，且部署目录会累积大量版本化 DLL（实测 49 个 /
# 106 MB）。
#
# 新布局：
#   <InstallDir>\zhu-ye-ime.dll          ← 固定名，宿主唯一加载入口，指针恒定
#   <InstallDir>\versions\<sha8>\zhu-ye-ime.dll  ← 各版本实现归档
#
# 升级 = 用 ReplaceFileW 原子替换固定名文件。该 API 专为「替换正在被使用
# 的文件」设计（保留被映射的旧句柄给已打开的宿主，同时让磁盘内容变为
# 新版），因此：
#   - 已打开的宿主继续用旧版，互不干扰；
#   - 新打开的宿主立刻加载新版，注册表指针从此不再变化。

if (-not ('ZhuYe.ReplaceFile' -as [type])) {
    Add-Type -Namespace ZhuYe -Name ReplaceFile -MemberDefinition @'
[DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)]
[return: MarshalAs(UnmanagedType.Bool)]
public static extern bool ReplaceFileW(string lpReplacedFileName, string lpReplacementFileName, string lpBackupFileName, uint dwReplaceFlags, IntPtr lpExclude, IntPtr lpReserved);
'@
}

function Get-TsfStableDllPath {
    <#
    .SYNOPSIS
    固定名壳 DLL 的完整路径（宿主唯一加载入口）。
    #>
    param(
        [Parameter(Mandatory)][string]$InstallDir
    )
    return (Join-Path $InstallDir $script:TsfIdentity['StableDllName'])
}

function Get-TsfVersionsDir {
    <#
    .SYNOPSIS
    版本归档目录（各版本实现的历史留存）。
    #>
    param(
        [Parameter(Mandatory)][string]$InstallDir
    )
    return (Join-Path $InstallDir $script:TsfIdentity['VersionsDirName'])
}

function Install-TsfStableDll {
    <#
    .SYNOPSIS
    以「固定名壳 + 版本归档」方式部署 TSF DLL（T-175）。

    .DESCRIPTION
    1. 把新实现复制到 versions\<sha8>\zhu-ye-ime.dll（版本归档）；
    2. 用 ReplaceFileW 原子替换固定名 zhu-ye-ime.dll——该 API 在目标文件正被
       宿主进程映射时依然成功（保留旧句柄给已打开的宿主，磁盘内容切为新版）；
    3. 首次安装（固定名不存在）时直接复制。

    注册表指针**恒定**指向固定名，升级过程无需改注册表。

    .OUTPUTS
    返回有序字典：InstalledPath / VersionId / Replaced / ArchivePath。
    #>
    param(
        [Parameter(Mandatory)][string]$SourceDll,
        [Parameter(Mandatory)][string]$InstallDir
    )

    if (-not (Test-Path -LiteralPath $SourceDll -PathType Leaf)) {
        throw "源 DLL 不存在: $SourceDll"
    }
    $null = New-Item -ItemType Directory -Path $InstallDir -Force
    $stableDll = Get-TsfStableDllPath -InstallDir $InstallDir
    $versionId = (Get-FileHash -LiteralPath $SourceDll -Algorithm SHA256).Hash.Substring(0, 8).ToLowerInvariant()

    # 1. 版本归档：把该版本实现留存到 versions\<sha8>\。
    $versionsDir = Get-TsfVersionsDir -InstallDir $InstallDir
    $versionDir = Join-Path $versionsDir $versionId
    $null = New-Item -ItemType Directory -Path $versionDir -Force
    $archivePath = Join-Path $versionDir $script:TsfIdentity['StableDllName']
    Copy-Item -LiteralPath $SourceDll -Destination $archivePath -Force

    # 2. 替换固定名。
    $replaced = $false
    if (Test-Path -LiteralPath $stableDll -PathType Leaf) {
        # ReplaceFileW 的替换源与被替换目标不可为同一路径，故用临时文件。
        $staging = Join-Path $InstallDir (".{0}.staging" -f $script:TsfIdentity['StableDllName'])
        Copy-Item -LiteralPath $SourceDll -Destination $staging -Force
        # lpBackupFileName 必须给真实路径：实测本机传 NULL 或空串均返回
        # ERROR_PATH_NOT_FOUND(3)，给真实路径才成功（与 MoveFileEx 延迟删除
        # 的同类本机限制一致）。备份即被替换掉的旧版本实现，归档到 versions\。
        $backup = Join-Path $versionDir (".{0}.replaced" -f $script:TsfIdentity['StableDllName'])
        try {
            $ok = [ZhuYe.ReplaceFile]::ReplaceFileW($stableDll, $staging, $backup, 0, [IntPtr]::Zero, [IntPtr]::Zero)
            if (-not $ok) {
                $errorCode = [System.Runtime.InteropServices.Marshal]::GetLastWin32Error()
                throw "ReplaceFileW 失败（Win32 $errorCode）: $stableDll"
            }
            $replaced = $true
        } finally {
            Remove-Item -LiteralPath $staging -Force -ErrorAction SilentlyContinue
        }
    } else {
        Copy-Item -LiteralPath $SourceDll -Destination $stableDll -Force
    }

    return [ordered]@{
        InstalledPath = $stableDll
        VersionId     = $versionId
        Replaced      = $replaced
        ArchivePath   = $archivePath
    }
}

function Remove-TsfOldVersionDlls {
    <#
    .SYNOPSIS
    清理 T-175 之前的版本化 DLL 残留与不再被引用的旧版本归档（T-175）。

    .DESCRIPTION
    清理两类文件：
      1. 旧布局遗留的 zhu-ye-ime-<sha8>.dll / zhu_ye_ime_<sha8>.dll 等
         版本化文件——固定名壳接管后它们永不再被加载；
      2. versions\ 下除当前版本外的旧归档（保留最近若干个以供回滚）。

    被进程占用的文件走 Add-TsfDelayedCleanup（改名 .zy-del + 重启后清理）。
    #>
    param(
        [Parameter(Mandatory)][string]$InstallDir,
        [string]$KeepVersionId = '',
        [int]$KeepArchiveCount = 3
    )

    $stableDll = Get-TsfStableDllPath -InstallDir $InstallDir
    $stale = @()

    # 1. 旧布局版本化残留（固定名与 .zy-del 之外的都清理）。
    #    需要同时识别两种历史命名：连字符 `zhu-ye-ime-<hash>.dll`（T-026 起）与
    #    下划线 `zhu_ye_ime_<hash>.dll`（更早的 T-026 之前布局）。
    #    T-175 期间用过的下划线**固定名** `zhu_ye_ime.dll` 同样属于旧布局，
    #    改名后需一并清掉；新的连字符固定名 `zhu-ye-ime.dll` 必须保留。
    $legacyStableNames = @('zhu_ye_ime.dll')
    foreach ($f in @(Get-ChildItem -LiteralPath $InstallDir -File -ErrorAction SilentlyContinue)) {
        if ($f.FullName -eq $stableDll) { continue }
        if ($legacyStableNames -contains $f.Name) {
            $stale += $f.FullName
            continue
        }
        if ($f.Name -like 'zhu_ye_ime_*.dll' -or $f.Name -like 'zhu-ye-ime-*.dll' -or
            $f.Name -like 'zhu_ye_ime_*.dll.zy-del*' -or $f.Name -like 'zhu-ye-ime-*.dll.zy-del*') {
            $stale += $f.FullName
        }
    }

    # 2. 版本归档：只保留最近 $KeepArchiveCount 个（含当前版本）。
    $versionsDir = Get-TsfVersionsDir -InstallDir $InstallDir
    if (Test-Path -LiteralPath $versionsDir -PathType Container) {
        $archives = @(Get-ChildItem -LiteralPath $versionsDir -Directory -ErrorAction SilentlyContinue |
            Sort-Object LastWriteTime -Descending)
        $keep = 0
        foreach ($dir in $archives) {
            $keep++
            if ($keep -le $KeepArchiveCount -or $dir.Name -eq $KeepVersionId) { continue }
            $stale += (Join-Path $dir.FullName $script:TsfIdentity['StableDllName'])
            # 空目录一并移除。
            Remove-Item -LiteralPath $dir.FullName -Recurse -Force -ErrorAction SilentlyContinue
        }
    }

    if ($stale.Count -gt 0) {
        Write-Host "清理旧版本 DLL（$($stale.Count) 个）："
        Add-TsfDelayedCleanup -Paths $stale
    }
    return $stale
}

# 重启 ctfmon（Windows 输入法框架进程）。
#
# 背景（T-176 实测）：ctfmon 会缓存语言档的 `IconFile` 路径与已加载的 TSF DLL。
# 部署位文件名变更（T-175 固定名壳统一为 `zhu-ye-ime.dll`）或 DLL 被替换后，
# 缓存指向的旧路径已不存在，任务栏输入法指示器就取不到「竹」图标而显示空白。
# 资源本身完好（PE 内 `RT_GROUP_ICON` 在位、`ExtractAssociatedIcon` 成功），
# 只是 ctfmon 没重新读注册表。
#
# 重启代价：输入法框架瞬间重建，正在进行的输入会中断不到 1 秒；不影响用户数据。
function Restart-TsfFramework {
    $before = @(Get-Process ctfmon -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Id)
    Stop-Process -Name ctfmon -Force -ErrorAction SilentlyContinue
    Start-Sleep -Milliseconds 500
    if (-not (Get-Process ctfmon -ErrorAction SilentlyContinue)) {
        Start-Process "$env:SystemRoot\System32\ctfmon.exe" -ErrorAction SilentlyContinue
    }
    Start-Sleep -Milliseconds 800
    $after = @(Get-Process ctfmon -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Id)
    if ($after.Count -gt 0) {
        Write-Host "已重启 ctfmon（$($before -join ',') -> $($after -join ',')），输入法指示器图标将重新加载。"
        return $true
    }
    Write-Warning 'ctfmon 重启后未检测到进程；输入法指示器图标可能需要注销后恢复。'
    return $false
}
