//! 口语联想覆盖表（T-105，T-058 数据项，2026-10-05 用户点名清单）。
//!
//! T-058 联想整词来自 bigram 语料（OPUS GlobalVoices 新闻语体），"怎么样/不错"
//! 等口语后继不在高频区。本表以**人工维护的中性日常口语后继**覆盖特定高频前词
//! 的联想整词区（替换 T-058 数据项的口径）：
//!
//! - 命中：整词区完全采用本表列表（≤ SUGGESTION_WORD_CAP），短语区仍由 bigram
//!   语料补充（"明天早上/今天天气"式短语仍有用），总量 ≤ SUGGESTION_CAP；
//! - 未命中：联想整词区维持 bigram 统计原逻辑（语体、排序行为零变化）；
//! - T-057 基准不回退：联想仍只出现在上屏后的空闲窗，不参与拼音候选排序
//!   （机制层零改动，仅数据层替换）。
//!
//! 选词原则：中性、常用、无敏感；每条均为「前词 + 后继」的常用口语搭配中的
//! 后继**整词**（确定性表序；规模预算与格式合规由测试强制）。

/// 口语联想覆盖表：前词 → 口语后继整词（≤ SUGGESTION_WORD_CAP）。
/// 前词需与联想触发词完全一致（确定性整词匹配，不做分词）。
pub const SPOKEN_SUGGESTIONS: &[(&str, &[&str])] = &[
    // ---- 天气/日常问候 ----
    ("天气", &["不错", "怎么样", "很好", "挺好", "很冷"]),
    ("今天", &["天气", "怎么样", "下雨", "忙", "累"]),
    ("明天", &["再说", "见", "去", "走", "休息"]),
    ("昨天", &["怎么样", "挺好", "不错", "好", "累"]),
    // ---- 人称 ----
    ("你", &["好", "在吗", "知道", "觉得", "想"]),
    ("我", &["好", "想", "觉得", "不知道", "要"]),
    ("他", &["好", "说", "觉得", "来", "去"]),
    ("她", &["好", "说", "觉得", "来", "去"]),
    ("大家", &["好", "辛苦了", "在吗", "觉得", "加油"]),
    // ---- 日常评价 ----
    ("这个", &["怎么样", "不错", "可以", "好", "贵"]),
    ("那个", &["怎么样", "不错", "可以", "好", "贵"]),
    ("东西", &["好吃", "不错", "贵", "便宜", "怎么样"]),
    ("感觉", &["不错", "很好", "怎么样", "一般", "奇怪"]),
    ("味道", &["不错", "怎么样", "好", "还行", "咸"]),
    // ---- 场景/状态 ----
    ("工作", &["怎么样", "忙", "累", "顺利", "不错"]),
    ("心情", &["不错", "很好", "不好", "怎么样", "还行"]),
    ("身体", &["怎么样", "还好", "不舒服", "很棒", "好"]),
    ("最近", &["怎么样", "忙", "还好", "很累", "好吗"]),
    ("孩子", &["怎么样", "听话", "乖", "上学", "在家"]),
    ("老板", &["怎么说", "在吗", "好吗", "怎么样", "忙"]),
    // ---- 交易/价格 ----
    ("价格", &["多少", "怎么样", "贵", "便宜", "合理"]),
    ("价钱", &["多少", "怎么样", "贵", "便宜", "合理"]),
    ("费用", &["多少", "怎么算", "贵", "便宜", "合理"]),
    ("礼物", &["怎么样", "喜欢", "不错", "合适", "贵"]),
    // ---- 吃饭/生活 ----
    ("饭", &["好了", "好吃", "香", "够", "少"]),
    ("菜", &["好吃", "怎么样", "咸", "辣", "不错"]),
    ("电影", &["好看", "怎么样", "不错", "一般", "精彩"]),
    ("音乐", &["好听", "怎么样", "不错", "舒缓", "吵"]),
    // ---- 方向/行动 ----
    ("路", &["好走", "堵", "远", "近", "怎么走"]),
    ("地方", &["不错", "好玩", "怎么样", "远", "近"]),
];

/// 口语覆盖命中：返回前词的口语后继整词列表；未命中返回 `None`。
#[must_use]
pub fn spoken_overrides(previous: &str) -> Option<&'static [&'static str]> {
    SPOKEN_SUGGESTIONS
        .iter()
        .find(|(key, _)| *key == previous)
        .map(|(_, words)| *words)
}

#[cfg(test)]
mod tests {
    use super::{spoken_overrides, SPOKEN_SUGGESTIONS};
    use crate::suggestion::SUGGESTION_WORD_CAP;

    #[test]
    fn 命中常见前词() {
        let weather = spoken_overrides("天气").expect("天气应命中");
        assert!(weather.contains(&"不错"));
        assert!(weather.contains(&"怎么样"));
        let you = spoken_overrides("你").expect("你应命中");
        assert!(you.contains(&"在吗"));
    }

    #[test]
    fn 未命中前词返回空() {
        assert_eq!(spoken_overrides("明天见"), None);
        assert_eq!(spoken_overrides(""), None);
    }

    #[test]
    fn 全表规模与格式合规() {
        let mut seen = std::collections::HashSet::new();
        let mut total = 0usize;
        for (key, words) in SPOKEN_SUGGESTIONS {
            assert!(!key.is_empty(), "前词不得为空");
            assert!(seen.insert(*key), "前词重复: {key}");
            assert!(
                !words.is_empty() && words.len() <= SUGGESTION_WORD_CAP,
                "前词 {key} 候选须 1..={SUGGESTION_WORD_CAP} 条"
            );
            let mut per_key = std::collections::HashSet::new();
            for word in *words {
                assert!(
                    !word.is_empty() && word.chars().any(char::is_alphabetic),
                    "前词 {key} 候选不得为空"
                );
                assert!(per_key.insert(*word), "前词 {key} 候选重复: {word}");
            }
            total += words.len();
        }
        // 规模预算：覆盖不影响热路径（整词匹配 O(表长)，表长受控）。
        assert!(
            (30..=512).contains(&total),
            "口语覆盖表规模预算 30..=512，实际 {total}"
        );
        assert!(!SPOKEN_SUGGESTIONS.is_empty());
    }
}
