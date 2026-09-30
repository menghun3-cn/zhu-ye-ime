//! 竹叶输入法开发调试 CLI。
//!
//! 提供候选演示、环境自检与简单性能基准，帮助无 GUI 环境联调核心算法。

use std::time::Instant;

use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use zhu_ye_core::candidate::{corrected_candidates, initial_candidates, sentence_candidates};
use zhu_ye_core::dict_loader::DictionaryFile;
use zhu_ye_core::{
    core_version, generate_candidates, BigramModel, Candidate, CandidateSorter, Dictionary,
    DictionaryEntry, InMemoryBigramModel, InMemoryDictionary, InMemoryTranslator, OfflineAiService,
    RankingConfig, RankingContext, RankingModel, StaticRankingModel, SyllableTable, Translator,
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
        Some("rank") => rank_command(args),
        Some("eval") => eval_command(args),
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
    println!("  dict update [check|apply|status]  触发词典更新器（M6-U，S-5）");
    println!("  rank <文件> <拼音> [前词]  加载 v2 词典并按上下文输出排序候选");
    println!("  eval <文件> <词样本.tsv> [整句样本.tsv] [--out-miss 文件]  命中率评测（T-057）");
}

/// `dict update` 子命令（M6-U，S-5）：触发独立更新器 exe。
///
/// 更新器是唯一联网组件；本命令只做进程转发，TSF DLL 侧不参与网络。
/// 找不到更新器时给出明确提示而不是静默失败。
fn dict_update_command(args: &[String]) {
    let sub = args.get(3).map(String::as_str).unwrap_or("status");
    let exe = updater_path();
    let Some(exe) = exe else {
        eprintln!(
            "未找到更新器 zhu-ye-updater.exe；请先构建（cargo build --release -p zhu-ye-updater）"
        );
        eprintln!("或把更新器放到 PATH / 与 zhu-ye-cli 同目录。");
        std::process::exit(1);
    };
    println!("调用更新器: {} {sub}", exe.display());
    let status = std::process::Command::new(&exe).arg(sub).status();
    match status {
        Ok(status) if status.success() => {}
        Ok(status) => {
            eprintln!("更新器退出码: {status}");
            std::process::exit(status.code().unwrap_or(1));
        }
        Err(error) => {
            eprintln!("启动更新器失败: {error}");
            std::process::exit(1);
        }
    }
}

/// 定位更新器：与当前可执行文件同目录优先，其次 PATH。
fn updater_path() -> Option<std::path::PathBuf> {
    let name = if cfg!(windows) {
        "zhu-ye-updater.exe"
    } else {
        "zhu-ye-updater"
    };
    if let Ok(current) = std::env::current_exe() {
        if let Some(dir) = current.parent() {
            let candidate = dir.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    // 开发布局：workspace target/<profile>/ 下两个二进制同目录，已由上面覆盖；
    // 兜底直接交给 PATH 解析。
    Some(std::path::PathBuf::from(name))
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
    // M6-R：多包场景（基础包 + 全部领域包）对比单词典查找延迟。
    let multi_pack_us = measure_multi_pack_lookup(20_000, true);
    // M7：三输入优化路径（简拼/纠错/整句）延迟（验收标准 8.2）。
    let (initial_us, corrected_us, sentence_us) = measure_m7_paths(2_000, true);
    println!(
        "指标: segment_us={segment_us:.3} lookup_us={lookup_us:.3} bigram_us={bigram_us:.3} zh_en_us={zh_en_us:.3} en_zh_us={en_zh_us:.3} multi_pack_us={multi_pack_us:.3} m7_initial_us={initial_us:.3} m7_corrected_us={corrected_us:.3} m7_sentence_us={sentence_us:.3}"
    );
}

/// M7 三路径延迟基准（FR-023 至 FR-025，验收标准 8.2：验收 ≤30ms、目标 ≤15ms）。
///
/// 使用 `ZYDT_DICT`（`dict_file_path`）指向的真实词典文件；文件不可用时
/// 返回全零（bench 仍可运行，验收数据以 release + 真实词典为准）。
fn measure_m7_paths(runs: usize, print: bool) -> (f64, f64, f64) {
    let path = dict_file_path();
    let Ok(file) = DictionaryFile::open(&path) else {
        if print {
            eprintln!("M7 路径基准跳过：无法打开词典 {}", path.display());
        }
        return (0.0, 0.0, 0.0);
    };
    let dictionary: Arc<dyn Dictionary> = Arc::new(file.clone());
    let table = SyllableTable::standard();
    let bigram = &file;

    let start = Instant::now();
    let mut initial_hits = 0usize;
    for _ in 0..runs {
        initial_hits =
            initial_hits.saturating_add(initial_candidates(dictionary.as_ref(), "nh").len());
    }
    let initial_us = us_per_op(start.elapsed(), runs);

    let start = Instant::now();
    let mut corrected_hits = 0usize;
    for _ in 0..runs {
        corrected_hits = corrected_hits
            .saturating_add(corrected_candidates(&table, dictionary.as_ref(), "niha").len());
    }
    let corrected_us = us_per_op(start.elapsed(), runs);

    let start = Instant::now();
    let mut sentence_hits = 0usize;
    for _ in 0..runs {
        sentence_hits = sentence_hits.saturating_add(
            sentence_candidates(
                &table,
                dictionary.as_ref(),
                bigram,
                "woxiangmingtianqubeijing",
            )
            .len(),
        );
    }
    let sentence_us = us_per_op(start.elapsed(), runs);

    if print {
        println!("M7 简拼基准: {runs} 次, 平均 {initial_us:.3}us/次, 累积命中 {initial_hits}");
        println!("M7 纠错基准: {runs} 次, 平均 {corrected_us:.3}us/次, 累积命中 {corrected_hits}");
        println!("M7 整句基准: {runs} 次, 平均 {sentence_us:.3}us/次, 累积命中 {sentence_hits}");
    }
    (initial_us, corrected_us, sentence_us)
}

/// 多包查找基准（M6-R，验收标准 7.2「候选刷新延迟（全包启用）」）。
///
/// 通过 `ZYDT_PACKS` 指定以逗号分隔的包路径；未设置时回退单词典路径，
/// 使本项在只有 seed 包的环境下仍可运行。
fn measure_multi_pack_lookup(runs: usize, print: bool) -> f64 {
    let paths: Vec<PathBuf> = match std::env::var_os("ZYDT_PACKS") {
        Some(value) => std::env::split_paths(&value).collect(),
        None => vec![dict_file_path()],
    };
    let (composite, skipped) = zhu_ye_core::CompositeDictionary::from_paths(&paths);
    if print && !skipped.is_empty() {
        println!("多包装载跳过 {} 个包: {skipped:?}", skipped.len());
    }
    let dictionary: Arc<dyn Dictionary> = Arc::new(composite);
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
            "多包查找基准: {} 个包, {ops} 次, 总 {elapsed:?}, 平均 {:?}/次, 累积词条 {hits}",
            paths.len(),
            elapsed / ops as u32
        );
    }
    us_per_op(elapsed, ops)
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
///
/// `dict update [check|apply|status]` 转发到独立更新器（M6-U，S-5）。
fn dict_command(args: Vec<String>) {
    if args.get(2).map(String::as_str) == Some("update") {
        dict_update_command(&args);
        return;
    }
    let Some(path) = args.get(2) else {
        println!("用法: zhu-ye-cli dict <词典文件> [拼音]");
        println!("      zhu-ye-cli dict <词典文件> -r 英文");
        println!("      zhu-ye-cli dict update [check|apply|status]");
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

/// `rank` 子命令：加载 v2 词典，生成候选并按上下文 bigram 排序。
fn rank_command(args: Vec<String>) {
    let Some(path) = args.get(2) else {
        println!("用法: zhu-ye-cli rank <词典文件> <拼音> [前词]");
        return;
    };
    let Some(pinyin) = args.get(3).map(String::as_str) else {
        println!("用法: zhu-ye-cli rank <词典文件> <拼音> [前词]");
        return;
    };
    let previous = args.get(4).map(String::as_str);
    match DictionaryFile::open(Path::new(path)) {
        Ok(file) => {
            let dictionary: Arc<dyn Dictionary> = Arc::new(file.clone());
            let bigram: Arc<dyn BigramModel> = Arc::new(file);
            let model = StaticRankingModel::new(RankingConfig::default(), bigram);
            let candidates =
                generate_candidates(&SyllableTable::standard(), dictionary.as_ref(), pinyin);
            let user = UserDictionary::new();
            let ranked = model.rank(candidates, &RankingContext::new(previous, &user));
            println!("候选排序: {pinyin} (前词: {})", previous.unwrap_or("无"));
            println!("候选数量: {}", ranked.len());
            for (index, candidate) in ranked.iter().take(20).enumerate() {
                let pinyin = candidate.pinyin.as_deref().unwrap_or("-");
                let translation = candidate.translation.as_deref().unwrap_or("-");
                println!(
                    "{:>2}. {}  [{}]  译文: {}  得分: {}",
                    index + 1,
                    candidate.text,
                    pinyin,
                    translation,
                    candidate.score
                );
            }
        }
        Err(error) => println!("加载失败: {error}"),
    }
}

/// `eval` 子命令（T-057）：命中率评测。
///
/// 词样本（`词<TAB>拼音<TAB>词频`）逐条走主候选路径（`generate_candidates` +
/// `StaticRankingModel`，无前词冷启动），判定 Top1（首候选==目标词）与 Top3
/// （前 3 位含目标词），并按词频分档报告 Top1 以暴露低频段退化；
/// 整句样本（`期望句<TAB>拼音串`）走 `sentence_candidates` 整句路径，
/// 判定首候选==期望句。Top1 未命中的词样本与整句未命中写入 MISS 文件。
/// 全程确定性输入 -> 确定性输出（用户词典置空）。
fn eval_command(args: Vec<String>) {
    let Some(path) = args.get(2).map(String::as_str) else {
        println!("用法: zhu-ye-cli eval <词典文件> <词样本.tsv> [整句样本.tsv] [--out-miss 文件]");
        return;
    };
    let Some(word_set) = args.get(3).map(String::as_str) else {
        println!("用法: zhu-ye-cli eval <词典文件> <词样本.tsv> [整句样本.tsv] [--out-miss 文件]");
        return;
    };
    let rest = &args[4..];
    let mut sentence_set: Option<String> = None;
    let mut out_miss = "eval-miss.tsv".to_owned();
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--out-miss" => {
                i += 1;
                if let Some(value) = rest.get(i) {
                    out_miss = value.clone();
                }
            }
            value if !value.starts_with("--") && sentence_set.is_none() => {
                sentence_set = Some(value.to_owned());
            }
            other => {
                println!("未知参数: {other}");
                return;
            }
        }
        i += 1;
    }
    let text = match std::fs::read_to_string(Path::new(word_set)) {
        Ok(text) => text,
        Err(error) => {
            println!("读取词样本失败: {error}");
            return;
        }
    };
    let file = match DictionaryFile::open(Path::new(path)) {
        Ok(file) => Arc::new(file),
        Err(error) => {
            println!("加载失败: {error}");
            return;
        }
    };
    let dictionary: Arc<dyn Dictionary> = file.clone();
    let bigram: Arc<dyn BigramModel> = file.clone();
    let model = StaticRankingModel::new(RankingConfig::default(), bigram.clone());
    let user = UserDictionary::new();
    let table = SyllableTable::standard();
    let mut miss_lines: Vec<String> = Vec::new();
    let mut total = 0usize;
    let mut top1 = 0usize;
    let mut top3 = 0usize;
    let mut buckets: [BucketStats; 4] = [BucketStats::new(); 4];

    // ---- 词样本：Top1 / Top3 + 词频分档 ----
    for line in text.lines() {
        let Some((word, pinyin, freq)) = parse_sample_line(line) else {
            continue;
        };
        let candidates = generate_candidates(&table, dictionary.as_ref(), pinyin);
        let ranked = model.rank(candidates, &RankingContext::new(None, &user));
        total += 1;
        let top1_hit = ranked.first().is_some_and(|c| c.text == word);
        buckets[bucket_index(freq)].record(top1_hit);
        if top1_hit {
            top1 += 1;
            top3 += 1;
        } else {
            if ranked.iter().take(3).any(|c| c.text == word) {
                top3 += 1;
            }
            miss_lines.push(format!(
                "词\t{word}\t{pinyin}\t{}",
                ranked.first().map(|c| c.text.as_str()).unwrap_or("-")
            ));
        }
    }
    println!(
        "词样本: N={total} Top1={:.1}% Top3={:.1}%",
        100.0 * top1 as f64 / total.max(1) as f64,
        100.0 * top3 as f64 / total.max(1) as f64
    );
    for (index, label) in BUCKET_LABELS.iter().enumerate() {
        println!(
            "  档位 {label}: n={} Top1={:.1}%",
            buckets[index].total,
            100.0 * buckets[index].top1 as f64 / buckets[index].total.max(1) as f64
        );
    }

    // ---- 整句样本：首候选命中 ----
    if let Some(sentence_set) = sentence_set {
        let sentence_text = match std::fs::read_to_string(Path::new(&sentence_set)) {
            Ok(text) => text,
            Err(error) => {
                println!("读取整句样本失败: {error}");
                return;
            }
        };
        let mut sentence_total = 0usize;
        let mut sentence_hit = 0usize;
        for line in sentence_text.lines() {
            let Some((expected, pinyin)) = parse_sentence_line(line) else {
                continue;
            };
            let candidates =
                sentence_candidates(&table, dictionary.as_ref(), bigram.as_ref(), pinyin);
            sentence_total += 1;
            if candidates.first().is_some_and(|c| c.text == expected) {
                sentence_hit += 1;
            } else {
                miss_lines.push(format!(
                    "句\t{expected}\t{pinyin}\t{}",
                    candidates.first().map(|c| c.text.as_str()).unwrap_or("-")
                ));
            }
        }
        println!(
            "整句: N={sentence_total} 首候选命中={:.1}%",
            100.0 * sentence_hit as f64 / sentence_total.max(1) as f64
        );
    }

    match std::fs::write(Path::new(&out_miss), miss_lines.join("\n") + "\n") {
        Ok(()) => println!("MISS 清单已写入 {out_miss}（{} 条）", miss_lines.len()),
        Err(error) => println!("MISS 清单写入失败（{out_miss}）: {error}"),
    }
}

/// 词频分档阈值与标签（≥1M / ≥100k / ≥10k / 其余）。
const BUCKET_LABELS: [&str; 4] = ["≥1,000,000", "≥100,000", "≥10,000", "<10,000"];

fn bucket_index(freq: u64) -> usize {
    if freq >= 1_000_000 {
        0
    } else if freq >= 100_000 {
        1
    } else if freq >= 10_000 {
        2
    } else {
        3
    }
}

#[derive(Clone, Copy)]
struct BucketStats {
    total: usize,
    top1: usize,
}

impl BucketStats {
    fn new() -> Self {
        Self { total: 0, top1: 0 }
    }

    fn record(&mut self, top1_hit: bool) {
        self.total += 1;
        if top1_hit {
            self.top1 += 1;
        }
    }
}

/// 解析词样本行 `词<TAB>拼音<TAB>词频`。
fn parse_sample_line(line: &str) -> Option<(&str, &str, u64)> {
    let mut parts = line.splitn(3, '\t');
    let word = parts.next()?.trim();
    let pinyin = parts.next()?.trim();
    let freq = parts.next()?.trim().parse::<u64>().ok()?;
    if word.is_empty() || pinyin.is_empty() {
        None
    } else {
        Some((word, pinyin, freq))
    }
}

/// 解析整句样本行 `期望句<TAB>拼音串`。
fn parse_sentence_line(line: &str) -> Option<(&str, &str)> {
    let mut parts = line.splitn(2, '\t');
    let expected = parts.next()?.trim();
    let pinyin = parts.next()?.trim();
    if expected.is_empty() || pinyin.is_empty() {
        None
    } else {
        Some((expected, pinyin))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_line_parses_valid_row() {
        let (word, pinyin, freq) = parse_sample_line("你好\tnihao\t900").unwrap();
        assert_eq!((word, pinyin, freq), ("你好", "nihao", 900));
    }

    #[test]
    fn sample_line_rejects_malformed_rows() {
        assert!(parse_sample_line("你好\tnihao").is_none()); // 缺词频
        assert!(parse_sample_line("\tnihao\t900").is_none()); // 空词
        assert!(parse_sample_line("你好\t\t900").is_none()); // 空拼音
        assert!(parse_sample_line("你好\tnihao\tabc").is_none()); // 坏词频
    }

    #[test]
    fn sentence_line_parses_valid_row() {
        let (expected, pinyin) = parse_sentence_line("明天见\tmingtianjian").unwrap();
        assert_eq!((expected, pinyin), ("明天见", "mingtianjian"));
        assert!(parse_sentence_line("明天见").is_none()); // 缺拼音
        assert!(parse_sentence_line("\tmingtianjian").is_none()); // 空期望句
    }

    #[test]
    fn bucket_index_boundaries() {
        assert_eq!(bucket_index(1_000_000), 0);
        assert_eq!(bucket_index(u64::MAX), 0);
        assert_eq!(bucket_index(999_999), 1);
        assert_eq!(bucket_index(100_000), 1);
        assert_eq!(bucket_index(99_999), 2);
        assert_eq!(bucket_index(10_000), 2);
        assert_eq!(bucket_index(9_999), 3);
        assert_eq!(bucket_index(0), 3);
    }
}
