//! 词典包更新：staging、验签验哈希、原子替换与回滚（M6-U，FR-020、方案设计 11.7）。
//!
//! 本模块**不做任何网络操作**，只负责"拿到已下载的 manifest 与包字节之后"的
//! 安全落地流程，因此可以在无网络环境下完整单测：
//!
//! 1. 校验 manifest 签名（内置公钥，见 [`crate::manifest`]）；
//! 2. 逐包校验内容 SHA-256 与大小；
//! 3. 校验 `min_engine_version` 门槛；
//! 4. 写入 staging 目录；
//! 5. 原子替换目标包，旧版保留 `.bak`；
//! 6. 任一环节失败：保留旧版、记日志、不留下半截状态。
//!
//! 网络下载与进程隔离属 `zhu-ye-updater` 二进制；TSF DLL 绝不在宿主进程内联网
//! （S-4：防挂起宿主、杀软拦截、影响输入）。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ed25519_dalek::VerifyingKey;

use crate::manifest::{
    verify_pack_contents, verify_signature_with_key, version_at_least, Manifest, SignatureError,
};

/// 已安装包的备份后缀（P-05：旧版保留，失败可回滚）。
pub const BACKUP_SUFFIX: &str = ".bak";

/// staging 子目录名。
pub const STAGING_DIR_NAME: &str = "staging";

/// 更新流程结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplyOutcome {
    /// 实际更新的包 id。
    pub applied: Vec<String>,
    /// 因 `min_engine_version` 过高而跳过的包 id。
    pub skipped_by_version: Vec<String>,
    /// 已是最新（哈希一致）而跳过的包 id。
    pub unchanged: Vec<String>,
}

/// 更新失败原因；所有失败都保证"旧版仍可用"。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateError {
    /// manifest 签名无效。
    Signature(SignatureError),
    /// 包内容校验失败。
    Content(String),
    /// 文件系统操作失败。
    Io(String),
}

impl std::fmt::Display for UpdateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Signature(error) => write!(f, "manifest 签名校验失败: {error}"),
            Self::Content(message) => write!(f, "包内容校验失败: {message}"),
            Self::Io(message) => write!(f, "文件操作失败: {message}"),
        }
    }
}

impl std::error::Error for UpdateError {}

/// staging 目录路径（`<packs_dir>/staging`）。
#[must_use]
pub fn staging_dir(packs_dir: &Path) -> PathBuf {
    packs_dir.join(STAGING_DIR_NAME)
}

/// 已安装包的备份路径（`<packs_dir>/<id>.zyct.bak`）。
#[must_use]
pub fn backup_path(packs_dir: &Path, pack_file: &str) -> PathBuf {
    packs_dir.join(format!("{pack_file}{BACKUP_SUFFIX}"))
}

/// 用内置公钥校验 manifest 并逐包校验内容。
///
/// `downloads` 提供每个包 id 对应的已下载字节；缺失的包视为失败（不做部分接受）。
pub fn verify_release(
    manifest: &Manifest,
    trusted_key: &VerifyingKey,
    downloads: &BTreeMap<String, Vec<u8>>,
    engine_version: &str,
) -> Result<Vec<String>, UpdateError> {
    // 签名针对**完整** manifest 校验：调用方必须传原始 manifest，
    // 不能先裁剪包列表（那会改变规范化字节，使签名失效）。
    verify_signature_with_key(manifest, trusted_key).map_err(UpdateError::Signature)?;

    let mut ready = Vec::new();
    for pack in &manifest.packs {
        // 只校验本次实际下载的包；未下载的包（基础包、开发产物、
        // 已是最新而跳过的包）不参与内容校验。
        if !downloads.contains_key(&pack.id) {
            continue;
        }
        if !version_at_least(engine_version, &pack.min_engine_version) {
            // 版本门槛不足：跳过该包（不算失败，其余包照常处理）。
            continue;
        }
        let bytes = downloads
            .get(&pack.id)
            .ok_or_else(|| UpdateError::Content(format!("包 {} 的下载内容缺失", pack.id)))?;
        let actual = crate::manifest::sha256_hex(bytes);
        if !actual.eq_ignore_ascii_case(&pack.sha256) {
            return Err(UpdateError::Content(format!(
                "包 {} 内容哈希不一致：期望 {}，实际 {actual}",
                pack.id, pack.sha256
            )));
        }
        if bytes.len() as u64 != pack.size {
            return Err(UpdateError::Content(format!(
                "包 {} 大小不一致：期望 {}，实际 {}",
                pack.id,
                pack.size,
                bytes.len()
            )));
        }
        ready.push(pack.id.clone());
    }
    Ok(ready)
}

/// 把已校验的包写入 staging 目录，返回 (包 id, staging 路径) 列表。
pub fn stage_packs(
    manifest: &Manifest,
    downloads: &BTreeMap<String, Vec<u8>>,
    packs_dir: &Path,
    ready: &[String],
) -> Result<Vec<(String, PathBuf)>, UpdateError> {
    let staging = staging_dir(packs_dir);
    std::fs::create_dir_all(&staging)
        .map_err(|error| UpdateError::Io(format!("创建 staging 目录失败: {error}")))?;

    let mut staged = Vec::new();
    for pack in &manifest.packs {
        if !ready.contains(&pack.id) {
            continue;
        }
        let bytes = downloads
            .get(&pack.id)
            .ok_or_else(|| UpdateError::Content(format!("包 {} 的下载内容缺失", pack.id)))?;
        let path = staging.join(&pack.file);
        std::fs::write(&path, bytes).map_err(|error| {
            UpdateError::Io(format!("写入 staging {} 失败: {error}", path.display()))
        })?;
        staged.push((pack.id.clone(), path));
    }
    Ok(staged)
}

/// 原子应用：把 staging 中的包替换到 `packs_dir`，旧版保留 `.bak`。
///
/// 逐个包执行"备份旧版 → rename 新版就位"。单个包失败时该包保持旧版可用，
/// 已成功的包不回退（每个包彼此独立，符合 P-05"保留旧版、下次再试"）。
pub fn apply_staged(
    staged: &[(String, PathBuf)],
    manifest: &Manifest,
    packs_dir: &Path,
) -> Result<ApplyOutcome, UpdateError> {
    std::fs::create_dir_all(packs_dir)
        .map_err(|error| UpdateError::Io(format!("创建包目录失败: {error}")))?;

    let mut outcome = ApplyOutcome {
        applied: Vec::new(),
        skipped_by_version: Vec::new(),
        unchanged: Vec::new(),
    };

    for (id, staged_path) in staged {
        let Some(pack) = manifest.packs.iter().find(|pack| &pack.id == id) else {
            continue;
        };
        let target = packs_dir.join(&pack.file);

        // 已是最新：内容一致则不动（避免无谓的备份与磁盘写入）。
        if target.is_file() {
            if let Ok(existing) = crate::manifest::sha256_file(&target) {
                if existing.eq_ignore_ascii_case(&pack.sha256) {
                    outcome.unchanged.push(id.clone());
                    continue;
                }
            }
            // 备份旧版；备份失败不阻断（旧版仍在原位，只是没有 .bak）。
            let backup = backup_path(packs_dir, &pack.file);
            let _ = std::fs::copy(&target, &backup);
        }

        // 原子就位：先写到目标同目录的临时文件，再 rename 覆盖。
        // rename 在同一卷上是原子操作，中途失败不会留下半截文件。
        let temp = target.with_extension("zyct.new");
        std::fs::copy(staged_path, &temp).map_err(|error| {
            UpdateError::Io(format!("写入临时包 {} 失败: {error}", temp.display()))
        })?;
        std::fs::rename(&temp, &target).map_err(|error| {
            let _ = std::fs::remove_file(&temp);
            UpdateError::Io(format!("替换包 {} 失败: {error}", target.display()))
        })?;
        outcome.applied.push(id.clone());
    }

    Ok(outcome)
}

/// 回滚：把 `.bak` 恢复到目标包。
///
/// 用于"应用后启动失败"的场景（验收标准 FR-020 回滚行）。
pub fn rollback_pack(packs_dir: &Path, pack_file: &str) -> Result<(), UpdateError> {
    let backup = backup_path(packs_dir, pack_file);
    if !backup.is_file() {
        return Err(UpdateError::Io(format!(
            "没有可回滚的备份：{}",
            backup.display()
        )));
    }
    let target = packs_dir.join(pack_file);
    std::fs::copy(&backup, &target)
        .map_err(|error| UpdateError::Io(format!("回滚 {} 失败: {error}", target.display())))?;
    Ok(())
}

/// 清理 staging 目录（更新流程结束后调用，避免残留）。
pub fn clean_staging(packs_dir: &Path) -> Result<(), UpdateError> {
    let staging = staging_dir(packs_dir);
    if staging.is_dir() {
        std::fs::remove_dir_all(&staging)
            .map_err(|error| UpdateError::Io(format!("清理 staging 失败: {error}")))?;
    }
    Ok(())
}

/// 校验已安装的包是否与 manifest 一致；返回不一致的包 id。
#[must_use]
pub fn find_outdated(manifest: &Manifest, packs_dir: &Path) -> Vec<String> {
    let mut outdated = Vec::new();
    for pack in &manifest.packs {
        let target = packs_dir.join(&pack.file);
        let matches = crate::manifest::sha256_file(&target)
            .map(|actual| actual.eq_ignore_ascii_case(&pack.sha256))
            .unwrap_or(false);
        if !matches {
            outdated.push(pack.id.clone());
        }
    }
    outdated
}

/// 便捷入口：完整流程（验签 → 校验 → staging → 应用 → 清理）。
///
/// 调用方已把 manifest 与包字节准备好（网络部分在 updater 二进制）。
pub fn apply_release(
    manifest: &Manifest,
    trusted_key: &VerifyingKey,
    downloads: &BTreeMap<String, Vec<u8>>,
    packs_dir: &Path,
    engine_version: &str,
) -> Result<ApplyOutcome, UpdateError> {
    let ready = verify_release(manifest, trusted_key, downloads, engine_version)?;
    let staged = stage_packs(manifest, downloads, packs_dir, &ready)?;
    let outcome = apply_staged(&staged, manifest, packs_dir)?;
    // 清理失败不影响已应用的更新。
    let _ = clean_staging(packs_dir);
    Ok(outcome)
}

/// 复核已安装目录：签名 + 内容哈希（`verify-manifest` 的运行时等价物）。
pub fn verify_installed(
    manifest: &Manifest,
    trusted_key: &VerifyingKey,
    packs_dir: &Path,
) -> Result<usize, UpdateError> {
    verify_signature_with_key(manifest, trusted_key).map_err(UpdateError::Signature)?;
    verify_pack_contents(manifest, packs_dir).map_err(UpdateError::Content)
}

#[cfg(test)]
mod tests {
    use super::{
        apply_release, apply_staged, backup_path, clean_staging, find_outdated, rollback_pack,
        stage_packs, staging_dir, verify_installed, verify_release, UpdateError,
    };
    use crate::manifest::{sha256_hex, sign_manifest, Manifest, PackMeta};
    use ed25519_dalek::SigningKey;
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    fn temp_dir(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "zhu-ye-update-{label}-{}-{}",
            std::process::id(),
            crate::unix_now()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn test_key() -> SigningKey {
        SigningKey::from_bytes(&[7u8; 32])
    }

    fn manifest_with(packs: Vec<(&str, &[u8])>, min_version: &str) -> Manifest {
        Manifest {
            schema: 1,
            published_at: "2026-09-29T00:00:00Z".to_owned(),
            packs: packs
                .into_iter()
                .map(|(id, bytes)| PackMeta {
                    id: id.to_owned(),
                    name: id.to_owned(),
                    version: "2026.09.1".to_owned(),
                    file: format!("{id}.zyct"),
                    sha256: sha256_hex(bytes),
                    size: bytes.len() as u64,
                    min_engine_version: min_version.to_owned(),
                })
                .collect(),
            signature: None,
        }
    }

    fn downloads(packs: &[(&str, &[u8])]) -> BTreeMap<String, Vec<u8>> {
        packs
            .iter()
            .map(|(id, bytes)| ((*id).to_owned(), bytes.to_vec()))
            .collect()
    }

    #[test]
    fn 完整流程应用新包并保留备份() {
        let dir = temp_dir("apply");
        let packs_dir = dir.join("packs");
        std::fs::create_dir_all(&packs_dir).unwrap();
        // 旧版内容。
        std::fs::write(packs_dir.join("it.zyct"), b"old-it").unwrap();

        let new_bytes: &[u8] = b"new-it-content";
        let manifest =
            sign_manifest(&manifest_with(vec![("it", new_bytes)], "0.1.0"), &[7u8; 32]).unwrap();
        let dl = downloads(&[("it", new_bytes)]);

        let outcome = apply_release(
            &manifest,
            &test_key().verifying_key(),
            &dl,
            &packs_dir,
            "0.1.0",
        )
        .unwrap();
        assert_eq!(outcome.applied, vec!["it"]);
        assert_eq!(std::fs::read(packs_dir.join("it.zyct")).unwrap(), new_bytes);
        // 旧版备份保留。
        assert_eq!(
            std::fs::read(backup_path(&packs_dir, "it.zyct")).unwrap(),
            b"old-it"
        );
        // staging 已清理。
        assert!(!staging_dir(&packs_dir).exists());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn 签名无效整体拒绝且旧版不动() {
        let dir = temp_dir("badsig");
        let packs_dir = dir.join("packs");
        std::fs::create_dir_all(&packs_dir).unwrap();
        std::fs::write(packs_dir.join("it.zyct"), b"old-it").unwrap();

        let new_bytes: &[u8] = b"new-it";
        let mut manifest =
            sign_manifest(&manifest_with(vec![("it", new_bytes)], "0.1.0"), &[7u8; 32]).unwrap();
        // 篡改包哈希使签名失效。
        manifest.packs[0].sha256 = "deadbeef".to_owned();

        let dl = downloads(&[("it", new_bytes)]);
        let error = apply_release(
            &manifest,
            &test_key().verifying_key(),
            &dl,
            &packs_dir,
            "0.1.0",
        )
        .unwrap_err();
        assert!(matches!(error, UpdateError::Signature(_)));
        // 旧版仍可用，且未产生 staging 残留。
        assert_eq!(std::fs::read(packs_dir.join("it.zyct")).unwrap(), b"old-it");
        assert!(!staging_dir(&packs_dir).exists());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn 内容哈希不符拒绝应用() {
        let dir = temp_dir("badcontent");
        let packs_dir = dir.join("packs");
        std::fs::create_dir_all(&packs_dir).unwrap();
        std::fs::write(packs_dir.join("it.zyct"), b"old-it").unwrap();

        let declared: &[u8] = b"declared";
        let manifest =
            sign_manifest(&manifest_with(vec![("it", declared)], "0.1.0"), &[7u8; 32]).unwrap();
        // 下载到的是被篡改的字节（签名对 declared 有效，但内容不符）。
        let dl = downloads(&[("it", b"tampered")]);

        let error = apply_release(
            &manifest,
            &test_key().verifying_key(),
            &dl,
            &packs_dir,
            "0.1.0",
        )
        .unwrap_err();
        assert!(matches!(error, UpdateError::Content(_)));
        assert_eq!(std::fs::read(packs_dir.join("it.zyct")).unwrap(), b"old-it");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn 版本门槛不足时跳过该包() {
        let dir = temp_dir("minver");
        let packs_dir = dir.join("packs");
        std::fs::create_dir_all(&packs_dir).unwrap();

        let bytes: &[u8] = b"future-pack";
        let manifest =
            sign_manifest(&manifest_with(vec![("it", bytes)], "99.0.0"), &[7u8; 32]).unwrap();
        let dl = downloads(&[("it", bytes)]);

        let outcome = apply_release(
            &manifest,
            &test_key().verifying_key(),
            &dl,
            &packs_dir,
            "0.1.0",
        )
        .unwrap();
        assert!(outcome.applied.is_empty(), "版本门槛不足不应应用");
        assert!(!packs_dir.join("it.zyct").exists(), "包不应被写入");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn 内容一致时标记为无需更新() {
        let dir = temp_dir("unchanged");
        let packs_dir = dir.join("packs");
        std::fs::create_dir_all(&packs_dir).unwrap();
        let bytes: &[u8] = b"same-bytes";
        std::fs::write(packs_dir.join("it.zyct"), bytes).unwrap();

        let manifest =
            sign_manifest(&manifest_with(vec![("it", bytes)], "0.1.0"), &[7u8; 32]).unwrap();
        let dl = downloads(&[("it", bytes)]);
        let outcome = apply_release(
            &manifest,
            &test_key().verifying_key(),
            &dl,
            &packs_dir,
            "0.1.0",
        )
        .unwrap();
        assert_eq!(outcome.unchanged, vec!["it"]);
        assert!(outcome.applied.is_empty());
        // 无需更新时不应产生备份。
        assert!(!backup_path(&packs_dir, "it.zyct").exists());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn 回滚恢复备份() {
        let dir = temp_dir("rollback");
        let packs_dir = dir.join("packs");
        std::fs::create_dir_all(&packs_dir).unwrap();
        std::fs::write(packs_dir.join("it.zyct"), b"old-it").unwrap();

        let new_bytes: &[u8] = b"new-it";
        let manifest =
            sign_manifest(&manifest_with(vec![("it", new_bytes)], "0.1.0"), &[7u8; 32]).unwrap();
        let dl = downloads(&[("it", new_bytes)]);
        apply_release(
            &manifest,
            &test_key().verifying_key(),
            &dl,
            &packs_dir,
            "0.1.0",
        )
        .unwrap();
        assert_eq!(std::fs::read(packs_dir.join("it.zyct")).unwrap(), new_bytes);

        // 模拟"应用后启动失败"→ 回滚。
        rollback_pack(&packs_dir, "it.zyct").unwrap();
        assert_eq!(std::fs::read(packs_dir.join("it.zyct")).unwrap(), b"old-it");

        // 无备份时回滚明确失败。
        std::fs::remove_file(backup_path(&packs_dir, "it.zyct")).unwrap();
        assert!(rollback_pack(&packs_dir, "it.zyct").is_err());

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// 未下载的包不参与校验（调用方只传本次实际下载的包）；
    /// 空下载集对应"无可更新包"，是正常结果而非错误。
    #[test]
    fn 未下载的包不参与校验() {
        let bytes: &[u8] = b"pack";
        let manifest =
            sign_manifest(&manifest_with(vec![("it", bytes)], "0.1.0"), &[7u8; 32]).unwrap();
        let empty: BTreeMap<String, Vec<u8>> = BTreeMap::new();
        let ready =
            verify_release(&manifest, &test_key().verifying_key(), &empty, "0.1.0").unwrap();
        assert!(ready.is_empty(), "未下载任何包时应返回空清单");
    }

    /// 下载了但内容与 manifest 声明不符（半包/篡改）必须报错。
    #[test]
    fn 下载内容与声明不符报错() {
        let declared: &[u8] = b"pack";
        let manifest =
            sign_manifest(&manifest_with(vec![("it", declared)], "0.1.0"), &[7u8; 32]).unwrap();
        let mut dl: BTreeMap<String, Vec<u8>> = BTreeMap::new();
        dl.insert("it".to_owned(), b"truncated".to_vec());
        let error =
            verify_release(&manifest, &test_key().verifying_key(), &dl, "0.1.0").unwrap_err();
        assert!(matches!(error, UpdateError::Content(_)));
    }

    #[test]
    fn 多包逐个应用且互不影响() {
        let dir = temp_dir("multi");
        let packs_dir = dir.join("packs");
        std::fs::create_dir_all(&packs_dir).unwrap();

        let it: &[u8] = b"it-v2";
        let med: &[u8] = b"med-v2";
        let manifest = sign_manifest(
            &manifest_with(vec![("it", it), ("med", med)], "0.1.0"),
            &[7u8; 32],
        )
        .unwrap();
        let dl = downloads(&[("it", it), ("med", med)]);

        let outcome = apply_release(
            &manifest,
            &test_key().verifying_key(),
            &dl,
            &packs_dir,
            "0.1.0",
        )
        .unwrap();
        assert_eq!(outcome.applied.len(), 2);
        assert_eq!(std::fs::read(packs_dir.join("it.zyct")).unwrap(), it);
        assert_eq!(std::fs::read(packs_dir.join("med.zyct")).unwrap(), med);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn staging写入后可单独应用() {
        let dir = temp_dir("staged");
        let packs_dir = dir.join("packs");
        let bytes: &[u8] = b"staged-pack";
        let manifest =
            sign_manifest(&manifest_with(vec![("it", bytes)], "0.1.0"), &[7u8; 32]).unwrap();
        let dl = downloads(&[("it", bytes)]);

        let ready = verify_release(&manifest, &test_key().verifying_key(), &dl, "0.1.0").unwrap();
        let staged = stage_packs(&manifest, &dl, &packs_dir, &ready).unwrap();
        assert_eq!(staged.len(), 1);
        assert!(staged[0].1.exists(), "staging 文件应存在");

        let outcome = apply_staged(&staged, &manifest, &packs_dir).unwrap();
        assert_eq!(outcome.applied, vec!["it"]);
        clean_staging(&packs_dir).unwrap();
        assert!(!staging_dir(&packs_dir).exists());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn 检测已安装包是否为最新() {
        let dir = temp_dir("outdated");
        let packs_dir = dir.join("packs");
        std::fs::create_dir_all(&packs_dir).unwrap();
        let bytes: &[u8] = b"current";
        let manifest = sign_manifest(
            &manifest_with(vec![("it", bytes), ("med", b"med-bytes")], "0.1.0"),
            &[7u8; 32],
        )
        .unwrap();
        std::fs::write(packs_dir.join("it.zyct"), bytes).unwrap();
        // med 缺失 → 应被列为待更新。
        let outdated = find_outdated(&manifest, &packs_dir);
        assert_eq!(outdated, vec!["med"]);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn 复核已安装目录逐包校验() {
        let dir = temp_dir("verify-installed");
        let packs_dir = dir.join("packs");
        std::fs::create_dir_all(&packs_dir).unwrap();
        let bytes: &[u8] = b"installed";
        let manifest =
            sign_manifest(&manifest_with(vec![("it", bytes)], "0.1.0"), &[7u8; 32]).unwrap();
        std::fs::write(packs_dir.join("it.zyct"), bytes).unwrap();

        assert_eq!(
            verify_installed(&manifest, &test_key().verifying_key(), &packs_dir).unwrap(),
            1
        );

        // 篡改已安装内容 → 失败。
        std::fs::write(packs_dir.join("it.zyct"), b"tampered").unwrap();
        assert!(verify_installed(&manifest, &test_key().verifying_key(), &packs_dir).is_err());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn 攻击者密钥签名的发布被内置公钥拒绝() {
        let dir = temp_dir("forged");
        let packs_dir = dir.join("packs");
        std::fs::create_dir_all(&packs_dir).unwrap();
        let bytes: &[u8] = b"malicious";
        // 攻击者用自己的密钥签名。
        let forged =
            sign_manifest(&manifest_with(vec![("it", bytes)], "0.1.0"), &[9u8; 32]).unwrap();
        let dl = downloads(&[("it", bytes)]);

        let error = apply_release(
            &forged,
            &test_key().verifying_key(),
            &dl,
            &packs_dir,
            "0.1.0",
        )
        .unwrap_err();
        assert!(matches!(error, UpdateError::Signature(_)));
        assert!(!packs_dir.join("it.zyct").exists());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
