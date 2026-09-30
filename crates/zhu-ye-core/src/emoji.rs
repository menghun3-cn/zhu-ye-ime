//! emoji 别名表（FR-029，场景 7）。
//!
//! 内置常用 emoji 的拼音别名（`xiao`→😄），按 alias 字节序升序维护保证确定性；
//! 上限 512 条（T-062 扩展至 300+ 仍在此内），查询用二分。
//! 字符均为 Unicode 标准字符，无外部数据源；首批 109 条高频，扩展见 T-062。

/// 别名表条目。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmojiEntry {
    /// 拼音/关键词别名（小写 ASCII 或中文）。
    pub alias: &'static str,
    /// 对应 emoji 字符。
    pub emoji: &'static str,
}

/// 首批高频别名表（按 alias 字节序升序；测试强制有序且无重复）。
pub static EMOJI_TABLE: &[EmojiEntry] = &[
    EmojiEntry {
        alias: "ai",
        emoji: "❤️",
    },
    EmojiEntry {
        alias: "aixin",
        emoji: "❤️",
    },
    EmojiEntry {
        alias: "bi",
        emoji: "✏️",
    },
    EmojiEntry {
        alias: "biye",
        emoji: "👋",
    },
    EmojiEntry {
        alias: "cai",
        emoji: "🥬",
    },
    EmojiEntry {
        alias: "caomei",
        emoji: "🍓",
    },
    EmojiEntry {
        alias: "cha",
        emoji: "👎",
    },
    EmojiEntry {
        alias: "changjinglu",
        emoji: "🦒",
    },
    EmojiEntry {
        alias: "che",
        emoji: "🚗",
    },
    EmojiEntry {
        alias: "dan",
        emoji: "🥚",
    },
    EmojiEntry {
        alias: "danche",
        emoji: "🚲",
    },
    EmojiEntry {
        alias: "dangao",
        emoji: "🎂",
    },
    EmojiEntry {
        alias: "deng",
        emoji: "💡",
    },
    EmojiEntry {
        alias: "diannao",
        emoji: "💻",
    },
    EmojiEntry {
        alias: "diqiu",
        emoji: "🌍",
    },
    EmojiEntry {
        alias: "dizhi",
        emoji: "📍",
    },
    EmojiEntry {
        alias: "e",
        emoji: "🦢",
    },
    EmojiEntry {
        alias: "feiji",
        emoji: "✈️",
    },
    EmojiEntry {
        alias: "feng",
        emoji: "🍃",
    },
    EmojiEntry {
        alias: "ganbei",
        emoji: "🥂",
    },
    EmojiEntry {
        alias: "gantan",
        emoji: "❗",
    },
    EmojiEntry {
        alias: "ganxie",
        emoji: "🙏",
    },
    EmojiEntry {
        alias: "gaoerfu",
        emoji: "⛳",
    },
    EmojiEntry {
        alias: "gou",
        emoji: "🐶",
    },
    EmojiEntry {
        alias: "haha",
        emoji: "😂",
    },
    EmojiEntry {
        alias: "hehe",
        emoji: "😁",
    },
    EmojiEntry {
        alias: "heixin",
        emoji: "🖤",
    },
    EmojiEntry {
        alias: "hongbao",
        emoji: "🧧",
    },
    EmojiEntry {
        alias: "hongdeng",
        emoji: "🚦",
    },
    EmojiEntry {
        alias: "hongxin",
        emoji: "❤️",
    },
    EmojiEntry {
        alias: "houzi",
        emoji: "🐵",
    },
    EmojiEntry {
        alias: "hua",
        emoji: "🌸",
    },
    EmojiEntry {
        alias: "huo",
        emoji: "🔥",
    },
    EmojiEntry {
        alias: "ji",
        emoji: "🐔",
    },
    EmojiEntry {
        alias: "jia",
        emoji: "🏠",
    },
    EmojiEntry {
        alias: "jian",
        emoji: "✂️",
    },
    EmojiEntry {
        alias: "jiayou",
        emoji: "💪",
    },
    EmojiEntry {
        alias: "jinbi",
        emoji: "🪙",
    },
    EmojiEntry {
        alias: "jinzhi",
        emoji: "🚫",
    },
    EmojiEntry {
        alias: "jitui",
        emoji: "🍗",
    },
    EmojiEntry {
        alias: "kafei",
        emoji: "☕",
    },
    EmojiEntry {
        alias: "keai",
        emoji: "🥰",
    },
    EmojiEntry {
        alias: "ku",
        emoji: "😂",
    },
    EmojiEntry {
        alias: "kuqi",
        emoji: "😭",
    },
    EmojiEntry {
        alias: "kuxiao",
        emoji: "😅",
    },
    EmojiEntry {
        alias: "lanqiu",
        emoji: "🏀",
    },
    EmojiEntry {
        alias: "laohu",
        emoji: "🐯",
    },
    EmojiEntry {
        alias: "leng",
        emoji: "🥶",
    },
    EmojiEntry {
        alias: "liwu",
        emoji: "🎁",
    },
    EmojiEntry {
        alias: "long",
        emoji: "🐲",
    },
    EmojiEntry {
        alias: "lvxing",
        emoji: "✈️",
    },
    EmojiEntry {
        alias: "ma",
        emoji: "🐴",
    },
    EmojiEntry {
        alias: "mao",
        emoji: "🐱",
    },
    EmojiEntry {
        alias: "meigui",
        emoji: "🌹",
    },
    EmojiEntry {
        alias: "mianbao",
        emoji: "🍞",
    },
    EmojiEntry {
        alias: "muma",
        emoji: "😘",
    },
    EmojiEntry {
        alias: "niao",
        emoji: "🐦",
    },
    EmojiEntry {
        alias: "niu",
        emoji: "🐮",
    },
    EmojiEntry {
        alias: "niunai",
        emoji: "🥛",
    },
    EmojiEntry {
        alias: "ok",
        emoji: "👌",
    },
    EmojiEntry {
        alias: "paobu",
        emoji: "🏃",
    },
    EmojiEntry {
        alias: "pengyou",
        emoji: "🤗",
    },
    EmojiEntry {
        alias: "pingguo",
        emoji: "🍎",
    },
    EmojiEntry {
        alias: "puke",
        emoji: "🃏",
    },
    EmojiEntry {
        alias: "qian",
        emoji: "💰",
    },
    EmojiEntry {
        alias: "qie",
        emoji: "🐧",
    },
    EmojiEntry {
        alias: "qinqin",
        emoji: "😘",
    },
    EmojiEntry {
        alias: "qiu",
        emoji: "⚽",
    },
    EmojiEntry {
        alias: "re",
        emoji: "🥵",
    },
    EmojiEntry {
        alias: "riqi",
        emoji: "📅",
    },
    EmojiEntry {
        alias: "shengqi",
        emoji: "😡",
    },
    EmojiEntry {
        alias: "shengri",
        emoji: "🎂",
    },
    EmojiEntry {
        alias: "shijian",
        emoji: "⏰",
    },
    EmojiEntry {
        alias: "shouji",
        emoji: "📱",
    },
    EmojiEntry {
        alias: "shu",
        emoji: "📖",
    },
    EmojiEntry {
        alias: "shui",
        emoji: "😴",
    },
    EmojiEntry {
        alias: "taiyang",
        emoji: "☀️",
    },
    EmojiEntry {
        alias: "tang",
        emoji: "🍬",
    },
    EmojiEntry {
        alias: "tianshi",
        emoji: "😇",
    },
    EmojiEntry {
        alias: "tu",
        emoji: "🐰",
    },
    EmojiEntry {
        alias: "wan'an",
        emoji: "🌙",
    },
    EmojiEntry {
        alias: "wangqiu",
        emoji: "🎾",
    },
    EmojiEntry {
        alias: "wei",
        emoji: "⚠️",
    },
    EmojiEntry {
        alias: "weixiao",
        emoji: "😊",
    },
    EmojiEntry {
        alias: "wenhao",
        emoji: "❓",
    },
    EmojiEntry {
        alias: "woshou",
        emoji: "🤝",
    },
    EmojiEntry {
        alias: "wugui",
        emoji: "🐢",
    },
    EmojiEntry {
        alias: "xia",
        emoji: "🦐",
    },
    EmojiEntry {
        alias: "xiang",
        emoji: "🐘",
    },
    EmojiEntry {
        alias: "xiangjiao",
        emoji: "🍌",
    },
    EmojiEntry {
        alias: "xiangyi",
        emoji: "🤔",
    },
    EmojiEntry {
        alias: "xianhua",
        emoji: "💐",
    },
    EmojiEntry {
        alias: "xiao",
        emoji: "😄",
    },
    EmojiEntry {
        alias: "xiaoyu",
        emoji: "🌧️",
    },
    EmojiEntry {
        alias: "xin",
        emoji: "💕",
    },
    EmojiEntry {
        alias: "xingxing",
        emoji: "⭐",
    },
    EmojiEntry {
        alias: "xinlv",
        emoji: "💓",
    },
    EmojiEntry {
        alias: "xiong",
        emoji: "🐻",
    },
    EmojiEntry {
        alias: "xue",
        emoji: "❄️",
    },
    EmojiEntry {
        alias: "xuegao",
        emoji: "🍦",
    },
    EmojiEntry {
        alias: "xuexiao",
        emoji: "🏫",
    },
    EmojiEntry {
        alias: "yanjing",
        emoji: "👀",
    },
    EmojiEntry {
        alias: "youyong",
        emoji: "🏊",
    },
    EmojiEntry {
        alias: "yu",
        emoji: "🐟",
    },
    EmojiEntry {
        alias: "yueliang",
        emoji: "🌙",
    },
    EmojiEntry {
        alias: "yumaoqiu",
        emoji: "🏸",
    },
    EmojiEntry {
        alias: "zan",
        emoji: "👍",
    },
    EmojiEntry {
        alias: "zhu",
        emoji: "🐷",
    },
    EmojiEntry {
        alias: "zui",
        emoji: "💋",
    },
];

/// 按别名查询 emoji；未命中返回 `None`（大小写敏感，别名均为小写）。
#[must_use]
pub fn emoji_for(alias: &str) -> Option<&'static str> {
    EMOJI_TABLE
        .binary_search_by(|entry| entry.alias.cmp(alias))
        .ok()
        .map(|index| EMOJI_TABLE[index].emoji)
}

#[cfg(test)]
mod tests {
    use super::{emoji_for, EMOJI_TABLE};
    use std::collections::HashSet;

    #[test]
    fn 表有序且无重复别名() {
        let mut seen: HashSet<&str> = HashSet::new();
        for pair in EMOJI_TABLE.windows(2) {
            assert!(
                pair[0].alias < pair[1].alias,
                "表未按 alias 升序: {} vs {}",
                pair[0].alias,
                pair[1].alias
            );
        }
        for entry in EMOJI_TABLE {
            assert!(seen.insert(entry.alias), "重复别名: {}", entry.alias);
            assert!(!entry.alias.is_empty() && !entry.emoji.is_empty());
        }
    }

    #[test]
    fn 表规模在约束内() {
        assert!(EMOJI_TABLE.len() >= 100, "首批应 ≥100 条");
        assert!(EMOJI_TABLE.len() <= 512, "上限 512 条");
    }

    #[test]
    fn 验收别名命中() {
        assert_eq!(emoji_for("xiao"), Some("😄"));
        assert_eq!(emoji_for("ku"), Some("😂"));
        assert_eq!(emoji_for("aixin"), Some("❤️"));
        assert_eq!(emoji_for("shui"), Some("😴"));
        assert_eq!(emoji_for("zan"), Some("👍"));
        assert_eq!(emoji_for("ok"), Some("👌"));
    }

    #[test]
    fn 不命中返回空() {
        assert_eq!(emoji_for("xyzabc"), None);
        assert_eq!(emoji_for(""), None);
        assert_eq!(emoji_for("XIAO"), None);
        assert_eq!(emoji_for("xiao "), None);
    }
}
