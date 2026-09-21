//! 词典构建 CLI：build / import / inspect / verify。
//!
//! `import` 把 CC-CEDICT 与开放词频语料清洗为真实词条并编译为 v2 词典，
//! 数据来源与许可证见 `docs/数据清单.md` 与 `docs/licenses.md`。

use std::path::{Path, PathBuf};

use zhu_ye_core::bigram::BigramModel;
use zhu_ye_core::dict::Dictionary;
use zhu_ye_core::dict_format::DictHeader;
use zhu_ye_core::dict_loader::DictionaryFile;
use zhu_ye_core::translate::Translator;
use zhu_ye_dict::{
    build_real_dictionary, build_v2, dict_schema_version, pipeline_status, seed_bigrams,
    seed_entries,
};

/// 默认构建产物路径；`data/artifacts/` 已由 `.gitignore` 排除。
const DEFAULT_OUTPUT: &str = "data/artifacts/seed.zyct";

/// 真实数据导入的默认产物路径。
const DEFAULT_REAL_OUTPUT: &str = "data/artifacts/real.zyct";

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
        Some("import") => import_command(&args),
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
    println!(
        "  import <CC-CEDICT> <词频> [输出路径] [上限]  导入真实数据构建词典（默认 {DEFAULT_REAL_OUTPUT}）"
    );
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

    write_dictionary_file(&output, &bytes)?;
    print_built_summary(&output, &bytes, &header);
    Ok(())
}

fn import_command(args: &[String]) -> Result<(), String> {
    let cedict_path = required_path(args, 2)?;
    let frequency_path = required_path(args, 3)?;
    let output = args
        .get(4)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_REAL_OUTPUT));
    let max_entries = args
        .get(5)
        .map(|value| {
            value
                .parse::<usize>()
                .map_err(|_| format!("词条上限必须是整数: {value}"))
        })
        .transpose()?;

    let cedict_text = std::fs::read_to_string(&cedict_path)
        .map_err(|error| format!("读取 CC-CEDICT 失败: {error}"))?;
    let frequency_text = std::fs::read_to_string(&frequency_path)
        .map_err(|error| format!("读取词频文件失败: {error}"))?;

    let (entries, stats) = build_real_dictionary(&cedict_text, &frequency_text, max_entries);
    let bytes = build_v2(&entries, &[]).map_err(|error| error.to_string())?;
    let header = DictHeader::from_bytes(&bytes).map_err(|error| error.to_string())?;
    write_dictionary_file(&output, &bytes)?;

    println!("已导入真实数据: {}", output.display());
    print_built_summary(&output, &bytes, &header);
    println!();
    println!(
        "清洗统计: CC-CEDICT {cedict_lines} 行，进入构建 {accepted_entries} 个，最终 {entry_count} 个",
        cedict_lines = stats.cedict_lines,
        accepted_entries = stats.accepted_entries,
        entry_count = stats.entry_count
    );
    println!(
        "丢弃: 无拼音 {dropped_no_pinyin}，无效音节 {dropped_bad_syllable}，词形不符 {dropped_word_shape}，音节数与字数不符 {dropped_syllable_count_mismatch}",
        dropped_no_pinyin = stats.dropped_no_pinyin,
        dropped_bad_syllable = stats.dropped_bad_syllable,
        dropped_word_shape = stats.dropped_word_shape,
        dropped_syllable_count_mismatch = stats.dropped_syllable_count_mismatch
    );
    let hit_rate = if stats.accepted_entries == 0 {
        0.0
    } else {
        stats.frequency_hits as f64 / stats.accepted_entries as f64 * 100.0
    };
    println!(
        "词频: 命中 {} / {}（{hit_rate:.2}%），未命中按 1 计",
        stats.frequency_hits, stats.accepted_entries
    );
    println!(
        "音节校验: 参与 {} 个，无效 {} 个",
        stats.total_syllables, stats.invalid_syllables
    );
    if !stats.unknown_syllables.is_empty() {
        println!("未知音节样本: {}", stats.unknown_syllables.join("、"));
    }
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

fn print_built_summary(path: &Path, bytes: &[u8], header: &DictHeader) {
    println!("已构建 v2 词典: {}", path.display());
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
}

fn write_dictionary_file(output: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = output.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .map_err(|error| format!("创建输出目录失败: {error}"))?;
        }
    }
    std::fs::write(output, bytes).map_err(|error| format!("写入词典文件失败: {error}"))
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
