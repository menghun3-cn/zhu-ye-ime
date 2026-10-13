//! 更新器桥（T-077 / FR-044，D-44）。
//!
//! 设置窗口**自身不发起任何网络请求**：检查与应用更新一律 spawn 独立的
//! `zhu-ye-updater.exe` 子进程并收集其输出；子进程完成检查/下载/应用后立即退出，
//! 不驻留（S-4：唯一联网组件边界）。
//!
//! `online_update` 默认关闭（P-03）：未开启时窗口不 spawn 任何进程、界面禁用
//! 检查/应用按钮并说明原因——保证"关闭时零出站连接"可测（验收 13.2）。
//!
//! 本模块只用 `std::process`，无第三方 Windows API 依赖，可在无 GUI 环境单测。

use std::path::{Path, PathBuf};
use std::process::Command;

/// 更新器可执行文件名（与 `zhu-ye-updater` crate 的二进制名一致）。
pub const UPDATER_EXE_NAME: &str = "zhu-ye-updater.exe";

/// 让控制台子系统子进程不新建可见控制台窗口（T-153）。
///
/// `zhu-ye-updater.exe` 是 CON 子系统程序；设置窗口（GUI 子系统进程）spawn 它时
/// 若不给标志，Windows 会为它**新建一个控制台窗口**——「检查更新/应用更新」时
/// 闪现的黑框 cmd 就是它。`CREATE_NO_WINDOW`（0x08000000，WinBase.h）让新控制台
/// 不创建可见窗口；输出仍经管道收集，行为不变。扩展来自 std 自带
/// `std::os::windows::process::CommandExt`，不引入第三方依赖。
#[cfg(windows)]
fn suppress_console_window(command: &mut Command) {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    command.creation_flags(CREATE_NO_WINDOW);
}

/// 非 Windows 平台无窗口概念，无操作（保持跨平台可编译）。
#[cfg(not(windows))]
fn suppress_console_window(_command: &mut Command) {}

/// 与本可执行文件同目录的更新器路径；不存在返回 `None`（未安装或开发目录缺失）。
#[must_use]
pub fn updater_exe_path() -> Option<PathBuf> {
    let path = crate::shell::exe_dir().join(UPDATER_EXE_NAME);
    path.is_file().then_some(path)
}

/// 运行更新器子命令并等待完成，返回合并的 stdout+stderr 文本。
///
/// 联网动作发生在**子进程**内；本函数自身不发起网络请求。非零退出码返回 `Err`，
/// 文本同样带回（供界面展示失败原因）。
///
/// # Errors
/// 进程无法启动，或退出码非零。
pub fn run(program: &Path, args: &[&str]) -> Result<String, String> {
    let mut command = Command::new(program);
    command.args(args);
    suppress_console_window(&mut command);
    let output = command
        .output()
        .map_err(|error| format!("无法启动更新器进程: {error}"))?;
    let mut text = String::new();
    text.push_str(&String::from_utf8_lossy(&output.stdout));
    text.push_str(&String::from_utf8_lossy(&output.stderr));
    if output.status.success() {
        Ok(text)
    } else {
        Err(text)
    }
}

/// 把更新器输出整理为界面的逐行文本：去掉空行与行尾空白（纯逻辑，便于单测）。
#[must_use]
pub fn output_lines(text: &str) -> Vec<String> {
    text.lines()
        .map(str::trim_end)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::output_lines;
    use std::path::Path;

    #[test]
    fn 输出整理去掉空行与行尾空白() {
        let lines = output_lines("  第一行  \r\n\n第二行\r\n\r\n");
        assert_eq!(lines, vec!["  第一行", "第二行"], "保留行内缩进、去空行");
    }

    #[test]
    fn 空输出为空列表() {
        assert!(output_lines("").is_empty());
        assert!(output_lines("\r\n\r\n").is_empty());
    }

    #[test]
    fn run正常退出返回输出() {
        // 注意：替身子进程用 cmd.exe，断言只用 ASCII——无窗口标志下子进程拥有
        // 独立新控制台，代码页回到系统默认（936），中文会以 GBK 字节写出；
        // 真实更新器是 Rust println!（UTF-8 写管道），不受代码页影响。
        let program = Path::new("cmd.exe");
        let output = super::run(program, &["/C", "echo", "updater-output"]).unwrap();
        assert!(
            output.contains("updater-output"),
            "stdout 应被收集：{output}"
        );
    }

    #[test]
    fn run非零退出返回错误且带回输出() {
        let program = Path::new("cmd.exe");
        let result = super::run(program, &["/C", "echo", "failure-reason", "&", "exit", "3"]);
        match result {
            Err(text) => assert!(
                text.contains("failure-reason"),
                "非零退出也要带回输出供界面展示：{text}"
            ),
            Ok(_) => panic!("exit 3 必须报错"),
        }
    }

    #[test]
    fn run启动失败返回错误() {
        let result = super::run(Path::new("不存在的程序名-xyz.exe"), &[]);
        assert!(result.is_err(), "程序不存在必须报错");
    }

    /// 无窗口标志下输出收集与退出码语义不变（T-153）：「检查更新/应用更新」
    /// 不再闪现 cmd 黑窗，但界面仍能拿到更新器输出。窗口本身是否存在属于
    /// 桌面观察范畴，由真机目视（验收 §21）兜底。
    #[cfg(windows)]
    #[test]
    fn 无窗口标志下子进程输出收集正常() {
        let program = Path::new("cmd.exe");
        let output = super::run(program, &["/C", "echo", "no-window-ok"]).unwrap();
        assert!(
            output.contains("no-window-ok"),
            "输出仍应被管道收集：{output}"
        );
    }
}
