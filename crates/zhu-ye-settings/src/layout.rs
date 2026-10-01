//! 设置窗口布局（纯逻辑，无 Win32 依赖）。
//!
//! 布局从 DPI 与客户区尺寸算出，绘制与命中测试共用同一份结果，避免"看得见的地方点不到"。
//! 尺寸常量按 96 DPI 的逻辑值书写，经 `scale` 换算为像素。

use zhu_ye_core::ThemeChoice;
use zhu_ye_ime::candidate_ui::{UiRect, BASE_DPI};

use crate::model::{Item, ItemControl, Page};

/// 期望客户区逻辑尺寸。
const LOGICAL_WIDTH: i32 = 880;
const LOGICAL_HEIGHT: i32 = 620;
/// 左侧导航列宽。
const LOGICAL_NAV_WIDTH: i32 = 184;
/// 导航行高。
const LOGICAL_NAV_ROW: i32 = 46;
/// 页标题区高度。
const LOGICAL_TITLE: i32 = 60;
/// 条目行高。
const LOGICAL_ITEM: i32 = 62;
/// 展开说明区高度。
const LOGICAL_EXPANDED: i32 = 46;
/// 内容区内边距。
const LOGICAL_PADDING: i32 = 22;
/// 底部提示条高度。
const LOGICAL_HINT: i32 = 34;
/// 二选一控件单块宽高。
const LOGICAL_CHIP_WIDTH: i32 = 82;
const LOGICAL_CHIP_HEIGHT: i32 = 30;
/// 控件块间距与通用小间距。
const LOGICAL_CHIP_GAP: i32 = 10;
const LOGICAL_GAP: i32 = 8;

/// 按 DPI 缩放逻辑尺寸（四舍五入）；低于 96 的 DPI 按 96 处理。
#[must_use]
pub fn scale(dpi: u32, logical: i32) -> i32 {
    let dpi = i32::try_from(dpi.max(BASE_DPI)).unwrap_or(BASE_DPI as i32);
    let base = BASE_DPI as i32;
    (logical * dpi + base / 2) / base
}

/// 布局尺寸（全部为缩放后的像素）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingsMetrics {
    /// 当前 DPI。
    pub dpi: u32,
    /// 导航列宽。
    pub nav_width: i32,
    /// 导航行高。
    pub nav_row_height: i32,
    /// 页标题区高度。
    pub title_height: i32,
    /// 条目行高。
    pub item_height: i32,
    /// 展开说明区高度。
    pub expanded_height: i32,
    /// 内边距。
    pub padding: i32,
    /// 底部提示条高度。
    pub hint_height: i32,
    /// 控件单块宽。
    pub chip_width: i32,
    /// 控件单块高。
    pub chip_height: i32,
    /// 控件块间距。
    pub chip_gap: i32,
    /// 通用小间距。
    pub gap: i32,
    /// 期望客户区宽。
    pub desired_width: i32,
    /// 期望客户区高。
    pub desired_height: i32,
}

impl SettingsMetrics {
    /// 按 DPI 建立尺寸集。
    #[must_use]
    pub fn new(dpi: u32) -> Self {
        let dpi = dpi.max(BASE_DPI);
        Self {
            dpi,
            nav_width: scale(dpi, LOGICAL_NAV_WIDTH),
            nav_row_height: scale(dpi, LOGICAL_NAV_ROW),
            title_height: scale(dpi, LOGICAL_TITLE),
            item_height: scale(dpi, LOGICAL_ITEM),
            expanded_height: scale(dpi, LOGICAL_EXPANDED),
            padding: scale(dpi, LOGICAL_PADDING),
            hint_height: scale(dpi, LOGICAL_HINT),
            chip_width: scale(dpi, LOGICAL_CHIP_WIDTH),
            chip_height: scale(dpi, LOGICAL_CHIP_HEIGHT),
            chip_gap: scale(dpi, LOGICAL_CHIP_GAP),
            gap: scale(dpi, LOGICAL_GAP),
            desired_width: scale(dpi, LOGICAL_WIDTH),
            desired_height: scale(dpi, LOGICAL_HEIGHT),
        }
    }

    /// 期望的客户区尺寸；窗口框架由调用方按 `AdjustWindowRectExForDpi` 补足。
    #[must_use]
    pub fn desired_client_size(&self) -> (i32, i32) {
        (self.desired_width, self.desired_height)
    }

    /// 内容区左边界（导航右侧再加内边距）。
    fn content_left(&self) -> i32 {
        self.nav_width + self.padding
    }
}

/// 导航行；按 `Page::ALL` 顺序从上到下排列。
#[must_use]
pub fn nav_rows(metrics: &SettingsMetrics, client: UiRect) -> Vec<(Page, UiRect)> {
    Page::ALL
        .iter()
        .copied()
        .enumerate()
        .map(|(index, page)| {
            let step = metrics.nav_row_height + metrics.gap;
            let top = client.top + metrics.padding + index as i32 * step;
            let rect = UiRect {
                left: metrics.gap,
                top,
                right: metrics.nav_width - metrics.gap,
                bottom: top + metrics.nav_row_height,
            };
            (page, rect)
        })
        .collect()
}

/// 页标题矩形。
#[must_use]
pub fn title_rect(metrics: &SettingsMetrics, client: UiRect) -> UiRect {
    let top = client.top + metrics.padding;
    UiRect {
        left: metrics.content_left(),
        top,
        right: client.right - metrics.padding,
        bottom: top + metrics.title_height,
    }
}

/// 条目内容区：标题下方到底部提示条上方。
#[must_use]
pub fn content_rect(metrics: &SettingsMetrics, client: UiRect) -> UiRect {
    UiRect {
        left: metrics.content_left(),
        top: title_rect(metrics, client).bottom,
        right: client.right - metrics.padding,
        bottom: client.bottom - metrics.padding - metrics.hint_height,
    }
}

/// 底部提示条矩形（保存结果等一次性反馈）。
#[must_use]
pub fn hint_rect(metrics: &SettingsMetrics, client: UiRect) -> UiRect {
    let bottom = client.bottom - metrics.padding / 2;
    UiRect {
        left: metrics.content_left(),
        top: bottom - metrics.hint_height,
        right: client.right - metrics.padding,
        bottom,
    }
}

/// 条目行布局结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemRow {
    /// 条目在所属页内的下标。
    pub index: usize,
    /// 行矩形（绘制与命中测试共用）。
    pub rect: UiRect,
    /// 展开的说明区；未展开为 `None`。
    pub expanded: Option<UiRect>,
    /// 二选一控件块；无控件时为空。
    pub chips: Vec<(ThemeChoice, UiRect)>,
}

/// 按顺序排列条目行；展开项追加说明区并把后续行下移。
#[must_use]
pub fn item_rows(
    metrics: &SettingsMetrics,
    client: UiRect,
    items: &[Item],
    expanded: Option<usize>,
) -> Vec<ItemRow> {
    let content = content_rect(metrics, client);
    let mut top = content.top;
    let mut rows = Vec::with_capacity(items.len());
    for (index, item) in items.iter().enumerate() {
        let rect = UiRect {
            left: content.left,
            top,
            right: content.right,
            bottom: top + metrics.item_height,
        };
        let mut next_top = rect.bottom;
        let mut expanded_rect = None;
        if expanded == Some(index) && item.state.message().is_some() {
            expanded_rect = Some(UiRect {
                left: content.left,
                top: next_top,
                right: content.right,
                bottom: next_top + metrics.expanded_height,
            });
            next_top += metrics.expanded_height;
        }
        let chips = if item.control == ItemControl::ThemeChoice {
            theme_chips(metrics, rect)
        } else {
            Vec::new()
        };
        rows.push(ItemRow {
            index,
            rect,
            expanded: expanded_rect,
            chips,
        });
        top = next_top;
    }
    rows
}

/// 主题二选一控件：两个等宽块右对齐于条目行，浅色在前、深色在后。
fn theme_chips(metrics: &SettingsMetrics, row: UiRect) -> Vec<(ThemeChoice, UiRect)> {
    let top = row.top + (row.height() - metrics.chip_height) / 2;
    let right = row.right - metrics.gap;
    let dark_left = right - metrics.chip_width;
    let light_left = dark_left - metrics.chip_gap - metrics.chip_width;
    let chip = |left: i32, choice: ThemeChoice| {
        (
            choice,
            UiRect {
                left,
                top,
                right: left + metrics.chip_width,
                bottom: top + metrics.chip_height,
            },
        )
    };
    vec![
        chip(light_left, ThemeChoice::Light),
        chip(dark_left, ThemeChoice::Dark),
    ]
}

/// 点是否落在矩形内；右、下边界为开区间，避免相邻行的边界点双重命中。
#[must_use]
pub fn contains(rect: UiRect, x: i32, y: i32) -> bool {
    x >= rect.left && x < rect.right && y >= rect.top && y < rect.bottom
}

#[cfg(test)]
mod tests {
    use super::{contains, content_rect, item_rows, nav_rows, scale, title_rect, SettingsMetrics};
    use crate::model::Page;
    use zhu_ye_ime::candidate_ui::UiRect;

    const CLIENT: UiRect = UiRect {
        left: 0,
        top: 0,
        right: 880,
        bottom: 620,
    };

    #[test]
    fn 缩放随dpi线性且不改写96基准() {
        assert_eq!(scale(96, 100), 100);
        assert_eq!(scale(192, 100), 200);
        assert_eq!(scale(144, 100), 150);
        // 低于 96 的 DPI 按 96 处理，不缩小到不可用。
        assert_eq!(scale(48, 100), 100);
        let m96 = SettingsMetrics::new(96);
        let m144 = SettingsMetrics::new(144);
        assert_eq!(m144.desired_width, m96.desired_width * 3 / 2);
        assert_eq!(m144.item_height, m96.item_height * 3 / 2);
    }

    #[test]
    fn 导航三行在导航列内且互不重叠() {
        let metrics = SettingsMetrics::new(96);
        let rows = nav_rows(&metrics, CLIENT);
        assert_eq!(rows.len(), Page::ALL.len());
        for (_, rect) in &rows {
            assert!(rect.left >= 0);
            assert!(
                rect.right <= metrics.nav_width,
                "导航行不得越过导航列右边界"
            );
        }
        for pair in rows.windows(2) {
            assert!(
                pair[0].1.bottom <= pair[1].1.top,
                "导航行不得重叠：{:?} / {:?}",
                pair[0].1,
                pair[1].1
            );
        }
    }

    #[test]
    fn 条目行依次下移且互不重叠() {
        let metrics = SettingsMetrics::new(96);
        let items = Page::Common.items();
        let rows = item_rows(&metrics, CLIENT, items, None);
        assert_eq!(rows.len(), items.len());
        let content = content_rect(&metrics, CLIENT);
        assert_eq!(rows[0].rect.top, content.top, "首行应紧贴内容区顶部");
        for (index, row) in rows.iter().enumerate() {
            assert_eq!(row.index, index);
            assert_eq!(row.rect.height(), metrics.item_height);
            assert_eq!(row.rect.left, content.left);
            assert_eq!(row.rect.right, content.right);
        }
        for pair in rows.windows(2) {
            assert_eq!(
                pair[0].rect.bottom, pair[1].rect.top,
                "相邻条目行必须首尾相接且不重叠"
            );
        }
        // 内容区不得与页标题重叠。
        assert!(title_rect(&metrics, CLIENT).bottom <= content.top);
    }

    #[test]
    fn 展开条目追加说明区并把后续行下移() {
        let metrics = SettingsMetrics::new(96);
        let items = Page::Common.items();
        let plain = item_rows(&metrics, CLIENT, items, None);
        let expanded = item_rows(&metrics, CLIENT, items, Some(1));
        // 展开项自身多出说明区。
        assert!(plain[1].expanded.is_none());
        let note = expanded[1].expanded.expect("展开项应有说明区");
        assert_eq!(note.height(), metrics.expanded_height);
        assert_eq!(note.top, expanded[1].rect.bottom, "说明区紧接条目行下方");
        // 后续行整体下移一个说明区高度。
        assert_eq!(
            expanded[2].rect.top,
            plain[2].rect.top + metrics.expanded_height
        );
        assert_eq!(
            expanded[2].rect.bottom,
            plain[2].rect.bottom + metrics.expanded_height
        );
        // 展开项之前的行不受影响。
        assert_eq!(expanded[0].rect, plain[0].rect);
        // 已接入条目没有说明区，即使下标指向它。
        let ready = item_rows(&metrics, CLIENT, items, Some(0));
        assert!(ready[0].expanded.is_none());
    }

    #[test]
    fn 主题条目有两个右对齐且不重叠的控件块() {
        let metrics = SettingsMetrics::new(96);
        let rows = item_rows(&metrics, CLIENT, Page::Common.items(), None);
        let chips = &rows[0].chips;
        assert_eq!(chips.len(), 2);
        assert_eq!(chips[0].0, zhu_ye_core::ThemeChoice::Light);
        assert_eq!(chips[1].0, zhu_ye_core::ThemeChoice::Dark);
        assert!(chips[0].1.right <= chips[1].1.left, "两个控件块不得重叠");
        assert_eq!(chips[1].1.right, rows[0].rect.right - metrics.gap);
        for (_, chip) in chips {
            assert_eq!(chip.height(), metrics.chip_height);
            assert!(chip.top >= rows[0].rect.top);
            assert!(chip.bottom <= rows[0].rect.bottom);
        }
        // 无控件条目没有块。
        assert!(rows[1].chips.is_empty());
        assert!(rows[rows.len() - 1].chips.is_empty());
    }

    #[test]
    fn 命中测试右下边界为开区间() {
        let rect = UiRect {
            left: 10,
            top: 20,
            right: 30,
            bottom: 40,
        };
        assert!(contains(rect, 10, 20));
        assert!(contains(rect, 29, 39));
        assert!(!contains(rect, 30, 39), "右边界为开区间");
        assert!(!contains(rect, 29, 40), "下边界为开区间");
        assert!(!contains(rect, 9, 20));
        assert!(!contains(rect, 10, 19));
    }
}
