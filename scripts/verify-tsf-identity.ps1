#requires -Version 5.1
<#
.SYNOPSIS
静态比对 TSF 标识常量的双份维护（Rust 侧 ↔ scripts/ime-identity.ps1）是否逐字一致。

.DESCRIPTION
TSF 标识常量（CLSID、Profile GUID、键盘 TFCAT、语言 ID、安装目录段、词典文件名）在
Rust 侧（crates/zhu-ye-ime/src/tsf.rs）与 PowerShell 侧（scripts/ime-identity.ps1 的
$script:TsfIdentity 与 Get-TsfInstallDir）中双份存在——这是 D-42 接受"注册表只由
install/uninstall 脚本管理"不变量被部分取代的代价。本脚本从两侧提取上述常量并逐字比对，
任一漂移即失败（退出码 1）。

与 scripts/verify-tsf-dll.ps1 同类：只读源码与常量，不改动注册表与文件系统。

.EXAMPLE
.\scripts\verify-tsf-identity.ps1
#>
[Diagnostics.CodeAnalysis.SuppressMessageAttribute('PSReviewUnusedParameter', '',
    Justification = 'dot-source ime-identity.ps1 暴露的函数在此使用')]
param()

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$repoRoot = Split-Path -Parent $PSScriptRoot
$rustSource = Join-Path $repoRoot 'crates\zhu-ye-ime\src\tsf.rs'
$identityScript = Join-Path $repoRoot 'scripts\ime-identity.ps1'

if (-not (Test-Path -LiteralPath $rustSource)) {
    throw "找不到 Rust 标识常量源: $rustSource"
}

# dot-source 安装/卸载共享工具读取 PowerShell 侧常量；本文件声明"只读共享、加载无副作用"，
# 与 verify-tsf-dll.ps1 加载被校验 DLL 的约束同属静态校验。
. $identityScript

$rust = Get-Content -Raw -LiteralPath $rustSource

function Get-RustGuidConst {
    param([string]$ConstName)

    $pattern = ("{0}\s*:\s*windows::core::GUID\s*=\s*windows::core::GUID::from_u128\(" +
        "0x([0-9A-Fa-f_]+)\)") -f $ConstName
    $match = [regex]::Match($rust, $pattern)
    if (-not $match.Success) {
        throw "tsf.rs 中找不到 GUID 常量 $ConstName"
    }
    $hex = ($match.Groups[1].Value -replace '_', '').ToUpperInvariant()
    if ($hex.Length -ne 32) {
        throw "$ConstName 的十六进制字面量长度异常（$($hex.Length)，期望 32）"
    }
    return ('{{{0}-{1}-{2}-{3}-{4}}}' -f $hex.Substring(0, 8), $hex.Substring(8, 4),
        $hex.Substring(12, 4), $hex.Substring(16, 4), $hex.Substring(20, 12))
}

function Get-RustStrConst {
    param([string]$ConstName)

    $pattern = ('{0}\s*:\s*&str\s*=\s*"([^"]+)"' -f $ConstName)
    $match = [regex]::Match($rust, $pattern)
    if (-not $match.Success) {
        throw "tsf.rs 中找不到 &str 常量 $ConstName"
    }
    # 展开 Rust 字符串字面量的转义：Rust 源码中的 `\\` 表示单个反斜杠（本项目常量只用到
    # 反斜杠转义）。用字面替换，避免正则替换语义的转义歧义。
    return $match.Groups[1].Value.Replace('\\', '\')
}

$psIdentity = $script:TsfIdentity
$psInstallDir = Get-TsfInstallDir
$psInstallSegments = @($psInstallDir -split '[\\/]')
$psInstallDirRelative = if ($psInstallSegments.Count -ge 2) {
    ($psInstallSegments | Select-Object -Last 2) -join '\'
} else {
    $psInstallDir
}

# (名称, Rust 侧值, PowerShell 侧值)
$pairs = @(
    @('TipClsid', (Get-RustGuidConst 'CLSID_ZHU_YE_TIP'), $psIdentity['TipClsid']),
    @('ProfileGuid', (Get-RustGuidConst 'PROFILE_GUID_ZHU_YE'), $psIdentity['ProfileGuid']),
    @('KeyboardCategoryGuid(TFCAT)', (Get-RustGuidConst 'TFCAT_ZHU_YE_KEYBOARD'),
        $psIdentity['KeyboardCategoryGuid']),
    @('LanguageIdHex', (Get-RustStrConst 'TSF_LANGUAGE_ID_HEX'), $psIdentity['LanguageIdHex']),
    @('InstallDir(相对段)', (Get-RustStrConst 'TSF_INSTALL_DIR_RELATIVE'), $psInstallDirRelative),
    @('DictionaryFileName', (Get-RustStrConst 'DICTIONARY_FILE_NAME'),
        $psIdentity['DictionaryFileName'])
)

$failed = $false
foreach ($pair in $pairs) {
    $name = $pair[0]
    $rustValue = [string]$pair[1]
    $psValue = [string]$pair[2]
    if ($rustValue -ceq $psValue) {
        Write-Host ("  {0,-26} PASS  Rust={1}  PS={2}" -f $name, $rustValue, $psValue)
    } else {
        Write-Host ("  {0,-26} FAIL  Rust={1}  PS={2}" -f $name, $rustValue, $psValue)
        $failed = $true
    }
}

if ($failed) {
    Write-Host ""
    throw "TSF 标识常量两侧不一致：Rust（tsf.rs）与 PowerShell（ime-identity.ps1）必须逐字一致，" +
        "请统一后重跑本脚本（D-42 双份维护约束）。"
}
Write-Host ""
Write-Host "TSF 标识常量交叉一致：$($pairs.Count) 项全部通过。"
