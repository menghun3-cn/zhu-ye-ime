//! 更新器桥（T-077 / FR-044，D-44）。
//!
//! 设置窗口**自身不发起任何网络请求**：检查与应用更新一律 spawn 独立的
//! `zhu-ye-updater.exe` 子进程并收集其输出；子进程完成检查/下载/应用后立即退出，
//! 不驻留（S-4：唯一联网组件边界）。
//!
//! `online_update` 默认关闭（P-03）：未开启时窗口不 spawn 任何进程、界面禁用
//! 检查/应用按钮并说明原因——保证"关闭时零出站连接"可测（验收 13.2）。
//!
//! 本模块只用 `std::process`，无 Windows API 依赖，可在无 GUI 环境单测。

use std::path::{Path, PathBuf};
use std::process::Command;

/// 更新器可执行文件名（与 `zhu-ye-updater` crate 的二进制名一致）。
pub const UPDATER_EXE_NAME: &str = "zhu-ye-updater.exe";

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
    let output = Command::new(program)
        .args(args)
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
        let program = Path::new("cmd.exe");
        let output = super::run(program, &["/C", "echo", "更新器输出"]).unwrap();
        assert!(output.contains("更新器输出"), "stdout 应被收集：{output}");
    }

    #[test]
    fn run非零退出返回错误且带回输出() {
        let program = Path::new("cmd.exe");
        let result = super::run(program, &["/C", "echo", "失败原因", "&", "exit", "3"]);
        match result {
            Err(text) => assert!(
                text.contains("失败原因"),
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
}
