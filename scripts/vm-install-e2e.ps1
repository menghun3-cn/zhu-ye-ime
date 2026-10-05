# 本机侧驱动：VM 安装/覆盖/卸载 e2e（§17.1）。
# 用法: .\scripts\vm-install-e2e.ps1 [-LocalExe target\portable\ai-zhu-ye-ime-setup-0.1.2-alpha.exe]
# 前置：docs/vm-win-info.md 或 ZHUYE_VM_IP/USER/PASS 已可解析；VM 已开管理员共享（C$）。
# 流程：上传 exe 与 VM 侧脚本 → schtasks /rl HIGHEST 运行 → 拉回结果 → 展示。
param(
    [string]$LocalExe
)

$ErrorActionPreference = 'Stop'
$repoRoot = Split-Path -Parent $PSScriptRoot
$common = Join-Path $repoRoot '.agents\skills\vm-accept-sop\scripts\vm-common.ps1'
if (-not (Test-Path -LiteralPath $common)) { throw "缺少 vm-common.ps1：$common" }
. $common

if (-not $LocalExe) {
    $candidates = @(Get-ChildItem (Join-Path $repoRoot 'target\portable') -Filter 'ai-zhu-ye-ime-setup-*.exe' -ErrorAction SilentlyContinue) +
        @(Get-ChildItem (Join-Path $repoRoot 'target') -Filter 'ai-zhu-ye-ime-setup-*.exe' -ErrorAction SilentlyContinue)
    $LocalExe = ($candidates | Sort-Object LastWriteTime -Descending | Select-Object -First 1).FullName
}
if (-not $LocalExe -or -not (Test-Path -LiteralPath $LocalExe)) {
    throw "未找到 setup exe。先用 build-setup.ps1 构建：$LocalExe"
}
Write-Host "setup exe: $LocalExe ($([math]::Round((Get-Item $LocalExe).Length / 1MB, 2)) MB)"

$side = Join-Path $repoRoot 'scripts\vm-side-install-e2e.ps1'
$vmRoot = 'C:\zhu-ye-e2e'
$vmExe = "$vmRoot\setup.exe"
$vmScript = "$vmRoot\vm-side-install-e2e.ps1"
$vmResult = "$vmRoot\result.txt"

Connect-VmShare
Push-VmFile $LocalExe $vmExe
Push-VmFile $side $vmScript
$lines = Invoke-VmTask -Name ZhuYeInstallE2E -ScriptPath $vmScript -ResultPath $vmResult -TimeoutSec 420 -Highest
if ($null -eq $lines) { throw '任务超时未产出结果文件。' }
Write-Host '===== VM e2e 结果 ====='
$lines | ForEach-Object { Write-Host $_ }
$failCount = ($lines | Where-Object { $_ -like 'FAIL*' }).Count
Write-Host "fail=$failCount"
if ($failCount -gt 0) { exit 1 }