//! v 模式符号表（FR-028，场景 7）与符号大全面板分类表（FR-040/D-33，T-088 扩充）。
//!
//! 空闲态以 `v` 开头进入符号分支，后接类型码（1-9/x/h）给出对应符号组：
//! - `v1`（及 `v2`-`v9`）→ 序号组；
//! - `vx` → 数学符号组；
//! - `vh` → 常用标点组。
//!
//! 每组固定 9 个候选（一页），按确定性顺序展示；未识别的类型码返回空。
//! v 模式三组保持 9 词不变；符号大全面板（`zhu-ye-settings` 的 panel.rs）在此基础上
//! 按 `EXTRA_SYMBOL_GROUPS` 分类扩充（D-33，规模 150-300 字符），面板每页 60 格复用。

/// 序号组（`v1`-`v9`）。
pub const NUMBER_SYMBOLS: [&str; 9] = ["①", "②", "③", "④", "⑤", "⑥", "⑦", "⑧", "⑨"];
/// 数学符号组（`vx`）。
pub const MATH_SYMBOLS: [&str; 9] = ["±", "×", "÷", "≈", "≠", "≤", "≥", "∞", "％"];
/// 常用标点组（`vh`）。
pub const PUNCT_SYMBOLS: [&str; 9] = ["，", "。", "！", "？", "、", "；", "：", "“", "”"];

/// 符号大全扩展分类（D-33）：`(组名, 符号列表)`，供面板平铺展示。
///
/// 不参与 v 模式类型码（v1-9/vx/vh 保持原状），只扩充面板数据。字符均为静态常量，
/// 无外部数据源；渲染依赖系统字体覆盖（主流 Windows 字体已覆盖常用几何/货币/箭头/
/// 希腊字母/制表符）。
pub const EXTRA_SYMBOL_GROUPS: [(&str, &[&str]); 9] = [
    (
        "货币",
        &["＄", "￥", "€", "£", "¥", "¢", "₩", "₹", "₽", "₪", "₫", "₿"],
    ),
    (
        "箭头",
        &[
            "↑", "↓", "←", "→", "↖", "↗", "↘", "↙", "↕", "↔", "↩", "↪", "⇄", "⇅", "⇆", "⇇", "⇈",
            "⇉", "⇊", "⇐", "⇑", "⇒", "⇓", "⇔", "⇕", "⟵", "⟶", "⟷", "⟲", "⟳",
        ],
    ),
    (
        "希腊大写",
        &[
            "Α", "Β", "Γ", "Δ", "Ε", "Ζ", "Η", "Θ", "Ι", "Κ", "Λ", "Μ", "Ν", "Ξ", "Ο", "Π", "Ρ",
            "Σ", "Τ", "Υ", "Φ", "Χ", "Ψ", "Ω",
        ],
    ),
    (
        "希腊小写",
        &[
            "α", "β", "γ", "δ", "ε", "ζ", "η", "θ", "ι", "κ", "λ", "μ", "ν", "ξ", "ο", "π", "ρ",
            "σ", "τ", "υ", "φ", "χ", "ψ", "ω",
        ],
    ),
    ("注音声调", &["ˉ", "ˊ", "ˇ", "ˋ", "˙", "̄", "́", "̈", "̃"]),
    (
        "制表符",
        &[
            "┬", "┴", "┤", "├", "─", "┼", "┌", "┐", "└", "┘", "│", "┃", "═", "║", "╔", "╗", "╚",
            "╝", "╠", "╣", "╦", "╩", "╬", "╭", "╮", "╰", "╯",
        ],
    ),
    (
        "装饰",
        &[
            "★", "☆", "♠", "♣", "♥", "♦", "♪", "♫", "♬", "☀", "☁", "☂", "☃", "☎", "☺", "☻", "☼",
            "✿", "❀", "❁", "❄", "❅", "✧", "✦",
        ],
    ),
    (
        "数学扩展",
        &[
            "＋", "－", "＜", "＞", "＝", "≦", "≧", "≡", "≒", "√", "∛", "∜", "∠", "⊥", "∥", "∝",
            "∑", "∏", "∫", "∂", "∆", "∇", "∈", "∉", "⊆", "⊇", "∪", "∩", "∅", "∘",
        ],
    ),
    (
        "罗马数字",
        &["Ⅰ", "Ⅱ", "Ⅲ", "Ⅳ", "Ⅴ", "Ⅵ", "Ⅶ", "Ⅷ", "Ⅸ", "Ⅹ", "Ⅺ", "Ⅻ"],
    ),
];

/// 全部面板符号分类（含 v 模式三组），供面板平铺与计数。
#[must_use]
pub fn all_panel_groups() -> Vec<(&'static str, &'static [&'static str])> {
    let mut groups: Vec<(&'static str, &'static [&'static str])> = vec![
        ("序号", &NUMBER_SYMBOLS),
        ("数学", &MATH_SYMBOLS),
        ("标点", &PUNCT_SYMBOLS),
    ];
    groups.extend(EXTRA_SYMBOL_GROUPS);
    groups
}

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
    use super::{
        all_panel_groups, symbol_group, EXTRA_SYMBOL_GROUPS, MATH_SYMBOLS, NUMBER_SYMBOLS,
        PUNCT_SYMBOLS,
    };

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
        for (name, group) in EXTRA_SYMBOL_GROUPS {
            let mut seen = std::collections::HashSet::new();
            for s in group {
                assert!(seen.insert(*s), "符号重复 [{name}]: {s}");
            }
        }
    }

    #[test]
    fn 面板扩充规模在预算内且组名唯一() {
        let groups = all_panel_groups();
        let total: usize = groups.iter().map(|(_, group)| group.len()).sum();
        assert!(
            (150..=300).contains(&total),
            "符号大全总规模 {total} 应在 150-300"
        );
        let mut names = std::collections::HashSet::new();
        for (name, _) in &groups {
            assert!(names.insert(*name), "组名重复: {name}");
        }
        // 各分组不空。
        for (name, group) in &groups {
            assert!(!group.is_empty(), "分组 {name} 为空");
        }
    }

    #[test]
    fn v模式三组未被面板扩充污染() {
        // D-33 只扩充面板，v 模式类型码行为保持：三组各 9 项。
        assert_eq!(NUMBER_SYMBOLS.len(), 9);
        assert_eq!(MATH_SYMBOLS.len(), 9);
        assert_eq!(PUNCT_SYMBOLS.len(), 9);
    }
}
