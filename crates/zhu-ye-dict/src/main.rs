//! 词典构建 CLI：build / import / inspect / verify。
//!
//! `import` 把 CC-CEDICT 与开放词频语料清洗为真实词条并编译为 v2 词典，
//! 数据来源与许可证见 `docs/数据清单.md` 与 `docs/licenses.md`。

use std::path::{Path, PathBuf};

use std::collections::HashSet;

use zhu_ye_core::bigram::BigramModel;
use zhu_ye_core::dict::Dictionary;
use zhu_ye_core::dict_format::DictHeader;
use zhu_ye_core::dict_loader::DictionaryFile;
use zhu_ye_core::translate::Translator;
use zhu_ye_dict::{
    build_base, build_manifest, build_pack, build_real_bigrams, build_real_dictionary, build_v2,
    dict_schema_version, pipeline_status, seed_bigrams, seed_entries, source_check, today,
    verify_manifest,
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
        Some("source-check") => source_check_command(),
        Some("build-pack") => build_pack_command(args.get(2).map(String::as_str)),
        Some("build-base") => build_base_command(&args),
        Some("build-manifest") => build_manifest_command(&args),
        Some("verify-manifest") => verify_manifest_command(required_path(&args, 2)?),
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
        "  import <CC-CEDICT> <词频> [输出路径] [上限] [--bigram 语料]  导入真实数据构建词典（默认 {DEFAULT_REAL_OUTPUT}）"
    );
    println!("  inspect <文件>        打印词典头部元数据与内容哈希");
    println!("  verify <文件>         完整加载校验并核对种子词条/bigram/翻译");
    println!("  source-check          核对 data/pins 全部源的缓存哈希（M6）");
    println!(
        "  build-pack <it|med>  构建领域词包（THUOCL + 词级/单字级注音，输出 data/artifacts/<id>.zyct）（M6）"
    );
    println!(
        "  build-base [--min-score N]  构建基础包（xdhyc 骨架 + wordfreq 词频 + jieba 扩充，N 默认 2000）（M6）"
    );
    println!(
        "  build-manifest [目录] [--version V] [--min-engine V]  扫描 *.zyct 生成 manifest.json（M6）"
    );
    println!("  verify-manifest <manifest.json>  逐包复核内容哈希与大小（M6）");
}

/// `source-check`：source_check 失败返回 Err（含逐源明细），帮助文本仍可读。
fn source_check_command() -> Result<(), String> {
    source_check(Path::new(".")).map(|stats| {
        println!(
            "核对完成：{} 个源，锁定一致 {}，未锁定 {}",
            stats.checked, stats.locked_ok, stats.unlocked
        );
    })
}

/// `build-pack <it|med>`：构建领域词包并打印内容哈希。
fn build_pack_command(pack_id: Option<&str>) -> Result<(), String> {
    let pack_id = pack_id.ok_or_else(|| "缺少包 id（支持：it / med）".to_owned())?;
    let stats = build_pack(pack_id, Path::new("."))?;
    println!("内容 SHA-256: {}", stats.sha256);
    Ok(())
}

/// `build-base [--min-score N]`：构建基础包（骨架 + 词频 + 扩充）。
fn build_base_command(args: &[String]) -> Result<(), String> {
    let mut min_score = 2000u32;
    let mut index = 2;
    while index < args.len() {
        match args[index].as_str() {
            "--min-score" => {
                index += 1;
                min_score = args
                    .get(index)
                    .ok_or_else(|| "--min-score 缺少数值".to_owned())?
                    .parse::<u32>()
                    .map_err(|error| format!("--min-score 解析失败: {error}"))?;
            }
            flag if flag.starts_with("--") => {
                return Err(format!("未知选项：{flag}"));
            }
            _ => {
                return Err(format!("多余参数：{}", args[index]));
            }
        }
        index += 1;
    }
    build_base(Path::new("."), min_score).map(|_| ())
}

/// `build-manifest [目录] [--version V] [--min-engine V]`：
/// 扫描目录内 `*.zyct` 生成 `manifest.json`（未签名，签名在 M6-U）。
fn build_manifest_command(args: &[String]) -> Result<(), String> {
    let mut dir = PathBuf::from("data/artifacts");
    let mut version = None;
    let mut min_engine_version = None;
    let mut index = 2;
    while index < args.len() {
        match args[index].as_str() {
            "--version" => {
                index += 1;
                version = Some(
                    args.get(index)
                        .ok_or_else(|| "--version 缺少版本号".to_owned())?,
                );
            }
            "--min-engine" => {
                index += 1;
                min_engine_version = Some(
                    args.get(index)
                        .ok_or_else(|| "--min-engine 缺少版本号".to_owned())?,
                );
            }
            flag if flag.starts_with("--") => {
                return Err(format!("未知选项：{flag}"));
            }
            _ => {
                dir = PathBuf::from(&args[index]);
            }
        }
        index += 1;
    }
    let version = version.cloned().unwrap_or_else(today);
    let min_engine = min_engine_version
        .cloned()
        .unwrap_or_else(|| "0.1.0".to_owned());
    let manifest = build_manifest(&dir, &version, &min_engine)?;
    let output = dir.join("manifest.json");
    let json = serde_json::to_string_pretty(&manifest)
        .map_err(|error| format!("序列化 manifest 失败: {error}"))?;
    std::fs::write(&output, json)
        .map_err(|error| format!("写入 manifest 失败（{}）: {error}", output.display()))?;
    println!("已写入: {}", output.display());
    Ok(())
}

/// `verify-manifest <文件>`：逐包复核内容哈希与大小。
fn verify_manifest_command(path: PathBuf) -> Result<(), String> {
    verify_manifest(&path).map(|_| ())
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
    let mut output = PathBuf::from(DEFAULT_REAL_OUTPUT);
    let mut max_entries = None;
    let mut bigram_path = None;
    let mut output_arg_seen = false;
    let mut index = 4;
    while index < args.len() {
        match args[index].as_str() {
            "--bigram" => {
                index += 1;
                bigram_path = Some(
                    args.get(index)
                        .map(PathBuf::from)
                        .ok_or_else(|| "--bigram 缺少语料路径".to_owned())?,
                );
                index += 1;
            }
            _ if !output_arg_seen => {
                output = PathBuf::from(&args[index]);
                output_arg_seen = true;
                index += 1;
            }
            _ if max_entries.is_none() => {
                max_entries = Some(
                    args[index]
                        .parse::<usize>()
                        .map_err(|_| format!("词条上限必须是整数: {}", args[index]))?,
                );
                index += 1;
            }
            _ => return Err(format!("未知参数: {}", args[index])),
        }
    }

    let cedict_text = read_text_file(&cedict_path, "CC-CEDICT")?;
    let frequency_text = read_text_file(&frequency_path, "词频文件")?;

    let (entries, stats) = build_real_dictionary(&cedict_text, &frequency_text, max_entries);
    let vocabulary: HashSet<&str> = entries.iter().map(|entry| entry.word.as_str()).collect();
    let mut bigram_owned = Vec::new();
    let mut bigram_stats = None;
    if let Some(bigram_path) = &bigram_path {
        let corpus_text = read_text_file(bigram_path, "bigram 语料")?;
        let (bigrams, corpus_stats) = build_real_bigrams(&corpus_text, &vocabulary);
        bigram_owned = bigrams;
        bigram_stats = Some(corpus_stats);
    }
    let bigram_refs: Vec<(&str, &str, u64)> = bigram_owned
        .iter()
        .map(|(previous, word, frequency)| (previous.as_str(), word.as_str(), *frequency))
        .collect();
    let bytes = build_v2(&entries, &bigram_refs).map_err(|error| error.to_string())?;
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
    if let Some(bigram_stats) = &bigram_stats {
        println!(
            "bigram 统计: 语料行 {corpus_lines}，分词 {tokens_total}，命中词表 {tokens_matched}，候选词对 {pairs_formed}，唯一词对 {unique_pairs}",
            corpus_lines = bigram_stats.corpus_lines,
            tokens_total = bigram_stats.tokens_total,
            tokens_matched = bigram_stats.tokens_matched,
            pairs_formed = bigram_stats.pairs_formed,
            unique_pairs = bigram_stats.unique_pairs
        );
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

fn read_text_file(path: &Path, label: &str) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|error| format!("读取{label}失败: {error}"))
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
