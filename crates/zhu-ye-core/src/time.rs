//! 无依赖的小型时间工具（T-088）。
//!
//! `today_compact` 供用户词表导出默认文件名（`user_words_YYYYMMDD.json`）使用；
//! 用系统内 Unix 秒换算，不引入 chrono 依赖。

use crate::user_store::unix_now;

/// 公历日历 → 自 1970-01-01 的天数（Howard Hinnant 的 `days_from_civil`）。
///
/// 与 [`civil_from_days`] 是互逆的一对，当前只在测试中作往返校验。
#[cfg(test)]
fn days_from_civil(year: i64, month: u32, day: u32) -> i64 {
    let y = if month <= 2 { year - 1 } else { year };
    let era = y.div_euclid(400);
    let yoe = y.rem_euclid(400);
    let doy = (153 * (month as i64 + if month > 2 { -3 } else { 9 }) + 2) / 5 + day as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// 自 1970-01-01 的天数 → 公历 `(年, 月, 日)`（Hinnant 的 `civil_from_days` 逆算法）。
#[must_use]
pub fn civil_from_days(days_since_epoch: i64) -> (i64, u32, u32) {
    let z = days_since_epoch + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    (if month <= 2 { y + 1 } else { y }, month as u32, day as u32)
}

/// 今天的 `YYYYMMDD` 紧凑串（用于导出默认文件名）。
#[must_use]
pub fn today_compact() -> String {
    let (year, month, day) = civil_from_days(i64::try_from(unix_now() / 86_400).unwrap_or(0));
    format!("{year:04}{month:02}{day:02}")
}

/// Unix 秒 → `YYYY-MM-DD`（用于关于页展示检查时间等；小时精度对展示足够）。
#[must_use]
pub fn format_date(seconds: u64) -> String {
    let (year, month, day) = civil_from_days(i64::try_from(seconds / 86_400).unwrap_or(0));
    format!("{year:04}-{month:02}-{day:02}")
}

#[cfg(test)]
mod tests {
    use super::{civil_from_days, days_from_civil};

    #[test]
    fn 纪元零点与公历互转() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(days_from_civil(1970, 1, 1), 0);
    }

    #[test]
    fn 基准日二零二六年十月四日() {
        // 2026-10-04 是文档示例导出的日期基准。
        assert_eq!(civil_from_days(20_730), (2026, 10, 4));
        assert_eq!(days_from_civil(2026, 10, 4), 20_730);
    }

    #[test]
    fn 闰年二月二十九日() {
        // 2024-01-01 = 19723 天。
        assert_eq!(civil_from_days(19_723 + 31 + 28), (2024, 2, 29));
        assert_eq!(days_from_civil(2024, 2, 29), 19_723 + 31 + 28);
    }

    #[test]
    fn 年月日格式化补零() {
        assert_eq!(today_compact_ref(), format!("{:04}", today_compact_ref()));
        assert_eq!(super::today_compact().len(), 8, "YYYYMMDD 恒为 8 位");
        let (y, m, d) =
            civil_from_days(i64::try_from(crate::user_store::unix_now() / 86_400).unwrap_or(0));
        assert_eq!(super::today_compact(), format!("{y:04}{m:02}{d:02}"));
    }

    #[test]
    fn 日期展示格式年月日按连字符连接() {
        assert_eq!(super::format_date(0), "1970-01-01");
        // 2026-10-04 = 第 20730 天。
        assert_eq!(super::format_date(20_730 * 86_400), "2026-10-04");
        assert_eq!(
            super::format_date(20_730 * 86_400 + 36_000),
            "2026-10-04",
            "不足一天仍同日"
        );
        assert_eq!(super::format_date((20_730 + 1) * 86_400), "2026-10-05");
    }

    fn today_compact_ref() -> String {
        let (y, m, d) =
            civil_from_days(i64::try_from(crate::user_store::unix_now() / 86_400).unwrap_or(0));
        format!("{y:04}{m:02}{d:02}")
    }
}
