#requires -Version 5.1
<#
.SYNOPSIS
从发行包安装竹叶输入法：TSF 服务（版本化 DLL）+ 设置窗口 + 领域包 + 快捷方式。

.DESCRIPTION
发行包布局（由 package-portable.ps1 生成，见 docs/安装与使用.md）：

    zhu-ye-ime-<version>/
      bin/zhu_ye_ime.dll             TSF 服务 DLL（版本化复制到安装目录）
      bin/zhu-ye-settings.exe        设置窗口
      bin/zhu-ye-updater.exe         词典更新器（唯一联网组件）
      bin/zhu-ye-tray.exe            托盘常驻进程（批六，中英状态图标）
      bin/dictionary.zyct            基础词典
      bin/en.zyen                    英文词表（T-085，英文前缀候选）
      packs/it.zyct med.zyct slang.zyct   预置领域包（D-46，可离线验收）
      scripts/                       安装/卸载/校验脚本
      docs/                          数据来源与许可证

安装行为：
1. 从发行包 bin\ 读取文件，不再依赖源码树与 cargo（T-078）
2. 版本化复制 zhu-ye-ime.dll 到安装目录并校验导出（沿用原事务与失败回滚）
3. 复制基础词典与英文词表（en.zyen，T-085）
4. 安装 zhu-ye-settings.exe / zhu-ye-updater.exe / zhu-ye-tray.exe 到 Program Files\zhu-ye-ime\bin，
   并注册 HKCU 自启"竹叶输入法托盘"（批六，常驻托盘）
5. 预置三个领域包到 %APPDATA%\zhu-ye-ime\packs\（不覆盖用户已有包以外的动作：
   同名覆盖，保证幂等）
6. 注册 HKLM TSF TIP/Category/LanguageProfile/CLSID 树并校验（失败回滚，沿用原逻辑）
7. 创建开始菜单快捷方式（D-26 唤起入口之一）
8. 延迟清理旧版本 DLL

重复执行安全；需要管理员权限（-SkipRegistration 演练可豁免）。

.EXAMPLE
.\scripts\install.ps1

.EXAMPLE
.\scripts\install.ps1 -PackageRoot C:\zhu-ye-ime-0.1.0 -InstallDir "C:\Program Files\zhu-ye-ime\tsf"

.EXAMPLE
.\scripts\install.ps1 -SkipRegistration -InstallDir C:\tmp\install-test -AppDataRoot C:\tmp\appdata-test
#>
[CmdletBinding()]
param(
    [string]$InstallDir,
    [string]$AppDataRoot,
    [string]$PackageRoot,
    [string]$DictionaryPath,
    [switch]$SkipBuild,
    [switch]$SkipRegistration
)

$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'ime-identity.ps1')

function Test-Admin {
    $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = [Security.Principal.WindowsPrincipal]::new($identity)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

function Set-TsfRegistrationRollback {
    <#
    .SYNOPSIS
    注册校验失败时的回滚：恢复到上一版本 DLL 路径；无上一版本则清空注册。
    #>
    param(
        [AllowNull()][string]$PreviousDll
    )
    if ($PreviousDll -and (Test-Path -LiteralPath $PreviousDll -PathType Leaf)) {
        Set-TsfInprocServerDefault -DllPath $PreviousDll
        $profilePath = "SOFTWARE\Microsoft\CTF\TIP\$($TsfIdentity['TipClsid'])\LanguageProfile\$($TsfIdentity['LanguageIdHex'])\$($TsfIdentity['ProfileGuid'])"
        Set-TsfRegistryValue -Path $profilePath -Name 'IconFile' -Value $PreviousDll
        Write-Host "已回滚 InProcServer32 到上一版本: $PreviousDll"
    } else {
        Remove-TsfRegistration
        Write-Host '注册校验失败且无上一版本可回滚，已清空 TSF 注册。'
    }
}

# ---- 0. 路径与权限 ----
if (-not $SkipRegistration -and -not (Test-Admin)) {
    throw '安装 TSF 服务需要管理员权限，请以管理员身份重新运行（-SkipRegistration 演练模式除外）。'
}

if (-not $PackageRoot) {
    $PackageRoot = Split-Path -Parent $PSScriptRoot
}
$PackageRoot = [System.IO.Path]::GetFullPath($PackageRoot)

if (-not $InstallDir) {
    $InstallDir = Get-TsfInstallDir
}
$InstallDir = [System.IO.Path]::GetFullPath($InstallDir)
$appRoot = Split-Path -Parent $InstallDir    # ...\zhu-ye-ime（DLL 的 tsf 子目录的上级）
$exeDir = Join-Path $appRoot 'bin'

if (-not $AppDataRoot) {
    $AppDataRoot = $env:APPDATA
    if (-not $AppDataRoot) {
        throw '未设置 APPDATA，无法定位数据目录。'
    }
}
$dataRoot = (Join-Path ([System.IO.Path]::GetFullPath($AppDataRoot)) 'zhu-ye-ime')
$packsDataDir = Join-Path $dataRoot 'packs'

if ($SkipBuild) {
    Write-Host '（发行包模式不使用源码树构建；-SkipBuild 已无作用，忽略）'
}
Write-Host "发行包根目录: $PackageRoot"

# ---- 1. 校验发行包内容 ----
$binDir = Join-Path $PackageRoot 'bin'
$packsDir = Join-Path $PackageRoot 'packs'
$sourceDll = Join-Path $binDir 'zhu_ye_ime.dll'
$settingsExe = Join-Path $binDir 'zhu-ye-settings.exe'
$updaterExe = Join-Path $binDir 'zhu-ye-updater.exe'
$trayExe = Join-Path $binDir 'zhu-ye-tray.exe'
$requireExe = @($settingsExe, $updaterExe, $trayExe)
foreach ($exe in $requireExe) {
    if (-not (Test-Path -LiteralPath $exe -PathType Leaf)) {
        throw "发行包缺少可执行文件：$exe（请先用 package-portable.ps1 生成发行包，或检查 -PackageRoot）"
    }
}
if (-not (Test-Path -LiteralPath $sourceDll -PathType Leaf)) {
    throw "发行包缺少 TSF DLL：$sourceDll"
}
if (-not (Test-Path -LiteralPath $packsDir -PathType Container)) {
    throw "发行包缺少领域包目录：$packsDir"
}
foreach ($pack in @('it.zyct', 'med.zyct', 'slang.zyct')) {
    if (-not (Test-Path -LiteralPath (Join-Path $packsDir $pack) -PathType Leaf)) {
        throw "发行包缺少预置领域包：$pack"
    }
}

# ---- 2. 复制 TSF DLL（版本化）与基础词典（沿用原事务与回滚） ----
$dllHash8 = (Get-FileHash -LiteralPath $sourceDll -Algorithm SHA256).Hash.Substring(0, 8).ToLowerInvariant()
$newDllName = "zhu-ye-ime-$dllHash8.dll"
$targetDll = Join-Path $InstallDir $newDllName
$previousDll = Get-TsfInprocServerDefault
if ($previousDll) {
    Write-Host "当前注册版本: $previousDll"
}

$null = New-Item -ItemType Directory -Path $InstallDir -Force
Copy-Item -LiteralPath $sourceDll -Destination $targetDll -Force

try {
    & (Join-Path $PSScriptRoot 'verify-tsf-dll.ps1') -DllPath $targetDll
    if (-not $?) { throw 'DLL 导出校验未通过。' }
} catch {
    if ($targetDll -ne $previousDll) {
        Remove-Item -LiteralPath $targetDll -Force -ErrorAction SilentlyContinue
    }
    throw "DLL 导出校验失败，安装中止（未通过校验的副本已删除，注册表未改动）: $($_.Exception.Message)"
}

if (-not $DictionaryPath) {
    $DictionaryPath = Join-Path $binDir 'dictionary.zyct'
}
if (-not (Test-Path -LiteralPath $DictionaryPath -PathType Leaf)) {
    throw "基础词典不存在：$DictionaryPath（发行包应含 bin\dictionary.zyct，或显式传 -DictionaryPath）"
}
$targetDictionary = Join-Path $InstallDir $TsfIdentity['DictionaryFileName']
if ([System.IO.Path]::GetFullPath($DictionaryPath) -eq [System.IO.Path]::GetFullPath($targetDictionary)) {
    Write-Host "词典已就位（源与目标相同），跳过复制: $targetDictionary"
} else {
    try {
        Copy-Item -LiteralPath $DictionaryPath -Destination $targetDictionary -Force
    } catch {
        if ($targetDll -ne $previousDll) {
            Remove-Item -LiteralPath $targetDll -Force -ErrorAction SilentlyContinue
        }
        throw "词典复制失败，安装中止（未提交的 DLL 副本已删除，注册表未改动）: $($_.Exception.Message)"
    }
}
$legacyDictionary = Join-Path $InstallDir 'seed.zyct'
if (Test-Path -LiteralPath $legacyDictionary -PathType Leaf) {
    Remove-Item -LiteralPath $legacyDictionary -Force
}

# ---- 2.5 复制英文词表 en.zyen（T-085：英文前缀候选；缺失即跳过，引擎回退内嵌静态表） ----
$enWordbook = Join-Path $binDir 'en.zyen'
if (Test-Path -LiteralPath $enWordbook -PathType Leaf) {
    $targetEnWordbook = Join-Path $InstallDir 'en.zyen'
    try {
        Copy-Item -LiteralPath $enWordbook -Destination $targetEnWordbook -Force
        Write-Host "已安装英文词表: $targetEnWordbook"
    } catch {
        if ($targetDll -ne $previousDll) {
            Remove-Item -LiteralPath $targetDll -Force -ErrorAction SilentlyContinue
        }
        throw "英文词表复制失败，安装中止（未提交的 DLL 副本已删除，注册表未改动）: $($_.Exception.Message)"
    }
} else {
    Write-Host '发行包未含 en.zyen（英文前缀候选回退内嵌静态表，不阻断安装）。'
}

# ---- 3. 安装设置窗口、更新器与托盘 ----
$null = New-Item -ItemType Directory -Path $exeDir -Force
foreach ($exe in $requireExe) {
    Copy-Item -LiteralPath $exe -Destination $exeDir -Force
    Write-Host "已安装: $(Join-Path $exeDir (Split-Path -Leaf $exe))"
}

# ---- 3.5 托盘自启（HKCU Run；批六：常驻托盘依赖登录会话，装到当前用户自启） ----
$runPath = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Run'
$runName = '竹叶输入法托盘'
$runValue = '"{0}"' -f (Join-Path $exeDir 'zhu-ye-tray.exe')
if (-not (Test-Path -LiteralPath $runPath)) { $null = New-Item -Path $runPath -Force }
Set-ItemProperty -LiteralPath $runPath -Name $runName -Value $runValue -Force
Write-Host "已注册托盘自启（HKCU Run）: $runName = $runValue"

# ---- 4. 预置领域包（同名覆盖，幂等） ----
$null = New-Item -ItemType Directory -Path $packsDataDir -Force
foreach ($pack in @('it.zyct', 'med.zyct', 'slang.zyct')) {
    $source = Join-Path $packsDir $pack
    $target = Join-Path $packsDataDir $pack
    Copy-Item -LiteralPath $source -Destination $target -Force
}
Write-Host "已预置三个领域包到: $packsDataDir"

if ($SkipRegistration) {
    Write-Host '演练模式（-SkipRegistration）：跳过 HKLM 注册与快捷方式，其余安装步骤已完成。'
    Write-Host "演练安装目录: $InstallDir"
    Write-Host "演练数据目录: $dataRoot"
    return
}

# ---- 5. 注册表切换到新版本（New-TsfRegistration 内部先清旧再重建） ----
try {
    New-TsfRegistration -DllPath $targetDll
} catch {
    Set-TsfRegistrationRollback -PreviousDll $previousDll
    throw "TSF 注册失败，已回滚；请检查注册表权限: $($_.Exception.Message)"
}
if (-not (Test-TsfRegistration -DllPath $targetDll)) {
    Set-TsfRegistrationRollback -PreviousDll $previousDll
    throw 'TSF 注册结果校验失败，已回滚；请检查注册表权限或使用 uninstall.ps1 清理。'
}

# ---- 6. 开始菜单快捷方式（D-26 唤起入口） ----
$startMenu = Join-Path ([Environment]::GetFolderPath('Programs')) '竹叶输入法设置.lnk'
$shell = New-Object -ComObject WScript.Shell
$shortcut = $shell.CreateShortcut($startMenu)
$shortcut.TargetPath = (Join-Path $exeDir 'zhu-ye-settings.exe')
$shortcut.WorkingDirectory = $exeDir
$shortcut.Description = '竹叶输入法设置：工具箱、常用设置、关于与更新'
$shortcut.Save()
Write-Host "已创建快捷方式: $startMenu"

# ---- 7. 延迟清理旧版本 DLL（含历史版本、固定名与 .zy-del 迁移残留） ----
$staleDlls = @(Get-ChildItem -LiteralPath $InstallDir -File -ErrorAction SilentlyContinue |
    Where-Object { $_.Name -like 'zhu-ye-ime*.dll*' -and $_.FullName -ne $targetDll } |
    ForEach-Object { $_.FullName })
if ($staleDlls.Count -gt 0) {
    Write-Host "清理旧版本 DLL（$($staleDlls.Count) 个）："
    Add-TsfDelayedCleanup -Paths $staleDlls
}

Write-Host "安装完成: $targetDll"
if ($previousDll -and $previousDll -ne $targetDll) {
    Write-Host "已从 $previousDll 升级；旧版本 DLL 将在系统重启后自动清理。"
}
Write-Host '输入法已在系统中注册；可在 设置 -> 时间和语言 -> 语言和区域 中切换到竹叶输入法。'
Write-Host "设置窗口: $(Join-Path $exeDir 'zhu-ye-settings.exe')"
