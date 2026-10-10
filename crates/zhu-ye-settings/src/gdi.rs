//! GDI 绘制原语：后备缓冲、字体、填充与文本。
//!
//! 设置窗口与工具箱面板是两个窗口，共用这套原语，避免各自维护一份位图与字体生命周期。
//! 字体取自系统消息字体（即系统默认 UI 字体，D-30），随 DPI 重建。

use std::mem::size_of;
use std::path::Path;

use windows::Win32::Foundation::{COLORREF, RECT, SIZE};
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleBitmap, CreateCompatibleDC, CreateFontIndirectW, CreatePen, CreateSolidBrush,
    DeleteDC, DeleteObject, DrawTextW, FillRect, GetDIBits, GetStockObject, GetTextExtentPoint32W,
    RoundRect, SelectObject, SetBkMode, SetTextColor, BITMAPINFO, BITMAPINFOHEADER, BI_RGB,
    CLEARTYPE_QUALITY, DEFAULT_CHARSET, DEFAULT_PITCH, DIB_RGB_COLORS, DRAW_TEXT_FORMAT,
    FF_DONTCARE, FW_BOLD, FW_MEDIUM, FW_NORMAL, FW_SEMIBOLD, HBITMAP, HDC, HFONT, HGDIOBJ,
    LOGFONTW, NULL_BRUSH, PS_NULL, PS_SOLID, TRANSPARENT,
};
use windows::Win32::UI::WindowsAndMessaging::{
    SystemParametersInfoW, NONCLIENTMETRICSW, SPI_GETNONCLIENTMETRICS,
    SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS,
};

use zhu_ye_ui::{UiColor, UiRect};

use crate::shell;
use crate::wide::to_utf16;

/// 用纯色填充矩形。
pub(crate) unsafe fn fill(hdc: HDC, rect: UiRect, color: UiColor) {
    unsafe {
        let brush = CreateSolidBrush(COLORREF(color.to_colorref()));
        let native = to_native(rect);
        FillRect(hdc, &native, brush);
        let _ = DeleteObject(brush.into());
    }
}

/// 用 1px 实色线画圆角矩形描边（chips 细描边、展开区边框等；GDI 圆角描边）。
pub(crate) unsafe fn round_rect_outline(hdc: HDC, rect: UiRect, radius: i32, color: UiColor) {
    unsafe {
        let pen = CreatePen(PS_SOLID, 1, COLORREF(color.to_colorref()));
        let previous_brush = SelectObject(hdc, GetStockObject(NULL_BRUSH));
        let previous_pen = SelectObject(hdc, pen.into());
        let diameter = radius.max(1) * 2;
        let _ = RoundRect(
            hdc,
            rect.left,
            rect.top,
            rect.right,
            rect.bottom,
            diameter,
            diameter,
        );
        let _ = SelectObject(hdc, previous_pen);
        let _ = SelectObject(hdc, previous_brush);
        let _ = DeleteObject(pen.into());
    }
}

/// 用指定字体测量一行文本的像素宽度（「规划中」标签跟在条目标题后的排版需要）。
/// 注意用裸 UTF-16（不带 `to_utf16` 的尾随 NUL），否则宽度多算半个字符。
pub(crate) unsafe fn text_width(hdc: HDC, text: &str, font: HFONT) -> i32 {
    unsafe {
        let wide: Vec<u16> = text.encode_utf16().collect();
        if wide.is_empty() {
            return 0;
        }
        let previous = SelectObject(hdc, font.into());
        let mut size = SIZE::default();
        let _ = GetTextExtentPoint32W(hdc, &wide, &mut size);
        let _ = SelectObject(hdc, previous);
        size.cx
    }
}

/// 用纯色填充圆角矩形。
pub(crate) unsafe fn fill_round(hdc: HDC, rect: UiRect, radius: i32, color: UiColor) {
    unsafe {
        let brush = CreateSolidBrush(COLORREF(color.to_colorref()));
        let pen = CreatePen(PS_NULL, 0, COLORREF(0));
        let previous_brush = SelectObject(hdc, brush.into());
        let previous_pen = SelectObject(hdc, pen.into());
        let diameter = radius.max(1) * 2;
        let _ = RoundRect(
            hdc,
            rect.left,
            rect.top,
            rect.right,
            rect.bottom,
            diameter,
            diameter,
        );
        let _ = SelectObject(hdc, previous_brush);
        let _ = SelectObject(hdc, previous_pen);
        let _ = DeleteObject(brush.into());
        let _ = DeleteObject(pen.into());
    }
}

/// 在矩形内绘制文本。
pub(crate) unsafe fn draw_text(
    hdc: HDC,
    text: &str,
    rect: UiRect,
    color: UiColor,
    font: HFONT,
    format: DRAW_TEXT_FORMAT,
) {
    unsafe {
        // `DrawTextW` 的窗口绑定按切片长度传参，因此这里不追加结尾 NUL。
        let mut wide: Vec<u16> = text.encode_utf16().collect();
        let mut native = to_native(rect);
        if !font.is_invalid() {
            let _ = SelectObject(hdc, font.into());
        }
        let _ = SetBkMode(hdc, TRANSPARENT);
        let _ = SetTextColor(hdc, COLORREF(color.to_colorref()));
        let _ = DrawTextW(hdc, &mut wide, &mut native, format);
    }
}

pub(crate) const fn to_native(rect: UiRect) -> RECT {
    RECT {
        left: rect.left,
        top: rect.top,
        right: rect.right,
        bottom: rect.bottom,
    }
}

/// GDI 后备缓冲；尺寸变化时重建位图。
#[derive(Default)]
pub(crate) struct BackBuffer {
    memory_dc: HDC,
    bitmap: HBITMAP,
    previous: HGDIOBJ,
    width: i32,
    height: i32,
}

impl BackBuffer {
    /// 准备一个不小于给定尺寸的内存 DC。
    pub(crate) unsafe fn prepare(&mut self, window_dc: HDC, width: i32, height: i32) -> HDC {
        unsafe {
            if self.memory_dc.is_invalid() {
                self.memory_dc = CreateCompatibleDC(Some(window_dc));
                if self.memory_dc.is_invalid() {
                    return HDC::default();
                }
            }
            if self.bitmap.is_invalid() || self.width < width || self.height < height {
                if !self.bitmap.is_invalid() {
                    let _ = SelectObject(self.memory_dc, self.previous);
                    let _ = DeleteObject(self.bitmap.into());
                }
                self.bitmap = CreateCompatibleBitmap(window_dc, width, height);
                if self.bitmap.is_invalid() {
                    return HDC::default();
                }
                self.previous = SelectObject(self.memory_dc, self.bitmap.into());
                self.width = width;
                self.height = height;
            }
            self.memory_dc
        }
    }

    /// 把后备缓冲写成 32 位 BMP（自顶向下行序，便于肉眼与像素取证核对）。
    pub(crate) unsafe fn save_bmp(
        &self,
        width: i32,
        height: i32,
        path: &Path,
    ) -> Result<(), String> {
        unsafe {
            if self.bitmap.is_invalid() {
                return Err("没有可导出的位图".to_owned());
            }
            let mut info = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: u32::try_from(size_of::<BITMAPINFOHEADER>()).unwrap_or(40),
                    biWidth: width,
                    biHeight: -height,
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB.0,
                    ..Default::default()
                },
                ..Default::default()
            };
            let mut pixels = vec![0u8; (width as usize) * (height as usize) * 4];
            let lines = GetDIBits(
                self.memory_dc,
                self.bitmap,
                0,
                u32::try_from(height).unwrap_or(0),
                Some(pixels.as_mut_ptr().cast()),
                &mut info,
                DIB_RGB_COLORS,
            );
            if lines == 0 {
                return Err("读取窗口像素失败".to_owned());
            }
            let mut bmp = Vec::with_capacity(54 + pixels.len());
            bmp.extend_from_slice(b"BM");
            bmp.extend_from_slice(&u32::try_from(54 + pixels.len()).unwrap_or(0).to_le_bytes());
            bmp.extend_from_slice(&0u16.to_le_bytes());
            bmp.extend_from_slice(&0u16.to_le_bytes());
            bmp.extend_from_slice(&54u32.to_le_bytes());
            bmp.extend_from_slice(&40u32.to_le_bytes());
            bmp.extend_from_slice(&width.to_le_bytes());
            bmp.extend_from_slice(&(-height).to_le_bytes());
            bmp.extend_from_slice(&1u16.to_le_bytes());
            bmp.extend_from_slice(&32u16.to_le_bytes());
            bmp.extend_from_slice(&0u32.to_le_bytes());
            bmp.extend_from_slice(&u32::try_from(pixels.len()).unwrap_or(0).to_le_bytes());
            bmp.extend_from_slice(&0i32.to_le_bytes());
            bmp.extend_from_slice(&0i32.to_le_bytes());
            bmp.extend_from_slice(&0u32.to_le_bytes());
            bmp.extend_from_slice(&0u32.to_le_bytes());
            bmp.extend_from_slice(&pixels);
            std::fs::write(path, bmp).map_err(|error| format!("写出截图失败: {error}"))
        }
    }
}

impl Drop for BackBuffer {
    fn drop(&mut self) {
        unsafe {
            if !self.bitmap.is_invalid() {
                let _ = SelectObject(self.memory_dc, self.previous);
                let _ = DeleteObject(self.bitmap.into());
            }
            if !self.memory_dc.is_invalid() {
                let _ = DeleteDC(self.memory_dc);
            }
        }
    }
}

/// 字体集合；随 DPI 变化重建。
#[derive(Default)]
pub(crate) struct Fonts {
    pub(crate) title: HFONT,
    pub(crate) body: HFONT,
    pub(crate) small: HFONT,
    /// 选中导航项专用：与 `body` 同字号、字重加粗（T-125 排版规范化）。
    pub(crate) nav_active: HFONT,
    /// 按钮文字：与 `small` 同字号、字重 500（T-148 视觉改版 §5 字体角色）。
    pub(crate) small_medium: HFONT,
    /// 选中 chips 文字：与 `small` 同字号、字重加粗（T-148 §7 选中浅底深字 600）。
    pub(crate) small_semibold: HFONT,
    pub(crate) body_height: i32,
    pub(crate) small_height: i32,
    dpi: u32,
}

impl Fonts {
    /// 按 DPI 准备字体；DPI 未变时复用。
    pub(crate) unsafe fn ensure(&mut self, dpi: u32) {
        unsafe {
            if self.dpi == dpi && !self.body.is_invalid() {
                return;
            }
            self.release();
            let base = system_message_font();
            let system_dpi = shell::system_dpi().max(1) as i32;
            let target_dpi = i32::try_from(dpi).unwrap_or(96);
            let scale = |logical: i32, numerator: i32, denominator: i32| -> i32 {
                (logical * numerator * target_dpi / (denominator * system_dpi)).max(1)
            };
            // `lfHeight` 为负表示字符高度（正数表示单元格高度）；先取绝对值再缩放，
            // 否则 `.max(1)` 会把负值钳成 1，字体退化成 1 像素高。
            let base_height = base.lfHeight.abs().max(1);

            // T-125 排版规范化：正文 16px（基础字号 14-16px 区间上限）、主标题 24px 加粗、
            // 说明文字 13px；相对系统消息字体（96dpi 通常 12px）换算。
            let mut body_font = base;
            body_font.lfHeight = -scale(base_height, 4, 3);
            let mut title_font = base;
            title_font.lfHeight = -scale(base_height, 6, 3);
            title_font.lfWeight = FW_BOLD.0 as i32;
            let mut small_font = base;
            small_font.lfHeight = -scale(base_height, 13, 12);
            let mut nav_active_font = body_font;
            nav_active_font.lfWeight = FW_SEMIBOLD.0 as i32;
            let mut small_medium_font = small_font;
            small_medium_font.lfWeight = FW_MEDIUM.0 as i32;
            let mut small_semibold_font = small_font;
            small_semibold_font.lfWeight = FW_SEMIBOLD.0 as i32;

            self.body = CreateFontIndirectW(&body_font);
            self.title = CreateFontIndirectW(&title_font);
            self.small = CreateFontIndirectW(&small_font);
            self.nav_active = CreateFontIndirectW(&nav_active_font);
            self.small_medium = CreateFontIndirectW(&small_medium_font);
            self.small_semibold = CreateFontIndirectW(&small_semibold_font);
            self.body_height = body_font.lfHeight.abs();
            self.small_height = small_font.lfHeight.abs();
            self.dpi = dpi;
        }
    }

    /// 释放字体；DPI 变化或窗口销毁时调用。
    pub(crate) unsafe fn release(&mut self) {
        unsafe {
            for font in [
                self.title,
                self.body,
                self.small,
                self.nav_active,
                self.small_medium,
                self.small_semibold,
            ] {
                if !font.is_invalid() {
                    let _ = DeleteObject(font.into());
                }
            }
            self.title = HFONT::default();
            self.body = HFONT::default();
            self.small = HFONT::default();
            self.nav_active = HFONT::default();
            self.small_medium = HFONT::default();
            self.small_semibold = HFONT::default();
            self.body_height = 0;
            self.small_height = 0;
            self.dpi = 0;
        }
    }
}

impl Drop for Fonts {
    fn drop(&mut self) {
        unsafe {
            self.release();
        }
    }
}

/// 读取系统消息字体（即系统默认 UI 字体，D-30）。
unsafe fn system_message_font() -> LOGFONTW {
    unsafe {
        let mut metrics = NONCLIENTMETRICSW {
            cbSize: u32::try_from(size_of::<NONCLIENTMETRICSW>()).unwrap_or(0),
            ..Default::default()
        };
        let _ = SystemParametersInfoW(
            SPI_GETNONCLIENTMETRICS,
            metrics.cbSize,
            Some((&mut metrics as *mut NONCLIENTMETRICSW).cast()),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS::default(),
        );
        let mut font = metrics.lfMessageFont;
        if font.lfHeight == 0 {
            // 读不到系统字体时回退，避免画出零高字体。
            font = LOGFONTW {
                lfHeight: -12,
                lfWeight: FW_NORMAL.0 as i32,
                lfCharSet: DEFAULT_CHARSET,
                lfQuality: CLEARTYPE_QUALITY,
                lfPitchAndFamily: DEFAULT_PITCH.0 | FF_DONTCARE.0,
                ..Default::default()
            };
            set_face_name(&mut font, &to_utf16("Microsoft YaHei UI"));
        }
        font
    }
}

/// 写入字体字面名（最多 `LF_FACESIZE - 1` 个 UTF-16 单元，含结尾 NUL）。
fn set_face_name(font: &mut LOGFONTW, name: &[u16]) {
    let capacity = font.lfFaceName.len();
    if capacity == 0 {
        return;
    }
    for (index, unit) in name.iter().take(capacity - 1).enumerate() {
        font.lfFaceName[index] = *unit;
    }
}
