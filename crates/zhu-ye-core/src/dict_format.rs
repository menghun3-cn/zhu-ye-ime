//! 词典二进制格式定义。
//!
//! 格式 v2：头部 + 拼音索引 + 词条表 + bigram 表 + 中英翻译索引 +
//! 英文反查索引 + 文本池。v1 的拼音/词条/bigram/文本池逻辑保留，
//! 头部扩展为 128 字节，并在 bigram 表与文本池之间新增两个只读索引。
//! 构建方（`zhu-ye-dict`）与加载方（core 的 mmap 词典）共用本模块，
//! 避免同一格式出现两套字节布局。所有多字节字段均为小端。

use sha2::{Digest, Sha256};

/// 文件魔数 `ZYDT`。
pub const MAGIC: [u8; 4] = *b"ZYDT";

/// 当前词典二进制格式版本。
pub const DICT_VERSION: u32 = 2;

/// 头部固定大小。
pub const HEADER_SIZE: usize = 128;

/// 拼音索引记录大小。
pub const PINYIN_INDEX_RECORD_SIZE: usize = 24;

/// 词条记录大小。
pub const ENTRY_RECORD_SIZE: usize = 24;

/// 中文词到译文索引记录大小。
pub const WORD_TRANSLATION_RECORD_SIZE: usize = 24;

/// 归一化英文到中文词索引记录大小。
pub const REVERSE_TRANSLATION_RECORD_SIZE: usize = 24;

/// bigram 记录大小。
pub const BIGRAM_RECORD_SIZE: usize = 16;

/// 文本池首个字节位置；offset 从 1 开始，0 表示“无文本/无译文”。
pub const TEXT_POOL_START: u32 = 1;

/// 对内容区（头部之后的全部字节）计算 SHA-256，供构建与加载共用。
#[must_use]
pub fn content_sha256(content: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(content);
    hasher.finalize().into()
}

/// 英文归一化：小写并折叠空白，用于反查索引键。
#[must_use]
pub fn normalize_translation_key(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut pending_space = false;
    for ch in text.chars() {
        if ch.is_whitespace() {
            pending_space = !out.is_empty();
        } else {
            if pending_space {
                out.push(' ');
                pending_space = false;
            }
            for lower in ch.to_lowercase() {
                out.push(lower);
            }
        }
    }
    out
}

/// 词性标注前缀白名单（m6 构建期拼入译文显示串，见 m6::pos_label）。
/// 反查键（英→中）必须剥掉前缀，否则 `adj. good` 无法按 `good` 反查命中。
pub const POS_PREFIXES: &[&str] = &[
    "n.", "v.", "adj.", "adv.", "pron.", "prep.", "conj.", "int.", "num.", "cls.", "aux.",
];

/// 剥掉译文开头的词性前缀（大小写不敏感），供反查键生成使用。
/// 前缀不在白名单时不剥；剥后为空串（纯前缀条目）时原样返回。
#[must_use]
pub fn strip_pos_prefix(text: &str) -> &str {
    let trimmed = text.trim_start();
    for prefix in POS_PREFIXES {
        if trimmed.len() >= prefix.len()
            && trimmed
                .as_bytes()
                .get(..prefix.len())
                .is_some_and(|head| head.eq_ignore_ascii_case(prefix.as_bytes()))
        {
            let rest = trimmed[prefix.len()..].trim_start();
            if rest.is_empty() {
                return trimmed;
            }
            return rest;
        }
    }
    trimmed
}

/// 把译文串按 `;` 拆成多条单义（T-131）：CEDICT/精修表多义以 `a; b` 存储，
/// 逐条 trim、去尾标点（`!`/`.`/`…`）、过滤空义，按首现顺序去重。
/// 词性前缀（`v.` 等）保留不动——显示需要；上屏时另行 `strip_pos_prefix` 剥离。
#[must_use]
pub fn split_translations(text: &str) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for part in text.split(';') {
        let trimmed = part.trim().trim_end_matches(['!', '.', '…']).trim();
        if trimmed.is_empty() || !seen.insert(trimmed.to_owned()) {
            continue;
        }
        out.push(trimmed.to_owned());
    }
    out
}

/// 词典头部。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DictHeader {
    /// 拼音索引条目数。
    pub pinyin_index_count: u32,
    /// 词条总数。
    pub entry_count: u32,
    /// bigram 总数。
    pub bigram_count: u32,
    /// 中文词到译文索引条目数。
    pub word_translation_count: u32,
    /// 归一化英文到中文词索引条目数。
    pub reverse_translation_count: u32,
    /// 中文词到译文索引区偏移。
    pub word_translation_offset: u64,
    /// 英文反查索引区偏移。
    pub reverse_translation_offset: u64,
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
    /// 序列化头部为固定 128 字节。
    #[must_use]
    pub fn to_bytes(&self) -> [u8; HEADER_SIZE] {
        let mut out = [0u8; HEADER_SIZE];
        out[..4].copy_from_slice(&MAGIC);
        out[4..8].copy_from_slice(&DICT_VERSION.to_le_bytes());
        out[8..12].copy_from_slice(&self.pinyin_index_count.to_le_bytes());
        out[12..16].copy_from_slice(&self.entry_count.to_le_bytes());
        out[16..20].copy_from_slice(&self.bigram_count.to_le_bytes());
        out[20..24].copy_from_slice(&self.word_translation_count.to_le_bytes());
        out[24..28].copy_from_slice(&self.reverse_translation_count.to_le_bytes());
        out[28..36].copy_from_slice(&self.word_translation_offset.to_le_bytes());
        out[36..44].copy_from_slice(&self.reverse_translation_offset.to_le_bytes());
        out[44..52].copy_from_slice(&self.pinyin_index_offset.to_le_bytes());
        out[52..60].copy_from_slice(&self.entry_table_offset.to_le_bytes());
        out[60..68].copy_from_slice(&self.bigram_offset.to_le_bytes());
        out[68..76].copy_from_slice(&self.text_pool_offset.to_le_bytes());
        out[80..112].copy_from_slice(&self.content_hash);
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
        hash.copy_from_slice(&bytes[80..112]);
        Ok(Self {
            pinyin_index_count: u32::from_le_bytes(bytes[8..12].try_into().expect("切片长度固定")),
            entry_count: u32::from_le_bytes(bytes[12..16].try_into().expect("切片长度固定")),
            bigram_count: u32::from_le_bytes(bytes[16..20].try_into().expect("切片长度固定")),
            word_translation_count: u32::from_le_bytes(
                bytes[20..24].try_into().expect("切片长度固定"),
            ),
            reverse_translation_count: u32::from_le_bytes(
                bytes[24..28].try_into().expect("切片长度固定"),
            ),
            word_translation_offset: u64::from_le_bytes(
                bytes[28..36].try_into().expect("切片长度固定"),
            ),
            reverse_translation_offset: u64::from_le_bytes(
                bytes[36..44].try_into().expect("切片长度固定"),
            ),
            pinyin_index_offset: u64::from_le_bytes(
                bytes[44..52].try_into().expect("切片长度固定"),
            ),
            entry_table_offset: u64::from_le_bytes(bytes[52..60].try_into().expect("切片长度固定")),
            bigram_offset: u64::from_le_bytes(bytes[60..68].try_into().expect("切片长度固定")),
            text_pool_offset: u64::from_le_bytes(bytes[68..76].try_into().expect("切片长度固定")),
            content_hash: hash,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{
        normalize_translation_key, split_translations, strip_pos_prefix, DictHeader, DICT_VERSION,
        HEADER_SIZE, MAGIC,
    };

    #[test]
    fn 头部序列化往返一致() {
        let header = DictHeader {
            pinyin_index_count: 3,
            entry_count: 9,
            bigram_count: 2,
            word_translation_count: 4,
            reverse_translation_count: 4,
            pinyin_index_offset: 128,
            entry_table_offset: 200,
            bigram_offset: 416,
            word_translation_offset: 448,
            reverse_translation_offset: 520,
            text_pool_offset: 592,
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
            word_translation_count: 0,
            reverse_translation_count: 0,
            pinyin_index_offset: 0,
            entry_table_offset: 0,
            bigram_offset: 0,
            word_translation_offset: 0,
            reverse_translation_offset: 0,
            text_pool_offset: 0,
            content_hash: [0; 32],
        }
        .to_bytes();
        bytes[5] = 99;
        assert!(DictHeader::from_bytes(&bytes).is_err());
    }

    #[test]
    fn 英文归一化折叠大小写与空白() {
        assert_eq!(normalize_translation_key("Hello  World"), "hello world");
        assert_eq!(normalize_translation_key("  Hello\tWorld  "), "hello world");
        assert_eq!(normalize_translation_key("XI'AN"), "xi'an");
        assert_eq!(normalize_translation_key(""), "");

        // 词性前缀剥除（反查键纯净）：小写/大写前缀都剥，非白名单不动，纯前缀保留
        assert_eq!(strip_pos_prefix("v. to generate"), "to generate");
        assert_eq!(strip_pos_prefix("adj. good day"), "good day");
        assert_eq!(strip_pos_prefix("N. China"), "China");
        assert_eq!(strip_pos_prefix("to be"), "to be");
        assert_eq!(strip_pos_prefix("  adv. go ahead"), "go ahead");
        assert_eq!(strip_pos_prefix("n."), "n.");
        assert_eq!(strip_pos_prefix("节点"), "节点");
    }

    #[test]
    fn 译文按分号拆成多条单义() {
        // T-131：多义译文 `a; b` 拆成独立单义（你好 → hello / hi 各一条候选）。
        assert_eq!(
            split_translations("hello; hi"),
            vec!["hello".to_owned(), "hi".to_owned()]
        );
        // 词性前缀保留（显示层需要；上屏时另剥）。
        assert_eq!(
            split_translations("int. hello; int. hi"),
            vec!["int. hello".to_owned(), "int. hi".to_owned()]
        );
        assert_eq!(
            split_translations("yes; to be"),
            vec!["yes".to_owned(), "to be".to_owned()]
        );
        // 尾标点/空白清理：CEDICT 段尾感叹号、多余分号段不产出空义。
        assert_eq!(
            split_translations("hello!; ; hi"),
            vec!["hello".to_owned(), "hi".to_owned()]
        );
        assert_eq!(split_translations("hello…"), vec!["hello".to_owned()]);
        // 重复义按首现顺序去重。
        assert_eq!(
            split_translations("a; a; b"),
            vec!["a".to_owned(), "b".to_owned()]
        );
        // 括号说明属于语义内容保留；空串/纯标点不产出义。
        assert_eq!(
            split_translations("to be (followed by substantives only)"),
            vec!["to be (followed by substantives only)".to_owned()]
        );
        assert!(split_translations("").is_empty());
        assert!(split_translations("; ; !").is_empty());
    }
}
