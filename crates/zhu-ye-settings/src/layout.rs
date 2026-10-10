//! 设置窗口布局（纯逻辑，无 Win32 依赖）。
//!
//! 布局从 DPI 与客户区尺寸算出，绘制与命中测试共用同一份结果，避免"看得见的地方点不到"。
//! 尺寸常量按 96 DPI 的逻辑值书写，经 `scale` 换算为像素。

use zhu_ye_core::{ModeChoice, ThemeChoice};
use zhu_ye_ui::{UiRect, BASE_DPI};

use crate::model::{Item, ItemControl, Page};
use crate::panel::{PANEL_COLUMNS, PANEL_ROWS};

/// 期望客户区逻辑尺寸。
const LOGICAL_WIDTH: i32 = 880;
const LOGICAL_HEIGHT: i32 = 700;
/// 左侧导航列宽。
const LOGICAL_NAV_WIDTH: i32 = 200;
/// 导航列顶品牌块高度（T-148 视觉改版，§6.2：竹叶 LOGO + 字标 + 竹节棕细线）。
const LOGICAL_BRAND: i32 = 64;
/// 品牌块底到导航首行的间距（对应视觉稿 `.nav` 上内边距）。
const LOGICAL_NAV_TOP_GAP: i32 = 10;
/// 品牌块左右内边距（对应视觉稿 `.brand` 的横向 padding）。
const LOGICAL_BRAND_PADDING: i32 = 20;
/// 品牌 LOGO 边长（竹叶图形，T-142 嵌入的 zhu.ico 复用为 DrawIconEx 位图）。
const LOGICAL_BRAND_ICON: i32 = 22;
/// 品牌 LOGO 与字标间距。
pub(crate) const LOGICAL_BRAND_TEXT_GAP: i32 = 10;
/// 导航行高。
const LOGICAL_NAV_ROW: i32 = 54;
/// 页标题区高度。
const LOGICAL_TITLE: i32 = 80;
/// 条目行高。
const LOGICAL_ITEM: i32 = 76;
/// 展开说明区高度。
const LOGICAL_EXPANDED: i32 = 54;
/// 内容区内边距。
const LOGICAL_PADDING: i32 = 26;
/// 底部提示条高度。
const LOGICAL_HINT: i32 = 36;
/// 二选一控件单块宽高。
const LOGICAL_CHIP_WIDTH: i32 = 88;
const LOGICAL_CHIP_HEIGHT: i32 = 32;
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
    /// 词库子视图行高。
    pub pack_row_height: i32,
    /// 词库子视图"不验签"说明区高。
    pub pack_note_height: i32,
    /// 词库启用开关块宽。
    pub pack_toggle_width: i32,
    /// 词库元信息列宽。
    pub pack_meta_width: i32,
    /// 词库底部按钮宽。
    pub pack_button_width: i32,
    /// 词库底部按钮高。
    pub pack_button_height: i32,
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
            pack_row_height: scale(dpi, LOGICAL_PACK_ROW),
            pack_note_height: scale(dpi, LOGICAL_PACK_NOTE),
            pack_toggle_width: scale(dpi, LOGICAL_PACK_TOGGLE_WIDTH),
            pack_meta_width: scale(dpi, LOGICAL_PACK_META_WIDTH),
            pack_button_width: scale(dpi, LOGICAL_PACK_BUTTON_WIDTH),
            pack_button_height: scale(dpi, LOGICAL_PACK_BUTTON_HEIGHT),
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

/// 导航列顶品牌块矩形（宽=导航列，底部 1px 竹节棕细线落在 `bottom-1` 行）。
#[must_use]
pub fn brand_rect(metrics: &SettingsMetrics, client: UiRect) -> UiRect {
    UiRect {
        left: client.left,
        top: client.top,
        right: metrics.nav_width,
        bottom: client.top + scale(metrics.dpi, LOGICAL_BRAND),
    }
}

/// 品牌块 LOGO 矩形（导航列内左上，垂直居中）。
#[must_use]
pub fn brand_icon_rect(metrics: &SettingsMetrics, client: UiRect) -> UiRect {
    let rect = brand_rect(metrics, client);
    let size = scale(metrics.dpi, LOGICAL_BRAND_ICON);
    let left = rect.left + scale(metrics.dpi, LOGICAL_BRAND_PADDING);
    let top = rect.top + (rect.height() - size) / 2;
    UiRect {
        left,
        top,
        right: left + size,
        bottom: top + size,
    }
}

/// 导航行；按 `Page::ALL` 顺序从上到下排列，整体位于品牌块之下。
#[must_use]
pub fn nav_rows(metrics: &SettingsMetrics, client: UiRect) -> Vec<(Page, UiRect)> {
    let first_top =
        client.top + scale(metrics.dpi, LOGICAL_BRAND) + scale(metrics.dpi, LOGICAL_NAV_TOP_GAP);
    Page::ALL
        .iter()
        .copied()
        .enumerate()
        .map(|(index, page)| {
            let step = metrics.nav_row_height + metrics.gap;
            let top = first_top + index as i32 * step;
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

/// 二选一控件块的取值。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChipValue {
    /// 主题选择（浅色 / 深色；自定义主题走"自定义主题"子视图）。
    Theme(ThemeChoice),
    /// 新会话默认中英模式（中文 / 英文，D-32 装配项）。
    Mode(ModeChoice),
    /// 在线更新开关（关闭 / 开启，P-03 默认关）。
    OnlineUpdate(bool),
    /// 候选框拼音行开关（关闭 / 显示，T-127 默认开）。
    CandidatePin(bool),
}

/// 二选一控件块：取值决定绘制时的选中状态与点击后的动作。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chip {
    /// 取值。
    pub value: ChipValue,
    /// 块上文字。
    pub label: &'static str,
    /// 块矩形。
    pub rect: UiRect,
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
    pub chips: Vec<Chip>,
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
        let chips = match item.control {
            ItemControl::ThemeChoice => theme_chips(metrics, rect),
            ItemControl::ModeChoice => mode_chips(metrics, rect),
            ItemControl::OnlineUpdate => online_update_chips(metrics, rect),
            ItemControl::CandidatePin => candidate_pin_chips(metrics, rect),
            _ => Vec::new(),
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
fn theme_chips(metrics: &SettingsMetrics, row: UiRect) -> Vec<Chip> {
    two_chips(
        metrics,
        row,
        ("浅色", ChipValue::Theme(ThemeChoice::Light)),
        ("深色", ChipValue::Theme(ThemeChoice::Dark)),
    )
}

/// 中英模式二选一控件：中文在前、英文在后（D-32 装配项）。
fn mode_chips(metrics: &SettingsMetrics, row: UiRect) -> Vec<Chip> {
    two_chips(
        metrics,
        row,
        ("中文", ChipValue::Mode(ModeChoice::Chinese)),
        ("英文", ChipValue::Mode(ModeChoice::English)),
    )
}

/// 在线更新二选一控件：关闭在前、开启在后（P-03 默认关，用户须显式开启）。
fn online_update_chips(metrics: &SettingsMetrics, row: UiRect) -> Vec<Chip> {
    two_chips(
        metrics,
        row,
        ("关闭", ChipValue::OnlineUpdate(false)),
        ("开启", ChipValue::OnlineUpdate(true)),
    )
}

/// 候选框拼音行开关（T-127）：关闭在前、显示在后（默认开，保持历史行为）。
fn candidate_pin_chips(metrics: &SettingsMetrics, row: UiRect) -> Vec<Chip> {
    two_chips(
        metrics,
        row,
        ("关闭", ChipValue::CandidatePin(false)),
        ("显示", ChipValue::CandidatePin(true)),
    )
}

/// 两个等宽块右对齐于条目行，前值在左、后值在右。
fn two_chips(
    metrics: &SettingsMetrics,
    row: UiRect,
    first: (&'static str, ChipValue),
    second: (&'static str, ChipValue),
) -> Vec<Chip> {
    let top = row.top + (row.height() - metrics.chip_height) / 2;
    let right = row.right - metrics.gap;
    let second_left = right - metrics.chip_width;
    let first_left = second_left - metrics.chip_gap - metrics.chip_width;
    let chip = |left: i32, (label, value): (&'static str, ChipValue)| Chip {
        value,
        label,
        rect: UiRect {
            left,
            top,
            right: left + metrics.chip_width,
            bottom: top + metrics.chip_height,
        },
    };
    vec![chip(first_left, first), chip(second_left, second)]
}

/// 点是否落在矩形内；右、下边界为开区间，避免相邻行的边界点双重命中。
#[must_use]
pub fn contains(rect: UiRect, x: i32, y: i32) -> bool {
    x >= rect.left && x < rect.right && y >= rect.top && y < rect.bottom
}

// ---------------------------------------------------------------------------
// 「添加词库」子视图（FR-022 / FR-042）布局
// ---------------------------------------------------------------------------

/// 词库行高。
const LOGICAL_PACK_ROW: i32 = 56;
/// "导入不验签"说明区高度。
const LOGICAL_PACK_NOTE: i32 = 44;
/// 启用开关块宽。
const LOGICAL_PACK_TOGGLE_WIDTH: i32 = 96;
/// 词条数/体积/版本元信息列宽。
const LOGICAL_PACK_META_WIDTH: i32 = 200;
/// 底部按钮宽高。
const LOGICAL_PACK_BUTTON_WIDTH: i32 = 180;
const LOGICAL_PACK_BUTTON_HEIGHT: i32 = 32;

/// 词库子视图的一行：行矩形供命中整行，分区供绘制。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PackRowLayout {
    /// 整行。
    pub row: UiRect,
    /// 名称 + 简介文字区。
    pub label: UiRect,
    /// 词条数 / 体积 / 版本元信息区。
    pub meta: UiRect,
    /// 启用开关块；基础包与不存在的包不画开关。
    pub toggle: UiRect,
}

/// 「添加词库」子视图整体布局。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacksLayout {
    /// "导入包不验签"说明区。
    pub note: UiRect,
    /// 包行；包很多时超出可用高度的行由绘制方截断，布局本身不裁。
    pub rows: Vec<PackRowLayout>,
    /// "返回常用设置"按钮。
    pub back: UiRect,
    /// "导入本地 .zyct…"按钮。
    pub import: UiRect,
}

/// 计算「添加词库」子视图布局。
///
/// 行高固定、超出部分截断：领域包数量受磁盘限制（可分发三个 + 开发产物 + 用户导入），
/// 实际场景放得下；极端堆叠时宁可截断也不压缩到不可点击。
#[must_use]
pub fn packs_layout(metrics: &SettingsMetrics, client: UiRect, pack_count: usize) -> PacksLayout {
    let content = content_rect(metrics, client);
    let note = UiRect {
        left: content.left,
        top: content.top,
        right: content.right,
        bottom: content.top + metrics.pack_note_height.min(content.height()),
    };
    let row_height = metrics.pack_row_height;
    let rows_top = note.bottom + metrics.gap;
    let rows: Vec<PackRowLayout> = (0..pack_count)
        .map(|index| {
            let row = UiRect {
                left: content.left,
                top: rows_top + index as i32 * row_height,
                right: content.right,
                bottom: rows_top + (index as i32 + 1) * row_height,
            };
            let toggle = UiRect {
                left: row.right - metrics.pack_toggle_width,
                top: row.top + (row.height() - metrics.chip_height) / 2,
                right: row.right,
                bottom: row.top + (row.height() + metrics.chip_height) / 2,
            };
            let meta = UiRect {
                left: toggle.left - metrics.pack_meta_width - metrics.gap,
                top: row.top,
                right: toggle.left - metrics.gap,
                bottom: row.bottom,
            };
            let label = UiRect {
                left: row.left,
                top: row.top,
                right: meta.left - metrics.gap,
                bottom: row.bottom,
            };
            PackRowLayout {
                row,
                label,
                meta,
                toggle,
            }
        })
        .collect();
    let button_top = content.bottom - metrics.pack_button_height;
    let back = UiRect {
        left: content.left,
        top: button_top,
        right: content.left + metrics.pack_button_width,
        bottom: button_top + metrics.pack_button_height,
    };
    let import = UiRect {
        left: content.right - metrics.pack_button_width,
        top: button_top,
        right: content.right,
        bottom: button_top + metrics.pack_button_height,
    };
    PacksLayout {
        note,
        rows,
        back,
        import,
    }
}

// ---------------------------------------------------------------------------
// 「管理输入法」/「修复输入法」子视图布局（T-076 / FR-043）
// ---------------------------------------------------------------------------

/// 「管理输入法」子视图整体布局。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ManageLayout {
    /// 注册状态详情区（多行文本，`detail_lines` 逐行绘制，超出截断）。
    pub status: UiRect,
    /// "打开系统输入法设置"按钮。
    pub sys_settings: UiRect,
    /// "返回常用设置"按钮。
    pub back: UiRect,
}

/// 「修复输入法」子视图整体布局。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepairLayout {
    /// 检测结果逐行区；行数由 `repair_scan.summary_lines()` 决定，超出可用高度截断。
    pub rows: Vec<UiRect>,
    /// "一级修复（无需管理员）"按钮。
    pub l1: UiRect,
    /// "二级修复（需管理员，UAC）"按钮。
    pub l2: UiRect,
    /// "返回常用设置"按钮。
    pub back: UiRect,
}

/// 计算「管理输入法」子视图布局：状态区在上，两个按钮在底部。
#[must_use]
pub fn manage_layout(metrics: &SettingsMetrics, client: UiRect) -> ManageLayout {
    let content = content_rect(metrics, client);
    let status = UiRect {
        left: content.left,
        top: content.top,
        right: content.right,
        bottom: content.top + metrics.pack_note_height * 3,
    };
    let button_top = content.bottom - metrics.pack_button_height;
    let back = UiRect {
        left: content.left,
        top: button_top,
        right: content.left + metrics.pack_button_width,
        bottom: button_top + metrics.pack_button_height,
    };
    let sys_settings = UiRect {
        left: back.right + metrics.gap,
        top: button_top,
        right: back.right + metrics.gap + metrics.pack_button_width,
        bottom: button_top + metrics.pack_button_height,
    };
    ManageLayout {
        status,
        sys_settings,
        back,
    }
}

/// 计算「修复输入法」子视图布局：检测行在上，三个按钮在底部。
#[must_use]
pub fn repair_layout(metrics: &SettingsMetrics, client: UiRect, row_count: usize) -> RepairLayout {
    let content = content_rect(metrics, client);
    let row_height = metrics.pack_row_height.saturating_sub(metrics.gap);
    let rows: Vec<UiRect> = (0..row_count)
        .map(|index| UiRect {
            left: content.left,
            top: content.top + index as i32 * row_height,
            right: content.right,
            bottom: content.top + (index as i32 + 1) * row_height,
        })
        .collect();
    let button_top = content.bottom - metrics.pack_button_height;
    let gap = metrics.gap;
    let width = (content.width() - gap * 2) / 3;
    let l1 = UiRect {
        left: content.left,
        top: button_top,
        right: content.left + width,
        bottom: button_top + metrics.pack_button_height,
    };
    let l2 = UiRect {
        left: l1.right + gap,
        top: button_top,
        right: l1.right + gap + width,
        bottom: button_top + metrics.pack_button_height,
    };
    let back = UiRect {
        left: l2.right + gap,
        top: button_top,
        right: content.right,
        bottom: button_top + metrics.pack_button_height,
    };
    RepairLayout { rows, l1, l2, back }
}

/// 「检查更新」与「版本与诊断信息」平铺在「关于与更新」页上的整体布局
/// （T-143：取消两级子视图，开关行之下依次是指标说明、按钮与结果区、诊断信息小节）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AboutLayout {
    /// 在线更新状态行（"在线更新：已开启/已关闭"）。
    pub status: UiRect,
    /// 状态行下方的提示行（未开启原因或更新器路径）。
    pub hint: UiRect,
    /// 检查更新结果区（逐行展示更新器输出，超出截断）。
    pub result: UiRect,
    /// "检查更新"按钮（强调色）。
    pub check: UiRect,
    /// "应用更新"按钮（普通色）。
    pub apply: UiRect,
    /// "版本与诊断信息"小节标题行。
    pub diag_title: UiRect,
    /// 诊断信息行；行数由诊断内容决定，超出可用高度截断。
    pub diag_rows: Vec<UiRect>,
}

/// 关于页扁平区块的行高常量（96 DPI 逻辑值；诊断行按 small 字体高度加呼吸留白）。
const LOGICAL_SECTION_ROW: i32 = 26;
const LOGICAL_HINT_ROW: i32 = 20;
const LOGICAL_DIAG_ROW: i32 = 21;
/// 结果区固定显示行数（更新器输出超出部分截断）。
const LOGICAL_RESULT_ROWS: i32 = 3;

/// 计算「关于与更新」页扁平布局：开关条目行（调用方用 `item_rows` 绘制）之下，
/// 依次是更新状态/提示、结果区、检查/应用按钮、诊断信息小节。
#[must_use]
pub fn about_layout(metrics: &SettingsMetrics, client: UiRect, diag_count: usize) -> AboutLayout {
    let content = content_rect(metrics, client);
    let gap = metrics.gap;
    let section = |top: i32, height: i32| UiRect {
        left: content.left,
        top,
        right: content.right,
        bottom: top + height,
    };
    let status_h = scale(metrics.dpi, LOGICAL_SECTION_ROW);
    let hint_h = scale(metrics.dpi, LOGICAL_HINT_ROW);
    let diag_row_h = scale(metrics.dpi, LOGICAL_DIAG_ROW);
    let result_h = scale(metrics.dpi, LOGICAL_RESULT_ROWS) * diag_row_h;

    // 第一行条目（"启用在线更新"）由 `item_rows` 绘制；区块从它下方开始。
    let mut top = content.top + metrics.item_height + gap;
    let status = section(top, status_h);
    top = status.bottom;
    let hint = section(top, hint_h);
    top = hint.bottom + gap;
    let result = section(top, result_h);
    top = result.bottom + gap;

    let button_h = metrics.pack_button_height;
    let width = (content.width() - gap) / 2;
    let check = UiRect {
        left: content.left,
        top,
        right: content.left + width,
        bottom: top + button_h,
    };
    let apply = UiRect {
        left: check.right + gap,
        top,
        right: content.right,
        bottom: top + button_h,
    };
    top = apply.bottom + gap;
    let diag_title = section(top, status_h);
    top = diag_title.bottom + gap;
    // 诊断行只生成到内容区底边内；超出部分由绘制端按行数截断（诊断内容至多十余行）。
    let diag_rows: Vec<UiRect> = (0..diag_count)
        .map(|index| {
            let row_top = top + index as i32 * diag_row_h;
            section(row_top, diag_row_h)
        })
        .take_while(|row| row.bottom <= content.bottom)
        .collect();
    AboutLayout {
        status,
        hint,
        result,
        check,
        apply,
        diag_title,
        diag_rows,
    }
}

// ---------------------------------------------------------------------------
// T-088 三子视图：用户词表 / 通讯录 / 自定义主题（FR-048）布局
// ---------------------------------------------------------------------------

/// 「用户词表」子视图布局（T-088 / FR-048）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserWordsLayout {
    /// 说明区（导出/导入口径）。
    pub info: UiRect,
    /// 结果/错误消息区（上次导入导出结果，随操作更新）。
    pub result: UiRect,
    /// "导出用户词表…"按钮。
    pub export: UiRect,
    /// "从文件导入…"按钮。
    pub import: UiRect,
    /// "返回常用设置"按钮。
    pub back: UiRect,
}

/// 计算「用户词表」子视图布局：说明在上、结果居中、三个按钮在底部等宽排列。
#[must_use]
pub fn userwords_layout(metrics: &SettingsMetrics, client: UiRect) -> UserWordsLayout {
    let content = content_rect(metrics, client);
    let row_height = metrics.pack_row_height.saturating_sub(metrics.gap);
    let info = UiRect {
        left: content.left,
        top: content.top,
        right: content.right,
        bottom: content.top + row_height * 3,
    };
    let button_top = content.bottom - metrics.pack_button_height;
    let gap = metrics.gap;
    let width = (content.width() - gap * 2) / 3;
    let export = UiRect {
        left: content.left,
        top: button_top,
        right: content.left + width,
        bottom: button_top + metrics.pack_button_height,
    };
    let import = UiRect {
        left: export.right + gap,
        top: button_top,
        right: export.right + gap + width,
        bottom: button_top + metrics.pack_button_height,
    };
    let back = UiRect {
        left: import.right + gap,
        top: button_top,
        right: content.right,
        bottom: button_top + metrics.pack_button_height,
    };
    let result = UiRect {
        left: content.left,
        top: info.bottom + gap,
        right: content.right,
        bottom: button_top - gap,
    };
    UserWordsLayout {
        info,
        result,
        export,
        import,
        back,
    }
}

/// 「通讯录」子视图布局（T-088 / FR-048：.vcf 界面化导入）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContactsLayout {
    /// 说明区。
    pub info: UiRect,
    /// 已登记 .vcf 路径的逐行区（超出可用高度截断并提示）。
    pub rows: Vec<UiRect>,
    /// "导入 .vcf…"按钮。
    pub import: UiRect,
    /// "返回常用设置"按钮。
    pub back: UiRect,
}

/// 计算「通讯录」子视图布局：说明在上、路径列表居中、导入/返回按钮在底部。
#[must_use]
pub fn contacts_layout(
    metrics: &SettingsMetrics,
    client: UiRect,
    row_count: usize,
) -> ContactsLayout {
    let content = content_rect(metrics, client);
    let row_height = 30.max(metrics.pack_row_height.saturating_sub(metrics.gap));
    let info = UiRect {
        left: content.left,
        top: content.top,
        right: content.right,
        bottom: content.top + row_height * 2,
    };
    let button_top = content.bottom - metrics.pack_button_height;
    let gap = metrics.gap;
    let width = (content.width() - gap) / 2;
    let import = UiRect {
        left: content.left,
        top: button_top,
        right: content.left + width,
        bottom: button_top + metrics.pack_button_height,
    };
    let back = UiRect {
        left: import.right + gap,
        top: button_top,
        right: content.right,
        bottom: button_top + metrics.pack_button_height,
    };
    let available = (button_top - gap - info.bottom) / row_height;
    let shown = row_count.min(available.max(0) as usize);
    let rows: Vec<UiRect> = (0..shown)
        .map(|index| UiRect {
            left: content.left,
            top: info.bottom + index as i32 * row_height,
            right: content.right,
            bottom: info.bottom + (index as i32 + 1) * row_height,
        })
        .collect();
    ContactsLayout {
        info,
        rows,
        import,
        back,
    }
}

/// 「自定义主题」子视图布局（T-088 / FR-048：主题文件列表与选择）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemesLayout {
    /// 说明区。
    pub info: UiRect,
    /// 主题文件逐行区（显示名 + 状态）。
    pub rows: Vec<UiRect>,
    /// "返回常用设置"按钮。
    pub back: UiRect,
}

/// 计算「自定义主题」子视图布局：说明在上、主题行居中、返回按钮在底部。
#[must_use]
pub fn themes_layout(metrics: &SettingsMetrics, client: UiRect, row_count: usize) -> ThemesLayout {
    let content = content_rect(metrics, client);
    let row_height = metrics.pack_row_height.saturating_sub(metrics.gap);
    let info = UiRect {
        left: content.left,
        top: content.top,
        right: content.right,
        bottom: content.top + row_height * 2,
    };
    let back = UiRect {
        left: content.left,
        top: content.bottom - metrics.pack_button_height,
        right: content.left + metrics.pack_button_width,
        bottom: content.bottom,
    };
    let available = (back.top - metrics.gap - info.bottom) / row_height;
    let shown = row_count.min(available.max(0) as usize);
    let rows: Vec<UiRect> = (0..shown)
        .map(|index| UiRect {
            left: content.left,
            top: info.bottom + index as i32 * row_height,
            right: content.right,
            bottom: info.bottom + (index as i32 + 1) * row_height,
        })
        .collect();
    ThemesLayout { info, rows, back }
}

// ---------------------------------------------------------------------------
// 工具箱面板（浮层）布局
// ---------------------------------------------------------------------------

/// 面板逻辑尺寸。
const LOGICAL_PANEL_WIDTH: i32 = 700;
const LOGICAL_PANEL_PADDING: i32 = 20;
const LOGICAL_PANEL_GAP: i32 = 6;
const LOGICAL_PANEL_HEADER: i32 = 54;
const LOGICAL_PANEL_FOOTER: i32 = 50;
const LOGICAL_PANEL_CELL_HEIGHT: i32 = 58;
const LOGICAL_PANEL_CELL_MIN_WIDTH: i32 = 36;
const LOGICAL_PANEL_BUTTON_WIDTH: i32 = 100;
const LOGICAL_PANEL_BUTTON_HEIGHT: i32 = 32;
const LOGICAL_PANEL_PAGE_LABEL_WIDTH: i32 = 76;
const LOGICAL_PANEL_HINT: i32 = 28;

/// 面板布局尺寸；格子宽度由期望宽度与列数反推，保证一整页刚好铺满一行。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PanelMetrics {
    /// 当前 DPI。
    pub dpi: u32,
    /// 内边距。
    pub padding: i32,
    /// 格子间距。
    pub gap: i32,
    /// 列数。
    pub columns: i32,
    /// 行数。
    pub rows: i32,
    /// 格子宽。
    pub cell_width: i32,
    /// 格子高。
    pub cell_height: i32,
    /// 标题区高。
    pub header_height: i32,
    /// 底栏高。
    pub footer_height: i32,
    /// 按钮宽。
    pub button_width: i32,
    /// 按钮高。
    pub button_height: i32,
    /// 页码文字宽。
    pub page_label_width: i32,
    /// 提示条高。
    pub hint_height: i32,
    /// 期望客户区宽。
    pub desired_width: i32,
    /// 期望客户区高。
    pub desired_height: i32,
}

impl PanelMetrics {
    /// 按 DPI 建立面板尺寸集。
    #[must_use]
    pub fn new(dpi: u32) -> Self {
        let dpi = dpi.max(BASE_DPI);
        let padding = scale(dpi, LOGICAL_PANEL_PADDING);
        let gap = scale(dpi, LOGICAL_PANEL_GAP);
        let columns = PANEL_COLUMNS as i32;
        let rows = PANEL_ROWS as i32;
        let desired_width = scale(dpi, LOGICAL_PANEL_WIDTH);
        let usable = desired_width - 2 * padding - (columns - 1) * gap;
        let cell_width = (usable / columns).max(scale(dpi, LOGICAL_PANEL_CELL_MIN_WIDTH));
        let cell_height = scale(dpi, LOGICAL_PANEL_CELL_HEIGHT);
        let header_height = scale(dpi, LOGICAL_PANEL_HEADER);
        let footer_height = scale(dpi, LOGICAL_PANEL_FOOTER);
        // 高度由一整页格子加页脚推出，保证整页无需滚动即可放下。
        let desired_height = header_height
            + padding
            + rows * cell_height
            + (rows - 1) * gap
            + padding
            + scale(dpi, LOGICAL_PANEL_HINT)
            + footer_height;
        Self {
            dpi,
            padding,
            gap,
            columns,
            rows,
            cell_width,
            cell_height,
            header_height,
            footer_height,
            button_width: scale(dpi, LOGICAL_PANEL_BUTTON_WIDTH),
            button_height: scale(dpi, LOGICAL_PANEL_BUTTON_HEIGHT),
            page_label_width: scale(dpi, LOGICAL_PANEL_PAGE_LABEL_WIDTH),
            hint_height: scale(dpi, LOGICAL_PANEL_HINT),
            desired_width,
            desired_height,
        }
    }

    /// 期望的客户区尺寸。
    #[must_use]
    pub fn desired_client_size(&self) -> (i32, i32) {
        (self.desired_width, self.desired_height)
    }
}

/// 面板布局结果；绘制与命中测试共用。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PanelLayout {
    /// 标题区。
    pub header: UiRect,
    /// 格子（行优先），数量等于本页条目数。
    pub cells: Vec<UiRect>,
    /// 返回设置。
    pub back: UiRect,
    /// 上一页。
    pub prev: UiRect,
    /// 下一页。
    pub next: UiRect,
    /// 页码文字。
    pub page_label: UiRect,
    /// 提示条。
    pub hint: UiRect,
}

/// 计算面板布局；`cell_count` 为本页条目数（末页可能少于整页）。
#[must_use]
pub fn panel_layout(metrics: &PanelMetrics, client: UiRect, cell_count: usize) -> PanelLayout {
    let header = UiRect {
        left: client.left,
        top: client.top,
        right: client.right,
        bottom: client.top + metrics.header_height,
    };
    let grid_top = header.bottom + metrics.padding;
    let columns = usize::try_from(metrics.columns).unwrap_or(1).max(1);
    let cells = (0..cell_count)
        .map(|index| {
            let row = i32::try_from(index / columns).unwrap_or(0);
            let column = i32::try_from(index % columns).unwrap_or(0);
            let left = client.left + metrics.padding + column * (metrics.cell_width + metrics.gap);
            let top = grid_top + row * (metrics.cell_height + metrics.gap);
            UiRect {
                left,
                top,
                right: left + metrics.cell_width,
                bottom: top + metrics.cell_height,
            }
        })
        .collect();

    let footer_top = client.bottom - metrics.footer_height;
    let button_top = footer_top + (metrics.footer_height - metrics.button_height) / 2;
    let button_bottom = button_top + metrics.button_height;
    let back = UiRect {
        left: client.left + metrics.padding,
        top: button_top,
        right: client.left + metrics.padding + metrics.button_width,
        bottom: button_bottom,
    };
    let next = UiRect {
        left: client.right - metrics.padding - metrics.button_width,
        top: button_top,
        right: client.right - metrics.padding,
        bottom: button_bottom,
    };
    let prev = UiRect {
        left: next.left - metrics.gap - metrics.button_width,
        top: button_top,
        right: next.left - metrics.gap,
        bottom: button_bottom,
    };
    let page_label = UiRect {
        left: prev.left - metrics.page_label_width,
        top: button_top,
        right: prev.left,
        bottom: button_bottom,
    };
    let hint = UiRect {
        left: client.left + metrics.padding,
        top: footer_top - metrics.hint_height,
        right: client.right - metrics.padding,
        bottom: footer_top,
    };
    PanelLayout {
        header,
        cells,
        back,
        prev,
        next,
        page_label,
        hint,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        contains, content_rect, item_rows, manage_layout, nav_rows, panel_layout, repair_layout,
        scale, title_rect, PanelMetrics, SettingsMetrics,
    };
    use crate::model::Page;
    use zhu_ye_ui::UiRect;

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
        // 「简繁切换」仍是规划中条目：点击可展开。
        let expand_index = 13;
        let plain = item_rows(&metrics, CLIENT, items, None);
        let expanded = item_rows(&metrics, CLIENT, items, Some(expand_index));
        // 展开项自身多出说明区。
        assert!(plain[expand_index].expanded.is_none());
        let note = expanded[expand_index].expanded.expect("展开项应有说明区");
        assert_eq!(note.height(), metrics.expanded_height);
        assert_eq!(
            note.top, expanded[expand_index].rect.bottom,
            "说明区紧接条目行下方"
        );
        // 后续行整体下移一个说明区高度。
        assert_eq!(
            expanded[expand_index + 1].rect.top,
            plain[expand_index + 1].rect.top + metrics.expanded_height
        );
        assert_eq!(
            expanded[expand_index + 1].rect.bottom,
            plain[expand_index + 1].rect.bottom + metrics.expanded_height
        );
        // 展开项之前的行不受影响。
        assert_eq!(expanded[0].rect, plain[0].rect);
        // 已接入条目没有说明区，即使下标指向它。
        let ready = item_rows(&metrics, CLIENT, items, Some(0));
        assert!(ready[0].expanded.is_none());
    }

    #[test]
    fn 管理子视图状态区在上按钮在底部不重叠() {
        let metrics = SettingsMetrics::new(96);
        let layout = manage_layout(&metrics, CLIENT);
        let content = content_rect(&metrics, CLIENT);
        assert!(layout.status.top >= content.top);
        assert!(
            layout.status.bottom <= content.bottom,
            "状态区不得越过内容区"
        );
        // 按钮贴内容区底部。
        assert_eq!(layout.back.bottom, content.bottom);
        assert_eq!(layout.sys_settings.bottom, content.bottom);
        assert_eq!(layout.back.height(), metrics.pack_button_height);
        assert_eq!(layout.sys_settings.height(), metrics.pack_button_height);
        // 两个按钮不重叠，都在内容区内。
        assert!(layout.back.right <= layout.sys_settings.left);
        assert!(layout.sys_settings.right <= content.right);
        // 按钮不得压住状态区。
        assert!(layout.status.bottom <= layout.back.top);
    }

    #[test]
    fn 修复子视图检测行数决定高度且按钮三列均分() {
        let metrics = SettingsMetrics::new(96);
        let layout = repair_layout(&metrics, CLIENT, 4);
        assert_eq!(layout.rows.len(), 4);
        assert_eq!(layout.l1.top, layout.back.top);
        assert_eq!(layout.l2.top, layout.back.top);
        assert_eq!(layout.l1.bottom, content_rect(&metrics, CLIENT).bottom);
        // 三列按钮依次相接（隔一个 gap）、不重叠，覆盖整个内容区宽度。
        assert_eq!(layout.l1.right + metrics.gap, layout.l2.left);
        assert_eq!(layout.l2.right + metrics.gap, layout.back.left);
        assert_eq!(layout.back.right, content_rect(&metrics, CLIENT).right);
        // 检测行都在按钮上方且逐行不重叠。
        for pair in layout.rows.windows(2) {
            assert!(pair[0].bottom <= pair[1].top);
        }
        assert!(layout
            .rows
            .last()
            .is_none_or(|row| row.bottom <= layout.l1.top));
    }

    #[test]
    fn 主题条目有两个右对齐且不重叠的控件块() {
        let metrics = SettingsMetrics::new(96);
        let rows = item_rows(&metrics, CLIENT, Page::Common.items(), None);
        let chips = &rows[0].chips;
        assert_eq!(chips.len(), 2);
        assert_eq!(
            chips[0].value,
            super::ChipValue::Theme(zhu_ye_core::ThemeChoice::Light)
        );
        assert_eq!(
            chips[1].value,
            super::ChipValue::Theme(zhu_ye_core::ThemeChoice::Dark)
        );
        assert_eq!(chips[0].label, "浅色");
        assert_eq!(chips[1].label, "深色");
        assert!(
            chips[0].rect.right <= chips[1].rect.left,
            "两个控件块不得重叠"
        );
        assert_eq!(chips[1].rect.right, rows[0].rect.right - metrics.gap);
        for chip in chips {
            assert_eq!(chip.rect.height(), metrics.chip_height);
            assert!(chip.rect.top >= rows[0].rect.top);
            assert!(chip.rect.bottom <= rows[0].rect.bottom);
        }
        // 无控件条目没有块（自定义主题与用户词表等子视图条目）。
        assert!(rows[1].chips.is_empty());
        assert!(rows[rows.len() - 1].chips.is_empty());
    }

    #[test]
    fn 候选拼音条目有两个开关控件块且关闭在先显示在后() {
        let metrics = SettingsMetrics::new(96);
        let rows = item_rows(&metrics, CLIENT, Page::Common.items(), None);
        let chips = &rows[2].chips;
        assert_eq!(chips.len(), 2, "候选拼音是二选一开关（T-127）");
        assert_eq!(
            chips[0].value,
            super::ChipValue::CandidatePin(false),
            "关闭在前"
        );
        assert_eq!(
            chips[1].value,
            super::ChipValue::CandidatePin(true),
            "显示在后"
        );
        assert_eq!(chips[0].label, "关闭");
        assert_eq!(chips[1].label, "显示");
        assert!(chips[0].rect.right <= chips[1].rect.left);
        assert_eq!(chips[1].rect.right, rows[2].rect.right - metrics.gap);
    }

    #[test]
    fn 英文输入法条目有两个中英模式控件块() {
        let metrics = SettingsMetrics::new(96);
        let rows = item_rows(&metrics, CLIENT, Page::Common.items(), None);
        let chips = &rows[3].chips;
        assert_eq!(chips.len(), 2, "英文输入法是 D-32 装配项二选一");
        assert_eq!(
            chips[0].value,
            super::ChipValue::Mode(zhu_ye_core::ModeChoice::Chinese)
        );
        assert_eq!(
            chips[1].value,
            super::ChipValue::Mode(zhu_ye_core::ModeChoice::English)
        );
        assert_eq!(chips[0].label, "中文");
        assert_eq!(chips[1].label, "英文");
        assert!(chips[0].rect.right <= chips[1].rect.left);
        assert_eq!(chips[1].rect.right, rows[3].rect.right - metrics.gap);
    }

    #[test]
    fn 在线更新条目有两个开关控件块且默认关闭在先() {
        use crate::model::Page;
        let metrics = SettingsMetrics::new(96);
        let rows = item_rows(&metrics, CLIENT, Page::About.items(), None);
        let chips = &rows[0].chips;
        assert_eq!(chips.len(), 2, "在线更新是二选一开关（P-03）");
        assert_eq!(
            chips[0].value,
            super::ChipValue::OnlineUpdate(false),
            "关闭在前"
        );
        assert_eq!(
            chips[1].value,
            super::ChipValue::OnlineUpdate(true),
            "开启在后"
        );
        assert_eq!(chips[0].label, "关闭");
        assert_eq!(chips[1].label, "开启");
        assert!(chips[0].rect.right <= chips[1].rect.left);
        assert_eq!(chips[1].rect.right, rows[0].rect.right - metrics.gap);
    }

    #[test]
    fn 关于页扁平布局各区块依次排列且不重叠() {
        let metrics = SettingsMetrics::new(96);
        let content = content_rect(&metrics, CLIENT);
        let layout = super::about_layout(&metrics, CLIENT, 5);
        // 状态行紧接开关条目行（条目行由 item_rows 绘制，高为 item_height）。
        assert_eq!(
            layout.status.top,
            content.top + metrics.item_height + metrics.gap
        );
        assert_eq!(layout.status.left, content.left);
        assert_eq!(layout.status.right, content.right);
        // 提示行紧随状态行，结果区在提示行与按钮之间，互不重叠。
        assert_eq!(layout.hint.top, layout.status.bottom);
        assert!(layout.result.top >= layout.hint.bottom + metrics.gap);
        assert!(layout.result.height() >= 2 * super::scale(96, super::LOGICAL_DIAG_ROW));
        assert!(layout.result.bottom <= layout.check.top);
        // 两按钮等宽相接（隔一个 gap）且左右覆盖整个内容区宽度。
        assert_eq!(layout.check.height(), metrics.pack_button_height);
        assert_eq!(layout.check.width(), layout.apply.width());
        assert_eq!(layout.check.right + metrics.gap, layout.apply.left);
        assert_eq!(layout.apply.right, content.right);
        assert_eq!(layout.check.bottom, layout.apply.bottom);
        // 诊断小节标题在按钮之下，诊断行首尾相接且按诊断数生成。
        assert_eq!(layout.diag_title.top, layout.apply.bottom + metrics.gap);
        assert_eq!(layout.diag_rows.len(), 5);
        assert_eq!(
            layout.diag_rows[0].top,
            layout.diag_title.bottom + metrics.gap
        );
        for pair in layout.diag_rows.windows(2) {
            assert_eq!(pair[1].top, pair[0].bottom);
        }
        // 5 行诊断全部落在内容区内。
        assert!(
            layout
                .diag_rows
                .last()
                .is_none_or(|row| row.bottom <= content.bottom),
            "常规诊断行不得越出内容区底部"
        );
    }

    #[test]
    fn 关于页诊断行超出内容区时截断不越界() {
        let metrics = SettingsMetrics::new(96);
        let content = content_rect(&metrics, CLIENT);
        // 9 行（接近真实诊断最大量）在 620 高的矮窗口里放不下时被截断：
        // 生成的行数全部落在内容区内、首尾相接，且不 panic。
        let crowded = super::about_layout(&metrics, CLIENT, 9);
        assert!(
            crowded
                .diag_rows
                .last()
                .is_none_or(|row| row.bottom <= content.bottom),
            "最后一行不得越出内容区底部"
        );
        for pair in crowded.diag_rows.windows(2) {
            assert_eq!(pair[1].top, pair[0].bottom, "行必须首尾相接");
        }
        // 极端数量同样截断在内容区内。
        let extreme = super::about_layout(&metrics, CLIENT, 200);
        assert!(!extreme.diag_rows.is_empty(), "常规量下至少一行诊断");
        assert!(
            extreme
                .diag_rows
                .last()
                .is_none_or(|row| row.bottom <= content.bottom),
            "极端数量下诊断行也必须在内容区内截断"
        );
    }

    #[test]
    fn 词库子视图行与按钮布局成立() {
        let metrics = SettingsMetrics::new(96);
        let content = content_rect(&metrics, CLIENT);
        // 基础包 + 三个领域包 + 两个开发产物，最典型的六行。
        let layout = super::packs_layout(&metrics, CLIENT, 6);
        assert_eq!(layout.rows.len(), 6);
        // 说明区在内容区顶部，行紧接其后。
        assert_eq!(layout.note.top, content.top);
        assert_eq!(layout.rows[0].row.top, layout.note.bottom + metrics.gap);
        for (index, pack) in layout.rows.iter().enumerate() {
            assert_eq!(pack.row.height(), metrics.pack_row_height);
            assert_eq!(pack.row.left, content.left);
            assert_eq!(pack.row.right, content.right);
            if index > 0 {
                assert_eq!(pack.row.top, layout.rows[index - 1].row.bottom);
            }
            // 分区互不重叠且都在行内。
            assert!(pack.label.right <= pack.meta.left);
            assert!(pack.meta.right <= pack.toggle.left);
            assert_eq!(pack.toggle.right, pack.row.right);
            assert!(pack.toggle.top >= pack.row.top && pack.toggle.bottom <= pack.row.bottom);
        }
        // 按钮贴底、互不重叠、左右分开。
        assert_eq!(layout.back.bottom, content.bottom);
        assert_eq!(layout.import.bottom, content.bottom);
        assert_eq!(layout.back.height(), metrics.pack_button_height);
        assert_eq!(layout.import.height(), metrics.pack_button_height);
        assert!(layout.back.right <= layout.import.left);
    }

    #[test]
    fn 词库子视图工具栏不与行重叠() {
        let metrics = SettingsMetrics::new(96);
        let layout = super::packs_layout(&metrics, CLIENT, 6);
        // 常规行数下最后一行不与底部按钮重叠。
        let last = layout.rows.last().expect("应有行");
        assert!(
            last.row.bottom <= layout.back.top,
            "最后一行不得压到按钮：{} > {}",
            last.row.bottom,
            layout.back.top
        );
        // 极端堆叠时布局不 panic、不越界：行矩形单调递增，按钮位置固定不动。
        let crowded = super::packs_layout(&metrics, CLIENT, 40);
        assert_eq!(crowded.rows.len(), 40);
        for pair in crowded.rows.windows(2) {
            assert_eq!(pair[1].row.top, pair[0].row.bottom, "行必须首尾相接");
        }
        assert_eq!(crowded.back, layout.back, "按钮位置与行数无关");
        assert_eq!(crowded.import, layout.import);
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

    fn panel_client(metrics: &PanelMetrics) -> UiRect {
        let (width, height) = metrics.desired_client_size();
        UiRect {
            left: 0,
            top: 0,
            right: width,
            bottom: height,
        }
    }

    #[test]
    fn 面板尺寸随dpi缩放且容纳整页() {
        let base = PanelMetrics::new(96);
        let scaled = PanelMetrics::new(144);
        assert_eq!(scaled.desired_width, base.desired_width * 3 / 2);
        assert_eq!(scaled.cell_height, base.cell_height * 3 / 2);
        // 一整页（列 × 行）加上页眉页脚必须放得进期望高度。
        let page = base.columns * base.rows;
        let layout = panel_layout(&base, panel_client(&base), page as usize);
        assert_eq!(layout.cells.len(), page as usize);
        let last = layout.cells.last().expect("整页应有格子");
        assert!(
            last.bottom <= layout.hint.top,
            "最后一格不得压到提示条：{} > {}",
            last.bottom,
            layout.hint.top
        );
    }

    #[test]
    fn 格子按行优先排列且互不重叠() {
        let metrics = PanelMetrics::new(96);
        let client = panel_client(&metrics);
        let columns = metrics.columns as usize;
        let layout = panel_layout(&metrics, client, columns * 2);
        assert_eq!(layout.cells.len(), columns * 2);
        // 同一行内左边界递增，第二行整体低于第一行。
        for index in 0..columns {
            assert_eq!(layout.cells[index].top, layout.cells[0].top);
            if index > 0 {
                assert!(layout.cells[index].left > layout.cells[index - 1].left);
            }
        }
        assert_eq!(
            layout.cells[columns].top,
            layout.cells[0].bottom + metrics.gap
        );
        assert_eq!(
            layout.cells[columns].left, layout.cells[0].left,
            "换行回到首列"
        );
        // 任意两格不重叠。
        for (index, cell) in layout.cells.iter().enumerate() {
            for other in layout.cells.iter().skip(index + 1) {
                let disjoint = cell.right <= other.left
                    || other.right <= cell.left
                    || cell.bottom <= other.top
                    || other.bottom <= cell.top;
                assert!(disjoint, "格子重叠：{cell:?} / {other:?}");
            }
            assert!(cell.left >= client.left, "格子越出左边界");
            assert!(cell.right <= client.right, "格子越出右边界");
            assert!(cell.top >= layout.header.bottom, "格子压到标题区");
        }
    }

    #[test]
    fn 末页格子数少于整页时布局仍成立() {
        let metrics = PanelMetrics::new(96);
        let client = panel_client(&metrics);
        let layout = panel_layout(&metrics, client, 3);
        assert_eq!(layout.cells.len(), 3);
        for cell in &layout.cells {
            assert_eq!(cell.top, layout.cells[0].top, "不足一行时都在首行");
        }
    }

    #[test]
    fn 页脚按钮与页码互不重叠且都在底栏内() {
        let metrics = PanelMetrics::new(96);
        let client = panel_client(&metrics);
        let layout = panel_layout(&metrics, client, 10);
        let footer_top = client.bottom - metrics.footer_height;
        for (name, rect) in [
            ("返回", layout.back),
            ("上一页", layout.prev),
            ("下一页", layout.next),
            ("页码", layout.page_label),
        ] {
            assert!(rect.top >= footer_top, "{name} 越出底栏上边界");
            assert!(rect.bottom <= client.bottom, "{name} 越出底栏下边界");
            assert_eq!(rect.height(), metrics.button_height, "{name} 高度不一致");
        }
        // 左到右：返回 | 页码 | 上一页 | 下一页，互不重叠。
        assert!(layout.back.right <= layout.page_label.left);
        assert!(layout.page_label.right <= layout.prev.left);
        assert!(layout.prev.right <= layout.next.left);
        assert_eq!(layout.next.right, client.right - metrics.padding);
    }

    #[test]
    fn 提示条在底栏之上且不与格子重叠() {
        let metrics = PanelMetrics::new(96);
        let client = panel_client(&metrics);
        let layout = panel_layout(&metrics, client, (metrics.columns * metrics.rows) as usize);
        assert_eq!(layout.hint.bottom, client.bottom - metrics.footer_height);
        assert_eq!(layout.hint.height(), metrics.hint_height);
        for cell in &layout.cells {
            assert!(cell.bottom <= layout.hint.top);
        }
    }

    #[test]
    fn 默认窗口高度下常用设置页至少显示八行() {
        // T-125：窗口 880×700、条目行 76 —— 常用设置 15 项至少可见 8 行（顶部标题区与
        // 底部提示条之外的可见行数），保证列表排版大气且核心功能全部可达。
        let metrics = SettingsMetrics::new(96);
        let client = UiRect {
            left: 0,
            top: 0,
            right: metrics.desired_width,
            bottom: metrics.desired_height,
        };
        let rows = item_rows(&metrics, client, Page::Common.items(), None);
        let visible = rows
            .iter()
            .filter(|row| row.rect.bottom <= content_rect(&metrics, client).bottom)
            .count();
        assert!(visible >= 7, "常用设置页可见行不足 7 行：{visible}");
    }
}
