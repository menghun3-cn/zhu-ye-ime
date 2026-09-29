//! M6 多词包管线：`source-check` / `build-pack` / `build-manifest` / `verify-manifest`。
//!
//! 数据源以 `data/pins/*.json` 为唯一权威（与 `scripts/fetch-sources.ps1` 相同
//! 约定），本模块在构建期校验缓存哈希、为领域词包注音（词级 CC-CEDICT 优先、
//! 单字级 Unihan kTGHZ2013 兜底）、编译 v2 词典包并生成/复核 manifest。
//! 许可与来源登记见 `docs/数据清单.md` 与 `docs/licenses.md`。

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use zhu_ye_core::dict::DictionaryEntry;
use zhu_ye_core::pinyin::SyllableTable;

use crate::import::{is_cjk_word, parse_cedict_line, split_pinyin_syllables};

/// 仓库相对目录常量（与 `scripts/fetch-sources.ps1` 保持一致）。
pub const PINS_DIR: &str = "data/pins";
pub const CACHE_DIR: &str = "data/cache";
pub const ARTIFACTS_DIR: &str = "data/artifacts";

/// 领域包 id ->（pin 前缀、缓存文件名、展示名）。
const PACK_SOURCES: [(&str, &str, &str, &str); 2] = [
    ("it", "thuocl-it", "THUOCL_IT.txt", "IT/编程"),
    ("med", "thuocl-med", "THUOCL_medical.txt", "医学"),
];

/// SHA-256 十六进制（小写）。
#[must_use]
pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(64);
    for byte in digest {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|error| format!("读取 {} 失败: {error}", path.display()))?;
    Ok(sha256_hex(&bytes))
}

// ---------------------------------------------------------------------------
// pins 读取与 source-check
// ---------------------------------------------------------------------------

/// 一条数据源锁。字段与 `data/pins/*.json` 一一对应，缺失字段按缺省处理。
#[derive(Debug, Default, Clone, Deserialize)]
#[serde(default)]
pub struct Pin {
    pub id: String,
    pub kind: String,
    pub url: Option<String>,
    pub cache_file: Option<String>,
    pub cache_rel: Option<String>,
    pub snapshot_file: Option<String>,
    pub sha256: Option<String>,
    pub size: Option<u64>,
    pub name: Option<String>,
}

/// source-check 统计。
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SourceCheckStats {
    pub checked: usize,
    pub locked_ok: usize,
    pub unlocked: usize,
    pub failures: Vec<String>,
}

/// 读取仓库内全部 pin（跳过无 `id` 的文件，如快照数据本身）。
pub fn load_pins(root: &Path) -> Result<Vec<Pin>, String> {
    let dir = root.join(PINS_DIR);
    let mut pins = Vec::new();
    let entries = fs::read_dir(&dir)
        .map_err(|error| format!("读取 pins 目录失败（{}）: {error}", dir.display()))?;
    for entry in entries {
        let entry = entry.map_err(|error| format!("读取 pins 目录项失败: {error}"))?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let text = fs::read_to_string(&path)
            .map_err(|error| format!("读取 pin 失败（{}）: {error}", path.display()))?;
        let pin: Pin = serde_json::from_str(&text)
            .map_err(|error| format!("解析 pin 失败（{}）: {error}", path.display()))?;
        if pin.id.is_empty() {
            continue;
        }
        pins.push(pin);
    }
    pins.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(pins)
}

/// 校验单个 pin 指向的缓存/快照文件哈希。
/// 返回 `Ok(true)`=哈希锁定且一致；`Ok(false)`=未锁定（pin 缺 sha256）。
fn verify_pin(root: &Path, pin: &Pin) -> Result<bool, String> {
    if pin.kind == "snapshot" {
        let Some(rel) = pin.snapshot_file.as_deref() else {
            return Err("快照型 pin 缺 snapshot_file".to_owned());
        };
        let path = root.join(rel);
        if !path.exists() {
            return Err(format!("快照文件缺失：{}", path.display()));
        }
        let actual = sha256_file(&path)?;
        match &pin.sha256 {
            Some(expected) if expected.eq_ignore_ascii_case(&actual) => Ok(true),
            Some(expected) => Err(format!(
                "快照哈希不一致：期望 {expected}，实际 {actual}（快照被改动）"
            )),
            None => Ok(false),
        }
    } else {
        let target = pin_target(root, pin)?;
        if !target.exists() {
            return Err(format!(
                "缓存缺失（先运行 scripts/fetch-sources.ps1）：{}",
                target.display()
            ));
        }
        let actual = sha256_file(&target)?;
        match &pin.sha256 {
            Some(expected) if expected.eq_ignore_ascii_case(&actual) => Ok(true),
            Some(expected) => Err(format!(
                "缓存哈希漂移：期望 {expected}，实际 {actual}（源内容已变化，人工审查 pin）"
            )),
            None => Ok(false),
        }
    }
}

fn pin_target(root: &Path, pin: &Pin) -> Result<PathBuf, String> {
    if let Some(rel) = &pin.cache_rel {
        return Ok(root.join(rel));
    }
    let Some(file) = &pin.cache_file else {
        return Err(format!("pin {} 缺 cache_file", pin.id));
    };
    Ok(root.join(CACHE_DIR).join(file))
}

/// `source-check`：核对全部 pin 指向文件哈希，任一漂移即返回 Err。
pub fn source_check(root: &Path) -> Result<SourceCheckStats, String> {
    let pins = load_pins(root)?;
    let mut stats = SourceCheckStats::default();
    for pin in &pins {
        stats.checked += 1;
        match verify_pin(root, pin) {
            Ok(true) => {
                stats.locked_ok += 1;
                println!("[{}] ok（哈希锁定一致）", pin.id);
            }
            Ok(false) => {
                stats.unlocked += 1;
                println!(
                    "[{}] 未锁定（pin 缺 sha256，先运行 fetch-sources.ps1 -WritePins）",
                    pin.id
                );
            }
            Err(message) => {
                stats.failures.push(format!("{}: {message}", pin.id));
                println!("[{}] 失败：{message}", pin.id);
            }
        }
    }
    if stats.failures.is_empty() {
        println!(
            "source-check 通过：{} 个源，锁定一致 {}，未锁定 {}",
            stats.checked, stats.locked_ok, stats.unlocked
        );
        Ok(stats)
    } else {
        Err(format!(
            "source-check 失败：{} 个源哈希不一致",
            stats.failures.len()
        ))
    }
}

// ---------------------------------------------------------------------------
// 拼音注音
// ---------------------------------------------------------------------------

/// 汉字拼音注音表：词级（CC-CEDICT）优先，单字级（kTGHZ2013）兜底。
///
/// 领域词包（THUOCL 等）只提供汉字词，不含拼音；本表负责补齐拼音并
/// 逐音节校验引擎标准全拼表，未覆盖字或非法音节的词不进入构建。
pub struct PinyinTables {
    word_pinyin: HashMap<String, String>,
    char_pinyin: HashMap<char, String>,
    table: SyllableTable,
}

impl PinyinTables {
    /// 由 CC-CEDICT 文本与 Unihan kTGHZ2013 文本构建注音表。
    pub fn from_texts(cedict_text: &str, ktghz_text: &str) -> Self {
        let mut tables = PinyinTables {
            word_pinyin: HashMap::new(),
            char_pinyin: HashMap::new(),
            table: SyllableTable::standard(),
        };
        for line in cedict_text.lines() {
            let Some((word, marked, _)) = parse_cedict_line(line) else {
                continue;
            };
            if !is_cjk_word(&word) {
                continue;
            }
            let Some(syllables) = split_pinyin_syllables(&marked) else {
                continue;
            };
            if syllables.len() != word.chars().count()
                || syllables
                    .iter()
                    .any(|s| !tables.table.is_complete_syllable(s))
            {
                continue;
            }
            // 一词多音时保留首个读音（CC-CEDICT 条目排序即常用度优先）。
            tables
                .word_pinyin
                .entry(word)
                .or_insert_with(|| syllables.concat());
        }
        for line in ktghz_text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            // 格式：`U+XXXX: 拼音1,拼音2  # 汉字`；行内 `#` 之后为注释。
            let Some(sharp) = line.find('#') else {
                continue;
            };
            let Some(colon) = line[..sharp].find(':') else {
                continue;
            };
            let Some(raw) = line[colon + 1..sharp].split(',').next().map(str::trim) else {
                continue;
            };
            let Some(pinyin) = strip_tone_marks(raw) else {
                continue;
            };
            let Some(hanzi) = line[sharp + 1..].trim().chars().next() else {
                continue;
            };
            tables.char_pinyin.insert(hanzi, pinyin);
        }
        tables
    }

    /// 为单个词注音：词级命中直接返回（已校验）；否则逐字查单字表并校验。
    pub fn annotate(&self, word: &str) -> Option<String> {
        if let Some(pinyin) = self.word_pinyin.get(word) {
            return Some(pinyin.clone());
        }
        let mut syllables = Vec::with_capacity(word.chars().count());
        for ch in word.chars() {
            let pinyin = self.char_pinyin.get(&ch)?;
            if !self.table.is_complete_syllable(pinyin) {
                return None;
            }
            syllables.push(pinyin.as_str());
        }
        if syllables.len() != word.chars().count() {
            return None;
        }
        Some(syllables.concat())
    }

    /// 词级注音表大小（供诊断使用）。
    #[must_use]
    pub fn word_map_len(&self) -> usize {
        self.word_pinyin.len()
    }

    /// 单字注音表大小（供诊断使用）。
    #[must_use]
    pub fn char_map_len(&self) -> usize {
        self.char_pinyin.len()
    }

    /// 单字注音表字键迭代（供覆盖统计使用）。
    pub fn char_map_keys(&self) -> impl Iterator<Item = &char> {
        self.char_pinyin.keys()
    }
}

/// 把带音调符号的拼音字母归一化为无调 ASCII（`ü` 系转 `v`）；非拼音字符返回 `None`。
fn strip_tone_marks(input: &str) -> Option<String> {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        let base = match ch {
            'ā' | 'á' | 'ǎ' | 'à' => 'a',
            'ē' | 'é' | 'ě' | 'è' => 'e',
            'ī' | 'í' | 'ǐ' | 'ì' => 'i',
            'ō' | 'ó' | 'ǒ' | 'ò' => 'o',
            'ū' | 'ú' | 'ǔ' | 'ù' => 'u',
            'ǖ' | 'ǘ' | 'ǚ' | 'ǜ' | 'ü' => 'v',
            'a'..='z' => ch,
            'A'..='Z' => ch.to_ascii_lowercase(),
            _ => return None,
        };
        out.push(base);
    }
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

// ---------------------------------------------------------------------------
// build-pack：领域词包构建
// ---------------------------------------------------------------------------

/// build-pack 统计。
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PackStats {
    pub source_lines: usize,
    /// MDN 术语表标题解析出的纯汉字术语数（仅 it 包）。
    pub mdn_terms: usize,
    /// MDN 术语入包数（去重、注音后）。
    pub mdn_accepted: usize,
    pub accepted: usize,
    pub annotation_failed: usize,
    pub duplicates: usize,
    pub entry_count: usize,
    pub file_size: u64,
    pub sha256: String,
}

/// MDN zh-cn 术语表打包文件（`scripts/fetch-mdn-glossary.ps1` 产出，pin 锁定）。
pub const MDN_BUNDLE: &str = "mdn-glossary-zh-pages.txt";

/// MDN 术语静态词频：取 THUOCL_IT 文档频率中位数量级（DF 中位数 441）。
pub const MDN_FREQUENCY: u64 = 500;

/// 从 MDN 打包文件提取术语：取每页 front matter 的 `title:`，去引号与括注
/// （全角/半角括号内的英文或缩写），并按顿号/斜杠拆分并列名；只保留 2 字以上纯汉字。
/// 结果按出现顺序去重。
pub fn parse_mdn_titles(bundle: &str) -> Vec<String> {
    let mut terms = Vec::new();
    let mut seen = HashSet::new();
    for line in bundle.lines() {
        let Some(title) = line.strip_prefix("title:") else {
            continue;
        };
        let title = title.trim().trim_matches(['"', '\'']);
        let mut stripped = String::with_capacity(title.len());
        let mut depth = 0usize;
        for ch in title.chars() {
            match ch {
                '（' | '(' => depth += 1,
                '）' | ')' => depth = depth.saturating_sub(1),
                _ if depth == 0 => stripped.push(ch),
                _ => {}
            }
        }
        for part in stripped.split(['、', '/', '／']) {
            let part = part.trim();
            if part.chars().count() >= 2 && is_cjk_word(part) && seen.insert(part.to_owned()) {
                terms.push(part.to_owned());
            }
        }
    }
    terms
}

/// 构建领域词包（thuocl 词表 -> 注音 -> v2 词典）。
///
/// `it` 包另并入 MDN zh-cn 术语表标题（CC BY-SA 2.5+，署名 Mozilla Contributors）。
///
/// 词频取 THUOCL 文档频率（DF）；领域词无译文（译文层按现有策略过滤空译文）。
pub fn build_pack(pack_id: &str, root: &Path) -> Result<PackStats, String> {
    let Some((_, cache_file, pack_name)) = PACK_SOURCES
        .iter()
        .find(|(id, _, _, _)| *id == pack_id)
        .map(|(_, file, cache, name)| (*file, *cache, *name))
    else {
        return Err(format!(
            "未知包 id：{pack_id}（支持：{}）",
            PACK_SOURCES
                .iter()
                .map(|(id, _, _, _)| *id)
                .collect::<Vec<_>>()
                .join(" / ")
        ));
    };

    source_check(root)?;

    let text = fs::read_to_string(root.join(CACHE_DIR).join(cache_file))
        .map_err(|error| format!("读取包源失败（{cache_file}）: {error}"))?;
    let cedict_text = fs::read_to_string(root.join("data/raw/cedict_ts.u8"))
        .map_err(|error| format!("读取 CC-CEDICT 失败: {error}"))?;
    let ktghz_text = fs::read_to_string(root.join(CACHE_DIR).join("kTGHZ2013.txt"))
        .map_err(|error| format!("读取注音底表失败: {error}"))?;
    let tables = PinyinTables::from_texts(&cedict_text, &ktghz_text);

    let mut entries = Vec::new();
    let mut seen = HashSet::new();
    let mut stats = PackStats::default();
    for line in text.lines() {
        stats.source_lines += 1;
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut parts = line.split_whitespace();
        let Some(word) = parts.next() else {
            continue;
        };
        let frequency = parts
            .next()
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(1)
            .min(u64::from(u32::MAX));
        if !is_cjk_word(word) {
            continue;
        }
        let Some(pinyin) = tables.annotate(word) else {
            stats.annotation_failed += 1;
            continue;
        };
        if !seen.insert((word.to_owned(), pinyin.clone())) {
            stats.duplicates += 1;
            continue;
        }
        stats.accepted += 1;
        entries.push(DictionaryEntry::new(word, pinyin, frequency));
    }

    if pack_id == "it" {
        let bundle = fs::read_to_string(root.join(CACHE_DIR).join(MDN_BUNDLE))
            .map_err(|error| format!("读取 MDN 术语表打包失败（{MDN_BUNDLE}）: {error}"))?;
        for term in parse_mdn_titles(&bundle) {
            stats.mdn_terms += 1;
            let Some(pinyin) = tables.annotate(&term) else {
                stats.annotation_failed += 1;
                continue;
            };
            if !seen.insert((term.clone(), pinyin.clone())) {
                stats.duplicates += 1;
                continue;
            }
            stats.accepted += 1;
            stats.mdn_accepted += 1;
            entries.push(DictionaryEntry::new(term, pinyin, MDN_FREQUENCY));
        }
    }

    let bytes = crate::build_v2(&entries, &[]).map_err(|error| error.to_string())?;
    let output = root.join(ARTIFACTS_DIR).join(format!("{pack_id}.zyct"));
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).map_err(|error| format!("创建输出目录失败: {error}"))?;
    }
    fs::write(&output, &bytes).map_err(|error| format!("写入词典包失败: {error}"))?;

    stats.entry_count = entries.len();
    stats.file_size = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
    stats.sha256 = sha256_hex(&bytes);
    println!(
        "领域包 {pack_name}（{pack_id}.zyct）构建完成：源行 {}，入包 {}，注音失败 {}，去重 {}，词条 {}，大小 {} 字节",
        stats.source_lines,
        stats.accepted,
        stats.annotation_failed,
        stats.duplicates,
        stats.entry_count,
        stats.file_size
    );
    if pack_id == "it" {
        println!(
            "MDN 术语表：纯汉字术语 {}，入包 {}",
            stats.mdn_terms, stats.mdn_accepted
        );
    }
    println!(
        "注音底表：词级 {}，单字级 {}",
        tables.word_map_len(),
        tables.char_map_len()
    );
    Ok(stats)
}

// ---------------------------------------------------------------------------
// manifest：生成与复核
// ---------------------------------------------------------------------------

/// manifest schema 版本（方案设计 11.7）。
pub const MANIFEST_SCHEMA: u32 = 1;

/// 单个词典包的发布元数据。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PackMeta {
    pub id: String,
    pub name: String,
    pub version: String,
    pub file: String,
    pub sha256: String,
    pub size: u64,
    pub min_engine_version: String,
}

/// 发布 manifest（签名在 M6-U 引入，M6-P 产出未签名 JSON）。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Manifest {
    pub schema: u32,
    pub published_at: String,
    pub packs: Vec<PackMeta>,
}

/// 生成 manifest：扫描目录内 `*.zyct`，计算每包内容 SHA-256 与大小。
pub fn build_manifest(
    dir: &Path,
    version: &str,
    min_engine_version: &str,
) -> Result<Manifest, String> {
    if !dir.is_dir() {
        return Err(format!("目录不存在：{}", dir.display()));
    }
    let mut files = Vec::new();
    let entries = fs::read_dir(dir).map_err(|error| format!("读取目录失败: {error}"))?;
    for entry in entries {
        let entry = entry.map_err(|error| format!("读取目录项失败: {error}"))?;
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("zyct") {
            files.push(path);
        }
    }
    files.sort();

    let mut packs = Vec::new();
    for path in files {
        let bytes = fs::read(&path).map_err(|error| format!("读取包失败: {error}"))?;
        let id = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| format!("包文件名无效：{}", path.display()))?
            .to_owned();
        let file = path
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| format!("包文件名无效：{}", path.display()))?
            .to_owned();
        packs.push(PackMeta {
            id: id.clone(),
            name: id,
            version: version.to_owned(),
            file,
            sha256: sha256_hex(&bytes),
            size: u64::try_from(bytes.len()).unwrap_or(u64::MAX),
            min_engine_version: min_engine_version.to_owned(),
        });
    }
    packs.sort_by(|left, right| left.id.cmp(&right.id));

    let manifest = Manifest {
        schema: MANIFEST_SCHEMA,
        published_at: today(),
        packs,
    };
    println!("manifest 生成完毕：共 {} 个包", manifest.packs.len());
    Ok(manifest)
}

/// 复核 manifest：逐包文件存在、内容哈希与大小一致。
pub fn verify_manifest(path: &Path) -> Result<usize, String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("读取 manifest 失败（{}）: {error}", path.display()))?;
    let manifest: Manifest =
        serde_json::from_str(&text).map_err(|error| format!("manifest 解析失败: {error}"))?;
    let base = path.parent().unwrap_or_else(|| Path::new("."));
    let mut checked = 0usize;
    for pack in &manifest.packs {
        checked += 1;
        let file_path = base.join(&pack.file);
        if !file_path.exists() {
            return Err(format!("包文件缺失：{}", file_path.display()));
        }
        let actual = sha256_file(&file_path)?;
        if !actual.eq_ignore_ascii_case(&pack.sha256) {
            return Err(format!(
                "包 {} 内容哈希不一致：期望 {}，实际 {}",
                pack.id, pack.sha256, actual
            ));
        }
        let size = fs::metadata(&file_path)
            .map_err(|error| format!("读取包大小失败: {error}"))?
            .len();
        if size != pack.size {
            return Err(format!(
                "包 {} 大小不一致：期望 {}，实际 {size}",
                pack.id, pack.size
            ));
        }
        println!("[{}] ok（{} 字节）", pack.id, size);
    }
    println!("verify-manifest 通过：{checked} 个包全部一致");
    Ok(checked)
}

// ---------------------------------------------------------------------------
// build-base：骨架 + 领域词频标定 + 注音合并
// ---------------------------------------------------------------------------

/// 解析 wordfreq 数据文件（cBpack）：gzip 内 msgpack，
/// 解码为 list：[{format:"cB", version:1}, 0cB 词表, -1cB 词表, ...]。
/// 第 k 个词表（跳过头部后）表示 -k cB，对应概率 10^(-k/100)，
/// zipf = 9 - k/100；返回 词 -> round(zipf×1000)（u32，单调排序用）。
pub fn load_wordfreq_zh(decoded: &[u8]) -> Result<HashMap<String, u32>, String> {
    let mut reader = std::io::Cursor::new(decoded);
    let value = rmpv::decode::read_value(&mut reader)
        .map_err(|error| format!("wordfreq msgpack 解析失败: {error}"))?;
    let rmpv::Value::Array(items) = value else {
        return Err("wordfreq 数据不是 msgpack 数组".to_owned());
    };
    if items.len() < 2 {
        return Err("wordfreq 数据缺少头部或词表".to_owned());
    }
    let mut frequencies = HashMap::new();
    for (index, chunk) in items.iter().enumerate().skip(1) {
        let rmpv::Value::Array(words) = chunk else {
            return Err(format!("wordfreq 词表 {index} 不是数组"));
        };
        // zipf×1000 = (9 - index/100)*1000 = 9000 - 10*index
        let frequency =
            9000u32.saturating_sub(u32::try_from(index).unwrap_or(u32::MAX).saturating_mul(10));
        for word in words {
            if let rmpv::Value::String(text) = word {
                if let Some(text) = text.as_str() {
                    frequencies.insert(text.to_owned(), frequency);
                }
            }
        }
    }
    Ok(frequencies)
}

/// 从 wordfreq wheel 内读取 `wordfreq/data/large_zh.msgpack.gz` 并解析。
pub fn load_wordfreq_wheel_zh(wheel_bytes: &[u8]) -> Result<HashMap<String, u32>, String> {
    use std::io::Read;
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(wheel_bytes))
        .map_err(|error| format!("wordfreq wheel 打开失败: {error}"))?;
    let mut entry = archive
        .by_name("wordfreq/data/large_zh.msgpack.gz")
        .map_err(|error| format!("wordfreq 内 large_zh.msgpack.gz 缺失: {error}"))?;
    let mut compressed = Vec::new();
    entry
        .read_to_end(&mut compressed)
        .map_err(|error| format!("wordfreq 条目读取失败: {error}"))?;
    let mut decoded = Vec::new();
    flate2::read::GzDecoder::new(&compressed[..])
        .read_to_end(&mut decoded)
        .map_err(|error| format!("wordfreq gzip 解压失败: {error}"))?;
    load_wordfreq_zh(&decoded)
}

/// 解析 jieba dict.txt 行（`词 频次 词性`），返回 词 -> 频次。
pub fn load_jieba_freq(text: &str) -> HashMap<String, u64> {
    let mut map = HashMap::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let mut parts = line.split_whitespace();
        let Some(word) = parts.next() else {
            continue;
        };
        let Some(count) = parts.next().and_then(|value| value.parse::<u64>().ok()) else {
            continue;
        };
        map.insert(word.to_owned(), count);
    }
    map
}

/// jieba 频次 -> zipf×1000 标定：与 wordfreq 同尺度（wordfreq 顶部 ≈ 7,500-7,600 对应
/// 的 3.2e5 词频；取 log10 斜率使 jieba 顶部亦落在 ~7,500 量级）。
/// `score = 2000 + 1000*log10(freq)`，封顶 9000。
#[must_use]
pub fn jieba_score(count: u64) -> u32 {
    if count == 0 {
        return 0;
    }
    let score = 2000.0 + 1000.0 * (count as f64).log10();
    score.round().clamp(0.0, 9000.0) as u32
}

/// 将镜像拼音中的儿化标记 `'r`（撇号后为独立 r 音节）归一为 `'er`；
/// 仅当 r 后不再紧跟字母时才判定为儿化（`hua1'r` -> `hua1'er`），
/// 避免误伤 r 开头音节（`sui1'ran2` 中的 `ran` 保持原样）。
fn xdhyc_unretroflex(marked: &str) -> String {
    let mut out = String::with_capacity(marked.len() + 2);
    let mut chars = marked.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\'' && matches!(chars.peek(), Some(&'r')) {
            let after_r = chars.clone().nth(1);
            if !matches!(after_r, Some(c) if c.is_ascii_alphabetic()) {
                out.push_str("'er");
                chars.next(); // 消费 r
                continue;
            }
        }
        out.push(ch);
    }
    out
}

/// 解析 xdhyc 文本镜像行（`词<TAB>拼音<TAB>序号`，拼音为带调数字 + 撇号分隔），
/// 归一化为无调全拼并按标准音节表校验；返回 词 -> 拼音。
///
/// 词表版式归一化：
/// - 异形词并列行 `甲;乙`（如 `年轻;年青`）拆为两条词，共享同一拼音；
/// - 儿化 `hua1'r` 归一为 `hua1'er`（见 [`xdhyc_unretroflex`]）；
/// - 顿号 `宁为玉碎,不为瓦全` 与间隔号 `一二·九运动` 合并为无分隔词形、
///   拼音侧 `sui4',bu4` 合并音节为 `sui4'bu4`（顿号无语义，输入时由分词边界覆盖）；
/// - 全角外来字符行 `阿Ｑ` 保留原词形（拼音音节与字符数对齐）。
fn load_xdhyc(text: &str) -> Vec<(String, String)> {
    let table = SyllableTable::standard();
    let mut words = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let mut parts = line.split('\t');
        let (Some(word), Some(marked)) = (parts.next(), parts.next()) else {
            continue;
        };
        // 儿化 'r -> 'er（仅独立 r 音节）；顿号 ', -> '（合并音节分隔）
        let syllables_marked = xdhyc_unretroflex(marked).replace("',", "'");
        let Some(syllables) = split_pinyin_syllables(&syllables_marked) else {
            continue;
        };
        if syllables.iter().any(|s| !table.is_complete_syllable(s)) {
            continue;
        }
        for token in word.split(';') {
            // 顿号/间隔号为无语义分隔符，去除后按词形校验
            let clean = token.replace(['·', ',', '，'], "");
            if !load_xdhyc_valid_word(&clean) || clean.chars().count() != syllables.len() {
                continue;
            }
            words.push((clean, syllables.concat()));
        }
    }
    words
}

/// xdhyc 词形校验：纯 CJK，或全角外来字符（ＱＯＫ）混排（`阿Ｑ`、`卡拉ＯＫ`）。
fn load_xdhyc_valid_word(word: &str) -> bool {
    !word.is_empty()
        && word
            .chars()
            .all(|ch| ('\u{4e00}'..='\u{9fff}').contains(&ch) || "ＱＯＫ".contains(ch))
}

/// 加载 CC-CEDICT：词 ->（无调全拼，首条可读译文）；一词多音保留首个。
fn load_cedict_with_translation(text: &str) -> HashMap<String, (String, Option<String>)> {
    let table = SyllableTable::standard();
    let mut map = HashMap::new();
    for line in text.lines() {
        let Some((word, marked, translation)) = parse_cedict_line(line) else {
            continue;
        };
        if !is_cjk_word(&word) {
            continue;
        }
        let Some(syllables) = split_pinyin_syllables(&marked) else {
            continue;
        };
        if syllables.len() != word.chars().count()
            || syllables.iter().any(|s| !table.is_complete_syllable(s))
        {
            continue;
        }
        map.entry(word)
            .or_insert_with(|| (syllables.concat(), translation));
    }
    map
}

/// build-base 统计（S-1 轨道的实测值来源）。
#[derive(Debug, Default, Clone, PartialEq)]
pub struct BaseStats {
    /// 骨架词（xdhyc 56,008）入包数。
    pub skeleton_words: usize,
    /// 骨架词中命中 wordfreq 主词频源的数量。
    pub wordfreq_hits: usize,
    /// 骨架词命中率 = wordfreq_hits / skeleton_words（验收标准 7.5 ≥70%）。
    pub wordfreq_hit_pct: f64,
    /// 仅由 jieba 扩充进入的词条数。
    pub jieba_expansion: usize,
    /// 由 CC-CEDICT 兜底注音进入的词条数（含译文）。
    pub cedict_words: usize,
    /// 规范字集（kTGHZ 8,102 字，约等于通用规范汉字表 8,105）中出现在 base 的字占比。
    pub char_set_coverage_pct: f64,
    /// 最终词条总数。
    pub entry_count: usize,
    /// 产物字节数（S-1：base ≤60MB）。
    pub file_size: u64,
    /// 产物内容 SHA-256（manifest 输入）。
    pub sha256: String,
}

/// 构建基础包：骨架（xdhyc 全部）→ CEDICT 词级兜底 → jieba 扩充（纯 CJK、频率标定 ≥ min_score）。
/// 词频：wordfreq 主源（zipf×1000）优先，未命中取 jieba 标定值，再未命中按 1。
/// 输出 `data/artifacts/base.zyct`。
pub fn build_base(root: &Path, min_score: u32) -> Result<BaseStats, String> {
    source_check(root)?;
    let cache = root.join(CACHE_DIR);
    let raw = root.join("data/raw");

    let xdhyc_text = fs::read_to_string(cache.join("xdhyc-2008.txt"))
        .map_err(|error| format!("读取 xdhyc 骨架失败: {error}"))?;
    let skeleton = load_xdhyc(&xdhyc_text);

    let wheel_bytes = fs::read(cache.join("wordfreq-3.1.1-py3-none-any.whl"))
        .map_err(|error| format!("读取 wordfreq wheel 失败: {error}"))?;
    let wordfreq = load_wordfreq_wheel_zh(&wheel_bytes)?;

    let jieba_text = fs::read_to_string(cache.join("jieba-dict.txt"))
        .map_err(|error| format!("读取 jieba 词表失败: {error}"))?;
    let jieba = load_jieba_freq(&jieba_text);

    let cedict_text = fs::read_to_string(raw.join("cedict_ts.u8"))
        .map_err(|error| format!("读取 CC-CEDICT 失败: {error}"))?;
    let cedict = load_cedict_with_translation(&cedict_text);
    let ktghz_text = fs::read_to_string(cache.join("kTGHZ2013.txt"))
        .map_err(|error| format!("读取注音底表失败: {error}"))?;
    let tables = PinyinTables::from_texts(&cedict_text, &ktghz_text);

    let frequency_of = |word: &str| -> u32 {
        wordfreq.get(word).copied().unwrap_or_else(|| {
            jieba
                .get(word)
                .map(|count| jieba_score(*count))
                .unwrap_or(1)
        })
    };

    // a) 骨架：全部入包，拼音取镜像自带（官方拼音）。
    let mut merged: HashMap<String, (String, u32, Option<String>)> = HashMap::new();
    for (word, pinyin) in &skeleton {
        let frequency = frequency_of(word);
        merged.insert(word.clone(), (pinyin.clone(), frequency, None));
    }

    // b) 骨架 + CEDICT 词条：保留 CEDICT 译文；已入包（骨架）的词不重复。
    let mut cedict_words = 0usize;
    for (word, (pinyin, translation)) in &cedict {
        if merged.contains_key(word) {
            continue;
        }
        let frequency = frequency_of(word);
        merged.insert(
            word.clone(),
            (pinyin.clone(), frequency, translation.clone()),
        );
        cedict_words += 1;
    }

    // c) jieba 扩充：纯 CJK、拼音注音（CEDICT 词级 -> kTGHZ 字级）、标定频率 ≥ min_score。
    let mut jieba_expansion = 0usize;
    for (word, count) in &jieba {
        if merged.contains_key(word) {
            continue;
        }
        let score = jieba_score(*count);
        if score < min_score {
            continue;
        }
        if !is_cjk_word(word) {
            continue;
        }
        let Some(pinyin) = tables.annotate(word) else {
            continue;
        };
        merged.insert(word.clone(), (pinyin, score, None));
        jieba_expansion += 1;
    }

    // 规范字集覆盖：以注音底表字集为基线（≈ 通用规范汉字表）。
    let mut merged_chars: HashSet<char> = HashSet::new();
    for (word, (_, _, _)) in &merged {
        merged_chars.extend(word.chars());
    }
    let covered_chars = tables
        .char_map_keys()
        .filter(|ch| merged_chars.contains(ch))
        .count();

    let mut entries: Vec<DictionaryEntry> = Vec::with_capacity(merged.len());
    for (word, (pinyin, frequency, translation)) in merged {
        let mut entry = DictionaryEntry::new(word, pinyin, u64::from(frequency));
        if let Some(translation) = translation {
            entry = entry.with_translation(translation);
        }
        entries.push(entry);
    }
    let bytes = crate::build_v2(&entries, &[]).map_err(|error| error.to_string())?;

    let output = root.join(ARTIFACTS_DIR).join("base.zyct");
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent).map_err(|error| format!("创建输出目录失败: {error}"))?;
    }
    fs::write(&output, &bytes).map_err(|error| format!("写入 base 包失败: {error}"))?;

    let skeleton_words = skeleton.len();
    let wordfreq_hits = skeleton
        .iter()
        .filter(|(word, _)| wordfreq.contains_key(word))
        .count();
    let wordfreq_hit_pct = if skeleton_words == 0 {
        0.0
    } else {
        wordfreq_hits as f64 / skeleton_words as f64 * 100.0
    };
    let char_set_coverage_pct = if tables.char_map_len() == 0 {
        0.0
    } else {
        covered_chars as f64 / tables.char_map_len() as f64 * 100.0
    };
    let stats = BaseStats {
        skeleton_words,
        wordfreq_hits,
        wordfreq_hit_pct,
        jieba_expansion,
        cedict_words,
        char_set_coverage_pct,
        entry_count: entries.len(),
        file_size: u64::try_from(bytes.len()).unwrap_or(u64::MAX),
        sha256: sha256_hex(&bytes),
    };
    println!(
        "base 包构建完成（data/artifacts/base.zyct）：词条 {}，大小 {:.1} MB",
        stats.entry_count,
        stats.file_size as f64 / 1_048_576.0
    );
    println!(
        "骨架 {} 词（含镜像拼音），wordfreq 命中 {}（{:.1}%），jieba 扩充 {}，CEDICT 兜底 {}，规范字集覆盖 {:.1}%",
        stats.skeleton_words,
        stats.wordfreq_hits,
        stats.wordfreq_hit_pct,
        stats.jieba_expansion,
        stats.cedict_words,
        stats.char_set_coverage_pct
    );
    println!("内容 SHA-256: {}", stats.sha256);
    Ok(stats)
}

/// UTC 日期 `YYYY-MM-DD`（构建期确定性优先，时区差异不影响哈希/校验路径）。
#[must_use]
pub fn today() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    let (year, month, day) = civil_from_days(i64::try_from(secs / 86_400).unwrap_or(0));
    format!("{year:04}-{month:02}-{day:02}")
}

/// 儒略日数（自 1970-01-01 起）转公历年月日（Howard Hinnant 算法）。
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let year = if month <= 2 { year + 1 } else { year };
    (year, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn 声调标记归一化() {
        assert_eq!(strip_tone_marks("zhōng").as_deref(), Some("zhong"));
        assert_eq!(strip_tone_marks("lǜ").as_deref(), Some("lv"));
        assert_eq!(strip_tone_marks("㑇"), None);
        assert_eq!(strip_tone_marks("").as_deref(), None);
    }

    #[test]
    fn 词级与单字级注音() {
        let cedict = "傳統 简体 [jian3 ti3] /simplified/\n繁體 繁体 [fan2 ti3] /traditional/\n";
        let ktghz = "U+4E2D: zhōng,zhòng  # 中\nU+56FD: guó  # 国\n";
        let tables = PinyinTables::from_texts(cedict, ktghz);
        // 词级命中（CC-CEDICT 提供且校验通过；同词多音保留首个）
        assert_eq!(tables.annotate("简体").as_deref(), Some("jianti"));
        assert_eq!(tables.annotate("繁体").as_deref(), Some("fanti"));
        // 单字级兜底
        assert_eq!(tables.annotate("中国").as_deref(), Some("zhongguo"));
        // 未覆盖字 -> 无注音
        assert_eq!(tables.annotate("㑇㑊"), None);
        assert_eq!(tables.word_map_len(), 2);
        assert_eq!(tables.char_map_len(), 2);
    }

    #[test]
    fn source_check_哈希锁定与漂移检测() {
        let dir = std::env::temp_dir().join(format!(
            "zhu-ye-dict-m6-src-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(dir.join("data/pins")).unwrap();
        fs::create_dir_all(dir.join("data/cache")).unwrap();
        fs::write(dir.join("data/cache/demo.txt"), b"hello").unwrap();
        let pin = r#"{"id":"demo","kind":"url","url":"https://x/demo.txt","cache_file":"demo.txt","sha256":"2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"}"#;
        fs::write(dir.join("data/pins/demo.json"), pin).unwrap();

        let stats = source_check(&dir).unwrap();
        assert_eq!(stats.checked, 1);
        assert_eq!(stats.locked_ok, 1);

        // 篡改缓存 -> 漂移失败
        fs::write(dir.join("data/cache/demo.txt"), b"tampered").unwrap();
        let error = source_check(&dir).unwrap_err();
        assert!(error.contains("失败"), "{error}");

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn manifest_generate_verify_roundtrip() {
        let dir = std::env::temp_dir().join(format!(
            "zhu-ye-dict-m6-manifest-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        let pack_path = dir.join("it.zyct");
        fs::write(&pack_path, b"ZYDT-FAKE-BYTES").unwrap();

        let manifest = build_manifest(&dir, "2026.09.28", "0.1.0").unwrap();
        assert_eq!(manifest.packs.len(), 1);
        assert_eq!(manifest.packs[0].id, "it");
        assert_eq!(manifest.packs[0].sha256, sha256_hex(b"ZYDT-FAKE-BYTES"));

        let manifest_path = dir.join("manifest.json");
        fs::write(
            &manifest_path,
            serde_json::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();
        assert_eq!(verify_manifest(&manifest_path).unwrap(), 1);

        // 篡改包内容 -> 复核失败
        fs::write(&pack_path, b"ZYDT-TAMPERED").unwrap();
        assert!(verify_manifest(&manifest_path).is_err());

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn wordfreq_cbpack_解析与zipf标定() {
        // 结构：[header, -1cB 词表, -2cB 词表(空), -3cB 词表]
        let encoded: Vec<u8> = vec![
            0x94, // fixarray(4)
            0x82, // fixmap(2): header
            0xa6, b'f', b'o', b'r', b'm', b'a', b't', 0xa2, b'c', b'B', // "format":"cB"
            0xa7, b'v', b'e', b'r', b's', b'i', b'o', b'n', 0x01, // "version":1
            0x91, 0xa3, 0xe7, 0x9a, 0x84, // [-1cB: ["的"]]
            0x90, // [-2cB: []]
            0x92, 0xa3, 0xe7, 0x9a, 0x84, 0xa6, 0xe4, 0xb8, 0xad, 0xe6, 0x96,
            0x87, // [-3cB: ["的","中文"]]（fixstr(6)）
        ];
        let map = load_wordfreq_zh(&encoded).unwrap();
        // -1cB 桶的"的"先插入，-3cB 桶覆盖为 8970（实际数据中词只出现在一个桶）
        assert_eq!(map.get("的").copied(), Some(8970));
        assert_eq!(map.get("中文").copied(), Some(8970));
    }

    #[test]
    fn jieba_频次标定与词表解析() {
        assert!(jieba_score(318_825) > 7400 && jieba_score(318_825) < 7600);
        assert_eq!(jieba_score(0), 0);
        let map = load_jieba_freq("的 318825 uj\n一 200000 m\n坏行\n");
        assert_eq!(map.get("的").copied(), Some(318_825));
        assert_eq!(map.len(), 2); // “坏行”缺频次行忽略
    }

    #[test]
    fn xdhyc_行解析与音节校验() {
        let text = "的\tde\t1\n是\tshi4\t2\n正方体\tzheng4'fang1'ti3\t56008\n坏行\tzheshi5buhefa\t0\n年轻;年青\tnian2'qing1\t697\n花儿\thua1'r\t10721\n虽然\tsui1'ran2\t1\n宁为玉碎,不为瓦全\tning2'wei2'yu4'sui4',bu4'wei2'wa3'quan2\t1\n一二·九运动\tyi1'er4'jiu3'yun4'dong4\t2\n阿Ｑ\ta1'qiu2\t3\n卡拉ＯＫ\tka3'la1'o1'kei1\t4\n";
        let words = load_xdhyc(text);
        assert_eq!(words.len(), 10);
        assert_eq!(words[0], ("的".to_owned(), "de".to_owned()));
        assert_eq!(words[1], ("是".to_owned(), "shi".to_owned()));
        assert_eq!(words[2], ("正方体".to_owned(), "zhengfangti".to_owned()));
        // 异形词行拆两条
        assert!(words.contains(&("年轻".to_owned(), "nianqing".to_owned())));
        assert!(words.contains(&("年青".to_owned(), "nianqing".to_owned())));
        // 儿化归一（'r 后无字母才判儿化）
        assert!(words.contains(&("花儿".to_owned(), "huaer".to_owned())));
        // r 开头音节不受儿化规则误伤
        assert!(words.contains(&("虽然".to_owned(), "suiran".to_owned())));
        // 顿号行：词形去顿号、音节合并
        assert!(words.contains(&(
            "宁为玉碎不为瓦全".to_owned(),
            "ningweiyusui".to_owned() + "buweiwaquan"
        )));
        // 间隔号行：词形去间隔号
        assert!(words.contains(&("一二九运动".to_owned(), "yierjiuyundong".to_owned())));
        // 全角外来字符行保留词形
        assert!(words.contains(&("阿Ｑ".to_owned(), "aqiu".to_owned())));
        // 卡拉ＯＫ 拼音含 kei1（非普通话标准音节）-> 按规则丢弃（已知排除项）
        assert!(!words.iter().any(|(w, _)| w.contains("ＯＫ")));
        // 音节数与字符数不匹配或音节非法 -> 丢弃
        assert!(!words.iter().any(|(w, _)| w == "坏行"));
    }

    #[test]
    fn mdn_标题提取去括注拆并列() {
        let bundle = "### api\n---\ntitle: API\nslug: Glossary/API\n---\n正文 title: 不算\n### alpn\n---\ntitle: 应用层协议协商（ALPN）\n---\n### cssom\n---\ntitle: \"CSS 对象模型\"\n---\n### x\n---\ntitle: 变量、常量/字面量 (literal)\n---\n### dup\n---\ntitle: 应用层协议协商\n---\n### one\n---\ntitle: 位\n---\n";
        assert_eq!(
            parse_mdn_titles(bundle),
            vec!["应用层协议协商", "变量", "常量", "字面量"]
        );
    }

    #[test]
    fn cedict_带译文加载与一词多音首读优先() {
        let text = "繁體 简体 [jian3 ti3] /simplified/\n繁體 简体 [fanti3] /variant/\n繁體 繁体 [fan2 ti3] /traditional/\n";
        let map = load_cedict_with_translation(text);
        assert_eq!(map.len(), 2);
        let (pinyin, translation) = &map["简体"];
        assert_eq!(pinyin, "jianti");
        assert_eq!(translation.as_deref(), Some("simplified"));
    }
}
