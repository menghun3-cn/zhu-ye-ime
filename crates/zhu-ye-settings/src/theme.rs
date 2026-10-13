//! 设置窗口配色（纯逻辑，无 Win32 依赖）。
//!
//! 与候选窗配色相互独立：候选窗受 T-030 约束固定浅色（不读系统深浅色），设置窗口是常规
//! 桌面窗口，跟随系统深浅色（D-30）。两个窗口服务不同任务，配色口径不共享，只共享
//! `zhu-ye-ui` 原语 crate 的颜色与主题种类类型（T-081 抽取，S-10 收尾）。

use zhu_ye_ui::{SystemColors, UiColor, UiThemeKind};

/// 设置窗口配色。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingsTheme {
    /// 窗口背景。
    pub window: UiColor,
    /// 左侧导航背景。
    pub nav_background: UiColor,
    /// 导航选中项背景。
    pub nav_selected: UiColor,
    /// 导航选中项文字。
    pub nav_selected_text: UiColor,
    /// 导航未选中项文字。
    pub nav_text: UiColor,
    /// 页标题文字。
    pub title_text: UiColor,
    /// 条目标题文字。
    pub item_text: UiColor,
    /// 条目说明与次要文字。
    pub secondary_text: UiColor,
    /// 分隔线与边框。
    pub border: UiColor,
    /// 强调色（选中控件）。
    pub accent: UiColor,
    /// 控件底色（未选中）。
    pub control_background: UiColor,
    /// 控件选中底色。
    pub control_selected: UiColor,
    /// 控件选中文字。
    pub control_selected_text: UiColor,
    /// 占位说明文字颜色（兼容旧主题文件；新界面占位呈现改标签式，见字段 `tag_*`）。
    pub placeholder_text: UiColor,
    /// 异常/警示文字（注册状态异常、修复建议）。
    pub warn_text: UiColor,
    /// 状态·正常文字（注册状态正常、在线更新已开启）。
    pub ok_text: UiColor,
    /// 实叶底上的前景：实底按钮与面板选中格字色（浅色白 `#FFFFFF` / 深色墨
    /// `#10241A`——深色白字对比仅 2.3:1 被否决，墨字 7.15:1，S-20）。
    pub on_accent: UiColor,
    /// chips 与描边按钮的细描边（1px）。
    pub chip_border: UiColor,
    /// 「规划中」标签底色。
    pub tag_bg: UiColor,
    /// 「规划中」标签文字色。
    pub tag_text: UiColor,
    /// 新芽记号圆点（仅「规划中」标签）。
    pub sprout: UiColor,
    /// 占位展开说明区底色。
    pub expanded_bg: UiColor,
    /// 品牌块竹节棕细线（品牌固定色，不进主题文件，T-148 §9）。
    pub bark: UiColor,
}

/// 返回预设配色。
#[must_use]
pub fn settings_theme(kind: UiThemeKind) -> SettingsTheme {
    match kind {
        UiThemeKind::Light => SettingsTheme {
            window: UiColor::rgb(0xF7, 0xF9, 0xF4),         // paper 竹纸
            nav_background: UiColor::rgb(0xEF, 0xF3, 0xEA), // rail
            nav_selected: UiColor::rgb(0xDF, 0xE9, 0xDB),   // rail_sel
            nav_selected_text: UiColor::rgb(0x27, 0x5A, 0x38), // leaf_deep
            nav_text: UiColor::rgb(0x3A, 0x46, 0x3E),       // nav_text
            title_text: UiColor::rgb(0x22, 0x30, 0x2A),     // ink
            item_text: UiColor::rgb(0x22, 0x30, 0x2A),      // ink
            secondary_text: UiColor::rgb(0x5C, 0x6B, 0x60), // secondary
            border: UiColor::rgb(0xDD, 0xE5, 0xD6),         // line
            accent: UiColor::rgb(0x3D, 0x7A, 0x4E),         // leaf
            control_background: UiColor::rgb(0xFF, 0xFF, 0xFF), // chip_bg
            control_selected: UiColor::rgb(0xDF, 0xE9, 0xDB), // rail_sel（浅底深字选中语言）
            control_selected_text: UiColor::rgb(0x27, 0x5A, 0x38), // leaf_deep
            placeholder_text: UiColor::rgb(0x4A, 0x6E, 0x28), // 兼容旧文件：= tag_text
            warn_text: UiColor::rgb(0xC5, 0x0F, 0x1F),      // warn
            ok_text: UiColor::rgb(0x2C, 0x6E, 0x3F),        // ok
            on_accent: UiColor::rgb(0xFF, 0xFF, 0xFF),      // 实叶底白字（5.13:1）
            chip_border: UiColor::rgb(0xC4, 0xCF, 0xBB),    // chip_border
            tag_bg: UiColor::rgb(0xEE, 0xF3, 0xE4),         // tag_bg
            tag_text: UiColor::rgb(0x4A, 0x6E, 0x28),       // tag_text
            sprout: UiColor::rgb(0x9A, 0xC3, 0x6A),         // sprout
            expanded_bg: UiColor::rgb(0xF1, 0xF4, 0xEC),    // 占位展开底
            bark: UiColor::rgb(0x9C, 0x7B, 0x4F),           // bark 竹节棕
        },
        UiThemeKind::Dark => SettingsTheme {
            window: UiColor::rgb(0x1F, 0x28, 0x22),         // paper 墨绿黑
            nav_background: UiColor::rgb(0x19, 0x20, 0x19), // rail
            nav_selected: UiColor::rgb(0x2A, 0x3A, 0x2F),   // rail_sel
            nav_selected_text: UiColor::rgb(0x9B, 0xD6, 0xA6), // leaf_deep 亮叶
            nav_text: UiColor::rgb(0xC6, 0xD0, 0xC8),       // nav_text
            title_text: UiColor::rgb(0xE8, 0xEF, 0xE8),     // ink
            item_text: UiColor::rgb(0xE8, 0xEF, 0xE8),      // ink
            secondary_text: UiColor::rgb(0x9F, 0xB0, 0xA3), // secondary
            border: UiColor::rgb(0x35, 0x40, 0x3A),         // line
            accent: UiColor::rgb(0x80, 0xB9, 0x8A),         // leaf 提亮
            control_background: UiColor::rgb(0x24, 0x30, 0x2A), // chip_bg
            control_selected: UiColor::rgb(0x2A, 0x3A, 0x2F), // rail_sel
            control_selected_text: UiColor::rgb(0x9B, 0xD6, 0xA6), // leaf_deep
            placeholder_text: UiColor::rgb(0xB5, 0xD3, 0x9B), // 兼容旧文件：= tag_text
            warn_text: UiColor::rgb(0xFF, 0x9A, 0x8C),      // warn
            ok_text: UiColor::rgb(0x8F, 0xD6, 0xA0),        // ok
            on_accent: UiColor::rgb(0x10, 0x24, 0x1A),      // 深色实叶底墨字（7.15:1）
            chip_border: UiColor::rgb(0x4A, 0x5A, 0x4F),    // chip_border
            tag_bg: UiColor::rgb(0x27, 0x33, 0x2B),         // tag_bg
            tag_text: UiColor::rgb(0xB5, 0xD3, 0x9B),       // tag_text
            sprout: UiColor::rgb(0x9A, 0xC3, 0x6A),         // sprout 两态同值
            expanded_bg: UiColor::rgb(0x24, 0x2E, 0x27),    // 占位展开底
            bark: UiColor::rgb(0xB9, 0x9B, 0x6D),           // bark 竹节棕提亮
        },
        UiThemeKind::HighContrast => settings_theme_from_system_colors(SystemColors {
            window: 0xFF00_0000,
            window_text: 0x00FF_FFFF,
            gray_text: 0x00FF_FFFF,
            highlight: 0x00FF_FF00,
            highlight_text: 0x0000_0000,
            btn_face: 0x0000_0000,
        }),
    }
}

impl SettingsTheme {
    /// 用自定义主题文件覆盖预设配色（T-088 / FR-048）：文件里给了的键覆盖，
    /// 缺键（或键值解析失败）回退 `self`（调用方传入的当前深浅预设）；高对比度
    /// 由系统接管，调用方在 `HighContrast` 时不调用本函数。
    ///
    /// 返回 `(覆盖后的配色, 实际覆盖的键数)`，键数供界面提示"应用了 N/22 键"。
    /// 品牌块竹节棕 `bark` 属品牌固定色，不参与主题文件覆盖。
    #[must_use]
    pub fn with_theme_file(self, file: &zhu_ye_core::ThemeFile) -> (SettingsTheme, usize) {
        // `ThemeColor(u32)` 与 `UiColor(u32)` 同为 0xRRGGBB 布局，可直接映射。
        let mut covered = 0usize;
        let mut overlay = |base: UiColor, color: Option<zhu_ye_core::ThemeColor>| {
            if let Some(color) = color {
                covered += 1;
                UiColor(color.0)
            } else {
                base
            }
        };
        let theme = SettingsTheme {
            window: overlay(self.window, file.settings.window),
            nav_background: overlay(self.nav_background, file.settings.nav_background),
            nav_selected: overlay(self.nav_selected, file.settings.nav_selected),
            nav_selected_text: overlay(self.nav_selected_text, file.settings.nav_selected_text),
            nav_text: overlay(self.nav_text, file.settings.nav_text),
            title_text: overlay(self.title_text, file.settings.title_text),
            item_text: overlay(self.item_text, file.settings.item_text),
            secondary_text: overlay(self.secondary_text, file.settings.secondary_text),
            border: overlay(self.border, file.settings.border),
            accent: overlay(self.accent, file.settings.accent),
            control_background: overlay(self.control_background, file.settings.control_background),
            control_selected: overlay(self.control_selected, file.settings.control_selected),
            control_selected_text: overlay(
                self.control_selected_text,
                file.settings.control_selected_text,
            ),
            placeholder_text: overlay(self.placeholder_text, file.settings.placeholder_text),
            warn_text: overlay(self.warn_text, file.settings.warn_text),
            // T-148 视觉改版新增键：可选，缺键回退当前深浅预设（S-25）。
            ok_text: overlay(self.ok_text, file.settings.ok_text),
            on_accent: overlay(self.on_accent, file.settings.on_accent),
            chip_border: overlay(self.chip_border, file.settings.chip_border),
            tag_bg: overlay(self.tag_bg, file.settings.tag_bg),
            tag_text: overlay(self.tag_text, file.settings.tag_text),
            sprout: overlay(self.sprout, file.settings.sprout),
            expanded_bg: overlay(self.expanded_bg, file.settings.expanded_bg),
            // bark 不随主题文件覆盖。
            bark: self.bark,
        };
        (theme, covered)
    }
}

/// 按系统高对比度配色构建设置窗口配色；系统色为 `0x00BBGGRR` 字节序。
#[must_use]
pub fn settings_theme_from_system_colors(colors: SystemColors) -> SettingsTheme {
    // 高对比度映射口径与 `candidate_ui::theme_from_system_colors` 完全一致（T-081
    // 抽取后本地化，设置窗口不再依赖 TSF 侧代码）：窗口/控件底=window，选中块=
    // highlight（文字取 highlight_text），主文字=window_text，弱化=gray_text，
    // 边框=btn_face，警示/占位沿用选中文字色。字节序 BGR(0x00BBGGRR)→RGB。
    let bgr_to_rgb =
        |color: u32| UiColor(((color & 0xFF) << 16) | (color & 0x00FF00) | (color >> 16));
    let window = bgr_to_rgb(colors.window);
    let foreground = bgr_to_rgb(colors.window_text);
    let secondary = bgr_to_rgb(colors.gray_text);
    let border = bgr_to_rgb(colors.btn_face);
    let highlight_background = bgr_to_rgb(colors.highlight);
    let highlight_foreground = bgr_to_rgb(colors.highlight_text);
    SettingsTheme {
        window,
        nav_background: window,
        nav_selected: highlight_background,
        nav_selected_text: highlight_foreground,
        nav_text: foreground,
        title_text: foreground,
        item_text: foreground,
        secondary_text: secondary,
        border,
        accent: highlight_background,
        control_background: window,
        control_selected: highlight_background,
        control_selected_text: highlight_foreground,
        placeholder_text: foreground,
        warn_text: highlight_foreground,
        // T-148 新字段高对比口径：全部落在系统色上（D-30/D-31 由系统接管）。
        ok_text: foreground,
        on_accent: highlight_foreground,
        chip_border: border,
        tag_bg: window,
        tag_text: foreground,
        sprout: foreground,
        expanded_bg: window,
        bark: border,
    }
}

#[cfg(test)]
mod tests {
    use super::{settings_theme, settings_theme_from_system_colors};
    use zhu_ye_ui::{SystemColors, UiColor, UiThemeKind};

    #[test]
    fn 浅色与深色在窗口底色上互异() {
        let light = settings_theme(UiThemeKind::Light);
        let dark = settings_theme(UiThemeKind::Dark);
        assert_ne!(light.window, dark.window);
        // 深色的每个通道都不高于浅色（确实是"深"）。
        let luma = |c: UiColor| (c.0 >> 16) + ((c.0 >> 8) & 0xFF) + (c.0 & 0xFF);
        assert!(luma(dark.window) < luma(light.window));
        assert!(luma(dark.nav_background) < luma(light.nav_background));
    }

    #[test]
    fn 选中与未选中控件颜色互异() {
        for kind in [UiThemeKind::Light, UiThemeKind::Dark] {
            let theme = settings_theme(kind);
            assert_ne!(theme.control_selected, theme.control_background);
            assert_ne!(theme.control_selected_text, theme.control_background);
            assert_ne!(theme.nav_selected, theme.nav_background);
            assert_ne!(theme.nav_selected_text, theme.nav_text);
        }
    }

    #[test]
    fn 高对比度配色取自系统色() {
        // `SystemColors` 字段是 COLORREF 的 `0x00BBGGRR` 字节序：低字节才是红。
        let colors = SystemColors {
            window: 0x0000_0000,      // 黑
            window_text: 0x00FF_FFFF, // 白
            gray_text: 0x00FF_FFFF,
            highlight: 0x0000_00FF, // 红
            highlight_text: 0x00FF_FFFF,
            btn_face: 0x0000_0000,
        };
        let theme = settings_theme_from_system_colors(colors);
        assert_eq!(theme.window, UiColor::rgb(0, 0, 0));
        assert_eq!(theme.item_text, UiColor::rgb(0xFF, 0xFF, 0xFF));
        assert_eq!(theme.nav_selected, UiColor::rgb(0xFF, 0, 0));
        assert_eq!(theme.nav_selected_text, UiColor::rgb(0xFF, 0xFF, 0xFF));
    }

    #[test]
    fn 预设高对比度与系统色路径给出同构结果() {
        // `settings_theme(HighContrast)` 只是用一个固定的高对比度色快照调用系统色路径。
        let preset = settings_theme(UiThemeKind::HighContrast);
        let explicit = settings_theme_from_system_colors(SystemColors {
            window: 0xFF00_0000,
            window_text: 0x00FF_FFFF,
            gray_text: 0x00FF_FFFF,
            highlight: 0x00FF_FF00,
            highlight_text: 0x0000_0000,
            btn_face: 0x0000_0000,
        });
        assert_eq!(preset, explicit);
        // 白色文本在两种字节序下都应为白。
        assert_eq!(preset.item_text, UiColor::rgb(0xFF, 0xFF, 0xFF));
        assert_eq!(preset.window, explicit.window);
    }

    #[test]
    fn 主题文件覆盖缺键回退预设并统计覆盖键数() {
        use zhu_ye_core::{parse_theme_file, ThemeColor};
        let light = settings_theme(UiThemeKind::Light);
        // 只给 3 键的单节自定义主题。
        let file = parse_theme_file(
            r##"{
                "version": 1,
                "name": "午夜蓝",
                "settings": {
                    "window": "#101418",
                    "nav_background": "1c2733",
                    "accent": "#42A5F5"
                }
            }"##,
        )
        .unwrap();
        let (theme, covered) = light.with_theme_file(&file);
        assert_eq!(covered, 3, "只覆盖文件里给出的 3 键");
        assert_eq!(theme.window, UiColor(ThemeColor(0x10_14_18).0));
        assert_eq!(theme.nav_background, UiColor(ThemeColor(0x1C_27_33).0));
        assert_eq!(theme.accent, UiColor(ThemeColor(0x42_A5_F5).0));
        // 缺键回退浅色预设。
        assert_eq!(theme.item_text, light.item_text);
        assert_eq!(theme.control_selected, light.control_selected);
        // 有候选节不影响设置窗（各读各节）。
        let file = parse_theme_file(
            r##"{
                "version": 1,
                "candidate": { "background": "#202020" }
            }"##,
        )
        .unwrap();
        let (theme, covered) = settings_theme(UiThemeKind::Dark).with_theme_file(&file);
        assert_eq!(covered, 0, "候选节不参与设置窗覆盖");
        assert_eq!(theme, settings_theme(UiThemeKind::Dark));
    }

    #[test]
    fn 主题文件键值非法按缺键回退() {
        use zhu_ye_core::parse_theme_file;
        let dark = settings_theme(UiThemeKind::Dark);
        // 颜色解析失败（"1234"）→ None → 回退深色预设。
        let file = parse_theme_file(
            r##"{
                "version": 1,
                "settings": { "window": "1234", "border": "#GGGGGG" }
            }"##,
        )
        .unwrap();
        let (theme, covered) = dark.with_theme_file(&file);
        assert_eq!(covered, 0);
        assert_eq!(theme, dark);
    }

    #[test]
    fn 视觉改版新字段随深浅预设且深浅互异() {
        let light = settings_theme(UiThemeKind::Light);
        let dark = settings_theme(UiThemeKind::Dark);
        // 选中语言浅底深字：chips 选中底与导航选中同源（rail_sel）。
        assert_eq!(light.control_selected, light.nav_selected);
        assert_eq!(dark.control_selected, dark.nav_selected);
        // 实叶底按钮字色：浅色白、深色墨（§4.2 对比度依据，白字在深叶上仅 2.3:1）。
        assert_eq!(light.on_accent, UiColor::rgb(0xFF, 0xFF, 0xFF));
        assert_eq!(dark.on_accent, UiColor::rgb(0x10, 0x24, 0x1A));
        // 深浅两态互异。
        assert_ne!(light.ok_text, dark.ok_text);
        assert_ne!(light.chip_border, dark.chip_border);
        assert_ne!(light.tag_bg, dark.tag_bg);
        assert_ne!(light.expanded_bg, dark.expanded_bg);
        assert_ne!(light.bark, dark.bark);
        // 新芽记号两态同值（S-20 记号色）。
        assert_eq!(light.sprout, dark.sprout);
        // 高对比全落系统色：实叶底字用 highlight 前景。
        let high = settings_theme(UiThemeKind::HighContrast);
        assert_eq!(high.on_accent, high.nav_selected_text);
        assert_eq!(high.chip_border, high.border);
        assert_eq!(high.bark, high.border);
    }

    #[test]
    fn 主题文件新键覆盖与缺键回退且bark不可覆盖() {
        use zhu_ye_core::{parse_theme_file, ThemeColor};
        let light = settings_theme(UiThemeKind::Light);
        let file = parse_theme_file(
            r##"{
                "version": 1,
                "settings": {
                    "on_accent": "#112233", "ok_text": "#445566",
                    "chip_border": "#778899", "tag_bg": "#AABBCC",
                    "tag_text": "#DDEEFF", "sprout": "#010203",
                    "expanded_bg": "#040506", "bark": "#123456"
                }
            }"##,
        )
        .unwrap();
        let (theme, covered) = light.with_theme_file(&file);
        // 7 个新字段逐一覆盖；`bark` 属品牌固定色不参与文件覆盖（不计数、不改值）。
        assert_eq!(covered, 7, "新增键 7 个，bark 不计入");
        assert_eq!(theme.on_accent, UiColor(ThemeColor(0x11_22_33).0));
        assert_eq!(theme.ok_text, UiColor(ThemeColor(0x44_55_66).0));
        assert_eq!(theme.chip_border, UiColor(ThemeColor(0x77_88_99).0));
        assert_eq!(theme.tag_bg, UiColor(ThemeColor(0xAA_BB_CC).0));
        assert_eq!(theme.tag_text, UiColor(ThemeColor(0xDD_EE_FF).0));
        assert_eq!(theme.sprout, UiColor(ThemeColor(0x01_02_03).0));
        assert_eq!(theme.expanded_bg, UiColor(ThemeColor(0x04_05_06).0));
        assert_eq!(theme.bark, light.bark, "bark 不随主题文件覆盖");
        // 缺新键回退预设（S-25 可选键兼容）。
        let bare = parse_theme_file(r#"{"version": 1, "settings": {}}"#).unwrap();
        let (theme, covered) = light.with_theme_file(&bare);
        assert_eq!(covered, 0);
        assert_eq!(theme, light);
    }
}
