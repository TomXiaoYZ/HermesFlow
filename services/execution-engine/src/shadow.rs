//! P6-2B: Shadow trading — signal-only observation before paper trading.
//!
//! Subscribes to Redis trade signals and logs them without executing.
//! Strategies must complete 7 **trading days** (excluding weekends and holidays)
//! in shadow mode before promotion to paper trading.
//!
//! **Scoping**: Shadow period is per (exchange, symbol, mode) — NOT per-genome.
//! A single strategy slot (e.g., binance:BTCUSDT:long_only) earns shadow days,
//! and the best genome within that slot can be swapped without resetting the clock.
//! This is correct: the observation period validates the (exchange, symbol, mode)
//! pipeline, not the specific genome weights.

use chrono::{DateTime, Datelike, NaiveDate, Utc, Weekday};
use tokio_postgres::Client;
use tracing::{info, warn};

/// US market holidays for shadow trading-day counting (2025-2028).
const US_HOLIDAYS: &[(i32, u32, u32)] = &[
    (2025, 1, 1),
    (2025, 1, 20),
    (2025, 2, 17),
    (2025, 4, 18),
    (2025, 5, 26),
    (2025, 6, 19),
    (2025, 7, 4),
    (2025, 9, 1),
    (2025, 11, 27),
    (2025, 12, 25),
    (2026, 1, 1),
    (2026, 1, 19),
    (2026, 2, 16),
    (2026, 4, 3),
    (2026, 5, 25),
    (2026, 6, 19),
    (2026, 7, 3),
    (2026, 9, 7),
    (2026, 11, 26),
    (2026, 12, 25),
    (2027, 1, 1),
    (2027, 1, 18),
    (2027, 2, 15),
    (2027, 3, 26),
    (2027, 5, 31),
    (2027, 6, 18),
    (2027, 7, 5),
    (2027, 9, 6),
    (2027, 11, 25),
    (2027, 12, 24),
    // 2028
    (2028, 1, 17),
    (2028, 2, 21),
    (2028, 4, 14),
    (2028, 5, 29),
    (2028, 6, 19),
    (2028, 7, 4),
    (2028, 9, 4),
    (2028, 11, 23),
    (2028, 12, 25),
];

/// Minimum trading days in shadow mode before paper promotion.
pub const MIN_SHADOW_TRADING_DAYS: i32 = 7;

/// P7-3C: Check that US_HOLIDAYS covers the current year.
/// Call at execution-engine startup.
pub fn check_holiday_coverage() {
    let current_year = Utc::now().year();
    let has_current = US_HOLIDAYS.iter().any(|&(y, _, _)| y == current_year);
    if !has_current {
        warn!(
            "Shadow US_HOLIDAYS does not cover year {}. Trading day count may be inaccurate.",
            current_year
        );
    } else {
        info!("Shadow US_HOLIDAYS coverage OK for year {}", current_year);
    }
}

/// Record a shadow signal observation (no execution).
#[allow(clippy::too_many_arguments)]
pub async fn record_shadow_signal(
    db: &Client,
    exchange: &str,
    symbol: &str,
    mode: &str,
    signal_value: f64,
    signal_timestamp: DateTime<Utc>,
    genome: &[i32],
    generation: i32,
    threshold_config: Option<&serde_json::Value>,
) -> anyhow::Result<()> {
    db.execute(
        "INSERT INTO shadow_signals \
         (exchange, symbol, mode, signal_value, signal_timestamp, genome, generation, threshold_config) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        &[
            &exchange,
            &symbol,
            &mode,
            &signal_value,
            &signal_timestamp,
            &genome,
            &generation,
            &threshold_config,
        ],
    )
    .await?;

    Ok(())
}

/// Count completed trading days in shadow mode for a strategy.
///
/// Counts distinct trading days (weekdays, non-holidays) that have
/// at least one shadow signal recorded.
pub async fn count_shadow_trading_days(
    db: &Client,
    exchange: &str,
    symbol: &str,
    mode: &str,
) -> anyhow::Result<i32> {
    let rows = db
        .query(
            "SELECT DISTINCT DATE(signal_timestamp AT TIME ZONE 'America/New_York') as trading_date \
             FROM shadow_signals \
             WHERE exchange = $1 AND symbol = $2 AND mode = $3",
            &[&exchange, &symbol, &mode],
        )
        .await?;

    let mut trading_days = 0i32;
    for row in &rows {
        let date: NaiveDate = row.get("trading_date");
        if is_trading_day(date) {
            trading_days += 1;
        }
    }

    Ok(trading_days)
}

/// Check if a strategy is eligible for paper trading promotion.
///
/// Returns true if the strategy has completed at least MIN_SHADOW_TRADING_DAYS
/// trading days in shadow mode.
pub async fn check_shadow_promotion_eligibility(
    db: &Client,
    exchange: &str,
    symbol: &str,
    mode: &str,
) -> anyhow::Result<bool> {
    let trading_days = count_shadow_trading_days(db, exchange, symbol, mode).await?;

    if trading_days >= MIN_SHADOW_TRADING_DAYS {
        info!(
            "[{}:{}:{}] Shadow promotion eligible: {} trading days (min={})",
            exchange, symbol, mode, trading_days, MIN_SHADOW_TRADING_DAYS
        );
        Ok(true)
    } else {
        warn!(
            "[{}:{}:{}] Shadow promotion blocked: {} trading days (need {})",
            exchange, symbol, mode, trading_days, MIN_SHADOW_TRADING_DAYS
        );
        Ok(false)
    }
}

/// Returns true if the given date is a US equity trading day.
fn is_trading_day(date: NaiveDate) -> bool {
    // Weekends are not trading days
    if matches!(date.weekday(), Weekday::Sat | Weekday::Sun) {
        return false;
    }
    // Holidays are not trading days
    !US_HOLIDAYS.iter().any(|&(y, m, d)| {
        NaiveDate::from_ymd_opt(y, m, d)
            .map(|h| h == date)
            .unwrap_or(false)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trading_day_weekday() {
        // Wednesday June 11, 2025
        let date = NaiveDate::from_ymd_opt(2025, 6, 11).unwrap();
        assert!(is_trading_day(date));
    }

    #[test]
    fn test_not_trading_day_weekend() {
        let saturday = NaiveDate::from_ymd_opt(2025, 6, 14).unwrap();
        assert!(!is_trading_day(saturday));
    }

    #[test]
    fn test_not_trading_day_holiday() {
        // July 4, 2025
        let july4 = NaiveDate::from_ymd_opt(2025, 7, 4).unwrap();
        assert!(!is_trading_day(july4));
    }

    #[test]
    fn test_min_shadow_days() {
        assert_eq!(MIN_SHADOW_TRADING_DAYS, 7);
    }
}
