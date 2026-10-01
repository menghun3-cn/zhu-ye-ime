//! 竹叶输入法设置窗口（第八期，FR-039 至 FR-045）。
//!
//! 独立进程的原生 Win32 窗口：不依赖任何 GUI 框架，复用 `zhu_ye_ime::candidate_ui`
//! 的纯逻辑主题与度量。模块按"可无 GUI 单测"与"必须 Win32"分层：
//!
//! - 纯逻辑：`model`（三页与条目）、`layout`（矩形计算）、`theme`（配色）、`config`（配置读写）
//! - Win32：`shell`（系统深浅色/高对比度/DPI）、`single_instance`、`window`
//!
//! 设计与需求依据见 docs/设置窗口设计.md 与需求规格说明书 §17。

pub mod config;
pub mod layout;
pub mod model;
pub mod shell;
pub mod single_instance;
pub mod theme;
pub mod wide;
pub mod window;

pub use model::{Item, ItemControl, ItemState, Page, SettingsState};
