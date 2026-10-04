//! 已安装包清单 `packs/installed.json`（需求 §17.7）。
//!
//! 承载**机器事实**——磁盘上实际有什么、什么版本——与承载**用户意图**的 `config.json`
//! （启用哪些包、是否联网）职责分离。写入方是在线更新器与设置窗口的导入动作。
//!
//! 清单是**非权威**的：缺失、损坏或版本过高时只降级为"元数据未知"，由上层继续列出
//! `packs/` 里实际存在的包，绝不让设置窗口因此不可用。

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// 清单格式版本；字段变更时递增。
pub const INSTALLED_FORMAT_VERSION: u32 = 1;

/// 清单文件名。
pub const INSTALLED_FILE_NAME: &str = "installed.json";

/// 包的来源。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PackSource {
    /// 由在线更新器应用（有签名、有版本）。
    Update,
    /// 由用户导入本地文件（**不验签**，可能没有版本）。
    Import,
}

/// 单个已安装包的记录。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstalledPack {
    /// 包 id，对应 `<id>.zyct`。
    pub id: String,
    /// 包版本；导入的本地包可能没有版本。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// 整文件 SHA-256（小写十六进制）；未知时为空串。
    #[serde(default)]
    pub sha256: String,
    /// 安装或导入时间（Unix 秒）。
    #[serde(default)]
    pub installed_at: u64,
    /// 来源。
    pub source: PackSource,
}

/// 清单文件内容。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstalledRecord {
    /// 格式版本。
    #[serde(default = "default_version")]
    pub version: u32,
    /// 已安装包记录。
    #[serde(default)]
    pub packs: Vec<InstalledPack>,
}

fn default_version() -> u32 {
    INSTALLED_FORMAT_VERSION
}

impl Default for InstalledRecord {
    fn default() -> Self {
        Self {
            version: INSTALLED_FORMAT_VERSION,
            packs: Vec::new(),
        }
    }
}

impl InstalledRecord {
    /// 从 JSON 解析；版本高于支持范围时返回错误，其余损坏由调用方降级。
    pub fn from_json(text: &str) -> Result<Self, String> {
        let parsed: Self =
            serde_json::from_str(text).map_err(|error| format!("清单解析失败: {error}"))?;
        if parsed.version > INSTALLED_FORMAT_VERSION {
            return Err(format!(
                "清单版本 {} 高于当前支持的 {}，元数据按未知处理",
                parsed.version, INSTALLED_FORMAT_VERSION
            ));
        }
        Ok(parsed)
    }

    /// 序列化为带缩进的 JSON。
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|error| format!("清单序列化失败: {error}"))
    }

    /// 按 id 取记录。
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&InstalledPack> {
        self.packs.iter().find(|pack| pack.id == id)
    }

    /// 插入或替换一条记录（同 id 只保留一条）。
    pub fn upsert(&mut self, pack: InstalledPack) {
        match self
            .packs
            .iter_mut()
            .find(|existing| existing.id == pack.id)
        {
            Some(existing) => *existing = pack,
            None => self.packs.push(pack),
        }
    }

    /// 移除一条记录；返回是否确有该记录。
    pub fn remove(&mut self, id: &str) -> bool {
        let before = self.packs.len();
        self.packs.retain(|pack| pack.id != id);
        self.packs.len() != before
    }
}

/// 清单路径。
#[must_use]
pub fn record_path(packs_dir: &Path) -> PathBuf {
    packs_dir.join(INSTALLED_FILE_NAME)
}

/// 读取清单；缺失返回空记录且无诊断，损坏或版本过高时回退空记录**并给出诊断**。
///
/// 返回空记录意味着"元数据未知"，而不是"没有包"——上层的包列表以磁盘为准。
#[must_use]
pub fn load(packs_dir: &Path) -> (InstalledRecord, Option<String>) {
    let path = record_path(packs_dir);
    let Ok(text) = std::fs::read_to_string(&path) else {
        return (InstalledRecord::default(), None);
    };
    match InstalledRecord::from_json(&text) {
        Ok(record) => (record, None),
        Err(diagnostic) => (InstalledRecord::default(), Some(diagnostic)),
    }
}

/// 原子保存清单：先写同目录临时文件，再改名替换。
///
/// 临时文件与目标同目录，保证改名在同一次文件系统操作内完成，避免半个文件。
pub fn save(packs_dir: &Path, record: &InstalledRecord) -> Result<(), String> {
    let path = record_path(packs_dir);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| format!("创建包目录失败: {error}"))?;
    }
    let text = record.to_json()?;
    let temporary = path.with_extension("json.tmp");
    std::fs::write(&temporary, text).map_err(|error| format!("写入清单失败: {error}"))?;
    std::fs::rename(&temporary, &path).map_err(|error| {
        let _ = std::fs::remove_file(&temporary);
        format!("替换清单失败: {error}")
    })
}

#[cfg(test)]
mod tests {
    use super::{
        load, record_path, save, InstalledPack, InstalledRecord, PackSource,
        INSTALLED_FORMAT_VERSION,
    };
    use std::path::PathBuf;

    fn temp_dir(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "zhu-ye-installed-{label}-{}-{}",
            std::process::id(),
            zhu_ye_core::unix_now()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn record(id: &str, version: Option<&str>, source: PackSource) -> InstalledPack {
        InstalledPack {
            id: id.to_owned(),
            version: version.map(str::to_owned),
            sha256: "ab".repeat(32),
            installed_at: 1_759_420_800,
            source,
        }
    }

    #[test]
    fn 缺失清单返回空记录且无诊断() {
        let dir = temp_dir("missing");
        let (loaded, diagnostic) = load(&dir);
        assert_eq!(loaded, InstalledRecord::default());
        assert_eq!(diagnostic, None, "文件不存在不算损坏");
        assert!(!record_path(&dir).exists());
    }

    #[test]
    fn 损坏清单降级并给出诊断() {
        let dir = temp_dir("broken");
        std::fs::write(record_path(&dir), "{ not json").unwrap();
        let (loaded, diagnostic) = load(&dir);
        assert!(loaded.packs.is_empty());
        assert!(diagnostic.is_some(), "损坏必须回报诊断而不是静默");
    }

    #[test]
    fn 高版本清单降级为元数据未知() {
        let dir = temp_dir("future");
        std::fs::write(
            record_path(&dir),
            r#"{ "version": 99, "packs": [ { "id": "it", "source": "update" } ] }"#,
        )
        .unwrap();
        let (loaded, diagnostic) = load(&dir);
        assert!(
            loaded.packs.is_empty(),
            "版本过高时按未知处理，不得采纳其中的包记录"
        );
        assert!(diagnostic.unwrap().contains("99"));
    }

    #[test]
    fn 保存后往返一致且无临时文件残留() {
        let dir = temp_dir("roundtrip");
        let mut record_value = InstalledRecord::default();
        record_value.upsert(record("it", Some("2026.09.29-p3"), PackSource::Update));
        record_value.upsert(record("med", None, PackSource::Import));
        save(&dir, &record_value).unwrap();

        let (loaded, diagnostic) = load(&dir);
        assert_eq!(diagnostic, None);
        assert_eq!(loaded, record_value);
        assert_eq!(loaded.version, INSTALLED_FORMAT_VERSION);
        assert!(
            !record_path(&dir).with_extension("json.tmp").exists(),
            "临时文件不得残留"
        );
        // 导入的包没有版本，序列化后应省略该字段而不是写成 null。
        let text = std::fs::read_to_string(record_path(&dir)).unwrap();
        assert!(text.contains(r#""source": "import""#));
        assert!(!text.contains("null"));
    }

    #[test]
    fn 同id重复插入只保留一条() {
        let mut record_value = InstalledRecord::default();
        record_value.upsert(record("it", Some("v1"), PackSource::Update));
        record_value.upsert(record("it", Some("v2"), PackSource::Import));
        assert_eq!(record_value.packs.len(), 1);
        let entry = record_value.get("it").expect("应有记录");
        assert_eq!(entry.version.as_deref(), Some("v2"), "后写覆盖");
        assert_eq!(entry.source, PackSource::Import);
    }

    #[test]
    fn 移除记录() {
        let mut record_value = InstalledRecord::default();
        record_value.upsert(record("it", None, PackSource::Update));
        assert!(record_value.remove("it"));
        assert!(!record_value.remove("it"), "重复移除返回 false");
        assert_eq!(record_value.get("it"), None);
    }

    #[test]
    fn 缺少可选字段仍可解析() {
        // 只写 id 与来源的最小记录必须可读，避免旧写入方或手改文件导致整份清单失效。
        let parsed =
            InstalledRecord::from_json(r#"{ "packs": [ { "id": "slang", "source": "update" } ] }"#)
                .unwrap();
        let entry = parsed.get("slang").expect("应有记录");
        assert_eq!(entry.version, None);
        assert_eq!(entry.sha256, "");
        assert_eq!(entry.installed_at, 0);
        assert_eq!(parsed.version, INSTALLED_FORMAT_VERSION);
    }
}
