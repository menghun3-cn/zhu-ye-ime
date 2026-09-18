#requires -Version 5.1
<#
.SYNOPSIS
生成可复制到其他 Windows 机器进行 TSF 注册/卸载测试的便携包。

.DESCRIPTION
1. 若缺少 release DLL，先构建 zhu-ye-ime
2. 将 DLL 与安装/卸载/校验脚本复制到 target/portable 暂存目录
3. 生成 zip，测试机无需安装 Rust，直接解压后以管理员运行
   scripts/install.ps1 -SkipBuild

.EXAMPLE
.\scripts\package-portable.ps1

.EXAMPLE
.\scripts\package-portable.ps1 -Version 0.1.0
#>
[CmdletBinding()]
param(
    [string]$Version
)

$ErrorActionPreference = 'Stop'
$repoRoot = Split-Path -Parent $PSScriptRoot

if (-not $Version) {
    $match = Select-String -LiteralPath (Join-Path $repoRoot 'Cargo.toml') -Pattern '^version = "([^"]+)"' |
        Select-Object -First 1
    if ($null -eq $match) {
        throw '无法从根 Cargo.toml 读取 version。'
    }
    $Version = $match.Matches[0].Groups[1].Value
}

$dllSource = Join-Path $repoRoot 'target\release\zhu_ye_ime.dll'
if (-not (Test-Path -LiteralPath $dllSource -PathType Leaf)) {
    Write-Host '未找到 release DLL，先执行构建...'
    Push-Location $repoRoot
    try {
        & cargo build -p zhu-ye-ime --release
        if ($LASTEXITCODE -ne 0) {
            throw 'cargo build 失败，无法生成测试包。'
        }
    } finally {
        Pop-Location
    }
}

$portableRoot = Join-Path $repoRoot 'target\portable'
$packageName = "ai-zhu-ye-ime-m1-test-$Version"
$staging = Join-Path $portableRoot $packageName
$zipPath = Join-Path $portableRoot "$packageName.zip"

if (Test-Path -LiteralPath $staging) {
    Remove-Item -LiteralPath $staging -Recurse -Force
}
if (Test-Path -LiteralPath $zipPath) {
    Remove-Item -LiteralPath $zipPath -Force
}

$scriptDir = Join-Path $staging 'scripts'
$dllDir = Join-Path $staging 'target\release'
$null = New-Item -ItemType Directory -Path $scriptDir, $dllDir -Force

foreach ($script in @('ime-identity.ps1', 'install.ps1', 'uninstall.ps1', 'verify-tsf-dll.ps1')) {
    Copy-Item -LiteralPath (Join-Path $PSScriptRoot $script) -Destination $scriptDir
}
Copy-Item -LiteralPath $dllSource -Destination (Join-Path $dllDir 'zhu_ye_ime.dll')

$testGuide = @'
# 竹叶输入法 M1 便携测试包

本包用于在隔离的 Windows 10/11 x64 机器上验证 TSF 注册、加载与卸载闭环（T-010）。
M1 尚未实现按键上屏，本次只验证输入法能注册、宿主能加载 DLL、卸载无残留。

## 测试步骤

1. 解压本包到测试机，例如 `C:\ai-zhu-ye-ime-m1-test-0.1.0`。
2. 推荐先创建系统还原点或虚拟机快照，便于反复执行安装/卸载。
3. 以管理员身份打开 PowerShell。
4. 执行安装：
   `.\scripts\install.ps1 -SkipBuild`
5. 到“设置 -> 时间和语言 -> 语言和区域”，确认竹叶输入法出现在输入法列表中。
6. 切换竹叶输入法后，观察宿主进程能否正常加载 DLL；若使用进程监视器，
   可确认 zhu-ye-ime.dll 被 TSF 宿主加载。
7. 执行卸载：
   `.\scripts\uninstall.ps1`
8. 确认输入法从系统列表消失，重复安装/卸载两次验证幂等。

## 参考检查

- DLL 导出由 `.\scripts\verify-tsf-dll.ps1 -DllPath .\target\release\zhu_ye_ime.dll` 校验。
- 本包不包含 Rust 工具链；请始终使用 `-SkipBuild`。
'@
[System.IO.File]::WriteAllText(
    (Join-Path $staging 'README-M1测试.txt'),
    ($testGuide -replace "`r?`n", "`r`n"),
    [System.Text.UTF8Encoding]::new($false)
)

Compress-Archive -Path (Join-Path $staging '*') -DestinationPath $zipPath -Force
Remove-Item -LiteralPath $staging -Recurse -Force

Write-Host "便携测试包已生成: $zipPath"
Write-Host '在测试机解压后执行：.\scripts\install.ps1 -SkipBuild'
