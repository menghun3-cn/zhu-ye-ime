#requires -Version 5.1
<#
.SYNOPSIS
生成可复制到其他 Windows 机器进行 TSF 注册/卸载测试的便携包。

.DESCRIPTION
1. 若缺少 release DLL，先构建 zhu-ye-ime
2. 准备 v2 词典（优先 data/artifacts/real.zyct，否则构建演示种子）
3. 将 DLL、词典与安装/卸载/校验脚本复制到 target/portable 暂存目录
4. 生成 zip，测试机无需安装 Rust，直接解压后以管理员运行
   scripts/install.ps1 -SkipBuild

.EXAMPLE
.\scripts\package-portable.ps1

.EXAMPLE
.\scripts\package-portable.ps1 -Version 0.1.0
#>
[CmdletBinding()]
param(
    [string]$Version,
    [string]$DictionaryPath
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
$packageName = "ai-zhu-ye-ime-$Version-test"
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
$docsDir = Join-Path $staging 'docs'
$null = New-Item -ItemType Directory -Path $docsDir -Force

# Windows PowerShell 5.1 需要 UTF-8 BOM 才能正确解析中文脚本。
foreach ($script in @('ime-identity.ps1', 'install.ps1', 'uninstall.ps1', 'verify-tsf-dll.ps1')) {
    $sourcePath = Join-Path $PSScriptRoot $script
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
Copy-Item -LiteralPath $dllSource -Destination (Join-Path $dllDir 'zhu_ye_ime.dll')
Copy-Item -LiteralPath (Join-Path $repoRoot 'docs\licenses.md') -Destination $docsDir
Copy-Item -LiteralPath (Join-Path $repoRoot 'docs\数据清单.md') -Destination $docsDir

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
            throw 'zhu-ye-dict build 失败，无法打包词典。'
        }
    } finally {
        Pop-Location
    }
    return (Join-Path $repoRoot 'data\artifacts\seed.zyct')
}

$dictionarySource = Resolve-DictionarySource -Path $DictionaryPath
Copy-Item -LiteralPath $dictionarySource -Destination (Join-Path $staging 'dictionary.zyct')

$testGuide = @'
# 竹叶输入法便携测试包

本包用于在隔离的 Windows 10/11 x64 机器上验证 TSF 注册、输入、候选窗、译文层、用户词与卸载闭环（T-010 至 T-013）。
包含 release DLL 与 v2 词典（优先真实词库；未生成真实词库时使用自建演示种子）。

## 测试步骤

1. 解压本包到测试机，例如 `C:\ai-zhu-ye-ime-0.1.0-test`。
2. 推荐先创建系统还原点或虚拟机快照，便于反复执行安装/卸载。
3. 以管理员身份打开 PowerShell。
4. 执行安装：
   `.\scripts\install.ps1 -SkipBuild`
5. 到“设置 -> 时间和语言 -> 语言和区域”，确认竹叶输入法出现在输入法列表中。
6. 切换竹叶输入法并输入 `nihao`，确认候选出现“你好”及右侧英文译文。
7. 按 `Tab` 进入译文层，按 `1` 或对应数字上屏英文；按 `Shift` 验证中英切换。
8. 连续上屏一个词后重启记事本，确认用户词仍然保留并可提升排序。
9. 执行卸载并确认输入法从系统列表消失；重复安装/卸载两次验证幂等。

## 参考检查

- DLL 导出由 `.\scripts\verify-tsf-dll.ps1 -DllPath .\target\release\zhu_ye_ime.dll` 校验。
- 安装器会把 `dictionary.zyct` 复制到 DLL 同目录；卸载时一并清理。
- 本包不包含 Rust 工具链；请始终使用 `-SkipBuild`。
- 词典数据来源与许可证见包内 `docs/licenses.md` 与 `docs/数据清单.md`。
'@
[System.IO.File]::WriteAllText(
    (Join-Path $staging 'README-测试.txt'),
    ($testGuide -replace "`r?`n", "`r`n"),
    [System.Text.UTF8Encoding]::new($true)
)

Compress-Archive -Path (Join-Path $staging '*') -DestinationPath $zipPath -Force
Remove-Item -LiteralPath $staging -Recurse -Force

Write-Host "便携测试包已生成: $zipPath"
Write-Host '在测试机解压后执行：.\scripts\install.ps1 -SkipBuild'
