//! 设置窗口的视图模型（纯逻辑，无 Win32 依赖；可无 GUI 单测）。
//!
//! 三页结构与条目清单的依据是 [设置窗口设计.md](../../../docs/设置窗口设计.md) §4 与
//! 需求 §17.4。本模块只描述"有哪些页、每页有哪些条目、条目处于什么状态"，
//! 不涉及绘制与命中测试。
//!
//! 条目状态区分三态，是为了不在骨架批次里把未接入的功能谎报为可用：
//! `Ready` 已接入；`Scheduled` 已在本阶段排期、由后续批次接入；`Planned` 是需求
//! §17.3 明确的本期非目标，界面按"正在规划中"呈现。

use zhu_ye_core::{ModeChoice, ThemeChoice};

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

/// 「更多设置」的打开动作（D-36）。
///
/// 以三个条目呈现，而不是一行三个按钮：条目行的控件机制只服务"二选一"，为三个动作再造
/// 一套按钮渲染不划算，三个条目在列表里同样一目了然。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenTarget {
    /// 配置文件 `config.json`。
    ConfigFile,
    /// 数据目录 `%APPDATA%\zhu-ye-ime`。
    DataDir,
    /// 文件日志所在目录。
    LogDir,
}

impl OpenTarget {
    /// 条目标题。
    #[must_use]
    pub const fn title(self) -> &'static str {
        match self {
            Self::ConfigFile => "更多设置：配置文件",
            Self::DataDir => "更多设置：数据目录",
            Self::LogDir => "更多设置：日志目录",
        }
    }

    /// 条目说明。
    #[must_use]
    pub const fn summary(self) -> &'static str {
        match self {
            Self::ConfigFile => "用系统默认方式打开 config.json（启用的领域包、主题等）",
            Self::DataDir => "配置、用户词库与已安装的领域包都在这里",
            Self::LogDir => {
                "产品日志目录（默认记录错误，级别见 config.json log_level）；首次写日志时自动创建"
            }
        }
    }
}

/// 内容区子视图（整区替换条目列表；互斥，最多一个生效）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Subview {
    /// 普通条目列表。
    #[default]
    None,
    /// 「添加词库」（FR-022 / FR-042）。
    Packs,
    /// 「管理输入法」（T-076 / FR-043：注册状态与打开系统输入法设置）。
    Manage,
    /// 「修复输入法」（T-076 / FR-043：一级/二级修复）。
    Repair,
    /// 「检查更新」（T-077 / FR-044：spawn 更新器检查与应用，窗口自身不联网）。
    Update,
    /// 「版本与诊断信息」（T-077 / FR-044：版本、路径与已装包列表）。
    Diagnostics,
    /// 「用户词表导入导出」（T-088 / FR-048）。
    UserWords,
    /// 「通讯录」（T-088 / FR-048：.vcf 界面化导入）。
    Contacts,
    /// 「自定义主题」（T-088 / FR-048：themes\*.json 加载与回退）。
    Themes,
}

/// 条目右侧控件。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemControl {
    /// 无控件。
    None,
    /// 主题二选一。
    ThemeChoice,
    /// 新会话默认中英模式二选一（D-32 装配项）。
    ModeChoice,
    /// 打开工具箱面板。
    OpenPanel(PanelKind),
    /// 打开一个路径。
    OpenPath(OpenTarget),
    /// 进入"添加词库"子视图（FR-022 领域包启停与导入，FR-042）。
    OpenPacks,
    /// 进入"管理输入法"子视图（T-076 / FR-043）。
    OpenManage,
    /// 进入"修复输入法"子视图（T-076 / FR-043）。
    OpenRepair,
    /// 恢复状态栏：重启输入法进程（ctfmon）重建语言栏；执行前弹确认（D-43）。
    RestoreLangBar,
    /// 在线更新开/关二选一（T-077 / FR-044；写 `config.json` 的 `online_update`，P-03 默认关）。
    OnlineUpdate,
    /// 候选框拼音行开/关二选一（T-127；写 `config.json` 的 `candidate_show_pin`，默认开）。
    CandidatePin,
    /// 进入"检查更新"子视图（T-077 / FR-044）。
    OpenUpdate,
    /// 进入"版本与诊断信息"子视图（T-077 / FR-044）。
    OpenDiagnostics,
    /// 进入"用户词表导入导出"子视图（T-088 / FR-048）。
    OpenUserWords,
    /// 进入"通讯录"子视图（T-088 / FR-048：.vcf 界面化导入）。
    OpenContacts,
    /// 进入"自定义主题"子视图（T-088 / FR-048：themes\*.json 加载与回退）。
    OpenThemes,
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
        "自定义主题",
        "从 themes\\*.json 加载配色；缺键回退预设，系统高对比度仍由系统接管（D-31）",
        ItemState::Ready,
        ItemControl::OpenThemes,
    ),
    item(
        "候选拼音",
        "候选框显示汉字上方的拼音及声调；保存后重启输入法生效",
        ItemState::Ready,
        ItemControl::CandidatePin,
    ),
    item(
        "英文输入法",
        "新会话默认中英模式；保存后重启输入法生效",
        ItemState::Ready,
        ItemControl::ModeChoice,
    ),
    item(
        "添加词库",
        "领域包启停与导入本地 .zyct",
        ItemState::Ready,
        ItemControl::OpenPacks,
    ),
    item(
        "用户词表",
        "导入导出 user_words.json；导入合并去重（同拼音同词取较大词频）",
        ItemState::Ready,
        ItemControl::OpenUserWords,
    ),
    item(
        "通讯录",
        "导入 .vcf 通讯录文件，生成联系人拼音候选",
        ItemState::Ready,
        ItemControl::OpenContacts,
    ),
    item(
        OpenTarget::ConfigFile.title(),
        OpenTarget::ConfigFile.summary(),
        ItemState::Ready,
        ItemControl::OpenPath(OpenTarget::ConfigFile),
    ),
    item(
        OpenTarget::DataDir.title(),
        OpenTarget::DataDir.summary(),
        ItemState::Ready,
        ItemControl::OpenPath(OpenTarget::DataDir),
    ),
    item(
        OpenTarget::LogDir.title(),
        OpenTarget::LogDir.summary(),
        ItemState::Ready,
        ItemControl::OpenPath(OpenTarget::LogDir),
    ),
    item(
        "恢复状态栏",
        "重启输入法进程，重建语言栏按钮",
        ItemState::Ready,
        ItemControl::RestoreLangBar,
    ),
    item(
        "管理输入法",
        "查看注册状态并打开系统输入法设置",
        ItemState::Ready,
        ItemControl::OpenManage,
    ),
    item(
        "修复输入法",
        "分级检测与修复",
        ItemState::Ready,
        ItemControl::OpenRepair,
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
        "启用在线更新",
        "由独立更新器进程联网；默认关闭，需显式开启（P-03）",
        ItemState::Ready,
        ItemControl::OnlineUpdate,
    ),
    item(
        "检查更新",
        "经独立更新器进程检查；窗口自身不联网",
        ItemState::Ready,
        ItemControl::OpenUpdate,
    ),
    item(
        "版本与诊断信息",
        "版本、配置路径、数据目录、日志目录、已装包",
        ItemState::Ready,
        ItemControl::OpenDiagnostics,
    ),
];

/// 设置窗口的交互状态（纯逻辑部分）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsState {
    /// 当前页。
    pub page: Page,
    /// 当前页内展开的条目下标；`None` 表示无展开。
    pub expanded: Option<usize>,
    /// 当前主题选择（来自 `config.json`）。
    pub theme: ThemeChoice,
    /// 新会话默认中英模式选择（来自 `config.json`，D-32 装配项）。
    pub default_mode: ModeChoice,
    /// 在线更新开关（来自 `config.json`，P-03 默认关）。
    pub online_update: bool,
    /// 候选框拼音行开关（来自 `config.json`，T-127 默认开；装配项）。
    pub candidate_show_pin: bool,
    /// 当前子视图（互斥）；子视图数据在窗口层，这里只记状态。
    pub subview: Subview,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            page: Page::Toolbox,
            expanded: None,
            theme: ThemeChoice::Light,
            default_mode: ModeChoice::Chinese,
            online_update: false,
            candidate_show_pin: true,
            subview: Subview::None,
        }
    }
}

impl SettingsState {
    /// 按已保存的主题建立状态（默认中英模式取配置装载后由窗口层覆盖）。
    #[must_use]
    pub fn new(theme: ThemeChoice) -> Self {
        Self {
            theme,
            ..Self::default()
        }
    }

    /// 按已保存的主题与默认中英模式建立状态。
    #[must_use]
    pub fn with_defaults(theme: ThemeChoice, default_mode: ModeChoice) -> Self {
        Self {
            theme,
            default_mode,
            ..Self::default()
        }
    }

    /// 按已保存的主题、默认中英模式、在线更新与候选拼音开关建立状态。
    #[must_use]
    pub fn with_config(
        theme: ThemeChoice,
        default_mode: ModeChoice,
        online_update: bool,
        candidate_show_pin: bool,
    ) -> Self {
        Self {
            theme,
            default_mode,
            online_update,
            candidate_show_pin,
            ..Self::default()
        }
    }

    /// 切换页面；展开态属于页内下标，切页时清空；子视图不跨页。
    pub fn select_page(&mut self, page: Page) {
        if self.page != page {
            self.page = page;
            self.expanded = None;
            self.subview = Subview::None;
        }
    }

    /// 进入「添加词库」子视图。
    pub fn open_packs(&mut self) {
        self.subview = Subview::Packs;
        self.expanded = None;
    }

    /// 进入「管理输入法」子视图。
    pub fn open_manage(&mut self) {
        self.subview = Subview::Manage;
        self.expanded = None;
    }

    /// 进入「修复输入法」子视图。
    pub fn open_repair(&mut self) {
        self.subview = Subview::Repair;
        self.expanded = None;
    }

    /// 进入「检查更新」子视图（T-077 / FR-044）。
    pub fn open_update(&mut self) {
        self.subview = Subview::Update;
        self.expanded = None;
    }

    /// 进入「版本与诊断信息」子视图（T-077 / FR-044）。
    pub fn open_diagnostics(&mut self) {
        self.subview = Subview::Diagnostics;
        self.expanded = None;
    }

    /// 进入「用户词表导入导出」子视图（T-088 / FR-048）。
    pub fn open_user_words(&mut self) {
        self.subview = Subview::UserWords;
        self.expanded = None;
    }

    /// 进入「通讯录」子视图（T-088 / FR-048）。
    pub fn open_contacts(&mut self) {
        self.subview = Subview::Contacts;
        self.expanded = None;
    }

    /// 进入「自定义主题」子视图（T-088 / FR-048）。
    pub fn open_themes(&mut self) {
        self.subview = Subview::Themes;
        self.expanded = None;
    }

    /// 退出子视图，回到条目列表。
    pub fn close_subview(&mut self) {
        self.subview = Subview::None;
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
    use super::{ItemControl, ItemState, OpenTarget, Page, SettingsState};
    use zhu_ye_core::{ModeChoice, ThemeChoice};

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
            "自定义主题",
            "英文输入法",
            "添加词库",
            "用户词表",
            "通讯录",
            "恢复状态栏",
            "管理输入法",
            "修复输入法",
            "简繁切换",
            "全半角切换",
            "生僻字输入",
        ] {
            assert!(titles.contains(&expected), "常用设置缺少条目：{expected}");
        }
        // 「更多设置」按 D-36 的三个动作拆成三个条目。
        for target in [
            OpenTarget::ConfigFile,
            OpenTarget::DataDir,
            OpenTarget::LogDir,
        ] {
            assert!(
                titles.contains(&target.title()),
                "缺少更多设置动作：{}",
                target.title()
            );
            assert!(!target.summary().is_empty());
        }
    }

    #[test]
    fn 更多设置三个动作都已接入且各自带打开控件() {
        let items = Page::Common.items();
        let mut found = 0;
        for target in [
            OpenTarget::ConfigFile,
            OpenTarget::DataDir,
            OpenTarget::LogDir,
        ] {
            let item = items
                .iter()
                .find(|item| item.title == target.title())
                .unwrap_or_else(|| panic!("缺少条目 {}", target.title()));
            assert_eq!(item.state, ItemState::Ready, "{} 应已接入", target.title());
            assert_eq!(item.control, ItemControl::OpenPath(target));
            found += 1;
        }
        assert_eq!(found, 3);
    }

    #[test]
    fn m12_4_已接入条目清单() {
        let ready: Vec<&str> = Page::ALL
            .iter()
            .flat_map(|page| page.items())
            .filter(|item| item.state == ItemState::Ready)
            .map(|item| item.title)
            .collect();
        assert_eq!(
            ready,
            vec![
                "emoji 面板",
                "符号大全",
                "主题",
                "自定义主题",
                "候选拼音",
                "英文输入法",
                "添加词库",
                "用户词表",
                "通讯录",
                "更多设置：配置文件",
                "更多设置：数据目录",
                "更多设置：日志目录",
                "恢复状态栏",
                "管理输入法",
                "修复输入法",
                "启用在线更新",
                "检查更新",
                "版本与诊断信息"
            ]
        );
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
    fn m12_4三项均已接入且控件齐备() {
        let items = Page::Common.items();
        let restore = items
            .iter()
            .find(|item| item.title == "恢复状态栏")
            .expect("缺少恢复状态栏条目");
        assert_eq!(restore.state, ItemState::Ready);
        assert_eq!(restore.control, ItemControl::RestoreLangBar);
        let manage = items
            .iter()
            .find(|item| item.title == "管理输入法")
            .expect("缺少管理输入法条目");
        assert_eq!(manage.state, ItemState::Ready);
        assert_eq!(manage.control, ItemControl::OpenManage);
        let repair = items
            .iter()
            .find(|item| item.title == "修复输入法")
            .expect("缺少修复输入法条目");
        assert_eq!(repair.state, ItemState::Ready);
        assert_eq!(repair.control, ItemControl::OpenRepair);
    }

    #[test]
    fn 英文输入法与添加词库条目形态() {
        let items = Page::Common.items();
        // D-32 装配项：新会话默认中英模式二选一，不再是会话状态。
        let english = items
            .iter()
            .find(|item| item.title == "英文输入法")
            .expect("缺少英文输入法条目");
        assert_eq!(english.state, ItemState::Ready);
        assert_eq!(english.control, ItemControl::ModeChoice);
        // FR-042：添加词库进入子视图，且已接入。
        let packs = items
            .iter()
            .find(|item| item.title == "添加词库")
            .expect("缺少添加词库条目");
        assert_eq!(packs.state, ItemState::Ready);
        assert_eq!(packs.control, ItemControl::OpenPacks);
    }

    #[test]
    fn 规划中条目说明以正在规划中开头() {
        let planned = Page::Common
            .items()
            .iter()
            .find(|item| item.state.is_planned())
            .expect("常用设置应有规划中条目");
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
        // 同页重复选择不清空（关于页现已全接入，用常用设置的规划中条目复验）。
        state.select_page(Page::Common);
        state.click_item(13);
        state.select_page(Page::Common);
        assert_eq!(state.expanded, Some(13));
    }

    #[test]
    fn 点击未接入条目切换展开而点击已接入条目无展开() {
        let mut state = SettingsState::new(ThemeChoice::Light);
        state.select_page(Page::Common);
        // 「简繁切换」仍是规划中条目：点击产生展开。
        let planned = 13;
        state.click_item(planned);
        assert_eq!(state.expanded, Some(planned));
        assert!(state
            .expanded_message()
            .unwrap()
            .starts_with("正在规划中。"));
        // 再点同一项收起。
        state.click_item(planned);
        assert_eq!(state.expanded, None);
        // 主题（index 0）与恢复状态栏（index 10）已接入：点击不产生展开。
        state.click_item(0);
        state.click_item(10);
        assert_eq!(state.expanded, None);
        assert_eq!(state.expanded_message(), None);
    }

    #[test]
    fn 子视图进入退出与切页清空() {
        use crate::model::Subview;

        let mut state = SettingsState::new(ThemeChoice::Light);
        assert_eq!(state.subview, Subview::None);
        // 三个子视图互斥进入。
        state.open_packs();
        assert_eq!(state.subview, Subview::Packs, "进入添加词库子视图");
        // 进入时清空展开态，避免与子视图叠加。
        state.click_item(13);
        state.open_manage();
        assert_eq!(state.subview, Subview::Manage, "管理输入法覆盖词库子视图");
        assert_eq!(state.expanded, None);
        state.open_repair();
        assert_eq!(state.subview, Subview::Repair);
        state.close_subview();
        assert_eq!(state.subview, Subview::None, "返回条目列表");
        // 子视图不跨页：直接切页也退出。
        state.open_manage();
        state.select_page(Page::About);
        assert_eq!(state.subview, Subview::None, "切页必须退出子视图");
    }

    #[test]
    fn 默认中英模式随配置装载() {
        let state = SettingsState::with_defaults(ThemeChoice::Dark, ModeChoice::English);
        assert_eq!(state.theme, ThemeChoice::Dark);
        assert_eq!(state.default_mode, ModeChoice::English);
        // 无装载时为中文（与其他旧字段同口径的默认值）。
        assert_eq!(
            SettingsState::new(ThemeChoice::Light).default_mode,
            ModeChoice::Chinese
        );
    }

    #[test]
    fn m12_5关于页三条目全部接入且控件齐备() {
        let items = Page::About.items();
        // 在线更新开关：P-03 默认关，界面显式开启。
        let online = items
            .iter()
            .find(|item| item.title == "启用在线更新")
            .expect("缺少启用在线更新条目");
        assert_eq!(online.state, ItemState::Ready);
        assert_eq!(online.control, ItemControl::OnlineUpdate);
        // 检查更新进入子视图。
        let update = items
            .iter()
            .find(|item| item.title == "检查更新")
            .expect("缺少检查更新条目");
        assert_eq!(update.state, ItemState::Ready);
        assert_eq!(update.control, ItemControl::OpenUpdate);
        // 版本与诊断信息进入子视图。
        let diagnostics = items
            .iter()
            .find(|item| item.title == "版本与诊断信息")
            .expect("缺少版本与诊断信息条目");
        assert_eq!(diagnostics.state, ItemState::Ready);
        assert_eq!(diagnostics.control, ItemControl::OpenDiagnostics);
    }

    #[test]
    fn 在线更新开关默认关闭且随配置装载() {
        // P-03：默认关闭，未显式开启不得联网。
        let state = SettingsState::default();
        assert!(
            !state.online_update,
            "online_update 默认必须为 false（P-03）"
        );
        assert!(!SettingsState::new(ThemeChoice::Light).online_update);
        let state = SettingsState::with_config(ThemeChoice::Dark, ModeChoice::English, true, false);
        assert!(state.online_update, "显式开启后应为 true");
        assert!(!state.candidate_show_pin, "候选拼音开关随配置装载（T-127）");
    }

    #[test]
    fn 候选拼音开关默认显示且随配置装载() {
        // T-127：默认显示（历史行为）；显式关闭随配置装载。
        let state = SettingsState::default();
        assert!(state.candidate_show_pin);
        assert!(SettingsState::new(ThemeChoice::Light).candidate_show_pin);
        let state =
            SettingsState::with_config(ThemeChoice::Light, ModeChoice::Chinese, false, false);
        assert!(!state.candidate_show_pin, "显式关闭后应为 false");
        let state =
            SettingsState::with_config(ThemeChoice::Light, ModeChoice::Chinese, false, true);
        assert!(state.candidate_show_pin, "显式开启后应为 true");
    }

    #[test]
    fn 更新与诊断子视图进入退出() {
        use crate::model::Subview;

        let mut state = SettingsState::new(ThemeChoice::Light);
        assert_eq!(state.subview, Subview::None);
        state.open_update();
        assert_eq!(state.subview, Subview::Update, "进入检查更新子视图");
        state.open_diagnostics();
        assert_eq!(state.subview, Subview::Diagnostics, "诊断子视图互斥覆盖");
        state.close_subview();
        assert_eq!(state.subview, Subview::None);
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
