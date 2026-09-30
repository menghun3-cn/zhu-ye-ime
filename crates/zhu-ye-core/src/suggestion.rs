//! 上下文联想候选（T-058，场景 5）。
//!
//! 以已上屏的前词为基础，从 bigram 语料检索高频后继**整词**，并为高频整词
//! 再拼出其"最佳后继"形成**两词短语**（如 今天天气→怎么样/不错/很好，
//! 明天见→明天见你），组成空闲候选窗的联想列表。全部输入确定性，用户字典
//! 不参与（联想只反映语料统计，不随用户历史漂移）。

use crate::bigram::BigramModel;

/// 联想候选总上限（整词 + 短语）。
pub const SUGGESTION_CAP: usize = 8;
/// 整词联想上限。
pub const SUGGESTION_WORD_CAP: usize = 5;
/// 短语联想上限。
pub const SUGGESTION_PHRASE_CAP: usize = 3;

/// 返回前词的高频后继整词（最多 `limit` 条，按语料频率降序、同频字典序）。
#[must_use]
pub fn suggest_words(model: &dyn BigramModel, previous: &str, limit: usize) -> Vec<String> {
    model
        .successors(previous, limit)
        .into_iter()
        .map(|(word, _)| word)
        .collect()
}

/// 为前词的高频后继整词拼接出"前词 + 后继"两词短语（如 明天见→明天见你），
/// 至多 `limit` 条、按后继频率降序；后继与短语与前词完全同形时跳过（无信息量）。
#[must_use]
pub fn suggest_phrases(model: &dyn BigramModel, previous: &str, limit: usize) -> Vec<String> {
    let words = model.successors(previous, limit);
    let mut out: Vec<String> = Vec::new();
    for (word, _) in words {
        if word.is_empty() || word == previous {
            continue;
        }
        let phrase = format!("{previous}{word}");
        if !out.contains(&phrase) {
            out.push(phrase);
            if out.len() >= limit {
                break;
            }
        }
    }
    out
}

/// 完整联想列表（T-058 口径：Top5 整词 + 至多 3 条两词短语，总量 ≤ 8）。
/// 前词为空时返回空（联想只在"刚上屏过一个词"后触发）。
#[must_use]
pub fn suggestion_candidates(model: &dyn BigramModel, previous: &str) -> Vec<String> {
    if previous.is_empty() {
        return Vec::new();
    }
    let mut out = suggest_words(model, previous, SUGGESTION_WORD_CAP);
    for phrase in suggest_phrases(model, previous, SUGGESTION_PHRASE_CAP) {
        if !out.contains(&phrase) {
            out.push(phrase);
        }
    }
    out.truncate(SUGGESTION_CAP);
    out
}

#[cfg(test)]
mod tests {
    use super::{suggest_phrases, suggest_words, suggestion_candidates};
    use crate::bigram::InMemoryBigramModel;

    fn model() -> InMemoryBigramModel {
        let mut model = InMemoryBigramModel::new();
        model.insert("今天天气", "怎么样", 600);
        model.insert("今天天气", "不错", 500);
        model.insert("今天天气", "很好", 480);
        model.insert("今天天气", "真好", 300);
        model.insert("今天天气", "不错哦", 40);
        model.insert("明天见", "你", 900);
        model.insert("你", "吧", 300);
        model.insert("好", "吗", 700);
        model
    }

    #[test]
    fn 高频后继整词按频率降序() {
        let m = model();
        let words = suggest_words(&m, "今天天气", 5);
        assert_eq!(words, vec!["怎么样", "不错", "很好", "真好", "不错哦"]);
    }

    #[test]
    fn 前词未收录或为空返回空() {
        let m = model();
        assert!(suggestion_candidates(&m, "").is_empty());
        assert!(suggestion_candidates(&m, "未收录词").is_empty());
    }

    #[test]
    fn 整词加短语组成联想列表短语置后() {
        let m = model();
        let candidates = suggestion_candidates(&m, "明天见");
        // 整词最前：你；其后是"你"的最优再后继拼出的 明天见你。
        assert_eq!(candidates, vec!["你", "明天见你"]);
    }

    #[test]
    fn 短语拼接取前词加后继并防同形() {
        let m = model();
        // 今天天气 的高频后继拼出 前词+后继 短语，取前 3、按后继频率降序。
        let phrases = suggest_phrases(&m, "今天天气", 3);
        assert_eq!(
            phrases,
            vec!["今天天气怎么样", "今天天气不错", "今天天气很好"]
        );
        // 同形短语（前词==后继）跳过。
        let mut m2 = InMemoryBigramModel::new();
        m2.insert("好", "好", 900);
        assert!(suggest_phrases(&m2, "好", 3).is_empty());
    }
}
