//! 词典包 manifest 的签名与验签（M6-U，FR-020、方案设计 11.7）。
//!
//! 信任链（P-04）：对**规范化 JSON** 用发布私钥（ed25519）签名；公钥内置引擎；
//! 客户端先验签，再逐包校验内容 SHA-256（双保险）。
//!
//! 规范化规则：签名对象是 manifest 的确定性序列化结果，而不是磁盘上的原始字节。
//! 这样重排键序、调整缩进都不会破坏签名，同时排除 `signature` 字段自身——
//! 否则签名无法自洽。
//!
//! 密钥管理：私钥只在发布环境使用，**绝不进入仓库**；仓库内只有公钥与测试用
//! 固定密钥对。丢失预案 = 换钥重新发布（包带 `min_engine_version` 约束旧客户端）。

use std::path::Path;

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// manifest schema 版本（与 `zhu-ye-dict` 的 `MANIFEST_SCHEMA` 对齐）。
pub const MANIFEST_SCHEMA: u32 = 1;

/// 签名算法标识；写入 manifest 便于将来换算法。
pub const SIGNATURE_ALGORITHM: &str = "ed25519";

/// 单个词典包的发布元数据。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PackMeta {
    /// 包 id，对应 `<id>.zyct`。
    pub id: String,
    /// 展示名。
    pub name: String,
    /// 包版本（独立于输入法版本，P-06）。
    pub version: String,
    /// 文件名。
    pub file: String,
    /// 内容 SHA-256（小写十六进制）。
    pub sha256: String,
    /// 文件字节数。
    pub size: u64,
    /// 最低引擎版本；高于当前引擎的包应跳过。
    pub min_engine_version: String,
}

/// 发布 manifest（含可选签名块）。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Manifest {
    /// schema 版本。
    pub schema: u32,
    /// 发布时间（ISO-8601）。
    pub published_at: String,
    /// 包列表。
    pub packs: Vec<PackMeta>,
    /// 签名块；未签名 manifest 为 `None`。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<ManifestSignature>,
}

/// manifest 签名块。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ManifestSignature {
    /// 算法标识，当前仅支持 `ed25519`。
    pub algorithm: String,
    /// 公钥（32 字节，十六进制）；便于发布方轮换与审计。
    pub public_key: String,
    /// 签名（64 字节，十六进制）。
    pub value: String,
}

/// 签名与验签错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignatureError {
    /// JSON 序列化/解析失败。
    Json(String),
    /// 十六进制解码失败或长度不符。
    Encoding(String),
    /// 公钥不是合法 ed25519 公钥。
    PublicKey(String),
    /// 签名验证失败。
    VerifyFailed,
    /// manifest 未签名。
    Missing,
    /// 算法不受支持。
    UnsupportedAlgorithm(String),
}

impl std::fmt::Display for SignatureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json(message) => write!(f, "manifest JSON 处理失败: {message}"),
            Self::Encoding(message) => write!(f, "签名编码无效: {message}"),
            Self::PublicKey(message) => write!(f, "公钥无效: {message}"),
            Self::VerifyFailed => write!(f, "manifest 签名验证失败"),
            Self::Missing => write!(f, "manifest 未签名"),
            Self::UnsupportedAlgorithm(name) => write!(f, "不支持的签名算法: {name}"),
        }
    }
}

impl std::error::Error for SignatureError {}

/// 待签名的规范化字节：排除 `signature` 字段后的紧凑 JSON。
///
/// 使用 `serde_json::to_vec`（紧凑、无空白）且字段顺序由结构体声明固定，
/// 因此同一 manifest 在任何平台产出同一字节序列。
pub fn canonical_bytes(manifest: &Manifest) -> Result<Vec<u8>, SignatureError> {
    let unsigned = Manifest {
        signature: None,
        ..manifest.clone()
    };
    serde_json::to_vec(&unsigned).map_err(|error| SignatureError::Json(error.to_string()))
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

fn hex_decode(text: &str) -> Result<Vec<u8>, SignatureError> {
    if !text.len().is_multiple_of(2) {
        return Err(SignatureError::Encoding(format!(
            "十六进制长度必须为偶数: {}",
            text.len()
        )));
    }
    let mut out = Vec::with_capacity(text.len() / 2);
    let bytes = text.as_bytes();
    for pair in bytes.chunks(2) {
        let high = (pair[0] as char)
            .to_digit(16)
            .ok_or_else(|| SignatureError::Encoding("包含非十六进制字符".to_owned()))?;
        let low = (pair[1] as char)
            .to_digit(16)
            .ok_or_else(|| SignatureError::Encoding("包含非十六进制字符".to_owned()))?;
        out.push(((high << 4) | low) as u8);
    }
    Ok(out)
}

/// 生成新的 ed25519 发布密钥对（P-04 信任锚）。
///
/// 返回 `(私钥 seed hex, 公钥 hex)`。私钥只应在发布环境中保存，绝不写入
/// 仓库或提交历史；公钥可入库，并在客户端构建期通过 `ZHU_YE_RELEASE_PUBLIC_KEY`
/// 注入更新器。供 `zhu-ye-dict keygen` 调用。
pub fn generate_keypair() -> Result<(String, String), String> {
    let mut seed = [0u8; 32];
    getrandom::getrandom(&mut seed).map_err(|error| format!("系统熵源不可用: {error}"))?;
    let signing_key = SigningKey::from_bytes(&seed);
    let public = signing_key.verifying_key().to_bytes();
    Ok((hex_encode(&seed), hex_encode(&public)))
}

/// 用私钥对 manifest 签名，返回带签名块的 manifest。
///
/// `secret` 是 32 字节 ed25519 私钥种子；调用方负责从发布环境 secret 读取，
/// 绝不写入仓库或日志。
pub fn sign_manifest(manifest: &Manifest, secret: &[u8; 32]) -> Result<Manifest, SignatureError> {
    let signing_key = SigningKey::from_bytes(secret);
    let canonical = canonical_bytes(manifest)?;
    let signature = signing_key.sign(&canonical);
    let verifying_key = signing_key.verifying_key();
    let mut signed = manifest.clone();
    signed.signature = Some(ManifestSignature {
        algorithm: SIGNATURE_ALGORITHM.to_owned(),
        public_key: hex_encode(verifying_key.as_bytes()),
        value: hex_encode(&signature.to_bytes()),
    });
    Ok(signed)
}

/// 用 manifest 内嵌公钥验签。
pub fn verify_signature(manifest: &Manifest) -> Result<(), SignatureError> {
    let signature = manifest.signature.as_ref().ok_or(SignatureError::Missing)?;
    if signature.algorithm != SIGNATURE_ALGORITHM {
        return Err(SignatureError::UnsupportedAlgorithm(
            signature.algorithm.clone(),
        ));
    }
    let public_bytes = hex_decode(&signature.public_key)?;
    let public_array: [u8; 32] = public_bytes.as_slice().try_into().map_err(|_| {
        SignatureError::PublicKey(format!("长度应为 32 字节，实际 {}", public_bytes.len()))
    })?;
    let verifying_key = VerifyingKey::from_bytes(&public_array)
        .map_err(|error| SignatureError::PublicKey(error.to_string()))?;

    let signature_bytes = hex_decode(&signature.value)?;
    let signature_array: [u8; 64] = signature_bytes.as_slice().try_into().map_err(|_| {
        SignatureError::Encoding(format!(
            "签名长度应为 64 字节，实际 {}",
            signature_bytes.len()
        ))
    })?;
    let parsed = Signature::from_bytes(&signature_array);

    let canonical = canonical_bytes(manifest)?;
    verifying_key
        .verify(&canonical, &parsed)
        .map_err(|_| SignatureError::VerifyFailed)
}

/// 用**内置公钥**验签：manifest 自带的公钥只用于审计，信任锚点是编译进引擎的
/// 这一个。这样即使攻击者替换 manifest 并附上自己的公钥与签名，验签仍会失败。
pub fn verify_signature_with_key(
    manifest: &Manifest,
    trusted_public_key: &VerifyingKey,
) -> Result<(), SignatureError> {
    let signature = manifest.signature.as_ref().ok_or(SignatureError::Missing)?;
    if signature.algorithm != SIGNATURE_ALGORITHM {
        return Err(SignatureError::UnsupportedAlgorithm(
            signature.algorithm.clone(),
        ));
    }
    let signature_bytes = hex_decode(&signature.value)?;
    let signature_array: [u8; 64] = signature_bytes.as_slice().try_into().map_err(|_| {
        SignatureError::Encoding(format!(
            "签名长度应为 64 字节，实际 {}",
            signature_bytes.len()
        ))
    })?;
    let parsed = Signature::from_bytes(&signature_array);
    let canonical = canonical_bytes(manifest)?;
    trusted_public_key
        .verify(&canonical, &parsed)
        .map_err(|_| SignatureError::VerifyFailed)
}

/// 从十六进制文本解析公钥。
pub fn parse_public_key(hex: &str) -> Result<VerifyingKey, SignatureError> {
    let bytes = hex_decode(hex)?;
    let array: [u8; 32] = bytes.as_slice().try_into().map_err(|_| {
        SignatureError::PublicKey(format!("长度应为 32 字节，实际 {}", bytes.len()))
    })?;
    VerifyingKey::from_bytes(&array).map_err(|error| SignatureError::PublicKey(error.to_string()))
}

/// 计算文件内容 SHA-256（小写十六进制）。
pub fn sha256_file(path: &Path) -> Result<String, String> {
    let bytes =
        std::fs::read(path).map_err(|error| format!("读取 {} 失败: {error}", path.display()))?;
    Ok(sha256_hex(&bytes))
}

/// 计算字节内容 SHA-256（小写十六进制）。
#[must_use]
pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex_encode(&hasher.finalize())
}

/// 解析 manifest JSON。
pub fn parse_manifest(text: &str) -> Result<Manifest, SignatureError> {
    serde_json::from_str(text).map_err(|error| SignatureError::Json(error.to_string()))
}

/// 逐包校验内容 SHA-256 与大小；`base_dir` 是包文件所在目录。
///
/// 返回通过校验的包数；任一包不一致即失败（不做部分接受）。
pub fn verify_pack_contents(manifest: &Manifest, base_dir: &Path) -> Result<usize, String> {
    let mut checked = 0usize;
    for pack in &manifest.packs {
        let path = base_dir.join(&pack.file);
        if !path.is_file() {
            return Err(format!("包文件缺失：{}", path.display()));
        }
        let actual = sha256_file(&path)?;
        if !actual.eq_ignore_ascii_case(&pack.sha256) {
            return Err(format!(
                "包 {} 内容哈希不一致：期望 {}，实际 {actual}",
                pack.id, pack.sha256
            ));
        }
        let size = std::fs::metadata(&path)
            .map_err(|error| format!("读取包大小失败: {error}"))?
            .len();
        if size != pack.size {
            return Err(format!(
                "包 {} 大小不一致：期望 {}，实际 {size}",
                pack.id, pack.size
            ));
        }
        checked += 1;
    }
    Ok(checked)
}

/// 比较版本号（点分数字）；`left >= right` 返回 true。
///
/// 仅用于 `min_engine_version` 门槛。每段取**前导数字**（忽略 pre-release
/// 后缀：`0.1.2-alpha` 按 `0.1.2` 参与比较），非数字段按 0 处理，避免因
/// 版本串格式差异误拒可用包——本仓库发布版本带 `alpha`/`beta` 后缀
///（如 `v0.1.2-alpha`），若把 `2-alpha` 整段按 0 解析，客户端会被自己的
/// 发布版本挡在门槛之外（T-094 e2e 发现）。
#[must_use]
pub fn version_at_least(current: &str, required: &str) -> bool {
    fn parts(text: &str) -> Vec<u64> {
        text.split('.')
            .map(|part| {
                let digits: String = part
                    .trim()
                    .chars()
                    .take_while(|c| c.is_ascii_digit())
                    .collect();
                digits.parse::<u64>().unwrap_or(0)
            })
            .collect()
    }
    let left = parts(current);
    let right = parts(required);
    let len = left.len().max(right.len());
    for index in 0..len {
        let a = left.get(index).copied().unwrap_or(0);
        let b = right.get(index).copied().unwrap_or(0);
        if a != b {
            return a > b;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::{
        canonical_bytes, generate_keypair, hex_decode, parse_manifest, parse_public_key,
        sha256_hex, sign_manifest, verify_pack_contents, verify_signature,
        verify_signature_with_key, version_at_least, Manifest, ManifestSignature, PackMeta,
        SignatureError, SIGNATURE_ALGORITHM,
    };
    use ed25519_dalek::SigningKey;
    use std::path::PathBuf;

    fn temp_dir(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "zhu-ye-manifest-{label}-{}-{}",
            std::process::id(),
            crate::unix_now()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn sample_manifest() -> Manifest {
        Manifest {
            schema: 1,
            published_at: "2026-09-29T00:00:00Z".to_owned(),
            packs: vec![PackMeta {
                id: "it".to_owned(),
                name: "IT/编程".to_owned(),
                version: "2026.09.1".to_owned(),
                file: "it.zyct".to_owned(),
                sha256: "abc123".to_owned(),
                size: 1234,
                min_engine_version: "0.2.0".to_owned(),
            }],
            signature: None,
        }
    }

    /// 测试固定私钥；仅用于单测，绝不用于发布。
    fn test_secret() -> [u8; 32] {
        [7u8; 32]
    }

    #[test]
    fn 规范化字节排除签名字段且确定() {
        let mut manifest = sample_manifest();
        let before = canonical_bytes(&manifest).unwrap();
        // 加上签名块后，待签名字节必须不变（否则签名无法自洽）。
        manifest.signature = Some(ManifestSignature {
            algorithm: SIGNATURE_ALGORITHM.to_owned(),
            public_key: "00".repeat(32),
            value: "00".repeat(64),
        });
        let after = canonical_bytes(&manifest).unwrap();
        assert_eq!(before, after, "规范化字节必须排除 signature 字段");
        assert!(!String::from_utf8_lossy(&before).contains("signature"));
    }

    #[test]
    fn 签名后可用内嵌公钥验签() {
        let signed = sign_manifest(&sample_manifest(), &test_secret()).unwrap();
        assert!(signed.signature.is_some());
        verify_signature(&signed).unwrap();
    }

    #[test]
    fn 篡改包哈希后验签失败() {
        let mut signed = sign_manifest(&sample_manifest(), &test_secret()).unwrap();
        signed.packs[0].sha256 = "tampered".to_owned();
        assert_eq!(verify_signature(&signed), Err(SignatureError::VerifyFailed));
    }

    #[test]
    fn 篡改版本后验签失败() {
        let mut signed = sign_manifest(&sample_manifest(), &test_secret()).unwrap();
        signed.packs[0].version = "9999".to_owned();
        assert_eq!(verify_signature(&signed), Err(SignatureError::VerifyFailed));
    }

    #[test]
    fn 未签名manifest报缺失() {
        assert_eq!(
            verify_signature(&sample_manifest()),
            Err(SignatureError::Missing)
        );
    }

    #[test]
    fn 替换公钥与签名仍被内置公钥拒绝() {
        // 攻击者用自己的密钥重签，manifest 内嵌的也是攻击者公钥。
        let attacker = SigningKey::from_bytes(&[9u8; 32]);
        let forged = sign_manifest(&sample_manifest(), &[9u8; 32]).unwrap();

        // 自洽的伪造签名能通过"内嵌公钥"验证——这正是内嵌公钥不能作信任锚点的原因。
        verify_signature(&forged).unwrap();
        assert_eq!(
            forged.signature.as_ref().unwrap().public_key,
            hex_of(attacker.verifying_key().as_bytes()),
            "manifest 内嵌的应是攻击者公钥"
        );

        // 内置公钥必须拒绝它。
        let trusted = SigningKey::from_bytes(&test_secret()).verifying_key();
        assert_eq!(
            verify_signature_with_key(&forged, &trusted),
            Err(SignatureError::VerifyFailed),
            "内置公钥必须拒绝攻击者签名的 manifest"
        );
        assert_ne!(
            forged.signature.as_ref().unwrap().public_key,
            hex_of(trusted.as_bytes()),
            "攻击者公钥必须与内置公钥不同"
        );
    }

    fn hex_of(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }

    #[test]
    fn 不支持算法被拒绝() {
        let mut signed = sign_manifest(&sample_manifest(), &test_secret()).unwrap();
        signed.signature.as_mut().unwrap().algorithm = "rsa".to_owned();
        assert_eq!(
            verify_signature(&signed),
            Err(SignatureError::UnsupportedAlgorithm("rsa".to_owned()))
        );
    }

    #[test]
    fn 非法十六进制与长度被拒绝() {
        let mut signed = sign_manifest(&sample_manifest(), &test_secret()).unwrap();
        signed.signature.as_mut().unwrap().value = "zz".to_owned();
        assert!(matches!(
            verify_signature(&signed),
            Err(SignatureError::Encoding(_))
        ));

        let mut short = sign_manifest(&sample_manifest(), &test_secret()).unwrap();
        short.signature.as_mut().unwrap().value = "abcd".to_owned();
        assert!(matches!(
            verify_signature(&short),
            Err(SignatureError::Encoding(_))
        ));

        let mut bad_key = sign_manifest(&sample_manifest(), &test_secret()).unwrap();
        bad_key.signature.as_mut().unwrap().public_key = "00".to_owned();
        assert!(matches!(
            verify_signature(&bad_key),
            Err(SignatureError::PublicKey(_))
        ));
    }

    #[test]
    fn 公钥解析往返() {
        let key = SigningKey::from_bytes(&test_secret()).verifying_key();
        let parsed = parse_public_key(&hex_of(key.as_bytes())).unwrap();
        assert_eq!(parsed.as_bytes(), key.as_bytes());
        assert!(parse_public_key("nothex").is_err());
    }

    #[test]
    fn 解析manifest含与不含签名() {
        let unsigned = serde_json::to_string(&sample_manifest()).unwrap();
        let parsed = parse_manifest(&unsigned).unwrap();
        assert!(parsed.signature.is_none());

        let signed = sign_manifest(&sample_manifest(), &test_secret()).unwrap();
        let text = serde_json::to_string(&signed).unwrap();
        let reparsed = parse_manifest(&text).unwrap();
        assert_eq!(reparsed, signed);
        verify_signature(&reparsed).unwrap();
    }

    #[test]
    fn 非法json解析失败() {
        assert!(matches!(
            parse_manifest("{ not json"),
            Err(SignatureError::Json(_))
        ));
    }

    #[test]
    fn 内容哈希校验通过并拒绝篡改() {
        let dir = temp_dir("content");
        let pack = dir.join("it.zyct");
        let bytes = b"pack-bytes".to_vec();
        std::fs::write(&pack, &bytes).unwrap();

        let mut manifest = sample_manifest();
        manifest.packs[0].file = "it.zyct".to_owned();
        manifest.packs[0].sha256 = sha256_hex(&bytes);
        manifest.packs[0].size = bytes.len() as u64;
        assert_eq!(verify_pack_contents(&manifest, &dir).unwrap(), 1);

        // 篡改内容 → 哈希不一致。
        std::fs::write(&pack, b"tampered").unwrap();
        assert!(verify_pack_contents(&manifest, &dir).is_err());

        // 大小不一致也失败。
        std::fs::write(&pack, &bytes).unwrap();
        manifest.packs[0].size = 999;
        assert!(verify_pack_contents(&manifest, &dir).is_err());

        // 缺失文件失败。
        manifest.packs[0].size = bytes.len() as u64;
        manifest.packs[0].file = "absent.zyct".to_owned();
        assert!(verify_pack_contents(&manifest, &dir).is_err());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn 版本门槛比较() {
        assert!(version_at_least("0.2.0", "0.2.0"));
        assert!(version_at_least("0.3.0", "0.2.0"));
        assert!(version_at_least("1.0.0", "0.9.9"));
        assert!(!version_at_least("0.1.0", "0.2.0"));
        assert!(!version_at_least("0.2.0", "0.2.1"));
        // 段数不同按 0 补齐。
        assert!(version_at_least("1.0.0", "1"));
        assert!(version_at_least("1", "1.0.0"));
        // 非数字段按 0 处理，不误拒。
        assert!(version_at_least("0.2.0", "0.2"));
        // pre-release 后缀取前导数字段（T-094：发布版本带 alpha/beta，
        // 整段按 0 会把客户端挡在自己的版本门槛之外）。
        assert!(version_at_least("0.1.2-alpha", "0.1.1"));
        assert!(version_at_least("0.1.2-beta.1", "0.1.2"));
        assert!(!version_at_least("0.1.1-alpha", "0.1.2"));
        assert!(version_at_least("1.0.0-alpha", "0.9.9"));
        assert!(version_at_least("0.1.2", "0.1.2-alpha"));
        assert!(!version_at_least("0.1.2-alpha", "0.1.3"));
    }

    #[test]
    fn keypair生成格式与派生一致() {
        let (secret, public) = generate_keypair().unwrap();
        assert_eq!(secret.len(), 64, "私钥应为 32 字节 hex");
        assert_eq!(public.len(), 64, "公钥应为 32 字节 hex");
        // 私钥种子派生的公钥与返回的公钥一致。
        let seed_bytes = hex_decode(&secret).unwrap();
        let seed_array: [u8; 32] = seed_bytes.try_into().unwrap();
        let derived = SigningKey::from_bytes(&seed_array)
            .verifying_key()
            .to_bytes();
        assert_eq!(hex_of(&derived), public);
        // 两次调用产生不同密钥对。
        let (secret2, _) = generate_keypair().unwrap();
        assert_ne!(secret, secret2);
    }
}
