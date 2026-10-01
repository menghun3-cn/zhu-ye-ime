//! 竹叶输入法设置窗口入口。
//!
//! 单实例：第二次唤起只激活已有窗口后退出（两次写 `config.json` 会互相覆盖）。
//! `--shot` 供验收取证：画出真实首帧后写出 BMP 并退出。

use std::path::PathBuf;
use std::process::ExitCode;

use zhu_ye_settings::model::Page;
use zhu_ye_settings::single_instance::{self, InstanceState};
use zhu_ye_settings::window::{self, RunOptions};

fn main() -> ExitCode {
    let options = match parse_args(std::env::args().skip(1)) {
        Ok(Some(options)) => options,
        Ok(None) => {
            print_usage();
            return ExitCode::SUCCESS;
        }
        Err(message) => {
            eprintln!("zhu-ye-settings: {message}");
            return ExitCode::from(2);
        }
    };

    // 凭据必须活到窗口退出：match 臂内绑定会在臂结束时释放互斥体。
    let guard = match single_instance::acquire() {
        Ok(InstanceState::Primary(guard)) => Some(guard),
        Ok(InstanceState::AlreadyRunning) => {
            if window::focus_existing() {
                println!("zhu-ye-settings: 已激活正在运行的设置窗口");
            } else {
                eprintln!("zhu-ye-settings: 已有实例在运行，但未找到其窗口");
            }
            return ExitCode::SUCCESS;
        }
        Err(message) => {
            // 单实例保证降级，但窗口仍可用：不因此拒绝启动。
            eprintln!("zhu-ye-settings: {message}（继续启动）");
            None
        }
    };

    let code = match window::run(options) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("zhu-ye-settings: {message}");
            ExitCode::from(1)
        }
    };
    drop(guard);
    code
}

/// 解析命令行。
///
/// 返回 `Ok(None)` 表示只要求打印用法。
fn parse_args(args: impl Iterator<Item = String>) -> Result<Option<RunOptions>, String> {
    let mut options = RunOptions::default();
    let mut args = args.peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--shot" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--shot 需要一个输出路径".to_owned())?;
                options.shot_path = Some(PathBuf::from(value));
            }
            "--page" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--page 需要一个页名".to_owned())?;
                options.shot_page = Some(parse_page(&value)?);
            }
            "--expand" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--expand 需要一个条目下标".to_owned())?;
                options.shot_expanded = Some(
                    value
                        .parse::<usize>()
                        .map_err(|_| format!("--expand 需要非负整数，收到：{value}"))?,
                );
            }
            "--help" | "-h" => return Ok(None),
            other => return Err(format!("未知参数：{other}")),
        }
    }
    Ok(Some(options))
}

fn parse_page(value: &str) -> Result<Page, String> {
    match value {
        "toolbox" => Ok(Page::Toolbox),
        "common" => Ok(Page::Common),
        "about" => Ok(Page::About),
        other => Err(format!("--page 仅支持 toolbox|common|about，收到：{other}")),
    }
}

fn print_usage() {
    println!("zhu-ye-settings — 竹叶输入法设置窗口");
    println!("用法:");
    println!("  zhu-ye-settings                     打开设置窗口");
    println!("  zhu-ye-settings --shot <文件.bmp>   画出首帧后写出 BMP 并退出（验收取证）");
    println!("                   [--page toolbox|common|about] [--expand <下标>]");
}
