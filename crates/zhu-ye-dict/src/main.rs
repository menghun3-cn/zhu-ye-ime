//! 词典构建 CLI：build / import / inspect / verify。
//!
//! `import` 把 CC-CEDICT 与开放词频语料清洗为真实词条并编译为 v2 词典，
//! 数据来源与许可证见 `docs/数据清单.md` 与 `docs/licenses.md`。

use std::path::{Path, PathBuf};

use std::collections::{HashMap, HashSet};

use zhu_ye_core::bigram::BigramModel;
use zhu_ye_core::dict::Dictionary;
use zhu_ye_core::dict_format::DictHeader;
use zhu_ye_core::dict_loader::DictionaryFile;
use zhu_ye_core::translate::Translator;
use zhu_ye_dict::{
    audit_coverage, build_base, build_manifest, build_pack, build_real_bigrams,
    build_real_dictionary, build_slang, build_v2, dict_schema_version, load_frequency_map,
    load_patch_table, load_standard_readings, parse_cedict_line, pipeline_status, polyphone_gaps,
    seed_bigrams, seed_entries, source_check, split_pinyin_syllables, today, verify_manifest,
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
        Some("audit-polyphone") => audit_polyphone_command(&args),
        Some("inspect") => inspect_command(required_path(&args, 2)?),
        Some("verify") => verify_command(required_path(&args, 2)?),
        Some("source-check") => source_check_command(),
        Some("build-pack") => build_pack_command(args.get(2).map(String::as_str)),
        Some("build-base") => build_base_command(&args),
        Some("audit-coverage") => audit_coverage_command(&args),
        Some("build-slang") => build_slang_command(),
        Some("build-manifest") => build_manifest_command(&args),
        Some("verify-manifest") => verify_manifest_command(required_path(&args, 2)?),
        Some("sign-manifest") => sign_manifest_command(required_path(&args, 2)?),
        Some("verify-signature") => verify_signature_command(required_path(&args, 2)?),
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
        "  import <CC-CEDICT> <词频> [输出路径] [上限] [--bigram 语料] [--polyphone 补丁表]  导入真实数据构建词典（默认 {DEFAULT_REAL_OUTPUT}）"
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
        "  audit-coverage [--sample N] [--base 文件]  常用词出候选率与首候选正确率抽检（S-1，N 默认 5000）（M6）"
    );
    println!(
        "  audit-polyphone <CC-CEDICT> <kTGHZ2013> [--freq 文件] [--min-freq N]  多音缺读审计（T-056）"
    );
    println!(
        "  build-slang           构建网络语包（种子表 + 把关抽查，输出 slang.zyct 与 slang.gate.json）（M6）"
    );
    println!(
        "  build-manifest [目录] [--version V] [--min-engine V]  扫描 *.zyct 生成 manifest.json（M6）"
    );
    println!("  verify-manifest <manifest.json>  逐包复核内容哈希与大小（M6）");
    println!(
        "  sign-manifest <manifest.json>  用发布私钥签名（读 ZHU_YE_RELEASE_SECRET_KEY，M6-U）"
    );
    println!(
        "  verify-signature <manifest.json>  用内置/指定公钥验签（读 ZHU_YE_RELEASE_PUBLIC_KEY，M6-U）"
    );
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

/// `build-slang`：把关抽查通过后构建网络语包并打印把关摘要。
fn build_slang_command() -> Result<(), String> {
    let report = build_slang(Path::new("."))?;
    let audit = &report.audit;
    println!(
        "把关表 {}：负例 {}/{} 拦截（漏放 0），正例误杀 {}/{}（{:.1}%）",
        report.blocklist_version,
        audit.negative_blocked,
        audit.negative_total,
        audit.positive_killed.len(),
        audit.positive_total,
        audit.false_kill_rate * 100.0
    );
    for killed in &audit.positive_killed {
        println!(
            "  误杀：{}（{}：{}）",
            killed.text, killed.category, killed.pattern
        );
    }
    println!(
        "网络语包（slang.zyct）构建完成：种子 {} 行，纯中文词 {}，缩写 {}，把关拦截 {}，排除 {}，词条 {}，大小 {} 字节",
        report.seed_rows,
        report.word_entries,
        report.abbreviation_entries,
        report.gate_blocked.len(),
        report.excluded.len(),
        report.entry_count,
        report.file_size
    );
    for item in report.gate_blocked.iter().chain(&report.excluded) {
        println!("  排除：{} [{}]（{}）", item.word, item.key, item.reason);
    }
    println!("内容 SHA-256: {}", report.sha256);
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

/// `audit-coverage [--sample N] [--base 文件]`：常用词覆盖与首候选抽检（S-1）。
///
/// 门槛与验收标准 FR-018 一致：出候选率 ≥98%、首候选正确率 ≥90%；
/// 任一不达标时以退出码 1 结束，使该命令可作为调参迭代的门禁。
fn audit_coverage_command(args: &[String]) -> Result<(), String> {
    let mut sample = 5000usize;
    let mut base = PathBuf::from("data/artifacts/base.zyct");
    let mut index = 2;
    while index < args.len() {
        match args[index].as_str() {
            "--sample" => {
                index += 1;
                sample = args
                    .get(index)
                    .ok_or_else(|| "--sample 缺少数值".to_owned())?
                    .parse::<usize>()
                    .map_err(|error| format!("--sample 解析失败: {error}"))?;
            }
            "--base" => {
                index += 1;
                base = PathBuf::from(
                    args.get(index)
                        .ok_or_else(|| "--base 缺少路径".to_owned())?,
                );
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

    let report = audit_coverage(Path::new("."), &base, sample)?;
    println!("抽检样本: {} 词（骨架前 {} 条）", report.sampled, sample);
    println!(
        "出候选率: {}/{} = {:.2}%（门槛 ≥98%）",
        report.with_candidates, report.sampled, report.coverage_pct
    );
    println!(
        "首候选=目标词: {}/{} = {:.2}%（同音冲突下有天花板，参考值）",
        report.first_hit, report.sampled, report.first_hit_pct
    );
    println!(
        "  样本含 {} 个不同拼音 → 该口径理论上限 {:.2}%",
        report.distinct_pinyins,
        report.distinct_pinyins as f64 / report.sampled.max(1) as f64 * 100.0
    );
    println!(
        "首候选=组内最常用词: {}/{} = {:.2}%（门槛 ≥90%，排序质量口径）",
        report.group_winner_hit, report.sampled, report.group_winner_pct
    );
    if !report.misses.is_empty() {
        println!(
            "未出候选（前 {} 条）: {}",
            report.misses.len(),
            report.misses.join("、")
        );
    }
    if !report.first_misses.is_empty() {
        println!("首候选非目标（前 {} 条）:", report.first_misses.len());
        for (target, actual) in &report.first_misses {
            println!("  {target} -> {actual}");
        }
    }

    // 判定：覆盖 ≥98% 且 组内最常用词居首 ≥90%（排除同音冲突的天花板口径）。
    let coverage_ok = report.coverage_pct >= 98.0;
    let first_ok = report.group_winner_pct >= 90.0;
    if coverage_ok && first_ok {
        println!("S-1 抽检通过。");
        Ok(())
    } else {
        Err(format!(
            "S-1 抽检未达标：出候选率 {:.2}%（{}），组内最常用词居首率 {:.2}%（{}）",
            report.coverage_pct,
            if coverage_ok { "达标" } else { "不达标" },
            report.group_winner_pct,
            if first_ok { "达标" } else { "不达标" }
        ))
    }
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

/// `sign-manifest <文件>`（M6-U）：用发布私钥对 manifest 签名并就地写回。
///
/// 私钥从环境变量 `ZHU_YE_RELEASE_SECRET_KEY` 读取（32 字节十六进制），
/// **绝不写入仓库、日志或命令行参数**——参数会留在进程列表与 shell 历史里。
fn sign_manifest_command(path: PathBuf) -> Result<(), String> {
    let secret_hex = std::env::var("ZHU_YE_RELEASE_SECRET_KEY").map_err(|_| {
        "未设置 ZHU_YE_RELEASE_SECRET_KEY（32 字节十六进制私钥）；\
         私钥只应存在于发布环境，不得写入仓库"
            .to_owned()
    })?;
    let secret_bytes = decode_hex(&secret_hex)?;
    let secret: [u8; 32] = secret_bytes.as_slice().try_into().map_err(|_| {
        format!(
            "私钥长度应为 32 字节（64 个十六进制字符），实际 {} 字节",
            secret_bytes.len()
        )
    })?;

    let text = std::fs::read_to_string(&path)
        .map_err(|error| format!("读取 manifest 失败（{}）: {error}", path.display()))?;
    let manifest = zhu_ye_core::parse_manifest(&text).map_err(|error| error.to_string())?;
    let signed =
        zhu_ye_core::sign_manifest(&manifest, &secret).map_err(|error| error.to_string())?;

    let json = serde_json::to_string_pretty(&signed)
        .map_err(|error| format!("序列化签名 manifest 失败: {error}"))?;
    std::fs::write(&path, json)
        .map_err(|error| format!("写入 manifest 失败（{}）: {error}", path.display()))?;

    let signature = signed
        .signature
        .as_ref()
        .ok_or_else(|| "签名后 manifest 缺少签名块".to_owned())?;
    println!("已签名: {}", path.display());
    println!("  算法: {}", signature.algorithm);
    println!("  公钥: {}", signature.public_key);
    println!("  请把该公钥配置到构建环境 ZHU_YE_RELEASE_PUBLIC_KEY，使客户端能验签");
    Ok(())
}

/// `verify-signature <文件>`（M6-U）：用指定或内置公钥验签。
fn verify_signature_command(path: PathBuf) -> Result<(), String> {
    let key_hex = std::env::var("ZHU_YE_RELEASE_PUBLIC_KEY")
        .map_err(|_| "未设置 ZHU_YE_RELEASE_PUBLIC_KEY（32 字节十六进制公钥）".to_owned())?;
    let key = zhu_ye_core::parse_public_key(&key_hex).map_err(|error| error.to_string())?;

    let text = std::fs::read_to_string(&path)
        .map_err(|error| format!("读取 manifest 失败（{}）: {error}", path.display()))?;
    let manifest = zhu_ye_core::parse_manifest(&text).map_err(|error| error.to_string())?;
    zhu_ye_core::verify_signature_with_key(&manifest, &key).map_err(|error| error.to_string())?;
    println!(
        "验签通过: {}（{} 个包）",
        path.display(),
        manifest.packs.len()
    );
    Ok(())
}

/// 解析十六进制字符串为字节。
fn decode_hex(text: &str) -> Result<Vec<u8>, String> {
    let text = text.trim();
    if !text.len().is_multiple_of(2) {
        return Err(format!("十六进制长度必须为偶数，实际 {}", text.len()));
    }
    let mut out = Vec::with_capacity(text.len() / 2);
    let bytes = text.as_bytes();
    for pair in bytes.chunks(2) {
        let high = (pair[0] as char)
            .to_digit(16)
            .ok_or_else(|| "包含非十六进制字符".to_owned())?;
        let low = (pair[1] as char)
            .to_digit(16)
            .ok_or_else(|| "包含非十六进制字符".to_owned())?;
        out.push(((high << 4) | low) as u8);
    }
    Ok(out)
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
    let mut polyphone_path: Option<PathBuf> = None;
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
            "--polyphone" => {
                index += 1;
                polyphone_path = Some(
                    args.get(index)
                        .map(PathBuf::from)
                        .ok_or_else(|| "--polyphone 缺少补丁表路径".to_owned())?,
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

    // T-056 多音缺读补丁：显式指定必须存在且合法；未指定时默认读取仓库补丁表，
    // 文件不存在则按无补丁处理（保持旧命令行为兼容）。
    let polyphone_patches = match &polyphone_path {
        Some(path) => load_patch_table(path)?,
        None => {
            let default = Path::new("data/patches/polyphone.tsv");
            if default.exists() {
                load_patch_table(default)?
            } else {
                Vec::new()
            }
        }
    };

    let (entries, stats) = build_real_dictionary(
        &cedict_text,
        &frequency_text,
        max_entries,
        &polyphone_patches,
    );
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
    println!(
        "多音补丁: 读取 {} 条，应用 {}，跳过 {}",
        polyphone_patches.len(),
        stats.polyphone_applied,
        stats.polyphone_skipped
    );
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

/// `audit-polyphone <CC-CEDICT> <kTGHZ2013> [--freq 文件] [--min-freq N]`：
/// 规范读音 vs 词库读音的多音缺读审计（T-056）。
///
/// 输出「规范读音存在而词库缺失」的（字, 读音）清单，附该字词频与已有读音；
/// 供人工甄选后写入 `data/patches/polyphone.tsv`（构建期 `import --polyphone` 应用）。
fn audit_polyphone_command(args: &[String]) -> Result<(), String> {
    let cedict_path = required_path(args, 2)?;
    let letters_path = required_path(args, 3)?;
    let mut freq_path: Option<PathBuf> = None;
    let mut min_freq: Option<u64> = None;
    let mut index = 4;
    while index < args.len() {
        match args[index].as_str() {
            "--freq" => {
                index += 1;
                freq_path = Some(
                    args.get(index)
                        .map(PathBuf::from)
                        .ok_or_else(|| "--freq 缺少词频文件路径".to_owned())?,
                );
                index += 1;
            }
            "--min-freq" => {
                index += 1;
                min_freq = Some(
                    args.get(index)
                        .ok_or_else(|| "--min-freq 缺少数值".to_owned())?
                        .parse::<u64>()
                        .map_err(|error| format!("--min-freq 解析失败: {error}"))?,
                );
                index += 1;
            }
            flag if flag.starts_with("--") => {
                return Err(format!("未知选项：{flag}"));
            }
            _ => return Err(format!("多余参数：{}", args[index])),
        }
    }

    let cedict_text = read_text_file(&cedict_path, "CC-CEDICT")?;
    let letters_text = read_text_file(&letters_path, "kTGHZ2013 读音表")?;

    // 与构建同口径：单字词条 -> 读音集合（split_pinyin_syllables 拒绝非法拼音）。
    let mut entry_readings: HashMap<String, HashSet<String>> = HashMap::new();
    for line in cedict_text.lines() {
        let Some((simplified, marked, _)) = parse_cedict_line(line) else {
            continue;
        };
        if simplified.chars().count() != 1 {
            continue;
        }
        let Some(syllables) = split_pinyin_syllables(&marked) else {
            continue;
        };
        if syllables.len() != 1 {
            continue;
        }
        entry_readings
            .entry(simplified)
            .or_default()
            .insert(syllables[0].clone());
    }

    let standard = load_standard_readings(&letters_text);
    let mut gaps = polyphone_gaps(&entry_readings, &standard);

    // 词频过滤仅为人工排序参考，不参与读音判定。
    let frequencies = match freq_path {
        Some(path) => {
            let freq_text = read_text_file(&path, "词频文件")?;
            Some(load_frequency_map(&freq_text))
        }
        None => None,
    };
    if let Some(frequencies) = &frequencies {
        gaps.retain(|gap| {
            frequencies.get(&gap.character).copied().unwrap_or(0) >= min_freq.unwrap_or(0)
        });
    }

    println!(
        "多音缺读审计: CEDICT 单字 {} 个，规范读音表 {} 字，缺读 {} 条",
        entry_readings.len(),
        standard.len(),
        gaps.len()
    );
    for gap in &gaps {
        let frequency = frequencies
            .as_ref()
            .and_then(|map| map.get(&gap.character))
            .copied()
            .map(|value| value.to_string())
            .unwrap_or_default();
        println!(
            "{}\t{}\t已有={}\t词频={}",
            gap.character,
            gap.missing,
            gap.existing.join("/"),
            frequency
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
