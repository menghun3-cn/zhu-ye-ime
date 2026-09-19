//! 竹叶输入法开发调试 CLI。
//!
//! 提供候选演示、环境自检与简单性能基准，帮助无 GUI 环境联调核心算法。

use std::time::Instant;

use zhu_ye_core::{
    core_version, Candidate, CandidateSorter, Dictionary, DictionaryEntry, InMemoryDictionary,
    OfflineAiService, SyllableTable, Translator,
};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("demo") => demo(args.get(2).map(String::as_str).unwrap_or("nihao")),
        Some("self-check") => self_check(),
        Some("bench") => bench(),
        _ => print_usage(),
    }
}

fn print_usage() {
    println!("zhu-ye-cli 命令：");
    println!("  demo [pinyin]      演示拼音切分与候选");
    println!("  self-check         环境与模块自检");
    println!("  bench              基础性能基准");
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
