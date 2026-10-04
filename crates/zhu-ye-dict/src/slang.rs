//! M6-P 网络语包：`build-slang` 与构建期内容把关（FR-016/017/021）。
//!
//! 输入全部在仓库内：`data/slang/seed.tsv`（种子表）、`data/slang/blocklist.tsv`
//! （把关负面清单，仅构建期使用、不进发行物）、`data/slang/gate-samples.tsv`
//! （正负抽查样例）。构建先跑抽查样例，负例漏放 > 0 或正例误杀 > 5% 即失败，
//! 通过后才过滤种子表并编译 `slang.zyct`，把关报告写入 `slang.gate.json`。

use std::collections::HashSet;
use std::fs;
use std::path::Path;

use serde::Serialize;

use zhu_ye_core::dict::DictionaryEntry;
use zhu_ye_core::pinyin::{segment_all, SyllableTable};

use crate::import::is_cjk_word;
use crate::m6::{sha256_hex, source_check, PinyinTables, ARTIFACTS_DIR, CACHE_DIR};

/// 种子表、负面清单与抽查样例的仓库相对路径。
pub const SLANG_SEED: &str = "data/slang/seed.tsv";
pub const SLANG_BLOCKLIST: &str = "data/slang/blocklist.tsv";
pub const SLANG_GATE_SAMPLES: &str = "data/slang/gate-samples.tsv";

/// 网络语词条统一静态词频（zipf×1000 尺度的中频值，与 base 同尺度）。
pub const SLANG_FREQUENCY: u64 = 5000;

/// 正例误杀上限（验收标准 FR-021：误杀 ≤5%）。
const MAX_FALSE_KILL_RATE: f64 = 0.05;

/// 把关判定结果：拦截时给出命中的清单串与类别，供审计。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateHit {
    pub pattern: String,
    pub category: String,
}

/// 构建期把关表（负面清单）。
#[derive(Debug, Default)]
pub struct Gate {
    pub version: String,
    contains: Vec<(String, String)>,
    exact: Vec<(String, String)>,
    allow: Vec<String>,
    keys: Vec<(String, String)>,
}

/// 比较前归一：全角 ASCII 转半角、大写转小写、去除全部空白。
#[must_use]
pub fn gate_normalize(text: &str) -> String {
    text.chars()
        .filter(|ch| !ch.is_whitespace())
        .map(|ch| match ch {
            '\u{FF01}'..='\u{FF5E}' => char::from_u32(ch as u32 - 0xFEE0).unwrap_or(ch),
            _ => ch,
        })
        .flat_map(char::to_lowercase)
        .collect()
}

impl Gate {
    /// 解析 `blocklist.tsv`：`词 模式 类别`，`# version:` 行给出版本号。
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut gate = Gate::default();
        for (number, line) in text.lines().enumerate() {
            let line = line.trim();
            if let Some(version) = line.strip_prefix("# version:") {
                gate.version = version.trim().to_owned();
                continue;
            }
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let fields: Vec<&str> = line.split_whitespace().collect();
            let [term, mode, category] = fields[..] else {
                return Err(format!("负面清单第 {} 行字段数不为 3：{line}", number + 1));
            };
            let term = gate_normalize(term);
            let category = category.to_owned();
            match mode {
                "包含" => gate.contains.push((term, category)),
                "精确" => gate.exact.push((term, category)),
                "放行" => gate.allow.push(term),
                "缩写键" => gate.keys.push((term, category)),
                other => {
                    return Err(format!("负面清单第 {} 行模式未知：{other}", number + 1));
                }
            }
        }
        if gate.version.is_empty() {
            return Err("负面清单缺少 `# version:` 版本行".to_owned());
        }
        Ok(gate)
    }

    /// 检查词形：精确命中，或包含命中且至少一处出现未被放行串覆盖。
    #[must_use]
    pub fn check_word(&self, word: &str) -> Option<GateHit> {
        let text = gate_normalize(word);
        for (term, category) in &self.exact {
            if &text == term {
                return Some(GateHit {
                    pattern: term.clone(),
                    category: category.clone(),
                });
            }
        }
        for (term, category) in &self.contains {
            let uncovered = text
                .match_indices(term.as_str())
                .any(|(start, _)| !self.is_allowed_at(&text, start, start + term.len()));
            if uncovered {
                return Some(GateHit {
                    pattern: term.clone(),
                    category: category.clone(),
                });
            }
        }
        None
    }

    /// 检查缩写键：与 `缩写键` 条目归一后完全相同即拦截。
    #[must_use]
    pub fn check_key(&self, key: &str) -> Option<GateHit> {
        let key = gate_normalize(key);
        self.keys
            .iter()
            .find(|(term, _)| *term == key)
            .map(|(term, category)| GateHit {
                pattern: term.clone(),
                category: category.clone(),
            })
    }

    /// 字节区间 `[start, end)` 是否落在某个放行串的一次出现之内。
    fn is_allowed_at(&self, text: &str, start: usize, end: usize) -> bool {
        self.allow.iter().any(|allow| {
            text.match_indices(allow.as_str())
                .any(|(at, _)| at <= start && at + allow.len() >= end)
        })
    }
}

/// 一条被拦截的正例（审计用）。
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct KilledSample {
    pub text: String,
    pub pattern: String,
    pub category: String,
}

/// 抽查样例评估结果。
#[derive(Debug, Default, Clone, Serialize, PartialEq)]
pub struct GateAudit {
    pub negative_total: usize,
    pub negative_blocked: usize,
    pub negative_leaked: Vec<String>,
    pub positive_total: usize,
    pub positive_killed: Vec<KilledSample>,
    pub false_kill_rate: f64,
}

/// 按 `gate-samples.tsv`（`标签<TAB>类型<TAB>文本`）评估把关表。
pub fn audit_gate(gate: &Gate, samples: &str) -> Result<GateAudit, String> {
    let mut audit = GateAudit::default();
    for (number, line) in samples.lines().enumerate() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        let [label, kind, text] = fields[..] else {
            return Err(format!("抽查样例第 {} 行字段数不为 3", number + 1));
        };
        let hit = match kind {
            "词" => gate.check_word(text),
            "键" => gate.check_key(text).or_else(|| gate.check_word(text)),
            other => return Err(format!("抽查样例第 {} 行类型未知：{other}", number + 1)),
        };
        match label {
            "负" => {
                audit.negative_total += 1;
                if hit.is_some() {
                    audit.negative_blocked += 1;
                } else {
                    audit.negative_leaked.push(text.to_owned());
                }
            }
            "正" => {
                audit.positive_total += 1;
                if let Some(hit) = hit {
                    audit.positive_killed.push(KilledSample {
                        text: text.to_owned(),
                        pattern: hit.pattern,
                        category: hit.category,
                    });
                }
            }
            other => return Err(format!("抽查样例第 {} 行标签未知：{other}", number + 1)),
        }
    }
    if audit.positive_total > 0 {
        audit.false_kill_rate = audit.positive_killed.len() as f64 / audit.positive_total as f64;
    }
    Ok(audit)
}

/// 种子表一行。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeedRow {
    pub word: String,
    pub key: String,
    pub abbreviation: bool,
}

/// 解析 `seed.tsv`（`词<TAB>键<TAB>类型<TAB>来源`）。
pub fn parse_seed(text: &str) -> Result<Vec<SeedRow>, String> {
    let mut rows = Vec::new();
    for (number, line) in text.lines().enumerate() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() != 4 {
            return Err(format!("种子表第 {} 行字段数不为 4：{line}", number + 1));
        }
        let abbreviation = match fields[2] {
            "词" => false,
            "缩写" => true,
            other => return Err(format!("种子表第 {} 行类型未知：{other}", number + 1)),
        };
        if fields[0].trim().is_empty() {
            return Err(format!("种子表第 {} 行词为空", number + 1));
        }
        if fields[3].trim().is_empty() {
            return Err(format!("种子表第 {} 行缺少来源注释", number + 1));
        }
        rows.push(SeedRow {
            word: fields[0].trim().to_owned(),
            key: fields[1].trim().to_owned(),
            abbreviation,
        });
    }
    Ok(rows)
}

/// 缩写键判定（方案 11.4）：小写字母/数字、至少 2 位；纯字母键必须整串不可切分为
/// 标准全拼音节（可切分串如 `wa`、`emo` 走拼音路径，绝不进入缩写路径）。
/// 返回归一后的键；不合格返回 `Err(原因)`。
pub fn abbreviation_key(table: &SyllableTable, key: &str) -> Result<String, &'static str> {
    let key = key.to_ascii_lowercase();
    if key.len() < 2 {
        return Err("长度不足 2");
    }
    if !key
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit())
    {
        return Err("含非字母数字字符");
    }
    if key.chars().all(|ch| ch.is_ascii_lowercase()) && !segment_all(table, &key).is_empty() {
        return Err("可整串切分为拼音");
    }
    Ok(key)
}

/// 纯中文网络词的显式拼音（撇号分隔）校验：音节合法且音节数 = 字数。
fn explicit_pinyin(table: &SyllableTable, word: &str, key: &str) -> Option<String> {
    let syllables: Vec<&str> = key.split('\'').collect();
    if syllables.len() != word.chars().count()
        || syllables.iter().any(|s| !table.is_complete_syllable(s))
    {
        return None;
    }
    Some(syllables.concat())
}

/// 被排除的种子条目（审计用）。
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ExcludedSeed {
    pub word: String,
    pub key: String,
    pub reason: String,
}

/// build-slang 统计与把关报告（写入 `slang.gate.json`）。
#[derive(Debug, Default, Clone, Serialize, PartialEq)]
pub struct SlangReport {
    pub blocklist_version: String,
    pub audit: GateAudit,
    pub seed_rows: usize,
    pub social_rows: usize,
    pub word_entries: usize,
    pub abbreviation_entries: usize,
    pub gate_blocked: Vec<ExcludedSeed>,
    pub excluded: Vec<ExcludedSeed>,
    pub entry_count: usize,
    pub file_size: u64,
    pub sha256: String,
}

/// 构建网络语包：抽查把关 → 过滤种子表 → 注音/缩写键校验 → v2 编译。
pub fn build_slang(root: &Path) -> Result<SlangReport, String> {
    source_check(root)?;
    let read = |rel: &str| {
        fs::read_to_string(root.join(rel)).map_err(|error| format!("读取 {rel} 失败: {error}"))
    };
    let gate = Gate::parse(&read(SLANG_BLOCKLIST)?)?;
    let audit = audit_gate(&gate, &read(SLANG_GATE_SAMPLES)?)?;
    if audit.negative_total < 200 || audit.positive_total < 200 {
        return Err(format!(
            "把关抽查样例不足：负例 {}、正例 {}（各需 ≥200）",
            audit.negative_total, audit.positive_total
        ));
    }
    if !audit.negative_leaked.is_empty() {
        return Err(format!(
            "把关负例漏放 {} 条（零容忍）：{:?}",
            audit.negative_leaked.len(),
            audit.negative_leaked
        ));
    }
    if audit.false_kill_rate > MAX_FALSE_KILL_RATE {
        return Err(format!(
            "把关正例误杀率 {:.1}% 超过 5%：{:?}",
            audit.false_kill_rate * 100.0,
            audit.positive_killed
        ));
    }

    let mut rows = parse_seed(&read(SLANG_SEED)?)?;
    let seed_rows = rows.len();
    let mut social_rows = 0usize;
    if root.join(crate::social::SLANG_SOCIAL).exists() {
        let social = parse_seed(&read(crate::social::SLANG_SOCIAL)?)?;
        social_rows = social.len();
        rows.extend(social);
    }
    let cedict_text = read("data/raw/cedict_ts.u8")?;
    let ktghz_text = read(&format!("{CACHE_DIR}/kTGHZ2013.txt"))?;
    let tables = PinyinTables::from_texts(&cedict_text, &ktghz_text);
    let table = SyllableTable::standard();

    let mut report = SlangReport {
        blocklist_version: gate.version.clone(),
        seed_rows,
        social_rows,
        ..SlangReport::default()
    };
    let mut entries = Vec::new();
    let mut seen = HashSet::new();
    for row in &rows {
        let hit = gate.check_word(&row.word).or_else(|| {
            if row.abbreviation {
                gate.check_key(&row.key)
            } else {
                None
            }
        });
        if let Some(hit) = hit {
            report.gate_blocked.push(ExcludedSeed {
                word: row.word.clone(),
                key: row.key.clone(),
                reason: format!("{}：{}", hit.category, hit.pattern),
            });
            continue;
        }
        let key = if row.abbreviation {
            match abbreviation_key(&table, &row.key) {
                Ok(key) => key,
                Err(reason) => {
                    report.excluded.push(ExcludedSeed {
                        word: row.word.clone(),
                        key: row.key.clone(),
                        reason: reason.to_owned(),
                    });
                    continue;
                }
            }
        } else {
            let pinyin = if !is_cjk_word(&row.word) {
                None
            } else if row.key.is_empty() {
                tables.annotate(&row.word)
            } else {
                explicit_pinyin(&table, &row.word, &row.key)
            };
            let Some(pinyin) = pinyin else {
                report.excluded.push(ExcludedSeed {
                    word: row.word.clone(),
                    key: row.key.clone(),
                    reason: "注音失败或非纯汉字".to_owned(),
                });
                continue;
            };
            pinyin
        };
        if !seen.insert((row.word.clone(), key.clone())) {
            continue;
        }
        if row.abbreviation {
            report.abbreviation_entries += 1;
        } else {
            report.word_entries += 1;
        }
        entries.push(DictionaryEntry::new(row.word.clone(), key, SLANG_FREQUENCY));
    }

    let bytes = crate::build_v2(&entries, &[]).map_err(|error| error.to_string())?;
    let artifacts = root.join(ARTIFACTS_DIR);
    fs::create_dir_all(&artifacts).map_err(|error| format!("创建输出目录失败: {error}"))?;
    fs::write(artifacts.join("slang.zyct"), &bytes)
        .map_err(|error| format!("写入 slang.zyct 失败: {error}"))?;
    report.audit = audit;
    report.entry_count = entries.len();
    report.file_size = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
    report.sha256 = sha256_hex(&bytes);
    let json = serde_json::to_string_pretty(&report)
        .map_err(|error| format!("序列化把关报告失败: {error}"))?;
    fs::write(artifacts.join("slang.gate.json"), json + "\n")
        .map_err(|error| format!("写入 slang.gate.json 失败: {error}"))?;
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    const BLOCKLIST: &str = "# version: t-1\n傻逼 包含 辱骂\n大麻 包含 违禁\n大麻烦 放行 豁免\n坏词 精确 广告\nnmsl 缩写键 辱骂\n";

    #[test]
    fn 把关归一与包含放行精确缩写键() {
        let gate = Gate::parse(BLOCKLIST).unwrap();
        assert_eq!(gate.version, "t-1");
        assert_eq!(gate_normalize("ＮＭ SL"), "nmsl");
        assert!(gate.check_word("你个傻 逼").is_some());
        // 放行串覆盖命中位置 -> 不拦截；未覆盖的另一处出现 -> 拦截
        assert!(gate.check_word("大麻烦").is_none());
        assert_eq!(
            gate.check_word("大麻烦的大麻").map(|hit| hit.category),
            Some("违禁".to_owned())
        );
        // 精确模式只拦截完全相同
        assert!(gate.check_word("坏词").is_some());
        assert!(gate.check_word("坏词语").is_none());
        assert!(gate.check_key("NMSL").is_some());
        assert!(gate.check_key("ｎｍｓｌ").is_some());
        assert!(gate.check_key("nmslx").is_none());
        assert!(Gate::parse("傻逼 包含 辱骂\n").is_err(), "缺版本行应失败");
        assert!(Gate::parse("# version: x\n傻逼 未知 辱骂\n").is_err());
    }

    #[test]
    fn 抽查样例统计漏放与误杀() {
        let gate = Gate::parse(BLOCKLIST).unwrap();
        let samples =
            "# 注释\n负\t词\t傻逼\n负\t键\tNMSL\n负\t词\t内卷\n正\t词\t大麻烦\n正\t词\t大麻哈\n";
        let audit = audit_gate(&gate, samples).unwrap();
        assert_eq!(audit.negative_total, 3);
        assert_eq!(audit.negative_blocked, 2);
        assert_eq!(audit.negative_leaked, vec!["内卷".to_owned()]);
        assert_eq!(audit.positive_total, 2);
        assert_eq!(audit.positive_killed.len(), 1);
        assert_eq!(audit.positive_killed[0].text, "大麻哈");
        assert!((audit.false_kill_rate - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn 缩写键长度与不可切分规则() {
        let table = SyllableTable::standard();
        assert_eq!(abbreviation_key(&table, "YYDS").as_deref(), Ok("yyds"));
        assert_eq!(abbreviation_key(&table, "u1s1").as_deref(), Ok("u1s1"));
        assert_eq!(abbreviation_key(&table, "996").as_deref(), Ok("996"));
        assert_eq!(abbreviation_key(&table, "y"), Err("长度不足 2"));
        // 可切分为拼音的串绝不进入缩写路径
        assert_eq!(abbreviation_key(&table, "wo"), Err("可整串切分为拼音"));
        assert_eq!(abbreviation_key(&table, "emo"), Err("可整串切分为拼音"));
        assert_eq!(abbreviation_key(&table, "a-b"), Err("含非字母数字字符"));
    }

    #[test]
    fn 种子表解析与显式拼音校验() {
        let text = "# 注释\n内卷\t\t词\t维护者\n长草\tzhang'cao\t词\t维护者\n永远的神\tyyds\t缩写\t维护者\n";
        let rows = parse_seed(text).unwrap();
        assert_eq!(rows.len(), 3);
        assert!(rows[2].abbreviation);
        assert!(parse_seed("内卷\t\t词\t\n").is_err(), "缺来源应失败");
        assert!(parse_seed("内卷\t\t未知\t维护者\n").is_err());
        let table = SyllableTable::standard();
        assert_eq!(
            explicit_pinyin(&table, "长草", "zhang'cao").as_deref(),
            Some("zhangcao")
        );
        assert_eq!(explicit_pinyin(&table, "长草", "zhangcao"), None);
        assert_eq!(explicit_pinyin(&table, "长草", "zhang'xx"), None);
    }
}
