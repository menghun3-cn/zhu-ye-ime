#requires -Version 5.1
<#
.SYNOPSIS
用 Inno Setup 生成 exe 安装包（T-097，D-74）：外壳复用 install.ps1/uninstall.ps1，
载荷与便携 zip 同一 staging（zip/exe 同源同哈希）。

.DESCRIPTION
流程：
1. 调用 stage-package.ps1 生成 staging（bin/scripts/packs/docs/README）
2. 由模板生成 setup.iss（UTF-8 BOM；AppName/版本/载荷路径/签名位按参数注入）
3. 调用 ISCC 编译，产出 target/portable/ai-zhu-ye-ime-setup-<ver>.exe
4. 清理 staging 与 iss，输出产物路径、大小与 SHA-256

Inno 安装语义（与验收标准 §17.1 对齐）：
- 载荷直接安装到 {app}（默认 %ProgramFiles%\ai-zhu-ye-ime），目录页禁用，
  固定目录保证与 install.ps1 的 Get-TsfInstallDir 语义一致
- [Run] 调用 install.ps1（-SkipBuild -InstallDir {app}\tsf）：版本化 DLL 复制、
  TSF 注册、领域包预置、快捷方式全部复用既有脚本，零行为改写
- [UninstallRun] 调用 uninstall.ps1（-InstallDir {app}\tsf）：TSF 注册清理 +
  程序文件清理；%APPDATA%\ai-zhu-ye-ime 用户数据保留
- Inno 卸载器随后删除自身注册的 scripts/packs/docs/bin 文件，与脚本清理互补

签名（D-75）：默认不签名（接受 SmartScreen）；-SignCommand 提供签名命令模板时
（$f 为产物占位），iss 注入 SignTool 段，ISCC 编译时对产物签名。

.EXAMPLE
.\scripts\build-setup.ps1                            # 无签名构建
.\scripts\build-setup.ps1 -Version 0.1.2-alpha
.\scripts\build-setup.ps1 -SignCommand "signtool sign /f cert.pfx /p xxx /fd SHA256 /t http://timestamp.digicert.com `$f"
#>
[CmdletBinding()]
param(
    [string]$Version,
    [string]$SignCommand
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

# ---- 定位 ISCC（Inno Setup 6 命令行编译器） ----
$iscc = $env:INNO_ISCC
if (-not $iscc) {
    $candidates = @(
        (Join-Path $env:LOCALAPPDATA 'Programs\Inno Setup 6\ISCC.exe'),
        'C:\Program Files (x86)\Inno Setup 6\ISCC.exe',
        'C:\Program Files\Inno Setup 6\ISCC.exe'
    ) + @(Get-ChildItem 'C:\ProgramData\chocolatey\lib\innosetup\tools\ISCC.exe' -ErrorAction SilentlyContinue |
        ForEach-Object { $_.FullName })
    $iscc = $candidates | Where-Object { Test-Path -LiteralPath $_ -PathType Leaf } | Select-Object -First 1
}
if (-not $iscc -or -not (Test-Path -LiteralPath $iscc -PathType Leaf)) {
    throw '未找到 Inno Setup 6 的 ISCC.exe。安装 Inno Setup 6 或设置环境变量 INNO_ISCC。'
}
Write-Host "ISCC: $iscc"

$portableRoot = Join-Path $repoRoot 'target\portable'
$staging = Join-Path $portableRoot "ai-zhu-ye-ime-$Version-stage"
$setupExe = Join-Path $portableRoot "ai-zhu-ye-ime-setup-$Version.exe"
$issPath = Join-Path $portableRoot "setup-$Version.iss"

if (Test-Path -LiteralPath $staging) {
    Remove-Item -LiteralPath $staging -Recurse -Force
}
if (Test-Path -LiteralPath $setupExe) {
    Remove-Item -LiteralPath $setupExe -Force
}

# ---- 1. staging（与便携 zip 同源） ----
# 子脚本 throw 会冒泡终止（父 ErrorActionPreference=Stop）；不检查 LASTEXITCODE
& (Join-Path $PSScriptRoot 'stage-package.ps1') -Version $Version -StagingDir $staging

# ---- 2. 生成 setup.iss（UTF-8 BOM，Inno Setup 6） ----
$signSection = ''
if ($SignCommand) {
    $escaped = ($SignCommand -replace '\$f', '$f').Trim()
    $signSection = "`nSignTool=$escaped"
}
$issTemplate = @"
; 竹叶输入法 exe 安装包（T-097 D-74）——由 build-setup.ps1 生成，勿手改
; 载荷 staging: $staging
#define Version "$Version"
[Setup]
AppId={{8B30A259-4D2F-4A6E-9C1E-0A5F3E7D9B24}}
AppName=竹叶输入法
AppVersion={#Version}
AppVerName=竹叶输入法 {#Version}
AppPublisher=menghun3-cn
AppPublisherURL=https://github.com/menghun3-cn/zhu-ye-ime
DefaultDirName={autopf}\ai-zhu-ye-ime
DisableDirPage=yes
DisableProgramGroupPage=yes
PrivilegesRequired=admin
ArchitecturesInstallIn64BitMode=x64compatible
OutputDir=$portableRoot
OutputBaseFilename=ai-zhu-ye-ime-setup-$Version
Compression=lzma2
SolidCompression=yes
UninstallDisplayName=竹叶输入法 {#Version}
UninstallDisplayIcon={app}\bin\zhu-ye-settings.exe
WizardStyle=modern
SetupLogging=yes
;uncomment with -SignCommand:    SignTool=...
$signSection
[Languages]
; 官方 Inno Setup 6 不捆绑简体中文语言文件（第三方翻译不可在 CI 复现），
; 向导使用英文内置语言；应用名/卸载名/状态文案均为中文（D-74 取舍记录）
Name: "english"; MessagesFile: "compiler:Default.isl"
[Files]
Source: "$staging\bin\*"; DestDir: "{app}\bin"; Flags: ignoreversion recursesubdirs
Source: "$staging\scripts\*"; DestDir: "{app}\scripts"; Flags: ignoreversion
Source: "$staging\packs\*"; DestDir: "{app}\packs"; Flags: ignoreversion
Source: "$staging\docs\*"; DestDir: "{app}\docs"; Flags: ignoreversion
Source: "$staging\README-测试.txt"; DestDir: "{app}"; Flags: ignoreversion
[Run]
; 复用发行包安装脚本：版本化 DLL、TSF 注册、领域包预置、快捷方式（零行为改写）
Filename: "{sys}\WindowsPowerShell\v1.0\powershell.exe"; Parameters: "-NoProfile -ExecutionPolicy Bypass -File ""{app}\scripts\install.ps1"" -SkipBuild -InstallDir ""{app}\tsf"""; Flags: runhidden waituntilterminated; StatusMsg: "正在注册 TSF 输入服务…"
[UninstallRun]
; 复用卸载脚本：TSF 注册清理 + 程序文件清理；%APPDATA% 用户数据保留
Filename: "{sys}\WindowsPowerShell\v1.0\powershell.exe"; Parameters: "-NoProfile -ExecutionPolicy Bypass -File ""{app}\scripts\uninstall.ps1"" -InstallDir ""{app}\tsf"""; Flags: runhidden waituntilterminated
"@
[System.IO.File]::WriteAllText(
    $issPath,
    ($issTemplate -replace "`r?`n", "`r`n"),
    [System.Text.UTF8Encoding]::new($true)
)

# ---- 3. 编译 ----
Write-Host '编译安装包（ISCC）...'
& $iscc /Qp $issPath
if ($LASTEXITCODE -ne 0) {
    throw "ISCC 编译失败（exit code: $LASTEXITCODE）。"
}
if (-not (Test-Path -LiteralPath $setupExe -PathType Leaf)) {
    throw "编译成功但产物缺失：$setupExe"
}

# ---- 4. 清理与报告 ----
Remove-Item -LiteralPath $staging -Recurse -Force
Remove-Item -LiteralPath $issPath -Force
$setupInfo = Get-Item -LiteralPath $setupExe
$hash = (Get-FileHash -LiteralPath $setupExe -Algorithm SHA256).Hash
Write-Host "安装包已生成: $setupExe（$([math]::Round($setupInfo.Length / 1MB, 2)) MB）"
Write-Host "SHA-256: $hash"
Write-Host '验收提示：干净机（或 VM 快照）以管理员运行，验证安装/覆盖/卸载闭环与 zip 载荷哈希一致（验收标准 §17.1）。'
