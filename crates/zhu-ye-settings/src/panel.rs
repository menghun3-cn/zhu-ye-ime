//! 工具箱面板的数据与分页（纯逻辑，无 Win32 依赖，可无 GUI 单测）。
//!
//! 需求 FR-040 要求工具箱提供 emoji 面板与符号大全面板。两份数据都已在 `zhu-ye-core`
//! 内（`EMOJI_TABLE` 与符号三组），本模块只做"呈现所需"的整理：
//!
//! - **emoji 必须按字符去重**：表以别名为键，同一 emoji 存在多个别名（`ai`/`aixin` 同为
//!   ❤️），直接平铺会出现重复格子；
//! - **分组取别名首字母**：表内没有分类字段，而分组是浏览 300+ 条目时的必要导航；
//!   表按别名字节序升序，因此分组顺序天然确定（`a`…`z` 之后是非 ASCII 别名的 `#`）。
//!
//! 面板渲染在哪一层（设置窗口内的子视图，还是独立的非激活弹窗）由 T-074 的实现决定，
//! 本模块只提供数据与分页，两种形态共用。

use std::sync::OnceLock;

use zhu_ye_core::{EMOJI_TABLE, MATH_SYMBOLS, NUMBER_SYMBOLS, PUNCT_SYMBOLS};

/// 面板每页列数与行数。
pub const PANEL_COLUMNS: usize = 10;
/// 面板每页行数。
pub const PANEL_ROWS: usize = 6;
/// 面板每页格数。
pub const PANEL_PAGE_SIZE: usize = PANEL_COLUMNS * PANEL_ROWS;

/// 非 ASCII 别名（中文关键词）的归组标记。
pub const OTHER_GROUP: &str = "#";

/// 面板种类。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelKind {
    /// emoji 面板。
    Emoji,
    /// 符号大全面板。
    Symbol,
}

impl PanelKind {
    /// 面板标题。
    #[must_use]
    pub const fn title(self) -> &'static str {
        match self {
            Self::Emoji => "emoji 面板",
            Self::Symbol => "符号大全",
        }
    }

    /// 面板条目（全量，未分页）。
    #[must_use]
    pub fn entries(self) -> &'static [PanelEntry] {
        match self {
            Self::Emoji => emoji_entries(),
            Self::Symbol => symbol_entries(),
        }
    }
}

/// 面板中的一格。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PanelEntry {
    /// 选中后上屏（或复制）的字符。
    pub text: &'static str,
    /// 显示用的副标签；emoji 为拼音别名，符号为组名。
    pub label: &'static str,
    /// 分组名；emoji 为别名首字母大写或 `#`，符号为组名。
    pub group: &'static str,
}

/// emoji 全量条目：按 emoji 字符去重后，保留首次出现的别名。
fn emoji_entries() -> &'static [PanelEntry] {
    static ENTRIES: OnceLock<Vec<PanelEntry>> = OnceLock::new();
    ENTRIES.get_or_init(|| {
        let mut seen = std::collections::HashSet::new();
        let mut entries = Vec::new();
        for item in EMOJI_TABLE {
            // 去重键用 emoji 字符本身：同一字符的多个别名只保留最先出现的一个。
            if !seen.insert(item.emoji) {
                continue;
            }
            entries.push(PanelEntry {
                text: item.emoji,
                label: item.alias,
                group: group_of(item.alias),
            });
        }
        entries
    })
}

/// 符号全量条目：三组各 9 个，组名直接作为分组与副标签。
fn symbol_entries() -> &'static [PanelEntry] {
    static ENTRIES: OnceLock<Vec<PanelEntry>> = OnceLock::new();
    ENTRIES.get_or_init(|| {
        let mut entries = Vec::new();
        for (group, symbols) in [
            ("序号", NUMBER_SYMBOLS.as_slice()),
            ("数学", MATH_SYMBOLS.as_slice()),
            ("标点", PUNCT_SYMBOLS.as_slice()),
        ] {
            for symbol in symbols {
                entries.push(PanelEntry {
                    text: symbol,
                    label: group,
                    group,
                });
            }
        }
        entries
    })
}

/// 别名归组：ASCII 字母取大写首字母，其余（中文关键词等）归入 `#`。
#[must_use]
pub fn group_of(alias: &str) -> &'static str {
    const GROUPS: [&str; 26] = [
        "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R",
        "S", "T", "U", "V", "W", "X", "Y", "Z",
    ];
    match alias.as_bytes().first() {
        Some(byte) if byte.is_ascii_alphabetic() => {
            GROUPS[usize::from(byte.to_ascii_uppercase() - b'A')]
        }
        _ => OTHER_GROUP,
    }
}

/// 面板的分页状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PanelView {
    /// 面板种类。
    pub kind: PanelKind,
    /// 当前页（从 0 起）。
    pub page: usize,
}

impl PanelView {
    /// 打开面板，从第一页开始。
    #[must_use]
    pub fn new(kind: PanelKind) -> Self {
        Self { kind, page: 0 }
    }

    /// 总页数；空数据也至少一页，避免出现"0/0"。
    #[must_use]
    pub fn page_count(&self) -> usize {
        let total = self.kind.entries().len();
        total.div_ceil(PANEL_PAGE_SIZE).max(1)
    }

    /// 本页条目。
    #[must_use]
    pub fn visible(&self) -> &'static [PanelEntry] {
        let entries = self.kind.entries();
        let start = (self.page * PANEL_PAGE_SIZE).min(entries.len());
        let end = (start + PANEL_PAGE_SIZE).min(entries.len());
        &entries[start..end]
    }

    /// 翻到下一页；已在末页时不动。
    pub fn next_page(&mut self) {
        if self.page + 1 < self.page_count() {
            self.page += 1;
        }
    }

    /// 翻到上一页；已在首页时不动。
    pub fn prev_page(&mut self) {
        self.page = self.page.saturating_sub(1);
    }

    /// 页码标签，形如 `2/4`。
    #[must_use]
    pub fn page_label(&self) -> String {
        format!("{}/{}", self.page + 1, self.page_count())
    }

    /// 本页第 `index` 格对应的条目。
    #[must_use]
    pub fn entry_at(&self, index: usize) -> Option<PanelEntry> {
        self.visible().get(index).copied()
    }

    /// 本页条目总数。
    #[must_use]
    pub fn visible_len(&self) -> usize {
        self.visible().len()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        group_of, PanelKind, PanelView, OTHER_GROUP, PANEL_COLUMNS, PANEL_PAGE_SIZE, PANEL_ROWS,
    };
    use std::collections::HashSet;

    #[test]
    fn 每页格数为列乘行() {
        assert_eq!(PANEL_PAGE_SIZE, PANEL_COLUMNS * PANEL_ROWS);
    }

    #[test]
    fn emoji按字符去重且数量少于别名条数() {
        let entries = PanelKind::Emoji.entries();
        let unique: HashSet<&str> = entries.iter().map(|entry| entry.text).collect();
        assert_eq!(
            unique.len(),
            entries.len(),
            "面板中每个 emoji 字符只能出现一次"
        );
        assert!(
            entries.len() < zhu_ye_core::EMOJI_TABLE.len(),
            "别名表存在同一 emoji 的多别名，去重后必然更少：{} vs {}",
            entries.len(),
            zhu_ye_core::EMOJI_TABLE.len()
        );
        // 去重保留首次出现的别名，因此每条都必须是表内真实存在的别名。
        let aliases: HashSet<&str> = zhu_ye_core::EMOJI_TABLE
            .iter()
            .map(|item| item.alias)
            .collect();
        for entry in entries {
            assert!(
                aliases.contains(entry.label),
                "别名不在表内：{}",
                entry.label
            );
        }
    }

    #[test]
    fn 同一emoji只保留首个别名() {
        // ❤️ 在表内有 ai / aixin 两个别名（ai 在前），面板应保留 ai。
        let hearts: Vec<&str> = PanelKind::Emoji
            .entries()
            .iter()
            .filter(|entry| entry.text == "❤️")
            .map(|entry| entry.label)
            .collect();
        assert_eq!(hearts, vec!["ai"]);
    }

    #[test]
    fn 别名归组取首字母大写其余归井号() {
        assert_eq!(group_of("ai"), "A");
        assert_eq!(group_of("zhuye"), "Z");
        assert_eq!(group_of(""), OTHER_GROUP);
        assert_eq!(group_of("爱心"), OTHER_GROUP);
        assert_eq!(group_of("1a"), OTHER_GROUP, "非字母开头归入 #");
    }

    #[test]
    fn emoji分组顺序确定且符合字节序() {
        // 表按别名字节序升序，分组首次出现的顺序应与之一致：字母组在前，# 在后。
        let mut order = Vec::new();
        for entry in PanelKind::Emoji.entries() {
            if order.last() != Some(&entry.group) && !order.contains(&entry.group) {
                order.push(entry.group);
            }
        }
        let hash_position = order.iter().position(|group| *group == OTHER_GROUP);
        if let Some(position) = hash_position {
            assert_eq!(
                position,
                order.len() - 1,
                "非 ASCII 别名排在最后，所以 # 组只能是最后一组"
            );
        }
        let letters: Vec<&str> = order
            .iter()
            .copied()
            .filter(|group| *group != OTHER_GROUP)
            .collect();
        let mut sorted = letters.clone();
        sorted.sort_unstable();
        // 分组出现的顺序即增序（不要求连续，只要求单调）。
        for pair in letters.windows(2) {
            assert!(pair[0] <= pair[1], "字母分组顺序应单调递增：{letters:?}");
        }
    }

    #[test]
    fn 符号面板三组各九个() {
        let entries = PanelKind::Symbol.entries();
        assert_eq!(entries.len(), 27);
        for group in ["序号", "数学", "标点"] {
            let count = entries.iter().filter(|entry| entry.group == group).count();
            assert_eq!(count, 9, "{group} 组应有 9 个符号");
        }
        assert_eq!(entries[0].text, "①");
        assert_eq!(entries[9].text, "±");
        assert_eq!(entries[18].text, "，");
    }

    #[test]
    fn 面板内容无重复字符() {
        for kind in [PanelKind::Emoji, PanelKind::Symbol] {
            let mut seen = HashSet::new();
            for entry in kind.entries() {
                assert!(
                    seen.insert(entry.text),
                    "{:?} 出现重复：{}",
                    kind,
                    entry.text
                );
            }
        }
    }

    #[test]
    fn 分页覆盖全部条目且每页不超页容量() {
        for kind in [PanelKind::Emoji, PanelKind::Symbol] {
            let mut view = PanelView::new(kind);
            let total = kind.entries().len();
            let mut collected = 0;
            for _ in 0..view.page_count() {
                assert!(view.visible_len() <= PANEL_PAGE_SIZE);
                collected += view.visible_len();
                view.next_page();
            }
            assert_eq!(collected, total, "{kind:?} 分页应恰好覆盖全部条目");
        }
    }

    #[test]
    fn 翻页不越界且页码标签从一起算() {
        let mut view = PanelView::new(PanelKind::Emoji);
        assert_eq!(view.page, 0);
        view.prev_page();
        assert_eq!(view.page, 0, "首页再上翻不动");
        assert!(view.page_label().starts_with("1/"), "{}", view.page_label());

        let last = view.page_count() - 1;
        for _ in 0..view.page_count() + 3 {
            view.next_page();
        }
        assert_eq!(view.page, last, "末页再下翻不动");
        assert_eq!(view.page_label(), format!("{}/{}", last + 1, last + 1));
    }

    #[test]
    fn 末页取条目与格子数一致() {
        let mut view = PanelView::new(PanelKind::Emoji);
        while view.page + 1 < view.page_count() {
            view.next_page();
        }
        let len = view.visible_len();
        assert!(len > 0 && len <= PANEL_PAGE_SIZE);
        assert!(view.entry_at(len - 1).is_some());
        assert!(view.entry_at(len).is_none(), "越界格子必须为 None");
        assert_eq!(view.entry_at(0), view.visible().first().copied());
    }
}
