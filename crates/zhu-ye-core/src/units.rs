//! v 模式单位换算全量表（FR-028 扩展，场景7；T-104，2026-10-05 用户点名清单）。
//!
//! 原 v 模式只有符号类型码（`v1`-`v9`/`vx`/`vh`，见 [`symbols`]），需求 13.3 曾把
//! "单位换算全量表"列为非目标；本表按点名清单补齐：`v` + 单位键（拼音/英文缩写，
//! 2-4 位小写字母）给出该单位与同族常用单位的等值换算串候选，一页 ≤9 条。
//!
//! 数据口径（确定性静态表，可审查、可测试，非外部数据源）：
//! - 国际单位制与常用导出单位按精确十进制倍数（米/升/克/秒/字节…）；
//! - 市制按通认折算（1 米 = 3 市尺、1 市斤 = 0.5 千克、1 市里 = 0.5 千米）；
//! - 英制与温度等按常用近似值（小数 4-7 位）固化，雷同主流输入法展示口径；
//! - 温度给出常见定点换算行（0 摄氏度 = 32 华氏度 = 273.15 开尔文）与温差行，
//!   均为常识常量不做运行时计算。
//!
//! 键路约定：`v` 后输入单位键的**前缀字母**时留在 v 模式继续收码；完整键命中出
//! 换算候选；字母串不再构成任何键的前缀才回退拼音（`vi` 等原有回退语义保持）。
//! 类型码 `1-9`/`x`/`h` 仍归符号组优先，单位键不以 `x`/`h` 开头避免与符号码冲突。

/// 单位换算表项：`key` 为 v 模式单位键（2-4 位小写字母），`candidates` 为等值
/// 换算串（均以该单位为中心，格式 `1 单位 = 等值`，确定性排序由表序给出）。
pub struct UnitEntry {
    pub key: &'static str,
    /// 单位中文名（用于诊断与文档，不上屏）。
    pub name: &'static str,
    pub candidates: &'static [&'static str],
}

/// v 模式单位换算全量表（T-104）。
pub const UNIT_CONVERSIONS: &[UnitEntry] = &[
    // ---- 长度 ----
    UnitEntry {
        key: "mi",
        name: "米",
        candidates: &[
            "1 米 = 10 分米",
            "1 米 = 100 厘米",
            "1 米 = 1000 毫米",
            "1 米 = 0.001 千米",
            "1 米 = 3 市尺",
            "1 米 = 39.3701 英寸",
            "1 米 = 3.2808 英尺",
            "1 米 = 0.0006214 英里",
        ],
    },
    UnitEntry {
        key: "chi",
        name: "市尺",
        candidates: &[
            "1 市尺 = 0.3333 米",
            "1 市尺 = 33.3333 厘米",
            "1 市尺 = 10 市寸",
            "1 市尺 = 1.0936 英尺",
            "1 市尺 = 13.1234 英寸",
        ],
    },
    UnitEntry {
        key: "li",
        name: "市里",
        candidates: &[
            "1 市里 = 0.5 千米",
            "1 市里 = 500 米",
            "1 市里 = 1500 市尺",
            "1 市里 = 0.3107 英里",
        ],
    },
    // ---- 面积 ----
    UnitEntry {
        key: "mu",
        name: "亩",
        candidates: &[
            "1 亩 = 666.6667 平方米",
            "1 亩 = 0.01 公顷",
            "1 亩 = 0.001 平方千米",
            "1 亩 = 0.1647 英亩",
        ],
    },
    UnitEntry {
        key: "gongqing",
        name: "公顷",
        candidates: &[
            "1 公顷 = 15 亩",
            "1 公顷 = 10000 平方米",
            "1 公顷 = 0.01 平方千米",
            "1 公顷 = 2.4711 英亩",
        ],
    },
    // ---- 体积 ----
    UnitEntry {
        key: "sheng",
        name: "升",
        candidates: &[
            "1 升 = 1000 毫升",
            "1 升 = 0.001 立方米",
            "1 升 = 1 立方分米",
            "1 升 = 0.2642 美制加仑",
            "1 升 = 1.0567 美制夸脱",
        ],
    },
    UnitEntry {
        key: "fang",
        name: "立方米",
        candidates: &[
            "1 立方米 = 1000 升",
            "1 立方米 = 1000 立方分米",
            "1 立方米 = 1000000 立方厘米",
            "1 立方米 = 35.3147 立方英尺",
            "1 立方米 = 264.1721 美制加仑",
        ],
    },
    // ---- 质量 ----
    UnitEntry {
        key: "kg",
        name: "千克",
        candidates: &[
            "1 千克 = 1000 克",
            "1 千克 = 2 市斤",
            "1 千克 = 0.001 吨",
            "1 千克 = 2.2046 磅",
            "1 千克 = 35.2740 盎司",
        ],
    },
    UnitEntry {
        key: "jin",
        name: "市斤",
        candidates: &[
            "1 市斤 = 500 克",
            "1 市斤 = 0.5 千克",
            "1 市斤 = 10 市两",
            "1 市斤 = 1.1023 磅",
            "1 市斤 = 17.6370 盎司",
        ],
    },
    UnitEntry {
        key: "liang",
        name: "市两",
        candidates: &[
            "1 市两 = 50 克",
            "1 市两 = 0.05 千克",
            "1 市两 = 0.1 市斤",
            "1 市两 = 1.7637 盎司",
        ],
    },
    UnitEntry {
        key: "dun",
        name: "吨",
        candidates: &[
            "1 吨 = 1000 千克",
            "1 吨 = 2000 市斤",
            "1 吨 = 2204.6226 磅",
            "1 吨 = 1.1023 美吨",
        ],
    },
    // ---- 时间 ----
    UnitEntry {
        key: "miao",
        name: "秒",
        candidates: &[
            "1 秒 = 1000 毫秒",
            "1 秒 = 0.0166667 分钟",
            "1 秒 = 0.0002778 小时",
        ],
    },
    UnitEntry {
        key: "shi",
        name: "小时",
        candidates: &[
            "1 小时 = 60 分钟",
            "1 小时 = 3600 秒",
            "1 小时 = 0.0416667 天",
            "1 小时 = 0.0059524 周",
        ],
    },
    UnitEntry {
        key: "tian",
        name: "天",
        candidates: &[
            "1 天 = 24 小时",
            "1 天 = 1440 分钟",
            "1 天 = 86400 秒",
            "1 天 = 0.142857 周",
            "1 天 = 0.032877 月",
            "1 天 = 0.0027397 年",
        ],
    },
    // ---- 温度 ----
    UnitEntry {
        key: "she",
        name: "摄氏度",
        candidates: &[
            "0 摄氏度 = 32 华氏度 = 273.15 开尔文",
            "1 摄氏度 = 33.8 华氏度 = 274.15 开尔文",
            "温差 1 摄氏度 = 温差 1.8 华氏度 = 温差 1 开尔文",
        ],
    },
    UnitEntry {
        key: "far",
        name: "华氏度",
        candidates: &[
            "32 华氏度 = 0 摄氏度 = 273.15 开尔文",
            "温差 1 华氏度 = 温差 0.5556 摄氏度 = 温差 0.5556 开尔文",
        ],
    },
    // ---- 速度 ----
    UnitEntry {
        key: "kmh",
        name: "千米每小时",
        candidates: &[
            "1 千米/小时 = 0.2778 米/秒",
            "1 千米/小时 = 0.6214 英里/小时",
            "1 千米/小时 = 0.53996 节",
        ],
    },
    // ---- 数据量 ----
    UnitEntry {
        key: "kb",
        name: "千字节",
        candidates: &[
            "1 KB = 1024 字节",
            "1 KB = 8192 比特",
            "1 KB = 0.0009766 MB",
            "1 KB = 0.0000009537 GB",
        ],
    },
    UnitEntry {
        key: "mb",
        name: "兆字节",
        candidates: &[
            "1 MB = 1024 KB",
            "1 MB = 1048576 字节",
            "1 MB = 0.0009766 GB",
            "1 MB = 0.0000009537 TB",
        ],
    },
    UnitEntry {
        key: "gb",
        name: "吉字节",
        candidates: &[
            "1 GB = 1024 MB",
            "1 GB = 1048576 KB",
            "1 GB = 1073741824 字节",
            "1 GB = 0.0009766 TB",
        ],
    },
    // ---- 角度 ----
    UnitEntry {
        key: "du",
        name: "度",
        candidates: &[
            "1 度 = 0.0174533 弧度",
            "1 度 = 60 角分",
            "1 度 = 3600 角秒",
        ],
    },
    UnitEntry {
        key: "rad",
        name: "弧度",
        candidates: &["1 弧度 = 57.2958 度", "1 弧度 = 3437.75 角分"],
    },
];

/// 谓词：`key` 是否为某个单位键的前缀（含完整键本身）。
///
/// 供 v 模式判定「继续收字母」：完整键或前缀返回 `true`，否则 `false`。
/// 空串返回 `false`（纯 `v` 等待类型码，不出任何候选）。
#[must_use]
pub fn unit_key_prefix(key: &str) -> bool {
    if key.is_empty() || !key.is_ascii() || key.chars().any(|c| !c.is_ascii_lowercase()) {
        return false;
    }
    UNIT_CONVERSIONS
        .iter()
        .any(|entry| entry.key.starts_with(key))
}

/// 完整单位键的换算候选；未命中（含前缀未完成）返回 `None`。
#[must_use]
pub fn unit_candidates(key: &str) -> Option<&'static [&'static str]> {
    UNIT_CONVERSIONS
        .iter()
        .find(|entry| entry.key == key)
        .map(|entry| entry.candidates)
}

#[cfg(test)]
mod tests {
    use super::{unit_candidates, unit_key_prefix, UNIT_CONVERSIONS};

    #[test]
    fn 米键换算候选() {
        let candidates = unit_candidates("mi").expect("mi 应命中米");
        assert_eq!(candidates[0], "1 米 = 10 分米");
        assert_eq!(candidates[3], "1 米 = 0.001 千米");
        assert!(candidates.len() <= 9, "每键候选不超过一页");
    }

    #[test]
    fn 市斤与数据量键() {
        let jin = unit_candidates("jin").expect("jin 应命中市斤");
        assert!(jin.contains(&"1 市斤 = 500 克"));
        let gb = unit_candidates("gb").expect("gb 应命中吉字节");
        assert!(gb.contains(&"1 GB = 1024 MB"));
    }

    #[test]
    fn 未命中键返回空() {
        assert_eq!(unit_candidates("vi"), None);
        assert_eq!(unit_candidates("mi_"), None);
        assert_eq!(unit_candidates(""), None);
    }

    #[test]
    fn 前缀与完整键判定() {
        assert!(unit_key_prefix("m"), "m 是 mi/miao/mu/mb 的前缀");
        assert!(unit_key_prefix("mi"));
        assert!(unit_key_prefix("sheng"));
        assert!(!unit_key_prefix("x"), "x 是符号类型码，不是单位键前缀");
        assert!(!unit_key_prefix("h"), "h 是符号类型码，不是单位键前缀");
        assert!(!unit_key_prefix(""), "空串不算前缀（v 等待类型码）");
        assert!(!unit_key_prefix("1"), "数字不进单位键");
        assert!(!unit_key_prefix("Mi"), "大写不进单位键");
        assert!(!unit_key_prefix("zh"), "zh 不是任何单位键前缀");
    }

    #[test]
    fn 全表键与候选格式合规() {
        let mut seen = std::collections::HashSet::new();
        for entry in UNIT_CONVERSIONS {
            assert!(seen.insert(entry.key), "键重复: {}", entry.key);
            let key = entry.key;
            assert!(key.len() >= 2 && key.len() <= 8, "键长度 2-8: {key}");
            assert!(
                key.chars().all(|c| c.is_ascii_lowercase()),
                "键须为小写字母: {key}"
            );
            assert!(
                !key.starts_with('x') && !key.starts_with('h'),
                "键不得以符号类型码 x/h 开头: {key}"
            );
            let candidates = entry.candidates;
            assert!(!candidates.is_empty(), "键 {key} 候选为空");
            assert!(candidates.len() <= 9, "键 {key} 候选超过一页");
            assert!(
                !entry.name.is_empty() && entry.name.chars().any(char::is_alphabetic),
                "键 {key} 缺中文名"
            );
        }
    }
}
