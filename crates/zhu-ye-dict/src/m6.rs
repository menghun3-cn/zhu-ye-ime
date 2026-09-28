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
    pub accepted: usize,
    pub annotation_failed: usize,
    pub duplicates: usize,
    pub entry_count: usize,
    pub file_size: u64,
    pub sha256: String,
}

/// 构建领域词包（thuocl 词表 -> 注音 -> v2 词典）。
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
}
