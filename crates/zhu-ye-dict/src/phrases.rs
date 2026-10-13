//! 统计驱动日常短语扩充（T-147）：语料 2/3-gram 计数 → 融合频率表。
//!
//! 背景：CEDICT/jieba/wordfreq 均以单字/整词收录，"你是/我有/我在"这类
//! 跨词邻接短语没有现成词表收录（T-145 已证），必须由语料统计补齐。
//!
//! 数据来源（许可细节见 `docs/licenses.md`）：
//! - OpenSubtitles zh 字幕单语（口语对话，许可不明 → 仅内部聚合统计、不重新分发原文）；
//! - 中文维基百科 dump（书面，CC BY-SA 4.0）；
//! - SUBTLEX-CH 整词频率表（影视字幕，学术署名）与《现代汉语常用词表（草案）》
//!   （国家语委公开文件）作为现成整词频率补充（第二阶段并入）。
//!
//! 频率语义：产出 zipf×1000 兼容的分值（区间 [`PHRASE_FREQ_MIN`..=
//! [`PHRASE_FREQ_MAX`]），供 `build_base` 统计驱动覆盖层使用——高于 T-145
//! 枚举/种子档位（4000/4200）、低于绝对高频单字（是 7160、你 6690），
//! 保证单字组排序不受扰动。

use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

use bzip2::read::BzDecoder;
use flate2::read::GzDecoder;

/// 短语频率分下界（ppm≈1 的对数映射）。
pub const PHRASE_FREQ_MIN: u32 = 3800;
/// 短语频率分上界（ppm≈1e4 的对数映射）。
pub const PHRASE_FREQ_MAX: u32 = 6300;

/// 2-gram 原始计数门槛（过滤句级一次性拼写噪音）。
pub const BIGRAM_MIN_COUNT: u64 = 20;
/// 3-gram 原始计数门槛（更长组合更稀少，放低）。
pub const TRIGRAM_MIN_COUNT: u64 = 10;

/// 摘要输出表：`词\t频率分`（注音由 build_base 统一做，本表不存拼音）。
pub const PHRASES_FREQ_OUT: &str = "data/artifacts/phrases-freq.tsv";

/// 语料缓存路径（与 `scripts/fetch-sources.ps1` 的 pin 表一致）。
pub const OPENSUBTITLES_GZ: &str = "data/cache/opensubtitles-zh-cn.txt.gz";
pub const ZHWIKI_BZ2: &str = "data/cache/zhwiki-pages-articles.xml.bz2";

/// 语料来源权重：OpenSubtitles 口语权重高于维基书面语（T-147 权重 2:1，
/// 口语字序更贴近日常输入；实测 1:1 时"下半/他在"等书面高频词会压过
/// "下班/她在"等口语词）。
pub const WEIGHT_SUBS: f64 = 2.0;
pub const WEIGHT_WIKI: f64 = 1.0;

/// 残段降档阈值（T-147）：n-gram 作为更长 n+1-gram 前缀的延伸占比超过该值，
/// 视为长词内部残段（如「己的」≈「自己的」）而非独立短语，频率分 ×[`EXT_WEIGHT_FACTOR`]。
/// 只压住明显残段、不动真实短语——「你是」延伸占比虽高（你是谁/你是我的），
/// 但它是 T-145 已有词，由 m6 覆盖语义保证按统计分进库不受影响。
pub const EXT_RATIO: f64 = 0.6;
/// 残段降档系数（详见 [`EXT_RATIO`]）。
pub const EXT_WEIGHT_FACTOR: f64 = 0.9;

/// 维基采样间隔（T-147 性能口径）：统计时每 N 个页面取 1 个（`page_idx % N == 0`），
/// 快照文件固定 → 样本固定 → 结果可复现。1/3 采样把 3.2GB dump 处理时间降到
/// 全量的约 1/3，频率误差远小于两语料间固有差异。
pub const WIKI_PAGE_SAMPLE_EVERY: u64 = 3;

/// `build_phrases` 结果统计。
#[derive(Debug, Default, Clone)]
pub struct PhraseStats {
    /// 字幕语料中 CJK 字符总数。
    pub subs_chars: u64,
    /// 维基语料中文本行数。
    pub wiki_rows: u64,
    /// 维基语料中 CJK 字符总数。
    pub wiki_chars: u64,
    /// 参与融合的语料源个数（0/1/2）。
    pub corpora: usize,
    /// 最终入表短语数。
    pub phrases: usize,
}

/// 是否是 CJK 统一表意文字（基础区 + 扩展 A + 扩展 B）。
///
/// T-147 采用宽松字桶（Q3a）：计数与读表都以本判定为准——语料里出现的
/// 生僻字（如扩展 A「䲁」）组成的 2-3-gram 允许进频率表；入 base 时
/// `merge_statistical_phrases` 仍按严格的 `import::is_cjk_word` 过滤，双层防御。
pub fn is_cjk_char(c: char) -> bool {
    matches!(c as u32, 0x4E00..=0x9FFF | 0x3400..=0x4DBF | 0x20000..=0x2A6DF)
}

/// 对一行文本做 CJK 连续段滑窗 2..=5-gram 计数。
///
/// `counts[i]` 存 (i+2)-gram（counts[0] 为 2-gram，…，counts[3] 为 5-gram）；
/// `max_n` 限制本语料计到的最大窗口（字幕计 2-5，维基计 2-4 以控成本）。
fn count_line(line: &str, counts: &mut [HashMap<String, u64>; 4], max_n: usize) {
    let mut seg = String::new();
    for c in line.chars() {
        if is_cjk_char(c) {
            seg.push(c);
        } else {
            count_segment(&seg, counts, max_n);
            seg.clear();
        }
    }
    count_segment(&seg, counts, max_n);
}

/// 对一段连续 CJK 字符做滑窗计数：取所有相邻 2..=max_n 子串（不足长度自然不计）。
fn count_segment(seg: &str, counts: &mut [HashMap<String, u64>; 4], max_n: usize) {
    let chars: Vec<char> = seg.chars().collect();
    for n in 2..=max_n.min(5) {
        for w in chars.windows(n) {
            let s: String = w.iter().collect();
            *counts[n - 2].entry(s).or_insert(0) += 1;
        }
    }
}

/// 流式计数一个 gz 语料（一行一句文本）。
fn count_gz_text(
    path: &Path,
    counts: &mut [HashMap<String, u64>; 4],
    max_n: usize,
    chars: &mut u64,
) -> Result<(), String> {
    let file = fs::File::open(path)
        .map_err(|error| format!("打开语料 {} 失败: {error}", path.display()))?;
    let decoder = GzDecoder::new(file);
    let reader = BufReader::new(decoder);
    for line in reader.lines() {
        let line = line.map_err(|error| format!("读取语料 {} 失败: {error}", path.display()))?;
        for c in line.chars() {
            if is_cjk_char(c) {
                *chars += 1;
            }
        }
        count_line(&line, counts, max_n);
    }
    Ok(())
}

/// 流式计数维基 bz2 dump：提取每页 `<text …>…</text>` 内的文本行。
///
/// 按页采样（[`WIKI_PAGE_SAMPLE_EVERY`] 取 1）控制处理成本：快照文件固定，
/// 同样本固定、结果可复现。
fn count_wiki_bz2(
    path: &Path,
    counts: &mut [HashMap<String, u64>; 4],
    max_n: usize,
    chars: &mut u64,
    rows: &mut u64,
) -> Result<(), String> {
    let file = fs::File::open(path)
        .map_err(|error| format!("打开维基语料 {} 失败: {error}", path.display()))?;
    let decoder = BzDecoder::new(file);
    let reader = BufReader::new(decoder);
    let mut in_text = false;
    let mut page_idx: u64 = 0;
    let mut sampled = false;
    for line in reader.lines() {
        let line =
            line.map_err(|error| format!("读取维基语料 {} 失败: {error}", path.display()))?;
        if line.contains("<page") {
            page_idx += 1;
            sampled = page_idx.is_multiple_of(WIKI_PAGE_SAMPLE_EVERY);
        }
        if line.contains("<text") {
            in_text = true;
        }
        if !in_text || !sampled {
            continue;
        }
        *rows += 1;
        // 文本内可能含维基标记（[[|]]、{{}}），CJK 段提取天然忽略之。
        for c in line.chars() {
            if is_cjk_char(c) {
                *chars += 1;
            }
        }
        count_line(&line, counts, max_n);
        if line.contains("</text>") {
            in_text = false;
        }
    }
    Ok(())
}

/// 每百万字频率。
fn ppm(count: u64, total_chars: u64) -> f64 {
    if total_chars == 0 {
        return 0.0;
    }
    count as f64 / total_chars as f64 * 1_000_000.0
}

/// 每百万频 → zipf 兼容频率分：`3800 + 2500·log10(ppm)/4`，ppm∈[1, 1e4]
/// 映射到 [MIN, MAX]；ppm<1 的自然得分低于 [`PHRASE_FREQ_MIN`]，由入表门槛
/// 丢弃（低置信 2/3-gram 不会进表），ppm>1e4 封顶 [`PHRASE_FREQ_MAX`]。
fn score(ppm: f64) -> u32 {
    let v = PHRASE_FREQ_MIN as f64
        + (PHRASE_FREQ_MAX as f64 - PHRASE_FREQ_MIN as f64)
            * (ppm.max(f64::MIN_POSITIVE).log10() / 4.0);
    v.round().min(PHRASE_FREQ_MAX as f64) as u32
}

/// 构建统计短语频率表（`data/artifacts/phrases-freq.tsv`）。
///
/// 语料缺失时按可用语料降级；两个语料都缺失则报错（提示先跑 fetch）。
pub fn build_phrases(root: &Path) -> Result<PhraseStats, String> {
    let cache = root.join("data/cache");
    let subs_path = cache.join("opensubtitles-zh-cn.txt.gz");
    let wiki_path = cache.join("zhwiki-pages-articles.xml.bz2");
    if !subs_path.exists() && !wiki_path.exists() {
        return Err("缺少语料（opensubtitles-zh-cn.txt.gz / zhwiki-pages-articles.xml.bz2），请先执行 scripts/fetch-sources.ps1".to_owned());
    }

    let mut stats = PhraseStats::default();

    // 源 → (ppm, 权重, 是否残段降档)；仅统计存在的语料。
    let mut sources: HashMap<String, Vec<(f64, f64, bool)>> = HashMap::new();
    if subs_path.exists() {
        // 每源独立计数表（下标 i 存 (i+2)-gram），避免共享表把字幕计数
        // 混进维基 ppm 分子。
        let mut counts = std::array::from_fn(|_| HashMap::new());
        let mut total = 0;
        count_gz_text(&subs_path, &mut counts, 5, &mut total)?;
        trim_low_confidence(&mut counts, total);
        stats.subs_chars = total;
        absorb_counts(&mut sources, &counts, total, WEIGHT_SUBS);
        stats.corpora += 1;
    }
    if wiki_path.exists() {
        let mut counts = std::array::from_fn(|_| HashMap::new());
        let mut total = 0;
        let mut rows = 0;
        count_wiki_bz2(&wiki_path, &mut counts, 4, &mut total, &mut rows)?;
        trim_low_confidence(&mut counts, total);
        stats.wiki_chars = total;
        stats.wiki_rows = rows;
        absorb_counts(&mut sources, &counts, total, WEIGHT_WIKI);
        stats.corpora += 1;
    }

    // 融合：多源加权平均 ppm → 频率分；低于门槛组合丢弃；任一源判定为残段
    // 则该短语整体 ×EXT_WEIGHT_FACTOR（残段在哪个语料都是残段）。
    // 表值三元组 (词, 降档后分, 降档前分)：降档前分供 m6 侧"官方先验词
    // 豁免"恢复（如「时间」因「的时间」自然搭配被降档，官方词表确认其为
    // 真实常用词，应恢复原始分）；未降档时两分相同。
    let mut merged: Vec<(String, u32, u32)> = Vec::new();
    for (word, list) in &sources {
        let weighted: f64 = list.iter().map(|(p, w, _)| p * w).sum();
        let weight_sum: f64 = list.iter().map(|(_, w, _)| w).sum();
        if weight_sum <= 0.0 {
            continue;
        }
        let ppm = weighted / weight_sum;
        let raw = score(ppm);
        if raw < PHRASE_FREQ_MIN {
            continue;
        }
        let mut s = raw;
        if list.iter().any(|(_, _, degraded)| *degraded) {
            s = (s as f64 * EXT_WEIGHT_FACTOR).round() as u32;
        }
        merged.push((word.clone(), s, raw));
    }
    merged.sort();
    merged.dedup_by(|a, b| a.0 == b.0);

    // 按分数降序写表（同分按词序），保证可复现输出。降档过的一行输出
    // 第 3 列降档前分（m6 官方词豁免用）；未降档只写两列与旧格式兼容。
    merged.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    let out = root.join(PHRASES_FREQ_OUT);
    let mut buf = String::with_capacity(merged.len() * 20);
    for (word, s, raw) in &merged {
        buf.push_str(word);
        buf.push('\t');
        buf.push_str(&s.to_string());
        if *s != *raw {
            buf.push('\t');
            buf.push_str(&raw.to_string());
        }
        buf.push('\n');
    }
    fs::write(&out, buf).map_err(|error| format!("写短语频率表失败: {error}"))?;
    stats.phrases = merged.len();
    Ok(stats)
}

/// 无损裁剪低置信计数（T-147 性能修复）：
/// `score(ppm) >= PHRASE_FREQ_MIN ⟺ ppm >= 1.0 ⟺ count >= total/1e6`。
/// ppm<1 的 n-gram 融合后分数必然不足 3800（残段降档只会更低），保留它们
/// 只会放大 absorb 的插入与延伸遍历成本——4/5-gram 表 90%+ 是 count 1-2 的
/// 冷键，全量 absorb 曾导致单轮统计超过 3 小时仍未完成。裁剪对产物零影响。
fn trim_low_confidence(counts: &mut [HashMap<String, u64>; 4], total_chars: u64) {
    let floor = (total_chars as f64 / 1_000_000.0).ceil() as u64;
    for table in counts {
        table.retain(|_, count| *count >= floor);
    }
}

/// 把一份 2..=5-gram 计数按 ppm 吸收进融合表（字长 2..=5）。
///
/// 残段判定（T-147）：n-gram 若在更长 (n+1)-gram 的计数中作为前缀的延伸
/// 占比 > [`EXT_RATIO`]，标记为长词内部残段（如「己的」≈「自己的」延伸）。
/// 判定仅在同一份语料的计数内进行（两源同词标记一致）。
fn absorb_counts(
    sources: &mut HashMap<String, Vec<(f64, f64, bool)>>,
    counts: &[HashMap<String, u64>; 4],
    total_chars: u64,
    weight: f64,
) {
    // 前后向延伸总量：pre_sum[n][w] = Σ count((n+1)-gram x) where w 是 x 的
    // 前 n 字；suf_sum[n][w] = Σ count(x) where w 是 x 的后 n 字。下标对齐
    // counts 下标（word 为 n-gram，n = 下标+2）。「己的」是「自己的」后缀残段、
    // 「什么意」是「什么意思」前缀残段——两个方向都查。
    let mut pre_sum: [HashMap<String, u64>; 3] = std::array::from_fn(|_| HashMap::new());
    let mut suf_sum: [HashMap<String, u64>; 3] = std::array::from_fn(|_| HashMap::new());
    for n in 0..3 {
        let window = n + 2; // word 窗口 = (n+2) 字；gram 是 (n+3) 字
        for (gram, count) in &counts[n + 1] {
            let chars: Vec<char> = gram.chars().collect();
            let pre: String = chars.iter().take(window).collect();
            *pre_sum[n].entry(pre).or_insert(0) += count;
            let suf: String = chars[chars.len() - window..].iter().collect();
            *suf_sum[n].entry(suf).or_insert(0) += count;
        }
    }
    for (index, gram_counts) in counts.iter().enumerate() {
        let n = index + 2; // 2..=5
        let pre = pre_sum.get(index); // 5-gram 无更高一级延伸表
        let suf = suf_sum.get(index);
        for (word, count) in gram_counts {
            if word.chars().count() != n {
                continue;
            }
            let degraded = [pre, suf].iter().any(|ext| {
                ext.is_some_and(|m| {
                    m.get(word)
                        .is_some_and(|ext_count| *ext_count as f64 / *count as f64 > EXT_RATIO)
                })
            });
            sources.entry(word.clone()).or_default().push((
                ppm(*count, total_chars),
                weight,
                degraded,
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{absorb_counts, count_line, score, trim_low_confidence, PHRASE_FREQ_MIN};
    use std::collections::HashMap;

    #[test]
    fn 行滑窗计数_相邻二字短语入表_跨标点不粘连() {
        let mut counts = std::array::from_fn(|_| HashMap::new());
        count_line("你是我的。你是谁？", &mut counts, 3);
        // 两个句子各含一次"你是"
        assert_eq!(counts[0].get("你是"), Some(&2));
        assert_eq!(counts[0].get("我的"), Some(&1));
        assert!(counts[0].contains_key("是谁"));
        // 标点隔开不成对
        assert_eq!(counts[0].get("的谁"), None);
        // 3-gram：你是我 / 是我的
        assert_eq!(counts[1].get("你是我"), Some(&1));
        assert_eq!(counts[1].get("是我的"), Some(&1));
        // max_n=3 时不计 4-gram
        assert_eq!(counts[2].get("你是我的"), None);
    }

    #[test]
    fn 行滑窗_非中文段与短段不计() {
        let mut counts = std::array::from_fn(|_| HashMap::new());
        count_line("abc123 你好 xyz", &mut counts, 5);
        assert_eq!(counts[0].get("你好"), Some(&1));
        assert_eq!(counts[0].len(), 1, "只应有 你好 一个 2-gram");
    }

    #[test]
    fn 行滑窗_四字五字短语入表() {
        // 4/5-gram 计数在 max_n 内正常产生（T-147 修复：我不知道/你喜欢我吗
        // 这类整词此前缺 4-gram 计数完全不入表）。
        let mut counts = std::array::from_fn(|_| HashMap::new());
        count_line("我不知道。我喜欢你吗。", &mut counts, 5);
        assert_eq!(counts[2].get("我不知道"), Some(&1));
        assert_eq!(counts[2].get("我喜欢你"), Some(&1));
        assert_eq!(counts[3].get("我喜欢你吗"), Some(&1));
        // 标点截断：跨句不粘连
        assert_eq!(counts[2].get("道我喜"), None);
    }

    #[test]
    fn 残段判定_延伸占比高者标记降档() {
        // 源计数：2-gram「己的」出现 10 次，其中「自己的」3-gram 出现 8 次
        // （8/10 > EXT_RATIO）→ 残段；「记得」出现 10 次、延伸「记得你」仅 1 次
        // → 非残段。
        let mut counts: [HashMap<String, u64>; 4] = std::array::from_fn(|_| HashMap::new());
        counts[0].insert("己的".to_owned(), 10);
        counts[0].insert("记得".to_owned(), 10);
        counts[1].insert("自己的".to_owned(), 8);
        counts[1].insert("记得你".to_owned(), 1);
        let mut sources: HashMap<String, Vec<(f64, f64, bool)>> = HashMap::new();
        absorb_counts(&mut sources, &counts, 1000, 1.0);
        let degraded = |word: &str| sources[word].iter().any(|(_, _, d)| *d);
        assert!(degraded("己的"), "己的 应是残段（自己的 延伸占比高）");
        assert!(!degraded("记得"), "记得 不应是残段");
    }

    #[test]
    fn 频率分_泊松区域单调且低置信出表() {
        // ppm=1 → 3800；ppm=1e4 → 6300；ppm<1 低于门槛（入表时被拒）。
        assert_eq!(score(1.0), PHRASE_FREQ_MIN);
        assert_eq!(score(10_000.0), 6300);
        assert!(score(0.1) < PHRASE_FREQ_MIN);
        assert!(score(100.0) > score(10.0));
    }

    #[test]
    fn 融合_重复行累计计数() {
        // 内部函数契约：同一短语多次出现累计；滑窗跨标点不粘连。
        let mut counts = std::array::from_fn(|_| HashMap::new());
        count_line("你是不是知道了，你是不是知道了", &mut counts, 4);
        assert_eq!(counts[0].get("你是"), Some(&2));
        // 3-gram：你+是+不（"你是不是"为 4 字，首三元组是"你是不"）
        assert_eq!(counts[1].get("你是不"), Some(&2));
        assert_eq!(counts[1].get("知道了"), Some(&2));
        // 4-gram：你是不是
        assert_eq!(counts[2].get("你是不是"), Some(&2));
    }

    #[test]
    fn 低置信裁剪_ppm不足1的冷键出表_保底值无损() {
        // total=5,000,000 → floor=5：count<5 的 n-gram ppm<1，融合后分数必然
        // 不足 3800，扔掉不影响产物；count≥5 全部保留。
        let mut counts = std::array::from_fn(|_| HashMap::new());
        counts[2].insert("我不知道".to_owned(), 9);
        counts[3].insert("我喜欢你吗".to_owned(), 5);
        counts[0].insert("己的".to_owned(), 4);
        counts[0].insert("记得".to_owned(), 100);
        trim_low_confidence(&mut counts, 5_000_000);
        assert!(counts[2].contains_key("我不知道"));
        assert!(counts[3].contains_key("我喜欢你吗"));
        assert!(
            !counts[0].contains_key("己的"),
            "count=4 < floor=5 应被裁剪"
        );
        assert!(counts[0].contains_key("记得"));
    }
}
