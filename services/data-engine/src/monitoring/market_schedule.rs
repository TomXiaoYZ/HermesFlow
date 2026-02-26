use chrono::{DateTime, Datelike, NaiveDate, NaiveTime, Utc, Weekday};
use chrono_tz::Tz;

/// US market holidays (NYSE/NASDAQ observed) for 2025-2027.
/// Updated annually; a hardcoded list avoids external dependencies.
const US_HOLIDAYS: &[(i32, u32, u32)] = &[
    // 2025
    (2025, 1, 1),   // New Year's Day
    (2025, 1, 20),  // MLK Jr. Day
    (2025, 2, 17),  // Presidents' Day
    (2025, 4, 18),  // Good Friday
    (2025, 5, 26),  // Memorial Day
    (2025, 6, 19),  // Juneteenth
    (2025, 7, 4),   // Independence Day
    (2025, 9, 1),   // Labor Day
    (2025, 11, 27), // Thanksgiving Day
    (2025, 12, 25), // Christmas Day
    // 2026
    (2026, 1, 1),   // New Year's Day
    (2026, 1, 19),  // MLK Jr. Day
    (2026, 2, 16),  // Presidents' Day
    (2026, 4, 3),   // Good Friday
    (2026, 5, 25),  // Memorial Day
    (2026, 6, 19),  // Juneteenth
    (2026, 7, 3),   // Independence Day (observed, Jul 4 is Saturday)
    (2026, 9, 7),   // Labor Day
    (2026, 11, 26), // Thanksgiving Day
    (2026, 12, 25), // Christmas Day
    // 2027
    (2027, 1, 1),   // New Year's Day
    (2027, 1, 18),  // MLK Jr. Day
    (2027, 2, 15),  // Presidents' Day
    (2027, 3, 26),  // Good Friday
    (2027, 5, 31),  // Memorial Day
    (2027, 6, 18),  // Juneteenth (observed, Jun 19 is Saturday)
    (2027, 7, 5),   // Independence Day (observed, Jul 4 is Sunday)
    (2027, 9, 6),   // Labor Day
    (2027, 11, 25), // Thanksgiving Day
    (2027, 12, 24), // Christmas Day (observed, Dec 25 is Saturday)
];

/// Returns `true` if the exchange is expected to produce live data at `now`.
///
/// Polymarket is 24/7. Traditional equity markets respect their local
/// trading hours and holidays.
pub fn is_market_open(exchange: &str, now: DateTime<Utc>) -> bool {
    match exchange.to_lowercase().as_str() {
        // Prediction markets: 24/7
        "polymarket" => true,

        // US equities: 09:30-16:00 ET, Mon-Fri, excl. US holidays
        "polygon" | "ibkr" | "alpaca" | "massive" => is_open_us_market(now),

        // Hong Kong: 09:30-16:00 HKT, Mon-Fri
        "futu" => is_open_hk_market(now),

        // China A-shares: 09:30-15:00 CST, Mon-Fri
        "akshare" => is_open_cn_market(now),

        // Unknown: assume open (safe default — won't suppress real alerts)
        _ => true,
    }
}

fn is_open_us_market(now: DateTime<Utc>) -> bool {
    let tz: Tz = chrono_tz::America::New_York;
    let local = now.with_timezone(&tz);

    if matches!(local.weekday(), Weekday::Sat | Weekday::Sun) {
        return false;
    }

    let date = local.date_naive();
    if is_us_holiday(date) {
        return false;
    }

    let time = local.time();
    // 09:30 - 16:00 ET (DST handled automatically by chrono-tz)
    let open = NaiveTime::from_hms_opt(9, 30, 0).unwrap();
    let close = NaiveTime::from_hms_opt(16, 0, 0).unwrap();
    time >= open && time < close
}

fn is_us_holiday(date: NaiveDate) -> bool {
    US_HOLIDAYS.iter().any(|&(y, m, d)| {
        NaiveDate::from_ymd_opt(y, m, d)
            .map(|h| h == date)
            .unwrap_or(false)
    })
}

fn is_open_hk_market(now: DateTime<Utc>) -> bool {
    let tz: Tz = chrono_tz::Asia::Hong_Kong;
    let local = now.with_timezone(&tz);

    if matches!(local.weekday(), Weekday::Sat | Weekday::Sun) {
        return false;
    }

    let time = local.time();
    let open = NaiveTime::from_hms_opt(9, 30, 0).unwrap();
    let close = NaiveTime::from_hms_opt(16, 0, 0).unwrap();
    time >= open && time < close
}

fn is_open_cn_market(now: DateTime<Utc>) -> bool {
    let tz: Tz = chrono_tz::Asia::Shanghai;
    let local = now.with_timezone(&tz);

    if matches!(local.weekday(), Weekday::Sat | Weekday::Sun) {
        return false;
    }

    let time = local.time();
    let open = NaiveTime::from_hms_opt(9, 30, 0).unwrap();
    let close = NaiveTime::from_hms_opt(15, 0, 0).unwrap();
    time >= open && time < close
}

/// P6-3B: Returns `true` if any tracked equity market is open at `now`.
///
/// Used by the monitoring pipeline to auto-suspend flow-based (non-infrastructure)
/// alerts when all markets are closed, preventing false alarms during off-hours.
pub fn is_any_equity_market_open(now: DateTime<Utc>) -> bool {
    // Check all known equity exchanges
    is_open_us_market(now) || is_open_hk_market(now) || is_open_cn_market(now)
}

/// P6-3B: Returns `true` if this is a trading day for the given exchange,
/// regardless of current time. Used to distinguish "market closed for the day"
/// from "before/after hours on a trading day".
pub fn is_trading_day(exchange: &str, now: DateTime<Utc>) -> bool {
    match exchange.to_lowercase().as_str() {
        "polymarket" => true,
        "polygon" | "ibkr" | "alpaca" | "massive" => {
            let tz: Tz = chrono_tz::America::New_York;
            let local = now.with_timezone(&tz);
            !matches!(local.weekday(), Weekday::Sat | Weekday::Sun)
                && !is_us_holiday(local.date_naive())
        }
        "futu" => {
            let tz: Tz = chrono_tz::Asia::Hong_Kong;
            let local = now.with_timezone(&tz);
            !matches!(local.weekday(), Weekday::Sat | Weekday::Sun)
        }
        "akshare" => {
            let tz: Tz = chrono_tz::Asia::Shanghai;
            let local = now.with_timezone(&tz);
            !matches!(local.weekday(), Weekday::Sat | Weekday::Sun)
        }
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn polymarket_always_open() {
        let saturday_midnight = Utc.with_ymd_and_hms(2025, 6, 14, 0, 0, 0).unwrap(); // Saturday
        assert!(is_market_open("polymarket", saturday_midnight));
    }

    #[test]
    fn us_market_open_during_trading_hours() {
        // Wednesday 2025-06-11 at 14:00 UTC = 10:00 ET (EDT, UTC-4)
        let midday_et = Utc.with_ymd_and_hms(2025, 6, 11, 14, 0, 0).unwrap();
        assert!(is_market_open("polygon", midday_et));
        assert!(is_market_open("massive", midday_et));
    }

    #[test]
    fn us_market_closed_at_night() {
        // Wednesday 2025-06-11 at 02:00 UTC = 22:00 ET previous day
        let night_et = Utc.with_ymd_and_hms(2025, 6, 11, 2, 0, 0).unwrap();
        assert!(!is_market_open("polygon", night_et));
    }

    #[test]
    fn us_market_closed_on_weekend() {
        // Saturday 2025-06-14 at 14:00 UTC
        let saturday = Utc.with_ymd_and_hms(2025, 6, 14, 14, 0, 0).unwrap();
        assert!(!is_market_open("polygon", saturday));
    }

    #[test]
    fn us_market_closed_on_holiday() {
        // 2025-07-04 (Independence Day) at 14:00 UTC = 10:00 ET
        let july4 = Utc.with_ymd_and_hms(2025, 7, 4, 14, 0, 0).unwrap();
        assert!(!is_market_open("polygon", july4));
    }

    #[test]
    fn hk_market_open_during_trading_hours() {
        // Wednesday 2025-06-11 at 03:00 UTC = 11:00 HKT
        let midday_hkt = Utc.with_ymd_and_hms(2025, 6, 11, 3, 0, 0).unwrap();
        assert!(is_market_open("futu", midday_hkt));
    }

    #[test]
    fn hk_market_closed_at_night() {
        // Wednesday 2025-06-11 at 14:00 UTC = 22:00 HKT
        let night_hkt = Utc.with_ymd_and_hms(2025, 6, 11, 14, 0, 0).unwrap();
        assert!(!is_market_open("futu", night_hkt));
    }

    #[test]
    fn cn_market_open_during_trading_hours() {
        // Wednesday 2025-06-11 at 03:00 UTC = 11:00 CST
        let midday_cst = Utc.with_ymd_and_hms(2025, 6, 11, 3, 0, 0).unwrap();
        assert!(is_market_open("akshare", midday_cst));
    }

    #[test]
    fn cn_market_closed_after_3pm() {
        // Wednesday 2025-06-11 at 08:00 UTC = 16:00 CST (after 15:00 close)
        let afternoon_cst = Utc.with_ymd_and_hms(2025, 6, 11, 8, 0, 0).unwrap();
        assert!(!is_market_open("akshare", afternoon_cst));
    }

    #[test]
    fn unknown_exchange_defaults_open() {
        let anytime = Utc.with_ymd_and_hms(2025, 6, 14, 0, 0, 0).unwrap();
        assert!(is_market_open("unknown_exchange", anytime));
    }

    #[test]
    fn case_insensitive_exchange_name() {
        let midday_et = Utc.with_ymd_and_hms(2025, 6, 11, 14, 0, 0).unwrap();
        assert!(is_market_open("Polygon", midday_et));
        assert!(is_market_open("POLYMARKET", midday_et));
    }

    // ── P6-3B: Timezone-aware calendar tests ─────────────────────────────

    #[test]
    fn any_equity_market_open_during_us_hours() {
        // Wednesday 2025-06-11 at 14:00 UTC = 10:00 ET (US market open)
        let us_hours = Utc.with_ymd_and_hms(2025, 6, 11, 14, 0, 0).unwrap();
        assert!(is_any_equity_market_open(us_hours));
    }

    #[test]
    fn any_equity_market_open_during_hk_hours() {
        // Wednesday 2025-06-11 at 03:00 UTC = 11:00 HKT (HK open, US closed)
        let hk_hours = Utc.with_ymd_and_hms(2025, 6, 11, 3, 0, 0).unwrap();
        assert!(is_any_equity_market_open(hk_hours));
    }

    #[test]
    fn no_equity_market_open_on_weekend() {
        // Saturday 2025-06-14 at 12:00 UTC — no equity market is open
        let weekend = Utc.with_ymd_and_hms(2025, 6, 14, 12, 0, 0).unwrap();
        assert!(!is_any_equity_market_open(weekend));
    }

    #[test]
    fn is_trading_day_weekday() {
        // Wednesday 2025-06-11 at 02:00 UTC (before US open, but IS a trading day)
        let before_open = Utc.with_ymd_and_hms(2025, 6, 11, 2, 0, 0).unwrap();
        assert!(is_trading_day("polygon", before_open));
    }

    #[test]
    fn is_not_trading_day_weekend() {
        let saturday = Utc.with_ymd_and_hms(2025, 6, 14, 14, 0, 0).unwrap();
        assert!(!is_trading_day("polygon", saturday));
    }

    #[test]
    fn is_not_trading_day_holiday() {
        let july4 = Utc.with_ymd_and_hms(2025, 7, 4, 14, 0, 0).unwrap();
        assert!(!is_trading_day("polygon", july4));
    }
}
