//! 竹叶输入法设置窗口入口。
//!
//! 单实例：第二次唤起只激活已有窗口后退出（两次写 `config.json` 会互相覆盖）。
//! `--shot` / `--shot-panel` 供验收取证：画出真实首帧后写出 BMP 并退出。

use std::path::PathBuf;
use std::process::ExitCode;

use zhu_ye_settings::model::Page;
use zhu_ye_settings::panel::{PanelKind, PanelView};
use zhu_ye_settings::panel_window;
use zhu_ye_settings::single_instance::{self, InstanceState};
use zhu_ye_settings::window::{self, RunOptions};

/// 一次运行要做的事。
enum Mode {
    /// 打开设置窗口。
    Settings(Box<RunOptions>),
    /// 画出工具箱面板的一帧并写出 BMP（验收取证）。
    PanelShot { view: PanelView, path: PathBuf },
}

fn main() -> ExitCode {
    let mode = match parse_args(std::env::args().skip(1)) {
        Ok(Some(mode)) => mode,
        Ok(None) => {
            print_usage();
            return ExitCode::SUCCESS;
        }
        Err(message) => {
            eprintln!("zhu-ye-settings: {message}");
            return ExitCode::from(2);
        }
    };

    // 取证模式不落任何状态，也不与正在运行的设置窗口互斥。
    let options = match mode {
        Mode::PanelShot { view, path } => {
            return match panel_window::run_shot(view, &path) {
                Ok(()) => ExitCode::SUCCESS,
                Err(message) => {
                    eprintln!("zhu-ye-settings: {message}");
                    ExitCode::from(1)
                }
            };
        }
        Mode::Settings(options) => *options,
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
fn parse_args(args: impl Iterator<Item = String>) -> Result<Option<Mode>, String> {
    let mut options = RunOptions::default();
    let mut shot_panel: Option<PanelKind> = None;
    let mut panel_path: Option<PathBuf> = None;
    let mut panel_page = 0usize;
    let mut args = args.peekable();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--shot" => {
                options.shot_path = Some(PathBuf::from(
                    args.next()
                        .ok_or_else(|| "--shot 需要一个输出路径".to_owned())?,
                ));
            }
            "--page" => {
                options.shot_page = Some(parse_page(&next_value(&mut args, "--page")?)?);
            }
            "--expand" => {
                options.shot_expanded = Some(
                    next_value(&mut args, "--expand")?
                        .parse::<usize>()
                        .map_err(|_| "「--expand」需要非负整数".to_owned())?,
                );
            }
            "--packs" => {
                options.shot_packs = true;
            }
            "--shot-panel" => {
                let kind = match next_value(&mut args, "--shot-panel")?.as_str() {
                    "emoji" => PanelKind::Emoji,
                    "symbol" => PanelKind::Symbol,
                    other => {
                        return Err(format!("--shot-panel 仅支持 emoji|symbol，收到：{other}"))
                    }
                };
                shot_panel = Some(kind);
                panel_path = Some(PathBuf::from(next_value(
                    &mut args,
                    "--shot-panel 的输出路径",
                )?));
            }
            "--panel-page" => {
                panel_page = next_value(&mut args, "--panel-page")?
                    .parse::<usize>()
                    .map_err(|_| "「--panel-page」需要非负整数".to_owned())?;
            }
            "--help" | "-h" => return Ok(None),
            other => return Err(format!("未知参数：{other}")),
        }
    }

    if let (Some(kind), Some(path)) = (shot_panel, panel_path) {
        if options.shot_path.is_some() {
            return Err("--shot 与 --shot-panel 不能同时使用".to_owned());
        }
        return Ok(Some(Mode::PanelShot {
            view: PanelView {
                kind,
                page: panel_page,
            },
            path,
        }));
    }
    Ok(Some(Mode::Settings(Box::new(options))))
}

fn next_value(
    args: &mut std::iter::Peekable<impl Iterator<Item = String>>,
    flag: &str,
) -> Result<String, String> {
    args.next().ok_or_else(|| format!("{flag} 需要一个值"))
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
    println!("  zhu-ye-settings --shot <文件.bmp>   画出设置窗口首帧并写出 BMP（验收取证）");
    println!("                   [--page toolbox|common|about] [--expand <下标>]");
    println!("                   [--packs]              截图时进入「添加词库」子视图");
    println!("  zhu-ye-settings --shot-panel emoji|symbol <文件.bmp>");
    println!("                   [--panel-page <页码>]  画出工具箱面板一帧并写出 BMP");
}
