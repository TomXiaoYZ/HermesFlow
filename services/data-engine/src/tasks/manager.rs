use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{error, info, warn};

use crate::config::AppConfig;
use crate::monitoring::metrics::{TASK_DURATION_SECONDS, TASK_OVERLAP_SKIPPED, TASK_TIMEOUT_TOTAL};
use crate::repository::postgres::PostgresRepositories;

/// Run a named task with overlap prevention and timeout.
///
/// - If `running` is already `true`, the invocation is skipped (logged + metric).
/// - Otherwise, the task runs with `timeout_secs` deadline.
/// - Duration is recorded in `TASK_DURATION_SECONDS`.
async fn guarded_task<F, Fut>(task_name: &str, running: &AtomicBool, timeout_secs: u64, f: F)
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    // Overlap check
    if running
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        warn!(task = task_name, "Skipping overlapping execution");
        TASK_OVERLAP_SKIPPED.with_label_values(&[task_name]).inc();
        return;
    }

    let start = Instant::now();
    let result = tokio::time::timeout(tokio::time::Duration::from_secs(timeout_secs), f()).await;

    let elapsed = start.elapsed().as_secs_f64();
    TASK_DURATION_SECONDS
        .with_label_values(&[task_name])
        .observe(elapsed);

    running.store(false, Ordering::SeqCst);

    if result.is_err() {
        error!(
            task = task_name,
            timeout_secs = timeout_secs,
            elapsed_secs = format!("{:.1}", elapsed),
            "Task timed out"
        );
        TASK_TIMEOUT_TOTAL.with_label_values(&[task_name]).inc();
    }
}

pub struct TaskManager {
    scheduler: Arc<RwLock<JobScheduler>>,
    repos: Arc<PostgresRepositories>,
    config: AppConfig,
}

impl TaskManager {
    pub async fn new(
        config: AppConfig,
        repos: Arc<PostgresRepositories>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let scheduler = JobScheduler::new().await?;
        Ok(Self {
            scheduler: Arc::new(RwLock::new(scheduler)),
            repos,
            config,
        })
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting Task Manager...");
        let scheduler = self.scheduler.read().await;
        scheduler.start().await?;

        // Launch Historical Backfill on Startup (if Birdeye enabled)
        // Check config first
        if let Some(birdeye_config) = &self.config.birdeye {
            if birdeye_config.enabled {
                info!("Triggering Startup Candle Collection Check for Birdeye Symbols...");
                let task = crate::tasks::historical_sync::HistoricalSyncTask::new(
                    birdeye_config.clone(),
                    self.repos.clone(),
                );

                // Spawn task to avoid blocking main thread start
                tokio::spawn(async move {
                    task.run().await;
                });
            }
        }

        Ok(())
    }

    pub async fn register_eod_job(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Registering EOD Data Collection Job...");

        let repos_clone = self.repos.clone();
        let config_clone = self.config.clone();

        // Schedule: Every day at 01:00 UTC
        let job = Job::new_async("0 0 1 * * *", move |_uuid, _l| {
            let repos = repos_clone.clone();
            let config = config_clone.clone();

            Box::pin(async move {
                info!("Executing Scheduled EOD Data Collection...");
                use crate::collectors::MassiveConnector;
                // use crate::models::Candle; // Already available if imported at top or re-import
                // To be safe:
                use crate::models::Candle;
                use crate::repository::MarketDataRepository;
                use chrono::{TimeZone, Utc};
                use rust_decimal::Decimal;
                use serde_json::Value;

                let yesterday = Utc::now().date_naive().pred_opt().unwrap();
                let date_str = yesterday.format("%Y-%m-%d").to_string();

                match repos.market_data.get_watchlist_symbols().await {
                    Ok(symbols) => {
                        info!(
                            "Found {} watchlist symbols for EOD collection",
                            symbols.len()
                        );
                        if let Some(massive_cfg) = config.massive {
                            let connector = MassiveConnector::new(massive_cfg, vec![]);

                            for symbol in symbols {
                                let fetch_result = connector
                                    .fetch_history_candles(&symbol, 1, "day", &date_str, &date_str)
                                    .await;

                                let (status, err_msg) = match &fetch_result {
                                    Ok(candles) => {
                                        let mut insert_ok = true;
                                        for data in candles {
                                            let meta_value =
                                                serde_json::from_str::<Value>(&data.raw_data).ok();
                                            let open = if let Some(json) = &meta_value {
                                                json.get("open")
                                                    .and_then(|v| v.as_f64())
                                                    .map(|f| {
                                                        Decimal::from_f64_retain(f)
                                                            .unwrap_or_default()
                                                    })
                                                    .unwrap_or(data.price)
                                            } else {
                                                data.price
                                            };

                                            let candle = Candle {
                                                exchange: "Polygon".to_string(),
                                                symbol: data.symbol.clone(),
                                                resolution: "1d".to_string(),
                                                open,
                                                high: data.high_24h.unwrap_or(data.price),
                                                low: data.low_24h.unwrap_or(data.price),
                                                close: data.price,
                                                volume: data.quantity,
                                                amount: None,
                                                liquidity: None,
                                                fdv: None,
                                                metadata: meta_value,
                                                time: Utc
                                                    .timestamp_opt(data.timestamp / 1000, 0)
                                                    .unwrap(),
                                            };

                                            if let Err(e) =
                                                repos.market_data.insert_candle(&candle).await
                                            {
                                                error!(
                                                    "EOD: Failed to insert {} for {}: {}",
                                                    symbol, date_str, e
                                                );
                                                insert_ok = false;
                                            }
                                        }
                                        if insert_ok {
                                            ("completed", None)
                                        } else {
                                            ("failed", Some("Insert errors".to_string()))
                                        }
                                    }
                                    Err(e) => {
                                        error!(
                                            "EOD: Failed to fetch {} for {}: {}",
                                            symbol, date_str, e
                                        );
                                        ("failed", Some(e.to_string()))
                                    }
                                };

                                // Update sync status
                                if let Err(e) = sqlx::query(
                                    r#"
                                    INSERT INTO market_sync_status (exchange, symbol, resolution, last_synced_time, last_sync_at, status, error_message)
                                    VALUES ('Polygon', $1, '1d', NOW(), NOW(), $2, $3)
                                    ON CONFLICT (exchange, symbol, resolution) DO UPDATE SET
                                        last_synced_time = NOW(),
                                        last_sync_at = NOW(),
                                        status = $2,
                                        error_message = $3,
                                        retry_count = CASE WHEN $2 = 'failed' THEN market_sync_status.retry_count + 1 ELSE 0 END
                                    "#,
                                )
                                .bind(&symbol)
                                .bind(status)
                                .bind(&err_msg)
                                .execute(&repos.pool)
                                .await
                                {
                                    error!("EOD: Failed to update sync status for {}: {}", symbol, e);
                                }

                                // Rate limit sleep
                                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                            }
                        } else {
                            error!("EOD: Massive/Polygon not configured.");
                        }
                    }
                    Err(e) => error!("Failed to fetch active symbols: {}", e),
                }
                info!("EOD Data Collection Completed.");
            })
        })?;

        let scheduler = self.scheduler.write().await;
        scheduler.add(job).await?;

        // ONE-TIME: Backfill last 30 days of EOD data for all watchlist symbols on startup
        let repos_backfill = self.repos.clone();
        let config_backfill = self.config.clone();
        tokio::spawn(async move {
            use crate::collectors::MassiveConnector;
            use crate::models::Candle;
            use crate::repository::MarketDataRepository;
            use chrono::{TimeZone, Utc};
            use rust_decimal::Decimal;
            use serde_json::Value;

            info!("Running ONE-TIME stock EOD backfill (30 days)...");

            let symbols = match repos_backfill.market_data.get_watchlist_symbols().await {
                Ok(s) => s,
                Err(e) => {
                    error!("Backfill: Failed to fetch watchlist: {}", e);
                    return;
                }
            };

            if symbols.is_empty() {
                info!("Backfill: No watchlist symbols found, skipping.");
                return;
            }

            let massive_cfg = match config_backfill.massive {
                Some(cfg) => cfg,
                None => {
                    error!("Backfill: Massive/Polygon not configured, skipping.");
                    return;
                }
            };

            let connector = MassiveConnector::new(massive_cfg, vec![]);
            let now = Utc::now();
            let mut total_inserted = 0u64;

            for day_offset in 1..=30 {
                let date = (now - chrono::Duration::days(day_offset)).date_naive();
                let date_str = date.format("%Y-%m-%d").to_string();

                for symbol in &symbols {
                    match connector
                        .fetch_history_candles(symbol, 1, "day", &date_str, &date_str)
                        .await
                    {
                        Ok(candles) => {
                            for data in candles {
                                let meta_value = serde_json::from_str::<Value>(&data.raw_data).ok();
                                let open = if let Some(json) = &meta_value {
                                    json.get("open")
                                        .and_then(|v| v.as_f64())
                                        .map(|f| Decimal::from_f64_retain(f).unwrap_or_default())
                                        .unwrap_or(data.price)
                                } else {
                                    data.price
                                };

                                let candle = Candle {
                                    exchange: "Polygon".to_string(),
                                    symbol: data.symbol.clone(),
                                    resolution: "1d".to_string(),
                                    open,
                                    high: data.high_24h.unwrap_or(data.price),
                                    low: data.low_24h.unwrap_or(data.price),
                                    close: data.price,
                                    volume: data.quantity,
                                    amount: None,
                                    liquidity: None,
                                    fdv: None,
                                    metadata: meta_value,
                                    time: Utc.timestamp_opt(data.timestamp / 1000, 0).unwrap(),
                                };

                                if let Err(e) =
                                    repos_backfill.market_data.insert_candle(&candle).await
                                {
                                    error!(
                                        "Backfill: Failed to insert {} for {}: {}",
                                        symbol, date_str, e
                                    );
                                } else {
                                    total_inserted += 1;
                                }
                            }
                        }
                        Err(e) => {
                            warn!(
                                "Backfill: Failed to fetch {} for {}: {}",
                                symbol, date_str, e
                            );
                        }
                    }
                    // Rate limit
                    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                }
            }

            info!(
                "Stock EOD backfill completed. Inserted {} candles for {} symbols.",
                total_inserted,
                symbols.len()
            );
        });

        Ok(())
    }

    pub async fn trigger_backfill(
        &self,
        symbol: String,
        from: String,
        to: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!(
            "Triggering manual backfill for {} from {} to {:?}",
            symbol, from, to
        );

        let config = self.config.clone();
        let repos_clone_for_task = self.repos.clone();

        // Spawn async task to avoid blocking API
        tokio::spawn(async move {
            info!("Starting background backfill for {}", symbol);

            use crate::collectors::{BirdeyeConnector, MassiveConnector};
            use crate::models::Candle;
            use crate::repository::MarketDataRepository; // Import Trait
            use chrono::{TimeZone, Utc};
            use rust_decimal::Decimal;
            use serde_json::Value;

            // Simple heuristic: If symbol length > 10 and no dot, likely a contract address (e.g. Solana)
            let is_contract = symbol.len() > 10 && !symbol.contains('.');

            if is_contract {
                if let Some(birdeye_cfg) = config.birdeye {
                    let connector = BirdeyeConnector::new(birdeye_cfg);
                    info!("Backfill (Birdeye) task running for {}...", symbol);
                    let to_ts = Utc::now().timestamp();
                    let from_ts = Utc::now()
                        .checked_sub_signed(chrono::Duration::days(30))
                        .unwrap()
                        .timestamp(); // Default 30 days? API needs explicit.

                    match connector
                        .fetch_history_candles(&symbol, "1h", from_ts, to_ts)
                        .await
                    {
                        Ok(candles) => {
                            let total = candles.len();
                            let mut success_count = 0;
                            for candle in candles {
                                // BirdeyeConnector returns Candle with None liquidity/fdv usually,
                                // unless we enhanced it.
                                if let Err(e) = repos_clone_for_task
                                    .market_data
                                    .insert_candle(&candle)
                                    .await
                                {
                                    error!("Failed to insert Birdeye candle for {}: {}", symbol, e);
                                } else {
                                    success_count += 1;
                                }
                            }
                            info!(
                                "Birdeye Backfill complete for {}. Inserted {}/{}",
                                symbol, success_count, total
                            );
                        }
                        Err(e) => error!("Failed to fetch Birdeye history for {}: {}", symbol, e),
                    }
                } else {
                    error!("Birdeye not configured for contract address {}", symbol);
                }
            } else if let Some(massive_cfg) = config.massive {
                let connector = MassiveConnector::new(massive_cfg, vec![]);
                info!("Backfill (Massive) task running for {}...", symbol);

                // Default to 5 years if 'to' is not specified
                let to_date = to.unwrap_or_else(|| Utc::now().format("%Y-%m-%d").to_string());

                // Fetch Day Aggregates (1d)
                match connector
                    .fetch_history_candles(&symbol, 1, "day", &from, &to_date)
                    .await
                {
                    Ok(candles) => {
                        let total_candles = candles.len();
                        info!("Fetched {} candles for {}", total_candles, symbol);
                        let mut success_count = 0;
                        for data in candles {
                            // Extract OHLC from raw_data or fields
                            let meta_value = serde_json::from_str::<Value>(&data.raw_data).ok();

                            let open = if let Some(json) = &meta_value {
                                json.get("open")
                                    .and_then(|v| v.as_f64())
                                    .map(|f| Decimal::from_f64_retain(f).unwrap_or_default())
                                    .unwrap_or(data.price)
                            } else {
                                data.price // Fallback
                            };

                            let high = data.high_24h.unwrap_or(data.price);
                            let low = data.low_24h.unwrap_or(data.price);

                            let candle = Candle {
                                exchange: "Polygon".to_string(),
                                symbol: data.symbol.clone(),
                                resolution: "1d".to_string(),
                                open,
                                high,
                                low,
                                close: data.price,
                                volume: data.quantity,
                                amount: None,
                                liquidity: None,
                                fdv: None,
                                metadata: meta_value,
                                time: Utc.timestamp_opt(data.timestamp / 1000, 0).unwrap(),
                            };

                            if let Err(e) = repos_clone_for_task
                                .market_data
                                .insert_candle(&candle)
                                .await
                            {
                                error!("Failed to insert candle for {}: {}", symbol, e);
                            } else {
                                success_count += 1;
                            }
                        }
                        info!(
                            "Backfill complete for {}. Inserted {}/{}",
                            symbol, success_count, total_candles
                        );
                    }
                    Err(e) => {
                        error!("Failed to fetch history for {}: {}", symbol, e);
                    }
                }
            } else {
                error!("Massive/Polygon is not configured!");
            }
        });

        Ok(())
    }

    pub async fn trigger_discovery(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("🔍 Manually Triggering Token Discovery...");
        let repos = self.repos.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            if let Some(birdeye_cfg) = config.birdeye {
                use crate::tasks::token_discovery::TokenDiscoveryTask;

                let task = TokenDiscoveryTask::new(
                    birdeye_cfg.clone(),
                    repos.token.clone(),
                    repos.market_data.clone(),
                    500_000.0,     // min_liquidity_usd: $500k
                    10_000_000.0,  // min_fdv: $10M
                    f64::INFINITY, // max_fdv: unlimited
                );

                task.run().await;
            } else {
                info!("Birdeye not configured, skipping token discovery");
            }
        });
        Ok(())
    }

    pub async fn register_token_discovery_job(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Registering Token Discovery Job (Every 15 min)...");
        let repos_clone = self.repos.clone();
        let config_clone = self.config.clone();

        // Every 1 hour: 0 0 * * * *
        let job = Job::new_async("0 0 * * * *", move |_uuid, _l| {
            let repos = repos_clone.clone();
            let config = config_clone.clone();
            Box::pin(async move {
                info!("🔍 Executing Scheduled Token Discovery...");
                if let Some(birdeye_cfg) = config.birdeye {
                    use crate::tasks::token_discovery::TokenDiscoveryTask;
                    let task = TokenDiscoveryTask::new(
                        birdeye_cfg.clone(),
                        repos.token.clone(),
                        repos.market_data.clone(),
                        500_000.0,
                        10_000_000.0,
                        f64::INFINITY,
                    );
                    task.run().await;
                }
            })
        })?;

        let scheduler = self.scheduler.write().await;
        scheduler.add(job).await?;
        Ok(())
    }

    pub async fn trigger_aggregation(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Manually Triggering Candle Aggregation...");
        let pool = self.repos.pool.clone();

        tokio::spawn(async move {
            let mut aggregator = crate::tasks::candle_aggregation::CandleAggregator::new(pool);
            // Short resolutions: no exchange filter (small lookback, fast)
            if let Err(e) = aggregator.aggregate_candles(20, "1m", 1, None).await {
                error!("Agg 1m failed: {}", e);
            }
            if let Err(e) = aggregator.aggregate_candles(10, "5m", 5, None).await {
                error!("Agg 5m failed: {}", e);
            }
            if let Err(e) = aggregator.aggregate_candles(30, "15m", 15, None).await {
                error!("Agg 15m failed: {}", e);
            }
            // Large resolutions: split by exchange to reduce scanned rows
            let exchanges = ["Polygon", "Jupiter", "Birdeye", "Binance", "OKX", "Bybit"];
            for (lookback, res, bucket) in [
                (120, "1h", 60),
                (300, "4h", 240),
                (1500, "1d", 1440),
                (11520, "1w", 10080),
            ] {
                for exchange in &exchanges {
                    if let Err(e) = aggregator
                        .aggregate_candles(lookback, res, bucket, Some(exchange))
                        .await
                    {
                        error!("Agg {} ({}) failed: {}", res, exchange, e);
                    }
                }
            }

            info!("Manual Aggregation Completed.");
        });
        Ok(())
    }

    pub async fn register_data_quality_job(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Registering tiered Data Quality Jobs (30s / 5min / 1h)...");

        use crate::monitoring::quality::CheckTier;
        use crate::tasks::data_quality::DataQualityTask;

        // Overlap guards — shared across invocations via Arc
        let critical_running = Arc::new(AtomicBool::new(false));
        let warning_running = Arc::new(AtomicBool::new(false));
        let audit_running = Arc::new(AtomicBool::new(false));

        // ── Critical tier: every 30 seconds (25s timeout) ──────────────
        let repos_critical = self.repos.clone();
        let guard_critical = critical_running.clone();
        let job_critical = Job::new_async("*/30 * * * * *", move |_uuid, _l| {
            let repos = repos_critical.clone();
            let running = guard_critical.clone();
            Box::pin(async move {
                guarded_task("dq_critical", &running, 25, || async {
                    let task = DataQualityTask::new(repos, CheckTier::Critical);
                    task.run().await;
                })
                .await;
            })
        })?;

        // ── Warning tier: every 5 minutes (4min timeout) ───────────────
        let repos_warning = self.repos.clone();
        let guard_warning = warning_running.clone();
        let job_warning = Job::new_async("0 */5 * * * *", move |_uuid, _l| {
            let repos = repos_warning.clone();
            let running = guard_warning.clone();
            Box::pin(async move {
                guarded_task("dq_warning", &running, 240, || async {
                    let task = DataQualityTask::new(repos, CheckTier::Warning);
                    task.run().await;
                })
                .await;
            })
        })?;

        // ── Full audit tier: every hour (50min timeout) ────────────────
        let repos_audit = self.repos.clone();
        let guard_audit = audit_running.clone();
        let job_audit = Job::new_async("0 0 * * * *", move |_uuid, _l| {
            let repos = repos_audit.clone();
            let running = guard_audit.clone();
            Box::pin(async move {
                guarded_task("dq_full_audit", &running, 3000, || async {
                    let task = DataQualityTask::new(repos, CheckTier::FullAudit);
                    task.run().await;
                })
                .await;
            })
        })?;

        let scheduler = self.scheduler.write().await;
        scheduler.add(job_critical).await?;
        scheduler.add(job_warning).await?;
        scheduler.add(job_audit).await?;

        // Run initial critical check on startup for visibility
        let repos_startup = self.repos.clone();
        tokio::spawn(async move {
            info!("Running initial Critical data quality check...");
            let task = DataQualityTask::new(repos_startup, CheckTier::Critical);
            task.run().await;
        });

        Ok(())
    }

    pub async fn register_candle_aggregation_job(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Registering Candle Aggregation Jobs (fast=1min, slow=15min)...");

        let pool = self.repos.pool.clone();
        let fast_running = Arc::new(AtomicBool::new(false));
        let slow_running = Arc::new(AtomicBool::new(false));

        // ONE-TIME: Run historical backfill on startup (past 48 hours)
        info!("Running ONE-TIME historical candle backfill (48 hours)...");
        let pool_startup = pool.clone();
        tokio::spawn(async move {
            let mut aggregator =
                crate::tasks::candle_aggregation::CandleAggregator::new(pool_startup);
            if let Err(e) = aggregator.aggregate_candles(48 * 60, "15m", 15, None).await {
                error!("Historical backfill failed: {}", e);
            } else {
                info!("Historical candle backfill completed successfully");
            }
        });

        // FAST JOB: Every 1 minute — short resolutions (1m, 5m, 15m, 1h)
        let pool_fast = pool.clone();
        let guard_fast = fast_running.clone();
        let fast_job = Job::new_async("0 * * * * *", move |_uuid, _l| {
            let pool_clone = pool_fast.clone();
            let running = guard_fast.clone();
            Box::pin(async move {
                guarded_task("candle_agg_fast", &running, 50, || async {
                    let mut aggregator =
                        crate::tasks::candle_aggregation::CandleAggregator::new(pool_clone);

                    // Short resolutions: no exchange filter (small lookback, fast)
                    if let Err(e) = aggregator.aggregate_candles(20, "1m", 1, None).await {
                        error!("Agg 1m failed: {}", e);
                    }
                    if let Err(e) = aggregator.aggregate_candles(10, "5m", 5, None).await {
                        error!("Agg 5m failed: {}", e);
                    }
                    if let Err(e) = aggregator.aggregate_candles(30, "15m", 15, None).await {
                        error!("Agg 15m failed: {}", e);
                    }
                    // 1h: split by exchange but still fast (120min lookback)
                    let exchanges = ["Polygon", "Jupiter", "Birdeye", "Binance", "OKX", "Bybit"];
                    for exchange in &exchanges {
                        if let Err(e) = aggregator
                            .aggregate_candles(120, "1h", 60, Some(exchange))
                            .await
                        {
                            error!("Agg 1h ({}) failed: {}", exchange, e);
                        }
                    }
                })
                .await;
            })
        })?;

        // SLOW JOB: Every 15 minutes — large resolutions (4h, 1d, 1w)
        let pool_slow = pool.clone();
        let guard_slow = slow_running.clone();
        let slow_job = Job::new_async("0 */15 * * * *", move |_uuid, _l| {
            let pool_clone = pool_slow.clone();
            let running = guard_slow.clone();
            Box::pin(async move {
                guarded_task("candle_agg_slow", &running, 600, || async {
                    info!("Running slow candle aggregation (4h, 1d, 1w)...");
                    let mut aggregator =
                        crate::tasks::candle_aggregation::CandleAggregator::new(pool_clone);

                    let exchanges = ["Polygon", "Jupiter", "Birdeye", "Binance", "OKX", "Bybit"];
                    for (lookback, res, bucket) in
                        [(300, "4h", 240), (1500, "1d", 1440), (11520, "1w", 10080)]
                    {
                        for exchange in &exchanges {
                            if let Err(e) = aggregator
                                .aggregate_candles(lookback, res, bucket, Some(exchange))
                                .await
                            {
                                error!("Agg {} ({}) failed: {}", res, exchange, e);
                            }
                        }
                    }

                    info!("Slow candle aggregation complete.");
                })
                .await;
            })
        })?;

        let scheduler = self.scheduler.write().await;
        scheduler.add(fast_job).await?;
        scheduler.add(slow_job).await?;

        Ok(())
    }

    pub async fn register_polymarket_job(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.config.polymarket.is_none() {
            info!("Polymarket not configured, skipping scheduled jobs");
            return Ok(());
        }

        info!("Registering Polymarket Discovery Job (Every hour)...");
        let repos_clone = self.repos.clone();
        let config_clone = self.config.clone();

        // Full discovery every hour: 0 0 * * * *
        let job = Job::new_async("0 0 * * * *", move |_uuid, _l| {
            let repos = repos_clone.clone();
            let config = config_clone.clone();
            Box::pin(async move {
                info!("Executing Scheduled Polymarket Discovery...");
                if let Some(pm_config) = config.polymarket {
                    let collector = crate::collectors::PolymarketCollector::new(
                        pm_config,
                        repos.prediction.clone(),
                    );
                    match collector.discover_markets().await {
                        Ok(count) => info!("Polymarket discovery: upserted {} markets", count),
                        Err(e) => error!("Polymarket discovery failed: {}", e),
                    }
                }
            })
        })?;

        let scheduler = self.scheduler.write().await;
        scheduler.add(job).await?;

        Ok(())
    }

    pub async fn register_polygon_sync_job(&self) -> Result<(), Box<dyn std::error::Error>> {
        let massive_config = match &self.config.massive {
            Some(c) if c.enabled => c.clone(),
            _ => {
                info!("Massive/Polygon not configured/enabled, skipping Polygon sync job");
                return Ok(());
            }
        };

        info!("Registering Polygon OHLCV Sync Job (Every 30 minutes)...");

        let repos = self.repos.clone();

        // Cron: every 30 min at :10 and :40 (offset from Birdeye at :05 and :35)
        let job = Job::new_async("0 10,40 * * * *", move |_uuid, _l| {
            let config = massive_config.clone();
            let repos = repos.clone();
            Box::pin(async move {
                info!("Running periodic Polygon OHLCV sync...");
                let task = crate::tasks::polygon_sync::PolygonSyncTask::new(config, repos);
                task.run().await;
            })
        })?;

        let scheduler = self.scheduler.write().await;
        scheduler.add(job).await?;

        // Run initial sync on startup
        if let Some(massive_cfg) = &self.config.massive {
            if massive_cfg.enabled {
                let repos_startup = self.repos.clone();
                let cfg_startup = massive_cfg.clone();
                tokio::spawn(async move {
                    info!("Triggering startup Polygon OHLCV sync...");
                    let task = crate::tasks::polygon_sync::PolygonSyncTask::new(
                        cfg_startup,
                        repos_startup,
                    );
                    task.run().await;
                });
            }
        }

        Ok(())
    }

    pub async fn register_birdeye_sync_job(&self) -> Result<(), Box<dyn std::error::Error>> {
        let birdeye_config = match &self.config.birdeye {
            Some(c) if c.enabled => c.clone(),
            _ => {
                info!("Birdeye not configured/enabled, skipping periodic sync job");
                return Ok(());
            }
        };

        info!("Registering Birdeye OHLCV Sync Job (Every 30 minutes)...");

        let repos = self.repos.clone();

        // Cron: every 30 minutes (at :05 and :35 to offset from other jobs)
        let job = Job::new_async("0 5,35 * * * *", move |_uuid, _l| {
            let config = birdeye_config.clone();
            let repos = repos.clone();
            Box::pin(async move {
                info!("Running periodic Birdeye OHLCV sync...");
                let task = crate::tasks::historical_sync::HistoricalSyncTask::new(config, repos);
                task.run().await;
            })
        })?;

        let scheduler = self.scheduler.write().await;
        scheduler.add(job).await?;

        Ok(())
    }
}
