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
use crate::user_dict::{UserDictionary, UserWord};

/// 当前用户词库文件格式版本。
pub const USER_DICT_FORMAT_VERSION: u32 = 1;

/// 磁盘文件格式容器；`entries` 由 `words_sorted` 产出，顺序确定。
#[derive(Debug, Serialize, Deserialize)]
struct FileFormat {
    version: u32,
    entries: Vec<UserWord>,
}

/// 当前 Unix 时间秒；仅供 IME/CLI 记录选择时间使用。
#[must_use]
pub fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs())
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
        if format.version < USER_DICT_FORMAT_VERSION {
            return self.recover_corrupt();
        }
        Ok(UserDictionary::from_entries(format.entries))
    }

    /// 原子保存用户词库：先写同目录临时文件，再替换目标文件。
    pub fn save(&self, dictionary: &UserDictionary) -> Result<()> {
        let format = FileFormat {
            version: USER_DICT_FORMAT_VERSION,
            entries: dictionary.words_sorted(),
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

    use super::{unix_now, UserDictStore};
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
