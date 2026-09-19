//! 竹叶输入法开发调试 CLI。
//!
//! 提供候选演示、环境自检与简单性能基准，帮助无 GUI 环境联调核心算法。

use std::time::Instant;

use std::path::PathBuf;

use zhu_ye_core::{
    core_version, Candidate, CandidateSorter, Dictionary, DictionaryEntry, InMemoryDictionary,
    OfflineAiService, SyllableTable, Translator,
};
use zhu_ye_core::{UserDictStore, UserDictionary};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("demo") => demo(args.get(2).map(String::as_str).unwrap_or("nihao")),
        Some("self-check") => self_check(),
        Some("bench") => bench(),
        Some("user") => user_command(args),
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
}

fn demo(pinyin: &str) {
    let table = SyllableTable::standard();
    let segments = zhu_ye_core::segment_all(&table, pinyin);
    println!("输入: {pinyin}");
    println!("切分方案: {}", segments.len());
    for (index, segment) in segments.iter().enumerate().take(8) {
        println!("  {}: {}", index + 1, segment.join("-"));
    }

    let dictionary = demo_dictionary();
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
    let _ = service;
}

fn bench() {
    let table = SyllableTable::standard();
    let runs = 50_000;
    let start = Instant::now();
    let mut hits = 0usize;
    for _ in 0..runs {
        hits += zhu_ye_core::segment_all(&table, "nihao").len();
    }
    let elapsed = start.elapsed();
    println!(
        "切分基准: {runs} 次, 总 {elapsed:?}, 平均 {:?}/次",
        elapsed / runs
    );
    println!("累积命中切分: {hits}");
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
