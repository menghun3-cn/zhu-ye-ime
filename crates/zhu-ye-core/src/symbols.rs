//! v 模式符号表（FR-028，场景 7）。
//!
//! 空闲态以 `v` 开头进入符号分支，后接类型码（1-9/x/h）给出对应符号组：
//! - `v1`（及 `v2`-`v9`）→ 序号组；
//! - `vx` → 数学符号组；
//! - `vh` → 常用标点组。
//!
//! 每组固定 9 个候选（一页），按确定性顺序展示；未识别的类型码返回空。

/// 序号组（`v1`-`v9`）。
pub const NUMBER_SYMBOLS: [&str; 9] = ["①", "②", "③", "④", "⑤", "⑥", "⑦", "⑧", "⑨"];
/// 数学符号组（`vx`）。
pub const MATH_SYMBOLS: [&str; 9] = ["±", "×", "÷", "≈", "≠", "≤", "≥", "∞", "％"];
/// 常用标点组（`vh`）。
pub const PUNCT_SYMBOLS: [&str; 9] = ["，", "。", "！", "？", "、", "；", "：", "“", "”"];

/// 按类型码取符号组；未识别返回 `None`（v 模式应回退拼音路径）。
#[must_use]
pub fn symbol_group(code: char) -> Option<&'static [&'static str; 9]> {
    match code {
        '1'..='9' => Some(&NUMBER_SYMBOLS),
        'x' | 'X' => Some(&MATH_SYMBOLS),
        'h' | 'H' => Some(&PUNCT_SYMBOLS),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{symbol_group, MATH_SYMBOLS, NUMBER_SYMBOLS, PUNCT_SYMBOLS};

    #[test]
    fn 序号组九项() {
        let group = symbol_group('1').expect("v1 应命中序号组");
        assert_eq!(group, &NUMBER_SYMBOLS);
        assert_eq!(group[0], "①");
        assert_eq!(group.len(), 9);
        assert_eq!(symbol_group('9'), Some(&NUMBER_SYMBOLS));
    }

    #[test]
    fn 数字类型码共享序号组() {
        for code in '1'..='9' {
            assert_eq!(symbol_group(code), Some(&NUMBER_SYMBOLS));
        }
    }

    #[test]
    fn 数学与标点组() {
        assert_eq!(symbol_group('x'), Some(&MATH_SYMBOLS));
        assert_eq!(symbol_group('h'), Some(&PUNCT_SYMBOLS));
        assert_eq!(symbol_group('X'), Some(&MATH_SYMBOLS));
        assert_eq!(symbol_group('H'), Some(&PUNCT_SYMBOLS));
    }

    #[test]
    fn 未识别类型码返回空() {
        assert_eq!(symbol_group('i'), None);
        assert_eq!(symbol_group('a'), None);
        assert_eq!(symbol_group('0'), None);
    }

    #[test]
    fn 组内无重复() {
        for group in [&NUMBER_SYMBOLS, &MATH_SYMBOLS, &PUNCT_SYMBOLS] {
            let mut seen = std::collections::HashSet::new();
            for s in group {
                assert!(seen.insert(*s), "符号重复: {s}");
            }
        }
    }
}
