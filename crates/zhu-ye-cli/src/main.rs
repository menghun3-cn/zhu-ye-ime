//! 竹叶输入法开发调试 CLI。
//!
//! 提供候选演示、环境自检与简单性能基准，帮助无 GUI 环境联调核心算法。

use std::time::Instant;

use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use zhu_ye_core::dict_loader::DictionaryFile;
use zhu_ye_core::{
    core_version, BigramModel, Candidate, CandidateSorter, Dictionary, DictionaryEntry,
    InMemoryBigramModel, InMemoryDictionary, InMemoryTranslator, OfflineAiService, SyllableTable,
    Translator,
};
use zhu_ye_core::{UserDictStore, UserDictionary};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("demo") => demo(args.get(2).map(String::as_str).unwrap_or("nihao")),
        Some("self-check") => self_check(),
        Some("bench") => bench(),
        Some("user") => user_command(args),
        Some("dict") => dict_command(args),
        _ => print_usage(),
    }
}

fn print_usage() {
    println!("zhu-ye-cli 命令：");
    println!("  demo [pinyin]      演示拼音切分与候选");
    println!("  self-check         环境与模块自检");
    println!("  bench              基础性能基准");
    println!("  user [list]        查看用户词库");
    println!("  user delete W P    删除用户词（词与拼音）");
    println!("  user reset         清空用户词库");
    println!("  dict <文件> [拼]   加载 v2 词典并查询词条");
    println!("  dict <文件> -r 英文  加载 v2 词典并通过译文反查中文");
}

fn demo(pinyin: &str) {
    let table = SyllableTable::standard();
    let segments = zhu_ye_core::segment_all(&table, pinyin);
    println!("输入: {pinyin}");
    println!("切分方案: {}", segments.len());
    for (index, segment) in segments.iter().enumerate().take(8) {
        println!("  {}: {}", index + 1, segment.join("-"));
    }

    let dictionary: Arc<dyn Dictionary> = load_dictionary();
    println!("词典: {}", dictionary_source_label());
    let mut candidates = Vec::new();
    let mut seen_pinyin = std::collections::HashSet::new();
    for segment in &segments {
        let joined = segment.join("");
        if !seen_pinyin.insert(joined.clone()) {
            continue;
        }
        if let Some(entry) = dictionary.lookup(&joined).first() {
            let score = i64::try_from(entry.frequency).unwrap_or(i64::MAX);
            let mut candidate = Candidate::new(entry.word.clone(), score);
            if let Some(translation) = &entry.translation {
                candidate = candidate.with_translation(translation.clone());
            }
            candidates.push(candidate);
        }
    }
    let sorted = CandidateSorter::sort(candidates);
    println!("候选:");
    for candidate in sorted.iter().take(9) {
        match &candidate.translation {
            Some(translation) => println!("  {}  [{}]", candidate.text, translation),
            None => println!("  {}", candidate.text),
        }
    }
}

fn demo_dictionary() -> InMemoryDictionary {
    InMemoryDictionary::from_entries(vec![
        DictionaryEntry::new("你好", "nihao", 1000).with_translation("hello"),
        DictionaryEntry::new("泥好", "nihao", 10).with_translation("muddy good"),
        DictionaryEntry::new("你", "ni", 10_000).with_translation("you"),
        DictionaryEntry::new("好", "hao", 9000).with_translation("good"),
        DictionaryEntry::new("竹叶", "zhuye", 100).with_translation("bamboo leaf"),
    ])
}

fn self_check() {
    let table = SyllableTable::standard();
    let service = OfflineAiService;
    let translator = zhu_ye_core::InMemoryTranslator::new();
    println!("核心库版本: {}", core_version());
    println!(
        "音节表大小: {} (标准全拼表)",
        table.complete_syllables_with_prefix("").len()
    );
    println!("词典: {}", dictionary_source_label());
    println!("AI 服务: OfflineAiService (零网络)");
    println!(
        "翻译器: {}",
        if translator.zh_to_en("你好").is_none() {
            "内存空表"
        } else {
            "内存表"
        }
    );
    println!(
        "演示输入: nihao -> 切分 {} 种",
        zhu_ye_core::segment_all(&table, "nihao").len()
    );

    let ranking = zhu_ye_core::StaticRankingModel::default();
    let config = ranking.config();
    println!(
        "候选排序: StaticRankingModel (unigram={} bigram={} user={})",
        config.unigram_weight, config.bigram_weight, config.user_weight
    );
    let store = UserDictStore::new(user_words_path());
    match store.load() {
        Ok(user_dict) => println!(
            "用户词库: {} ({} 条)",
            store.path().display(),
            user_dict.len()
        ),
        Err(error) => println!("用户词库: {} 读取失败: {error}", store.path().display()),
    }
    let segment_us = measure_segment(2_000, false);
    let lookup_us = measure_lookup(1_000, false);
    let bigram_us = measure_bigram(2_000, false);
    let (zh_en_us, en_zh_us) = measure_translation(1_000, false);
    println!(
        "性能基线(快速): segment={segment_us:.2}us lookup={lookup_us:.2}us bigram={bigram_us:.2}us zh_en={zh_en_us:.2}us en_zh={en_zh_us:.2}us"
    );
    let _ = service;
}

fn bench() {
    if cfg!(debug_assertions) {
        println!("性能基准（debug 数值仅供趋势参考，验收以 release 为准）");
    } else {
        println!("性能基准（release）");
    }
    let segment_us = measure_segment(50_000, true);
    let lookup_us = measure_lookup(20_000, true);
    let bigram_us = measure_bigram(100_000, true);
    let (zh_en_us, en_zh_us) = measure_translation(50_000, true);
    println!(
        "指标: segment_us={segment_us:.3} lookup_us={lookup_us:.3} bigram_us={bigram_us:.3} zh_en_us={zh_en_us:.3} en_zh_us={en_zh_us:.3}"
    );
}

fn us_per_op(elapsed: std::time::Duration, ops: usize) -> f64 {
    elapsed.as_secs_f64() * 1_000_000.0 / ops as f64
}

fn measure_segment(runs: usize, print: bool) -> f64 {
    let table = SyllableTable::standard();
    let start = Instant::now();
    let mut hits = 0usize;
    for _ in 0..runs {
        hits += zhu_ye_core::segment_all(&table, "nihao").len();
    }
    let elapsed = start.elapsed();
    if print {
        println!(
            "切分基准: {runs} 次, 总 {elapsed:?}, 平均 {:?}/次, 累积切分 {hits}",
            elapsed / runs as u32
        );
    }
    us_per_op(elapsed, runs)
}

fn measure_lookup(runs: usize, print: bool) -> f64 {
    let dictionary: Arc<dyn Dictionary> = load_dictionary();
    let queries = ["nihao", "xian", "de", "shuru", "zaoshanghao"];
    let start = Instant::now();
    let mut hits = 0usize;
    for _ in 0..runs {
        for query in queries {
            hits = hits.saturating_add(dictionary.lookup(query).len());
        }
    }
    let ops = runs.saturating_mul(queries.len());
    let elapsed = start.elapsed();
    if print {
        println!(
            "候选查找基准: {ops} 次, 总 {elapsed:?}, 平均 {:?}/次, 累积词条 {hits}",
            elapsed / ops as u32
        );
    }
    us_per_op(elapsed, ops)
}

fn measure_bigram(runs: usize, print: bool) -> f64 {
    let bigram: Arc<dyn BigramModel> = match DictionaryFile::open(&dict_file_path()) {
        Ok(file) => Arc::new(file),
        Err(_) => Arc::new(InMemoryBigramModel::new()),
    };
    let start = Instant::now();
    let mut hits = 0u64;
    for _ in 0..runs {
        hits = hits.wrapping_add(bigram.frequency("你好", "世界"));
    }
    let elapsed = start.elapsed();
    if print {
        println!(
            "bigram 基准: {runs} 次, 总 {elapsed:?}, 平均 {:?}/次, 命中 {hits}",
            elapsed / runs as u32
        );
    }
    us_per_op(elapsed, runs)
}

fn measure_translation(runs: usize, print: bool) -> (f64, f64) {
    let translator: Arc<dyn Translator> = match DictionaryFile::open(&dict_file_path()) {
        Ok(file) => Arc::new(file),
        Err(_) => Arc::new(InMemoryTranslator::new()),
    };
    let start = Instant::now();
    let mut zh_hits = 0usize;
    for _ in 0..runs {
        zh_hits += usize::from(translator.zh_to_en("你好").is_some());
    }
    let zh_elapsed = start.elapsed();
    let start = Instant::now();
    let mut en_hits = 0usize;
    for _ in 0..runs {
        en_hits += usize::from(translator.en_to_zh("hello").is_some());
    }
    let en_elapsed = start.elapsed();
    if print {
        println!(
            "翻译正查基准: {runs} 次, 总 {zh_elapsed:?}, 平均 {:?}/次, 命中 {zh_hits}",
            zh_elapsed / runs as u32
        );
        println!(
            "翻译反查基准: {runs} 次, 总 {en_elapsed:?}, 平均 {:?}/次, 命中 {en_hits}",
            en_elapsed / runs as u32
        );
    }
    (us_per_op(zh_elapsed, runs), us_per_op(en_elapsed, runs))
}

/// 加载 v2 词典文件；文件缺失或损坏时回退内置演示词典。
fn load_dictionary() -> Arc<dyn Dictionary> {
    let path = dict_file_path();
    match DictionaryFile::open(&path) {
        Ok(file) => Arc::new(file),
        Err(_) => Arc::new(demo_dictionary()),
    }
}

/// 返回词典来源说明，供演示与自检输出。
fn dictionary_source_label() -> String {
    let path = dict_file_path();
    match DictionaryFile::open(&path) {
        Ok(file) => {
            let header = file.header();
            format!(
                "{} (mmap v2，{} 词条，{} bigram，{} 译文)",
                path.display(),
                header.entry_count,
                header.bigram_count,
                header.word_translation_count
            )
        }
        Err(_) => format!("内置演示词典（未找到 {}）", path.display()),
    }
}

/// 词典文件路径：`ZYDT_DICT` 环境变量优先，默认为工作目录下数据产物。
fn dict_file_path() -> PathBuf {
    std::env::var_os("ZYDT_DICT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("data/artifacts/seed.zyct"))
}

/// `dict` 子命令：加载 v2 词典，查询拼音词条或通过 `-r` 反查英文。
fn dict_command(args: Vec<String>) {
    let Some(path) = args.get(2) else {
        println!("用法: zhu-ye-cli dict <词典文件> [拼音]");
        println!("      zhu-ye-cli dict <词典文件> -r 英文");
        return;
    };
    match DictionaryFile::open(Path::new(path)) {
        Ok(file) => {
            let header = file.header();
            println!("词典文件: {path}");
            println!(
                "格式 v2：词条 {}，拼音索引 {}，bigram {}，译文 {}，反查 {}，文件大小 {} 字节",
                header.entry_count,
                header.pinyin_index_count,
                header.bigram_count,
                header.word_translation_count,
                header.reverse_translation_count,
                file.file_size()
            );
            if args.get(3).map(String::as_str) == Some("-r") {
                let Some(text) = args.get(4).map(String::as_str) else {
                    println!("用法: zhu-ye-cli dict <词典文件> -r 英文");
                    return;
                };
                match file.en_to_zh(text) {
                    Some(word) => println!("反查 {text}: {word}"),
                    None => println!("反查 {text}: 无结果"),
                }
                return;
            }
            let pinyin = args.get(3).map(String::as_str).unwrap_or("nihao");
            let found = file.lookup(pinyin);
            if found.is_empty() {
                println!("拼音 {pinyin} 无词条");
                return;
            }
            println!("查询 {pinyin}:");
            for entry in &found {
                match &entry.translation {
                    Some(translation) => println!(
                        "  {} [{}] 词频 {}",
                        entry.word, translation, entry.frequency
                    ),
                    None => println!("  {} 词频 {}", entry.word, entry.frequency),
                }
            }
        }
        Err(error) => println!("加载失败: {error}"),
    }
}
/// 用户词库路径：Windows 使用 `%APPDATA%`，其他环境回退到工作目录。
fn user_words_path() -> PathBuf {
    std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .map(|root| root.join("ai-zhu-ye-ime").join("user_words.json"))
        .unwrap_or_else(|| PathBuf::from("user_words.json"))
}

fn user_command(args: Vec<String>) {
    let store = UserDictStore::new(user_words_path());
    match args.get(2).map(String::as_str).unwrap_or("list") {
        "list" => {
            let user_dict = store.load().unwrap_or_default();
            println!(
                "用户词库: {} ({} 条)",
                store.path().display(),
                user_dict.len()
            );
            for entry in user_dict.words_sorted() {
                println!(
                    "  {} [{}] 选择 {} 次, 最近 {}",
                    entry.word, entry.pinyin, entry.frequency, entry.last_used
                );
            }
        }
        "delete" => {
            let Some(word) = args.get(3) else {
                println!("用法: zhu-ye-cli user delete <词> <拼音>");
                return;
            };
            let Some(pinyin) = args.get(4) else {
                println!("用法: zhu-ye-cli user delete <词> <拼音>");
                return;
            };
            let mut user_dict = store.load().unwrap_or_default();
            match store.delete_word(&mut user_dict, word, pinyin) {
                Ok(true) => println!("已删除 {word} [{pinyin}]"),
                Ok(false) => println!("未找到 {word} [{pinyin}]"),
                Err(error) => println!("删除失败: {error}"),
            }
        }
        "reset" => {
            let mut user_dict = UserDictionary::new();
            match store.reset(&mut user_dict) {
                Ok(()) => println!("用户词库已清空: {}", store.path().display()),
                Err(error) => println!("清空失败: {error}"),
            }
        }
        _ => println!("user 子命令: list / delete <词> <拼音> / reset"),
    }
}
