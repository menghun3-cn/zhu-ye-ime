//! T-087 网络语扩充（FR-047，D-52）：social-media-chinese-words 清洗出高频子集。
//!
//! 输入（仓库外，pin 锁定）：`data/cache/social-media-chinese-words.txt`（gitignored，
//! 由 `scripts/fetch-social-media.ps1` 下载合并，SHA-256 见 `data/pins/social-media-zh.json`）。
//! 规则输入（仓库内）：`data/slang/blocklist.tsv`（把关负面清单）、`data/slang/seed.tsv`
//! （种子表，重复词剔除、权重并入既有项不新增）。
//! 输出：`data/slang/social-words.tsv`（`词<TAB>键<TAB>类型<TAB>来源`，与 seed.tsv 同构，
//! commit 进仓库，构建期由 `build-slang` 合并入包）。
//!
//! 清洗规则（确定性、可复现，版本 v1）：
//! - 词形 = 汉字 + ASCII 字母/数字，且至少 1 个汉字（纯英文/纯数字/含符号都剔除）
//! - 长度（字符数）2..=12
//! - 去重：按小写归一保存首个出现（同形同义仅保首现）
//! - 把关负面清单（blocklist）拦截即剔除
//! - 与种子表词条完全重复即剔除（权重并入既有项，不新增）
//! - 词频：按原语料词频字段降序排列后截取前 10000（稳定性保文件顺序）

use std::collections::HashSet;
use std::fs;
use std::path::Path;

use serde::Serialize;

use crate::m6::sha256_file;
use crate::slang::{parse_seed, Gate, SLANG_BLOCKLIST, SLANG_SEED};

/// 合并语料缓存相对路径（gitignored；pin：social-media-zh）。
pub const SOCIAL_CACHE: &str = "data/cache/social-media-chinese-words.txt";
/// 清洗产物相对路径（commit 进仓库，构建期由 build-slang 合并）。
pub const SLANG_SOCIAL: &str = "data/slang/social-words.tsv";
/// 高频子集条数上限（D-52：约 1 万）。
pub const SOCIAL_LIMIT: usize = 10_000;

/// 一条通过过滤的词条（附原始词频供排序）。
struct Candidate {
    word: String,
    freq: u64,
}

/// social-clean 统计与产物报告（命令打印；哈希供验收回填）。
#[derive(Debug, Default, Clone, Serialize, PartialEq)]
pub struct SocialCleanReport {
    pub input_rows: usize,
    pub shape_rejected: usize,
    pub gate_blocked: usize,
    pub seed_duplicates: usize,
    pub deduplicated: usize,
    pub output_rows: usize,
    pub output_sha256: String,
    pub cache_sha256: String,
}

/// 解析语料行 `词 词频`（词不含空白；末字段为频次，防御性 join 剩余部分为词）。
fn parse_line(line: &str) -> Option<(String, u64)> {
    let mut tokens = line.split_whitespace();
    let last = tokens.next_back()?;
    let freq: u64 = last.parse().ok()?;
    let word: String = tokens.collect::<Vec<_>>().join(" ");
    if word.is_empty() {
        return None;
    }
    Some((word, freq))
}

/// 词形过滤：汉字 + ASCII 字母/数字，至少 1 个汉字，长度 2..=12。
fn is_clean_shape(word: &str) -> bool {
    let mut has_han = false;
    let mut chars = 0usize;
    for ch in word.chars() {
        let cjk =
            ('\u{4e00}'..='\u{9fff}').contains(&ch) || ('\u{3400}'..='\u{4dbf}').contains(&ch);
        let ascii = ch.is_ascii_alphanumeric();
        if !(cjk || ascii) {
            return false;
        }
        if cjk {
            has_han = true;
        }
        chars += 1;
    }
    has_han && (2..=12).contains(&chars)
}

/// 去重归一键：ASCII 部分转小写（`iPhone`/`iphone` 视为同形）。
fn dedup_key(word: &str) -> String {
    word.chars()
        .map(|ch| {
            if ch.is_ascii() {
                ch.to_ascii_lowercase()
            } else {
                ch
            }
        })
        .collect()
}

/// 清洗主流程：读合并缓存 → 过滤 → 按词频降序 → 去重保首现 → 截取前 1 万 → 写产出。
///
/// 不在此处做 `source_check`（调用方负责；测试保持纯文件逻辑）。
pub fn clean_social(root: &Path) -> Result<SocialCleanReport, String> {
    let read = |rel: &str| {
        fs::read_to_string(root.join(rel)).map_err(|error| format!("读取 {rel} 失败: {error}"))
    };
    let cache_text = read(SOCIAL_CACHE)?;
    let gate = Gate::parse(&read(SLANG_BLOCKLIST)?)?;
    let seed_words: HashSet<String> = parse_seed(&read(SLANG_SEED)?)?
        .into_iter()
        .map(|row| row.word)
        .collect();

    let mut report = SocialCleanReport {
        cache_sha256: sha256_file(&root.join(SOCIAL_CACHE))?,
        ..SocialCleanReport::default()
    };

    // 1) 行解析 + 形状/把关/种子重复过滤
    let mut candidates: Vec<Candidate> = Vec::new();
    let mut seen_norm: HashSet<String> = HashSet::new();
    for line in cache_text.lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        report.input_rows += 1;
        let Some((word, freq)) = parse_line(line) else {
            report.shape_rejected += 1;
            continue;
        };
        if !is_clean_shape(&word) {
            report.shape_rejected += 1;
            continue;
        }
        if gate.check_word(&word).is_some() {
            report.gate_blocked += 1;
            continue;
        }
        if seed_words.contains(&word) {
            report.seed_duplicates += 1;
            continue;
        }
        let norm = dedup_key(&word);
        if !seen_norm.insert(norm) {
            report.deduplicated += 1;
            continue;
        }
        candidates.push(Candidate { word, freq });
    }

    // 2) 按词频降序（稳定：同频保语料顺序）→ 截取前 1 万
    candidates.sort_by_key(|candidate| std::cmp::Reverse(candidate.freq));
    candidates.truncate(SOCIAL_LIMIT);
    report.output_rows = candidates.len();

    // 3) 写出 tsv（4 列同 seed.tsv；来源列标注上游与许可）
    let mut out = String::new();
    out.push_str("# 竹叶输入法 网络语扩充子集（T-087，FR-047，D-52）\n");
    out.push_str("# 源：social-media-chinese-words（MIT，github.com/jilelab/social-media-chinese-words，2021-05；合并文件 SHA-256 ");
    out.push_str(&report.cache_sha256);
    out.push_str("，107.78 万行）\n");
    out.push_str("# 清洗规则 v1：词形=汉字+ASCII 字母/数字且≥1 汉字、长度 2-12、去重保首现、\n");
    out.push_str(
        "#   把关负面清单拦截、与种子表重复剔除（权重并入既有项）、按原语料词频降序取前 10000\n",
    );
    out.push_str(
        "# 由 zhu-ye-dict social-clean 生成（确定性可复现）；格式同 data/slang/seed.tsv\n",
    );
    out.push_str("# 许可同源（MIT），来源列 = social-media-chinese-words\n");
    for candidate in &candidates {
        out.push_str(&candidate.word);
        out.push_str("\t\t词\tsocial-media-chinese-words\n");
    }
    fs::write(root.join(SLANG_SOCIAL), out)
        .map_err(|error| format!("写入 {SLANG_SOCIAL} 失败: {error}"))?;
    report.output_sha256 = sha256_file(&root.join(SLANG_SOCIAL))?;
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 语料行解析() {
        assert_eq!(
            parse_line("绘画教程 40210").unwrap(),
            ("绘画教程".into(), 40210)
        );
        assert_eq!(parse_line("slime 36329").unwrap(), ("slime".into(), 36329));
        assert_eq!(parse_line("  abc  12  ").unwrap(), ("abc".into(), 12));
        assert!(parse_line("没有频次").is_none());
        assert!(parse_line("频次非数字 x").is_none());
        assert!(parse_line("").is_none());
    }

    #[test]
    fn 词形过滤规则() {
        assert!(is_clean_shape("绘画教程"));
        assert!(is_clean_shape("iPhone15Pro小")); // 汉字+字母数字
        assert!(is_clean_shape("栓Q"));
        assert!(!is_clean_shape("slime")); // 纯英文（无汉字）
        assert!(!is_clean_shape("iphone15pro")); // 纯字母数字（无汉字）
        assert!(!is_clean_shape("12345")); // 纯数字（无汉字）
        assert!(!is_clean_shape("画")); // 1 字符
        assert!(!is_clean_shape("一二三四五六七八九十甲乙丙丁戊")); // 13 字符
        assert!(!is_clean_shape("绘画&教程")); // 含符号
        assert!(!is_clean_shape("")); // 空
    }

    #[test]
    fn 清洗端到端过滤与截取() {
        // 临时仓库：cache 语料 + blocklist + seed + data/slang 输出目录
        let dir = std::env::temp_dir().join(format!("social-clean-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let cache = dir.join(SOCIAL_CACHE);
        fs::create_dir_all(cache.parent().unwrap()).unwrap();
        let text = "热门词 9000\n普通词 300\n长被拦截词 8000\n阳台词 7000\niPhone15Pro小 6000\niphone15pro小 6000\n短 1\n";
        fs::write(&cache, text).unwrap();
        let blocklist = dir.join(SLANG_BLOCKLIST);
        fs::create_dir_all(blocklist.parent().unwrap()).unwrap();
        fs::write(&blocklist, "# version: v1\n拦截 包含 测试\n").unwrap();
        let seed = dir.join(SLANG_SEED);
        fs::create_dir_all(seed.parent().unwrap()).unwrap();
        fs::write(&seed, "# 种子\n阳台词\t\t词\t维护者\n").unwrap();

        let report = clean_social(&dir).unwrap();
        // 输入占位行解析合格：热门词/普通词/长被拦截词/阳台词 /iPhone15Pro×2/短 = 7 行
        assert_eq!(report.input_rows, 7);
        assert_eq!(report.shape_rejected, 1); // "短"（1 字符）
        assert_eq!(report.gate_blocked, 1); // 长被拦截词
        assert_eq!(report.seed_duplicates, 1); // 阳台词（种子重复）
        assert_eq!(report.deduplicated, 1); // iphone15pro（同形小写）

        let out = fs::read_to_string(dir.join(SLANG_SOCIAL)).unwrap();
        let rows: Vec<&str> = out
            .lines()
            .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
            .collect();
        assert_eq!(rows.len(), 3); // 热门词 + iPhone15Pro小 + 普通词（截取上限内）
        assert!(rows[0].starts_with("热门词\t\t词\t")); // 词频最高者在首行
        assert!(rows[1].starts_with("iPhone15Pro小\t\t词\t")); // 同形去重后保留首现原形
        assert!(rows[2].contains("普通词"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn 截取上限一万物种保持() {
        let dir = std::env::temp_dir().join(format!("social-limit-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let cache = dir.join(SOCIAL_CACHE);
        fs::create_dir_all(cache.parent().unwrap()).unwrap();
        let mut text = String::new();
        // 20_000 行：freq = i，第 i 行为 "词{i} {i}"
        for i in 1..=20_000u64 {
            text.push_str(&format!("词{i} {i}\n"));
        }
        fs::write(&cache, text).unwrap();
        let blocklist = dir.join(SLANG_BLOCKLIST);
        fs::create_dir_all(blocklist.parent().unwrap()).unwrap();
        fs::write(&blocklist, "# version: v1\n").unwrap();
        let seed = dir.join(SLANG_SEED);
        fs::create_dir_all(seed.parent().unwrap()).unwrap();
        fs::write(&seed, "# 种子\n").unwrap();

        let report = clean_social(&dir).unwrap();
        assert_eq!(report.input_rows, 20_000);
        assert_eq!(report.output_rows, SOCIAL_LIMIT);
        let out = fs::read_to_string(dir.join(SLANG_SOCIAL)).unwrap();
        let rows: Vec<&str> = out
            .lines()
            .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
            .collect();
        assert_eq!(rows.len(), SOCIAL_LIMIT);
        // 频次最高的在首行：词20000
        assert!(rows[0].starts_with("词20000\t"));
        // 频次最低的被截断：词1 不在产物
        assert!(rows.iter().all(|r| !r.starts_with("词1\t")));
        let _ = fs::remove_dir_all(&dir);
    }
}
