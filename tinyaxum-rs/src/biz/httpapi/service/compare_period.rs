use chrono::{Months, NaiveDate};

/// 严格按 `YYYY-MM-DD` 解析日期字符串;失败返回 None,不向调用方暴露具体错误。
pub fn parse_date_param(s: &str) -> Option<NaiveDate> {
    let trimmed = s.trim();
    if trimmed.len() != 10 {
        return None;
    }
    NaiveDate::parse_from_str(trimmed, "%Y-%m-%d").ok()
}

/// 同比:start 与 end 各减 12 个月。
/// 任一减法溢出返回 None(实际只可能在公元 1 年附近触发,业务不会到)。
pub fn yoy_range(
    start: NaiveDate,
    end: NaiveDate,
) -> Option<(NaiveDate, NaiveDate)> {
    let m = Months::new(12);
    Some((start.checked_sub_months(m)?, end.checked_sub_months(m)?))
}

/// 环比:start 与 end 各减 1 个月,月末自动 clamp 到目标月最后一天。
pub fn mom_range(
    start: NaiveDate,
    end: NaiveDate,
) -> Option<(NaiveDate, NaiveDate)> {
    let m = Months::new(1);
    Some((start.checked_sub_months(m)?, end.checked_sub_months(m)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    #[test]
    fn parse_strict_yyyy_mm_dd_ok() {
        assert_eq!(parse_date_param("2026-05-13"), Some(d("2026-05-13")));
        assert_eq!(parse_date_param(" 2026-05-13 "), Some(d("2026-05-13")));
    }

    #[test]
    fn parse_rejects_non_iso_or_partial() {
        assert_eq!(parse_date_param("2026/05/13"), None);
        assert_eq!(parse_date_param("2026-5-13"), None);
        assert_eq!(parse_date_param(""), None);
        assert_eq!(parse_date_param("not-a-date"), None);
        assert_eq!(parse_date_param("2026-05-13T00:00:00"), None);
    }

    #[test]
    fn yoy_normal_year_shift() {
        let (s, e) = yoy_range(d("2026-03-15"), d("2026-03-31")).unwrap();
        assert_eq!(s, d("2025-03-15"));
        assert_eq!(e, d("2025-03-31"));
    }

    #[test]
    fn yoy_leap_day_clamps_to_feb_28() {
        let (s, e) = yoy_range(d("2024-02-29"), d("2024-02-29")).unwrap();
        assert_eq!(s, d("2023-02-28"));
        assert_eq!(e, d("2023-02-28"));
    }

    #[test]
    fn yoy_feb_28_into_non_leap_stays_feb_28() {
        let (s, e) = yoy_range(d("2027-02-28"), d("2027-02-28")).unwrap();
        assert_eq!(s, d("2026-02-28"));
        assert_eq!(e, d("2026-02-28"));
    }

    #[test]
    fn mom_month_end_clamps() {
        let (s, e) = mom_range(d("2026-03-15"), d("2026-03-31")).unwrap();
        assert_eq!(s, d("2026-02-15"));
        assert_eq!(e, d("2026-02-28"));
    }

    #[test]
    fn mom_crosses_year_boundary() {
        let (s, e) = mom_range(d("2026-01-15"), d("2026-01-31")).unwrap();
        assert_eq!(s, d("2025-12-15"));
        assert_eq!(e, d("2025-12-31"));
    }

    #[test]
    fn mom_range_spans_year_input_still_shifts_each() {
        // 输入跨年: 2025-12-15 → 2026-01-15;环比应整体 -1 月
        let (s, e) = mom_range(d("2025-12-15"), d("2026-01-15")).unwrap();
        assert_eq!(s, d("2025-11-15"));
        assert_eq!(e, d("2025-12-15"));
    }
}
