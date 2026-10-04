//! 英文词表（`en.zyen`）构建管线（T-085，方案设计 14.2）。
//!
//! 数据流：
//!   1. ECDICT 全量 CSV（D-002，CC BY-SA 4.0，`data/cache/ecdict-full.csv`）
//!      解析 word/bnc/frq 三列 → 清洗（ASCII 词形/长度 2..=40/污染条目过滤）→
//!      小写归一化去重（同键取 frq 最高、次 bnc、再词形字典序确定）；
//!   2. 合并：FrequencyWords 英文词频 top10000（D-018，ECDICT 未收录者补入，
//!      以其词频数占位 frq 参与排序）+ CC-CEDICT 英文侧（D-001，纯单词词目
//!      补入，frq=0 排在 ECDICT 词频条目之后）；
//!   3. 补丁：`data/patches/en-capitals.tsv`（改原形/强推排序）与
//!      `data/patches/en-exclude.tsv`（剔除噪声词）；
//!   4. 排序：frq>0 组优先、frq 降序、bnc 降序、norm 字典序 → 应用 rank 补丁
//!      （1 基目标位）→ 赋最终 rank（0 = 最常用）→ 按 norm 升序编译 ZYEN v1
//!      （`compile_en_records`；rank 随记录携带，展示序不受存储序影响）。
//!
//! 大小写策略（D-09）：同一 norm 保留词形确定性最优（ECDICT word 列原形；
//! 全小写词条 norm 与 word 共享池段）；`en-capitals.tsv` 可覆盖原形。
//! 排序口径（D-10）：rank 仅英文候选组内排序，不参与中文静态排序。
//!
//! 本模块不联网：ECDICT 下载/校验由 `scripts/build-en-wordbook.ps1`
//! （SHA-256 锁定，数据清单 D-002）负责，这里只消费本地文件。

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use zhu_ye_core::en_lexicon::{compile_en_records, EnRecord};

/// ECDICT 词条清洗长度上限（污染条目过滤：词典内混入整句释义的长串）。
pub const ECDICT_MAX_WORD_LEN: usize = 40;

/// FrequencyWords 补入上限（D-018 top N）。
pub const FREQ_WORDS_TOP: usize = 10000;

/// 实现反馈的 ECDICT 全量统计。
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct EnWordbookStats {
    /// ECDICT 解析出的原始数据行数。
    pub ecdict_rows: u64,
    /// 清洗通过的行数。
    pub ecdict_kept: u64,
    /// ECDICT 小写去重后的唯一键数。
    pub unique_norms: u64,
    /// FrequencyWords 补入数（ECDICT 未收录且未被排除）。
    pub freqwords_added: u64,
    /// CC-CEDICT 英文侧补入数。
    pub cedict_added: u64,
    /// 排除清单剔除数（仅统计 ECDICT 段剔除；排除键亦阻止补入）。
    pub excluded: u64,
    /// 大小写补丁命中/新增数。
    pub capitals_patched: u64,
    /// 最终词条数（产物 count）。
    pub final_count: u64,
    /// 产物字节数。
    pub produced_bytes: u64,
    /// 含大写原形词条数（word != norm，需独立池段）。
    pub capitalized_words: u64,
}

/// 构建输入路径集合；`ecdict_csv` 必需，其余合并源可选。
#[derive(Debug, Clone)]
pub struct EnWordbookInputs<'a> {
    /// ECDICT 全量 CSV（必需，含 word/bnc/frq 列）。
    pub ecdict_csv: &'a Path,
    /// FrequencyWords 英文词频文本（D-018）。
    pub freqwords: Option<&'a Path>,
    /// CC-CEDICT 原文（D-001，英文侧提取）。
    pub cedict: Option<&'a Path>,
    /// 大小写补丁 TSV（norm、word、[rank]）。
    pub capitals: Option<&'a Path>,
    /// 排除清单（每行一个或多个 tab 分隔的 norm）。
    pub exclude: Option<&'a Path>,
}

/// 中间词条。
#[derive(Debug, Clone)]
struct Entry {
    norm: String,
    word: String,
    /// ECDICT frq 列（0 = 缺失）；FrequencyWords 补入以其词频数占位。
    frq: u64,
    /// ECDICT bnc 列（0 = 缺失）。
    bnc: u64,
    /// 排名覆盖（capitals 补丁提供，1 基目标位）。
    rank_override: Option<u32>,
}

/// 最小 CSV 字段读取：处理引号包裹、`""` 转义与字段内换行。
///
/// 返回字段切片并在 `pos` 处推进；`None` 表示记录结束。ECDICT 的
/// definition 字段含逗号与引号，解析必须容错。
fn csv_field_reader<'a>(record: &'a str, pos: &mut usize) -> Option<&'a str> {
    let bytes = record.as_bytes();
    if *pos >= bytes.len() {
        return None;
    }
    if bytes[*pos] == b',' {
        // 空字段：只消费一个分隔逗号。
        *pos += 1;
        return Some("");
    }
    if bytes[*pos] == b'"' {
        let mut end = *pos + 1;
        loop {
            if end >= bytes.len() {
                break;
            }
            if bytes[end] == b'"' {
                // `""` 为转义引号（字段内容），成对跳过；单个 `"` 为结束符。
                if end + 1 < bytes.len() && bytes[end + 1] == b'"' {
                    end += 2;
                } else {
                    break;
                }
            } else {
                end += 1;
            }
        }
        let field = &record[*pos + 1..end];
        // 越过结束引号与其后的分隔逗号（若存在）。
        *pos = end + 1;
        if *pos < bytes.len() && bytes[*pos] == b',' {
            *pos += 1;
        }
        Some(field)
    } else {
        let next = record[*pos..]
            .find(',')
            .map(|i| *pos + i)
            .unwrap_or(bytes.len());
        let field = &record[*pos..next];
        // 越过分隔逗号（若存在），下个字段从其后开始。
        *pos = if next < bytes.len() { next + 1 } else { next };
        Some(field)
    }
}

/// ECDICT CSV 一行的字段列表（`""` 转义已还原为 `"`）。
fn parse_csv_row(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut pos = 0usize;
    while let Some(field) = csv_field_reader(line, &mut pos) {
        out.push(field.replace("\"\"", "\""));
    }
    // 行以分隔逗号结尾时，末尾还有 1 个空字段（如 `a,b,` → [a, b, ""]）；
    // ECDICT 尾列（audio）多为空，行普遍以逗号结尾。
    if line.ends_with(',') {
        out.push(String::new());
    }
    out
}

/// 词形清洗：`word` 是否保留为英文词条。
///
/// 规则：仅 `[A-Za-z'\- ]`，至少含一个字母，长度 2..=40；
/// >40 为污染条目（如混入整句释义的 "madrigal: an addition..."）。
fn word_acceptable(word: &str) -> bool {
    let bytes = word.as_bytes();
    if bytes.len() < 2 || bytes.len() > ECDICT_MAX_WORD_LEN {
        return false;
    }
    let mut has_letter = false;
    for &b in bytes {
        if b.is_ascii_alphabetic() {
            has_letter = true;
        } else if !matches!(b, b'\'' | b'-' | b' ') {
            return false;
        }
    }
    has_letter
}

/// 读 ECDICT CSV，返回按 norm 去重后的词条表（同键取 frq 高、次 bnc 高、
/// 再词形字典序大——确定性）。
fn load_ecdict(path: &Path, stats: &mut EnWordbookStats) -> Result<Vec<Entry>, String> {
    let file =
        File::open(path).map_err(|e| format!("打开 ECDICT CSV 失败（{}）：{e}", path.display()))?;
    let mut lines = BufReader::new(file).lines();
    let header = lines
        .next()
        .transpose()
        .map_err(|e| format!("读 ECDICT 头行失败：{e}"))?
        .ok_or_else(|| "ECDICT CSV 为空（缺头行）".to_owned())?;
    let header_cols = parse_csv_row(&header);
    let col_word = header_cols
        .iter()
        .position(|c| c == "word")
        .ok_or_else(|| "ECDICT 头行缺 word 列".to_owned())?;
    let col_bnc = header_cols
        .iter()
        .position(|c| c == "bnc")
        .ok_or_else(|| "ECDICT 头行缺 bnc 列".to_owned())?;
    let col_frq = header_cols
        .iter()
        .position(|c| c == "frq")
        .ok_or_else(|| "ECDICT 头行缺 frq 列".to_owned())?;

    let mut by_norm: HashMap<String, Entry> = HashMap::new();
    let mut order: Vec<String> = Vec::new();
    for (line_no, line) in lines.enumerate() {
        let line = line.map_err(|e| format!("读 ECDICT 第 {} 行失败：{e}", line_no + 2))?;
        if line.trim().is_empty() {
            continue;
        }
        stats.ecdict_rows += 1;
        let cols = parse_csv_row(&line);
        let word = cols.get(col_word).map(String::as_str).unwrap_or("");
        if !word_acceptable(word) {
            continue;
        }
        let parse_u64 = |s: &str| s.trim().parse::<u64>().unwrap_or(0);
        let frq = cols.get(col_frq).map(|s| parse_u64(s)).unwrap_or(0);
        let bnc = cols.get(col_bnc).map(|s| parse_u64(s)).unwrap_or(0);
        stats.ecdict_kept += 1;
        let norm = word.to_ascii_lowercase();
        let better = match by_norm.get(&norm) {
            None => true,
            Some(prev) => (frq, bnc, word) > (prev.frq, prev.bnc, prev.word.as_str()),
        };
        if better {
            if !by_norm.contains_key(&norm) {
                order.push(norm.clone());
            }
            by_norm.insert(
                norm.clone(),
                Entry {
                    norm,
                    word: word.to_owned(),
                    frq,
                    bnc,
                    rank_override: None,
                },
            );
        }
    }
    let mut entries = Vec::with_capacity(order.len());
    let mut seen: HashSet<String> = HashSet::with_capacity(order.len());
    for norm in order {
        if seen.insert(norm.clone()) {
            if let Some(entry) = by_norm.remove(&norm) {
                entries.push(entry);
            }
        }
    }
    stats.unique_norms = entries.len() as u64;
    Ok(entries)
}

/// 读 FrequencyWords 文本（第 1 列词形、第 2 列频次，词形全小写），
/// 返回按频次降序截断 top N 的（词形, 词频）列表。
fn load_freqwords_top(path: &Path, top: usize) -> Result<Vec<(String, u64)>, String> {
    let file = File::open(path)
        .map_err(|e| format!("打开 FrequencyWords 失败（{}）：{e}", path.display()))?;
    let reader = BufReader::new(file);
    let mut rows: Vec<(String, u64)> = Vec::new();
    for (line_no, line) in reader.lines().enumerate() {
        let line = line.map_err(|e| format!("读 FrequencyWords 第 {} 行失败：{e}", line_no + 1))?;
        if line.is_empty() {
            continue;
        }
        let sp = line.rfind(' ');
        let (w, freq) = match sp {
            Some(sp) => (&line[..sp], line[sp + 1..].trim().parse::<u64>()),
            None => continue,
        };
        let freq = match freq {
            Ok(f) => f,
            Err(_) => continue,
        };
        if w.len() < 2
            || w.len() > ECDICT_MAX_WORD_LEN
            || !w.bytes().all(|b| b.is_ascii_lowercase() || b == b'\'')
        {
            continue;
        }
        rows.push((w.to_owned(), freq));
    }
    // 频次降序；同频按词形字典序稳定。
    rows.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    rows.truncate(top);
    Ok(rows)
}

/// 读 CC-CEDICT 原文，提取英文侧纯单词词目（与第五期口径一致：
/// 第一条译文、无括号注释、无空格、纯字母、长度 2..=32）。
/// `present` 记录已占用的 norm（ECDICT 已收录的不再补入），返回新增项。
fn load_cedict_english(
    path: &Path,
    present: &HashSet<String>,
) -> Result<Vec<(String, String)>, String> {
    let file =
        File::open(path).map_err(|e| format!("打开 CC-CEDICT 失败（{}）：{e}", path.display()))?;
    let reader = BufReader::new(file);
    let mut out: Vec<(String, String)> = Vec::new();
    let mut seen = present.clone();
    for (line_no, line) in reader.lines().enumerate() {
        let line = line.map_err(|e| format!("读 CC-CEDICT 第 {} 行失败：{e}", line_no + 1))?;
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let gloss = line
            .find('/')
            .map(|open| &line[open + 1..])
            .unwrap_or_default();
        let gloss = gloss.trim_end_matches('/');
        let gloss = gloss.split(';').next().unwrap_or_default();
        if gloss.contains('(') || gloss.contains(' ') || gloss.is_empty() {
            continue;
        }
        if !gloss.bytes().all(|b| b.is_ascii_alphabetic()) {
            continue;
        }
        if gloss.len() < 2 || gloss.len() > 32 {
            continue;
        }
        if !seen.insert(gloss.to_ascii_lowercase()) {
            continue;
        }
        out.push((gloss.to_ascii_lowercase(), gloss.to_owned()));
    }
    Ok(out)
}

/// 读大小写补丁表：`norm<TAB>word[<TAB>rank]`（rank 为 1 基目标位）。
fn load_capitals_patch(path: &Path) -> Result<Vec<(String, String, Option<u32>)>, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("读大小写补丁失败（{}）：{e}", path.display()))?;
    let mut out = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() < 2 {
            return Err(format!("大小写补丁行格式错误：{line}"));
        }
        let norm = cols[0].to_ascii_lowercase();
        if norm != cols[0] {
            return Err(format!("补丁小写键必须全小写：{line}"));
        }
        let rank = if cols.len() >= 3 && !cols[2].trim().is_empty() {
            let r = cols[2]
                .trim()
                .parse::<u32>()
                .map_err(|_| format!("补丁排名必须为整数：{line}"))?;
            if r == 0 {
                return Err(format!("补丁排名必须 ≥1（1 为最常用）：{line}"));
            }
            Some(r)
        } else {
            None
        };
        out.push((norm, cols[1].to_owned(), rank));
    }
    Ok(out)
}

/// 读排除清单：每行一个或多个 tab 分隔的 norm（小写化比较）。
fn load_exclude_list(path: &Path) -> Result<Vec<String>, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("读排除清单失败（{}）：{e}", path.display()))?;
    let mut out = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        for item in line.split('\t') {
            let norm = item.trim().to_ascii_lowercase();
            if !norm.is_empty() {
                out.push(norm);
            }
        }
    }
    Ok(out)
}

/// 执行完整构建：清洗 → 合并 → 补丁 → 排序 → 编译 → 写盘。
///
/// 返回统计；`out` 为 ZYEN v1 文件目标路径（父目录必须存在）。
pub fn build_en_wordbook(
    inputs: &EnWordbookInputs<'_>,
    out: &Path,
) -> Result<EnWordbookStats, String> {
    let mut stats = EnWordbookStats::default();
    let mut entries = load_ecdict(inputs.ecdict_csv, &mut stats)?;

    // ---- 排除清单（先剔除 ECDICT 段，并阻止后续段补回） ----
    let exclude: HashSet<String> = match &inputs.exclude {
        Some(path) => load_exclude_list(path)?.into_iter().collect(),
        None => HashSet::new(),
    };
    let before_exclude = entries.len();
    entries.retain(|e| !exclude.contains(&e.norm));
    stats.excluded = (before_exclude - entries.len()) as u64;

    // ---- FrequencyWords top10000 补入（ECDICT 未收录且未被排除） ----
    let mut present: HashSet<String> = entries.iter().map(|e| e.norm.clone()).collect();
    if let Some(path) = inputs.freqwords {
        for (norm, freq) in load_freqwords_top(path, FREQ_WORDS_TOP)? {
            if !exclude.contains(&norm) && present.insert(norm.clone()) {
                entries.push(Entry {
                    word: norm.clone(),
                    norm,
                    // 以词频数占位参与排序；无 bnc。
                    frq: freq,
                    bnc: 0,
                    rank_override: None,
                });
                stats.freqwords_added += 1;
            }
        }
    }

    // ---- CC-CEDICT 英文侧补入（frq=0，排在词频条目之后） ----
    if let Some(path) = inputs.cedict {
        for (norm, word) in load_cedict_english(path, &present)? {
            if !exclude.contains(&norm) {
                entries.push(Entry {
                    word,
                    norm,
                    frq: 0,
                    bnc: 0,
                    rank_override: None,
                });
                stats.cedict_added += 1;
            }
        }
    }

    // ---- 大小写补丁：覆盖原形 / 强推 rank ----
    if let Some(path) = inputs.capitals {
        for (norm, word, rank) in load_capitals_patch(path)? {
            if let Some(entry) = entries.iter_mut().find(|e| e.norm == norm) {
                entry.word = word.clone();
                entry.rank_override = rank;
                stats.capitals_patched += 1;
            } else if !exclude.contains(&norm) {
                // 补丁可新增词条（低频强推）。
                entries.push(Entry {
                    norm,
                    word,
                    frq: 0,
                    bnc: 0,
                    rank_override: rank,
                });
                stats.capitals_patched += 1;
            }
        }
    }

    // ---- 排序：frq>0 组优先 → frq 降序 → bnc 降序 → norm 字典序（确定性） ----
    entries.sort_by(|a, b| {
        let ka = (a.frq > 0, a.frq, a.bnc);
        let kb = (b.frq > 0, b.frq, b.bnc);
        kb.cmp(&ka).then_with(|| a.norm.cmp(&b.norm))
    });
    apply_rank_overrides(&mut entries)?;

    // ---- 编译（ZYEN v1）：rank = 最终顺序下标；存储按 norm 升序 ----
    let mut records: Vec<EnRecord> = Vec::with_capacity(entries.len());
    for (rank, entry) in entries.iter().enumerate() {
        if !entry
            .norm
            .bytes()
            .all(|b| b.is_ascii() && !b.is_ascii_uppercase())
        {
            return Err(format!("构建防御：norm 含大写或非 ASCII：{}", entry.norm));
        }
        if entry.word != entry.norm {
            stats.capitalized_words += 1;
        }
        records.push(EnRecord {
            norm: &entry.norm,
            word: &entry.word,
            rank: rank as u32,
        });
    }
    // 存储序与展示序解耦：按 norm 升序摆放（compile 断言），rank 随记录携带。
    records.sort_by(|a, b| a.norm.as_bytes().cmp(b.norm.as_bytes()));
    let produced = compile_en_records(&records);
    // 干净检出下 data/artifacts 目录可能不存在（git 不跟踪空目录），先建父目录（CI 试跑 T-095 暴露）。
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("创建输出目录失败（{}）：{e}", parent.display()))?;
    }
    std::fs::write(out, &produced)
        .map_err(|e| format!("写 en 词表失败（{}）：{e}", out.display()))?;

    stats.final_count = records.len() as u64;
    stats.produced_bytes = produced.len() as u64;
    Ok(stats)
}

/// 应用 rank 补丁：把 `rank_override = Some(r)` 的记录移到目标位
/// （1 基，r=1 即下标 0），其余记录保持原相对顺序填充空位。
fn apply_rank_overrides(entries: &mut Vec<Entry>) -> Result<(), String> {
    let overrides: Vec<(usize, u32)> = entries
        .iter()
        .enumerate()
        .filter_map(|(src, e)| e.rank_override.map(|r| (src, r)))
        .collect();
    if overrides.is_empty() {
        return Ok(());
    }
    let n = entries.len();
    // 目标位冲突检查（多个词条指向同一位 → 报错）。
    let mut by_target: HashMap<u32, usize> = HashMap::new();
    for (src, rank) in &overrides {
        let target = *rank as usize;
        if target == 0 || target > n {
            return Err(format!(
                "补丁 rank 越界：{target} 超出 1..={n}（词条 {src}）"
            ));
        }
        if let Some(prev) = by_target.insert(*rank, *src) {
            return Err(format!(
                "补丁 rank 冲突：多个词条指向 {rank}（源 {prev} 与 {src}）"
            ));
        }
    }
    // 搬移：先占位目标位，再把其余记录按原序填入空位。
    let mut slots: Vec<Option<Entry>> = entries.drain(..).map(Some).collect();
    let mut result: Vec<Option<Entry>> = (0..n).map(|_| None).collect();
    for (src, rank) in &overrides {
        let entry = slots[*src]
            .take()
            .ok_or_else(|| "补丁应用内部错误：源已被取走".to_owned())?;
        result[*rank as usize - 1] = Some(entry);
    }
    let mut fill = slots.into_iter().flatten();
    for slot in &mut result {
        if slot.is_none() {
            *slot = fill.next();
        }
    }
    debug_assert!(fill.next().is_none(), "补丁应用内部错误：残留未填充项");
    let mut out: Vec<Entry> = Vec::with_capacity(n);
    for slot in result {
        out.push(slot.ok_or_else(|| "补丁应用内部错误：位置未填充".to_owned())?);
    }
    *entries = out;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn temp_dir() -> std::path::PathBuf {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("zhu-ye-dict-en-{}-{now}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_file(dir: &std::path::Path, name: &str, content: &str) -> std::path::PathBuf {
        let path = dir.join(name);
        let mut f = File::create(&path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
        path
    }

    const ECDICT_HEADER: &str =
        "word,phonetic,definition,translation,pos,collins,oxford,tag,bnc,frq,exchange,detail,audio";

    #[test]
    fn csv解析引号逗号与转义() {
        let row = r#"python,phonetic,"a language, ""famous"" for snakes",翻译,n,""#;
        let cols = parse_csv_row(row);
        assert_eq!(cols[0], "python");
        assert_eq!(cols[1], "phonetic");
        assert_eq!(cols[2], "a language, \"famous\" for snakes");
        assert_eq!(cols[3], "翻译");
        assert_eq!(cols[4], "n");
        // 常规行（13 列）也应完整解析。
        let cols = parse_csv_row("python,phon,def,trans,n,0,0,,58,9,,,");
        assert_eq!(cols.len(), 13);
        assert_eq!(cols[0], "python");
        assert_eq!(cols[8], "58");
        assert_eq!(cols[9], "9");
    }

    #[test]
    fn 词形清洗规则() {
        assert!(word_acceptable("python"));
        assert!(word_acceptable("iPhone"));
        assert!(word_acceptable("good morning"));
        assert!(word_acceptable("merry-go-round"));
        assert!(word_acceptable("'a"));
        assert!(!word_acceptable(""));
        assert!(!word_acceptable("a")); // 长度 1
        assert!(!word_acceptable("中文字")); // 非 ASCII
        assert!(!word_acceptable("12345")); // 纯数字
        assert!(!word_acceptable("hello世界")); // 混合
        assert!(!word_acceptable("3d")); // 字母数字混合
        assert!(!word_acceptable(
            "madrigal: an addition to the text of a poem" // 污染条目（>40）
        ));
        assert!(!word_acceptable("-")); // 纯符号
    }

    #[test]
    fn 构建查询闭环() {
        let dir = temp_dir();
        let csv = write_file(
            &dir,
            "ecdict.csv",
            &format!(
                "{ECDICT_HEADER}\n\
                 python,phon,def,trans,n,0,0,,58,9,,,\n\
                 API,phon,def,trans,n,0,0,,42,7,,,\n\
                 apple,phon,def,trans,n,0,0,,30,8,,,\n\
                 iPhone,phon,def,trans,n,0,0,,12,6,,,\n\
                 good morning,phon,def,trans,n,0,0,,5,3,,,\n\
                 中文污染,phon,def,trans,n,0,0,,1,1,,,\n\
                 zzzzzzzzzzzzzzzzzzz9,phon,def,trans,n,0,0,,1,1,,,\n\
                 api,phon,def,trans,n,0,0,,2,4,,,\n"
            ),
        );
        let out = dir.join("en.zyen");
        let stats = build_en_wordbook(
            &EnWordbookInputs {
                ecdict_csv: &csv,
                freqwords: None,
                cedict: None,
                capitals: None,
                exclude: None,
            },
            &out,
        )
        .expect("构建成功");
        assert_eq!(stats.ecdict_rows, 8);
        assert_eq!(stats.ecdict_kept, 6); // 中文污染 + 含数字超长串拒绝
        assert_eq!(stats.final_count, 5); // API/api 同键去重 → 5 唯一键

        let lex = zhu_ye_core::en_lexicon::EnLexicon::open(&out).expect("加载");
        let hit = |p: &str| {
            lex.words_with_prefix(p, 8)
                .into_iter()
                .map(|(w, r)| (w.to_owned(), r))
                .collect::<Vec<_>>()
        };
        assert_eq!(hit("pytho"), vec![("python".to_owned(), 0)]);
        assert_eq!(hit("iphone"), vec![("iPhone".to_owned(), 3)]); // 原形大小写保留（D-09）
        assert_eq!(hit("api"), vec![("API".to_owned(), 2)]); // frq 7 > 4 → API 胜出
                                                             // "ap" 前缀：apple(frq8→r1)、API(frq7→r2)，按 rank 升序。
        assert_eq!(
            hit("ap"),
            vec![("apple".to_owned(), 1), ("API".to_owned(), 2)]
        );
        assert_eq!(hit("good"), vec![("good morning".to_owned(), 4)]);
        assert!(lex.words_with_prefix("zzzz", 8).is_empty());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn 合并补丁与排除() {
        let dir = temp_dir();
        let csv = write_file(
            &dir,
            "ecdict.csv",
            &format!(
                "{ECDICT_HEADER}\n\
                 python,phon,def,trans,n,0,0,,58,9,,,\n\
                 iPhone,phon,def,trans,n,0,0,,12,6,,,\n"
            ),
        );
        let freqwords = write_file(
            &dir,
            "freq.txt",
            "the 100000\nand 90000\nwanna 500\n", // the/wanna 未收录 → 补入；and 被排除清单剔除
        );
        let cedict = write_file(
            &dir,
            "cedict.txt",
            "# comment\nhello 你好 /hello/\npython 蟒蛇 /python/\n", // hello 补入；python 已在
        );
        let capitals = write_file(
            &dir,
            "capitals.tsv",
            "python\tPython\t1\n", // 强推 rank=1（下标 0）
        );
        let exclude = write_file(&dir, "exclude.tsv", "and\n");
        let out = dir.join("en.zyen");
        let stats = build_en_wordbook(
            &EnWordbookInputs {
                ecdict_csv: &csv,
                freqwords: Some(&freqwords),
                cedict: Some(&cedict),
                capitals: Some(&capitals),
                exclude: Some(&exclude),
            },
            &out,
        )
        .expect("构建成功");
        assert_eq!(stats.freqwords_added, 2); // the、wanna（and 被排除清单阻止补入）
        assert_eq!(stats.cedict_added, 1); // hello
        assert_eq!(stats.excluded, 0); // ECDICT 段本身不含 and，无剔除；排除仅阻止补入
        assert_eq!(stats.final_count, 5); // python/iPhone + the + wanna + hello

        let lex = zhu_ye_core::en_lexicon::EnLexicon::open(&out).expect("加载");
        // python 原形被补丁改为 Python 且 rank=0（最常用）。
        let hits = lex.words_with_prefix("pyth", 8);
        assert_eq!(hits[0].0, "Python");
        assert_eq!(hits[0].1, 0);
        // the（frq 100000）次之。
        let the_hits = lex.words_with_prefix("the", 8);
        assert_eq!(the_hits[0].0, "the");
        assert_eq!(the_hits[0].1, 1);
        assert_eq!(lex.words_with_prefix("wan", 8)[0].0, "wanna");
        assert_eq!(lex.words_with_prefix("hel", 8)[0].0, "hello");
        assert!(lex.words_with_prefix("and", 8).is_empty()); // 已排除
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn rank补丁冲突报错() {
        let mut entries = vec![
            Entry {
                norm: "a".to_owned(),
                word: "a".to_owned(),
                frq: 1,
                bnc: 0,
                rank_override: None,
            },
            Entry {
                norm: "b".to_owned(),
                word: "b".to_owned(),
                frq: 1,
                bnc: 0,
                rank_override: Some(1),
            },
            Entry {
                norm: "c".to_owned(),
                word: "c".to_owned(),
                frq: 1,
                bnc: 0,
                rank_override: Some(1), // 与 b 冲突
            },
        ];
        assert!(apply_rank_overrides(&mut entries).is_err());
    }
}
