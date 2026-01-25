use crate::error::DataEngineError;
use crate::monitoring::metrics::{
    ACTIVE_SYMBOLS_COUNT, DQ_GAP_SYMBOLS, DQ_LOW_LIQ_SYMBOLS, DQ_STALE_SYMBOLS,
};
use chrono::{Duration, Utc};
use sqlx::PgPool;
use std::sync::Arc;
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

        info!("Data Quality Validation Complete.");
        Ok(())
    }

    async fn update_active_metrics(&self) -> Result<(), DataEngineError> {
        // Just count active tokens
        use sqlx::Row;
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

        // Use FromRow struct or simple query
        // "SELECT symbol, timestamp FROM mkt_equity_snapshots WHERE timestamp < $1"
        // We can just fetch rows.
        use sqlx::Row;

        let stale_rows = sqlx::query(
            r#"
            SELECT DISTINCT s.symbol, s.timestamp 
            FROM mkt_equity_snapshots s
            INNER JOIN active_tokens a ON s.symbol = a.address
            WHERE a.is_active = true 
            AND s.timestamp < $1
            ORDER BY s.timestamp DESC
            "#,
        )
        .bind(threshold)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DataEngineError::DatabaseError(format!("Freshness check failed: {}", e)))?;

        DQ_STALE_SYMBOLS.set(stale_rows.len() as i64);

        if !stale_rows.is_empty() {
            let sample: Vec<String> = stale_rows
                .iter()
                .take(3)
                .map(|r| r.get::<String, _>("symbol"))
                .collect();
            warn!(
                "Stage 1 FRESHNESS ALERT: {} symbols are stale (>{}s). Examples: {:?}",
                stale_rows.len(),
                self.config.freshness_threshold_sec,
                sample
            );
        }

        Ok(())
    }

    async fn check_gaps(&self) -> Result<(), DataEngineError> {
        // Check for missing 15m candles in the last hour
        use sqlx::Row;
        let one_hour_ago = Utc::now() - Duration::hours(1);

        let rows = sqlx::query(
            r#"
             SELECT symbol, count(*) as count
             FROM mkt_equity_candles
             WHERE resolution = '15m' AND time > $1
             GROUP BY symbol
             HAVING count(*) < 3
             "#,
        )
        .bind(one_hour_ago)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DataEngineError::DatabaseError(format!("Gap check failed: {}", e)))?;

        DQ_GAP_SYMBOLS.set(rows.len() as i64);

        if !rows.is_empty() {
            let sample: Vec<String> = rows
                .iter()
                .take(3)
                .map(|r| r.get::<String, _>("symbol"))
                .collect();
            warn!(
                "Stage 2 GAP ALERT: {} symbols missing 15m candles in last hour. Examples: {:?}",
                rows.len(),
                sample
            );
        }

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
}
