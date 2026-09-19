//! 词典二进制格式定义。
//!
//! 格式为 v0 草案的正式化 v1：头部 + 拼音索引 + 词条表 + bigram 表 + 文本池。
//! 构建方（`zhu-ye-dict`）与加载方（core 的 mmap 词典）共用本模块，
//! 避免同一格式出现两套字节布局。所有多字节字段均为小端。

use sha2::{Digest, Sha256};

/// 文件魔数 `ZYDT`。
pub const MAGIC: [u8; 4] = *b"ZYDT";

/// 当前词典二进制格式版本。
pub const DICT_VERSION: u32 = 1;

/// 头部固定大小。
pub const HEADER_SIZE: usize = 96;

/// 拼音索引记录大小。
pub const PINYIN_INDEX_RECORD_SIZE: usize = 24;

/// 词条记录大小。
pub const ENTRY_RECORD_SIZE: usize = 24;

/// 对内容区（头部之后的全部字节）计算 SHA-256，供构建与加载共用。
#[must_use]
pub fn content_sha256(content: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(content);
    hasher.finalize().into()
}

/// bigram 记录大小。
pub const BIGRAM_RECORD_SIZE: usize = 16;

/// 文本池首个字节位置；offset 从 1 开始，0 表示“无译文”。
pub const TEXT_POOL_START: u32 = 1;

/// 词典头部。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DictHeader {
    /// 拼音索引条目数。
    pub pinyin_index_count: u32,
    /// 词条总数。
    pub entry_count: u32,
    /// bigram 总数。
    pub bigram_count: u32,
    /// 拼音索引区偏移。
    pub pinyin_index_offset: u64,
    /// 词条表偏移。
    pub entry_table_offset: u64,
    /// bigram 区偏移。
    pub bigram_offset: u64,
    /// 文本池偏移。
    pub text_pool_offset: u64,
    /// 内容区 SHA-256（头部之后的全部字节）。
    pub content_hash: [u8; 32],
}

impl DictHeader {
    /// 序列化头部为固定 96 字节。
    #[must_use]
    pub fn to_bytes(&self) -> [u8; HEADER_SIZE] {
        let mut out = [0u8; HEADER_SIZE];
        out[..4].copy_from_slice(&MAGIC);
        out[4..8].copy_from_slice(&DICT_VERSION.to_le_bytes());
        out[8..12].copy_from_slice(&self.pinyin_index_count.to_le_bytes());
        out[12..16].copy_from_slice(&self.entry_count.to_le_bytes());
        out[16..20].copy_from_slice(&self.bigram_count.to_le_bytes());
        out[20..24].copy_from_slice(&0u32.to_le_bytes());
        out[24..32].copy_from_slice(&self.pinyin_index_offset.to_le_bytes());
        out[32..40].copy_from_slice(&self.entry_table_offset.to_le_bytes());
        out[40..48].copy_from_slice(&self.bigram_offset.to_le_bytes());
        out[48..56].copy_from_slice(&self.text_pool_offset.to_le_bytes());
        out[56..88].copy_from_slice(&self.content_hash);
        out
    }

    /// 解析头部；魔数、版本或长度不合法时返回错误描述。
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < HEADER_SIZE {
            return Err("文件过小，不足头部大小".to_owned());
        }
        if bytes[..4] != MAGIC {
            return Err("魔数不匹配，不是词典文件".to_owned());
        }
        let version = u32::from_le_bytes(bytes[4..8].try_into().expect("切片长度固定"));
        if version != DICT_VERSION {
            return Err(format!(
                "词典版本 {version} 不受支持，当前支持 {DICT_VERSION}"
            ));
        }
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&bytes[56..88]);
        Ok(Self {
            pinyin_index_count: u32::from_le_bytes(bytes[8..12].try_into().expect("切片长度固定")),
            entry_count: u32::from_le_bytes(bytes[12..16].try_into().expect("切片长度固定")),
            bigram_count: u32::from_le_bytes(bytes[16..20].try_into().expect("切片长度固定")),
            pinyin_index_offset: u64::from_le_bytes(
                bytes[24..32].try_into().expect("切片长度固定"),
            ),
            entry_table_offset: u64::from_le_bytes(bytes[32..40].try_into().expect("切片长度固定")),
            bigram_offset: u64::from_le_bytes(bytes[40..48].try_into().expect("切片长度固定")),
            text_pool_offset: u64::from_le_bytes(bytes[48..56].try_into().expect("切片长度固定")),
            content_hash: hash,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{DictHeader, DICT_VERSION, HEADER_SIZE, MAGIC};

    #[test]
    fn 头部序列化往返一致() {
        let header = DictHeader {
            pinyin_index_count: 3,
            entry_count: 9,
            bigram_count: 2,
            pinyin_index_offset: 96,
            entry_table_offset: 144,
            bigram_offset: 360,
            text_pool_offset: 400,
            content_hash: [7u8; 32],
        };
        let bytes = header.to_bytes();
        assert_eq!(bytes.len(), HEADER_SIZE);
        assert_eq!(&bytes[..4], &MAGIC);
        assert_eq!(
            u32::from_le_bytes(bytes[4..8].try_into().unwrap()),
            DICT_VERSION
        );
        assert_eq!(DictHeader::from_bytes(&bytes).unwrap(), header);
    }

    #[test]
    fn 头部非法输入被拒绝() {
        assert!(DictHeader::from_bytes(&[]).is_err());
        let mut bytes = DictHeader {
            pinyin_index_count: 0,
            entry_count: 0,
            bigram_count: 0,
            pinyin_index_offset: 0,
            entry_table_offset: 0,
            bigram_offset: 0,
            text_pool_offset: 0,
            content_hash: [0; 32],
        }
        .to_bytes();
        bytes[5] = 99;
        assert!(DictHeader::from_bytes(&bytes).is_err());
    }
}
