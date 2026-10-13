//! 输入法应用版本元数据（T-154）：独立于词典 manifest 的 `app.json`。
//!
//! 为什么独立成文件而不并入 `manifest.json`：manifest 的签名是对**规范化
//! JSON**（结构体字段序序列化）计算的，客户端按其类型反序列化后重新规范化。
//! 若在 manifest 里加新字段，**旧版**客户端解析时会丢掉未知字段，重新规范化
//! 的结果与发布方不一致 → 验签失败 → 旧版用户的所有更新检查报"签名验证失败"。
//! 单独一个 `app.json`（全新文件，旧版更新器根本不读取）则天然向后兼容。
//!
//! 信任链与 manifest 完全同构（P-04）：对规范化 JSON 用发布私钥（ed25519）
//! 签名，客户端用**内置公钥**验签（文件自带公钥只用于审计）。`app.json`
//! 与 `manifest.json` 同放在镜像更新仓库的 release 资产里。
//!
//! 签名机制完全对称：
//! - 发布：`zhu-ye-dict sign-app <app.json>`（读无签名文件 → 签名 → 写回）；
//! - 检查：更新器拉取 → `parse_app_meta` → `AppMeta::verify`（内置公钥）。
//!
//! `release_notes` 是 CHANGELOG 最新版本段的纯文本（发布脚本提取，去 Markdown），
//! 只作展示，不参与应用逻辑。

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};

use crate::manifest::{version_at_least, SignatureError};

/// 应用元数据文件名（发布资产；与 `manifest.json` 同目录发布）。
pub const APP_META_FILE_NAME: &str = "app.json";
/// 应用元数据格式标识。
pub const APP_META_FORMAT: &str = "zhu-ye-app";
/// 应用元数据格式版本。
pub const APP_META_VERSION: u32 = 1;

/// 应用元数据签名块（与 manifest 签名块同构，独立类型避免耦合）。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppMetaSignature {
    /// 算法标识，当前仅支持 `ed25519`。
    pub algorithm: String,
    /// 公钥（32 字节，十六进制）；便于发布方轮换与审计。
    pub public_key: String,
    /// 签名（64 字节，十六进制）。
    pub value: String,
}

/// 应用版本元数据（磁盘 `app.json` 结构；字段顺序即规范化签名顺序）。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppMeta {
    /// 格式标识。
    pub format: String,
    /// 格式版本。
    pub version: u32,
    /// 输入法应用版本（如 `0.1.3`；与根 Cargo.toml 一致，前缀无 `v`）。
    pub app_version: String,
    /// 发布时间（ISO-8601）。
    pub published_at: String,
    /// 该版本的更新日志（纯文本，多行）；仅展示不参与逻辑。
    pub release_notes: String,
    /// 签名块；未签名时为 `None`。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<AppMetaSignature>,
}

impl AppMeta {
    /// 待签名的规范化字节：排除 `signature` 字段后的紧凑 JSON。
    ///
    /// 与 manifest 同规则：`serde_json::to_vec`（紧凑、无空白），字段顺序由
    /// 结构体声明固定，任何平台产出同一字节序列。
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, String> {
        let unsigned = AppMeta {
            signature: None,
            ..self.clone()
        };
        serde_json::to_vec(&unsigned).map_err(|error| format!("app 元数据序列化失败: {error}"))
    }

    /// 用**内置公钥**验签（信任锚与 manifest 同一个，P-04）。
    ///
    /// 文件自带公钥只用于审计；攻击者替换 app.json 并附自己的公钥与签名仍会失败。
    /// 验证动作只与 `trusted_public_key` 有关，与 `signature.public_key` 字段无关。
    pub fn verify(&self, trusted_public_key: &VerifyingKey) -> Result<(), SignatureError> {
        let signature = self.signature.as_ref().ok_or(SignatureError::Missing)?;
        if signature.algorithm != crate::manifest::SIGNATURE_ALGORITHM {
            return Err(SignatureError::UnsupportedAlgorithm(
                signature.algorithm.clone(),
            ));
        }
        let signature_bytes = crate::manifest::hex_decode(&signature.value)?;
        let signature_array: [u8; 64] = signature_bytes.as_slice().try_into().map_err(|_| {
            SignatureError::Encoding(format!(
                "签名长度应为 64 字节，实际 {}",
                signature_bytes.len()
            ))
        })?;
        let parsed = Signature::from_bytes(&signature_array);

        let canonical = self.canonical_bytes().map_err(SignatureError::Json)?;
        trusted_public_key
            .verify(&canonical, &parsed)
            .map_err(|_| SignatureError::VerifyFailed)
    }
}

/// 用发布私钥对应用元数据签名，返回带签名块的副本（发布侧，T-154）。
///
/// `secret` 是 32 字节 ed25519 私钥种子；调用方负责从发布环境 secret 读取，
/// 绝不写入仓库或日志。
pub fn sign_app_meta(meta: &AppMeta, secret: &[u8; 32]) -> Result<AppMeta, String> {
    let signing_key = SigningKey::from_bytes(secret);
    let canonical = meta.canonical_bytes()?;
    let signature = signing_key.sign(&canonical);
    let verifying_key = signing_key.verifying_key();
    let mut signed = meta.clone();
    signed.signature = Some(AppMetaSignature {
        algorithm: crate::manifest::SIGNATURE_ALGORITHM.to_owned(),
        public_key: crate::manifest::hex_encode(verifying_key.as_bytes()),
        value: crate::manifest::hex_encode(&signature.to_bytes()),
    });
    Ok(signed)
}

/// 解析并校验 `app.json` 文本；格式/版本不合法返回 `Err`。
pub fn parse_app_meta(text: &str) -> Result<AppMeta, String> {
    let meta: AppMeta =
        serde_json::from_str(text).map_err(|error| format!("app 元数据不是合法 JSON: {error}"))?;
    if meta.format != APP_META_FORMAT {
        return Err(format!(
            "app 元数据格式不是“{}”（实际：{}）",
            APP_META_FORMAT, meta.format
        ));
    }
    if meta.version > APP_META_VERSION {
        return Err(format!(
            "app 元数据版本 {} 高于当前支持的 {}，请升级输入法",
            meta.version, APP_META_VERSION
        ));
    }
    Ok(meta)
}

/// 判断应用是否可更新：`app_version > core_version` 时返回 `true`。
///
/// 比较规则复用 `version_at_least`（点分数字逐段比较，忽略 pre-release 后缀：
/// `0.1.3-alpha` 按 `0.1.3` 参与），与词典包 `min_engine_version` 同一口径。
/// 用双向比较排除相等（pre-release 版本号 `0.1.3-alpha` 与 `0.1.3` 视为同一）。
#[must_use]
pub fn app_update_available(app_version: &str, core_version: &str) -> bool {
    version_at_least(app_version, core_version) && !version_at_least(core_version, app_version)
}

#[cfg(test)]
mod tests {
    use super::{app_update_available, parse_app_meta, sign_app_meta, AppMeta, APP_META_FORMAT};

    fn sample_meta() -> AppMeta {
        AppMeta {
            format: APP_META_FORMAT.to_owned(),
            version: 1,
            app_version: "0.1.3".to_owned(),
            published_at: "2026-10-13T00:00:00Z".to_owned(),
            release_notes: "- 修复描述文字底裁\n- 检查更新不再弹窗".to_owned(),
            signature: None,
        }
    }

    #[test]
    fn 签名验签往返一致() {
        let secret = [7u8; 32];
        let signed = sign_app_meta(&sample_meta(), &secret).unwrap();
        let key = ed25519_dalek::SigningKey::from_bytes(&secret).verifying_key();
        assert!(signed.verify(&key).is_ok(), "自签自验必须通过");
        // 公钥入库审计位可读。
        let sig = signed.signature.as_ref().unwrap();
        assert_eq!(sig.algorithm, "ed25519");
    }

    #[test]
    fn 篡改内容验签失败() {
        let secret = [9u8; 32];
        let mut signed = sign_app_meta(&sample_meta(), &secret).unwrap();
        let key = ed25519_dalek::SigningKey::from_bytes(&secret).verifying_key();
        // 篡改 app_version 或更新日志都必然破坏签名。
        signed.app_version = "0.1.4".to_owned();
        assert!(signed.verify(&key).is_err(), "篡改必须被拒绝");
        let mut signed2 = sign_app_meta(&sample_meta(), &secret).unwrap();
        signed2.release_notes = "被篡改的日志".to_owned();
        assert!(signed2.verify(&key).is_err());
    }

    #[test]
    fn 缺失签名被拒() {
        let key = ed25519_dalek::SigningKey::from_bytes(&[1u8; 32]).verifying_key();
        assert!(sample_meta().verify(&key).is_err(), "无签名必须报错");
    }

    #[test]
    fn 错公钥验签失败() {
        let secret = [3u8; 32];
        let signed = sign_app_meta(&sample_meta(), &secret).unwrap();
        let wrong = ed25519_dalek::SigningKey::from_bytes(&[5u8; 32]).verifying_key();
        assert!(signed.verify(&wrong).is_err(), "信任锚之外的公钥必须失败");
    }

    #[test]
    fn 解析校验格式与版本() {
        let text = serde_json::to_string(&sample_meta()).unwrap();
        let parsed = parse_app_meta(&text).unwrap();
        assert_eq!(parsed.app_version, "0.1.3");
        // 格式标识错误被拒。
        let bad = text.replace(APP_META_FORMAT, "wrong-format");
        assert!(parse_app_meta(&bad).is_err());
        // 版本过高被拒。
        let too_new = text.replace("\"version\":1", "\"version\":99");
        assert!(parse_app_meta(&too_new).is_err());
    }

    #[test]
    fn 应用版本比较满足语义() {
        assert!(app_update_available("0.1.3", "0.1.2"));
        assert!(app_update_available("0.1.3", "0.1.2-alpha"));
        assert!(!app_update_available("0.1.2", "0.1.2"));
        assert!(!app_update_available("0.1.1", "0.1.2"));
        assert!(app_update_available("1.0.0", "0.9.9"));
    }
}
