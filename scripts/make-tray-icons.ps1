# 生成托盘状态图标（T-123）：白"中"/白"英" + 品牌橙方形底。
#
# 背景：原 tray-zh.ico / tray-en.ico 为白字形 + 全透明底，在 Win11 托盘
# 圆角遮罩下呈现"圆形图案中英字"。本次改为 16/32/48 三帧橙底白字方形
# 图标，与 DLL 指示器品牌源 zhu-16.png（#E5881E 方底白字"竹"）同款配色。
#
# 输出：crates/zhu-ye-tray/assets/tray-zh.ico / tray-en.ico（Vista+ PNG 帧
# ICO，LoadImageW 原生支持；build.rs 的 101/102 语义不变）。
#
# 用法：powershell -ExecutionPolicy Bypass -File scripts/make-tray-icons.ps1
# 依赖：仅 .NET System.Drawing（Windows 自带）。

Add-Type -AssemblyName System.Drawing

$ErrorActionPreference = 'Stop'

$OrangeArgb = [System.Drawing.Color]::FromArgb(255, 229, 136, 30)   # #E5881E 品牌橙
$Sizes = @(16, 32, 48)

function New-TrayPng {
    param([int]$Size, [string]$Glyph)
    $bmp = New-Object System.Drawing.Bitmap($Size, $Size, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.Clear($OrangeArgb)
    $g.TextRenderingHint = [System.Drawing.Text.TextRenderingHint]::AntiAliasGridFit
    $fontSize = [Math]::Round($Size * 0.72)
    $font = New-Object System.Drawing.Font('Microsoft YaHei', $fontSize, [System.Drawing.FontStyle]::Bold, [System.Drawing.GraphicsUnit]::Pixel)
    $brush = [System.Drawing.Brushes]::White
    $fmt = New-Object System.Drawing.StringFormat
    $fmt.Alignment = [System.Drawing.StringAlignment]::Center
    $fmt.LineAlignment = [System.Drawing.StringAlignment]::Center
    $rect = New-Object System.Drawing.RectangleF(0, 0, $Size, $Size)
    $g.DrawString($Glyph, $font, $brush, $rect, $fmt)
    $ms = New-Object System.IO.MemoryStream
    $bmp.Save($ms, [System.Drawing.Imaging.ImageFormat]::Png)
    $bytes = $ms.ToArray()
    $ms.Dispose(); $g.Dispose(); $font.Dispose(); $bmp.Dispose()
    Write-Output (, $bytes)          # 逗号包裹防字节数组被 PowerShell 展开
}

function New-IconFile {
    param([string]$Glyph, [string]$OutPath)
    $frames = New-Object 'System.Collections.Generic.List[object]'
    foreach ($s in $Sizes) {
        $png = New-TrayPng -Size $s -Glyph $Glyph
        $frames.Add(@{ Size = $s; Data = $png })
        Write-Output "  frame ${s}x${s}: $($png.Length) bytes"
    }
    $count = $frames.Count
    $headerSize = 6 + 16 * $count
    $ms = New-Object System.IO.MemoryStream
    $bw = New-Object System.IO.BinaryWriter($ms)
    $bw.Write([UInt16]0); $bw.Write([UInt16]1); $bw.Write([UInt16]$count)
    $offset = $headerSize
    foreach ($f in $frames) {
        $s = $f.Size
        $bw.Write([Byte]$s); $bw.Write([Byte]$s)
        $bw.Write([Byte]0); $bw.Write([Byte]0)
        $bw.Write([UInt16]1); $bw.Write([UInt16]32)
        $bw.Write([UInt32]$f.Data.Length); $bw.Write([UInt32]$offset)
        $offset += $f.Data.Length
    }
    foreach ($f in $frames) { $bw.Write($f.Data) }
    $bw.Flush()
    $out = $ms.ToArray()
    $bw.Dispose(); $ms.Dispose()
    [System.IO.File]::WriteAllBytes($OutPath, $out)
    Write-Output "wrote $OutPath ($($out.Length) bytes, frames=$count)"
}

New-IconFile -Glyph '中' -OutPath 'D:\pcdata\code\aicg\ai-zhu-ye-ime\crates\zhu-ye-tray\assets\tray-zh.ico'
New-IconFile -Glyph '英' -OutPath 'D:\pcdata\code\aicg\ai-zhu-ye-ime\crates\zhu-ye-tray\assets\tray-en.ico'
