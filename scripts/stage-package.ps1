#requires -Version 5.1
<#
.SYNOPSIS
生成发行包 staging 目录（zip 便携包与 Inno exe 安装包共用载荷，T-097）。

.DESCRIPTION
staging 布局（与便携 zip 完全一致，保证 zip/exe 同源同哈希）：

    <StagingDir>/
      bin/zhu_ye_ime.dll             TSF 服务 DLL
      bin/zhu-ye-settings.exe        设置窗口
      bin/zhu-ye-updater.exe         词典更新器（唯一联网组件）
      bin/zhu-ye-tray.exe            托盘常驻进程（批六，中英状态图标）
      bin/dictionary.zyct            基础词典（data/artifacts/base.zyct，随安装只读）
      bin/dictionary.zyct.tones      带调拼音旁挂表（base.zyct.tones，批四候选声调显示）
      bin/en.zyen                    英文词表（T-085，data/artifacts/en.zyen，随安装只读）
      packs/it.zyct med.zyct slang.zyct  预置领域包（D-46，可离线验收）
      scripts/ime-identity.ps1 install.ps1 uninstall.ps1 verify-tsf-dll.ps1 verify-tsf-identity.ps1
      docs/licenses.md 数据清单.md
      README-测试.txt

脚本不清理 StagingDir 已有内容（调用方负责）；5 个脚本转 UTF-8 BOM（Windows
PowerShell 5.1 中文解析，T-079 身份门禁与 bug-fix Agent Note 2026-10-02 要求）。

.EXAMPLE
.\scripts\stage-package.ps1 -Version 0.1.2 -StagingDir target\portable\stage-012
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$Version,
    [Parameter(Mandatory)][string]$StagingDir
)

$ErrorActionPreference = 'Stop'
$repoRoot = Split-Path -Parent $PSScriptRoot

$needed = @(
    (Join-Path $repoRoot 'target\release\zhu_ye_ime.dll'),
    (Join-Path $repoRoot 'target\release\zhu-ye-settings.exe'),
    (Join-Path $repoRoot 'target\release\zhu-ye-updater.exe'),
    (Join-Path $repoRoot 'target\release\zhu-ye-tray.exe')
)
foreach ($artifact in $needed) {
    if (-not (Test-Path -LiteralPath $artifact -PathType Leaf)) {
        throw "release 产物缺失：$artifact（先运行 cargo build --release -p zhu-ye-ime -p zhu-ye-settings -p zhu-ye-updater -p zhu-ye-tray）"
    }
}

$baseDictionary = Join-Path $repoRoot 'data\artifacts\base.zyct'
if (-not (Test-Path -LiteralPath $baseDictionary -PathType Leaf)) {
    throw "基础词典缺失：$baseDictionary（先运行 zhu-ye-dict build-base）"
}
$toneSource = Join-Path $repoRoot 'data\artifacts\base.zyct.tones'
if (-not (Test-Path -LiteralPath $toneSource -PathType Leaf)) {
    throw "带调拼音旁挂表缺失：$toneSource（先运行 zhu-ye-dict build-base）"
}
$enWordbook = Join-Path $repoRoot 'data\artifacts\en.zyen'
if (-not (Test-Path -LiteralPath $enWordbook -PathType Leaf)) {
    throw "英文词表缺失：$enWordbook（先运行 scripts\build-en-wordbook.ps1）"
}

$scriptDir = Join-Path $StagingDir 'scripts'
$binDir = Join-Path $StagingDir 'bin'
$packsDir = Join-Path $StagingDir 'packs'
$docsDir = Join-Path $StagingDir 'docs'
$null = New-Item -ItemType Directory -Path $scriptDir, $binDir, $packsDir, $docsDir -Force

# Windows PowerShell 5.1 需要 UTF-8 BOM 才能正确解析中文脚本
#（含 T-079 身份门禁，见 bug-fix Agent Note 2026-10-02）。
foreach ($script in @(
    'ime-identity.ps1', 'install.ps1', 'uninstall.ps1',
    'verify-tsf-dll.ps1', 'verify-tsf-identity.ps1'
)) {
    $sourcePath = Join-Path $PSScriptRoot $script
    if (-not (Test-Path -LiteralPath $sourcePath -PathType Leaf)) {
        throw "脚本缺失：$sourcePath"
    }
    $scriptText = [System.IO.File]::ReadAllText(
        $sourcePath,
        [System.Text.UTF8Encoding]::new($false)
    )
    [System.IO.File]::WriteAllText(
        (Join-Path $scriptDir $script),
        $scriptText,
        [System.Text.UTF8Encoding]::new($true)
    )
}

# ---- bin：DLL、设置窗口、更新器、托盘、基础词典 ----
Copy-Item -LiteralPath (Join-Path $repoRoot 'target\release\zhu_ye_ime.dll') -Destination $binDir
Copy-Item -LiteralPath (Join-Path $repoRoot 'target\release\zhu-ye-settings.exe') -Destination $binDir
Copy-Item -LiteralPath (Join-Path $repoRoot 'target\release\zhu-ye-updater.exe') -Destination $binDir
Copy-Item -LiteralPath (Join-Path $repoRoot 'target\release\zhu-ye-tray.exe') -Destination $binDir
Copy-Item -LiteralPath $baseDictionary -Destination (Join-Path $binDir 'dictionary.zyct')
Copy-Item -LiteralPath $toneSource -Destination (Join-Path $binDir 'dictionary.zyct.tones')
Copy-Item -LiteralPath $enWordbook -Destination (Join-Path $binDir 'en.zyen')

# ---- packs：三个领域包（D-46，可离线验收） ----
foreach ($pack in @('it.zyct', 'med.zyct', 'slang.zyct')) {
    $source = Join-Path $repoRoot "data\artifacts\$pack"
    if (-not (Test-Path -LiteralPath $source -PathType Leaf)) {
        throw "领域包缺失：$source（先运行 zhu-ye-dict 的领域包构建）"
    }
    Copy-Item -LiteralPath $source -Destination (Join-Path $packsDir $pack)
}

Copy-Item -LiteralPath (Join-Path $repoRoot 'docs\licenses.md') -Destination $docsDir
Copy-Item -LiteralPath (Join-Path $repoRoot 'docs\数据清单.md') -Destination $docsDir

$packsSize = (Get-ChildItem -LiteralPath $packsDir -File | Measure-Object -Property Length -Sum).Sum
$testGuide = @'
# 竹叶输入法发行包（测试）

本包用于在隔离的 Windows 10/11 x64 机器上验证：TSF 注册、输入、候选窗、译文层、
用户词、设置窗口、领域包离线可用与卸载闭环（T-073 至 T-078）。
包含 TSF 服务 DLL、基础词典（base.zyct 裁剪版）、设置窗口、更新器与三个领域包
（it/med/slang，合计约 2.3 MB）。

## 结构

    bin/zhu_ye_ime.dll             TSF 服务 DLL
    bin/zhu-ye-settings.exe        设置窗口
    bin/zhu-ye-updater.exe         词典更新器（唯一联网组件）
    bin/zhu-ye-tray.exe            托盘常驻进程（中英状态图标，批六）
    bin/dictionary.zyct            基础词典
    bin/dictionary.zyct.tones      带调拼音旁挂表（候选拼音声调，批四）
    bin/en.zyen                    英文词表（英文前缀候选，T-085）
    packs/                         三个领域包（安装时预置到 %APPDATA%）
    scripts/                       安装/卸载/校验脚本

## 测试步骤

1. 解压本包到测试机，例如 `C:\zhu-ye-ime-0.1.0-test`。
2. 推荐先创建系统还原点或虚拟机快照，便于反复执行安装/卸载。
3. 以管理员身份打开 PowerShell。
4. 执行安装：`.\scripts\install.ps1`
   （发行包模式从包内 bin/ 取文件，不依赖源码树与 cargo；无 Rust 工具链也可安装）
5. 到“设置 -> 时间和语言 -> 语言和区域”，确认竹叶输入法出现在输入法列表中。
6. 切换竹叶输入法并输入 `nihao`，确认候选出现“你好”及右侧英文译文。
7. 按 `Tab` 进入译文层，按 `1` 或对应数字上屏英文；按 `Shift` 验证中英切换。
8. 连续上屏一个词后重启记事本，确认用户词仍然保留并可提升排序。
9. 从开始菜单打开“竹叶输入法设置”，确认三页可浏览、添加词库页列出三个领域包。
10. 执行卸载：`.\scripts\uninstall.ps1`，确认输入法从系统列表消失、
    设置窗口与快捷方式被清理；重复安装/卸载两次验证幂等。

## 参考检查

- DLL 导出由 `.\scripts\verify-tsf-dll.ps1 -DllPath .\bin\zhu_ye_ime.dll` 校验。
- 标识常量由 `.\scripts\verify-tsf-identity.ps1` 交叉比对（7 项）。
- 安装器会把 `bin\dictionary.zyct` 与 `bin\en.zyen` 复制到 DLL 同目录，并把三个领域包预置到
  `%APPDATA%\zhu-ye-ime\packs\`；卸载清理程序文件但保留用户数据目录。
- 本包不包含 Rust 工具链；请勿在包内运行 cargo 类命令。
- 词典数据来源与许可证见包内 `docs/licenses.md` 与 `docs/数据清单.md`。
'@
[System.IO.File]::WriteAllText(
    (Join-Path $StagingDir 'README-测试.txt'),
    ($testGuide -replace "`r?`n", "`r`n"),
    [System.Text.UTF8Encoding]::new($true)
)

Write-Host "staging 已生成: $StagingDir"
Write-Host "领域包合计: $([math]::Round($packsSize / 1MB, 2)) MB（D-46 口径）"
