//! 竹叶输入法设置窗口（第八期，FR-039 至 FR-045）。
//!
//! 独立进程的原生 Win32 窗口：不依赖任何 GUI 框架，复用 `zhu_ye_ime::candidate_ui`
//! 的纯逻辑主题与度量。模块按"可无 GUI 单测"与"必须 Win32"分层：
//!
//! - 纯逻辑：`model`（三页与条目）、`layout`（矩形计算）、`theme`（配色）、`config`（配置读写）、
//!   `panel`（工具箱面板的数据与分页）、`installed`（已安装包清单）、`inventory`（领域包清单）
//! - Win32：`shell`（系统深浅色/高对比度/DPI）、`single_instance`、`window`、`panel_window`、
//!   `gdi`（绘制原语）、`deliver`（字符投递）
//!
//! 设计与需求依据见 docs/设置窗口设计.md 与需求规格说明书 §17。

pub mod config;
pub mod deliver;
pub mod gdi;
pub mod installed;
pub mod inventory;
pub mod layout;
pub mod model;
pub mod panel;
pub mod panel_window;
pub mod registry;
pub mod repair;
pub mod shell;
pub mod single_instance;
pub mod theme;
pub mod wide;
pub mod window;

pub use inventory::PackInfo;
pub use model::{Item, ItemControl, ItemState, Page, SettingsState, Subview};
pub use panel::{PanelEntry, PanelKind, PanelView};
pub use registry::{re_register, restart_ctfmon, run_repair_default};
pub use repair::{evaluate_registration, scan_l1, L1Outcome, RegistrationStatus, RepairScan};
