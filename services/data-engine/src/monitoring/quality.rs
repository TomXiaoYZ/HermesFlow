use crate::error::DataEngineError;
use crate::monitoring::metrics::{
    ACTIVE_SYMBOLS_COUNT, DQ_GAP_SYMBOLS, DQ_LOW_LIQ_SYMBOLS, DQ_SPIKE_SYMBOLS, DQ_STALE_SYMBOLS,
};
use chrono::{Duration, Utc};
use sqlx::PgPool;
use tracing::{error, info, warn};

/// Configuration for Data Quality Checks
#[derive(Debug, Clone)]
pub struct DataQualityConfig {
    pub freshness_threshold_sec: i64, // Alert if no snapshot in X sec (Default: 30)
    pub liquidity_min_usd: f64,       // Alert if liquidity < $100k
    pub price_change_threshold_pct: f64, // Alert if price moves > 50% in one tick
}

impl Default for DataQualityConfig {
    fn default() -> Self {
        Self {
            freshness_threshold_sec: 30,
            liquidity_min_usd: 100_000.0,
            price_change_threshold_pct: 0.50,
        }
    }
}

pub struct DataMonitor {
    pool: PgPool,
    config: DataQualityConfig,
}

impl DataMonitor {
    pub fn new(pool: PgPool, config: DataQualityConfig) -> Self {
        Self { pool, config }
    }

    /// Run all checks and return a report (or just log/alert internally)
    pub async fn run_checks(&self) -> Result<(), DataEngineError> {
        info!("Starting 5-Stage Data Quality Validation...");

        // 0. Update Active Count first (Metric)
        self.update_active_metrics().await?;

        // 1. Freshness Check
        self.check_freshness().await?;

        // 2. Gap Detection
        self.check_gaps().await?;

        // 3. Liquidity Guard
        self.check_liquidity().await?;

        // 4. Price Spike Detection
        self.check_price_spikes().await?;

        info!("Data Quality Validation Complete.");
        Ok(())
    }

    async fn update_active_metrics(&self) -> Result<(), DataEngineError> {
        // Just count active tokens
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM active_tokens WHERE is_active = true")
                .fetch_one(&self.pool)
                .await
                .map_err(|e| {
                    DataEngineError::DatabaseError(format!("Failed to count active tokens: {}", e))
                })?;

        ACTIVE_SYMBOLS_COUNT.set(count);
        Ok(())
    }

    async fn check_freshness(&self) -> Result<(), DataEngineError> {
        // Query active symbols that haven't updated in X seconds
        let threshold = Utc::now() - Duration::seconds(self.config.freshness_threshold_sec);

        use sqlx::Row;

        // 1. Check active_tokens (Solana DEX / Birdeye)
        let stale_tokens = sqlx::query(
            r#"
            SELECT a.address as symbol, NULL::timestamptz as timestamp
            FROM active_tokens a
            WHERE a.is_active = true
            AND NOT EXISTS (
                SELECT 1 FROM mkt_equity_snapshots s
                WHERE s.symbol = a.address
                AND s.timestamp >= $1
            )
            LIMIT 100
            "#,
        )
        .bind(threshold)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            DataEngineError::DatabaseError(format!("Freshness check (tokens) failed: {}", e))
        })?;

        // 2. Check mkt_equity_snapshots for Polygon stocks and AkShare A-shares
        // Note: table uses 'exchange' column and 'timestamp' column
        let threshold_minutes = self.config.freshness_threshold_sec / 60;
        let stale_equities = sqlx::query(
            r#"
            SELECT symbol, exchange, MAX(timestamp) as last_ts
            FROM mkt_equity_snapshots
            WHERE timestamp > NOW() - INTERVAL '24 hours'
            GROUP BY symbol, exchange
            HAVING MAX(timestamp) < NOW() - make_interval(mins => $1)
            "#,
        )
        .bind(threshold_minutes as i32)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            DataEngineError::DatabaseError(format!("Freshness check (equities) failed: {}", e))
        })?;

        let total_stale = stale_tokens.len() + stale_equities.len();
        DQ_STALE_SYMBOLS.set(total_stale as i64);

        if !stale_tokens.is_empty() {
            let sample: Vec<String> = stale_tokens
                .iter()
                .take(3)
                .map(|r| r.get::<String, _>("symbol"))
                .collect();
            warn!(
                "Stage 1 FRESHNESS ALERT (tokens): {} symbols are stale (>{}s). Examples: {:?}",
                stale_tokens.len(),
                self.config.freshness_threshold_sec,
                sample
            );
        }

        if !stale_equities.is_empty() {
            let sample: Vec<String> = stale_equities
                .iter()
                .take(3)
                .map(|r| {
                    let symbol: String = r.get("symbol");
                    let exchange: String = r.get("exchange");
                    format!("{}({})", symbol, exchange)
                })
                .collect();
            warn!(
                "Stage 1 FRESHNESS ALERT (equities): {} symbols are stale (>{}m). Examples: {:?}",
                stale_equities.len(),
                threshold_minutes,
                sample
            );
        }

        Ok(())
    }

    async fn check_gaps(&self) -> Result<(), DataEngineError> {
        use sqlx::Row;

        // (resolution, lookback_hours, min_expected_count)
        let timeframes: &[(&str, i64, i64)] = &[
            ("1m", 4, 200),
            ("5m", 8, 80),
            ("15m", 24, 80),
            ("1h", 72, 60),
            ("4h", 168, 36),
            ("1d", 720, 25),
        ];

        let mut total_gap_symbols: i64 = 0;

        for &(resolution, lookback_hours, min_expected) in timeframes {
            let cutoff = Utc::now() - Duration::hours(lookback_hours);

            let rows = sqlx::query(
                r#"
                SELECT symbol, count(*) as count
                FROM mkt_equity_candles
                WHERE resolution = $1 AND time > $2
                GROUP BY symbol
                HAVING count(*) < $3
                "#,
            )
            .bind(resolution)
            .bind(cutoff)
            .bind(min_expected)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| {
                DataEngineError::DatabaseError(format!(
                    "Gap check failed for {}: {}",
                    resolution, e
                ))
            })?;

            total_gap_symbols += rows.len() as i64;

            if !rows.is_empty() {
                let sample: Vec<String> = rows
                    .iter()
                    .take(3)
                    .map(|r| r.get::<String, _>("symbol"))
                    .collect();
                warn!(
                    "Stage 2 GAP ALERT [{}]: {} symbols missing candles in last {}h (expected >= {}). Examples: {:?}",
                    resolution,
                    rows.len(),
                    lookback_hours,
                    min_expected,
                    sample
                );
            }
        }

        DQ_GAP_SYMBOLS.set(total_gap_symbols);

        Ok(())
    }

    async fn check_liquidity(&self) -> Result<(), DataEngineError> {
        // Check if any tracked symbol has dropped below min liquidity
        use sqlx::Row;
        // Ensure accurate type casting in SQL if needed, but here binding f64 should work if column is numeric/float.
        // Postgres numeric <-> Rust BigDecimal usually, but f64 binds to double precision.
        // Cast to float8 to be safe for comparison if column is generic numeric.
        let low_liq_rows = sqlx::query(
            r#"
            SELECT symbol, liquidity_usd
            FROM active_tokens
            WHERE is_active = true AND liquidity_usd < $1
            "#,
        )
        .bind(self.config.liquidity_min_usd)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DataEngineError::DatabaseError(format!("Liquidity check failed: {}", e)))?;

        DQ_LOW_LIQ_SYMBOLS.set(low_liq_rows.len() as i64);

        if !low_liq_rows.is_empty() {
            let sample: Vec<String> = low_liq_rows
                .iter()
                .take(3)
                .map(|r| r.get::<String, _>("symbol"))
                .collect();
            error!("Stage 3 LIQUIDITY GUARD: {} symbols dropped below ${}. Shutting them down recommended. Examples: {:?}",
                 low_liq_rows.len(),
                 self.config.liquidity_min_usd,
                 sample
             );
        }

        Ok(())
    }

    /// Stage 4: Detect price spikes where price changed >threshold% vs previous snapshot within 10 minutes
    async fn check_price_spikes(&self) -> Result<Vec<(String, f64)>, DataEngineError> {
        use sqlx::Row;
        let threshold = self.config.price_change_threshold_pct;
        let ten_minutes_ago = Utc::now() - Duration::minutes(10);

        let spike_rows = sqlx::query(
            r#"
            WITH lagged AS (
                SELECT
                    symbol,
                    price,
                    LAG(price) OVER (PARTITION BY symbol ORDER BY timestamp) AS prev_price,
                    timestamp
                FROM mkt_equity_snapshots
                WHERE timestamp >= $1
            )
            SELECT DISTINCT
                symbol,
                ABS((price - prev_price) / NULLIF(prev_price, 0))::float8 AS pct_change
            FROM lagged
            WHERE prev_price IS NOT NULL
              AND prev_price != 0
              AND ABS((price - prev_price) / prev_price) > $2
            LIMIT 100
            "#,
        )
        .bind(ten_minutes_ago)
        .bind(threshold)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DataEngineError::DatabaseError(format!("Price spike check failed: {}", e)))?;

        DQ_SPIKE_SYMBOLS.set(spike_rows.len() as i64);

        let results: Vec<(String, f64)> = spike_rows
            .iter()
            .map(|r| {
                let symbol: String = r.get("symbol");
                let pct_change: f64 = r.get("pct_change");
                (symbol, pct_change)
            })
            .collect();

        if !results.is_empty() {
            let sample: Vec<_> = results
                .iter()
                .take(3)
                .map(|(s, pct)| format!("{}({:.1}%)", s, pct * 100.0))
                .collect();
            warn!(
                "Stage 4 PRICE SPIKE ALERT: {} symbols moved >{:.0}% in 10min. Examples: {:?}",
                results.len(),
                threshold * 100.0,
                sample
            );
        }

        Ok(results)
    }
}
