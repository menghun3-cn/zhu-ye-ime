//! 用户词库持久化。
//!
//! 文件格式为版本化 JSON；写入采用“临时文件 + 原子替换”，
//! 读取遇到损坏内容时自动备份为 `.bak` 并重建空库，满足
//! FR-003 本地持久化与 NFR-002 损坏自动恢复要求。

use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::user_dict::{UserDictionary, UserTransition, UserWord};

/// 当前用户词库文件格式版本。
///
/// v2（T-174）新增 `transitions` 段（用户级二元语法）；v1 文件读取时
/// **无损迁移**（词条保留、转移置空）并原子重写为 v2。
pub const USER_DICT_FORMAT_VERSION: u32 = 2;

/// 最低可迁移版本（T-174）。
///
/// 低于该版本的文件**无法**无损迁移（无版本字段的早期格式），按损坏处理
/// （备份 `.bak` 后重建空库）。
pub const MIN_MIGRATABLE_USER_DICT_VERSION: u32 = 1;

/// 磁盘文件格式容器；`entries` 由 `words_sorted` 产出，顺序确定。
#[derive(Debug, Serialize, Deserialize)]
struct FileFormat {
    version: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    entries: Vec<UserWord>,
    /// 用户级二元转移（T-174）；v1 文件无此段，读取时置空。
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    transitions: Vec<UserTransition>,
}

/// 当前 Unix 时间秒；仅供 IME/CLI 记录选择时间使用。
#[must_use]
pub fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs())
}

/// 只读探测用户词库文件的健康状态（不写盘、不备份）。
///
/// `UserDictStore::load` 在损坏时会立即备份并重建，那属于"修复动作"；本探测供
/// "先报告后动手"的修复入口（设置窗口一级修复，T-076 / FR-043）使用，只回答
/// "这个文件目前是否损坏"，不产生任何副作用。
///
/// - 文件不存在 → `UserWordsProbe::Ok`（首次运行正常，无需修复）
/// - 可解析且版本受支持 → `Ok`
/// - JSON 损坏或版本低于当前支持 → `Corrupt`（与 `load` 的恢复语义一致）
/// - 版本高于当前支持 → `NewerVersion`（应升级读取方，改名重建会丢内容）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UserWordsProbe {
    #[default]
    Ok,
    Corrupt,
    NewerVersion {
        found: u32,
    },
}

/// 只读探测用户词库文件；见 [`UserWordsProbe`]。
///
/// **版本语义（T-174）**：v1（无 `transitions` 段）**可无损迁移**为 v2，
/// 因此**不算损坏**——若把 v1 判为损坏，设置窗口的「一级修复」会建议重建
/// 并丢掉用户全部词条。只有**低于可迁移版本**（当前为 v0，即无版本字段的
/// 早期格式）才按损坏处理，与 `load` 的恢复语义保持一致。
#[must_use]
pub fn probe_user_words_file(path: &Path) -> UserWordsProbe {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return UserWordsProbe::Ok,
        Err(_) => return UserWordsProbe::Corrupt,
    };
    let format: FileFormat = match serde_json::from_slice(&bytes) {
        Ok(format) => format,
        Err(_) => return UserWordsProbe::Corrupt,
    };
    if format.version > USER_DICT_FORMAT_VERSION {
        UserWordsProbe::NewerVersion {
            found: format.version,
        }
    } else if format.version >= MIN_MIGRATABLE_USER_DICT_VERSION {
        UserWordsProbe::Ok
    } else {
        // 低于可迁移版本：与 `load` 的恢复语义一致，视为损坏（备份重建）。
        UserWordsProbe::Corrupt
    }
}

/// 用户词库 JSON 文件访问器。
///
/// `UserDictionary` 只维护内存状态，本类型负责把状态落到指定路径；
/// 路径由调用方提供，便于 IME 使用 `%APPDATA%`，CLI 与测试使用独立目录。
#[derive(Debug, Clone)]
pub struct UserDictStore {
    path: PathBuf,
}

impl UserDictStore {
    /// 创建指向指定 JSON 文件的访问器；目录不存在时在首次保存时创建。
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// 用户词库文件路径。
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// 加载用户词库；文件不存在返回空库，内容损坏时备份并重建空库。
    ///
    /// **版本迁移（T-174）**：v1 文件（无 `transitions` 段）读取时按「词条完整
    /// 保留、转移置空」在**内存中**升级为 v2，**词条零丢失**；读取本身**不
    /// 写盘**（只读访问者不应有副作用），磁盘升级发生在下一次 `save`——保存
    /// 一律写当前版本（v2），迁移因此仍自动完成。低于可迁移版本（v0）按损坏
    /// 处理（备份重建），版本高于当前支持时拒绝读（避免丢内容）。
    pub fn load(&self) -> Result<UserDictionary> {
        let bytes = match fs::read(&self.path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Ok(UserDictionary::new());
            }
            Err(error) => {
                return Err(Error::UserDictionary(format!(
                    "读取 {} 失败: {error}",
                    self.path.display()
                )));
            }
        };

        let format: FileFormat = match serde_json::from_slice(&bytes) {
            Ok(format) => format,
            Err(_) => return self.recover_corrupt(),
        };

        if format.version > USER_DICT_FORMAT_VERSION {
            return Err(Error::UserDictionary(format!(
                "用户词库版本 {} 高于当前支持的 {}，请升级后再打开 {}",
                format.version,
                USER_DICT_FORMAT_VERSION,
                self.path.display()
            )));
        }
        if format.version < MIN_MIGRATABLE_USER_DICT_VERSION {
            // 低于可迁移版本（无版本字段的早期格式）：按损坏处理（备份重建）。
            return self.recover_corrupt();
        }
        if format.version < USER_DICT_FORMAT_VERSION {
            // v1 → v2 无损迁移：**只在内存中**升级（词条保留、转移置空）。
            //
            // 此处**不写盘**——`load` 是读操作，让只读访问者（CLI `user list`、
            // 设置窗口的状态检查）触发写入是副作用污染。磁盘升级发生在下一次
            // `save` 时：保存一律写当前版本（v2），迁移因此仍然自动完成，且
            // 词条零丢失。
            return Ok(UserDictionary::from_entries_and_transitions(
                format.entries,
                format.transitions,
            ));
        }
        Ok(UserDictionary::from_entries_and_transitions(
            format.entries,
            format.transitions,
        ))
    }

    /// 原子保存用户词库：先写同目录临时文件，再替换目标文件。
    pub fn save(&self, dictionary: &UserDictionary) -> Result<()> {
        let format = FileFormat {
            version: USER_DICT_FORMAT_VERSION,
            entries: dictionary.words_sorted(),
            transitions: dictionary.transitions_sorted(),
        };
        let bytes = serde_json::to_vec_pretty(&format)
            .map_err(|error| Error::UserDictionary(format!("序列化用户词库失败: {error}")))?;
        self.write_atomic(&bytes)
    }

    /// 删除一个用户词并落盘；词不存在返回 `Ok(false)`。
    pub fn delete_word(
        &self,
        dictionary: &mut UserDictionary,
        word: &str,
        pinyin: &str,
    ) -> Result<bool> {
        if !dictionary.delete(word, pinyin) {
            return Ok(false);
        }
        self.save(dictionary)?;
        Ok(true)
    }

    /// 清空用户词库并落盘。
    pub fn reset(&self, dictionary: &mut UserDictionary) -> Result<()> {
        dictionary.reset();
        self.save(dictionary)
    }

    /// 把损坏文件备份为 `.bak`，再用空库覆盖原路径。
    fn recover_corrupt(&self) -> Result<UserDictionary> {
        let backup = self.backup_path();
        fs::rename(&self.path, &backup).map_err(|error| {
            Error::UserDictionary(format!(
                "备份损坏的用户词库 {} 到 {} 失败: {error}",
                self.path.display(),
                backup.display()
            ))
        })?;
        self.save(&UserDictionary::new())?;
        Ok(UserDictionary::new())
    }

    /// 写入临时文件后原子替换目标，确保中途失败不破坏原文件。
    fn write_atomic(&self, bytes: &[u8]) -> Result<()> {
        let parent = self
            .path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(parent).map_err(|error| {
            Error::UserDictionary(format!(
                "创建用户词库目录 {} 失败: {error}",
                parent.display()
            ))
        })?;

        let tmp = self.tmp_path();
        let write_result = (|| -> io::Result<()> {
            let mut file = fs::File::create(&tmp)?;
            file.write_all(bytes)?;
            file.sync_all()?;
            fs::rename(&tmp, &self.path)?;
            Ok(())
        })();
        if let Err(error) = write_result {
            let _ = fs::remove_file(&tmp);
            return Err(Error::UserDictionary(format!(
                "写入 {} 失败: {error}",
                self.path.display()
            )));
        }
        Ok(())
    }

    fn backup_path(&self) -> PathBuf {
        self.path.with_extension("bak")
    }

    fn tmp_path(&self) -> PathBuf {
        let name = self.path.file_name().map_or_else(
            || "user_words.json.tmp".to_owned(),
            |name| name.to_string_lossy().into_owned(),
        );
        self.path.with_file_name(format!("{name}.tmp"))
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{probe_user_words_file, unix_now, UserDictStore, UserWordsProbe};
    use crate::user_dict::UserDictionary;

    fn temp_dir(name: &str) -> PathBuf {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("zhu-ye-{name}-{}-{now}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn store_in(name: &str) -> (UserDictStore, PathBuf) {
        let dir = temp_dir(name);
        let store = UserDictStore::new(dir.join("user_words.json"));
        (store, dir)
    }

    #[test]
    fn 保存后重新加载一致() {
        let (store, dir) = store_in("roundtrip");
        let mut dict = UserDictionary::new();
        dict.record_selection("竹叶", "zhuye", 1);
        dict.record_selection("竹叶", "zhuye", 2);
        store.save(&dict).unwrap();

        let loaded = store.load().unwrap();
        assert_eq!(loaded.frequency("竹叶", "zhuye"), 2);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn 文件不存在返回空库() {
        let (store, dir) = store_in("missing");
        assert!(store.load().unwrap().is_empty());
        assert!(!store.path().exists());
        fs::remove_dir_all(dir).unwrap();
    }

    /// T-174：v1 文件读取时**无损迁移**为 v2——词条完整保留、转移置空；
    /// 读取本身**不写盘**（只读访问者无副作用），磁盘升级在下次 `save` 时
    /// 完成。
    #[test]
    fn v1文件无损迁移为v2() {
        let (store, dir) = store_in("migrate-v1");
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            store.path(),
            r#"{"version": 1, "entries": [{"word": "竹叶", "pinyin": "zhuye", "frequency": 3, "last_used": 100}]}"#,
        )
        .unwrap();

        let loaded = store.load().unwrap();
        assert_eq!(loaded.frequency("竹叶", "zhuye"), 3, "v1 词条必须完整保留");
        assert_eq!(loaded.transition_count(), 0, "v1 无转移段");

        // 读取不写盘：磁盘仍是 v1（只读访问者不应产生副作用）。
        let text = fs::read_to_string(store.path()).unwrap();
        let value: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(value["version"], 1, "load 不应改写磁盘文件");

        // 下次保存按当前版本（v2）写出，迁移随之完成，词条仍在。
        store.save(&loaded).unwrap();
        let text = fs::read_to_string(store.path()).unwrap();
        let value: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(value["version"], 2, "保存后应写为 v2");
        assert_eq!(value["entries"][0]["word"], "竹叶", "重写后词条仍在");
        assert_eq!(value["entries"][0]["frequency"], 3, "词频不丢");
        fs::remove_dir_all(dir).unwrap();
    }

    /// T-174：v2 文件的转移段往返保持一致。
    #[test]
    fn v2转移段往返一致() {
        let (store, dir) = store_in("roundtrip-v2");
        let mut dict = UserDictionary::new();
        dict.record_selection("你好", "nihao", 1);
        dict.record_transition("我说", "你好", "nihao", 2);
        store.save(&dict).unwrap();

        let loaded = store.load().unwrap();
        assert_eq!(loaded.transition_frequency("我说", "你好"), 1);
        assert_eq!(loaded.frequency("你好", "nihao"), 1);
        fs::remove_dir_all(dir).unwrap();
    }

    /// T-174：v1 之后的旧版本（v0）仍按损坏处理（备份重建），保持既有语义。
    #[test]
    fn v0文件按损坏处理备份重建() {
        let (store, dir) = store_in("v0");
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            store.path(),
            r#"{"version": 0, "entries": [{"word": "竹叶", "pinyin": "zhuye", "frequency": 3}]}"#,
        )
        .unwrap();

        let loaded = store.load().unwrap();
        assert!(loaded.is_empty(), "v0 视为损坏并重建空库");
        assert!(store.path().with_extension("bak").exists());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn 损坏文件备份并重建空库() {
        let (store, dir) = store_in("corrupt");
        fs::create_dir_all(&dir).unwrap();
        fs::write(store.path(), "not json").unwrap();
        fs::create_dir_all(&dir).unwrap();

        let loaded = store.load().unwrap();
        assert!(loaded.is_empty());
        assert!(store.path().with_extension("bak").exists());

        let reloaded = store.load().unwrap();
        assert!(reloaded.is_empty());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn 未知高版本不覆盖原文件() {
        let (store, dir) = store_in("future-version");
        fs::create_dir_all(&dir).unwrap();
        fs::write(store.path(), r#"{"version": 99, "entries": []}"#).unwrap();

        let error = store.load().unwrap_err();
        assert!(error.to_string().contains("高于当前支持"));
        assert!(store.path().exists());
        assert!(!store.path().with_extension("bak").exists());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn 探针区分健康损坏与高一版本() {
        let (store, dir) = store_in("probe");
        fs::create_dir_all(&dir).unwrap();
        let path = store.path();

        // 不存在：首次运行正常，无需修复。
        assert_eq!(probe_user_words_file(path), UserWordsProbe::Ok);

        // 合法内容（当前版本）：健康。
        let mut dict = UserDictionary::new();
        dict.record_selection("竹叶", "zhuye", 1);
        store.save(&dict).unwrap();
        assert_eq!(probe_user_words_file(path), UserWordsProbe::Ok);

        // JSON 损坏：破坏内容；探针必须不写盘（不产生 .bak、原文件不动）。
        fs::write(path, "not json").unwrap();
        assert_eq!(probe_user_words_file(path), UserWordsProbe::Corrupt);
        assert!(!path.with_extension("bak").exists(), "探针不得备份或重建");
        assert!(path.exists());

        // 低于可迁移版本（v0，无版本字段的早期格式）：与 load 的恢复语义
        // 一致视为损坏。
        fs::write(path, r#"{"version": 0, "entries": []}"#).unwrap();
        assert_eq!(probe_user_words_file(path), UserWordsProbe::Corrupt);

        // v1：可无损迁移为 v2，**不算损坏**——否则设置窗口的「一级修复」会
        // 建议重建并丢掉用户全部词条。
        fs::write(path, r#"{"version": 1, "entries": []}"#).unwrap();
        assert_eq!(probe_user_words_file(path), UserWordsProbe::Ok);

        // 高于当前版本：应升级读取方，不属于"改名重建"场景。
        fs::write(path, r#"{"version": 99, "entries": []}"#).unwrap();
        assert_eq!(
            probe_user_words_file(path),
            UserWordsProbe::NewerVersion { found: 99 }
        );
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn 删除词与重置均落盘() {
        let (store, dir) = store_in("delete-reset");
        let mut dict = UserDictionary::new();
        dict.record_selection("竹叶", "zhuye", 1);
        store.save(&dict).unwrap();

        assert!(store.delete_word(&mut dict, "竹叶", "zhuye").unwrap());
        assert!(!store.delete_word(&mut dict, "竹叶", "zhuye").unwrap());
        assert!(store.load().unwrap().is_empty());

        dict.record_selection("竹叶", "zhuye", 2);
        store.reset(&mut dict).unwrap();
        assert!(store.load().unwrap().is_empty());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn 原子保存不残留临时文件() {
        let (store, dir) = store_in("atomic");
        let mut dict = UserDictionary::new();
        dict.record_selection("竹叶", "zhuye", 1);
        store.save(&dict).unwrap();
        let tmp = store.tmp_path();
        assert!(!tmp.exists());
        assert!(unix_now() > 0);
        fs::remove_dir_all(dir).unwrap();
    }
}
