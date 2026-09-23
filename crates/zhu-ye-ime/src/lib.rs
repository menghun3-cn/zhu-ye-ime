//! 竹叶输入法 TSF 适配层。
//!
//! 本 crate 编译为可供 TSF 宿主进程加载的 DLL。
//! M1 里程碑以实现可注册、可加载、可卸载、可上屏的 TSF 服务闭环为目标；
//! 候选窗与完整键位交互由 M3 任务接入。

#![allow(linker_messages)] // MSVC 创建 DLL 导入库时的正常输出，不视为警告

pub mod candidate_ui;
pub mod candidate_window;
pub mod input;
pub mod tsf;

pub use input::{m1_seed_dictionary, InputEngine, InputMode};
pub use tsf::{dll_probe, paired_core_version};
