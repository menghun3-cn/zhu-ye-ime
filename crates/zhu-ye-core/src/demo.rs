//! 项目自建的演示种子数据。
//!
//! 该词表由项目自行编写，不嵌入任何第三方词表内容；M1 演示词表
//! 与 T-006 数据管线共用同一份种子，避免两处词条漂移。正式数据源
//! 接入前必须按 `docs/licenses.md` 登记。

use crate::dict::DictionaryEntry;

/// 返回演示种子词条：中文词、无调全拼、静态词频与中英译文。
#[must_use]
pub fn seed_entries() -> Vec<DictionaryEntry> {
    const WORDS: &[(&str, &str, u64, Option<&str>)] = &[
        ("你好", "nihao", 100, Some("hello")),
        ("尼好", "nihao", 1, None),
        ("世界", "shijie", 90, Some("world")),
        ("中国", "zhongguo", 95, Some("China")),
        ("先", "xian", 70, Some("first")),
        ("西安", "xian", 60, Some("Xi'an")),
        ("西", "xi", 55, Some("west")),
        ("安", "an", 45, Some("safe")),
        ("我", "wo", 80, Some("I")),
        ("你", "ni", 65, Some("you")),
        ("好", "hao", 58, Some("good")),
        ("的", "de", 100, Some("of")),
        ("得", "de", 55, Some("get")),
        ("地", "de", 40, Some("land")),
        ("爱", "ai", 70, Some("love")),
        ("输入", "shuru", 55, Some("input")),
        ("打字", "dazi", 50, Some("type")),
        ("谢谢", "xiexie", 65, Some("thank you")),
        ("再见", "zaijian", 50, Some("goodbye")),
        ("早上好", "zaoshanghao", 42, Some("good morning")),
    ];

    WORDS
        .iter()
        .map(|&(word, pinyin, frequency, translation)| {
            let mut entry = DictionaryEntry::new(word, pinyin, frequency);
            if let Some(translation) = translation {
                entry = entry.with_translation(translation);
            }
            entry
        })
        .collect()
}

/// 返回演示 bigram 共现对；前词/后词均来自种子词条，频率为自定相对值。
#[must_use]
pub fn seed_bigrams() -> Vec<(&'static str, &'static str, u64)> {
    vec![
        ("你好", "世界", 120),
        ("世界", "你好", 90),
        ("中国", "世界", 80),
        ("世界", "中国", 60),
        ("谢谢", "你", 55),
        ("早上好", "世界", 40),
        ("爱", "世界", 35),
        ("输入", "世界", 30),
        ("打字", "世界", 25),
        ("再见", "世界", 20),
    ]
}
