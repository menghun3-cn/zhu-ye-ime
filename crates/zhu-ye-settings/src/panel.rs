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

use serde::{Deserialize, Serialize};

use zhu_ye_core::EMOJI_TABLE;

/// 面板每页列数与行数。
pub const PANEL_COLUMNS: usize = 10;
/// 面板每页行数。
pub const PANEL_ROWS: usize = 6;
/// 面板每页格数。
pub const PANEL_PAGE_SIZE: usize = PANEL_COLUMNS * PANEL_ROWS;

/// 非 ASCII 别名（中文关键词）的归组标记。
pub const OTHER_GROUP: &str = "#";

/// 面板种类。
///
/// `Serialize`/`Deserialize` 用于面板状态文件（T-151 页签记忆，S-33）：变体名落入
/// `panel.json`（`"Emoji"`/`"Symbol"`），旧文件/损坏文件解析失败回默认。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

/// 符号全量条目：三组各 9 个 + 扩展分类（D-33，T-088 扩充至 150-300 字符），
/// 组名直接作为分组与副标签。
fn symbol_entries() -> &'static [PanelEntry] {
    static ENTRIES: OnceLock<Vec<PanelEntry>> = OnceLock::new();
    ENTRIES.get_or_init(|| {
        let mut entries = Vec::new();
        for (group, symbols) in zhu_ye_core::all_panel_groups() {
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

/// 按查询过滤面板条目（T-151 搜索态，S-32）。
///
/// 匹配 `label`（emoji 为拼音别名）/ `group` / `text`（字符本身）三者的**子串**，
/// 大小写不敏感；空查询返回全量（按原表顺序）。emoji 按拼音别名过滤是其主用途——
/// 搜索框经 `WM_CHAR` 只接收 ASCII，中文别名查询暂不可达（记录在案的已知限制，
/// 需求 §23.3 非目标）。
#[must_use]
pub fn filter_entries(kind: PanelKind, query: &str) -> Vec<PanelEntry> {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return kind.entries().to_vec();
    }
    kind.entries()
        .iter()
        .copied()
        .filter(|entry| {
            entry.label.to_lowercase().contains(&query)
                || entry.group.to_lowercase().contains(&query)
                || entry.text.to_lowercase().contains(&query)
        })
        .collect()
}

/// 对任意条目序列分页（搜索命中跨页浏览与全量浏览共用同一口径）。
///
/// 与 [`PanelView::visible`] 的分页规则一致：空序列也至少一页（页码 `1/1`、空页）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageSlice<'a> {
    /// 本页条目。
    pub items: &'a [PanelEntry],
    /// 当前页（从 0 起）。
    pub page: usize,
    /// 总页数。
    pub page_count: usize,
}

/// 对条目序列按页切片。
#[must_use]
pub fn page_of<'a>(items: &'a [PanelEntry], page: usize, page_size: usize) -> PageSlice<'a> {
    let page_size = page_size.max(1);
    let page_count = items.len().div_ceil(page_size).max(1);
    let page = page.min(page_count - 1);
    let start = (page * page_size).min(items.len());
    let end = (start + page_size).min(items.len());
    PageSlice {
        items: &items[start..end],
        page,
        page_count,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        filter_entries, group_of, page_of, PanelEntry, PanelKind, PanelView, OTHER_GROUP,
        PANEL_COLUMNS, PANEL_PAGE_SIZE, PANEL_ROWS,
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
    fn 符号面板按全部组展开且规模合规() {
        let entries = PanelKind::Symbol.entries();
        // T-088 扩展后总规模在 150-300（D-33），v 模式三组仍各 9 个打头。
        assert!(
            (150..=300).contains(&entries.len()),
            "符号大全规模 {} 应在 150-300",
            entries.len()
        );
        let original: [(&str, usize); 3] = [("序号", 9), ("数学", 9), ("标点", 9)];
        for (group, count) in original {
            let got = entries.iter().filter(|entry| entry.group == group).count();
            assert_eq!(got, count, "{group} 组符号数保持 {count}");
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

    #[test]
    fn 搜索空查询返回全量且顺序不变() {
        let all = PanelKind::Emoji.entries();
        let hits = filter_entries(PanelKind::Emoji, "  ");
        assert_eq!(hits, all.to_vec(), "空/空白查询 = 全量且顺序不变");
    }

    #[test]
    fn 搜索按拼音别名子串过滤且大小写不敏感() {
        // 别名 `ai`/`aixin` 存在（emoji 按字符去重后至少保留一个）；输入 `ai` 应命中其
        // 全部别名含 `ai` 的 emoji，且每个命中项的别名/字符至少一处包含查询。
        let hits = filter_entries(PanelKind::Emoji, "AI");
        assert!(!hits.is_empty(), "`ai` 常见别名必有命中");
        let entry = hits[0];
        let label_has = entry.label.to_lowercase().contains("ai");
        let text_has = entry.text.to_lowercase().contains("ai");
        let group_has = entry.group.to_lowercase().contains("ai");
        assert!(label_has || text_has || group_has, "命中项必须匹配");
    }

    #[test]
    fn 搜索按字符与组名匹配() {
        // 字符匹配：查询"★"命中装饰组里的实心星。
        let hits = filter_entries(PanelKind::Symbol, "★");
        assert!(!hits.is_empty(), "符号表含 ★");
        assert!(hits.iter().any(|e| e.text == "★"), "命中含 ★ 本身");
        // 组名匹配：查询中文组名（纯函数不限于 ASCII）命中的都属于该组。
        let by_group = filter_entries(PanelKind::Symbol, "箭头");
        assert!(!by_group.is_empty(), "箭头组必有符号");
        assert!(by_group.iter().all(|e| e.group == "箭头"), "命中均属箭头组");
    }

    #[test]
    fn 通用分页与面板分页口径一致() {
        // page_of 在"空序列"下也至少一页，与 PanelView::page_count 的空口径一致。
        let empty: Vec<PanelEntry> = Vec::new();
        let page = page_of(&empty, 0, PANEL_PAGE_SIZE);
        assert_eq!(page.items.len(), 0);
        assert_eq!(page.page_count, 1);
        assert_eq!(page.page, 0);
        // 越界页钳制到末页。
        let all = PanelKind::Symbol.entries().to_vec();
        let page = page_of(&all[..], 999, PANEL_PAGE_SIZE);
        assert_eq!(page.page, page.page_count - 1);
        assert!(!page.items.is_empty());
        // 与 PanelView::visible 口径一致：全量时首页切片相同。
        let view = PanelView::new(PanelKind::Emoji);
        let page0 = page_of(PanelKind::Emoji.entries(), 0, PANEL_PAGE_SIZE);
        assert_eq!(page0.items, view.visible());
    }
}
