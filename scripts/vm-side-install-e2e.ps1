# VM 侧执行脚本：安装/覆盖/卸载 e2e（§17.1）。由本机 scripts/vm-install-e2e.ps1 推送并在
# 管理员任务（/rl HIGHEST）中运行。约定：内容尽量 ASCII；结果文件**最后**写入（UTF-8）。
#
# 用法: powershell -NoProfile -ExecutionPolicy Bypass -File <此脚本> -SetupExe C:\zhu-ye-e2e\setup.exe -Result C:\zhu-ye-e2e\result.txt
param(
    [Parameter(Mandatory)] [string]$SetupExe,
    [Parameter(Mandatory)] [string]$Result
)

$ErrorActionPreference = 'Stop'
$log = @()
function Write-Line([string]$text) { $script:log += $text }
function Check([string]$name, [bool]$cond, [string]$detail) {
    if ($cond) { Write-Line "PASS $name $detail" } else { Write-Line "FAIL $name $detail" }
}
$appDir = Join-Path $env:ProgramFiles 'ai-zhu-ye-ime'
$tipKey = 'HKLM:\SOFTWARE\Microsoft\CTF\TIP\{E54D6682-8650-40E7-A9EE-6FD1137849AE}'
$profileKey = 'HKLM:\SOFTWARE\Microsoft\CTF\TIP\{E54D6682-8650-40E7-A9EE-6FD1137849AE}\LanguageProfile\0x00000804\{6315FE74-92C3-439B-8CDF-FDB6E43EDAF1}'

$setupArgs = @('/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART', '/SP-')
function Invoke-Setup { param([string]$Path)
    $p = Start-Process -FilePath $Path -ArgumentList $setupArgs -PassThru -Wait
    return $p.ExitCode
}

$null = New-Item -ItemType Directory -Path (Split-Path $Result -Parent) -Force | Out-Null
if (-not (Test-Path -LiteralPath $SetupExe -PathType Leaf)) {
    Write-Line "FAIL 前置 setup.exe 缺失: $SetupExe"
    [System.IO.File]::WriteAllLines($Result, $log, (New-Object System.Text.UTF8Encoding($false)))
    exit 1
}

# ---- 阶段 0：干净前提（卸载/注册残留不得存在） ----
Check '前置干净: 目录不存在' (-not (Test-Path $appDir)) ''
Check '前置干净: HKLM TIP 键不存在' (-not (Test-Path $tipKey)) ''

# ---- 阶段 1：静默安装 ----
$code = Invoke-Setup $SetupExe
Check '安装 exit=0' ($code -eq 0) "exit=$code"
Check '安装后: 程序目录存在' (Test-Path $appDir) ''
Check '安装后: tsf 目录存在' (Test-Path (Join-Path $appDir 'tsf')) ''
Check '安装后: packs 预置存在' ((Test-Path (Join-Path $appDir 'packs\it.zyct')) -and (Test-Path (Join-Path $appDir 'packs\med.zyct')) -and (Test-Path (Join-Path $appDir 'packs\slang.zyct'))) ''
Check '安装后: HKLM TIP 注册存在' (Test-Path $tipKey) ''
Check '安装后: 语言档案 Enable=1' ((Get-ItemProperty $profileKey -Name Enable -ErrorAction SilentlyContinue).Enable -eq 1) "Enable=$((Get-ItemProperty $profileKey -Name Enable -ErrorAction SilentlyContinue).Enable)"

# ---- 阶段 2：同版本覆盖安装（升级路径语义） ----
$code2 = Invoke-Setup $SetupExe
Check '覆盖安装 exit=0' ($code2 -eq 0) "exit=$code2"
Check '覆盖后: 程序目录与 DLL 完好' ((Test-Path $appDir) -and (Test-Path (Join-Path $appDir 'bin\zhu-ye-ime.dll'))) ''
Check '覆盖后: TSF 注册仍存在' (Test-Path $tipKey) ''

# ---- 阶段 3：卸载（unins000 静默，内部调 uninstall.ps1 清 TSF 注册） ----
$unins = Join-Path $appDir 'unins000.exe'
if (Test-Path -LiteralPath $unins) {
    $p = Start-Process -FilePath $unins -ArgumentList @('/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART') -PassThru -Wait
    Start-Sleep -Seconds 3
    Check '卸载 exit=0' ($p.ExitCode -eq 0) "exit=$($p.ExitCode)"
} else {
    Check '卸载入口存在' $false "unins000 缺失: $appDir"
}

# ---- 阶段 4：清场验证 ----
Check '卸载后: 程序目录已移除' (-not (Test-Path $appDir)) ''
Check '卸载后: HKLM TIP 键已移除' (-not (Test-Path $tipKey)) ''

# ---- 结果文件最后写 ----
[System.IO.File]::WriteAllLines($Result, $log, (New-Object System.Text.UTF8Encoding($false)))
Write-Host 'done'