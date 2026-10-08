//! 候选窗演示程序：验证主题、DPI、译文层渲染与截图输出。
//!
//! 用法示例：
//! candidate-demo --theme dark --dpi 192 --seconds 2 --shot target/candidate-dark.bmp
//!
//! 本二进制通过 `#[path]` 独立编译候选窗模块；TSF 受控窗口只属于库目标，
//! 因此在演示拷贝里允许 dead_code，避免重复编译产生无关告警。
#![allow(dead_code)]

use std::path::PathBuf;

use zhu_ye_core::candidate::CandidateSource;

use candidate_ui::{CandidateUiItem, CandidateUiView, DEFAULT_PAGE_SIZE};
use candidate_window::{run_candidate_demo, CandidateWindowOptions, ThemePreference};

#[path = "../candidate_ui.rs"]
mod candidate_ui;

#[path = "../candidate_window.rs"]
mod candidate_window;

#[path = "../color_text.rs"]
mod color_text;

fn main() {
    if let Err(error) = run() {
        eprintln!("候选窗演示失败: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let mut theme = ThemePreference::Auto;
    let mut dpi = None;
    let mut seconds = None;
    let mut shot_path = None;
    let mut translation_mode = false;
    let mut emoji_row = false;
    let mut long_input = false;

    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--theme" => {
                let value = take_value(&args, &mut index, "--theme")?;
                theme = parse_theme(&value)?;
            }
            "--dpi" => {
                let value = take_value(&args, &mut index, "--dpi")?;
                dpi = Some(
                    value
                        .parse::<u32>()
                        .map_err(|_| format!("无效 DPI: {value}"))?,
                );
            }
            "--seconds" => {
                let value = take_value(&args, &mut index, "--seconds")?;
                seconds = Some(
                    value
                        .parse::<u64>()
                        .map_err(|_| format!("无效秒数: {value}"))?,
                );
            }
            "--shot" => {
                let value = take_value(&args, &mut index, "--shot")?;
                shot_path = Some(PathBuf::from(value));
            }
            "--translation-mode" => translation_mode = true,
            "--emoji" => emoji_row = true,
            "--long" => long_input = true,
            "--help" | "-h" => {
                println!(
                    "用法: candidate-demo [--theme auto|light|dark|high-contrast] \
                     [--dpi <px>] [--seconds <s>] [--shot <bmp>] [--translation-mode] \
                     [--emoji] [--long]"
                );
                return Ok(());
            }
            other => return Err(format!("未知参数: {other}，使用 --help 查看用法")),
        }
        index += 1;
    }

    let options = CandidateWindowOptions {
        theme,
        dpi,
        seconds,
        shot_path,
        custom_theme: None,
    };
    run_candidate_demo(demo_view(translation_mode, emoji_row, long_input), &options)
}

fn take_value(args: &[String], index: &mut usize, arg: &str) -> Result<String, String> {
    *index += 1;
    args.get(*index)
        .cloned()
        .ok_or_else(|| format!("缺少 {arg} 的值"))
}

fn parse_theme(value: &str) -> Result<ThemePreference, String> {
    match value {
        "auto" => Ok(ThemePreference::Auto),
        "light" => Ok(ThemePreference::Light),
        "dark" => Ok(ThemePreference::Dark),
        "high-contrast" => Ok(ThemePreference::HighContrast),
        other => Err(format!(
            "未知主题 {other:?}，可选 auto/light/dark/high-contrast"
        )),
    }
}

fn demo_view(translation_mode: bool, emoji_row: bool, long_input: bool) -> CandidateUiView {
    // T-122：演示项带带调拼音，便于截图验收拼音行（字号/字重/对比度/行距）。
    let mut items = vec![
        item("你好", "Hello", "nǐ hǎo", CandidateSource::Static),
        item(
            "你们好",
            "Hello everyone",
            "nǐ men hǎo",
            CandidateSource::Static,
        ),
        item("你好呀", "Hi there", "", CandidateSource::User),
        item("耐火", "Fire-resistant", "", CandidateSource::Static),
        item("拟稿", "Draft", "", CandidateSource::Static),
    ];
    if emoji_row {
        // T-074：注入彩色 emoji 候选行供截图验收（其余行保持 GDI 基线）。
        items.insert(2, item("😂 笑声😀 开心", "", "", CandidateSource::Static));
        items.push(item("完成 🚀", "", "", CandidateSource::Static));
    }
    items.extend([
        item("溺爱", "Dote on", "", CandidateSource::Static),
        item("逆光", "Backlight", "", CandidateSource::Static),
        item("泥泞", "Muddy", "", CandidateSource::Static),
        item("妮好", "", "", CandidateSource::Static),
    ]);
    // T-124：长拼音输入场景——组合串超固定 360dp 面板输入串区
    // （约 141dp）会被右侧省略号截断；带调提示同时存在以复现用户反馈
    // （`youmeiyoushenmeren` → `youmeiyoushenmere...`）。
    let (composition, pinyin_hint) = if long_input {
        (
            "youmeiyoushenmeren".to_owned(),
            "yǒu méi yǒu shén me rén".to_owned(),
        )
    } else {
        ("ni hao".to_owned(), "ni hao".to_owned())
    };
    CandidateUiView {
        composition,
        pinyin_hint,
        page: 0,
        page_size: DEFAULT_PAGE_SIZE,
        page_count: 1,
        selected: 0,
        translation_mode,
        items,
    }
}

fn item(
    text: &str,
    translation: &str,
    pinyin_tone: &str,
    source: CandidateSource,
) -> CandidateUiItem {
    CandidateUiItem {
        text: text.to_owned(),
        translation: translation.to_owned(),
        pinyin: String::new(),
        pinyin_tone: pinyin_tone.to_owned(),
        source,
    }
}
