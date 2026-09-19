//! v1 词典 mmap 只读加载器。
//!
//! `DictionaryFile` 同时实现 `Dictionary` 与 `BigramModel`：加载时完整
//! 校验头部、内容 SHA-256、分区边界、文本 UTF-8 与记录排序；查询阶段
//! 只按偏移读取命中记录，不再构造额外索引。

use std::fmt;
use std::fs::File;
use std::path::Path;
use std::sync::Arc;

use memmap2::{Mmap, MmapOptions};

use crate::bigram::BigramModel;
use crate::dict::{Dictionary, DictionaryEntry};
use crate::dict_format::{
    content_sha256, BIGRAM_RECORD_SIZE, ENTRY_RECORD_SIZE, HEADER_SIZE, PINYIN_INDEX_RECORD_SIZE,
};
use crate::{Error, Result};

/// mmap 词典文件；加载方只需持有文件与头部，字符串按偏移即时读取。
#[derive(Clone)]
pub struct DictionaryFile {
    map: Arc<Mmap>,
    header: crate::dict_format::DictHeader,
}

impl fmt::Debug for DictionaryFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DictionaryFile")
            .field("file_size", &self.map.len())
            .field("pinyin_index_count", &self.header.pinyin_index_count)
            .field("entry_count", &self.header.entry_count)
            .field("bigram_count", &self.header.bigram_count)
            .finish()
    }
}

impl DictionaryFile {
    /// 打开并校验词典文件；失败时返回可诊断错误，不产生部分可用状态。
    pub fn open(path: &Path) -> Result<Self> {
        let file = File::open(path)
            .map_err(|error| Error::Dictionary(format!("打开词典文件失败: {error}")))?;
        let map = unsafe { MmapOptions::new().map(&file) }
            .map_err(|error| Error::Dictionary(format!("内存映射失败: {error}")))?;
        Self::from_map(map)
    }

    fn from_map(map: Mmap) -> Result<Self> {
        let header = crate::dict_format::DictHeader::from_bytes(&map).map_err(Error::Dictionary)?;
        let actual_hash = content_sha256(&map[HEADER_SIZE..]);
        if actual_hash != header.content_hash {
            return Err(Error::Dictionary(
                "内容 SHA-256 不匹配，词典文件可能已损坏".to_owned(),
            ));
        }
        let file = Self {
            map: Arc::new(map),
            header,
        };
        file.validate_layout()?;
        Ok(file)
    }

    /// 返回解析后的头部元数据，供检查与日志使用。
    #[must_use]
    pub fn header(&self) -> &crate::dict_format::DictHeader {
        &self.header
    }

    /// 返回映射文件总字节数。
    #[must_use]
    pub fn file_size(&self) -> usize {
        self.map.len()
    }

    fn pool(&self) -> &[u8] {
        &self.map[self.header.text_pool_offset as usize..]
    }

    fn index_records(&self) -> &[u8] {
        let start = self.header.pinyin_index_offset as usize;
        let end = start + PINYIN_INDEX_RECORD_SIZE * self.header.pinyin_index_count as usize;
        &self.map[start..end]
    }

    fn entry_records(&self) -> &[u8] {
        let start = self.header.entry_table_offset as usize;
        let end = start + ENTRY_RECORD_SIZE * self.header.entry_count as usize;
        &self.map[start..end]
    }

    fn bigram_records(&self) -> &[u8] {
        let start = self.header.bigram_offset as usize;
        let end = start + BIGRAM_RECORD_SIZE * self.header.bigram_count as usize;
        &self.map[start..end]
    }

    /// 校验分区边界与全部记录；只允许合法文件进入查询阶段。
    fn validate_layout(&self) -> Result<()> {
        let file_len = u64::try_from(self.map.len())
            .map_err(|_| Error::Dictionary("文件长度溢出".to_owned()))?;
        if file_len < HEADER_SIZE as u64 {
            return Err(Error::Dictionary("文件长度小于头部".to_owned()));
        }

        let expected_index = HEADER_SIZE as u64;
        let index_bytes = u64::from(self.header.pinyin_index_count)
            .checked_mul(PINYIN_INDEX_RECORD_SIZE as u64)
            .ok_or_else(|| Error::Dictionary("拼音索引区大小溢出".to_owned()))?;
        let expected_entry = expected_index
            .checked_add(index_bytes)
            .ok_or_else(|| Error::Dictionary("词条区偏移溢出".to_owned()))?;
        let entry_bytes = u64::from(self.header.entry_count)
            .checked_mul(ENTRY_RECORD_SIZE as u64)
            .ok_or_else(|| Error::Dictionary("词条区大小溢出".to_owned()))?;
        let expected_bigram = expected_entry
            .checked_add(entry_bytes)
            .ok_or_else(|| Error::Dictionary("bigram 区偏移溢出".to_owned()))?;
        let bigram_bytes = u64::from(self.header.bigram_count)
            .checked_mul(BIGRAM_RECORD_SIZE as u64)
            .ok_or_else(|| Error::Dictionary("bigram 区大小溢出".to_owned()))?;
        let expected_pool = expected_bigram
            .checked_add(bigram_bytes)
            .ok_or_else(|| Error::Dictionary("文本池偏移溢出".to_owned()))?;

        if self.header.pinyin_index_offset != expected_index
            || self.header.entry_table_offset != expected_entry
            || self.header.bigram_offset != expected_bigram
            || self.header.text_pool_offset != expected_pool
        {
            return Err(Error::Dictionary("分区偏移与 v1 固定布局不符".to_owned()));
        }
        if expected_pool >= file_len {
            return Err(Error::Dictionary("文本池为空或越界".to_owned()));
        }

        self.validate_index_and_entries()?;
        self.validate_bigrams()?;
        Ok(())
    }

    fn validate_index_and_entries(&self) -> Result<()> {
        let mut expected_start = 0u32;
        let mut previous_key: Option<&str> = None;
        for index in 0..self.header.pinyin_index_count as usize {
            let record = PinyinIndexRecord::new(
                &self.index_records()
                    [index * PINYIN_INDEX_RECORD_SIZE..(index + 1) * PINYIN_INDEX_RECORD_SIZE],
            );
            if record.entry_start() != expected_start {
                return Err(Error::Dictionary(format!(
                    "拼音索引 {index} 起始序号不连续"
                )));
            }
            if u64::from(record.entry_count()) > u64::from(self.header.entry_count - expected_start)
            {
                return Err(Error::Dictionary(format!("拼音索引 {index} 条目数量越界")));
            }
            if record.pinyin_len() == 0 {
                return Err(Error::Dictionary(format!("拼音索引 {index} 文本为空")));
            }
            let pinyin_offset = u32::try_from(record.pinyin_offset())
                .map_err(|_| Error::Dictionary(format!("拼音索引 {index} 偏移超过 u32 上限")))?;
            let key = self.text_at(pinyin_offset, record.pinyin_len())?;
            if let Some(previous) = previous_key {
                if previous >= key {
                    return Err(Error::Dictionary("拼音索引未按文本严格递增".to_owned()));
                }
            }
            previous_key = Some(key);

            let range = record.entry_start()..record.entry_start() + record.entry_count();
            for entry_index in range {
                let index = entry_index as usize;
                let entry = EntryRecord::new(
                    &self.entry_records()
                        [index * ENTRY_RECORD_SIZE..(index + 1) * ENTRY_RECORD_SIZE],
                );
                if entry.word_offset() == 0
                    || entry.word_len() == 0
                    || entry.pinyin_offset() == 0
                    || entry.pinyin_len() == 0
                {
                    return Err(Error::Dictionary(format!("词条 {index} 文本字段非法")));
                }
                if (entry.translation_offset() == 0) != (entry.translation_len() == 0) {
                    return Err(Error::Dictionary(format!(
                        "词条 {index} 译文偏移与长度不一致"
                    )));
                }
                let word = self.text_at(entry.word_offset(), entry.word_len())?;
                let pinyin = self.text_at(entry.pinyin_offset(), entry.pinyin_len())?;
                if word.is_empty() || pinyin != key {
                    return Err(Error::Dictionary(format!(
                        "词条 {index} 拼音与拼音索引分组不符"
                    )));
                }
                if entry.translation_len() > 0 {
                    let _ = self.text_at(entry.translation_offset(), entry.translation_len())?;
                }
            }
            expected_start += record.entry_count();
        }
        if expected_start != self.header.entry_count {
            return Err(Error::Dictionary("拼音索引未覆盖全部词条".to_owned()));
        }

        Ok(())
    }

    fn validate_bigrams(&self) -> Result<()> {
        let mut previous_key: Option<(&str, &str)> = None;
        for index in 0..self.header.bigram_count as usize {
            let record = BigramRecord::new(
                &self.bigram_records()
                    [index * BIGRAM_RECORD_SIZE..(index + 1) * BIGRAM_RECORD_SIZE],
            );
            if record.previous_len() == 0 || record.word_len() == 0 {
                return Err(Error::Dictionary(format!("bigram {index} 文本为空")));
            }
            let key = (
                self.text_at(record.previous_offset(), record.previous_len())?,
                self.text_at(record.word_offset(), record.word_len())?,
            );
            if let Some(previous) = previous_key {
                if previous >= key {
                    return Err(Error::Dictionary("bigram 未按前词/后词严格递增".to_owned()));
                }
            }
            previous_key = Some(key);
        }
        Ok(())
    }

    /// 读取文本池中的字符串；校验阶段对 UTF-8 与边界严格检查。
    fn text_at(&self, offset: u32, len: u16) -> Result<&str> {
        if offset == 0 {
            return Err(Error::Dictionary("文本偏移为 0".to_owned()));
        }
        let start = usize::try_from(offset)
            .map_err(|_| Error::Dictionary("文本偏移超出 usize".to_owned()))?;
        let end = start
            .checked_add(usize::from(len))
            .ok_or_else(|| Error::Dictionary("文本区间溢出".to_owned()))?;
        let pool = self.pool();
        let bytes = pool
            .get(start..end)
            .ok_or_else(|| Error::Dictionary("文本区间越界".to_owned()))?;
        std::str::from_utf8(bytes)
            .map_err(|_| Error::Dictionary("文本池包含非 UTF-8 字节".to_owned()))
    }

    /// 查询路径使用的无检查读取；调用前保证文本已通过完整校验。
    fn text_unchecked(&self, offset: u32, len: u16) -> &str {
        let start = offset as usize;
        let bytes = &self.pool()[start..start + usize::from(len)];
        // SAFETY: `validate_layout` 已校验所有文本的 UTF-8 与边界；
        // 此处复用同一布局读取，不会出现非法字节或越界。
        unsafe { std::str::from_utf8_unchecked(bytes) }
    }

    fn find_pinyin_index(&self, pinyin: &str) -> Option<usize> {
        let mut low = 0usize;
        let mut high = self.header.pinyin_index_count as usize;
        while low < high {
            let mid = low + (high - low) / 2;
            let record = PinyinIndexRecord::new(
                &self.index_records()
                    [mid * PINYIN_INDEX_RECORD_SIZE..(mid + 1) * PINYIN_INDEX_RECORD_SIZE],
            );
            let pinyin_offset = u32::try_from(record.pinyin_offset()).ok()?;
            let key = self.text_unchecked(pinyin_offset, record.pinyin_len());
            match key.as_bytes().cmp(pinyin.as_bytes()) {
                std::cmp::Ordering::Less => low = mid + 1,
                std::cmp::Ordering::Greater => high = mid,
                std::cmp::Ordering::Equal => return Some(mid),
            }
        }
        None
    }

    fn find_bigram(&self, previous: &str, word: &str) -> Option<u64> {
        let mut low = 0usize;
        let mut high = self.header.bigram_count as usize;
        while low < high {
            let mid = low + (high - low) / 2;
            let record = BigramRecord::new(
                &self.bigram_records()[mid * BIGRAM_RECORD_SIZE..(mid + 1) * BIGRAM_RECORD_SIZE],
            );
            let key = (
                self.text_unchecked(record.previous_offset(), record.previous_len()),
                self.text_unchecked(record.word_offset(), record.word_len()),
            );
            match key.cmp(&(previous, word)) {
                std::cmp::Ordering::Less => low = mid + 1,
                std::cmp::Ordering::Greater => high = mid,
                std::cmp::Ordering::Equal => return Some(u64::from(record.frequency())),
            }
        }
        None
    }
}

impl Dictionary for DictionaryFile {
    fn lookup(&self, pinyin: &str) -> Vec<DictionaryEntry> {
        let Some(index) = self.find_pinyin_index(pinyin) else {
            return Vec::new();
        };
        let record = PinyinIndexRecord::new(
            &self.index_records()
                [index * PINYIN_INDEX_RECORD_SIZE..(index + 1) * PINYIN_INDEX_RECORD_SIZE],
        );
        let mut found = Vec::with_capacity(record.entry_count() as usize);
        for entry_index in record.entry_start()..record.entry_start() + record.entry_count() {
            let index = entry_index as usize;
            let entry = EntryRecord::new(
                &self.entry_records()[index * ENTRY_RECORD_SIZE..(index + 1) * ENTRY_RECORD_SIZE],
            );
            let word = self.text_unchecked(entry.word_offset(), entry.word_len());
            let entry_pinyin = self.text_unchecked(entry.pinyin_offset(), entry.pinyin_len());
            let translation = if entry.translation_len() == 0 {
                None
            } else {
                Some(
                    self.text_unchecked(entry.translation_offset(), entry.translation_len())
                        .to_owned(),
                )
            };
            found.push(DictionaryEntry {
                word: word.to_owned(),
                pinyin: entry_pinyin.to_owned(),
                translation,
                frequency: u64::from(entry.frequency()),
            });
        }
        // 与内存实现保持一致：同频词保持文件顺序，总顺序按词频降序。
        found.sort_by_key(|entry| std::cmp::Reverse(entry.frequency));
        found
    }
}

impl BigramModel for DictionaryFile {
    fn frequency(&self, previous: &str, word: &str) -> u64 {
        self.find_bigram(previous, word).unwrap_or(0)
    }
}

/// 拼音索引记录视图：24 字节。
struct PinyinIndexRecord<'a> {
    bytes: &'a [u8],
}

impl<'a> PinyinIndexRecord<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }

    fn pinyin_offset(&self) -> u64 {
        u64::from_le_bytes(self.bytes[0..8].try_into().expect("固定切片"))
    }

    fn entry_start(&self) -> u32 {
        u32::from_le_bytes(self.bytes[8..12].try_into().expect("固定切片"))
    }

    fn entry_count(&self) -> u32 {
        u32::from_le_bytes(self.bytes[12..16].try_into().expect("固定切片"))
    }

    fn pinyin_len(&self) -> u16 {
        u16::from_le_bytes(self.bytes[16..18].try_into().expect("固定切片"))
    }
}

/// 词条记录视图：24 字节。
struct EntryRecord<'a> {
    bytes: &'a [u8],
}

impl<'a> EntryRecord<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }

    fn word_offset(&self) -> u32 {
        u32::from_le_bytes(self.bytes[0..4].try_into().expect("固定切片"))
    }

    fn pinyin_offset(&self) -> u32 {
        u32::from_le_bytes(self.bytes[4..8].try_into().expect("固定切片"))
    }

    fn translation_offset(&self) -> u32 {
        u32::from_le_bytes(self.bytes[8..12].try_into().expect("固定切片"))
    }

    fn frequency(&self) -> u32 {
        u32::from_le_bytes(self.bytes[12..16].try_into().expect("固定切片"))
    }

    fn word_len(&self) -> u16 {
        u16::from_le_bytes(self.bytes[16..18].try_into().expect("固定切片"))
    }

    fn pinyin_len(&self) -> u16 {
        u16::from_le_bytes(self.bytes[18..20].try_into().expect("固定切片"))
    }

    fn translation_len(&self) -> u16 {
        u16::from_le_bytes(self.bytes[20..22].try_into().expect("固定切片"))
    }
}

/// bigram 记录视图：16 字节。
struct BigramRecord<'a> {
    bytes: &'a [u8],
}

impl<'a> BigramRecord<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }

    fn previous_offset(&self) -> u32 {
        u32::from_le_bytes(self.bytes[0..4].try_into().expect("固定切片"))
    }

    fn previous_len(&self) -> u16 {
        u16::from_le_bytes(self.bytes[4..6].try_into().expect("固定切片"))
    }

    fn word_offset(&self) -> u32 {
        u32::from_le_bytes(self.bytes[6..10].try_into().expect("固定切片"))
    }

    fn word_len(&self) -> u16 {
        u16::from_le_bytes(self.bytes[10..12].try_into().expect("固定切片"))
    }

    fn frequency(&self) -> u32 {
        u32::from_le_bytes(self.bytes[12..16].try_into().expect("固定切片"))
    }
}

#[cfg(test)]
mod tests {
    use super::DictionaryFile;
    use crate::bigram::BigramModel;
    use crate::demo::{seed_bigrams, seed_entries};
    use crate::dict::Dictionary;
    use crate::dict_builder::build_v1;

    fn temp_dir(label: &str) -> std::path::PathBuf {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir =
            std::env::temp_dir().join(format!("zhu-ye-dict-{label}-{}-{now}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn 构建文件可加载并按拼音与bigram查询() {
        let dir = temp_dir("roundtrip");
        let path = dir.join("seed.zyct");
        let bytes = build_v1(&seed_entries(), &seed_bigrams()).unwrap();
        std::fs::write(&path, &bytes).unwrap();

        let file = DictionaryFile::open(&path).unwrap();
        let nihao = file.lookup("nihao");
        assert_eq!(nihao.len(), 2);
        assert_eq!(nihao[0].word, "你好");
        assert_eq!(nihao[0].translation.as_deref(), Some("hello"));
        assert!(file.lookup("haha").is_empty());

        assert_eq!(file.frequency("你好", "世界"), 120);
        assert_eq!(file.frequency("你好", "中国"), 0);
        assert_eq!(file.frequency("我们", "的"), 0);

        drop(file);
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn 内容篡改会被拒绝() {
        let dir = temp_dir("tamper");
        let path = dir.join("tampered.zyct");
        let mut bytes = build_v1(&seed_entries(), &seed_bigrams()).unwrap();
        let last = bytes.len() - 1;
        bytes[last] ^= 0x01;
        std::fs::write(&path, &bytes).unwrap();
        assert!(DictionaryFile::open(&path).is_err());
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn 同频词条保持稳定顺序() {
        let dir = temp_dir("stable-order");
        let path = dir.join("stable.zyct");
        let bytes = build_v1(&seed_entries(), &seed_bigrams()).unwrap();
        std::fs::write(&path, &bytes).unwrap();
        let file = DictionaryFile::open(&path).unwrap();
        let first = file.lookup("de");
        let texts: Vec<&str> = first.iter().map(|entry| entry.word.as_str()).collect();
        assert_eq!(texts, vec!["的", "得", "地"]);
        drop(file);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
