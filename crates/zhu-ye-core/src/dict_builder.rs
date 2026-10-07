//! v2 词典二进制构建器。
//!
//! 输入词条与 bigram 后生成确定字节：相同输入 + 相同格式版本必定得到
//! 相同内容哈希。v2 在 v1 布局基础上新增中文词→译文索引与归一化英文→
//! 中文词索引；构建方（`zhu-ye-dict`）直接调用本模块，加载方
//! （`dict_loader`）解析同一布局。

use std::collections::HashMap;

use crate::dict::DictionaryEntry;
use crate::dict_format::{
    content_sha256, normalize_translation_key, strip_pos_prefix, BIGRAM_RECORD_SIZE,
    ENTRY_RECORD_SIZE, HEADER_SIZE, PINYIN_INDEX_RECORD_SIZE, REVERSE_TRANSLATION_RECORD_SIZE,
    TEXT_POOL_START, WORD_TRANSLATION_RECORD_SIZE,
};
use crate::{Error, Result};

/// 文本池引用：偏移 0 且长度 0 表示“无文本”。
type TextRef = (u32, u16);

/// 中文词到译文的索引源。
struct TranslationSource<'a> {
    word: &'a str,
    translation: &'a str,
}

/// 构建 v2 词典字节序列。
///
/// 词条按拼音分组、组内按词频降序与文本升序排列；bigram 去重合并后
/// 按前词/后词字典序排列；翻译索引按中文词排序，英文反查索引按归一化
/// 英文与中文词排序，保证输出确定。
pub fn build_v2(entries: &[DictionaryEntry], bigrams: &[(&str, &str, u64)]) -> Result<Vec<u8>> {
    if entries.len() > u32::MAX as usize {
        return Err(Error::Internal("词条数量超过 u32 上限".to_owned()));
    }
    if bigrams.len() > u32::MAX as usize {
        return Err(Error::Internal("bigram 数量超过 u32 上限".to_owned()));
    }
    for entry in entries {
        if entry.word.is_empty() || entry.pinyin.is_empty() {
            return Err(Error::Internal("词条文本与拼音不允许为空".to_owned()));
        }
    }
    for (previous, word, _) in bigrams {
        if previous.is_empty() || word.is_empty() {
            return Err(Error::Internal("bigram 文本不允许为空".to_owned()));
        }
    }

    let mut sorted_entries: Vec<&DictionaryEntry> = entries.iter().collect();
    sorted_entries.sort_by(|a, b| {
        a.pinyin
            .cmp(&b.pinyin)
            .then_with(|| b.frequency.cmp(&a.frequency))
            .then_with(|| a.word.cmp(&b.word))
    });

    let mut text_pool = vec![0u8];
    let mut text_refs: HashMap<String, TextRef> = HashMap::new();
    let mut index_bytes =
        Vec::with_capacity(PINYIN_INDEX_RECORD_SIZE * sorted_entries.len().min(u32::MAX as usize));
    let mut entry_bytes = Vec::with_capacity(ENTRY_RECORD_SIZE * sorted_entries.len());
    let mut entry_start = 0usize;

    while entry_start < sorted_entries.len() {
        let pinyin = sorted_entries[entry_start].pinyin.as_str();
        let mut entry_end = entry_start + 1;
        while entry_end < sorted_entries.len() && sorted_entries[entry_end].pinyin == pinyin {
            entry_end += 1;
        }
        let count = entry_end - entry_start;
        let (pinyin_offset, pinyin_len) = intern(pinyin, &mut text_pool, &mut text_refs)?;

        let mut index_record = [0u8; PINYIN_INDEX_RECORD_SIZE];
        index_record[..8].copy_from_slice(&u64::from(pinyin_offset).to_le_bytes());
        index_record[8..12].copy_from_slice(
            &u32::try_from(entry_start)
                .map_err(|_| Error::Internal("拼音索引起始序号溢出".to_owned()))?
                .to_le_bytes(),
        );
        index_record[12..16].copy_from_slice(
            &u32::try_from(count)
                .map_err(|_| Error::Internal("拼音索引数量溢出".to_owned()))?
                .to_le_bytes(),
        );
        index_record[16..18].copy_from_slice(&pinyin_len.to_le_bytes());
        index_bytes.extend_from_slice(&index_record);

        for entry in &sorted_entries[entry_start..entry_end] {
            let (word_offset, word_len) = intern(&entry.word, &mut text_pool, &mut text_refs)?;
            let (entry_pinyin_offset, entry_pinyin_len) =
                intern(&entry.pinyin, &mut text_pool, &mut text_refs)?;
            let (translation_offset, translation_len) = match &entry.translation {
                Some(translation) if !translation.is_empty() => {
                    intern(translation, &mut text_pool, &mut text_refs)?
                }
                _ => (0, 0),
            };
            let frequency = u32::try_from(entry.frequency)
                .map_err(|_| Error::Internal("词频超过 u32 上限".to_owned()))?;

            let mut record = [0u8; ENTRY_RECORD_SIZE];
            record[0..4].copy_from_slice(&word_offset.to_le_bytes());
            record[4..8].copy_from_slice(&entry_pinyin_offset.to_le_bytes());
            record[8..12].copy_from_slice(&translation_offset.to_le_bytes());
            record[12..16].copy_from_slice(&frequency.to_le_bytes());
            record[16..18].copy_from_slice(&word_len.to_le_bytes());
            record[18..20].copy_from_slice(&entry_pinyin_len.to_le_bytes());
            record[20..22].copy_from_slice(&translation_len.to_le_bytes());
            entry_bytes.extend_from_slice(&record);
        }
        entry_start = entry_end;
    }

    let mut sorted_bigrams: Vec<(&str, &str, u64)> = bigrams.to_vec();
    sorted_bigrams.sort_by(|a, b| {
        a.0.cmp(b.0)
            .then_with(|| a.1.cmp(b.1))
            .then_with(|| a.2.cmp(&b.2))
    });
    let mut merged_bigrams: Vec<(&str, &str, u64)> = Vec::new();
    for (previous, word, frequency) in sorted_bigrams {
        if let Some(last) = merged_bigrams.last_mut() {
            if last.0 == previous && last.1 == word {
                last.2 = last.2.saturating_add(frequency);
                continue;
            }
        }
        merged_bigrams.push((previous, word, frequency));
    }

    let mut bigram_bytes = Vec::with_capacity(BIGRAM_RECORD_SIZE * merged_bigrams.len());
    for (previous, word, frequency) in merged_bigrams {
        let (previous_offset, previous_len) = intern(previous, &mut text_pool, &mut text_refs)?;
        let (word_offset, word_len) = intern(word, &mut text_pool, &mut text_refs)?;
        let frequency = u32::try_from(frequency)
            .map_err(|_| Error::Internal("bigram 频率超过 u32 上限".to_owned()))?;

        let mut record = [0u8; BIGRAM_RECORD_SIZE];
        record[0..4].copy_from_slice(&previous_offset.to_le_bytes());
        record[4..6].copy_from_slice(&previous_len.to_le_bytes());
        record[6..10].copy_from_slice(&word_offset.to_le_bytes());
        record[10..12].copy_from_slice(&word_len.to_le_bytes());
        record[12..16].copy_from_slice(&frequency.to_le_bytes());
        bigram_bytes.extend_from_slice(&record);
    }

    // 中文词到译文索引：同一中文词只保留首个（频率最高、输入顺序稳定的）译文。
    let mut translation_by_word: HashMap<&str, &DictionaryEntry> = HashMap::new();
    for entry in &sorted_entries {
        if entry
            .translation
            .as_deref()
            .is_some_and(|text| !text.is_empty())
        {
            translation_by_word
                .entry(entry.word.as_str())
                .or_insert(entry);
        }
    }
    let mut translation_sources: Vec<TranslationSource<'_>> = Vec::new();
    for (word, entry) in translation_by_word {
        let Some(translation) = entry.translation.as_deref() else {
            continue;
        };
        translation_sources.push(TranslationSource { word, translation });
    }
    translation_sources.sort_by(|a, b| a.word.cmp(b.word));

    let mut word_translation_bytes =
        Vec::with_capacity(WORD_TRANSLATION_RECORD_SIZE * translation_sources.len());
    let mut reverse_sources: Vec<(String, &str, u32, u16, u32, u16)> = Vec::new();
    for source in &translation_sources {
        let (word_offset, word_len) = intern(source.word, &mut text_pool, &mut text_refs)?;
        let (translation_offset, translation_len) =
            intern(source.translation, &mut text_pool, &mut text_refs)?;

        let mut record = [0u8; WORD_TRANSLATION_RECORD_SIZE];
        record[0..4].copy_from_slice(&word_offset.to_le_bytes());
        record[4..6].copy_from_slice(&word_len.to_le_bytes());
        record[6..10].copy_from_slice(&translation_offset.to_le_bytes());
        record[10..12].copy_from_slice(&translation_len.to_le_bytes());
        word_translation_bytes.extend_from_slice(&record);

        let key = normalize_translation_key(strip_pos_prefix(source.translation));
        let (key_offset, key_len) = intern(&key, &mut text_pool, &mut text_refs)?;
        reverse_sources.push((key, source.word, key_offset, key_len, word_offset, word_len));
    }
    reverse_sources.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(b.1)));

    let mut reverse_translation_bytes =
        Vec::with_capacity(REVERSE_TRANSLATION_RECORD_SIZE * reverse_sources.len());
    for (_, _, key_offset, key_len, word_offset, word_len) in reverse_sources {
        let mut record = [0u8; REVERSE_TRANSLATION_RECORD_SIZE];
        record[0..4].copy_from_slice(&key_offset.to_le_bytes());
        record[4..6].copy_from_slice(&key_len.to_le_bytes());
        record[6..10].copy_from_slice(&word_offset.to_le_bytes());
        record[10..12].copy_from_slice(&word_len.to_le_bytes());
        reverse_translation_bytes.extend_from_slice(&record);
    }

    let pinyin_index_offset = HEADER_SIZE as u64;
    let entry_table_offset = pinyin_index_offset
        .checked_add(
            u64::try_from(index_bytes.len())
                .map_err(|_| Error::Internal("拼音索引区长度溢出".to_owned()))?,
        )
        .ok_or_else(|| Error::Internal("拼音索引区偏移溢出".to_owned()))?;
    let bigram_offset = entry_table_offset
        .checked_add(
            u64::try_from(entry_bytes.len())
                .map_err(|_| Error::Internal("词条区长度溢出".to_owned()))?,
        )
        .ok_or_else(|| Error::Internal("词条区偏移溢出".to_owned()))?;
    let word_translation_offset = bigram_offset
        .checked_add(
            u64::try_from(bigram_bytes.len())
                .map_err(|_| Error::Internal("bigram 区长度溢出".to_owned()))?,
        )
        .ok_or_else(|| Error::Internal("bigram 区偏移溢出".to_owned()))?;
    let reverse_translation_offset = word_translation_offset
        .checked_add(
            u64::try_from(word_translation_bytes.len())
                .map_err(|_| Error::Internal("译文索引区长度溢出".to_owned()))?,
        )
        .ok_or_else(|| Error::Internal("译文索引区偏移溢出".to_owned()))?;
    let text_pool_offset = reverse_translation_offset
        .checked_add(
            u64::try_from(reverse_translation_bytes.len())
                .map_err(|_| Error::Internal("反查索引区长度溢出".to_owned()))?,
        )
        .ok_or_else(|| Error::Internal("反查索引区偏移溢出".to_owned()))?;

    let content_len = index_bytes
        .len()
        .checked_add(entry_bytes.len())
        .and_then(|len| len.checked_add(bigram_bytes.len()))
        .and_then(|len| len.checked_add(word_translation_bytes.len()))
        .and_then(|len| len.checked_add(reverse_translation_bytes.len()))
        .and_then(|len| len.checked_add(text_pool.len()))
        .ok_or_else(|| Error::Internal("内容区长度溢出".to_owned()))?;
    let mut content = Vec::with_capacity(content_len);
    content.extend_from_slice(&index_bytes);
    content.extend_from_slice(&entry_bytes);
    content.extend_from_slice(&bigram_bytes);
    content.extend_from_slice(&word_translation_bytes);
    content.extend_from_slice(&reverse_translation_bytes);
    content.extend_from_slice(&text_pool);

    let header = crate::dict_format::DictHeader {
        pinyin_index_count: u32::try_from(index_bytes.len() / PINYIN_INDEX_RECORD_SIZE)
            .map_err(|_| Error::Internal("拼音索引数量溢出".to_owned()))?,
        entry_count: u32::try_from(sorted_entries.len())
            .map_err(|_| Error::Internal("词条数量溢出".to_owned()))?,
        bigram_count: u32::try_from(bigram_bytes.len() / BIGRAM_RECORD_SIZE)
            .map_err(|_| Error::Internal("bigram 数量溢出".to_owned()))?,
        word_translation_count: u32::try_from(
            word_translation_bytes.len() / WORD_TRANSLATION_RECORD_SIZE,
        )
        .map_err(|_| Error::Internal("译文索引数量溢出".to_owned()))?,
        reverse_translation_count: u32::try_from(
            reverse_translation_bytes.len() / REVERSE_TRANSLATION_RECORD_SIZE,
        )
        .map_err(|_| Error::Internal("反查索引数量溢出".to_owned()))?,
        pinyin_index_offset,
        entry_table_offset,
        bigram_offset,
        word_translation_offset,
        reverse_translation_offset,
        text_pool_offset,
        content_hash: [0u8; 32],
    };

    let mut output = header.to_bytes().to_vec();
    output.extend_from_slice(&content);
    let hash = content_sha256(&output[HEADER_SIZE..]);
    output[80..112].copy_from_slice(&hash);
    debug_assert!(output.len() >= HEADER_SIZE + TEXT_POOL_START as usize);
    Ok(output)
}

/// 文本池登记：命中缓存直接返回既有引用，否则写入池尾并登记。
fn intern(
    text: &str,
    text_pool: &mut Vec<u8>,
    text_refs: &mut HashMap<String, TextRef>,
) -> Result<TextRef> {
    if text.is_empty() {
        return Ok((0, 0));
    }
    if let Some(&text_ref) = text_refs.get(text) {
        return Ok(text_ref);
    }
    let len = text.len();
    if len > u16::MAX as usize {
        return Err(Error::Internal("文本长度超过 u16 上限".to_owned()));
    }
    let offset =
        u32::try_from(text_pool.len()).map_err(|_| Error::Internal("文本池偏移溢出".to_owned()))?;
    if u64::from(offset)
        .checked_add(len as u64)
        .filter(|end| *end <= u32::MAX as u64)
        .is_none()
    {
        return Err(Error::Internal("文本池末端超过 u32 上限".to_owned()));
    }
    text_pool.extend_from_slice(text.as_bytes());
    let text_ref = (offset, len as u16);
    text_refs.insert(text.to_owned(), text_ref);
    Ok(text_ref)
}

#[cfg(test)]
mod tests {
    use super::build_v2;
    use crate::demo::{seed_bigrams, seed_entries};
    use crate::dict::DictionaryEntry;
    use crate::dict_format::{DictHeader, DICT_VERSION, HEADER_SIZE};

    #[test]
    fn 种子构建输出头部可解析且哈希写入() {
        let bytes = build_v2(&seed_entries(), &seed_bigrams()).unwrap();
        let header = DictHeader::from_bytes(&bytes).unwrap();
        assert_eq!(header.entry_count, 20);
        assert_eq!(header.bigram_count, 10);
        assert_eq!(
            crate::dict_format::content_sha256(&bytes[HEADER_SIZE..]),
            header.content_hash
        );
        assert_eq!(header.word_translation_count, 19);
        assert_eq!(header.reverse_translation_count, 19);
        assert!(header.pinyin_index_count > 0);
        assert!(bytes.len() > HEADER_SIZE);
    }

    #[test]
    fn 相同输入两次构建字节一致() {
        let entries = seed_entries();
        let bigrams = seed_bigrams();
        let first = build_v2(&entries, &bigrams).unwrap();
        let second = build_v2(&entries, &bigrams).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn 拒绝空词条与超限词频() {
        let entries = seed_entries();
        let bigrams = seed_bigrams();
        let mut bad_entries = entries.clone();
        bad_entries.push(DictionaryEntry::new("", "kong", 1));
        assert!(build_v2(&bad_entries, &bigrams).is_err());

        let mut big_frequency = entries.clone();
        big_frequency.push(DictionaryEntry::new(
            "超频",
            "chaopin",
            u64::from(u32::MAX) + 1,
        ));
        assert!(build_v2(&big_frequency, &bigrams).is_err());
    }

    #[test]
    fn 重复bigram合并而不是重复记录() {
        let entries = seed_entries();
        let bigrams = vec![("你好", "世界", 10), ("你好", "世界", 5)];
        let bytes = build_v2(&entries, &bigrams).unwrap();
        let header = DictHeader::from_bytes(&bytes).unwrap();
        assert_eq!(header.bigram_count, 1);
        assert_eq!(DICT_VERSION, 2);
    }

    #[test]
    fn 同一中文词只保留首个译文() {
        let entries = vec![
            DictionaryEntry::new("你好", "nihao", 100).with_translation("hello"),
            DictionaryEntry::new("你好", "nihao", 1).with_translation("hi"),
            DictionaryEntry::new("世界", "shijie", 90).with_translation("world"),
        ];
        let bytes = build_v2(&entries, &[]).unwrap();
        let header = DictHeader::from_bytes(&bytes).unwrap();
        assert_eq!(header.word_translation_count, 2);
        assert_eq!(header.reverse_translation_count, 2);
    }
}
