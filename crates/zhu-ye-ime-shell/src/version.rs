//! 版本指针的读写：决定壳当前加载哪个版本化实现。
//!
//! 指针文件 `<InstallDir>\versions\current.txt` 内容为 8 位 sha8 版本 id
//! 加换行。写入指针即「发布」——所有已打开的宿主在下一次获得输入焦点时
//! （`Activate`）会读到新指针并换用新实现。
//!
//! 指针文件缺失或内容非法时，壳退回「就地实现」模式：直接加载与壳同目录的
//! 同名实现 DLL（便携开发目录的用法），保证壳永不因指针缺失而不可用。

use std::path::{Path, PathBuf};

/// 指针文件名（位于 `versions\` 下）。
pub const POINTER_FILE_NAME: &str = "current.txt";

/// 实现 DLL 的文件名（位于 `versions\<id>\` 下）。
pub const IMPL_FILE_NAME: &str = "zhu-ye-impl.dll";

/// 版本 id 的字符集：8 位十六进制 sha8。
const ID_LEN: usize = 8;

/// 读出版本指针。
///
/// 返回 `None` 表示「无有效指针」（文件缺失、为空、长度不符或含非十六进制
/// 字符）。调用方据此退回就地实现模式，而不是把错误版本号当成有效版本。
#[must_use]
pub fn read_pointer(versions_dir: &Path) -> Option<String> {
    let path = versions_dir.join(POINTER_FILE_NAME);
    let raw = std::fs::read_to_string(path).ok()?;
    let id = raw.trim();
    if id.len() != ID_LEN || !id.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    Some(id.to_ascii_lowercase())
}

/// 原子写出版本指针（先写临时文件再改名，避免读到半截内容）。
///
/// 写失败返回错误信息而不 panic：发布失败必须让调用方能回滚到旧版本。
pub fn write_pointer(versions_dir: &Path, id: &str) -> Result<(), String> {
    let id = id.trim().to_ascii_lowercase();
    if id.len() != ID_LEN || !id.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(format!("版本 id 非法: {id}"));
    }
    std::fs::create_dir_all(versions_dir)
        .map_err(|error| format!("创建 {} 失败: {error}", versions_dir.display()))?;
    let path = versions_dir.join(POINTER_FILE_NAME);
    let tmp = versions_dir.join(format!("{POINTER_FILE_NAME}.staging"));
    std::fs::write(&tmp, format!("{id}\n"))
        .map_err(|error| format!("写 {} 失败: {error}", tmp.display()))?;
    std::fs::rename(&tmp, &path)
        .map_err(|error| format!("替换 {} 失败: {error}", path.display()))?;
    Ok(())
}

/// 版本化实现 DLL 的完整路径。
#[must_use]
pub fn impl_path(versions_dir: &Path, id: &str) -> PathBuf {
    versions_dir.join(id).join(IMPL_FILE_NAME)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("zy-shell-test-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("建临时目录");
        dir
    }

    #[test]
    fn 指针缺失返回无() {
        let dir = temp_dir("missing");
        assert!(read_pointer(&dir).is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn 指针写入后可读回() {
        let dir = temp_dir("roundtrip");
        let versions = dir.join("versions");
        write_pointer(&versions, "147501ad").expect("写指针");
        assert_eq!(read_pointer(&versions).as_deref(), Some("147501ad"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn 非法版本号被拒绝且不写盘() {
        let dir = temp_dir("invalid");
        let versions = dir.join("versions");
        assert!(write_pointer(&versions, "xyz").is_err());
        assert!(write_pointer(&versions, "147501a").is_err());
        assert!(write_pointer(&versions, "147501adZ").is_err());
        assert!(read_pointer(&versions).is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn 磁盘上的非法内容被当作无指针() {
        let dir = temp_dir("garbage");
        let versions = dir.join("versions");
        std::fs::create_dir_all(&versions).expect("建目录");
        for content in ["", "  ", "not-hex", "147501ad extra", "147501a"] {
            std::fs::write(versions.join(POINTER_FILE_NAME), content).expect("写脏数据");
            assert!(
                read_pointer(&versions).is_none(),
                "内容 {content:?} 应被判为无效"
            );
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn 实现路径落在版本子目录下() {
        let dir = temp_dir("implpath");
        let path = impl_path(&dir, "147501ad");
        // 逐段核对：版本目录是实现文件的直接父目录。
        assert_eq!(
            path.parent().and_then(std::path::Path::file_name),
            Some(std::ffi::OsStr::new("147501ad")),
            "实现文件应直接位于版本目录下：{path:?}"
        );
        assert_eq!(
            path.file_name(),
            Some(std::ffi::OsStr::new(IMPL_FILE_NAME)),
            "实现文件名应为 {IMPL_FILE_NAME}：{path:?}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
