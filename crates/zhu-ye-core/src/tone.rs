//! 带调拼音显示表（T-112 后续验收批四：候选拼音显示声调）。
//!
//! 词典（zyct）拼音键是**无调全拼**（输入匹配用）；候选窗上方的小字
//! 拼音行需要**带调**形式（`生成` → `shēng chéng`）。带调拼音由构建
//! 期（`zhu-ye-dict` build_base）从 CC-CEDICT 词级数字调注音与
//! kTGHZ2013 字级符号调注音导出为旁挂文本文件（`dictionary.zyct.tones`，
//! 与词典同目录），运行期由输入法加载成本表。zyct 格式因此零改动。

use std::collections::HashMap;

/// 带调拼音表：词级优先，字级兜底（逐字拼合）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ToneMap {
    /// 词 → 空格分隔的带调拼音（`想着` → `xiǎng zhe`）。
    pub words: HashMap<String, String>,
    /// 字 → 带调拼音（`着` → `zhe`，轻声无调号）。
    pub chars: HashMap<char, String>,
}

impl ToneMap {
    /// 解析旁挂文本：`#word` 段（词<TAB>带调拼音）与 `#char` 段（字<TAB>带调音）。
    /// 未知段名忽略；坏行跳过。
    #[must_use]
    pub fn from_lines(text: &str) -> Self {
        let mut map = ToneMap::default();
        let mut section = "";
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if let Some(head) = line.strip_prefix('#') {
                section = head.trim();
                continue;
            }
            let Some((key, value)) = line.split_once('\t') else {
                continue;
            };
            match section {
                "word" => {
                    map.words.insert(key.to_owned(), value.to_owned());
                }
                "char" => {
                    let mut chars = key.chars();
                    let Some(ch) = chars.next() else {
                        continue;
                    };
                    if chars.next().is_none() {
                        map.chars.insert(ch, value.to_owned());
                    }
                }
                _ => {}
            }
        }
        map
    }

    /// 词带调拼音：词级命中直接返回；否则逐字查字级表拼合（空格分隔）。
    /// 任一字符缺字级音时返回 `None`（调用方回退无调拼音）。
    #[must_use]
    pub fn word_tone_spaced(&self, word: &str) -> Option<String> {
        if let Some(spaced) = self.words.get(word) {
            return Some(spaced.clone());
        }
        let mut syllables = Vec::with_capacity(word.chars().count());
        for ch in word.chars() {
            let syllable = self.chars.get(&ch)?;
            syllables.push(syllable.as_str());
        }
        Some(syllables.join(" "))
    }
}

#[cfg(test)]
mod tests {
    use super::ToneMap;

    fn sample() -> &'static str {
        "#word\n想着\txiǎng zhe\n正确\tzhèng què\n\n#char\n着\tzhe\n说\tshuō\n还\thái\n"
    }

    #[test]
    fn parses_word_and_char_sections() {
        let map = ToneMap::from_lines(sample());
        assert_eq!(map.words.len(), 2);
        assert_eq!(map.chars.len(), 3);
        assert_eq!(map.words.get("想着").map(String::as_str), Some("xiǎng zhe"));
        assert_eq!(map.chars.get(&'着').map(String::as_str), Some("zhe"));
    }

    #[test]
    fn word_hit_prefers_word_level() {
        let map = ToneMap::from_lines(sample());
        assert_eq!(map.word_tone_spaced("想着").as_deref(), Some("xiǎng zhe"));
    }

    #[test]
    fn char_fallback_joins_spaced() {
        let map = ToneMap::from_lines(sample());
        assert_eq!(map.word_tone_spaced("还说").as_deref(), Some("hái shuō"));
    }

    #[test]
    fn unknown_char_returns_none() {
        let map = ToneMap::from_lines(sample());
        assert_eq!(map.word_tone_spaced("想着X"), None);
        assert_eq!(ToneMap::default().word_tone_spaced("想着"), None);
    }

    #[test]
    fn skips_bad_lines_and_unknown_sections() {
        let map = ToneMap::from_lines("#word\n无制表符行\nx\ty\n#bogus\nz\tw\n");
        assert_eq!(map.words.len(), 1);
        assert_eq!(map.words.get("x").map(String::as_str), Some("y"));
        assert!(map.chars.is_empty());
    }
}
