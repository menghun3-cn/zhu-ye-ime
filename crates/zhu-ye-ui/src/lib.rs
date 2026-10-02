//! UI 原语：颜色、主题种类、系统色快照、布局矩形与文本测量。
//!
//! 本 crate 刻意不依赖 Win32，也不依赖任何领域 crate：深浅色、高对比度配色
//! 与不同 DPI 下的行布局都可以纯单元测试验证；渲染层（候选窗、设置窗口）
//! 只负责把计算结果画出来。第八期 T-081 把候选窗与设置窗口共享的这些原语
//! 从 `zhu-ye-ime` 的 `candidate_ui` 抽出，设置窗口随之不再连带链接 TSF 侧
//! 代码（S-10，见 [todos-list](../../docs/todos-list.md) 与设计文档 §1）。

/// 基准 DPI，布局全部按 `dpi / 96` 线性缩放。
pub const BASE_DPI: u32 = 96;

/// ARGB-24 颜色，使用 `0xRRGGBB` 表示。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiColor(pub u32);

impl UiColor {
    /// 从 RGB 分量构造颜色。
    #[must_use]
    pub const fn rgb(red: u8, green: u8, blue: u8) -> Self {
        Self(((red as u32) << 16) | ((green as u32) << 8) | blue as u32)
    }

    /// 转 GDI `COLORREF`（`0x00BBGGRR`）所需字节序。
    #[must_use]
    pub const fn to_colorref(self) -> u32 {
        (self.0 & 0xFF) << 16 | (self.0 & 0x00FF00) | (self.0 >> 16)
    }
}

/// 预设主题种类；系统高对比度走独立配色。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UiThemeKind {
    /// 浅色。
    #[default]
    Light,
    /// 深色。
    Dark,
    /// 系统高对比度。
    HighContrast,
}

/// 系统高对比度颜色快照；由 Win32 层读取后交给主题转换。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SystemColors {
    /// `COLOR_WINDOW`。
    pub window: u32,
    /// `COLOR_WINDOWTEXT`。
    pub window_text: u32,
    /// `COLOR_GRAYTEXT`。
    pub gray_text: u32,
    /// `COLOR_HIGHLIGHT`。
    pub highlight: u32,
    /// `COLOR_HIGHLIGHTTEXT`。
    pub highlight_text: u32,
    /// `COLOR_BTNFACE`。
    pub btn_face: u32,
}

/// 布局矩形。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiRect {
    /// 左边界。
    pub left: i32,
    /// 上边界。
    pub top: i32,
    /// 右边界（不含）。
    pub right: i32,
    /// 下边界（不含）。
    pub bottom: i32,
}

impl UiRect {
    /// 矩形宽度。
    #[must_use]
    pub fn width(&self) -> i32 {
        self.right - self.left
    }

    /// 矩形高度。
    #[must_use]
    pub fn height(&self) -> i32 {
        self.bottom - self.top
    }
}

/// 估算文本像素宽度：ASCII 约为 0.55 倍字高，CJK 与其他字符约 1 倍字高。
#[must_use]
pub fn estimate_text_width(text: &str, font_height: i32) -> f32 {
    let mut width = 0.0f32;
    for ch in text.chars() {
        if ch.is_ascii() {
            width += font_height.max(1) as f32 * 0.55;
        } else {
            width += font_height.max(1) as f32;
        }
    }
    width
}

/// 按估算宽度截断文本并追加省略号；宽于 `max_width` 时返回短文本。
#[must_use]
pub fn fit_text(text: &str, max_width: i32, font_height: i32) -> String {
    if max_width <= 0 {
        return String::new();
    }
    let ellipsis_width = estimate_text_width("…", font_height);
    let max_width = max_width.max(0) as f32 - ellipsis_width;
    let mut width = 0.0f32;
    let mut fit = String::new();
    for ch in text.chars() {
        let char_width = if ch.is_ascii() {
            font_height.max(1) as f32 * 0.55
        } else {
            font_height.max(1) as f32
        };
        if width + char_width > max_width {
            fit.push('…');
            return fit;
        }
        width += char_width;
        fit.push(ch);
    }
    fit
}

#[cfg(test)]
mod tests {
    use super::{estimate_text_width, fit_text, UiColor, UiRect, UiThemeKind, BASE_DPI};

    #[test]
    fn 颜色按分量构造且可还原颜色值() {
        let color = UiColor::rgb(0x1E, 0x88, 0xE5);
        assert_eq!(color.0, 0x001E_88E5);
        // 分量为零的通道参与位运算后仍为零。
        assert_eq!(UiColor::rgb(0, 0, 0).0, 0);
        assert_eq!(UiColor::rgb(0xFF, 0xFF, 0xFF).0, 0xFF_FFFF);
    }

    #[test]
    fn colorref字节序为bbggrr() {
        // 0xRRGGBB -> 0x00BBGGRR：红蓝互换、绿色居中。
        assert_eq!(UiColor::rgb(0x1E, 0x88, 0xE5).to_colorref(), 0x00E5_881E);
        assert_eq!(UiColor::rgb(0xFF, 0, 0).to_colorref(), 0x0000_00FF);
        assert_eq!(UiColor::rgb(0, 0, 0xFF).to_colorref(), 0x00FF_0000);
        // 纯灰互换后不变。
        assert_eq!(UiColor::rgb(0x7F, 0x7F, 0x7F).to_colorref(), 0x007F_7F7F);
    }

    #[test]
    fn 主题种类默认浅色() {
        assert_eq!(UiThemeKind::default(), UiThemeKind::Light);
        assert_ne!(UiThemeKind::Dark, UiThemeKind::HighContrast);
    }

    #[test]
    fn 矩形宽高为右减左与下减上() {
        let rect = UiRect {
            left: 3,
            top: 2,
            right: 13,
            bottom: 8,
        };
        assert_eq!(rect.width(), 10);
        assert_eq!(rect.height(), 6);
        // 空矩形不 panic（宽度可为 0 或负，由调用方解释）。
        let empty = UiRect {
            left: 4,
            top: 4,
            right: 4,
            bottom: 2,
        };
        assert_eq!(empty.width(), 0);
        assert_eq!(empty.height(), -2);
    }

    #[test]
    fn 基准dpi为96() {
        assert_eq!(BASE_DPI, 96);
    }

    #[test]
    fn 文本宽度按字符估算() {
        let height = 16;
        // ASCII 每字符 0.55 字高，CJK 每字符 1 字高。
        assert!((estimate_text_width("ab", height) - 0.55 * 2.0 * height as f32).abs() < 0.001);
        assert_eq!(estimate_text_width("你好", height), 2.0 * height as f32);
        assert_eq!(estimate_text_width("", height), 0.0);
        // 高度为 0 或负数时按 1px 计量，不 panic。
        assert_eq!(estimate_text_width("a", 0), 0.55);
    }

    #[test]
    fn 长文本按宽度截断且保留省略号() {
        let long_cjk = "这是一个非常长的中文候选词".repeat(12);
        let fitted = fit_text(&long_cjk, 180, 16);
        assert!(fitted.ends_with('…'));
        // 截断后宽度不超过上限。
        assert!(estimate_text_width(&fitted, 16) <= 180.0 + 0.001);
        // 短文本原样返回。
        let short = "短";
        assert_eq!(fit_text(short, 180, 16), short);
        // 非法宽度返回空串。
        assert_eq!(fit_text(long_cjk.as_str(), 0, 16), "");
    }
}
