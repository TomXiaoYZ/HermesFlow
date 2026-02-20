use redis::AsyncCommands;
use serde::Deserialize;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::collections::HashMap;
use std::env;
use std::time::Duration;
use tracing::{error, info, warn};

mod api;
mod backtest;
mod genetic;

use backtest::StrategyMode;
use backtest::{is_sentinel, sentinel_label, Backtester, WalkForwardConfig};
use backtest_engine::config::FactorConfig;
use genetic::{AlpsGA, PromotionStats};

/// Rolling-window tracker for ALPS layer promotion rates.
/// Used to detect convergence slowdown as a P2 trigger condition:
/// if Layer 0→1 promotion rate drops significantly vs baseline,
/// the search space is too large for random exploration alone.
struct PromotionRateTracker {
    /// Circular buffer of per-generation promotion stats.
    history: Vec<PromotionStats>,
    /// Rolling window size (number of generations to average over).
    window: usize,
    /// Write index in the circular buffer.
    cursor: usize,
    /// Total samples recorded (may exceed window for wrap-around).
    count: usize,
}

impl PromotionRateTracker {
    fn new(window: usize) -> Self {
        Self {
            history: Vec::with_capacity(window),
            window,
            cursor: 0,
            count: 0,
        }
    }

    fn push(&mut self, stats: PromotionStats) {
        if self.history.len() < self.window {
            self.history.push(stats);
        } else {
            self.history[self.cursor] = stats;
        }
        self.cursor = (self.cursor + 1) % self.window;
        self.count += 1;
    }

    /// Rolling average promotion rate for a specific layer boundary.
    /// Returns None if no data or no genomes aged out at this boundary.
    fn avg_rate(&self, boundary: usize) -> Option<f64> {
        let n = self.history.len();
        if n == 0 {
            return None;
        }
        let mut total_promoted = 0usize;
        let mut total_candidates = 0usize;
        for s in &self.history {
            total_promoted += s.promoted[boundary];
            total_candidates += s.promoted[boundary] + s.discarded[boundary];
        }
        if total_candidates == 0 {
            None
        } else {
            Some(total_promoted as f64 / total_candidates as f64)
        }
    }

    /// Rolling average promotion rates for all 4 boundaries [0→1, 1→2, 2→3, 3→4].
    fn avg_rates(&self) -> [Option<f64>; 4] {
        [
            self.avg_rate(0),
            self.avg_rate(1),
            self.avg_rate(2),
            self.avg_rate(3),
        ]
    }

    /// Total promotions across all boundaries in the rolling window.
    fn total_promoted_in_window(&self) -> usize {
        self.history.iter().map(|s| s.total_promoted()).sum()
    }

    /// Total discards across all boundaries in the rolling window.
    fn total_discarded_in_window(&self) -> usize {
        self.history
            .iter()
            .map(|s| s.discarded.iter().sum::<usize>())
            .sum()
    }
}

/// Optional walk-forward configuration, loaded from generator.yaml.
#[derive(Debug, Deserialize, Clone)]
pub struct WalkForwardYamlConfig {
    pub initial_train: Option<usize>,
    pub target_test_window: Option<usize>,
    pub min_test_window: Option<usize>,
    pub target_steps: Option<usize>,
}

/// Per-exchange evolution config, loaded from config/generator.yaml.
#[derive(Debug, Deserialize, Clone)]
pub struct ExchangeConfig {
    pub exchange: String,
    pub resolution: String,
    pub lookback_days: i64,
    pub factor_config: String,
    pub walk_forward: Option<WalkForwardYamlConfig>,
}

#[derive(Debug, Deserialize)]
struct GeneratorConfig {
    exchanges: Vec<ExchangeConfig>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    info!("Starting Strategy Generator (Multi-Exchange Evolutionary Optimizer)...");

    // Health check — single endpoint for the whole process
    tokio::spawn(common::health::start_health_server(
        "strategy-generator",
        8084,
    ));

    // Infrastructure
    let db_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:hermesflow@localhost:5432/hermesflow".to_string());
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&db_url)
        .await?;

    common::heartbeat::spawn_heartbeat("strategy-generator", &redis_url);

    // Load generator config
    let config_path =
        env::var("GENERATOR_CONFIG").unwrap_or_else(|_| "config/generator.yaml".to_string());
    let exchange_configs = load_exchange_configs(&config_path);
    info!(
        "Loaded {} exchange configs: {:?}",
        exchange_configs.len(),
        exchange_configs
            .iter()
            .map(|c| &c.exchange)
            .collect::<Vec<_>>()
    );

    // Build per-exchange factor configs for the API
    let mut api_exchanges: HashMap<String, api::ExchangeApiConfig> = HashMap::new();
    for ec in &exchange_configs {
        let factor_config = load_factor_config(&ec.factor_config);
        api_exchanges.insert(
            ec.exchange.to_lowercase(),
            api::ExchangeApiConfig {
                factor_config,
                exchange: ec.exchange.clone(),
                resolution: ec.resolution.clone(),
            },
        );
    }

    // Single API server serving all exchanges
    let api_port: u16 = env::var("GENERATOR_API_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8082);
    let pool_api = pool.clone();
    tokio::spawn(async move {
        api::start_api_server(pool_api, api_exchanges, api_port).await;
    });

    // Spawn two evolution tasks per (exchange, symbol) pair: long_only + long_short
    let mut handles = Vec::new();
    for ec in exchange_configs {
        let pool_sym = pool.clone();
        let symbols = load_symbols(&pool_sym, &ec.exchange).await;
        let modes = StrategyMode::all();
        info!(
            "[{}] Spawning {} per-symbol evolution tasks (x{} modes = {} total)",
            ec.exchange,
            symbols.len(),
            modes.len(),
            symbols.len() * modes.len()
        );
        for symbol in symbols {
            for &mode in modes {
                let pool = pool.clone();
                let redis_url = redis_url.clone();
                let config = ec.clone();
                let sym = symbol.clone();
                let ex_name = ec.exchange.clone();
                let handle = tokio::spawn(async move {
                    if let Err(e) =
                        run_symbol_evolution(pool, &redis_url, config, sym.clone(), mode).await
                    {
                        error!(
                            "[{}:{}:{}] Evolution loop failed: {}",
                            ex_name, sym, mode, e
                        );
                    }
                });
                handles.push(handle);
            }
        }
    }

    // Wait for all evolution tasks (they run forever unless errored)
    for h in handles {
        let _ = h.await;
    }

    Ok(())
}

fn load_exchange_configs(path: &str) -> Vec<ExchangeConfig> {
    match std::fs::read_to_string(path) {
        Ok(content) => match serde_yaml::from_str::<GeneratorConfig>(&content) {
            Ok(cfg) => cfg.exchanges,
            Err(e) => {
                warn!("Failed to parse {}: {}. Falling back to default.", path, e);
                default_exchange_configs()
            }
        },
        Err(e) => {
            warn!(
                "Failed to read {}: {}. Falling back to env vars / defaults.",
                path, e
            );
            // Backward-compat: single exchange from env vars
            let exchange = env::var("GENERATOR_EXCHANGE").unwrap_or_else(|_| "Birdeye".to_string());
            let resolution = env::var("GENERATOR_RESOLUTION").unwrap_or_else(|_| "15m".to_string());
            let lookback_days: i64 = env::var("GENERATOR_LOOKBACK_DAYS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(if exchange == "Polygon" { 365 } else { 7 });
            let factor_config =
                env::var("FACTOR_CONFIG").unwrap_or_else(|_| "config/factors.yaml".to_string());
            vec![ExchangeConfig {
                exchange,
                resolution,
                lookback_days,
                factor_config,
                walk_forward: None,
            }]
        }
    }
}

fn default_exchange_configs() -> Vec<ExchangeConfig> {
    vec![
        ExchangeConfig {
            exchange: "Birdeye".to_string(),
            resolution: "15m".to_string(),
            lookback_days: 7,
            factor_config: "config/factors.yaml".to_string(),
            walk_forward: None,
        },
        ExchangeConfig {
            exchange: "Polygon".to_string(),
            resolution: "1d".to_string(),
            lookback_days: 365,
            factor_config: "config/factors-stock.yaml".to_string(),
            walk_forward: None,
        },
    ]
}

fn load_factor_config(path: &str) -> FactorConfig {
    match FactorConfig::from_file(path) {
        Ok(cfg) => {
            info!("Loaded {} factors from {}", cfg.active_factors.len(), path);
            cfg
        }
        Err(e) => {
            warn!("Failed to load {}: {}. Using 6-factor default.", path, e);
            FactorConfig {
                active_factors: vec![
                    backtest_engine::config::FactorDefinition {
                        id: 0,
                        name: "return".to_string(),
                        description: "Return".to_string(),
                        normalization: backtest_engine::config::NormalizationType::Robust,
                    },
                    backtest_engine::config::FactorDefinition {
                        id: 1,
                        name: "liquidity_health".to_string(),
                        description: "Liquidity".to_string(),
                        normalization: backtest_engine::config::NormalizationType::None,
                    },
                    backtest_engine::config::FactorDefinition {
                        id: 2,
                        name: "buy_sell_pressure".to_string(),
                        description: "Pressure".to_string(),
                        normalization: backtest_engine::config::NormalizationType::None,
                    },
                    backtest_engine::config::FactorDefinition {
                        id: 3,
                        name: "fomo_acceleration".to_string(),
                        description: "FOMO".to_string(),
                        normalization: backtest_engine::config::NormalizationType::Robust,
                    },
                    backtest_engine::config::FactorDefinition {
                        id: 4,
                        name: "pump_deviation".to_string(),
                        description: "Deviation".to_string(),
                        normalization: backtest_engine::config::NormalizationType::Robust,
                    },
                    backtest_engine::config::FactorDefinition {
                        id: 5,
                        name: "log_volume".to_string(),
                        description: "LogVol".to_string(),
                        normalization: backtest_engine::config::NormalizationType::Robust,
                    },
                ],
            }
        }
    }
}

/// Load symbols for an exchange from DB with fallback defaults.
async fn load_symbols(pool: &PgPool, exchange: &str) -> Vec<String> {
    use sqlx::Row;
    let mut symbols: Vec<String> = if exchange == "Polygon" {
        sqlx::query(
            "SELECT symbol FROM market_watchlist WHERE exchange = 'Polygon' AND is_active = true",
        )
        .fetch_all(pool)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|r| r.get("symbol"))
        .collect()
    } else {
        sqlx::query("SELECT address FROM active_tokens WHERE is_active = true")
            .fetch_all(pool)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|r| r.get("address"))
            .collect()
    };

    if symbols.is_empty() {
        if exchange == "Polygon" {
            warn!("[{}] No active stocks in DB. Using defaults.", exchange);
            symbols = vec!["AAPL", "NVDA", "MSFT", "GOOGL"]
                .into_iter()
                .map(String::from)
                .collect();
        } else {
            warn!("[{}] No active tokens in DB. Using defaults.", exchange);
            symbols = vec![
                "So11111111111111111111111111111111111111112",
                "JUPyiwrYJFskUPiHa7hkeR8VUtkPHCLkdP9KcJQUE85",
                "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263",
                "EKpQGSJmxSWojVRHgWN2EWH18dPBfJCs8J6QW2K2pump",
            ]
            .into_iter()
            .map(String::from)
            .collect();
        }
    } else {
        info!("[{}] Loaded {} symbols from DB", exchange, symbols.len());
    }
    symbols
}

/// Run the evolution loop for a single (exchange, symbol, mode) triple.
async fn run_symbol_evolution(
    pool: PgPool,
    redis_url: &str,
    config: ExchangeConfig,
    symbol: String,
    mode: StrategyMode,
) -> anyhow::Result<()> {
    let exchange = &config.exchange;
    let resolution = &config.resolution;
    let exchange_lower = exchange.to_lowercase();
    let mode_str = mode.as_str();

    info!(
        "[{}:{}:{}] Starting per-symbol evolution: resolution={}, lookback={}d",
        exchange, symbol, mode_str, resolution, config.lookback_days
    );

    let factor_config = load_factor_config(&config.factor_config);
    let feat_offset = factor_config.feat_offset();

    let client = redis::Client::open(redis_url.to_string())?;
    let mut redis_conn = client.get_async_connection().await?;

    let redis_key_status = format!("strategy:{}:{}:{}:status", exchange_lower, symbol, mode_str);
    let redis_channel = format!(
        "strategy_updates:{}:{}:{}",
        exchange_lower, symbol, mode_str
    );

    let mut backtester = Backtester::new(
        pool.clone(),
        factor_config,
        exchange.clone(),
        resolution.clone(),
    );
    let mut ga = AlpsGA::new(feat_offset);

    // Build walk-forward config from YAML or use defaults
    let wf_config = match &config.walk_forward {
        Some(wf) => WalkForwardConfig {
            initial_train: wf.initial_train.unwrap_or(2500),
            target_test_window: wf.target_test_window.unwrap_or(1000),
            min_test_window: wf.min_test_window.unwrap_or(400),
            embargo: backtester.embargo_size(),
            target_steps: wf.target_steps.unwrap_or(3),
        },
        None => WalkForwardConfig {
            embargo: backtester.embargo_size(),
            ..WalkForwardConfig::default_1h()
        },
    };
    info!(
        "[{}:{}:{}] Walk-forward config: initial_train={}, test_window={}, min_test={}, embargo={}, steps={}",
        exchange, symbol, mode_str,
        wf_config.initial_train, wf_config.target_test_window,
        wf_config.min_test_window, wf_config.embargo, wf_config.target_steps
    );

    // Load data for this single symbol
    info!(
        "[{}:{}:{}] Loading {} days of data...",
        exchange, symbol, mode_str, config.lookback_days
    );
    if let Err(e) = backtester
        .load_data(std::slice::from_ref(&symbol), config.lookback_days)
        .await
    {
        error!(
            "[{}:{}:{}] Failed to load data: {}",
            exchange, symbol, mode_str, e
        );
    }

    // Load SPY reference data for cross-asset factors (Polygon only)
    if exchange == "Polygon" && symbol != "SPY" {
        if let Err(e) = backtester
            .load_reference_data("SPY", config.lookback_days)
            .await
        {
            warn!(
                "[{}:{}:{}] Failed to load SPY reference: {}",
                exchange, symbol, mode_str, e
            );
        }
    }

    // Resume from last generation for this (exchange, symbol, mode)
    use sqlx::Row;
    let resume_query = || {
        sqlx::query(
            "SELECT generation, best_genome, metadata FROM strategy_generations \
             WHERE exchange = $1 AND symbol = $2 AND mode = $3 ORDER BY generation DESC LIMIT 1",
        )
        .bind(exchange)
        .bind(&symbol)
        .bind(mode_str)
    };

    let apply_resume = |row: &sqlx::postgres::PgRow, ga: &mut AlpsGA| {
        if let Ok(max_gen) = row.try_get::<i32, _>("generation") {
            info!(
                "[{}:{}:{}] Resuming from generation {}",
                exchange, symbol, mode_str, max_gen
            );
            ga.generation = max_gen as usize + 1;
        }
        if let Ok(best_tokens) = row.try_get::<Vec<i32>, _>("best_genome") {
            // Check genome compatibility via stored feat_offset in metadata.
            // Old genomes from 13-factor space encode tokens differently than
            // 25-factor space — operator indices shift, making old genomes invalid.
            let stored_offset = row
                .try_get::<serde_json::Value, _>("metadata")
                .ok()
                .and_then(|m| m.get("feat_offset").and_then(|v| v.as_u64()))
                .map(|v| v as usize);

            match stored_offset {
                Some(old_offset) if old_offset != feat_offset => {
                    warn!(
                        "[{}:{}:{}] Skipping genome resume: feat_offset changed ({}→{}), old tokens incompatible",
                        exchange, symbol, mode_str, old_offset, feat_offset
                    );
                }
                None => {
                    // No stored feat_offset — legacy genome, assume incompatible
                    warn!(
                        "[{}:{}:{}] Skipping genome resume: no feat_offset in metadata, likely from older factor config (current={})",
                        exchange, symbol, mode_str, feat_offset
                    );
                }
                _ => {
                    // Compatible: same feat_offset
                    let tokens: Vec<usize> = best_tokens.into_iter().map(|x| x as usize).collect();
                    ga.best_genome = Some(genetic::Genome {
                        tokens,
                        fitness: 0.0,
                        age: 0,
                    });
                }
            }
        }
    };

    match resume_query().fetch_optional(&pool).await {
        Ok(Some(row)) => apply_resume(&row, &mut ga),
        Ok(None) => {
            info!(
                "[{}:{}:{}] No previous generations found, starting fresh",
                exchange, symbol, mode_str
            );
        }
        Err(e) => {
            error!(
                "[{}:{}:{}] Resume query failed: {}, retrying in 2s...",
                exchange, symbol, mode_str, e
            );
            tokio::time::sleep(Duration::from_secs(2)).await;
            match resume_query().fetch_optional(&pool).await {
                Ok(Some(row)) => apply_resume(&row, &mut ga),
                Ok(None) => {
                    info!(
                        "[{}:{}:{}] No previous generations on retry, starting fresh",
                        exchange, symbol, mode_str
                    );
                }
                Err(e2) => {
                    error!(
                        "[{}:{}:{}] Resume retry failed: {}, starting from gen 0",
                        exchange, symbol, mode_str, e2
                    );
                }
            }
        }
    }

    // Cleanup orphaned generations from previous runs
    let start_gen = ga.generation;
    if start_gen > 0 {
        // Successful resume: delete orphaned generations beyond our starting point
        match sqlx::query(
            "DELETE FROM strategy_generations \
             WHERE exchange = $1 AND symbol = $2 AND mode = $3 AND generation >= $4",
        )
        .bind(exchange)
        .bind(&symbol)
        .bind(mode_str)
        .bind(start_gen as i32)
        .execute(&pool)
        .await
        {
            Ok(r) if r.rows_affected() > 0 => {
                warn!(
                    "[{}:{}:{}] Cleaned {} orphaned generations (>= gen {})",
                    exchange,
                    symbol,
                    mode_str,
                    r.rows_affected(),
                    start_gen
                );
            }
            Err(e) => {
                error!(
                    "[{}:{}:{}] Failed to clean orphaned generations: {}",
                    exchange, symbol, mode_str, e
                );
            }
            _ => {}
        }
    } else {
        // gen 0 = fresh start or resume failure — check for stale data
        let old_max: Option<i32> = sqlx::query_scalar(
            "SELECT MAX(generation) FROM strategy_generations \
             WHERE exchange = $1 AND symbol = $2 AND mode = $3",
        )
        .bind(exchange)
        .bind(&symbol)
        .bind(mode_str)
        .fetch_one(&pool)
        .await
        .ok()
        .flatten();

        if let Some(max_gen) = old_max {
            warn!(
                "[{}:{}:{}] Starting from gen 0 but found data up to gen {}. Cleaning stale data.",
                exchange, symbol, mode_str, max_gen
            );
            let _ = sqlx::query(
                "DELETE FROM strategy_generations \
                 WHERE exchange = $1 AND symbol = $2 AND mode = $3",
            )
            .bind(exchange)
            .bind(&symbol)
            .bind(mode_str)
            .execute(&pool)
            .await;
        }
    }

    // Promotion rate tracker: rolling window of 50 generations for smoothed rates
    let mut promo_tracker = PromotionRateTracker::new(50);

    // Evolution loop
    loop {
        let gen = ga.generation;

        // Adaptive K: target ~300 bars per fold, K in [3, 8]
        let data_len = backtester.data_length(&symbol);
        let k = ((data_len as f64 / 300.0).round() as usize).clamp(3, 8);

        // Evaluate each genome via K-fold temporal cross-validation
        for genome in ga.all_genomes_mut() {
            backtester.evaluate_symbol_kfold(genome, &symbol, k, mode);
        }
        let promo_stats = ga.evolve();
        promo_tracker.push(promo_stats.clone());

        // Log ALPS layer summary + promotion rates periodically
        if gen.is_multiple_of(10) {
            let summary = ga.layer_summary();
            let rates = promo_tracker.avg_rates();
            let rate_strs: Vec<String> = rates
                .iter()
                .enumerate()
                .map(|(i, r)| match r {
                    Some(v) => format!("L{}→{}:{:.1}%", i, i + 1, v * 100.0),
                    None => format!("L{}→{}:n/a", i, i + 1),
                })
                .collect();
            info!(
                "[{}:{}:{}] Gen {} ALPS layers: {:?} promo: [{}] (window={}) top_purged: {} total_pop: {}",
                exchange,
                symbol,
                mode_str,
                gen,
                summary,
                rate_strs.join(", "),
                promo_tracker.history.len().min(promo_tracker.window),
                promo_stats.top_purged,
                ga.total_population()
            );
        }

        if let Some(best) = ga.best_genome.clone() {
            let wf_result =
                backtester.evaluate_walk_forward_oos_with_config(&best, &symbol, mode, &wf_config);
            let oos_psr = wf_result.aggregate_psr;
            let fold_psrs = backtester.evaluate_symbol_fold_psr_detail(&best, &symbol, k, mode);
            info!(
                "[{}:{}:{}] Gen {} IS: {:.4} OOS: {:.4} tokens: {} age: {} K: {} folds: {:?} wf_steps: {}/{}",
                exchange,
                symbol,
                mode_str,
                gen,
                best.fitness,
                oos_psr,
                best.tokens.len(),
                best.age,
                k,
                fold_psrs,
                wf_result.num_valid_steps,
                wf_result.num_steps
            );

            // IS-OOS gap monitoring with sentinel awareness
            let is_oos_gap = best.fitness - oos_psr;
            if best.fitness > 1.0 && is_sentinel(oos_psr) {
                warn!(
                    "[{}:{}:{}] Gen {} — OOS sentinel: {} (IS={:.3})",
                    exchange,
                    symbol,
                    mode_str,
                    gen,
                    sentinel_label(oos_psr),
                    best.fitness
                );
            } else if best.fitness > 1.0 && oos_psr < 0.0 && is_oos_gap > 2.0 {
                warn!(
                    "[{}:{}:{}] Gen {} — IS-OOS divergence (IS={:.3}, OOS={:.3}, gap={:.3}, wf_steps={})",
                    exchange, symbol, mode_str, gen, best.fitness, oos_psr, is_oos_gap,
                    wf_result.num_valid_steps
                );
            }

            let strategy_id = format!("{}_{}_{}_gen_{}", exchange_lower, symbol, mode_str, gen);
            let payload = serde_json::json!({
                "strategy_id": &strategy_id,
                "timestamp": chrono::Utc::now().timestamp(),
                "formula": best.tokens,
                "generation": gen,
                "fitness": best.fitness,
                "oos_psr": oos_psr,
                "age": best.age,
                "fold_psrs": fold_psrs,
                "best_tokens": best.tokens,
                "feat_offset": feat_offset,
                "exchange": exchange,
                "symbol": symbol,
                "mode": mode_str,
                "resolution": resolution,
                "walk_forward": {
                    "num_steps": wf_result.num_steps,
                    "num_valid": wf_result.num_valid_steps,
                    "mean_psr": wf_result.mean_psr,
                    "std_psr": wf_result.std_psr,
                    "steps": wf_result.steps,
                    "failure_mode": wf_result.failure_mode,
                },
                "alps_promotion": {
                    "promoted": promo_stats.promoted,
                    "discarded": promo_stats.discarded,
                    "top_purged": promo_stats.top_purged,
                    "rolling_rates": {
                        "l0_l1": promo_tracker.avg_rate(0),
                        "l1_l2": promo_tracker.avg_rate(1),
                        "l2_l3": promo_tracker.avg_rate(2),
                        "l3_l4": promo_tracker.avg_rate(3),
                    },
                    "rolling_window": promo_tracker.history.len().min(promo_tracker.window),
                    "rolling_total_promoted": promo_tracker.total_promoted_in_window(),
                    "rolling_total_discarded": promo_tracker.total_discarded_in_window(),
                },
                "meta": {
                    "name": format!("{}-{}-Gen{}-PSR{:.2}", symbol, mode_str, gen, best.fitness),
                    "description": format!("{} {} Evolved Strategy. IS PSR: {:.4}, OOS PSR: {:.4}", symbol, mode_str, best.fitness, oos_psr)
                }
            });
            let payload_str = payload.to_string();

            // Redis pub/sub + state
            let _: () = redis_conn
                .publish(&redis_channel, &payload_str)
                .await
                .unwrap_or(());
            let _: () = redis_conn
                .set(&redis_key_status, &payload_str)
                .await
                .unwrap_or(());

            // DB persist with (exchange, symbol, mode, generation) key
            let tokens_i32: Vec<i32> = best.tokens.iter().map(|&x| x as i32).collect();
            let _ = sqlx::query(
                "INSERT INTO strategy_generations (exchange, symbol, mode, generation, fitness, best_genome, metadata, strategy_id) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8) \
                 ON CONFLICT (exchange, symbol, mode, generation) DO UPDATE SET fitness = $5, best_genome = $6, metadata = $7, strategy_id = $8"
            )
            .bind(exchange)
            .bind(&symbol)
            .bind(mode_str)
            .bind(gen as i32)
            .bind(best.fitness)
            .bind(&tokens_i32)
            .bind(&payload)
            .bind(&strategy_id)
            .execute(&pool)
            .await
            .map_err(|e| error!("[{}:{}:{}] DB persist failed: {}", exchange, symbol, mode_str, e));

            // Single-symbol backtest every 5 generations
            if gen.is_multiple_of(5) {
                let tokens_i32: Vec<i32> = best.tokens.iter().map(|&x| x as i32).collect();
                match backtester
                    .run_detailed_simulation(&tokens_i32, &symbol, config.lookback_days, mode)
                    .await
                {
                    Ok(sim) => {
                        let m = sim["metrics"].as_object().unwrap();
                        info!(
                            "[{}:{}:{}] Backtest: PnL={:.2}%, Sharpe={:.2}, Sortino={:.2}, PF={:.2}, MaxDD={:.2}%",
                            exchange,
                            symbol,
                            mode_str,
                            m["total_return"].as_f64().unwrap_or(0.0) * 100.0,
                            m.get("sharpe_ratio").and_then(|v| v.as_f64()).unwrap_or(0.0),
                            m.get("sortino_ratio").and_then(|v| v.as_f64()).unwrap_or(0.0),
                            m.get("profit_factor").and_then(|v| v.as_f64()).unwrap_or(0.0),
                            m.get("max_drawdown").and_then(|v| v.as_f64()).unwrap_or(0.0) * 100.0,
                        );

                        // Keep only the latest backtest per (exchange, symbol, mode)
                        let _ = sqlx::query(
                            "DELETE FROM backtest_results \
                             WHERE token_address = $1 AND mode = $2 \
                             AND strategy_id LIKE $3",
                        )
                        .bind(&symbol)
                        .bind(mode_str)
                        .bind(format!("{}_%", exchange_lower))
                        .execute(&pool)
                        .await;

                        let _ = sqlx::query(
                            "INSERT INTO backtest_results \
                             (strategy_id, genome, token_address, mode, pnl_percent, win_rate, total_trades, \
                              sharpe_ratio, max_drawdown, equity_curve, trades, metrics, created_at) \
                             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, NOW())"
                        )
                        .bind(&strategy_id)
                        .bind(&tokens_i32)
                        .bind(&symbol)
                        .bind(mode_str)
                        .bind(m["total_return"].as_f64().unwrap_or(0.0))
                        .bind(m["win_rate"].as_f64().unwrap_or(0.0))
                        .bind(m["total_trades"].as_i64().unwrap_or(0) as i32)
                        .bind(m.get("sharpe_ratio").and_then(|v| v.as_f64()).unwrap_or(0.0))
                        .bind(m.get("max_drawdown").and_then(|v| v.as_f64()).unwrap_or(0.0))
                        .bind(&sim["equity_curve"])
                        .bind(&sim["trades"])
                        .bind(&sim["metrics"])
                        .execute(&pool)
                        .await
                        .map_err(|e| error!("[{}:{}:{}] Backtest persist failed: {}", exchange, symbol, mode_str, e));
                    }
                    Err(e) => error!(
                        "[{}:{}:{}] Backtest sim failed: {}",
                        exchange, symbol, mode_str, e
                    ),
                }
            }

            // Cleanup old + orphaned generations for this (symbol, mode)
            if gen.is_multiple_of(10) && gen > 100 {
                let _ = sqlx::query(
                    "DELETE FROM strategy_generations \
                     WHERE exchange = $1 AND symbol = $2 AND mode = $3 \
                     AND (generation < $4 OR generation > $5)",
                )
                .bind(exchange)
                .bind(&symbol)
                .bind(mode_str)
                .bind(gen as i32 - 1000)
                .bind(gen as i32 + 100)
                .execute(&pool)
                .await;
            }
        }

        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}
