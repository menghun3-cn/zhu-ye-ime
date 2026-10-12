//! 命中率评测样本生成（T-057）。
//!
//! 评测集生成器：以 CC-CEDICT 词级注音为读音来源、wordfreq 词频表为目标词分布，
//! 产出 `词<TAB>无调连续全拼<TAB>词频` 的评测词样本，供 `zhu-ye-cli eval` 判定
//! Top1 / Top3 命中率。清洗口径与 real dict 导入（`crate::import`）完全一致，
//! 保证样本本身在构建与评测两侧语义对齐。

use std::collections::HashSet;
use std::path::Path;

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

/// T-147 口语短语验收结果。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PhraseCheckReport {
    /// 清单总条数。
    pub total: usize,
    /// 拼音组有候选的条数。
    pub with_candidates: usize,
    /// 首候选正确条数。
    pub first_hit: usize,
    /// 首候选未命中明细（目标词, 实际首候选, 拼音）。
    pub misses: Vec<(String, String, String)>,
    /// 首候选正确率（= first_hit / total）。
    pub first_hit_pct: f64,
}

/// 口语短语验收（T-147，Q4a）：读验收清单（`词\t无调连续全拼`，`#` 注释与
/// 空行忽略），对每条按**引擎候选口径**（`generate_candidates`）查拼音组，
/// 首候选必须等于目标词。作为统计驱动短语扩充的回归防线：清单只增不减，
/// 语料每轮更新后跑 `check-phrases`，正确率门槛 ≥95%（CLI 侧以退出码门禁）。
pub fn check_phrase_checklist(
    base_path: &Path,
    checklist_text: &str,
) -> Result<PhraseCheckReport, String> {
    use zhu_ye_core::candidate::generate_candidates;
    use zhu_ye_core::dict::Dictionary;
    use zhu_ye_core::pinyin::SyllableTable;
    use zhu_ye_core::DictionaryFile;

    let file = DictionaryFile::open(base_path)
        .map_err(|error| format!("打开 base 包失败（{}）: {error}", base_path.display()))?;
    let dictionary: &dyn Dictionary = &file;
    let table = SyllableTable::standard();

    let mut report = PhraseCheckReport::default();
    for (index, raw) in checklist_text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut parts = line.split('\t');
        let word = parts.next().unwrap_or("").trim();
        let pinyin = parts.next().unwrap_or("").trim();
        if word.is_empty() || pinyin.is_empty() {
            return Err(format!(
                "口语短语验收清单第 {} 行缺词或拼音: {raw}",
                index + 1
            ));
        }
        if pinyin.contains(' ') || pinyin.chars().any(|c| c.is_ascii_uppercase()) {
            return Err(format!(
                "口语短语验收清单第 {} 行拼音须为无调小写连续全拼: {raw}",
                index + 1
            ));
        }
        report.total += 1;
        let candidates = generate_candidates(&table, dictionary, pinyin);
        if !candidates.is_empty() {
            report.with_candidates += 1;
        }
        match candidates.first() {
            Some(first) if first.text == word => report.first_hit += 1,
            Some(first) if report.misses.len() < 200 => {
                report
                    .misses
                    .push((word.to_owned(), first.text.clone(), pinyin.to_owned()));
            }
            None => report
                .misses
                .push((word.to_owned(), "(无候选)".to_owned(), pinyin.to_owned())),
            _ => {}
        }
    }
    if report.total > 0 {
        report.first_hit_pct = report.first_hit as f64 / report.total as f64 * 100.0;
    }
    Ok(report)
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

    #[test]
    fn check_phrase_checklist_按引擎口径命中首候选() {
        use std::path::PathBuf;

        let dir = std::env::temp_dir().join("zhu-ye-eval-t147-check");
        std::fs::create_dir_all(&dir).unwrap();
        let bytes = crate::build_v2(
            &[
                zhu_ye_core::dict::DictionaryEntry::new("你是", "nishi", 5100),
                zhu_ye_core::dict::DictionaryEntry::new("逆势", "nishi", 2600),
                zhu_ye_core::dict::DictionaryEntry::new("我有", "woyou", 4900),
            ],
            &[],
        )
        .expect("mini 词典构建");
        let path: PathBuf = dir.join("mini.zyct");
        std::fs::write(&path, bytes).unwrap();
        let report = check_phrase_checklist(&path, "你是\tnishi\n我有\twoyou\n").unwrap();
        assert_eq!(report.total, 2);
        assert_eq!(report.with_candidates, 2);
        assert_eq!(report.first_hit, 2);
        assert!(report.misses.is_empty());
        assert!((report.first_hit_pct - 100.0).abs() < 1e-9);
        // 同音组按词频降序：mini 词典 nishi 组首候选是 5100 的「你是」，
        // 2600 的「逆势」不是首候选 → 未命中且记入明细（引擎口径本身正确）。
        let report2 = check_phrase_checklist(&path, "逆势\tnishi\n").unwrap();
        assert_eq!(report2.first_hit, 0);
        assert_eq!(
            report2.misses[0],
            ("逆势".to_owned(), "你是".to_owned(), "nishi".to_owned())
        );
        // 缺失词 → 无候选且计入未命中明细。
        let report3 = check_phrase_checklist(&path, "我知道\twozhidao\n").unwrap();
        assert_eq!(report3.with_candidates, 0);
        assert_eq!(report3.first_hit, 0);
        assert_eq!(report3.misses.len(), 1);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn check_phrase_checklist_清单非法行报错() {
        let dir = std::env::temp_dir().join("zhu-ye-eval-t147-bad");
        std::fs::create_dir_all(&dir).unwrap();
        let bytes = crate::build_v2(&[], &[]).expect("mini 词典构建");
        let path = dir.join("mini.zyct");
        std::fs::write(&path, bytes).unwrap();
        // 拼音带空格/大写 → 报错提示无调小写连续全拼。
        let err = check_phrase_checklist(&path, "你是\tni shi\n").unwrap_err();
        assert!(err.contains("无调小写"), "实际: {err}");
        // 空词或空拼音 → 报错带行号。
        let err2 = check_phrase_checklist(&path, "你是\n").unwrap_err();
        assert!(err2.contains("第 1 行缺词或拼音"), "实际: {err2}");
        // "#" 注释与空行忽略。
        let report = check_phrase_checklist(&path, "# 注释\n\n你是\tnishi\n").unwrap();
        assert_eq!(report.total, 1);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
