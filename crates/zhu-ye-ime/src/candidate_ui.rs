//! 候选窗视图模型、主题与 DPI 布局计算。
//!
//! 本模块刻意不依赖 Win32，深浅色、高对比度配色与不同 DPI 下的行布局
//! 都可以纯单元测试验证；`candidate_window` 只负责把计算结果画出来。

use zhu_ye_core::candidate::CandidateSource;

/// 单页默认候选项数，与 1-9 数字选择保持一致。
pub const DEFAULT_PAGE_SIZE: usize = 9;

/// 基准 DPI，布局全部按 `dpi / 96` 线性缩放。
pub const BASE_DPI: u32 = 96;

/// 候选窗展示项。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateUiItem {
    /// 中文候选文本。
    pub text: String,
    /// 译文；无译文时为空字符串。
    pub translation: String,
    /// 候选来源，用于 UI 弱化标识。
    pub source: CandidateSource,
}

/// 一次候选窗快照。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateUiView {
    /// 当前拼音组合串。
    pub composition: String,
    /// 首行分组提示，如 `ni hao`。
    pub pinyin_hint: String,
    /// 当前页码，从 0 开始。
    pub page: usize,
    /// 每页最大条数。
    pub page_size: usize,
    /// 当前页中选中序号，从 0 开始。
    pub selected: usize,
    /// 是否处于译文层；译文层以译文作为主文本。
    pub translation_mode: bool,
    /// 全部候选；窗口只展示当前页。
    pub items: Vec<CandidateUiItem>,
}

impl CandidateUiView {
    /// 返回当前页可见候选。
    #[must_use]
    pub fn visible_items(&self) -> &[CandidateUiItem] {
        let page_size = self.page_size.max(1);
        let start = self.page.saturating_mul(page_size);
        if start >= self.items.len() {
            return &[];
        }
        let end = start.saturating_add(page_size).min(self.items.len());
        &self.items[start..end]
    }

    /// 当前页实际选中的序号；没有候选时返回 `None`。
    #[must_use]
    pub fn selected_on_page(&self) -> Option<usize> {
        if self.visible_items().is_empty() {
            return None;
        }
        Some(
            self.selected
                .min(self.visible_items().len().saturating_sub(1)),
        )
    }

    /// 计算窗口应该预留的总行高（含页眉与内边距）。
    #[must_use]
    pub fn panel_rows(&self) -> usize {
        let visible = self.visible_items().len();
        if visible == 0 {
            0
        } else {
            self.page_size.max(1)
        }
    }
}

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

/// 候选窗配色。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CandidateUiTheme {
    /// 窗口背景。
    pub background: UiColor,
    /// 主文本。
    pub foreground: UiColor,
    /// 弱化文本（译文、提示、拼音）。
    pub secondary: UiColor,
    /// 边框。
    pub border: UiColor,
    /// 选中行背景。
    pub highlight_background: UiColor,
    /// 选中行文本。
    pub highlight_foreground: UiColor,
    /// 页码标记。
    pub marker: UiColor,
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

/// 返回预设主题；高对比度使用系统色构建，深浅色使用项目自定配色。
#[must_use]
pub fn theme(kind: UiThemeKind) -> CandidateUiTheme {
    match kind {
        UiThemeKind::Light => CandidateUiTheme {
            background: UiColor::rgb(0xFA, 0xFA, 0xFA),
            foreground: UiColor::rgb(0x1F, 0x1F, 0x1F),
            secondary: UiColor::rgb(0x6E, 0x6E, 0x6E),
            border: UiColor::rgb(0xD7, 0xD7, 0xD7),
            highlight_background: UiColor::rgb(0xE8, 0xF0, 0xFE),
            highlight_foreground: UiColor::rgb(0x0B, 0x57, 0xD0),
            marker: UiColor::rgb(0x0B, 0x57, 0xD0),
        },
        UiThemeKind::Dark => CandidateUiTheme {
            background: UiColor::rgb(0x20, 0x20, 0x20),
            foreground: UiColor::rgb(0xED, 0xED, 0xED),
            secondary: UiColor::rgb(0x9E, 0x9E, 0x9E),
            border: UiColor::rgb(0x3C, 0x3C, 0x3C),
            highlight_background: UiColor::rgb(0x3A, 0x4A, 0x5C),
            highlight_foreground: UiColor::rgb(0xFF, 0xFF, 0xFF),
            marker: UiColor::rgb(0x8A, 0xB4, 0xF8),
        },
        UiThemeKind::HighContrast => theme_from_system_colors(SystemColors {
            window: 0xFF00_0000,
            window_text: 0x00FF_FFFF,
            gray_text: 0x00FF_FFFF,
            highlight: 0x00FF_FF00,
            highlight_text: 0x0000_0000,
            btn_face: 0x0000_0000,
        }),
    }
}

/// 按系统高对比度颜色构建主题；系统色为 `0x00BBGGRR` 字节序。
#[must_use]
pub fn theme_from_system_colors(colors: SystemColors) -> CandidateUiTheme {
    CandidateUiTheme {
        background: UiColor(bgr_to_rgb(colors.window)),
        foreground: UiColor(bgr_to_rgb(colors.window_text)),
        secondary: UiColor(bgr_to_rgb(colors.gray_text)),
        border: UiColor(bgr_to_rgb(colors.btn_face)),
        highlight_background: UiColor(bgr_to_rgb(colors.highlight)),
        highlight_foreground: UiColor(bgr_to_rgb(colors.highlight_text)),
        marker: UiColor(bgr_to_rgb(colors.highlight_text)),
    }
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

/// 候选窗布局尺寸；全部按 DPI 缩放，测试可验证比例关系。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CandidateMetrics {
    /// 当前 DPI。
    pub dpi: u32,
    /// 面板宽度。
    pub panel_width: i32,
    /// 面板水平内边距。
    pub padding_x: i32,
    /// 面板垂直线内边距。
    pub padding_y: i32,
    /// 页眉高度。
    pub header_height: i32,
    /// 单行高度。
    pub row_height: i32,
    /// 行间距。
    pub row_gap: i32,
    /// 数字标记列宽。
    pub marker_width: i32,
    /// 主文本与译文分栏间距。
    pub translation_gap: i32,
    /// 圆角半径。
    pub corner_radius: i32,
    /// 字体像素高度。
    pub font_height: i32,
}

impl CandidateMetrics {
    /// 按 DPI 构造布局；DPI 小于基准时按基准计算。
    #[must_use]
    pub fn new(dpi: u32) -> Self {
        let dpi = dpi.max(BASE_DPI);
        let scale = dpi as f32 / BASE_DPI as f32;
        let dp = |value: f32| (value * scale).round().max(1.0) as i32;
        Self {
            dpi,
            panel_width: dp(360.0),
            padding_x: dp(12.0),
            padding_y: dp(10.0),
            header_height: dp(38.0),
            row_height: dp(36.0),
            row_gap: dp(2.0),
            marker_width: dp(40.0),
            translation_gap: dp(16.0),
            corner_radius: dp(8.0),
            font_height: dp(16.0),
        }
    }

    /// 面板总尺寸；`rows` 为页面预留行数。
    #[must_use]
    pub fn panel_size(&self, rows: usize) -> (i32, i32) {
        let rows = i32::try_from(rows).unwrap_or(i32::MAX).max(1);
        let rows_height = rows * self.row_height + (rows - 1) * self.row_gap;
        (
            self.panel_width,
            self.header_height + rows_height + self.padding_y * 2,
        )
    }

    /// 页眉矩形。
    #[must_use]
    pub fn header_rect(&self) -> UiRect {
        UiRect {
            left: self.padding_x,
            top: self.padding_y,
            right: self.panel_width - self.padding_x,
            bottom: self.padding_y + self.header_height,
        }
    }

    /// 页眉左侧输入串矩形，与右侧提示互不重叠。
    #[must_use]
    pub fn header_text_rect(&self) -> UiRect {
        let header = self.header_rect();
        let hint = self.header_hint_rect();
        let right = hint
            .left
            .saturating_sub(self.translation_gap.max(1))
            .max(header.left + 1);
        UiRect {
            left: header.left,
            top: header.top,
            right,
            bottom: header.bottom,
        }
    }

    /// 页眉右侧拼音提示矩形。
    #[must_use]
    pub fn header_hint_rect(&self) -> UiRect {
        let header = self.header_rect();
        let hint_width = (header.width() * 5 / 9).max(1);
        UiRect {
            left: header.right - hint_width,
            top: header.top,
            right: header.right,
            bottom: header.bottom,
        }
    }

    /// 页面第 `index` 行（0 起）的整行矩形。
    #[must_use]
    pub fn row_rect(&self, index: usize) -> UiRect {
        let index = i32::try_from(index).unwrap_or(i32::MAX);
        let top = self.padding_y + self.header_height + index * (self.row_height + self.row_gap);
        UiRect {
            left: self.padding_x,
            top,
            right: self.panel_width - self.padding_x,
            bottom: top + self.row_height,
        }
    }

    /// 行内数字标记矩形。
    #[must_use]
    pub fn marker_rect(&self, row: UiRect) -> UiRect {
        UiRect {
            left: row.left,
            top: row.top,
            right: row.left + self.marker_width,
            bottom: row.bottom,
        }
    }

    /// 行内主文本矩形。
    #[must_use]
    pub fn text_rect(&self, row: UiRect) -> UiRect {
        let left = (row.left + self.marker_width).min(row.right);
        let translation_width = (row.width() - self.marker_width).max(0) / 3;
        let right = row
            .right
            .saturating_sub(translation_width.saturating_add(self.translation_gap))
            .max(left + 1);
        UiRect {
            left,
            top: row.top,
            right,
            bottom: row.bottom,
        }
    }

    /// 行内译文矩形（右侧弱化区）。
    #[must_use]
    pub fn translation_rect(&self, row: UiRect) -> UiRect {
        let right = row.right;
        let translation_width = (row.width() - self.marker_width).max(0) / 3;
        let left = right.saturating_sub(translation_width).max(row.left);
        UiRect {
            left,
            top: row.top,
            right,
            bottom: row.bottom,
        }
    }
}

/// 候选序号标记：1-9 使用圈数字，其余使用 `10.` 样式。
#[must_use]
pub fn index_marker(index: usize) -> String {
    match index {
        0 => "①",
        1 => "②",
        2 => "③",
        3 => "④",
        4 => "⑤",
        5 => "⑥",
        6 => "⑦",
        7 => "⑧",
        8 => "⑨",
        _ => return format!("{}.", index + 1),
    }
    .to_owned()
}

/// 估算文本像素宽度：ASCII 约为 0.55 倍字高，CJK 与其他字符约 1 倍字高。
#[allow(dead_code)] // 供后续候选窗精确排版与 T-013 文本测量使用。
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
#[allow(dead_code)] // 供后续候选窗精确排版与 T-013 文本测量使用。
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

fn bgr_to_rgb(color: u32) -> u32 {
    (color & 0xFF) << 16 | (color & 0x00FF00) | (color >> 16)
}

#[cfg(test)]
mod tests {
    use super::{
        fit_text, index_marker, theme, theme_from_system_colors, CandidateMetrics, CandidateUiItem,
        CandidateUiView, SystemColors, UiThemeKind, DEFAULT_PAGE_SIZE,
    };
    use zhu_ye_core::candidate::CandidateSource;

    fn view_with(items: usize) -> CandidateUiView {
        CandidateUiView {
            composition: "nihao".to_owned(),
            pinyin_hint: "ni hao".to_owned(),
            page: 0,
            page_size: DEFAULT_PAGE_SIZE,
            selected: 0,
            translation_mode: false,
            items: (0..items)
                .map(|index| CandidateUiItem {
                    text: format!("候选{index}"),
                    translation: if index % 2 == 0 {
                        format!("translation {index}")
                    } else {
                        String::new()
                    },
                    source: CandidateSource::Static,
                })
                .collect(),
        }
    }

    #[test]
    fn 可见候选按页截取() {
        let view = view_with(20);
        assert_eq!(view.visible_items().len(), DEFAULT_PAGE_SIZE);
        let mut page2 = view.clone();
        page2.page = 1;
        assert_eq!(page2.visible_items().len(), DEFAULT_PAGE_SIZE);
        assert_eq!(page2.visible_items()[0].text, "候选9");
    }

    #[test]
    fn 空候选返回空页且无选中() {
        let view = view_with(0);
        assert!(view.visible_items().is_empty());
        assert_eq!(view.selected_on_page(), None);
        assert_eq!(view.panel_rows(), 0);
    }

    #[test]
    fn 深浅色与高对比度配色可区分() {
        let light = theme(UiThemeKind::Light);
        let dark = theme(UiThemeKind::Dark);
        assert_ne!(light.background, dark.background);
        assert_ne!(light.foreground, dark.foreground);

        let high_contrast = theme_from_system_colors(SystemColors {
            window: 0x0000_0000,
            window_text: 0x00FF_FFFF,
            gray_text: 0x00FF_FFFF,
            highlight: 0x0000_FFFF,
            highlight_text: 0x0000_0000,
            btn_face: 0x0000_0000,
        });
        assert_eq!(high_contrast.background.to_colorref(), 0x0000_0000);
        assert_eq!(high_contrast.foreground.to_colorref(), 0x00FF_FFFF);
        assert_eq!(
            high_contrast.highlight_background.to_colorref(),
            0x0000_FFFF
        );
    }

    #[test]
    fn dpi缩放保持布局比例() {
        let base = CandidateMetrics::new(96);
        let scaled = CandidateMetrics::new(192);
        assert_eq!(scaled.panel_width, base.panel_width * 2);
        assert_eq!(scaled.row_height, base.row_height * 2);
        assert_eq!(scaled.font_height, base.font_height * 2);
        assert_eq!(scaled.panel_size(9).1, base.panel_size(9).1 * 2);
        assert_eq!(scaled.panel_size(9).0, base.panel_size(9).0 * 2);
    }

    #[test]
    fn 行矩形单调且不重叠() {
        let metrics = CandidateMetrics::new(96);
        let mut previous_bottom = 0i32;
        for index in 0..DEFAULT_PAGE_SIZE {
            let rect = metrics.row_rect(index);
            assert!(rect.top >= previous_bottom);
            assert!(rect.width() > 0);
            assert!(rect.height() == metrics.row_height);
            previous_bottom = rect.bottom;
            let text = metrics.text_rect(rect);
            let translation = metrics.translation_rect(rect);
            assert!(text.right <= translation.left);
            assert!(translation.right == rect.right);
        }
    }

    #[test]
    fn 页眉输入串与提示不重叠且译文与主文本保留间距() {
        let metrics = CandidateMetrics::new(96);
        let text = metrics.header_text_rect();
        let hint = metrics.header_hint_rect();
        assert!(text.right <= hint.left);
        assert!(text.left < text.right);
        assert!(hint.left < hint.right);

        let row = metrics.row_rect(0);
        let main = metrics.text_rect(row);
        let translation = metrics.translation_rect(row);
        assert!(translation.left - main.right >= metrics.translation_gap);
    }

    #[test]
    fn 长文本按宽度截断且保留省略号() {
        let long_cjk = "这是一个特别长的候选词用来验证文本不会溢出窗口边界".repeat(3);
        let fitted = fit_text(&long_cjk, 180, 16);
        assert!(fitted.ends_with('…'));
        assert!(fitted.chars().count() < long_cjk.chars().count());

        let short = "你好";
        assert_eq!(fit_text(short, 180, 16), short);
        assert_eq!(index_marker(0), "①");
        assert_eq!(index_marker(9), "10.");
    }
}
