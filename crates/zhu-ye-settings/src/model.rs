//! 设置窗口的视图模型（纯逻辑，无 Win32 依赖；可无 GUI 单测）。
//!
//! 三页结构与条目清单的依据是 [设置窗口设计.md](../../../docs/设置窗口设计.md) §4 与
//! 需求 §17.4。本模块只描述"有哪些页、每页有哪些条目、条目处于什么状态"，
//! 不涉及绘制与命中测试。
//!
//! 条目状态区分三态，是为了不在骨架批次里把未接入的功能谎报为可用：
//! `Ready` 已接入；`Scheduled` 已在本阶段排期、由后续批次接入；`Planned` 是需求
//! §17.3 明确的本期非目标，界面按"正在规划中"呈现。

use zhu_ye_core::ThemeChoice;

use crate::panel::PanelKind;

/// 设置窗口页面。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    /// 工具箱（FR-040）。
    Toolbox,
    /// 常用设置（FR-041、FR-042）。
    Common,
    /// 关于与更新（FR-044）。
    About,
}

impl Page {
    /// 导航顺序。
    pub const ALL: [Page; 3] = [Page::Toolbox, Page::Common, Page::About];

    /// 导航项与页标题文案。
    #[must_use]
    pub const fn title(self) -> &'static str {
        match self {
            Self::Toolbox => "工具箱",
            Self::Common => "常用设置",
            Self::About => "关于与更新",
        }
    }

    /// 该页条目。
    #[must_use]
    pub const fn items(self) -> &'static [Item] {
        match self {
            Self::Toolbox => TOOLBOX_ITEMS,
            Self::Common => COMMON_ITEMS,
            Self::About => ABOUT_ITEMS,
        }
    }
}

/// 条目状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemState {
    /// 已接入：本批次即可交互。
    Ready,
    /// 本阶段已排期，由该批次接入。
    Scheduled {
        /// 批次号（如 `M12-2`）。
        batch: &'static str,
    },
    /// 需求 §17.3 明确的本期非目标，后端尚未实现。
    Planned {
        /// 规划说明（不含"正在规划中"前缀）。
        note: &'static str,
    },
}

impl ItemState {
    /// 展开说明；`Ready` 条目没有可展开内容。
    #[must_use]
    pub fn message(self) -> Option<String> {
        match self {
            Self::Ready => None,
            Self::Scheduled { batch } => {
                Some(format!("本功能将在 {batch} 批次接入（本阶段已排期）。"))
            }
            Self::Planned { note } => Some(format!("正在规划中。{note}")),
        }
    }

    /// 是否为"正在规划中"的占位（区别于本阶段已排期的条目）。
    #[must_use]
    pub const fn is_planned(self) -> bool {
        matches!(self, Self::Planned { .. })
    }
}

/// 条目右侧控件。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemControl {
    /// 无控件。
    None,
    /// 主题二选一。
    ThemeChoice,
    /// 打开工具箱面板。
    OpenPanel(PanelKind),
}

/// 条目定义。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Item {
    /// 标题。
    pub title: &'static str,
    /// 一行说明。
    pub summary: &'static str,
    /// 状态。
    pub state: ItemState,
    /// 右侧控件。
    pub control: ItemControl,
}

const fn item(
    title: &'static str,
    summary: &'static str,
    state: ItemState,
    control: ItemControl,
) -> Item {
    Item {
        title,
        summary,
        state,
        control,
    }
}

/// 工具箱页条目（FR-040）。
static TOOLBOX_ITEMS: &[Item] = &[
    item(
        "emoji 面板",
        "浏览内置 emoji，选中即上屏",
        ItemState::Ready,
        ItemControl::OpenPanel(PanelKind::Emoji),
    ),
    item(
        "符号大全",
        "按分组浏览符号，选中即上屏",
        ItemState::Ready,
        ItemControl::OpenPanel(PanelKind::Symbol),
    ),
    item(
        "图片表情",
        "规划中",
        ItemState::Planned {
            note: "TSF 只能向宿主插入文本，无法插入图片；未来最多提供“复制到剪贴板”。",
        },
        ItemControl::None,
    ),
];

/// 常用设置页条目（FR-041、FR-042）。
static COMMON_ITEMS: &[Item] = &[
    item(
        "主题",
        "候选窗配色；保存后重启输入法生效",
        ItemState::Ready,
        ItemControl::ThemeChoice,
    ),
    item(
        "英文输入法",
        "显示当前中英模式并立即切换（会话状态，不记忆）",
        ItemState::Scheduled { batch: "M12-3" },
        ItemControl::None,
    ),
    item(
        "添加词库",
        "领域包启停与本地 .zyct 导入",
        ItemState::Scheduled { batch: "M12-3" },
        ItemControl::None,
    ),
    item(
        "更多设置",
        "打开配置文件、数据目录与日志目录",
        ItemState::Scheduled { batch: "M12-3" },
        ItemControl::None,
    ),
    item(
        "恢复状态栏",
        "重新注册语言栏按钮",
        ItemState::Scheduled { batch: "M12-4" },
        ItemControl::None,
    ),
    item(
        "管理输入法",
        "查看注册状态并打开系统输入法设置",
        ItemState::Scheduled { batch: "M12-4" },
        ItemControl::None,
    ),
    item(
        "修复输入法",
        "分级检测与修复",
        ItemState::Scheduled { batch: "M12-4" },
        ItemControl::None,
    ),
    item(
        "简繁切换",
        "规划中",
        ItemState::Planned {
            note: "需要独立的简繁转换表与上屏路径改造，属独立课题。",
        },
        ItemControl::None,
    ),
    item(
        "全半角切换",
        "规划中",
        ItemState::Planned {
            note: "标点目前一律放行宿主直出，尚无全半角开关。",
        },
        ItemControl::None,
    ),
    item(
        "生僻字输入",
        "规划中",
        ItemState::Planned {
            note: "需要部件或笔画检索，或扩展字符集，属独立课题。",
        },
        ItemControl::None,
    ),
];

/// 关于与更新页条目（FR-044）。
static ABOUT_ITEMS: &[Item] = &[
    item(
        "检查更新",
        "经独立更新器进程检查；窗口自身不联网",
        ItemState::Scheduled { batch: "M12-5" },
        ItemControl::None,
    ),
    item(
        "版本与诊断信息",
        "版本、配置路径、数据目录、日志目录、已装包",
        ItemState::Scheduled { batch: "M12-5" },
        ItemControl::None,
    ),
];

/// 设置窗口的交互状态（纯逻辑部分）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingsState {
    /// 当前页。
    pub page: Page,
    /// 当前页内展开的条目下标；`None` 表示无展开。
    pub expanded: Option<usize>,
    /// 当前主题选择（来自 `config.json`）。
    pub theme: ThemeChoice,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            page: Page::Toolbox,
            expanded: None,
            theme: ThemeChoice::Light,
        }
    }
}

impl SettingsState {
    /// 按已保存的主题建立状态。
    #[must_use]
    pub fn new(theme: ThemeChoice) -> Self {
        Self {
            theme,
            ..Self::default()
        }
    }

    /// 切换页面；展开态属于页内下标，切页时清空。
    pub fn select_page(&mut self, page: Page) {
        if self.page != page {
            self.page = page;
            self.expanded = None;
        }
    }

    /// 点击条目：仅未接入的条目切换展开说明，已接入条目的动作由调用方执行。
    pub fn click_item(&mut self, index: usize) {
        let Some(item) = self.page.items().get(index) else {
            return;
        };
        if item.state == ItemState::Ready {
            return;
        }
        self.expanded = if self.expanded == Some(index) {
            None
        } else {
            Some(index)
        };
    }

    /// 当前展开条目的说明文案。
    #[must_use]
    pub fn expanded_message(&self) -> Option<String> {
        let index = self.expanded?;
        self.page.items().get(index)?.state.message()
    }
}

#[cfg(test)]
mod tests {
    use super::{ItemControl, ItemState, Page, SettingsState};
    use zhu_ye_core::ThemeChoice;

    #[test]
    fn 三页标题与条目齐备() {
        let titles: Vec<&str> = Page::ALL.iter().map(|page| page.title()).collect();
        assert_eq!(titles, vec!["工具箱", "常用设置", "关于与更新"]);
        for page in Page::ALL {
            assert!(!page.items().is_empty(), "{} 页不得为空", page.title());
        }
    }

    #[test]
    fn 常用设置覆盖需求列出的全部条目() {
        let titles: Vec<&str> = Page::Common.items().iter().map(|item| item.title).collect();
        for expected in [
            "主题",
            "英文输入法",
            "添加词库",
            "恢复状态栏",
            "管理输入法",
            "修复输入法",
            "更多设置",
            "简繁切换",
            "全半角切换",
            "生僻字输入",
        ] {
            assert!(titles.contains(&expected), "常用设置缺少条目：{expected}");
        }
    }

    #[test]
    fn 已接入条目恰为工具箱两项与主题() {
        let ready: Vec<&str> = Page::ALL
            .iter()
            .flat_map(|page| page.items())
            .filter(|item| item.state == ItemState::Ready)
            .map(|item| item.title)
            .collect();
        assert_eq!(ready, vec!["emoji 面板", "符号大全", "主题"]);
        // 主题用二选一控件；工具箱两项用"打开面板"控件。
        assert_eq!(Page::Common.items()[0].control, ItemControl::ThemeChoice);
        assert_eq!(
            Page::Toolbox.items()[0].control,
            ItemControl::OpenPanel(crate::panel::PanelKind::Emoji)
        );
        assert_eq!(
            Page::Toolbox.items()[1].control,
            ItemControl::OpenPanel(crate::panel::PanelKind::Symbol)
        );
        // 图片表情仍是"正在规划中"，且没有控件。
        assert!(Page::Toolbox.items()[2].state.is_planned());
        assert_eq!(Page::Toolbox.items()[2].control, ItemControl::None);
    }

    #[test]
    fn 规划中条目说明以正在规划中开头() {
        let planned = Page::Common.items()[7];
        assert!(planned.state.is_planned());
        let message = planned.state.message().unwrap();
        assert!(message.starts_with("正在规划中。"), "实际：{message}");
    }

    #[test]
    fn 已排期条目说明含批次号() {
        let message = ItemState::Scheduled { batch: "M12-2" }.message().unwrap();
        assert!(message.contains("M12-2"), "实际：{message}");
        assert!(!ItemState::Scheduled { batch: "M12-2" }.is_planned());
        // 已接入条目没有可展开内容。
        assert_eq!(ItemState::Ready.message(), None);
    }

    #[test]
    fn 切页清空展开态() {
        let mut state = SettingsState::new(ThemeChoice::Light);
        // 工具箱首页前两项已接入（点开面板），只有"图片表情"仍是占位可展开。
        let placeholder = 2;
        state.click_item(placeholder);
        assert_eq!(state.expanded, Some(placeholder));
        state.select_page(Page::About);
        assert_eq!(state.expanded, None, "展开态属于页内下标，切页必须清空");
        // 同页重复选择不清空。
        state.click_item(0);
        state.select_page(Page::About);
        assert_eq!(state.expanded, Some(0));
    }

    #[test]
    fn 点击未接入条目切换展开而点击已接入条目无展开() {
        let mut state = SettingsState::new(ThemeChoice::Light);
        state.select_page(Page::Common);
        state.click_item(1);
        assert_eq!(state.expanded, Some(1));
        assert!(state.expanded_message().unwrap().contains("M12-3"));
        // 再点同一项收起。
        state.click_item(1);
        assert_eq!(state.expanded, None);
        // 主题（index 0）已接入：点击不产生展开。
        state.click_item(0);
        assert_eq!(state.expanded, None);
        assert_eq!(state.expanded_message(), None);
    }

    #[test]
    fn 越界下标不产生展开也不panic() {
        let mut state = SettingsState::new(ThemeChoice::Light);
        state.click_item(999);
        assert_eq!(state.expanded, None);
        assert_eq!(
            SettingsState {
                expanded: Some(999),
                ..state
            }
            .expanded_message(),
            None
        );
    }
}
