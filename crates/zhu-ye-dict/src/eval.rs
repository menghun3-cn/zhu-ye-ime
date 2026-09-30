//! 命中率评测样本生成（T-057）。
//!
//! 评测集生成器：以 CC-CEDICT 词级注音为读音来源、wordfreq 词频表为目标词分布，
//! 产出 `词<TAB>无调连续全拼<TAB>词频` 的评测词样本，供 `zhu-ye-cli eval` 判定
//! Top1 / Top3 命中率。清洗口径与 real dict 导入（`crate::import`）完全一致，
//! 保证样本本身在构建与评测两侧语义对齐。

use std::collections::HashSet;

use crate::import::{is_cjk_word, load_frequency_map, parse_cedict_line, split_pinyin_syllables};

/// 一条词样本：目标词、无调连续全拼、词频。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvalSample {
    /// 目标词（用户想打的词）。
    pub word: String,
    /// 无调连续全拼，与引擎拼音键一致（`ni3 hao3` → `nihao`）。
    pub pinyin: String,
    /// wordfreq 词频计数（v2 截断口径同导入管线）。
    pub frequency: u64,
}

/// 从 CEDICT 与 wordfreq 词频文本生成评测词样本。
///
/// 清洗：纯 CJK 词形；音节数 == 字数；逐音节无调归一化（`nǚ`→`nv`、删声调数字）；
/// 同一词在 CEDICT 有多行（多音多行，按常用度排序）时保留**首个**读音。两源交集后
/// 按词频降序（同频按词形字典序，保证确定性）截取前 `top` 条。CEDICT 无注音的词
/// （英文/数字词等）自然被排除。
pub fn generate_word_eval_set(
    cedict_text: &str,
    frequency_text: &str,
    top: usize,
) -> Vec<EvalSample> {
    let frequencies = load_frequency_map(frequency_text);
    let mut samples = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for line in cedict_text.lines() {
        let Some((word, marked, _)) = parse_cedict_line(line) else {
            continue;
        };
        if !is_cjk_word(&word) || !seen.insert(word.clone()) {
            continue;
        }
        let Some(syllables) = split_pinyin_syllables(&marked) else {
            continue;
        };
        if syllables.len() != word.chars().count() {
            continue;
        }
        let pinyin = syllables.concat();
        if let Some(&frequency) = frequencies.get(&word) {
            samples.push(EvalSample {
                word,
                pinyin,
                frequency,
            });
        }
    }
    samples.sort_by(|a, b| {
        b.frequency
            .cmp(&a.frequency)
            .then_with(|| a.word.cmp(&b.word))
    });
    samples.truncate(top);
    samples
}

/// 渲染为 TSV 行（`词<TAB>拼音<TAB>词频`），供 CLI 落盘与测试比对。
pub fn render_eval_set(samples: &[EvalSample]) -> String {
    let mut out = String::new();
    for sample in samples {
        out.push_str(&sample.word);
        out.push('\t');
        out.push_str(&sample.pinyin);
        out.push('\t');
        out.push_str(&sample.frequency.to_string());
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const CEDICT_SAMPLES: &str = "\
# comment line
你好 你好 [ni3 hao3] /hello/
世界 世界 [shi4 jie4] /world/
的 的 [de5] /of/
跑 跑 [pao3] /to run/
跑步 跑步 [pao3 bu4] /to jog/
長 长 [chang2] /long/
OK OK [ou1 kei1] /ok/
你好吗 你好吗 [ni3 hao3 ma5] /how are you/
的 的 [di4] /target/
";

    const FREQ_SAMPLES: &str = "\
的 1000
你好 900
世界 800
长 700
跑 600
跑步 500
不存在的词 400
";

    #[test]
    fn eval_set_cleans_and_intersects() {
        let samples = generate_word_eval_set(CEDICT_SAMPLES, FREQ_SAMPLES, 10);
        // 交集：的/你好/世界/长/跑/跑步；排除：OK（非 CJK）、你好吗（词频缺）。
        let words: Vec<&str> = samples.iter().map(|s| s.word.as_str()).collect();
        assert_eq!(words, vec!["的", "你好", "世界", "长", "跑", "跑步"]);
        // 多音词取 CEDICT 首读；声调数字归一化为无调连拼。
        let chang = samples.iter().find(|s| s.word == "长").unwrap();
        assert_eq!(chang.pinyin, "chang");
        assert_eq!(chang.frequency, 700);
        let nihao = samples.iter().find(|s| s.word == "你好").unwrap();
        assert_eq!(nihao.pinyin, "nihao");
        // 同词多行（多音多行）保留首个读音：的 → de（di4 行被去重）。
        let de = samples.iter().find(|s| s.word == "的").unwrap();
        assert_eq!(de.pinyin, "de");
        assert_eq!(samples.iter().filter(|s| s.word == "的").count(), 1);
    }

    #[test]
    fn eval_set_truncates_by_top() {
        let samples = generate_word_eval_set(CEDICT_SAMPLES, FREQ_SAMPLES, 3);
        let words: Vec<&str> = samples.iter().map(|s| s.word.as_str()).collect();
        assert_eq!(words, vec!["的", "你好", "世界"]);
    }

    #[test]
    fn eval_set_renders_tsv() {
        let samples = generate_word_eval_set(CEDICT_SAMPLES, FREQ_SAMPLES, 2);
        let rendered = render_eval_set(&samples);
        assert_eq!(rendered, "的\tde\t1000\n你好\tnihao\t900\n");
    }

    #[test]
    fn eval_set_syllable_count_mismatch_rejected() {
        // “跑” 有 [pao3] 也列为单音节词；构造音节数 != 字数的行（四个汉字只给三个音节）。
        let cedict = "\
我四个字 我四个字 [wo3 si4 ge4] /bad/
";
        let freq = "我四个字 500\n";
        let samples = generate_word_eval_set(cedict, freq, 10);
        assert!(samples.is_empty());
    }
}
