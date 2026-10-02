//! 领域包清单：扫描 `packs/` 与基础包目录，合并配置与已安装清单，产出 FR-022 的六字段。
//!
//! 与 `installed` 的分工：本模块负责"磁盘上实际有什么"，`installed` 负责"我们记录了什么"。
//! 两者都不权威，最终以磁盘为准——清单缺失时仍要列出实际存在的包，只是版本栏显示未知。

use std::path::{Path, PathBuf};

use zhu_ye_core::dict_format::{DictHeader, HEADER_SIZE};
use zhu_ye_core::{
    pack_display, ConfigFile, BASE_PACK_FILE_NAME, DISTRIBUTABLE_PACK_IDS, KNOWN_PACK_IDS,
    PACKS_DIR_NAME,
};

use crate::installed::{InstalledRecord, PackSource};

/// 一个包的界面信息（FR-022 的六字段加上呈现所需的派生信息）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackInfo {
    /// 包 id。
    pub id: String,
    /// 中文展示名。
    pub name: String,
    /// 一行简介。
    pub summary: String,
    /// 词条数；读不到头部时为 `None`。
    pub entry_count: Option<u64>,
    /// 文件字节数；文件不存在时为 `None`。
    pub size: Option<u64>,
    /// 版本；清单缺失或未记录时为 `None`。
    pub version: Option<String>,
    /// 是否已启用（来自配置）。
    pub enabled: bool,
    /// 是否为基础包：始终加载、不可停用。
    pub base: bool,
    /// 是否为开发期产物。
    pub development: bool,
    /// 来源；无记录时为 `None`。
    pub source: Option<PackSource>,
    /// 产物文件是否存在。
    pub present: bool,
}

impl PackInfo {
    /// 是否可勾选启停；基础包与不存在的包都不行。
    #[must_use]
    pub const fn toggleable(&self) -> bool {
        !self.base && self.present
    }

    /// 体积的界面文案。
    #[must_use]
    pub fn size_label(&self) -> String {
        match self.size {
            Some(bytes) => format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0)),
            None => "—".to_owned(),
        }
    }

    /// 词条数的界面文案。
    #[must_use]
    pub fn entry_label(&self) -> String {
        match self.entry_count {
            Some(count) => format!("{count} 条"),
            None => "—".to_owned(),
        }
    }

    /// 版本的界面文案；本地导入的包通常没有版本。
    #[must_use]
    pub fn version_label(&self) -> String {
        match (&self.version, self.source) {
            (Some(version), _) => version.clone(),
            (None, Some(PackSource::Import)) => "本地导入".to_owned(),
            (None, _) => "未知".to_owned(),
        }
    }
}

/// 包产物路径：基础包固定在基础目录，领域包在 `packs/`。
#[must_use]
pub fn pack_file(base_dir: &Path, packs_dir: &Path, id: &str) -> PathBuf {
    if id == "base" {
        base_dir.join(BASE_PACK_FILE_NAME)
    } else {
        packs_dir.join(format!("{id}.zyct"))
    }
}

/// `packs/` 目录路径。
#[must_use]
pub fn packs_dir(data_dir: &Path) -> PathBuf {
    data_dir.join(PACKS_DIR_NAME)
}

/// 只读 `.zyct` 头部解析词条数。
///
/// 不复用 `DictionaryFile::open`：后者会对整文件算 SHA-256 并逐条校验布局，而列表页只需
/// 词条数——对 23 MB 的基础包代价过大。**完整校验留给导入动作**，列表页不做信任判断。
#[must_use]
pub fn read_entry_count(path: &Path) -> Option<u64> {
    use std::io::Read;
    let mut file = std::fs::File::open(path).ok()?;
    let mut bytes = [0u8; HEADER_SIZE];
    file.read_exact(&mut bytes).ok()?;
    let header = DictHeader::from_bytes(&bytes).ok()?;
    Some(u64::from(header.entry_count))
}

/// 列出 `packs/` 目录里的 `.zyct` 文件名（去扩展名）；目录不存在或读取失败时为空。
///
/// 导入的本地包（FR-042）只以磁盘产物形式存在，不进入已知包表，必须靠这里发现。
fn read_dir_pack_ids(packs_dir: &Path) -> impl Iterator<Item = String> {
    let entries = match std::fs::read_dir(packs_dir) {
        Ok(entries) => entries,
        Err(_) => return Vec::new().into_iter(),
    };
    entries
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("zyct") {
                path.file_stem()
                    .and_then(|stem| stem.to_str())
                    .map(str::to_owned)
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .into_iter()
}

/// 列出全部包：基础包在最前，其后按可分发包顺序，再是磁盘上出现的其他已知包
/// （如开发期的 real/seed），最后是按字典序的磁盘未知包（本地导入，FR-042）。
///
/// 顺序完全确定，不依赖目录遍历顺序。
#[must_use]
pub fn list_packs(
    config: &ConfigFile,
    base_dir: &Path,
    packs_dir: &Path,
    record: &InstalledRecord,
) -> Vec<PackInfo> {
    let mut ids: Vec<String> = Vec::new();
    ids.push("base".to_owned());
    ids.extend(DISTRIBUTABLE_PACK_IDS.iter().map(|id| (*id).to_owned()));
    // 磁盘上出现的其余已知包（如开发期的 real/seed）追加在后，保持列表确定性。
    let mut extra: Vec<&str> = KNOWN_PACK_IDS
        .iter()
        .copied()
        .filter(|id| !ids.iter().any(|known| known == id))
        .filter(|id| pack_file(base_dir, packs_dir, id).is_file())
        .collect();
    extra.sort_unstable();
    ids.extend(extra.iter().map(|id| (*id).to_owned()));
    // 磁盘上出现的未知包（本地导入）排在已知集合之后，各自按字典序。
    let mut unknown: Vec<String> = read_dir_pack_ids(packs_dir)
        .filter(|id| !KNOWN_PACK_IDS.contains(&id.as_str()) && !ids.contains(id))
        .collect();
    unknown.sort();
    ids.extend(unknown);

    ids.into_iter()
        .map(|id| {
            let path = pack_file(base_dir, packs_dir, &id);
            let size = std::fs::metadata(&path).ok().map(|meta| meta.len());
            let entry = record.get(&id);
            let (name, summary, development) = match pack_display(&id) {
                Some(display) => (
                    display.name.to_owned(),
                    display.summary.to_owned(),
                    display.development,
                ),
                None => (id.clone(), "本地导入或未收录的词典包".to_owned(), false),
            };
            let enabled = id == "base" || config.enabled_packs.iter().any(|pack| pack == &id);
            let base = id == "base";
            PackInfo {
                id,
                name,
                summary,
                entry_count: if size.is_some() {
                    read_entry_count(&path)
                } else {
                    None
                },
                size,
                version: entry.and_then(|pack| pack.version.clone()),
                enabled,
                base,
                development,
                source: entry.map(|pack| pack.source),
                present: size.is_some(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{list_packs, pack_file, read_entry_count};
    use crate::installed::{InstalledPack, InstalledRecord, PackSource};
    use std::path::PathBuf;
    use zhu_ye_core::dict_format::{DictHeader, HEADER_SIZE};
    use zhu_ye_core::ConfigFile;

    fn temp_dir(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "zhu-ye-inventory-{label}-{}-{}",
            std::process::id(),
            zhu_ye_core::unix_now()
        ));
        std::fs::create_dir_all(dir.join("packs")).unwrap();
        dir
    }

    /// 写出一个只有头部合法、内容区为空的 `.zyct`；列表页只读头部，因此这足够。
    fn write_header_only(path: &PathBuf, entry_count: u32) {
        let header = DictHeader {
            pinyin_index_count: 1,
            entry_count,
            bigram_count: 0,
            word_translation_count: 0,
            reverse_translation_count: 0,
            word_translation_offset: 0,
            reverse_translation_offset: 0,
            pinyin_index_offset: 0,
            entry_table_offset: 0,
            bigram_offset: 0,
            text_pool_offset: 0,
            content_hash: [0u8; 32],
        };
        let mut bytes = header.to_bytes().to_vec();
        bytes.resize(HEADER_SIZE + 16, 0);
        std::fs::write(path, bytes).unwrap();
    }

    #[test]
    fn 只读头部取词条数() {
        let dir = temp_dir("header");
        let path = dir.join("packs").join("it.zyct");
        write_header_only(&path, 13_144);
        assert_eq!(read_entry_count(&path), Some(13_144));
        // 不是词典文件时返回 None 而不是 panic。
        let bogus = dir.join("packs").join("bogus.zyct");
        std::fs::write(
            &bogus,
            b"not a dict at all, but long enough to read past header size..",
        )
        .unwrap();
        assert_eq!(read_entry_count(&bogus), None);
        // 文件过短（读不满头部）同样返回 None。
        let short = dir.join("packs").join("short.zyct");
        std::fs::write(&short, b"ZYDT").unwrap();
        assert_eq!(read_entry_count(&short), None);
        assert_eq!(
            read_entry_count(&dir.join("packs").join("missing.zyct")),
            None
        );
    }

    #[test]
    fn 基础包排在最前且不可停用() {
        let dir = temp_dir("order");
        write_header_only(&dir.join("dictionary.zyct"), 378_312);
        let config = ConfigFile::default();
        let packs = list_packs(
            &config,
            &dir,
            &dir.join("packs"),
            &InstalledRecord::default(),
        );
        assert_eq!(packs[0].id, "base");
        assert!(packs[0].base);
        assert!(!packs[0].toggleable(), "基础包必须不可停用");
        assert!(packs[0].enabled, "基础包始终启用");
        assert_eq!(packs[0].name, "基础词典");
        assert_eq!(packs[0].entry_count, Some(378_312));
        assert!(packs[0].size.unwrap() > 0);
    }

    #[test]
    fn 可分发包顺序确定且与配置无关() {
        let dir = temp_dir("deterministic");
        for id in ["slang", "med", "it"] {
            write_header_only(&dir.join("packs").join(format!("{id}.zyct")), 10);
        }
        let config = ConfigFile::default();
        let packs = list_packs(
            &config,
            &dir,
            &dir.join("packs"),
            &InstalledRecord::default(),
        );
        let ids: Vec<&str> = packs.iter().map(|pack| pack.id.as_str()).collect();
        assert_eq!(ids, vec!["base", "it", "med", "slang"]);
        // 默认配置未启用任何领域包，因此它们存在但未启用。
        for pack in packs.iter().filter(|pack| !pack.base) {
            assert!(pack.present);
            assert!(!pack.enabled);
            assert!(pack.toggleable(), "存在的领域包可勾选");
        }
    }

    #[test]
    fn 勾选状态来自配置() {
        let dir = temp_dir("enabled");
        write_header_only(&dir.join("packs").join("it.zyct"), 10);
        write_header_only(&dir.join("packs").join("med.zyct"), 20);
        let config = ConfigFile {
            enabled_packs: vec!["med".to_owned()],
            ..ConfigFile::default()
        };
        let packs = list_packs(
            &config,
            &dir,
            &dir.join("packs"),
            &InstalledRecord::default(),
        );
        let it = packs.iter().find(|pack| pack.id == "it").unwrap();
        let med = packs.iter().find(|pack| pack.id == "med").unwrap();
        assert!(!it.enabled);
        assert!(med.enabled);
    }

    #[test]
    fn 清单缺失时版本未知但包仍列出() {
        let dir = temp_dir("no-record");
        write_header_only(&dir.join("packs").join("it.zyct"), 42);
        let packs = list_packs(
            &ConfigFile::default(),
            &dir,
            &dir.join("packs"),
            &InstalledRecord::default(),
        );
        let it = packs.iter().find(|pack| pack.id == "it").unwrap();
        assert_eq!(it.version, None, "清单缺失时版本为未知");
        assert_eq!(it.version_label(), "未知");
        assert_eq!(it.source, None);
        assert_eq!(it.entry_label(), "42 条", "词条数来自产物，不依赖清单");
        assert!(it.present);
    }

    #[test]
    fn 导入的包标注本地导入且无版本() {
        let dir = temp_dir("import");
        write_header_only(&dir.join("packs").join("it.zyct"), 7);
        let mut record = InstalledRecord::default();
        record.upsert(InstalledPack {
            id: "it".to_owned(),
            version: None,
            sha256: "cd".repeat(32),
            installed_at: 1,
            source: PackSource::Import,
        });
        let packs = list_packs(&ConfigFile::default(), &dir, &dir.join("packs"), &record);
        let it = packs.iter().find(|pack| pack.id == "it").unwrap();
        assert_eq!(it.source, Some(PackSource::Import));
        assert_eq!(it.version_label(), "本地导入");
    }

    #[test]
    fn 文件不存在的包仍列出但不可勾选() {
        let dir = temp_dir("absent");
        let packs = list_packs(
            &ConfigFile::default(),
            &dir,
            &dir.join("packs"),
            &InstalledRecord::default(),
        );
        let it = packs.iter().find(|pack| pack.id == "it").unwrap();
        assert!(!it.present, "产物不存在");
        assert_eq!(it.size, None);
        assert_eq!(it.entry_count, None);
        assert_eq!(it.size_label(), "—");
        assert_eq!(it.entry_label(), "—");
        assert!(!it.toggleable(), "不存在的包不能勾选");
    }

    #[test]
    fn 磁盘上未知的本地导入包被列出并排最后() {
        let dir = temp_dir("unknown-import");
        // 不止 .zyct：目录里的杂物（installed.json、临时文件）不得被当成包。
        write_header_only(&dir.join("packs").join("it.zyct"), 10);
        write_header_only(&dir.join("packs").join("real.zyct"), 5);
        write_header_only(&dir.join("packs").join("自建词库.zyct"), 3);
        write_header_only(&dir.join("packs").join("zz-extra.zyct"), 4);
        std::fs::write(dir.join("packs").join("installed.json"), "{}").unwrap();
        std::fs::write(dir.join("packs").join("scratch.txt"), "杂物").unwrap();
        let packs = list_packs(
            &ConfigFile::default(),
            &dir,
            &dir.join("packs"),
            &InstalledRecord::default(),
        );
        // 已知集合顺序不变，未知包按 UTF-8 字节字典序追加在最后（"z" 的字节低于汉字首字节）。
        let ids: Vec<&str> = packs.iter().map(|pack| pack.id.as_str()).collect();
        assert_eq!(
            ids,
            vec!["base", "it", "med", "slang", "real", "zz-extra", "自建词库"]
        );
        let unknown = packs
            .iter()
            .find(|pack| pack.id == "自建词库")
            .expect("未知包应出现在清单里");
        assert_eq!(unknown.name, "自建词库", "未知包用原始 id 作为展示名");
        assert!(!unknown.development, "未知包不是开发期产物");
        assert_eq!(unknown.summary, "本地导入或未收录的词典包");
        assert!(unknown.present);
        assert!(unknown.toggleable());
        assert_eq!(unknown.entry_count, Some(3));
        assert_eq!(unknown.source, None);
        // 杂物文件不被当成包。
        assert!(!packs.iter().any(|pack| pack.id == "scratch"));
    }

    #[test]
    fn 开发期产物出现在磁盘上时被标记() {
        let dir = temp_dir("development");
        write_header_only(&dir.join("packs").join("real.zyct"), 5);
        write_header_only(&dir.join("packs").join("seed.zyct"), 5);
        let packs = list_packs(
            &ConfigFile::default(),
            &dir,
            &dir.join("packs"),
            &InstalledRecord::default(),
        );
        let ids: Vec<&str> = packs.iter().map(|pack| pack.id.as_str()).collect();
        // 开发期产物排在可分发包之后，且按字典序。
        assert_eq!(ids, vec!["base", "it", "med", "slang", "real", "seed"]);
        assert!(packs.iter().filter(|pack| pack.development).count() == 2);
        // 不在磁盘上的开发期产物不出现。
        let empty = temp_dir("absent-dev");
        let listed = list_packs(
            &ConfigFile::default(),
            &empty,
            &empty.join("packs"),
            &InstalledRecord::default(),
        );
        assert!(listed.iter().all(|pack| !pack.development));
    }

    #[test]
    fn 包文件路径按基础包与领域包分流() {
        let base = PathBuf::from("C:/app");
        let packs = PathBuf::from("C:/data/packs");
        assert_eq!(
            pack_file(&base, &packs, "base"),
            base.join("dictionary.zyct")
        );
        assert_eq!(pack_file(&base, &packs, "it"), packs.join("it.zyct"));
    }
}
