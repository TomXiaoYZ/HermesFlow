use crate::error::DataEngineError;
use crate::monitoring::market_schedule;
use crate::monitoring::metrics::{
    ACTIVE_SYMBOLS_COUNT, DQ_CROSS_SOURCE_DIVERGENCE, DQ_GAP_SYMBOLS, DQ_INCIDENTS_TOTAL,
    DQ_LOW_LIQ_SYMBOLS, DQ_SOURCE_SCORE, DQ_SPIKE_SYMBOLS, DQ_STALE_SYMBOLS, DQ_TIMESTAMP_DRIFT,
    DQ_VOLUME_ANOMALY, DQ_WATCHLIST_MISSING, DQ_WATCHLIST_STALE,
};
use chrono::{Duration, Utc};
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Mutex;
use tracing::{debug, error, info, warn};

/// Check frequency tier for the data quality pipeline.
///
/// - `Critical` (every 30s): freshness, active count — detect outages fast.
/// - `Warning` (every 5min): gaps, spikes, cross-source divergence.
/// - `FullAudit` (every 1h): all of the above + liquidity, volume anomaly, timestamp drift.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckTier {
    Critical,
    Warning,
    FullAudit,
}

impl std::fmt::Display for CheckTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CheckTier::Critical => write!(f, "Critical"),
            CheckTier::Warning => write!(f, "Warning"),
            CheckTier::FullAudit => write!(f, "FullAudit"),
        }
    }
}

/// Configuration for Data Quality Checks
#[derive(Debug, Clone)]
pub struct DataQualityConfig {
    pub freshness_threshold_sec: i64,
    pub liquidity_min_usd: f64,
    pub price_change_threshold_pct: f64,
    /// Cross-source price divergence threshold (default: 1%)
    pub cross_source_divergence_pct: f64,
    /// Volume anomaly threshold: alert if current hour volume < this fraction of 7d average (default: 0.1 = 10%)
    pub volume_anomaly_ratio: f64,
    /// Timestamp drift threshold in seconds (default: 30)
    pub timestamp_drift_threshold_sec: i64,
    /// P6-3A: Poisson staleness detection threshold.
    /// Alert only when P(0 ticks in elapsed time) < this value.
    /// Lower values = fewer false positives for low-liquidity symbols.
    pub poisson_staleness_threshold: f64,
    /// P6-3A: EWMA decay factor for tick arrival rate (0.0–1.0).
    /// Higher = more responsive to recent data; lower = smoother.
    pub poisson_ewma_alpha: f64,
}

impl Default for DataQualityConfig {
    fn default() -> Self {
        Self {
            freshness_threshold_sec: 30,
            liquidity_min_usd: 100_000.0,
            price_change_threshold_pct: 0.50,
            cross_source_divergence_pct: 0.01,
            volume_anomaly_ratio: 0.10,
            timestamp_drift_threshold_sec: 30,
            poisson_staleness_threshold: 0.001,
            poisson_ewma_alpha: 0.05,
        }
    }
}

/// P6-3A: Per-symbol EWMA tick arrival rate for Poisson staleness detection.
///
/// Tracks the expected tick arrival rate (λ) as an EWMA of inter-tick intervals.
/// Staleness is flagged only when P(0 ticks in Δt) = e^(-λ·Δt) < threshold,
/// automatically adapting to each symbol's normal tick frequency.
#[derive(Debug, Clone)]
struct TickRateState {
    /// EWMA of tick arrival rate (ticks per second)
    lambda: f64,
    /// Last observed tick timestamp (epoch seconds)
    last_tick_epoch: f64,
}

pub struct DataMonitor {
    pool: PgPool,
    config: DataQualityConfig,
    /// P6-3A: Per-symbol (exchange:symbol) tick arrival rate tracking
    tick_rates: Mutex<HashMap<String, TickRateState>>,
    /// P7-0B: Track whether any market was open on last check.
    /// On closed→open transition, tick_rates EWMA is reset to prevent
    /// stale lambda values from weekend causing Monday false positives.
    was_market_open: Mutex<bool>,
}

impl DataMonitor {
    pub fn new(pool: PgPool, config: DataQualityConfig) -> Self {
        Self {
            pool,
            config,
            tick_rates: Mutex::new(HashMap::new()),
            was_market_open: Mutex::new(false),
        }
    }

    /// Run quality checks for the given tier.
    ///
    /// P6-3B: Flow-based alerts (freshness, gaps, price spikes, cross-source divergence,
    /// volume anomaly) are auto-suppressed when no equity market is currently open.
    /// Infrastructure alerts (active count, watchlist, timestamp drift, source scores)
    /// run 24/7 regardless of market hours.
    pub async fn run_checks(&self, tier: CheckTier) -> Result<(), DataEngineError> {
        let now = Utc::now();
        let any_market_open = market_schedule::is_any_equity_market_open(now);

        // P7-0B: Detect closed→open transition and reset EWMA tick rates
        // to prevent stale lambda values from weekend/holiday causing false positives.
        {
            let mut was_open = self
                .was_market_open
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            if any_market_open && !*was_open {
                let mut tick_rates = self.tick_rates.lock().unwrap_or_else(|e| e.into_inner());
                let count = tick_rates.len();
                tick_rates.clear();
                info!(
                    "Market open transition detected — reset {} tick rate EWMA entries",
                    count
                );
            }
            *was_open = any_market_open;
        }

        if !any_market_open {
            debug!(
                tier = %tier,
                "All equity markets closed — suppressing flow-based alerts"
            );
        }

        info!(tier = %tier, "Starting data quality checks");

        match tier {
            CheckTier::Critical => {
                self.update_active_metrics().await?; // infrastructure: always
                if any_market_open {
                    self.check_freshness().await?; // flow: market hours only
                }
            }
            CheckTier::Warning => {
                if any_market_open {
                    self.check_gaps().await?;
                    self.check_price_spikes().await?;
                    self.check_cross_source_divergence().await?;
                }
                self.check_watchlist_completeness().await?; // infrastructure: always
            }
            CheckTier::FullAudit => {
                self.update_active_metrics().await?; // infrastructure: always
                if any_market_open {
                    self.check_freshness().await?;
                    self.check_gaps().await?;
                    self.check_liquidity().await?;
                    self.check_price_spikes().await?;
                    self.check_cross_source_divergence().await?;
                    self.check_volume_anomaly().await?;
                }
                self.check_watchlist_completeness().await?; // infrastructure: always
                self.check_timestamp_drift().await?; // infrastructure: always
                self.calculate_source_scores().await?; // infrastructure: always
            }
        }

        info!(tier = %tier, "Data quality checks complete");
        Ok(())
    }

    // ── Stage 0: Active Count ───────────────────────────────────────────

    async fn update_active_metrics(&self) -> Result<(), DataEngineError> {
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

    // ── Stage 1: Freshness (P6-3A Poisson model) ────────────────────────

    /// Check data freshness using adaptive Poisson staleness detection.
    ///
    /// P6-3A: Instead of a static "no data for N minutes" threshold, maintains
    /// a per-symbol EWMA tick arrival rate (λ). Staleness is flagged only when
    /// P(0 ticks in elapsed time) = e^(-λ·Δt) < poisson_threshold.
    ///
    /// This auto-adapts to each symbol's tick frequency:
    /// - High-frequency stocks (e.g., AAPL): seconds of silence triggers alert
    /// - Low-liquidity/OTC stocks: hours of silence is normal, auto-suppressed
    async fn check_freshness(&self) -> Result<(), DataEngineError> {
        use sqlx::Row;

        let now = Utc::now();
        let now_epoch = now.timestamp() as f64;
        let alpha = self.config.poisson_ewma_alpha;
        let poisson_threshold = self.config.poisson_staleness_threshold;

        // Fetch latest tick timestamp per symbol/exchange (active in last 24h)
        let rows = sqlx::query(
            r#"
            SELECT symbol, exchange, MAX(time) as last_ts
            FROM mkt_equity_snapshots
            WHERE time > NOW() - INTERVAL '24 hours'
            GROUP BY symbol, exchange
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            DataEngineError::DatabaseError(format!("Freshness check (equities) failed: {}", e))
        })?;

        let mut poisson_stale: Vec<(String, String, f64, f64)> = Vec::new();

        {
            let mut tick_rates = self.tick_rates.lock().unwrap_or_else(|e| e.into_inner());

            for row in &rows {
                let symbol: String = row.get("symbol");
                let exchange: String = row.get("exchange");

                // Skip symbols whose exchange is currently closed
                if !market_schedule::is_market_open(&exchange, now) {
                    continue;
                }

                let last_ts: chrono::DateTime<chrono::Utc> = row.get("last_ts");
                let last_epoch = last_ts.timestamp() as f64;
                let elapsed = (now_epoch - last_epoch).max(0.0);

                let key = format!("{}:{}", exchange, symbol);

                let state = tick_rates.entry(key).or_insert_with(|| TickRateState {
                    // Initialize with a conservative rate estimate:
                    // assume 1 tick per freshness_threshold_sec
                    lambda: 1.0 / self.config.freshness_threshold_sec.max(1) as f64,
                    last_tick_epoch: last_epoch,
                });

                // Update EWMA rate if we got a newer tick
                if last_epoch > state.last_tick_epoch {
                    let interval = last_epoch - state.last_tick_epoch;
                    if interval > 0.0 {
                        let observed_rate = 1.0 / interval;
                        state.lambda = alpha * observed_rate + (1.0 - alpha) * state.lambda;
                    }
                    state.last_tick_epoch = last_epoch;
                }

                // Poisson probability of zero arrivals in elapsed time:
                // P(0) = e^(-λ·Δt)
                let p_zero = (-state.lambda * elapsed).exp();

                if p_zero < poisson_threshold {
                    poisson_stale.push((symbol, exchange, elapsed, p_zero));
                }
            }
        }

        DQ_STALE_SYMBOLS.set(poisson_stale.len() as i64);

        if !poisson_stale.is_empty() {
            let sample: Vec<String> = poisson_stale
                .iter()
                .take(3)
                .map(|(sym, exch, elapsed, p)| {
                    format!("{}({}) stale {:.0}s P(0)={:.2e}", sym, exch, elapsed, p)
                })
                .collect();
            warn!(
                "Stage 1 FRESHNESS ALERT (Poisson): {} symbol-exchange pairs stale (P(0)<{:.0e}). Examples: {:?}",
                poisson_stale.len(),
                poisson_threshold,
                sample
            );
            self.record_incident(
                "freshness",
                "warning",
                None,
                None,
                Some(serde_json::json!({
                    "type": "equity_poisson",
                    "stale_count": poisson_stale.len(),
                    "poisson_threshold": poisson_threshold,
                    "sample": sample,
                })),
            )
            .await;
        }

        Ok(())
    }

    // ── Stage 2: Gap Detection ──────────────────────────────────────────

    async fn check_gaps(&self) -> Result<(), DataEngineError> {
        use sqlx::Row;

        let now = Utc::now();
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
            let cutoff = now - Duration::hours(lookback_hours);

            let rows = sqlx::query(
                r#"
                SELECT symbol, exchange, count(*) as count
                FROM mkt_equity_candles
                WHERE resolution = $1 AND time > $2
                GROUP BY symbol, exchange
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

            // Only count gaps for exchanges whose market is currently open
            let filtered: Vec<_> = rows
                .into_iter()
                .filter(|r| {
                    let exchange: String = r.get("exchange");
                    market_schedule::is_market_open(&exchange, now)
                })
                .collect();

            total_gap_symbols += filtered.len() as i64;

            if !filtered.is_empty() {
                let sample: Vec<String> = filtered
                    .iter()
                    .take(3)
                    .map(|r| {
                        let symbol: String = r.get("symbol");
                        let exchange: String = r.get("exchange");
                        format!("{}({})", symbol, exchange)
                    })
                    .collect();
                warn!(
                    "Stage 2 GAP ALERT [{}]: {} symbol-exchange pairs below expected candle count in last {}h (expected >= {}). Examples: {:?}",
                    resolution,
                    filtered.len(),
                    lookback_hours,
                    min_expected,
                    sample
                );
            }
        }

        DQ_GAP_SYMBOLS.set(total_gap_symbols);

        Ok(())
    }

    // ── Stage 3: Liquidity Guard ────────────────────────────────────────

    async fn check_liquidity(&self) -> Result<(), DataEngineError> {
        use sqlx::Row;
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
            error!(
                "Stage 3 LIQUIDITY GUARD: {} symbols below ${}. Examples: {:?}",
                low_liq_rows.len(),
                self.config.liquidity_min_usd,
                sample
            );
        }

        Ok(())
    }

    // ── Stage 4: Price Spike Detection ──────────────────────────────────

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
                    LAG(price) OVER (PARTITION BY symbol ORDER BY time) AS prev_price,
                    time
                FROM mkt_equity_snapshots
                WHERE time >= $1
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
            self.record_incident(
                "price_spike",
                "warning",
                None,
                None,
                Some(serde_json::json!({
                    "count": results.len(),
                    "threshold_pct": threshold * 100.0,
                    "sample": sample,
                })),
            )
            .await;
        }

        Ok(results)
    }

    // ── Stage 5: Cross-Source Price Divergence ───────────────────────────

    /// Detect symbols with >1% price difference across different exchanges.
    async fn check_cross_source_divergence(&self) -> Result<(), DataEngineError> {
        use sqlx::Row;
        let threshold = self.config.cross_source_divergence_pct;
        let window = Utc::now() - Duration::minutes(5);

        let rows = sqlx::query(
            r#"
            WITH latest_per_source AS (
                SELECT DISTINCT ON (symbol, exchange)
                    symbol, exchange, price::float8 as price
                FROM mkt_equity_snapshots
                WHERE time >= $1
                ORDER BY symbol, exchange, time DESC
            ),
            divergence AS (
                SELECT
                    a.symbol,
                    a.exchange AS exchange_a,
                    b.exchange AS exchange_b,
                    ABS(a.price - b.price) / NULLIF(GREATEST(a.price, b.price), 0) AS pct_diff
                FROM latest_per_source a
                JOIN latest_per_source b
                    ON a.symbol = b.symbol
                    AND a.exchange < b.exchange
            )
            SELECT symbol, exchange_a, exchange_b, pct_diff
            FROM divergence
            WHERE pct_diff > $2
            LIMIT 50
            "#,
        )
        .bind(window)
        .bind(threshold)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            DataEngineError::DatabaseError(format!("Cross-source divergence check failed: {}", e))
        })?;

        DQ_CROSS_SOURCE_DIVERGENCE.set(rows.len() as i64);

        if !rows.is_empty() {
            let sample: Vec<String> = rows
                .iter()
                .take(3)
                .map(|r| {
                    let symbol: String = r.get("symbol");
                    let a: String = r.get("exchange_a");
                    let b: String = r.get("exchange_b");
                    let pct: f64 = r.get("pct_diff");
                    format!("{} ({} vs {} = {:.2}%)", symbol, a, b, pct * 100.0)
                })
                .collect();
            warn!(
                "Stage 5 CROSS-SOURCE DIVERGENCE: {} pairs exceed {:.0}%. Examples: {:?}",
                rows.len(),
                threshold * 100.0,
                sample
            );
            self.record_incident(
                "cross_source_divergence",
                "warning",
                None,
                None,
                Some(serde_json::json!({
                    "count": rows.len(),
                    "threshold_pct": threshold * 100.0,
                    "sample": sample,
                })),
            )
            .await;
        }

        Ok(())
    }

    // ── Stage 6: Volume Anomaly Detection ───────────────────────────────

    /// Detect symbols where recent message volume is abnormally low compared
    /// to the 7-day rolling average. This catches silent data feed failures.
    async fn check_volume_anomaly(&self) -> Result<(), DataEngineError> {
        use sqlx::Row;
        let now = Utc::now();
        let ratio = self.config.volume_anomaly_ratio;

        let rows = sqlx::query(
            r#"
            WITH hourly_counts AS (
                SELECT
                    symbol,
                    exchange,
                    date_trunc('hour', time) AS hour,
                    COUNT(*) AS cnt
                FROM mkt_equity_snapshots
                WHERE time > NOW() - INTERVAL '7 days'
                GROUP BY symbol, exchange, date_trunc('hour', time)
            ),
            stats AS (
                SELECT
                    symbol,
                    exchange,
                    AVG(cnt) AS avg_cnt,
                    (SELECT cnt FROM hourly_counts hc2
                     WHERE hc2.symbol = hourly_counts.symbol
                     AND hc2.exchange = hourly_counts.exchange
                     AND hc2.hour = date_trunc('hour', NOW())
                     LIMIT 1) AS current_cnt
                FROM hourly_counts
                GROUP BY symbol, exchange
                HAVING AVG(cnt) > 10
            )
            SELECT symbol, exchange, avg_cnt, COALESCE(current_cnt, 0) AS current_cnt
            FROM stats
            WHERE COALESCE(current_cnt, 0)::float8 < avg_cnt * $1
            LIMIT 50
            "#,
        )
        .bind(ratio)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            DataEngineError::DatabaseError(format!("Volume anomaly check failed: {}", e))
        })?;

        // Only flag anomalies for symbols whose exchange is currently trading
        let filtered: Vec<_> = rows
            .into_iter()
            .filter(|r| {
                let exchange: String = r.get("exchange");
                market_schedule::is_market_open(&exchange, now)
            })
            .collect();

        DQ_VOLUME_ANOMALY.set(filtered.len() as i64);

        if !filtered.is_empty() {
            let sample: Vec<String> = filtered
                .iter()
                .take(3)
                .map(|r| {
                    let symbol: String = r.get("symbol");
                    let exchange: String = r.get("exchange");
                    let avg: f64 = r.get("avg_cnt");
                    let cur: i64 = r.get("current_cnt");
                    format!("{}({}) (cur={}, avg={:.0})", symbol, exchange, cur, avg)
                })
                .collect();
            warn!(
                "Stage 6 VOLUME ANOMALY: {} symbols below {:.0}% of 7d average. Examples: {:?}",
                filtered.len(),
                ratio * 100.0,
                sample
            );
        }

        Ok(())
    }

    // ── Stage 7: Timestamp Drift Detection ──────────────────────────────

    /// Detect exchanges/symbols where the gap between exchange timestamp
    /// and system received_at exceeds the threshold, indicating clock skew
    /// or stale feeds.
    async fn check_timestamp_drift(&self) -> Result<(), DataEngineError> {
        use sqlx::Row;
        let threshold_sec = self.config.timestamp_drift_threshold_sec;
        let window = Utc::now() - Duration::minutes(10);

        let rows = sqlx::query(
            r#"
            SELECT
                exchange,
                COUNT(DISTINCT symbol) AS drift_symbols,
                PERCENTILE_CONT(0.5) WITHIN GROUP (
                    ORDER BY EXTRACT(EPOCH FROM (NOW() - time))
                )::float8 AS median_drift_sec
            FROM mkt_equity_snapshots
            WHERE time >= $1
            GROUP BY exchange
            HAVING PERCENTILE_CONT(0.5) WITHIN GROUP (
                ORDER BY EXTRACT(EPOCH FROM (NOW() - time))
            ) > $2
            "#,
        )
        .bind(window)
        .bind(threshold_sec as f64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            DataEngineError::DatabaseError(format!("Timestamp drift check failed: {}", e))
        })?;

        let total_drift: i64 = rows.iter().map(|r| r.get::<i64, _>("drift_symbols")).sum();
        DQ_TIMESTAMP_DRIFT.set(total_drift);

        if !rows.is_empty() {
            let sample: Vec<String> = rows
                .iter()
                .take(5)
                .map(|r| {
                    let exchange: String = r.get("exchange");
                    let drift: f64 = r.get("median_drift_sec");
                    let count: i64 = r.get("drift_symbols");
                    format!("{} ({} symbols, median {:.1}s)", exchange, count, drift)
                })
                .collect();
            warn!(
                "Stage 7 TIMESTAMP DRIFT: {} exchanges exceed {}s threshold. {:?}",
                rows.len(),
                threshold_sec,
                sample
            );
        }

        Ok(())
    }

    // ── Stage 8: Watchlist Completeness ─────────────────────────────────

    /// Check that every active symbol in market_watchlist has recent candle data.
    /// Detects two categories:
    /// - "missing": symbol has zero candle data in mkt_equity_candles
    /// - "stale": latest 1d candle is older than 4 calendar days (covers weekends)
    async fn check_watchlist_completeness(&self) -> Result<(), DataEngineError> {
        use sqlx::Row;

        let rows = sqlx::query(
            r#"
            SELECT
                w.symbol,
                w.exchange,
                MAX(c.time) as latest_candle
            FROM market_watchlist w
            LEFT JOIN mkt_equity_candles c
                ON c.symbol = w.symbol
                AND c.exchange = w.exchange
                AND c.resolution = '1d'
            WHERE w.is_active = true
            GROUP BY w.symbol, w.exchange
            HAVING MAX(c.time) IS NULL
                OR MAX(c.time) < NOW() - INTERVAL '4 days'
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            DataEngineError::DatabaseError(format!("Watchlist completeness check failed: {}", e))
        })?;

        let mut missing_count: i64 = 0;
        let mut stale_count: i64 = 0;
        let mut missing_sample: Vec<String> = Vec::new();
        let mut stale_sample: Vec<String> = Vec::new();

        for row in &rows {
            let symbol: String = row.get("symbol");
            let exchange: String = row.get("exchange");
            let latest: Option<chrono::DateTime<chrono::Utc>> =
                row.try_get("latest_candle").ok().flatten();

            if latest.is_none() {
                missing_count += 1;
                if missing_sample.len() < 5 {
                    missing_sample.push(format!("{}({})", symbol, exchange));
                }
            } else {
                stale_count += 1;
                if stale_sample.len() < 5 {
                    stale_sample.push(format!("{}({})", symbol, exchange));
                }
            }
        }

        DQ_WATCHLIST_MISSING.set(missing_count);
        DQ_WATCHLIST_STALE.set(stale_count);

        if missing_count > 0 {
            warn!(
                "Stage 8 WATCHLIST MISSING: {} symbols have NO candle data. Examples: {:?}",
                missing_count, missing_sample
            );
            self.record_incident(
                "watchlist_missing",
                "critical",
                None,
                None,
                Some(serde_json::json!({
                    "missing_count": missing_count,
                    "sample": missing_sample,
                })),
            )
            .await;
        }

        if stale_count > 0 {
            warn!(
                "Stage 8 WATCHLIST STALE: {} symbols have candle data older than 4 days. Examples: {:?}",
                stale_count, stale_sample
            );
            self.record_incident(
                "watchlist_stale",
                "warning",
                None,
                None,
                Some(serde_json::json!({
                    "stale_count": stale_count,
                    "threshold_days": 4,
                    "sample": stale_sample,
                })),
            )
            .await;
        }

        if missing_count == 0 && stale_count == 0 {
            info!("Stage 8 WATCHLIST OK: All watchlist symbols have recent candle data.");
        }

        Ok(())
    }

    // ── Incident Recording ────────────────────────────────────────────────

    /// Record a data quality incident to the `dq_incidents` table and bump the
    /// `DQ_INCIDENTS_TOTAL` counter.  Best-effort: DB errors are logged, not propagated.
    async fn record_incident(
        &self,
        check_type: &str,
        severity: &str,
        symbol: Option<&str>,
        source: Option<&str>,
        details: Option<serde_json::Value>,
    ) {
        if let Err(e) = sqlx::query(
            "INSERT INTO dq_incidents (check_type, severity, symbol, source, details) \
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(check_type)
        .bind(severity)
        .bind(symbol)
        .bind(source)
        .bind(&details)
        .execute(&self.pool)
        .await
        {
            error!("Failed to record DQ incident: {}", e);
        }

        DQ_INCIDENTS_TOTAL
            .with_label_values(&[check_type, severity])
            .inc();
    }

    // ── Per-Source Scoring ───────────────────────────────────────────────

    /// Calculate a 0.0–1.0 quality score per data source based on freshness,
    /// error rate, and completeness. Exported as a Prometheus gauge per source.
    async fn calculate_source_scores(&self) -> Result<(), DataEngineError> {
        use sqlx::Row;

        let rows = sqlx::query(
            r#"
            WITH source_stats AS (
                SELECT
                    exchange,
                    COUNT(*) AS total_snapshots,
                    COUNT(DISTINCT symbol) AS distinct_symbols,
                    MAX(time) AS last_ts,
                    EXTRACT(EPOCH FROM (NOW() - MAX(time)))::float8 AS staleness_sec
                FROM mkt_equity_snapshots
                WHERE time > NOW() - INTERVAL '1 hour'
                GROUP BY exchange
            )
            SELECT
                exchange,
                total_snapshots,
                distinct_symbols,
                staleness_sec
            FROM source_stats
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            DataEngineError::DatabaseError(format!("Source score calculation failed: {}", e))
        })?;

        let mut scores: HashMap<String, f64> = HashMap::new();

        for row in &rows {
            let exchange: String = row.get("exchange");
            let staleness: f64 = row.get("staleness_sec");
            let total: i64 = row.get("total_snapshots");

            // Freshness score: 1.0 if <30s stale, linearly decays to 0 at 300s
            let freshness_score = (1.0 - (staleness / 300.0)).clamp(0.0, 1.0);
            // Volume score: 1.0 if >100 snapshots/hour, 0 if 0
            let volume_score = ((total as f64) / 100.0).clamp(0.0, 1.0);
            // Combined score (weighted average)
            let score = freshness_score * 0.6 + volume_score * 0.4;

            DQ_SOURCE_SCORE.with_label_values(&[&exchange]).set(score);
            scores.insert(exchange, score);
        }

        if !scores.is_empty() {
            info!("Source quality scores: {:?}", scores);
        }

        Ok(())
    }
}
