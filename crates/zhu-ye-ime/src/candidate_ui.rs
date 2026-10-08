//! 候选窗视图模型、主题与 DPI 布局计算。
//!
//! 本模块刻意不依赖 Win32，深浅色、高对比度配色与不同 DPI 下的行布局
//! 都可以纯单元测试验证；`candidate_window` 只负责把计算结果画出来。
//! 颜色、主题种类、系统色、矩形与文本测量等原语自 T-081 起抽取到独立的
//! `zhu-ye-ui` crate，这里以 `pub use` 转发保持既有调用路径不变。

/// 原语转发：`zhu-ye-ui`（T-081 抽取，见 [todos-list](../../docs/todos-list.md)）。
/// `fit_text` 无生产代码消费者（仅测试使用），由 `zhu_ye_ui` 直接承接，不复转发。
pub use zhu_ye_ui::{estimate_text_width, SystemColors, UiColor, UiRect, UiThemeKind, BASE_DPI};

use zhu_ye_core::candidate::CandidateSource;

/// 单页默认候选项数，与 1-9 数字选择保持一致。
pub const DEFAULT_PAGE_SIZE: usize = 9;

/// 候选窗展示项。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateUiItem {
    /// 中文候选文本。
    pub text: String,
    /// 译文；无译文时为空字符串。
    pub translation: String,
    /// 拼音串（T-115 后续：候选窗按字展示"词（音节）"用；无拼音时为空）。
    pub pinyin: String,
    /// 空格分隔的带调拼音（T-112 后续批四：候选窗上方拼音行显示声调；
    /// 如 `生成` → `shēng chéng`）。为空时回退 `pinyin` 的无调拼注。
    pub pinyin_tone: String,
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
    /// 候选总页数（页脚 m/n 指示用，T-040）。
    pub page_count: usize,
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

    /// 页脚翻页指示文本 `m/n`；总页数 ≤1 时无指示（T-040）。
    #[must_use]
    pub fn footer_label(&self) -> Option<String> {
        page_footer_label(self.page, self.page_count)
    }
}

/// 网络语候选的标注文本（M6-R，方案设计 11.4）。
pub const SLANG_LABEL: &str = "[网络]";

/// 候选主文本的展示形式：网络语缩写候选追加 `[网络]` 标注。
///
/// 标注并入主文本而非独立列，因此 `row_split` 的宽度估算天然把它算进去，
/// 不会与译文区重叠。译文层展示译文本身，不加标注。
#[must_use]
pub fn display_main_text(item: &CandidateUiItem, translation_mode: bool) -> String {
    if translation_mode && !item.translation.is_empty() {
        return item.translation.clone();
    }
    if item.source == CandidateSource::Slang {
        return format!("{} {}", item.text, SLANG_LABEL);
    }
    item.text.clone()
}

/// 候选窗配色。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CandidateUiTheme {
    /// 窗口背景。
    pub background: UiColor,
    /// 主文本。
    pub foreground: UiColor,
    /// 弱化文本（译文、提示、序号）。
    pub secondary: UiColor,
    /// 拼音行小字专用色（T-122：加深灰独立于译文/序号，保证小字对比度）。
    pub pin: UiColor,
    /// 边框。
    pub border: UiColor,
    /// 选中行背景。
    pub highlight_background: UiColor,
    /// 选中行文本。
    pub highlight_foreground: UiColor,
    /// 页码标记。
    pub marker: UiColor,
}

/// 返回预设主题；高对比度使用系统色构建，深浅色使用项目自定配色。
#[must_use]
pub fn theme(kind: UiThemeKind) -> CandidateUiTheme {
    match kind {
        UiThemeKind::Light => CandidateUiTheme {
            // T-032 指令配色：白底、亮蓝圆角边框、候选词亮蓝、选中行红字（浅蓝块保留）、序号/译文浅灰。
            background: UiColor::rgb(0xFF, 0xFF, 0xFF),
            foreground: UiColor::rgb(0x1E, 0x88, 0xE5),
            secondary: UiColor::rgb(0x99, 0x99, 0x99),
            // T-122：拼音行深灰（对比度 ≈7:1，小字清晰；仍弱于主文本蓝色）。
            // T-126 用户指令：拼音作辅助提示，改为较淡灰 #888888
            //（对比度 ≈3.4:1，弱于译文/序号 #999999，视觉层级：汉字 > 译文 >
            // 序号 ≈ 拼音 > 背景；声调可读性由 Segoe UI 字体保障）。
            pin: UiColor::rgb(0x88, 0x88, 0x88),
            border: UiColor::rgb(0x1E, 0x88, 0xE5),
            highlight_background: UiColor::rgb(0xE6, 0xF2, 0xFE),
            highlight_foreground: UiColor::rgb(0xD3, 0x2F, 0x2F),
            marker: UiColor::rgb(0x99, 0x99, 0x99),
        },
        UiThemeKind::Dark => CandidateUiTheme {
            // T-032 深色同构：深底、亮蓝边框与候选词、选中亮红字（深蓝灰块保留）、浅灰序号/译文。
            background: UiColor::rgb(0x20, 0x20, 0x20),
            foreground: UiColor::rgb(0x64, 0xB5, 0xF6),
            secondary: UiColor::rgb(0x9E, 0x9E, 0x9E),
            // T-122：拼音行亮灰（比译文/序号更亮一档，深底上保持可读）。
            pin: UiColor::rgb(0xC9, 0xC9, 0xC9),
            border: UiColor::rgb(0x42, 0xA5, 0xF5),
            highlight_background: UiColor::rgb(0x3A, 0x4A, 0x5C),
            highlight_foreground: UiColor::rgb(0xFF, 0x8A, 0x80),
            marker: UiColor::rgb(0x9E, 0x9E, 0x9E),
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
        // T-122：高对比度下拼音行沿用系统 gray_text（与译文/序号同色），
        // 不另起对比度——系统高对比方案本身保证前景可辨。
        pin: UiColor(bgr_to_rgb(colors.gray_text)),
        border: UiColor(bgr_to_rgb(colors.btn_face)),
        highlight_background: UiColor(bgr_to_rgb(colors.highlight)),
        highlight_foreground: UiColor(bgr_to_rgb(colors.highlight_text)),
        marker: UiColor(bgr_to_rgb(colors.highlight_text)),
    }
}

/// 把主题文件的候选窗节叠加到基础配色上：缺失的键保留基础值（"缺键用默认值"，
/// T-088 / FR-048）。`palette` 来自 `candidate` 节，颜色已是 `0xRRGGBB`。
#[must_use]
pub fn theme_with_candidate(
    base: CandidateUiTheme,
    palette: &zhu_ye_core::CandidatePalette,
) -> CandidateUiTheme {
    let overlay = |current: UiColor, theme_color: Option<zhu_ye_core::ThemeColor>| {
        theme_color.map_or(current, |color| UiColor(color.0))
    };
    CandidateUiTheme {
        background: overlay(base.background, palette.background),
        foreground: overlay(base.foreground, palette.foreground),
        secondary: overlay(base.secondary, palette.secondary),
        pin: overlay(base.pin, palette.pin),
        border: overlay(base.border, palette.border),
        highlight_background: overlay(base.highlight_background, palette.highlight_background),
        highlight_foreground: overlay(base.highlight_foreground, palette.highlight_foreground),
        marker: overlay(base.marker, palette.marker),
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
    /// 数字标记列宽（= 主文本左对齐起点偏移）。
    pub marker_width: i32,
    /// 数字与候选词的固定间距（T-122：序号右对齐到列右缘后与词的间距）。
    pub marker_text_gap: i32,
    /// 主文本与译文分栏间距。
    pub translation_gap: i32,
    /// 选中块相对面板左右边框的内缩量（T-043）。
    ///
    /// 固定为 1 物理像素、不做 DPI 缩放：面板边框线由 GDI 1px 笔画绘制，
    /// 任何 DPI 下都恰好占最外圈一像素；选中块内缩 1px 即可贴边同时保留
    /// 边框线不被高亮块覆盖。
    pub highlight_inset_x: i32,
    /// 页脚高度（m/n 翻页指示条）；无候选页（仅页眉条）时不占空间。
    pub footer_height: i32,
    /// 圆角半径。
    pub corner_radius: i32,
    /// 字体像素高度。
    pub font_height: i32,
    /// 拼音行小字体像素高度（T-115 后续：候选词上方拼音小字，仿微软拼音布局）。
    pub pin_font_height: i32,
    /// 拼音行与主文本的垂直间距（T-115 后续；T-126 定稿 4dp——用户要求
    /// "拼音和中文距离减少一半"，原 8dp 减半）。
    pub pin_line_gap: i32,
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
            // T-126：垂直内边距 10→12（面板上下留白更舒展）。
            padding_y: dp(12.0),
            // T-126：页眉 38→48（顶部输入缓冲区更舒展，输入串垂直居中留白
            // 由 16px 字高 + 16px 上下空间组成）。
            header_height: dp(48.0),
            // T-126：候选行 36→48（拼音带 + 主文本 + 每侧 ≈4-5px 呼吸内边距）。
            row_height: dp(48.0),
            // T-126：行间距 2→4（候选行之间留白加大）。
            row_gap: dp(4.0),
            // T-037：序号列收窄使候选词更贴近序号；译文紧随主文本间距减小。
            // T-122：列宽 26→22（词起点左移 4px）并让序号右对齐、词起点固定
            // 留 `marker_text_gap` 间距（4-8px 档），消除数字后多余留白。
            marker_width: dp(22.0),
            marker_text_gap: dp(6.0),
            translation_gap: dp(8.0),
            // T-043：选中块贴面板左右边框（仅保留 1px 边框线内缩）。
            highlight_inset_x: 1,
            // T-040：页脚 m/n 翻页指示条。
            footer_height: dp(20.0),
            corner_radius: dp(8.0),
            font_height: dp(16.0),
            // T-122：拼音行字 11→13、与主文本间距 3→4（小尺寸下更清晰、留出呼吸感）。
            // T-126：拼音-主文本间距取用户 4-8px 档上限 8px；用户此后要求
            // "拼音和中文距离减少一半"，定稿 4px（档内偏紧凑，拼音贴近汉字）。
            // 字号保持 13dp（用户 12/13px 档）。
            pin_font_height: dp(13.0),
            pin_line_gap: dp(4.0),
        }
    }

    /// 以给定面板宽度布局（T-124 长拼音自适应）：候选窗宽度随页眉内容
    /// 扩展后，本帧所有矩形（页眉/行/页脚）按实际客户区宽度计算。
    /// 其余尺寸（字高/内边距/行高）保持不变。
    #[must_use]
    pub fn with_panel_width(&self, panel_width: i32) -> Self {
        let mut layout = *self;
        layout.panel_width = panel_width.max(1);
        layout
    }

    /// 面板总尺寸；`rows` 为页面预留行数，0 表示无候选（只显示页眉条）。
    /// 有候选行时底部追加页脚条（T-040）。
    #[must_use]
    pub fn panel_size(&self, rows: usize) -> (i32, i32) {
        let rows = i32::try_from(rows).unwrap_or(i32::MAX).max(0);
        let rows_height = rows * self.row_height + (rows - 1).max(0) * self.row_gap;
        let footer = if rows > 0 { self.footer_height } else { 0 };
        (
            self.panel_width,
            self.header_height + rows_height + footer + self.padding_y * 2,
        )
    }

    /// 页脚矩形（m/n 翻页指示条，T-040）；位于最后一行下方、底部内边距
    /// 之上，右对齐绘制页码。`rows` 为当前面板候选行数（0 时无意义）。
    #[must_use]
    pub fn footer_rect(&self, rows: usize) -> UiRect {
        let (_, panel_height) = self.panel_size(rows);
        let bottom = panel_height - self.padding_y;
        UiRect {
            left: self.padding_x,
            top: bottom - self.footer_height,
            right: self.panel_width - self.padding_x,
            bottom,
        }
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

    /// 页眉两栏宽度分配（T-124 长拼音自适应）：返回 (输入串宽, 提示宽)。
    ///
    /// 输入串完整优先：短输入（估算总宽仍在固定面板宽内）时不扩面板，
    /// 提示按其内容宽度完整分配；长输入（超出固定宽）时面板按内容扩展，
    /// 输入串完整容纳，提示最多占两者总和的 2/3（输入串至少保底 1/3，
    /// 提示超限截断——提示是辅助信息，用户输入串不可省略）。
    /// 提示为空或与输入串相同时提示宽返回 0（输入串单独占整行）。
    #[must_use]
    pub fn header_widths(&self, composition: &str, hint: &str) -> (i32, i32) {
        let main_w = estimate_text_width(composition, self.font_height)
            .ceil()
            .max(1.0) as i32;
        if hint.is_empty() || hint == composition {
            return (main_w, 0);
        }
        let hint_w = estimate_text_width(hint, self.font_height).ceil().max(1.0) as i32;
        let gap = self.translation_gap.max(1);
        let need = main_w + gap + hint_w + self.padding_x * 2;
        let hint_alloc = if need > self.panel_width {
            // 超固定宽：输入串完整，提示最多占 (输入+间距+提示) 的 2/3。
            hint_w.min(((main_w + gap + hint_w) * 2 / 3).max(1))
        } else {
            hint_w
        };
        (main_w, hint_alloc)
    }

    /// 按 `hint_alloc`（见 [`Self::header_widths`]）布局页眉输入串矩形；
    /// 提示不显示时（alloc=0）输入串占满页眉整行。
    #[must_use]
    pub fn header_text_rect_with(&self, hint_alloc: i32) -> UiRect {
        let header = self.header_rect();
        let right = if hint_alloc > 0 {
            header
                .right
                .saturating_sub(hint_alloc)
                .saturating_sub(self.translation_gap.max(1))
                .max(header.left + 1)
        } else {
            header.right
        };
        UiRect {
            left: header.left,
            top: header.top,
            right,
            bottom: header.bottom,
        }
    }

    /// 按 `hint_alloc` 布局页眉右侧提示矩形（右对齐）；`hint_alloc<=0`
    /// 时返回零宽矩形（左侧输入串占满整行）。
    #[must_use]
    pub fn header_hint_rect_with(&self, hint_alloc: i32) -> UiRect {
        let header = self.header_rect();
        let hint_width = hint_alloc.clamp(1, header.width());
        UiRect {
            left: header.right - hint_width,
            top: header.top,
            right: header.right,
            bottom: header.bottom,
        }
    }

    /// 面板总尺寸（内容自适应宽，T-124）；`rows` 为页面预留行数，
    /// 0 表示无候选（只显示页眉条）。宽度按页眉输入串/提示内容扩展，
    /// 不低于固定 `panel_width`（保底 360dp）；高度与 [`Self::panel_size`]
    /// 一致（有候选行时底部追加页脚条，T-040）。
    #[must_use]
    pub fn panel_size_for(&self, rows: usize, composition: &str, hint: &str) -> (i32, i32) {
        let (_, height) = self.panel_size(rows);
        let (main_w, hint_alloc) = self.header_widths(composition, hint);
        let gap = if hint_alloc > 0 {
            self.translation_gap.max(1)
        } else {
            0
        };
        let width = self
            .panel_width
            .max(main_w + gap + hint_alloc + self.padding_x * 2);
        (width, height)
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

    /// 选中行高亮块矩形（T-043）。
    ///
    /// 上下与 `row_rect` 一致；左右只内缩 `highlight_inset_x`（1 物理像素），
    /// 即高亮块几乎贴满面板左右边框（搜狗风整行选中块），而序号/文本等行
    /// 内容仍按 `row_rect` 布局，序号因此相对高亮块左缘保留足量内边距。
    #[must_use]
    pub fn highlight_rect(&self, index: usize) -> UiRect {
        let row = self.row_rect(index);
        UiRect {
            left: self.highlight_inset_x,
            top: row.top,
            right: self.panel_width - self.highlight_inset_x,
            bottom: row.bottom,
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

    /// 行内拼音行矩形（T-126）。
    ///
    /// 左缘与主文本列严格对齐（序号、拼音、汉字同一左缘，上下对齐关系
    /// 稳定）；宽度取「主文本列宽」与「拼音内容估算宽」的较大者（不超行
    /// 右缘）——拼音比汉字窄时与汉字同宽，拼音比汉字宽时完整容纳不截断，
    /// 拼音因此不会"乱跑"。顶边=行顶，底边=顶边+拼音字高（**纯字形区**，
    /// 不含间距：拼音-汉字视觉间距由主文本区顶边 = 本矩形底 + `pin_line_gap`
    /// 提供，`DT_VCENTER` 不会被带高吸收，间距精确可测）。
    #[must_use]
    pub fn pin_row_rect(&self, row: UiRect, main_col: UiRect, pin: &str) -> UiRect {
        let pin_width = estimate_text_width(pin, self.pin_font_height)
            .ceil()
            .max(1.0) as i32;
        let right = main_col
            .left
            .saturating_add(pin_width)
            .max(main_col.right)
            .min(row.right)
            .max(main_col.left + 1);
        let glyph_bottom = row.top.saturating_add(self.pin_font_height);
        UiRect {
            left: main_col.left,
            top: row.top,
            right,
            bottom: glyph_bottom.min(row.bottom).max(row.top + 1),
        }
    }

    /// 行内按内容动态划分主文本与译文矩形（T-037）。
    ///
    /// 译文不再固定占用右侧 1/3 列，而是**紧跟主文本估算宽度之后**（含
    /// `translation_gap` 间距）一直延伸到行尾：英文译文因此更靠左、可用宽度大
    /// 幅增加（长词不易截断）。主文本宽度按 `estimate_text_width` 估算，
    /// 有译文时上限保证译文区（含间距）至少占可用宽的 1/3；无译文时主文本
    /// 可占满行。
    #[must_use]
    pub fn row_split(&self, row: UiRect, main: &str, translation: &str) -> (UiRect, UiRect) {
        let marker_right = (row.left + self.marker_width).min(row.right);
        let usable = (row.width() - self.marker_width).max(0);
        let min_translation = if translation.is_empty() {
            1
        } else {
            (usable / 3).max(1)
        };
        let max_main = (usable - min_translation - self.translation_gap).max(1);
        let main_width = estimate_text_width(main, self.font_height).round().max(1.0) as i32;
        let main_width = main_width.clamp(1, max_main);
        let text = UiRect {
            left: marker_right,
            top: row.top,
            right: marker_right + main_width,
            bottom: row.bottom,
        };
        let translation_left = (text.right + self.translation_gap)
            .min(row.right.saturating_sub(1))
            .max(text.right);
        let translation = UiRect {
            left: translation_left,
            top: row.top,
            right: row.right,
            bottom: row.bottom,
        };
        // `translation` 参数为镜像绘制语义保留：主文本与译文区由间距隔开，
        // 译文为空时布局同样成立（右侧留白，供后续按内容定制）。
        (text, translation)
    }
}

/// 候选序号标记：纯数字（无圈无点），翻页后 10/11 等样式一致。
#[must_use]
pub fn index_marker(index: usize) -> String {
    format!("{}", index + 1)
}

/// 页脚翻页指示 `m/n`（当前页+1 / 总页数）；总页数 ≤1 时返回 `None`（T-040）。
#[must_use]
pub fn page_footer_label(page: usize, page_count: usize) -> Option<String> {
    if page_count <= 1 {
        return None;
    }
    let page = page.min(page_count.saturating_sub(1));
    Some(format!("{}/{}", page + 1, page_count))
}

fn bgr_to_rgb(color: u32) -> u32 {
    (color & 0xFF) << 16 | (color & 0x00FF00) | (color >> 16)
}

#[cfg(test)]
mod tests {
    use super::{
        display_main_text, estimate_text_width, index_marker, page_footer_label, theme,
        theme_from_system_colors, theme_with_candidate, CandidateMetrics, CandidateUiItem,
        CandidateUiView, SystemColors, UiColor, UiThemeKind, BASE_DPI, DEFAULT_PAGE_SIZE,
        SLANG_LABEL,
    };
    use zhu_ye_core::candidate::CandidateSource;
    use zhu_ye_ui::fit_text;

    fn view_with(items: usize) -> CandidateUiView {
        CandidateUiView {
            composition: "nihao".to_owned(),
            pinyin_hint: "ni hao".to_owned(),
            page: 0,
            page_size: DEFAULT_PAGE_SIZE,
            page_count: 1,
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
                    pinyin: String::new(),
                    pinyin_tone: String::new(),
                    source: CandidateSource::Static,
                })
                .collect(),
        }
    }

    fn slang_item(text: &str, translation: &str) -> CandidateUiItem {
        CandidateUiItem {
            text: text.to_owned(),
            translation: translation.to_owned(),
            pinyin: String::new(),
            pinyin_tone: String::new(),
            source: CandidateSource::Slang,
        }
    }

    #[test]
    fn 网络语候选主文本追加标注() {
        let item = slang_item("永远的神", "");
        assert_eq!(
            display_main_text(&item, false),
            format!("永远的神 {SLANG_LABEL}")
        );
    }

    #[test]
    fn 静态候选不加标注() {
        let item = CandidateUiItem {
            text: "你好".to_owned(),
            translation: "hello".to_owned(),
            pinyin: "nihao".to_owned(),
            pinyin_tone: "nǐ hǎo".to_owned(),
            source: CandidateSource::Static,
        };
        assert_eq!(display_main_text(&item, false), "你好");
        // 用户词同样不加标注。
        let user = CandidateUiItem {
            source: CandidateSource::User,
            ..item.clone()
        };
        assert_eq!(display_main_text(&user, false), "你好");
    }

    #[test]
    fn 译文层展示译文不加网络标注() {
        let item = slang_item("永远的神", "GOAT");
        // 译文层以译文为主文本，不应追加标注。
        assert_eq!(display_main_text(&item, true), "GOAT");
        // 译文层但无译文时回落中文文本并保留标注。
        let no_translation = slang_item("永远的神", "");
        assert_eq!(
            display_main_text(&no_translation, true),
            format!("永远的神 {SLANG_LABEL}")
        );
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
    fn 主题文件候选节叠加缺键回退() {
        use zhu_ye_core::{CandidatePalette, ThemeColor};
        let base = theme(UiThemeKind::Light);
        let palette = CandidatePalette {
            background: Some(ThemeColor(0x11_22_33)),
            marker: Some(ThemeColor(0xAA_BB_CC)),
            ..CandidatePalette::default()
        };
        let applied = theme_with_candidate(base, &palette);
        assert_eq!(applied.background, UiColor(0x11_22_33));
        assert_eq!(applied.marker, UiColor(0xAA_BB_CC));
        // 未写键保留基础值（缺键用默认值）。
        assert_eq!(applied.foreground, base.foreground);
        assert_eq!(applied.highlight_background, base.highlight_background);
        // 空节叠加等于原样。
        assert_eq!(
            theme_with_candidate(base, &CandidatePalette::default()),
            base
        );
        // 深色基础 + 单键覆盖。
        let dark_base = theme(UiThemeKind::Dark);
        let one_key = CandidatePalette {
            foreground: Some(ThemeColor(0xFF_EE_00)),
            ..CandidatePalette::default()
        };
        let applied_dark = theme_with_candidate(dark_base, &one_key);
        assert_eq!(applied_dark.foreground, UiColor(0xFF_EE_00));
        assert_eq!(applied_dark.background, dark_base.background);
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
        // T-043：选中块内缩量固定 1 物理像素（对齐 1px 边框线），不随 DPI 缩放。
        assert_eq!(scaled.highlight_inset_x, base.highlight_inset_x);
        assert_eq!(scaled.highlight_rect(0).left, 1);
        assert_eq!(
            scaled.highlight_rect(0).right,
            scaled.panel_width - scaled.highlight_inset_x
        );
    }

    #[test]
    fn 零行面板只含页眉条() {
        let metrics = CandidateMetrics::new(BASE_DPI);
        let (width, height) = metrics.panel_size(0);
        assert_eq!(width, metrics.panel_width);
        assert_eq!(height, metrics.header_height + metrics.padding_y * 2);
        // 一行面板比零行面板多一个整行 + 页脚条（有候选行才显示页脚，T-040）。
        let (_, one) = metrics.panel_size(1);
        assert_eq!(one - height, metrics.row_height + metrics.footer_height);
        // 与既有九行尺寸一致（回归保护）；T-040 起含页脚条。
        // T-126：页眉 38→48、行高 36→48、行距 2→4、内边距 10→12
        // → 九行面板 398 → 536（+138dp：输入缓冲区与候选行呼吸空间）。
        assert_eq!(metrics.panel_size(9).1, 536 + metrics.footer_height);
        assert_eq!(metrics.panel_size(9).0, 360);
        // 零行面板不占页脚空间（T-031 页眉条行为保持）。
        assert_eq!(
            metrics.panel_size(0).1,
            metrics.panel_size(1).1 - metrics.row_height - metrics.footer_height
        );
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
            let (text, translation) = metrics.row_split(rect, "你好", "hello");
            assert!(text.left == rect.left + metrics.marker_width);
            assert!(text.right <= translation.left);
            assert!(translation.right == rect.right);
        }
    }

    #[test]
    fn 选中块贴面板左右边框且序号留内边距() {
        let metrics = CandidateMetrics::new(96);
        for index in 0..DEFAULT_PAGE_SIZE {
            let row = metrics.row_rect(index);
            let highlight = metrics.highlight_rect(index);
            // 上下与行一致。
            assert_eq!(highlight.top, row.top);
            assert_eq!(highlight.bottom, row.bottom);
            // 左右几乎与面板边框齐平（仅保留 1px 边框线内缩），且比内容行更靠边框。
            assert_eq!(highlight.left, metrics.highlight_inset_x);
            assert_eq!(
                highlight.right,
                metrics.panel_width - metrics.highlight_inset_x
            );
            assert!(
                highlight.left < row.left,
                "高亮块左缘应比内容行左缘更靠边框"
            );
            assert!(
                highlight.right > row.right,
                "高亮块右缘应比内容行右缘更靠边框"
            );
            // 序号列相对高亮块左缘保留足量内边距（不再贴高亮块边缘）。
            let marker = metrics.marker_rect(row);
            assert!(
                marker.left - highlight.left >= 10,
                "序号列起点相对高亮块左缘应至少留 10px"
            );
        }
    }

    #[test]
    fn 页眉输入串与提示不重叠且译文紧跟主文本() {
        let metrics = CandidateMetrics::new(96);
        let text = metrics.header_text_rect();
        let hint = metrics.header_hint_rect();
        assert!(text.right <= hint.left);
        assert!(text.left < text.right);
        assert!(hint.left < hint.right);
        let row = metrics.row_rect(0);
        let (main, translation) = metrics.row_split(row, "你好", "hello world");
        // 译文紧跟主文本（恰为间距，而非旧的固定 1/3 右列）。
        assert_eq!(translation.left - main.right, metrics.translation_gap);
        assert!(main.left - row.left == metrics.marker_width);
        // 序号列收窄于旧值（T-037 回归保护：候选词更贴近序号）。
        assert!(metrics.marker_width < 34);
    }

    #[test]
    fn 长拼音输入时窗口自适应扩展且输入串完整() {
        // T-124：固定 360dp 面板下，长拼音输入串 + 带调提示并存时输入串区
        // 被提示区挤压（约 141dp），17 字符 ASCII 拼音（估算 ≈150px @16px
        // 字高）放不下 → `DT_END_ELLIPSIS` 右侧省略号；此后窗口按内容扩展。
        let metrics = CandidateMetrics::new(96);
        let long = "youmeiyoushenmeren";
        let long_hint = "yǒu méi yǒu shén me rén";
        let est = estimate_text_width(long, metrics.font_height).ceil() as i32;
        let (main_w, hint_alloc) = metrics.header_widths(long, long_hint);
        assert!(hint_alloc >= 1, "提示应分配非零宽度");
        let (width, _) = metrics.panel_size_for(1, long, long_hint);
        assert!(width > metrics.panel_width, "长输入+提示应扩展面板宽度");
        // 与 paint 一致：布局按实际客户区宽度（with_panel_width）。
        let layout = metrics.with_panel_width(width);
        let main_rect = layout.header_text_rect_with(hint_alloc);
        assert!(main_rect.width() >= est, "输入串区应容纳完整拼音");
        assert!(main_w == est);
        let hint_rect = layout.header_hint_rect_with(hint_alloc);
        assert!(main_rect.right <= hint_rect.left, "输入串与提示不得重叠");
        assert!(hint_rect.left < hint_rect.right);
    }

    #[test]
    fn 页眉提示按内容分配且输入串完整优先() {
        let metrics = CandidateMetrics::new(96);
        let long = "youmeiyoushenmeren";
        let long_hint = "yǒu méi yǒu shén me rén";
        let hint_full = estimate_text_width(long_hint, metrics.font_height).ceil() as i32;
        let gap = metrics.translation_gap.max(1);
        let (main_w, hint_alloc) = metrics.header_widths(long, long_hint);
        assert!(hint_alloc >= 1, "提示应分配非零宽度");
        assert!(
            hint_alloc <= (main_w + gap + hint_full) * 2 / 3,
            "提示不得挤占输入串至 1/3 以下"
        );
        // 提示未超上限时按内容完整分配（不无故压缩提示）。
        assert!(
            hint_alloc >= hint_full.min(main_w + gap + hint_full),
            "未超限的提示应完整分配"
        );
        let (width, _) = metrics.panel_size_for(1, long, long_hint);
        let layout = metrics.with_panel_width(width);
        let text = layout.header_text_rect_with(hint_alloc);
        assert!(
            text.width() >= estimate_text_width(long, metrics.font_height) as i32,
            "输入串完整容纳"
        );
        let hint = layout.header_hint_rect_with(hint_alloc);
        assert!(text.right <= hint.left, "输入串与提示不得重叠");
        assert!(hint.left < hint.right);
        // 短输入 + 短提示：保持固定面板宽，提示完整分配。
        let (main2, hint2) = metrics.header_widths("你好", "nǐ hǎo");
        let need2 = main2 + gap + hint2 + metrics.padding_x * 2;
        let (width, _) = metrics.panel_size_for(1, "你好", "nǐ hǎo");
        assert_eq!(width, metrics.panel_width, "短输入保持固定宽");
        assert!(need2 <= metrics.panel_width, "短输入提示完整放得下");
    }

    #[test]
    fn 动态分栏译文紧跟主文本且保底宽度() {
        let metrics = CandidateMetrics::new(96);
        let row = metrics.row_rect(0);
        let usable = row.width() - metrics.marker_width;
        let min_translation = usable / 3;
        let max_main = usable - min_translation - metrics.translation_gap;

        // 短主文本：译文紧跟估算宽度之后。
        let (main, translation) = metrics.row_split(row, "你好", "how are you");
        assert!(
            main.right - main.left <= estimate_text_width("你好", metrics.font_height) as i32 + 1
        );
        assert_eq!(translation.left - main.right, metrics.translation_gap);

        // 超长主文本：主文本封顶，译文保底 1/3。
        let long = "这是一个特别长的词条用于测试主文本宽度上限不会挤占译文区".repeat(3);
        let (main, translation) = metrics.row_split(row, &long, "translation");
        assert_eq!(main.width(), max_main);
        assert!(translation.width() >= min_translation);
        assert!(translation.right == row.right);

        // 无译文：主文本可占满行（不再保留右侧留白）。
        let (main, translation) = metrics.row_split(row, &long, "");
        assert_eq!(translation.left - main.right, metrics.translation_gap);
        assert!(translation.width() >= 1);
    }

    #[test]
    fn 页脚页码指示仅在多页时显示且格式为mn() {
        // 单页无指示。
        assert_eq!(page_footer_label(0, 1), None);
        // 多页：当前页 +1 / 总页数。
        assert_eq!(page_footer_label(0, 2).as_deref(), Some("1/2"));
        assert_eq!(page_footer_label(1, 2).as_deref(), Some("2/2"));
        assert_eq!(page_footer_label(1, 3).as_deref(), Some("2/3"));
        // 页码越界时收敛到末页。
        assert_eq!(page_footer_label(9, 3).as_deref(), Some("3/3"));
        // 视图快照联动。
        let mut view = view_with(20);
        assert_eq!(view.footer_label(), None);
        view.page_count = 3;
        view.page = 1;
        assert_eq!(view.footer_label().as_deref(), Some("2/3"));
    }

    #[test]
    fn 页脚矩形位于最后一行下方且不与行重叠() {
        let metrics = CandidateMetrics::new(96);
        for rows in [1usize, 5, 9] {
            let last_row = metrics.row_rect(rows - 1);
            let footer = metrics.footer_rect(rows);
            assert!(footer.top >= last_row.bottom, "rows={rows}");
            assert!(footer.bottom <= metrics.panel_size(rows).1 - metrics.padding_y);
            assert!(footer.left < footer.right);
            assert!(footer.height() == metrics.footer_height);
        }
    }

    #[test]
    fn 序号纯数字无圈无点且翻页后样式一致() {
        for index in 0..=13 {
            let marker = index_marker(index);
            assert_eq!(marker, format!("{}", index + 1), "序号 {index}");
            assert!(!marker.contains('①') && !marker.contains('⑨'));
            assert!(!marker.ends_with('.'));
        }
    }

    #[test]
    fn 长文本按宽度截断且保留省略号() {
        let long_cjk = "这是一个特别长的候选词用来验证文本不会溢出窗口边界".repeat(3);
        let fitted = fit_text(&long_cjk, 180, 16);
        assert!(fitted.ends_with('…'));
        assert!(fitted.chars().count() < long_cjk.chars().count());

        let short = "你好";
        assert_eq!(fit_text(short, 180, 16), short);
        assert_eq!(index_marker(0), "1");
        assert_eq!(index_marker(9), "10");
    }

    #[test]
    fn 搜狗风色板结构与强调关系() {
        let light = theme(UiThemeKind::Light);
        assert_eq!(light.background, UiColor::rgb(0xFF, 0xFF, 0xFF));
        assert_eq!(light.highlight_background, UiColor::rgb(0xE6, 0xF2, 0xFE));
        // T-032：蓝框蓝字、选中红字。
        assert_eq!(light.border, UiColor::rgb(0x1E, 0x88, 0xE5));
        assert_eq!(light.foreground, UiColor::rgb(0x1E, 0x88, 0xE5));
        assert_eq!(light.highlight_foreground, UiColor::rgb(0xD3, 0x2F, 0x2F));
        // 序号/译文浅灰：与主文本和选中块均可区分。
        assert_eq!(light.marker, light.secondary);
        assert_ne!(light.marker, light.foreground);

        let dark = theme(UiThemeKind::Dark);
        assert_eq!(dark.border, UiColor::rgb(0x42, 0xA5, 0xF5));
        assert_eq!(dark.foreground, UiColor::rgb(0x64, 0xB5, 0xF6));
        assert_eq!(dark.highlight_foreground, UiColor::rgb(0xFF, 0x8A, 0x80));
        assert_eq!(dark.marker, dark.secondary);
        assert_ne!(dark.marker, dark.foreground);
        // T-126：浅色主题拼音行 = 用户指定辅助提示淡灰 #888888。
        assert_eq!(light.pin, UiColor::rgb(0x88, 0x88, 0x88));
    }

    #[test]
    fn 拼音行色独立于序号译文且对比更强() {
        // T-122：拼音行用专用加深灰——浅色底下比 secondary 更暗、深色底下更亮，
        // 保证小字可读；高对比度沿用系统 gray_text（与译文同源）。
        let light = theme(UiThemeKind::Light);
        let dark = theme(UiThemeKind::Dark);
        assert_ne!(light.pin, light.secondary, "拼音行不得沿用译文浅灰");
        assert_ne!(dark.pin, dark.secondary, "拼音行不得沿用译文浅灰");
        // 浅色底：pin 应比 secondary 更暗（灰度分量更低）。
        assert!((light.pin.0 >> 16) < (light.secondary.0 >> 16));
        assert!((light.pin.0 & 0xFF) < (light.secondary.0 & 0xFF));
        // 深色底：pin 应比 secondary 更亮。
        assert!(
            (dark.pin.0 & 0xFF) > (dark.secondary.0 & 0xFF),
            "深色底拼音行应亮于译文/序号"
        );
    }

    #[test]
    fn 序号与候选词间距稳定在四到八像素() {
        // T-122：数字右对齐于标记列右缘并让出 marker_text_gap，96dpi 下
        // 单/双位数字与候选词间距都落在用户要求的 4-8px 档。
        let metrics = CandidateMetrics::new(BASE_DPI);
        assert!(
            (4..=8).contains(&metrics.marker_text_gap),
            "marker_text_gap 应在 4-8px，实际 {}",
            metrics.marker_text_gap
        );
        // 词起点比旧值（26px）明显左移，且标记列仍留有双位数字空间。
        assert!(metrics.marker_width < 26, "词起点应比旧版更靠左");
        assert!(metrics.marker_width - metrics.marker_text_gap >= 14);
        // 拼音行字号提升且仍小于主文本（层级：主文本 > 拼音）。
        assert!(metrics.pin_font_height > 12, "拼音行应大于 12px");
        assert!(metrics.pin_font_height < metrics.font_height);
        // T-126：拼音与汉字垂直间距先取 4-8px 档上限 8px，用户随后要求
        // "拼音和中文距离减少一半"→ 定稿 4px（档内偏紧凑）。
        assert!(
            (4..=8).contains(&metrics.pin_line_gap),
            "拼音行与主文本间距应在 4-8px，实际 {}",
            metrics.pin_line_gap
        );
        assert_eq!(metrics.pin_line_gap, 4, "用户要求间距减半，定稿 4px");
        // 拼音带（字号+行距）不应挤占主文本区：行高 48 ≥ 拼音带 + 16px 主字。
        let pin_band = metrics.pin_font_height + metrics.pin_line_gap;
        assert!(metrics.row_height - pin_band >= metrics.font_height);
    }

    #[test]
    fn 拼音与主文本对齐且行内留呼吸空间() {
        // T-126 ①④：拼音字形区从行顶开始、高度=字高（不含间距），主文本区
        // 顶边 = 拼音字形区底 + pin_line_gap —— 视觉间距精确等于 pin_line_gap
        // （`DT_VCENTER` 无居中余量可吸收）；行内剩余高度作为主文本下方留白，
        // 让候选项不拥挤。
        let metrics = CandidateMetrics::new(BASE_DPI);
        let row = metrics.row_rect(0);
        let (main_col, _) = metrics.row_split(row, "你好", "");
        let pin_rect = metrics.pin_row_rect(row, main_col, "nǐ hǎo");
        // 拼音行与汉字列同左缘（上下严格对齐）。
        assert_eq!(pin_rect.left, main_col.left);
        // 拼音字形区底 = 行顶 + 字高（间距不含在拼音矩形内）。
        assert_eq!(pin_rect.bottom, row.top + metrics.pin_font_height);
        // 拼音宽于汉字时右缘 = 内容估算宽（按内容放宽、不截断），
        // 左缘保持与汉字列对齐（上下严格对齐关系）。
        let est = estimate_text_width("nǐ hǎo", metrics.pin_font_height).ceil() as i32;
        assert_eq!(pin_rect.right, main_col.left + est);
        assert!(pin_rect.right >= main_col.right, "拼音更宽时完整容纳");
        // 拼音窄于汉字时与汉字列同宽（宽度对齐，拼音不乱跑）。
        let short = metrics.pin_row_rect(row, main_col, "ni");
        assert_eq!(short.right, main_col.right);
        // 主文本区顶边 = 拼音字形区底 + pin_line_gap（视觉间距精确 = 4px）。
        let main_top = pin_rect.bottom + metrics.pin_line_gap;
        assert!(main_top - row.top >= metrics.pin_font_height + 4);
        assert_eq!(
            main_top - row.top,
            metrics.pin_font_height + metrics.pin_line_gap
        );
        // 行内呼吸：主文本字形带下方仍留有 ≥ 主字号 + 4px 的行内空间。
        let main_zone = row.bottom - main_top;
        assert!(main_zone >= metrics.font_height + 4, "行内应留呼吸空间");
        // 整个拼音字形区不越出行底。
        assert!(pin_rect.bottom <= row.bottom);
    }

    #[test]
    fn 长拼音超出汉字宽度时完整容纳不截断() {
        // T-126 ④：拼音宽度取 max(汉字列宽, 拼音估算宽)——"zhang hao" 比
        // 两个汉字宽时按拼音内容放宽（不截断），且不越过行右缘。
        let metrics = CandidateMetrics::new(BASE_DPI);
        let row = metrics.row_rect(0);
        let (main_col, _) = metrics.row_split(row, "你好", "");
        let pin_rect = metrics.pin_row_rect(row, main_col, "zhang hao");
        assert_eq!(pin_rect.left, main_col.left);
        assert!(pin_rect.right > main_col.right, "长拼音应按内容放宽");
        assert!(pin_rect.right <= row.right, "拼音不得越出行右缘");
        // 行右缘边界：超行宽时收缩到行右缘（不溢出窗口）。
        let (wide_main, _) = metrics.row_split(row, "这是一个特别长的词条用于测试", "");
        let edge = metrics.pin_row_rect(row, wide_main, "zhang hao shi yin");
        assert!(edge.right <= row.right);
        assert!(edge.right >= edge.left);
    }
}
