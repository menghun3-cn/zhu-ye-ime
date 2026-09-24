#requires -Version 5.1
<#
.SYNOPSIS
构建并安装竹叶输入法 TSF 服务（版本化 DLL 无锁升级机制）。

.DESCRIPTION
1. 以 release 模式构建 zhu-ye-ime（可跳过）
2. 校验 DLL 导出
3. 复制 DLL 与 v2 词典（优先 data/artifacts/real.zyct）到 Program Files 安装目录；
   目标 DLL 使用版本化文件名，旧版本 DLL 由系统在重启后延迟清理
4. 注册 HKLM TSF TIP/Category/LanguageProfile/CLSID 键并切换到新版本
5. 校验注册结果；失败时自动回滚 InProcServer32 到上一版本

版本化命名规则：
- 未指定 -Version 时使用源 DLL 的 SHA-256 前 8 位作为内容指纹
  （同一构建幂等、不同构建天然不同名，升级零锁冲突）
- 显式传入 -Version abc 时目标文件名为 zhu-ye-ime-abc.dll（用于发布或演练）

重复执行安全；每次安装会先清理旧注册再写入，保证无重复路径残留。
需要管理员权限。

.EXAMPLE
.\scripts\install.ps1

.EXAMPLE
.\scripts\install.ps1 -SkipBuild

.EXAMPLE
.\scripts\install.ps1 -Version 0.1.0

.EXAMPLE
.\scripts\install.ps1 -DictionaryPath .\data\artifacts\real.zyct
#>
[CmdletBinding()]
param(
    [string]$InstallDir,
    [string]$DictionaryPath,
    [string]$Version,
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

function Resolve-DictionarySource {
    param([string]$Path)

    if ($Path) {
        $absolute = [System.IO.Path]::GetFullPath($Path)
        if (-not (Test-Path -LiteralPath $absolute -PathType Leaf)) {
            throw "词典文件不存在: $absolute"
        }
        return $absolute
    }
    foreach ($candidate in @(
        (Join-Path $repoRoot 'dictionary.zyct'),
        (Join-Path $repoRoot 'data\artifacts\real.zyct'),
        (Join-Path $repoRoot 'data\artifacts\seed.zyct')
    )) {
        if (Test-Path -LiteralPath $candidate -PathType Leaf) {
            return $candidate
        }
    }
    Write-Host '未找到现成词典产物，先生成自建演示种子...'
    Push-Location $repoRoot
    try {
        & cargo run -q -p zhu-ye-dict -- build
        if ($LASTEXITCODE -ne 0) {
            throw 'zhu-ye-dict build 失败，无法安装词典。'
        }
    } finally {
        Pop-Location
    }
    $built = Join-Path $repoRoot 'data\artifacts\seed.zyct'
    if (-not (Test-Path -LiteralPath $built -PathType Leaf)) {
        throw '词典构建完成后仍缺失，无法安装。'
    }
    return $built
}

if (-not (Test-Admin)) {
    throw '安装 TSF 服务需要管理员权限，请以管理员身份重新运行。'
}

if (-not $InstallDir) {
    $InstallDir = Get-TsfInstallDir
}
$InstallDir = [System.IO.Path]::GetFullPath($InstallDir)

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

# ---- 版本化 DLL 命名与上一版本记录（升级事务开始前快照，供失败回滚） ----
$dllHash8 = (Get-FileHash -LiteralPath $sourceDll -Algorithm SHA256).Hash.Substring(0, 8).ToLowerInvariant()
$newDllName = if ($Version) { "zhu-ye-ime-$Version.dll" } else { "zhu-ye-ime-$dllHash8.dll" }
$targetDll = Join-Path $InstallDir $newDllName
$previousDll = Get-TsfInprocServerDefault
if ($previousDll) {
    Write-Host "当前注册版本: $previousDll"
}

# 复制新版本：版本化文件名天然无锁（旧 DLL 原地保留，不再抛"被占用"）。
Copy-Item -LiteralPath $sourceDll -Destination $targetDll -Force

try {
    & (Join-Path $PSScriptRoot 'verify-tsf-dll.ps1') -DllPath $targetDll
    if (-not $?) { throw 'DLL 导出校验未通过。' }
} catch {
    # 校验失败视为升级事务未开始：删除未通过校验的副本，注册表保持原状。
    if ($targetDll -ne $previousDll) {
        Remove-Item -LiteralPath $targetDll -Force -ErrorAction SilentlyContinue
    }
    throw "DLL 导出校验失败，安装中止（未通过校验的副本已删除，注册表未改动）: $($_.Exception.Message)"
}

$dictionarySource = Resolve-DictionarySource -Path $DictionaryPath
$targetDictionary = Join-Path $InstallDir $TsfIdentity['DictionaryFileName']
if ([System.IO.Path]::GetFullPath($dictionarySource) -eq [System.IO.Path]::GetFullPath($targetDictionary)) {
    # 源与目标相同（例如 -DictionaryPath 直接指向安装目录中的词典）：无需复制。
    Write-Host "词典已就位（源与目标相同），跳过复制: $targetDictionary"
} else {
    try {
        Copy-Item -LiteralPath $dictionarySource -Destination $targetDictionary -Force
    } catch {
        # 词典复制属于升级事务：失败时撤下新 DLL 副本，保持目录与注册表一致。
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

# ---- 注册表切换到新版本（New-TsfRegistration 内部先清旧再重建） ----
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

# ---- 延迟清理旧版本 DLL（含历史版本、固定名与 .zy-del 迁移残留） ----
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
