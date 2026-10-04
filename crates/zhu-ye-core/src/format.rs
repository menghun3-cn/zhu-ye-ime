//! 数字格式候选（FR-027，场景 7）。
//!
//! 对数字串按确定性规则生成常用格式候选（日期/年月/年份/金额/电话/千分位）；
//! 无法识别时返回空，不劫持数字正常输入。全部规则为纯算法，无外部数据。

/// 数字格式候选：上屏文本与类别说明。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormatCandidate {
    /// 上屏文本（格式化结果）。
    pub text: String,
    /// 类别说明（"日期"/"年月"/"年份"/"金额"/"电话"/"千分位"），供 CLI/调试展示。
    pub label: &'static str,
}

/// 对数字串生成格式候选（确定性顺序）。
///
/// 规则表：
/// - 8 位纯数字且 01≤月≤12、01≤日≤31 → 日期 4 式（`-`、`/`、`年月日`、`.`）
/// - 6 位纯数字且 01≤月≤12 → 年月 3 式（`-`、`/`、`年月`）
/// - 4 位纯数字且 1900≤年≤2099 → 年份（`年`）
/// - `\d+\.\d{1,2}` → 金额：千分位 + 中文读数
/// - 11 位且 `1[3-9]\d{9}` → 电话：空格分段 / 连字符分段
/// - ≥5 位纯数字其它 → 千分位
/// - 其余（<5 位纯数字、非法日期、超长、多小数点等）→ 空
#[must_use]
pub fn format_candidates(input: &str) -> Vec<FormatCandidate> {
    // 输入合法性：只接受 ASCII 数字与至多一个小数点，长度 ≤32。
    if input.is_empty() || input.len() > 32 {
        return Vec::new();
    }
    let mut dot_count = 0usize;
    for ch in input.chars() {
        if ch == '.' {
            dot_count += 1;
            if dot_count > 1 {
                return Vec::new();
            }
        } else if !ch.is_ascii_digit() {
            return Vec::new();
        }
    }

    // 金额：含小数点（整数至少 1 位、小数 1-2 位）。
    if let Some((int_part, frac_part)) = input.split_once('.') {
        if int_part.is_empty() || frac_part.is_empty() || frac_part.len() > 2 {
            return Vec::new();
        }
        let mut out = Vec::with_capacity(2);
        out.push(FormatCandidate {
            text: format!("{}.{}", group_thousands(int_part), frac_part),
            label: "金额",
        });
        // 中文读数：整数部分超大时不读（返回空则跳过该项）。
        if let Some(cn_int) = cn_numeral(int_part) {
            let mut cn = cn_int;
            if !frac_part.chars().all(|c| c == '0') {
                cn.push('点');
                cn.push_str(&split_digits_cn(frac_part));
            }
            out.push(FormatCandidate {
                text: cn,
                label: "金额",
            });
        }
        return out;
    }

    // 纯数字分支（位数决定格式）。
    let len = input.len();
    match len {
        8 => {
            // 日期 yyyymmdd。
            if let Some((y, m, d)) = split_date(input) {
                return vec![
                    FormatCandidate {
                        text: format!("{y}-{m}-{d}"),
                        label: "日期",
                    },
                    FormatCandidate {
                        text: format!("{y}/{m}/{d}"),
                        label: "日期",
                    },
                    FormatCandidate {
                        text: format!(
                            "{y}年{}月{}日",
                            trim_leading_zeros(&m),
                            trim_leading_zeros(&d)
                        ),
                        label: "日期",
                    },
                    FormatCandidate {
                        text: format!("{y}.{m}.{d}"),
                        label: "日期",
                    },
                ];
            }
            vec![FormatCandidate {
                text: group_thousands(input),
                label: "千分位",
            }]
        }
        6 => {
            // 年月 yyyymm。
            let y = &input[0..4];
            let m = &input[4..6];
            if valid_year(y) && is_month(m) {
                return vec![
                    FormatCandidate {
                        text: format!("{y}-{m}"),
                        label: "年月",
                    },
                    FormatCandidate {
                        text: format!("{y}/{m}"),
                        label: "年月",
                    },
                    FormatCandidate {
                        text: format!("{y}年{}月", trim_leading_zeros(m)),
                        label: "年月",
                    },
                ];
            }
            vec![FormatCandidate {
                text: group_thousands(input),
                label: "千分位",
            }]
        }
        4 => {
            if valid_year(input) {
                return vec![FormatCandidate {
                    text: format!("{input} 年"),
                    label: "年份",
                }];
            }
            Vec::new()
        }
        11 => {
            // 手机号。
            if input.starts_with('1') && is_mobile(input) {
                return vec![
                    FormatCandidate {
                        text: format!("{} {} {}", &input[0..3], &input[3..7], &input[7..11]),
                        label: "电话",
                    },
                    FormatCandidate {
                        text: format!("{}-{}-{}", &input[0..3], &input[3..7], &input[7..11]),
                        label: "电话",
                    },
                ];
            }
            vec![FormatCandidate {
                text: group_thousands(input),
                label: "千分位",
            }]
        }
        n if n >= 5 => vec![FormatCandidate {
            text: group_thousands(input),
            label: "千分位",
        }],
        _ => Vec::new(), // <5 位不触发。
    }
}

/// 千分位分组（整数部分）：`1234567` → `1,234,567`。
#[must_use]
fn group_thousands(int_part: &str) -> String {
    let mut out = String::with_capacity(int_part.len() + int_part.len() / 3);
    let bytes = int_part.as_bytes();
    let first = (bytes.len() - 1) % 3 + 1;
    out.push_str(&int_part[..first]);
    for chunk in bytes[first..].chunks(3) {
        out.push(',');
        // chunks 均为 ASCII 数字，安全转 str。
        out.push_str(std::str::from_utf8(chunk).expect("digits are ASCII"));
    }
    out
}

/// 取 8 位串的年/月/日并校验月份与日期范围。
fn split_date(input: &str) -> Option<(String, String, String)> {
    let y = input[0..4].to_owned();
    let m = input[4..6].to_owned();
    let d = input[6..8].to_owned();
    if valid_year(&y) && is_month(&m) && is_day(&d) {
        Some((y, m, d))
    } else {
        None
    }
}

fn valid_year(y: &str) -> bool {
    (1900..=2099).contains(&y.parse::<u32>().unwrap_or(0))
}

fn is_month(m: &str) -> bool {
    let v = m.parse::<u32>().unwrap_or(0);
    (1..=12).contains(&v)
}

fn is_day(d: &str) -> bool {
    let v = d.parse::<u32>().unwrap_or(0);
    (1..=31).contains(&v)
}

/// 去前导零的月/日（`09` → `9`）；全零时保留原串（`00` 不应出现于此路径）。
fn trim_leading_zeros(s: &str) -> &str {
    let trimmed = s.trim_start_matches('0');
    if trimmed.is_empty() {
        s
    } else {
        trimmed
    }
}

fn is_mobile(input: &str) -> bool {
    let third = input.as_bytes()[2];
    (b'3'..=b'9').contains(&third)
}

/// 中文读数（亿/万四位制），支持非负整数与至多 2 位小数。
/// `12345` → `一万二千三百四十五`；`10005` → `一万零五`；`0.5` → `零点五`。
#[must_use]
pub fn cn_numeral(input: &str) -> Option<String> {
    if input.is_empty() || !input.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    if input.len() > 18 {
        return None; // 防止解析溢出（u64 上限 20 位，留余量）。
    }
    let value = input.parse::<u64>().ok()?;
    Some(integer_to_cn(value))
}

fn integer_to_cn(mut v: u64) -> String {
    if v == 0 {
        return "零".to_owned();
    }
    let yi = v / 100_000_000;
    v %= 100_000_000;
    let wan = v / 10_000;
    let ge = v % 10_000;
    let mut out = String::new();
    if yi > 0 {
        out.push_str(&read_section(yi as u32));
        out.push('亿');
    }
    if wan > 0 {
        if yi > 0 && wan < 1000 {
            out.push('零');
        }
        out.push_str(&read_section(wan as u32));
        out.push('万');
    }
    if ge > 0 {
        if (yi > 0 || wan > 0) && ge < 1000 {
            out.push('零');
        }
        out.push_str(&read_section(ge as u32));
    }
    out
}

const SMALL: [&str; 10] = ["零", "一", "二", "三", "四", "五", "六", "七", "八", "九"];
const UNITS: [&str; 4] = ["", "十", "百", "千"];

/// 读取 <10000 的一段数字（不含段名）：`2345` → `二千三百四十五`。
fn read_section(v: u32) -> String {
    debug_assert!(v < 10_000);
    let digits = [v / 1000 % 10, v / 100 % 10, v / 10 % 10, v % 10];
    let mut out = String::new();
    let mut pending_zero = false;
    let mut emitted = false; // 本段已输出任何内容。
    for (index, d) in digits.iter().enumerate() {
        if *d > 0 {
            if pending_zero && emitted {
                out.push('零');
            }
            let unit = UNITS[3 - index];
            // 段首"一十"省略"一"（`15`→十五）；其余情况正常读（`115`→一百一十五）。
            let omit_one = *d == 1 && unit == "十" && !emitted;
            if !omit_one {
                out.push_str(SMALL[*d as usize]);
            }
            out.push_str(unit);
            pending_zero = false;
            emitted = true;
        } else if emitted {
            // 仅当本段已有内容时，零才可能出现在读数中间。
            pending_zero = true;
        }
    }
    out
}

/// 逐位读出小数部分（`05` → `零五`）。
fn split_digits_cn(frac: &str) -> String {
    let mut out = String::with_capacity(frac.len() * 3);
    for ch in frac.chars() {
        out.push_str(SMALL[ch.to_digit(10).unwrap_or(0) as usize]);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{cn_numeral, format_candidates};

    fn texts(input: &str) -> Vec<String> {
        format_candidates(input)
            .into_iter()
            .map(|c| c.text)
            .collect()
    }

    #[test]
    fn 日期八位四种格式() {
        assert_eq!(
            texts("20260930"),
            vec!["2026-09-30", "2026/09/30", "2026年9月30日", "2026.09.30"]
        );
    }

    #[test]
    fn 日期去前导零() {
        assert!(texts("20260105").contains(&"2026年1月5日".to_owned()));
    }

    #[test]
    fn 非法日期回退千分位() {
        assert_eq!(texts("20261332"), vec!["20,261,332"]);
    }

    #[test]
    fn 年月三种格式() {
        assert_eq!(texts("202609"), vec!["2026-09", "2026/09", "2026年9月"]);
    }

    #[test]
    fn 年份() {
        assert_eq!(texts("2026"), vec!["2026 年"]);
        assert!(texts("1300").is_empty());
    }

    #[test]
    fn 金额千分位与中文读数() {
        assert_eq!(texts("12345.6"), vec!["12,345.6", "一万二千三百四十五点六"]);
    }

    #[test]
    fn 电话分段() {
        assert_eq!(texts("13800138000"), vec!["138 0013 8000", "138-0013-8000"]);
        // 非 1[3-9] 开头 11 位 → 千分位。
        assert_eq!(texts("92345678901"), vec!["92,345,678,901"]);
    }

    #[test]
    fn 纯数字千分位() {
        assert_eq!(texts("54321"), vec!["54,321"]);
    }

    #[test]
    fn 不足五位不触发() {
        assert!(texts("12").is_empty());
        assert!(texts("0").is_empty());
        assert!(texts("1234").is_empty());
    }

    #[test]
    fn 非法输入不触发() {
        assert!(texts("").is_empty());
        assert!(texts("12a34").is_empty());
        assert!(texts("1.2.3").is_empty());
        assert!(texts("1.234").is_empty());
        assert!(texts(".").is_empty());
    }

    #[test]
    fn 超长串不崩() {
        let long = "9".repeat(40);
        assert!(texts(&long).is_empty());
        let thirty = "9".repeat(30);
        assert_eq!(texts(&thirty), vec![group(&thirty)]);
    }

    fn group(s: &str) -> String {
        let mut out = String::new();
        let bytes = s.as_bytes();
        let first = (bytes.len() - 1) % 3 + 1;
        out.push_str(&s[..first]);
        for chunk in bytes[first..].chunks(3) {
            out.push(',');
            out.push_str(std::str::from_utf8(chunk).unwrap());
        }
        out
    }

    #[test]
    fn 中文读数基本() {
        assert_eq!(cn_numeral("0"), Some("零".to_owned()));
        assert_eq!(cn_numeral("5"), Some("五".to_owned()));
        assert_eq!(cn_numeral("10"), Some("十".to_owned()));
        assert_eq!(cn_numeral("15"), Some("十五".to_owned()));
        assert_eq!(cn_numeral("20"), Some("二十".to_owned()));
        assert_eq!(cn_numeral("99"), Some("九十九".to_owned()));
        assert_eq!(cn_numeral("100"), Some("一百".to_owned()));
        assert_eq!(cn_numeral("105"), Some("一百零五".to_owned()));
        assert_eq!(cn_numeral("110"), Some("一百一十".to_owned()));
        assert_eq!(cn_numeral("115"), Some("一百一十五".to_owned()));
        assert_eq!(cn_numeral("1000"), Some("一千".to_owned()));
        assert_eq!(cn_numeral("1005"), Some("一千零五".to_owned()));
        assert_eq!(cn_numeral("10000"), Some("一万".to_owned()));
        assert_eq!(cn_numeral("10005"), Some("一万零五".to_owned()));
        assert_eq!(cn_numeral("12345"), Some("一万二千三百四十五".to_owned()));
        assert_eq!(
            cn_numeral("5201314"),
            Some("五百二十万一千三百一十四".to_owned())
        );
        assert_eq!(cn_numeral("100000000"), Some("一亿".to_owned()));
        assert_eq!(
            cn_numeral("123456789"),
            Some("一亿二千三百四十五万六千七百八十九".to_owned())
        );
        assert_eq!(cn_numeral("100000005"), Some("一亿零五".to_owned()));
        assert_eq!(cn_numeral("100010000"), Some("一亿零一万".to_owned()));
    }

    #[test]
    fn 中文读数拒绝非数字() {
        assert_eq!(cn_numeral(""), None);
        assert_eq!(cn_numeral("1.5"), None);
        assert_eq!(cn_numeral("abc"), None);
        assert!(cn_numeral(&"9".repeat(19)).is_none());
    }
}
