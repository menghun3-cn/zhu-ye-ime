//! 词典构建 CLI：build / inspect / verify。

use std::path::{Path, PathBuf};

use zhu_ye_core::bigram::BigramModel;
use zhu_ye_core::dict::Dictionary;
use zhu_ye_core::dict_format::DictHeader;
use zhu_ye_core::dict_loader::DictionaryFile;
use zhu_ye_core::translate::Translator;
use zhu_ye_dict::{build_v2, dict_schema_version, pipeline_status, seed_bigrams, seed_entries};

/// 默认构建产物路径；`data/artifacts/` 已由 `.gitignore` 排除。
const DEFAULT_OUTPUT: &str = "data/artifacts/seed.zyct";

/// 反查校验样例：归一化英文 -> 中文词。
const REVERSE_SAMPLES: &[(&str, &str)] = &[
    ("hello", "你好"),
    ("china", "中国"),
    ("good morning", "早上好"),
];

fn main() {
    if let Err(message) = run() {
        eprintln!("zhu-ye-dict: {message}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("build") => build_command(args.get(2).map(PathBuf::from)),
        Some("inspect") => inspect_command(required_path(&args, 2)?),
        Some("verify") => verify_command(required_path(&args, 2)?),
        _ => {
            print_usage();
            Ok(())
        }
    }
}

fn print_usage() {
    println!("zhu-ye-dict 命令：");
    println!("  build [输出路径]      构建自建演示种子词典（默认 {DEFAULT_OUTPUT}）");
    println!("  inspect <文件>        打印词典头部元数据与内容哈希");
    println!("  verify <文件>         完整加载校验并核对种子词条/bigram/翻译");
}

fn required_path(args: &[String], index: usize) -> Result<PathBuf, String> {
    args.get(index)
        .map(PathBuf::from)
        .ok_or_else(|| "缺少词典文件参数".to_owned())
}

fn build_command(output: Option<PathBuf>) -> Result<(), String> {
    let output = output.unwrap_or_else(|| PathBuf::from(DEFAULT_OUTPUT));
    let entries = seed_entries();
    let bigrams = seed_bigrams();
    let bytes = build_v2(&entries, &bigrams).map_err(|error| error.to_string())?;
    let header = DictHeader::from_bytes(&bytes).map_err(|error| error.to_string())?;

    if let Some(parent) = output.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .map_err(|error| format!("创建输出目录失败: {error}"))?;
        }
    }
    std::fs::write(&output, &bytes).map_err(|error| format!("写入词典文件失败: {error}"))?;

    println!("已构建 v2 词典: {}", output.display());
    println!(
        "词条 {}，拼音索引 {}，bigram {}，译文 {}，反查 {}，文件大小 {} 字节",
        header.entry_count,
        header.pinyin_index_count,
        header.bigram_count,
        header.word_translation_count,
        header.reverse_translation_count,
        bytes.len()
    );
    println!("内容 SHA-256: {}", hex(&header.content_hash));
    Ok(())
}

fn inspect_command(path: PathBuf) -> Result<(), String> {
    let file = DictionaryFile::open(&path).map_err(|error| error.to_string())?;
    print_header(&path, &file);
    Ok(())
}

fn verify_command(path: PathBuf) -> Result<(), String> {
    let file = DictionaryFile::open(&path).map_err(|error| error.to_string())?;
    print_header(&path, &file);

    let entries = seed_entries();
    let mut checked_entries = 0usize;
    let mut missing_entries = 0usize;
    for entry in &entries {
        let found = file
            .lookup(&entry.pinyin)
            .into_iter()
            .any(|candidate| candidate.word == entry.word);
        checked_entries += 1;
        if !found {
            missing_entries += 1;
        }
    }

    let mut checked_bigrams = 0usize;
    let mut missing_bigrams = 0usize;
    for (previous, word, _) in seed_bigrams() {
        checked_bigrams += 1;
        if file.frequency(previous, word) == 0 {
            missing_bigrams += 1;
        }
    }

    let mut checked_translations = 0usize;
    let mut missing_translations = 0usize;
    for entry in &entries {
        let Some(expected) = entry.translation.as_deref() else {
            continue;
        };
        checked_translations += 1;
        if file.zh_to_en(&entry.word).as_deref() != Some(expected) {
            missing_translations += 1;
        }
    }
    let mut checked_reverse = 0usize;
    let mut missing_reverse = 0usize;
    for (key, expected) in REVERSE_SAMPLES {
        checked_reverse += 1;
        if file.en_to_zh(key).as_deref() != Some(*expected) {
            missing_reverse += 1;
        }
    }

    println!();
    println!(
        "校验: 词条 {checked_entries} 命中 {}，bigram {missing_bigrams} 缺失 / {checked_bigrams}",
        checked_entries - missing_entries
    );
    println!(
        "翻译: 正查 {missing_translations} 缺失 / {checked_translations}，反查 {missing_reverse} 缺失 / {checked_reverse}"
    );
    if missing_entries > 0 || missing_bigrams > 0 || missing_translations > 0 || missing_reverse > 0
    {
        return Err(format!(
            "种子校验失败：缺词条 {missing_entries} 个，缺 bigram {missing_bigrams} 个，缺正查 {missing_translations} 个，缺反查 {missing_reverse} 个"
        ));
    }
    println!("校验通过：完整加载成功，种子词条/bigram/翻译全部命中");
    Ok(())
}

fn print_header(path: &Path, file: &DictionaryFile) {
    let header = file.header();
    println!("词典文件: {}", path.display());
    println!(
        "格式 v{}：词条 {}，拼音索引 {}，bigram {}，译文 {}，反查 {}，文件大小 {} 字节",
        dict_schema_version(),
        header.entry_count,
        header.pinyin_index_count,
        header.bigram_count,
        header.word_translation_count,
        header.reverse_translation_count,
        file.file_size()
    );
    println!("内容 SHA-256: {}", hex(&header.content_hash));
    println!("管线状态: {}", pipeline_status());
}

fn hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push_str(&format!("{byte:02x}"));
    }
    output
}
