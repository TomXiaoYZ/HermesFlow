use rayon::prelude::*;
use redis::AsyncCommands;
use serde::Deserialize;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info, warn};

mod api;
mod backtest;
mod genetic;
mod genome_decoder;
mod llm_oracle;
mod mcts;

use backtest::StrategyMode;
use backtest::{
    adjust_threshold_params, is_sentinel, sentinel_label, Backtester, ThresholdConfig,
    UtilizationTracker, WalkForwardConfig,
};
use backtest_engine::config::{FactorConfig, MultiTimeframeFactorConfig};
use genetic::{AlpsGA, PromotionStats};
use llm_oracle::LlmOracleConfig;

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

/// Rolling-window tracker for too_few_trades rate.
/// Used by the LLM oracle trigger to detect when evolution is stuck
/// producing genomes that can't generate enough trades.
struct TftTracker {
    history: Vec<bool>,
    window: usize,
    cursor: usize,
    count: usize,
}

impl TftTracker {
    fn new(window: usize) -> Self {
        Self {
            history: Vec::with_capacity(window),
            window,
            cursor: 0,
            count: 0,
        }
    }

    fn push(&mut self, is_tft: bool) {
        if self.history.len() < self.window {
            self.history.push(is_tft);
        } else {
            self.history[self.cursor] = is_tft;
        }
        self.cursor = (self.cursor + 1) % self.window;
        self.count += 1;
    }

    /// Fraction of recent generations where best genome had too_few_trades.
    fn rate(&self) -> f64 {
        let n = self.history.len();
        if n == 0 {
            return 0.0;
        }
        let tft_count = self.history.iter().filter(|&&v| v).count();
        tft_count as f64 / n as f64
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

/// P3 multi-timeframe configuration, loaded from generator.yaml.
#[derive(Debug, Deserialize, Clone)]
pub struct MultiTimeframeYamlConfig {
    pub enabled: bool,
    pub resolutions: Vec<String>,
    /// P6-1A: Publication delay per resolution (seconds).
    /// A bar's data is not available until `bar_close_timestamp + delay`.
    /// Prevents look-ahead bias when aligning lower-frequency bars to higher-frequency.
    #[serde(default)]
    pub publication_delays: std::collections::HashMap<String, i64>,
}

/// P7-1A: MCTS configuration loaded from generator.yaml `mcts` section.
/// Defaults match generator.yaml values; `use_max_reward=true` enables
/// Extreme Bandit PUCT (tracks max OOS-PSR, not mean) per Gemini advisor.
#[derive(Debug, Deserialize, Clone)]
pub struct MctsYamlConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_mcts_budget")]
    pub budget: usize,
    #[serde(default = "default_mcts_seeds")]
    pub seeds_per_round: usize,
    #[serde(default = "default_mcts_interval")]
    pub interval: usize,
    #[serde(default = "default_mcts_exploration")]
    pub exploration_c: f64,
    #[serde(default = "default_mcts_max_len")]
    pub max_length: usize,
    #[serde(default = "default_true")]
    pub use_max_reward: bool,
}

fn default_mcts_budget() -> usize { 1000 }
fn default_mcts_seeds() -> usize { 5 }
fn default_mcts_interval() -> usize { 50 }
fn default_mcts_exploration() -> f64 { 1.414 }
fn default_mcts_max_len() -> usize { 20 }
fn default_true() -> bool { true }

impl Default for MctsYamlConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            budget: default_mcts_budget(),
            seeds_per_round: default_mcts_seeds(),
            interval: default_mcts_interval(),
            exploration_c: default_mcts_exploration(),
            max_length: default_mcts_max_len(),
            use_max_reward: default_true(),
        }
    }
}

/// Per-exchange evolution config, loaded from config/generator.yaml.
#[derive(Debug, Deserialize, Clone)]
pub struct ExchangeConfig {
    pub exchange: String,
    pub resolution: String,
    pub lookback_days: i64,
    pub factor_config: String,
    pub multi_timeframe: Option<MultiTimeframeYamlConfig>,
    pub walk_forward: Option<WalkForwardYamlConfig>,
}

#[derive(Debug, Deserialize)]
struct GeneratorConfig {
    exchanges: Vec<ExchangeConfig>,
    #[serde(default)]
    llm_oracle: LlmOracleConfig,
    #[serde(default)]
    threshold_config: ThresholdConfig,
    #[serde(default)]
    ensemble: backtest::ensemble::EnsembleConfig,
    #[serde(default)]
    mcts: MctsYamlConfig,
    #[serde(default)]
    lfdr: backtest::hypothesis::LfdrConfig,
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
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&db_url)
        .await?;

    common::heartbeat::spawn_heartbeat("strategy-generator", &redis_url);

    // Load generator config
    let config_path =
        env::var("GENERATOR_CONFIG").unwrap_or_else(|_| "config/generator.yaml".to_string());
    let (exchange_configs, oracle_config, threshold_config, ensemble_config, mcts_config, lfdr_config) =
        load_exchange_configs(&config_path);
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

    // P6a-G2: Dedicated rayon thread pool for CPU-bound genome evaluation.
    // Reserve 2 cores for Tokio I/O + OS, rest for parallel fitness evaluation.
    let rayon_threads = num_cpus::get().saturating_sub(2).max(1);
    let rayon_pool = Arc::new(
        rayon::ThreadPoolBuilder::new()
            .num_threads(rayon_threads)
            .thread_name(|idx| format!("genome-eval-{}", idx))
            .build()
            .expect("failed to create rayon thread pool"),
    );
    info!(
        "Created rayon pool: {} threads (CPUs: {})",
        rayon_threads,
        num_cpus::get()
    );

    // P7-1B: Dedicated MCTS rayon pool (isolated from genome evaluation)
    let mcts_threads: usize = env::var("MCTS_THREADS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(2);
    let mcts_pool = Arc::new(
        rayon::ThreadPoolBuilder::new()
            .num_threads(mcts_threads)
            .thread_name(|idx| format!("mcts-{}", idx))
            .build()
            .expect("failed to create MCTS rayon pool"),
    );
    info!("Created MCTS rayon pool: {} threads", mcts_threads);

    let mcts_config = Arc::new(mcts_config);
    let lfdr_config = Arc::new(lfdr_config);

    // Spawn two evolution tasks per (exchange, symbol) pair: long_only + long_short
    let mut handles = Vec::new();
    for ec in &exchange_configs {
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
                let oracle_cfg = oracle_config.clone();
                let thresh_cfg = threshold_config.clone();
                let rayon_pool = rayon_pool.clone();
                let mcts_pool = mcts_pool.clone();
                let mcts_cfg = mcts_config.clone();
                let sym = symbol.clone();
                let ex_name = ec.exchange.clone();
                let handle = tokio::spawn(async move {
                    if let Err(e) = run_symbol_evolution(
                        pool,
                        &redis_url,
                        config,
                        oracle_cfg,
                        thresh_cfg,
                        rayon_pool,
                        mcts_pool,
                        mcts_cfg,
                        sym.clone(),
                        mode,
                    )
                    .await
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

    // P5: Spawn ensemble rebalance loop per exchange (if enabled)
    if ensemble_config.enabled {
        for ec in &exchange_configs {
            let pool_ens = pool.clone();
            let redis_url_ens = redis_url.clone();
            let ens_cfg = ensemble_config.clone();
            let thresh_cfg = threshold_config.clone();
            let lfdr_cfg = (*lfdr_config).clone();
            let exchange = ec.exchange.clone();
            let resolution = ec.resolution.clone();
            let factor_config_path = ec.factor_config.clone();
            let lookback_days = ec.lookback_days;
            let handle = tokio::spawn(async move {
                if let Err(e) = run_ensemble_loop(
                    pool_ens,
                    &redis_url_ens,
                    &exchange,
                    &resolution,
                    &factor_config_path,
                    lookback_days,
                    thresh_cfg,
                    ens_cfg,
                    lfdr_cfg,
                )
                .await
                {
                    error!("[{}] Ensemble rebalance loop failed: {}", exchange, e);
                }
            });
            handles.push(handle);
        }
        // P6b-F4: Spawn daily ensemble walk-forward backtest per exchange
        for ec in &exchange_configs {
            let pool_bt = pool.clone();
            let exchange = ec.exchange.clone();
            let handle = tokio::spawn(async move {
                run_daily_backtest_loop(pool_bt, &exchange).await;
            });
            handles.push(handle);
        }
    } else {
        info!("Ensemble rebalance loop disabled by config");
    }

    // Wait for all evolution tasks (they run forever unless errored)
    for h in handles {
        let _ = h.await;
    }

    Ok(())
}

fn load_exchange_configs(
    path: &str,
) -> (
    Vec<ExchangeConfig>,
    LlmOracleConfig,
    ThresholdConfig,
    backtest::ensemble::EnsembleConfig,
    MctsYamlConfig,
    backtest::hypothesis::LfdrConfig,
) {
    match std::fs::read_to_string(path) {
        Ok(content) => match serde_yaml::from_str::<GeneratorConfig>(&content) {
            Ok(cfg) => {
                info!(
                    "LLM oracle config: enabled={}, provider={}, model={}",
                    cfg.llm_oracle.enabled, cfg.llm_oracle.provider, cfg.llm_oracle.model
                );
                info!(
                    "Threshold config: LO upper_pct={}, LS upper_pct={}/lower_pct={}, {} overrides",
                    cfg.threshold_config.long_only.percentile_upper,
                    cfg.threshold_config.long_short.percentile_upper,
                    cfg.threshold_config.long_short.percentile_lower,
                    cfg.threshold_config.overrides.len()
                );
                info!(
                    "Ensemble config: enabled={}, max_strategies={}, rebalance_interval={}min",
                    cfg.ensemble.enabled,
                    cfg.ensemble.max_total_strategies,
                    cfg.ensemble.rebalance_interval_minutes
                );
                info!(
                    "MCTS config: enabled={}, budget={}, interval={}, seeds={}, use_max_reward={}",
                    cfg.mcts.enabled, cfg.mcts.budget, cfg.mcts.interval,
                    cfg.mcts.seeds_per_round, cfg.mcts.use_max_reward
                );
                info!(
                    "lFDR config: enabled={}, fdr_level={}, min_cluster_size={}",
                    cfg.lfdr.enabled, cfg.lfdr.fdr_level, cfg.lfdr.min_cluster_size
                );
                (
                    cfg.exchanges,
                    cfg.llm_oracle,
                    cfg.threshold_config,
                    cfg.ensemble,
                    cfg.mcts,
                    cfg.lfdr,
                )
            }
            Err(e) => {
                warn!("Failed to parse {}: {}. Falling back to default.", path, e);
                (
                    default_exchange_configs(),
                    LlmOracleConfig::default(),
                    ThresholdConfig::default(),
                    backtest::ensemble::EnsembleConfig::default(),
                    MctsYamlConfig::default(),
                    backtest::hypothesis::LfdrConfig::default(),
                )
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
            (
                vec![ExchangeConfig {
                    exchange,
                    resolution,
                    lookback_days,
                    factor_config,
                    multi_timeframe: None,
                    walk_forward: None,
                }],
                LlmOracleConfig::default(),
                ThresholdConfig::default(),
                backtest::ensemble::EnsembleConfig::default(),
                MctsYamlConfig::default(),
                backtest::hypothesis::LfdrConfig::default(),
            )
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
            multi_timeframe: None,
            walk_forward: None,
        },
        ExchangeConfig {
            exchange: "Polygon".to_string(),
            resolution: "1d".to_string(),
            lookback_days: 365,
            factor_config: "config/factors-stock.yaml".to_string(),
            multi_timeframe: None,
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
#[allow(clippy::too_many_arguments)]
async fn run_symbol_evolution(
    pool: PgPool,
    redis_url: &str,
    config: ExchangeConfig,
    oracle_config: LlmOracleConfig,
    threshold_config: ThresholdConfig,
    rayon_pool: Arc<rayon::ThreadPool>,
    mcts_pool: Arc<rayon::ThreadPool>,
    mcts_config: Arc<MctsYamlConfig>,
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

    // P3: Multi-timeframe factor stacking
    let mtf_enabled = config
        .multi_timeframe
        .as_ref()
        .map(|m| m.enabled)
        .unwrap_or(false);
    let mtf_resolutions: Vec<String> = config
        .multi_timeframe
        .as_ref()
        .filter(|m| m.enabled)
        .map(|m| m.resolutions.clone())
        .unwrap_or_default();
    // P6-1A: Extract per-resolution publication delays (seconds)
    let publication_delays: std::collections::HashMap<String, i64> = config
        .multi_timeframe
        .as_ref()
        .filter(|m| m.enabled)
        .map(|m| m.publication_delays.clone())
        .unwrap_or_default();

    let mtf_config = if mtf_enabled {
        Some(MultiTimeframeFactorConfig::new(
            factor_config.clone(),
            mtf_resolutions.clone(),
        ))
    } else {
        None
    };

    let (feat_offset, factor_names) =
        if let Some(ref mtf) = mtf_config {
            let names = mtf.factor_names();
            info!(
            "[{}:{}:{}] P3 MTF enabled: {} factors x {} resolutions = {} features (feat_offset={})",
            exchange, symbol, mode_str,
            mtf.base_feat_count(), mtf.resolutions.len(),
            mtf.feat_count(), mtf.feat_offset()
        );
            (mtf.feat_offset(), names)
        } else {
            let names: Vec<String> = factor_config
                .active_factors
                .iter()
                .map(|f| f.name.clone())
                .collect();
            (factor_config.feat_offset(), names)
        };

    let client = redis::Client::open(redis_url.to_string())?;
    let mut redis_conn = client.get_async_connection().await?;

    let redis_key_status = format!("strategy:{}:{}:{}:status", exchange_lower, symbol, mode_str);
    let redis_channel = format!(
        "strategy_updates:{}:{}:{}",
        exchange_lower, symbol, mode_str
    );

    let mut backtester = Backtester::with_threshold_config(
        pool.clone(),
        factor_config,
        exchange.clone(),
        resolution.clone(),
        threshold_config,
    );
    // Sync the backtester's VM feat_offset with the GA's.  When multi-timeframe
    // is enabled, the GA uses mtf.feat_offset() (e.g. 75) while the backtester's
    // VM was initialised from factor_config.feat_offset() (e.g. 25).  Without this
    // fix every genome token in the [25..75) range is misinterpreted as an operator
    // instead of a feature index, causing silent wrong results or VM failures.
    backtester.vm.feat_offset = feat_offset;
    let mut ga = AlpsGA::new(feat_offset);

    // P7-2B: CCIPCA diagnostic — tracks feature orthogonality per symbol
    // Initialized lazily after we know actual feature dimension from cache
    let mut ccipca: Option<backtest::incremental_pca::CcipcaState> = None;

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
        "[{}:{}:{}] Loading {} days of data{}...",
        exchange,
        symbol,
        mode_str,
        config.lookback_days,
        if mtf_enabled {
            " (multi-timeframe)"
        } else {
            ""
        }
    );
    if mtf_enabled {
        // P3: Load SPY reference first (MTF loads per-resolution internally)
        // then load multi-timeframe data
        if let Err(e) = backtester
            .load_data_multi_timeframe(
                std::slice::from_ref(&symbol),
                config.lookback_days,
                mtf_config.as_ref().unwrap(),
                &publication_delays,
            )
            .await
        {
            error!(
                "[{}:{}:{}] Failed to load MTF data: {}",
                exchange, symbol, mode_str, e
            );
        }
    } else {
        // P2: Single-resolution data loading
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
                    let len = tokens.len();
                    ga.best_genome = Some(genetic::Genome {
                        tokens,
                        fitness: 0.0,
                        age: 0,
                        block_mask: vec![0; len],
                        block_age: vec![0; len],
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

    // TFT tracker + oracle state for P2 LLM-guided mutation
    let mut tft_tracker = TftTracker::new(50);
    // P4: Utilization tracker for adaptive threshold tuning
    let mut util_tracker = UtilizationTracker::new(50);
    let mut last_oracle_gen: usize = 0;
    let mut last_oracle_time = std::time::Instant::now();
    let mut oracle_invocations: usize = 0;
    let mut oracle_injected_total: usize = 0;

    // P7-2B: Feed CCIPCA with cached feature data (diagnostic only)
    // Features shape: (n_symbols=1, n_factors, n_bars)
    if let Some(cached) = backtester.cache.get(&symbol) {
        let features = &cached.features;
        let n_feat = features.shape()[1].min(feat_offset); // axis 1 = factors
        let n_bars = features.shape()[2];                   // axis 2 = bars
        if n_feat >= 2 {
            // Lazily initialize CCIPCA with actual feature dimension
            let pca = ccipca.get_or_insert_with(|| {
                backtest::incremental_pca::CcipcaState::new(
                    n_feat,
                    backtest::incremental_pca::CcipcaConfig {
                        n_components: 5.min(n_feat),
                        amnesic: 2.0,
                        min_observations: 50,
                        enabled: true,
                    },
                )
            });
            // Feed last 200 bars (or all if fewer) as observations to CCIPCA
            let start_bar = n_bars.saturating_sub(200);
            for bar in start_bar..n_bars {
                let obs: Vec<f64> = (0..n_feat).map(|f| features[[0, f, bar]]).collect();
                let obs_arr = ndarray::Array1::from_vec(obs);
                pca.update_view(obs_arr.view());
            }
            if pca.is_valid() {
                let ev = pca.explained_variance();
                let total_var: f64 = ev.iter().sum();
                let ratios: Vec<f64> = ev.iter().map(|&v| if total_var > 0.0 { v / total_var } else { 0.0 }).collect();
                info!(
                    "[{}:{}:{}] CCIPCA: {} observations, top-{} explained variance ratios: {:?}",
                    exchange, symbol, mode_str, pca.n_observations(), ev.len(), ratios
                );
            }
        }
    }

    // Evolution loop
    loop {
        let gen = ga.generation;

        // Adaptive K: target ~300 bars per fold, K in [3, 8]
        let data_len = backtester.data_length(&symbol);
        let k = ((data_len as f64 / 300.0).round() as usize).clamp(3, 8);

        // P6a-G2: Parallel genome evaluation via rayon thread pool.
        // Each genome is independent; Backtester is Sync (immutable &self).
        let mut genomes = ga.all_genomes_mut();
        rayon_pool.install(|| {
            genomes.par_iter_mut().for_each(|genome| {
                backtester.evaluate_symbol_kfold(genome, &symbol, k, mode);
            });
        });
        let promo_stats = ga.evolve();
        promo_tracker.push(promo_stats.clone());

        // ═══ P7-1C: MCTS Seed Injection ═══
        let mut mcts_injected_this_gen = 0usize;
        if mcts_config.enabled && gen > 0 && gen.is_multiple_of(mcts_config.interval) {
            let action_space = mcts::state::ActionSpace::new(feat_offset);
            let policy = mcts::policy::UniformPolicy;
            let search_config = mcts::search::MctsConfig {
                budget: mcts_config.budget,
                exploration_c: mcts_config.exploration_c,
                seeds_per_round: mcts_config.seeds_per_round,
                max_length: mcts_config.max_length,
                use_max_reward: mcts_config.use_max_reward,
                deception_ngram_size: 0,
                deception_decay: 0.1,
            };

            let backtester_ref = &backtester;
            let symbol_ref = &symbol;
            let result = mcts_pool.install(|| {
                mcts::search::run_mcts_round(
                    &action_space,
                    &policy,
                    &search_config,
                    |tokens: &[u32]| {
                        let usize_tokens: Vec<usize> = tokens.iter().map(|&t| t as usize).collect();
                        let len = usize_tokens.len();
                        let mut temp = genetic::Genome {
                            tokens: usize_tokens,
                            fitness: 0.0,
                            age: 0,
                            block_mask: vec![0; len],
                            block_age: vec![0; len],
                        };
                        backtester_ref.evaluate_symbol_kfold(&mut temp, symbol_ref, k, mode);
                        temp.fitness
                    },
                )
            });

            // Filter positive-PSR seeds, convert to Genome, inject into ALPS L0
            let seeds: Vec<genetic::Genome> = result
                .formulas
                .iter()
                .zip(result.scores.iter())
                .filter(|(_, &score)| score > 0.0)
                .map(|(formula, _)| {
                    let tokens: Vec<usize> = formula.iter().map(|&t| t as usize).collect();
                    let len = tokens.len();
                    genetic::Genome {
                        tokens,
                        fitness: 0.0,
                        age: 0,
                        block_mask: vec![0; len],
                        block_age: vec![0; len],
                    }
                })
                .collect();

            mcts_injected_this_gen = seeds.len();
            if !seeds.is_empty() {
                ga.inject_genomes(0, seeds);
                info!(
                    "[{}:{}:{}] Gen {} MCTS: injected {}/{} seeds into L0 (budget={}, unique={})",
                    exchange, symbol, mode_str, gen, mcts_injected_this_gen,
                    result.formulas.len(), result.total_rollouts, result.unique_terminals
                );
            }
        }

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

            // P7-5B: Log genome diversity every 50 gens
            if gen.is_multiple_of(50) {
                let diversity = ga.layer_diversity();
                let div_strs: Vec<String> = diversity
                    .iter()
                    .map(|(i, n, d)| format!("L{}:{:.2}(n={})", i, d, n))
                    .collect();
                info!(
                    "[{}:{}:{}] Gen {} diversity (Hamming): [{}]",
                    exchange, symbol, mode_str, gen, div_strs.join(", ")
                );
            }
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
            // P6-3C: During evolution phase, VM failures and sentinels are expected
            // (random genomes in ALPS L0 produce NaN/None). Log at DEBUG to avoid
            // flooding Discord webhook. Only paper/live errors should reach ERROR/WARN.
            let is_oos_gap = best.fitness - oos_psr;
            if best.fitness > 1.0 && is_sentinel(oos_psr) {
                debug!(
                    "[{}:{}:{}] Gen {} — OOS sentinel: {} (IS={:.3})",
                    exchange,
                    symbol,
                    mode_str,
                    gen,
                    sentinel_label(oos_psr),
                    best.fitness
                );
            } else if best.fitness > 1.0 && oos_psr < 0.0 && is_oos_gap > 2.0 {
                debug!(
                    "[{}:{}:{}] Gen {} — IS-OOS divergence (IS={:.3}, OOS={:.3}, gap={:.3}, wf_steps={})",
                    exchange, symbol, mode_str, gen, best.fitness, oos_psr, is_oos_gap,
                    wf_result.num_valid_steps
                );
            }

            // Track TFT rate for LLM oracle trigger
            let is_tft = wf_result.failure_mode.as_deref() == Some("too_few_trades");
            tft_tracker.push(is_tft);

            // P4: Feed utilization metrics from walk-forward steps
            let wf_total_bars: u32 = wf_result
                .steps
                .iter()
                .map(|s| (s.test_end - s.test_start) as u32)
                .sum();
            let wf_active_bars: u32 = wf_result.steps.iter().map(|s| s.active_bars).sum();
            let wf_long_bars: u32 = wf_result.steps.iter().map(|s| s.long_bars).sum();
            let wf_short_bars: u32 = wf_result.steps.iter().map(|s| s.short_bars).sum();
            util_tracker.push(wf_total_bars, wf_active_bars, wf_long_bars, wf_short_bars);

            // P4: Adaptive threshold adjustment every 50 generations
            if gen > 0
                && gen.is_multiple_of(50)
                && adjust_threshold_params(
                    &mut backtester.threshold_config,
                    &symbol,
                    mode,
                    &util_tracker,
                )
            {
                let resolved_upper = backtester.threshold_config.resolve_upper(&symbol, mode);
                info!(
                    "[{}:{}:{}] Gen {} — threshold adjusted: util={:.2}%, long_r={:.2}%, short_r={:.2}%, upper_pct={:.1}",
                    exchange, symbol, mode_str, gen,
                    util_tracker.utilization() * 100.0,
                    util_tracker.long_ratio() * 100.0,
                    util_tracker.short_ratio() * 100.0,
                    resolved_upper.percentile * 100.0,
                );
            }

            // ═══ LLM Oracle Trigger Check ═══
            let mut oracle_injected_this_gen = 0usize;
            let mut trigger_reason: Option<&str> = None;
            let mut oracle_log: Option<llm_oracle::OracleResult> = None;
            let mut oracle_cross_elites: Vec<llm_oracle::CrossSymbolElite> = Vec::new();
            if oracle_config.enabled {
                let trigger = should_trigger_oracle(
                    gen,
                    &promo_tracker,
                    &tft_tracker,
                    last_oracle_gen,
                    last_oracle_time,
                    &oracle_config,
                );

                if let Some(reason) = trigger {
                    trigger_reason = Some(reason);
                    // Collect elites from all layers (3 per layer → 15 total)
                    // Clone to release borrow on `ga` before inject
                    let elites_owned: Vec<(usize, genetic::Genome)> = ga
                        .collect_elites(3)
                        .into_iter()
                        .map(|(layer, g)| (layer, g.clone()))
                        .collect();

                    let elites: Vec<llm_oracle::EliteContext> = elites_owned
                        .iter()
                        .map(|(layer_idx, genome)| llm_oracle::EliteContext {
                            formula: genome_decoder::decode_genome(
                                &genome.tokens,
                                feat_offset,
                                &factor_names,
                            ),
                            fitness: genome.fitness,
                            oos_psr: 0.0,
                            layer: *layer_idx,
                            age: genome.age,
                        })
                        .collect();

                    let existing_tokens: Vec<Vec<usize>> =
                        elites_owned.iter().map(|(_, g)| g.tokens.clone()).collect();

                    let cross_symbol_elites = fetch_cross_symbol_elites(
                        &pool,
                        exchange,
                        &symbol,
                        mode_str,
                        feat_offset,
                        &factor_names,
                        10,
                    )
                    .await;
                    if !cross_symbol_elites.is_empty() {
                        info!(
                            "[{}:{}:{}] Cross-symbol context: {} elites from other symbols",
                            exchange,
                            symbol,
                            mode_str,
                            cross_symbol_elites.len()
                        );
                    }

                    oracle_cross_elites = cross_symbol_elites.clone();

                    let ctx = llm_oracle::OracleContext {
                        symbol: symbol.clone(),
                        mode: mode_str.to_string(),
                        generation: gen,
                        feat_offset,
                        factor_names: factor_names.clone(),
                        best_oos_psr: oos_psr,
                        tft_rate: tft_tracker.rate(),
                        elites,
                        genomes_requested: oracle_config.genomes_per_invocation,
                        cross_symbol_elites,
                    };

                    match llm_oracle::generate_mutations(&oracle_config, &ctx, &existing_tokens)
                        .await
                    {
                        Ok(mut result) => {
                            oracle_injected_this_gen = result.genomes.len();
                            oracle_invocations += 1;
                            last_oracle_gen = gen;
                            last_oracle_time = std::time::Instant::now();
                            info!(
                                "[{}:{}:{}] LLM oracle: {}/{} valid genomes injected into L0 (invocation #{}, total injected: {})",
                                exchange, symbol, mode_str,
                                oracle_injected_this_gen, result.raw_count,
                                oracle_invocations, oracle_injected_total + oracle_injected_this_gen
                            );
                            if oracle_injected_this_gen > 0 {
                                // Extract genomes before moving result into oracle_log
                                let genomes = std::mem::take(&mut result.genomes);
                                oracle_log = Some(result);
                                ga.inject_genomes(0, genomes);
                                oracle_injected_total += oracle_injected_this_gen;
                            } else {
                                oracle_log = Some(result);
                            }
                        }
                        Err(e) => {
                            warn!(
                                "[{}:{}:{}] LLM oracle failed: {}",
                                exchange, symbol, mode_str, e
                            );
                            // Update cooldown to avoid hammering a broken API
                            last_oracle_gen = gen;
                            last_oracle_time = std::time::Instant::now();
                        }
                    }
                }
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
                "llm_oracle": build_oracle_metadata(&OracleSnapshot {
                    enabled: oracle_config.enabled,
                    invocations: oracle_invocations,
                    total_injected: oracle_injected_total,
                    injected_this_gen: oracle_injected_this_gen,
                    last_oracle_gen,
                    tft_rate: tft_tracker.rate(),
                    trigger_reason,
                    log: &oracle_log,
                    cross_symbol_elites: &oracle_cross_elites,
                }),
                "mcts": {
                    "enabled": mcts_config.enabled,
                    "injected_this_gen": mcts_injected_this_gen,
                    "interval": mcts_config.interval,
                    "budget": mcts_config.budget,
                    "use_max_reward": mcts_config.use_max_reward,
                },
                "utilization": {
                    "long_ratio": util_tracker.long_ratio(),
                    "short_ratio": util_tracker.short_ratio(),
                    "total_utilization": util_tracker.utilization(),
                    "window": util_tracker.len(),
                },
                "meta": {
                    "name": format!("{}-{}-Gen{}-PSR{:.2}", symbol, mode_str, gen, best.fitness),
                    "description": format!("{} {} Evolved Strategy. IS PSR: {:.4}, OOS PSR: {:.4}", symbol, mode_str, best.fitness, oos_psr)
                }
            });
            let payload_str = payload.to_string();

            // P7-3B: Defense-in-depth payload size guard (RUSTSEC-2024-0363 mitigation)
            const MAX_PAYLOAD_BYTES: usize = 16 * 1024 * 1024; // 16 MiB — our payloads are typically <10 KiB
            if payload_str.len() > MAX_PAYLOAD_BYTES {
                error!(
                    "[{}:{}:{}] Gen {} payload exceeds {}B limit ({}B) — skipping DB persist",
                    exchange, symbol, mode_str, gen, MAX_PAYLOAD_BYTES, payload_str.len()
                );
            }

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

/// Snapshot of oracle state for a single generation, used to build metadata.
struct OracleSnapshot<'a> {
    enabled: bool,
    invocations: usize,
    total_injected: usize,
    injected_this_gen: usize,
    last_oracle_gen: usize,
    tft_rate: f64,
    trigger_reason: Option<&'a str>,
    log: &'a Option<llm_oracle::OracleResult>,
    cross_symbol_elites: &'a [llm_oracle::CrossSymbolElite],
}

/// Build the `llm_oracle` metadata JSONB value.
///
/// Includes full interaction details (prompt, response, formulas, rejection reasons)
/// on generations where the oracle was invoked.
fn build_oracle_metadata(snap: &OracleSnapshot<'_>) -> serde_json::Value {
    let mut obj = serde_json::json!({
        "enabled": snap.enabled,
        "invocations": snap.invocations,
        "total_injected": snap.total_injected,
        "injected_this_gen": snap.injected_this_gen,
        "last_oracle_gen": snap.last_oracle_gen,
        "tft_rate_50gen": snap.tft_rate,
    });

    if let Some(reason) = snap.trigger_reason {
        obj["trigger_reason"] = serde_json::json!(reason);
    }

    if let Some(log) = snap.log {
        obj["prompt"] = serde_json::json!(log.prompt);
        obj["response"] = serde_json::json!(log.response_text);
        obj["parsed_formulas"] = serde_json::json!(log.parsed_formulas);
        obj["accepted_formulas"] = serde_json::json!(log.accepted_formulas);
        let rejected: Vec<serde_json::Value> = log
            .rejected_details
            .iter()
            .map(|(formula, reason)| serde_json::json!([formula, reason]))
            .collect();
        obj["rejected_details"] = serde_json::json!(rejected);

        if !snap.cross_symbol_elites.is_empty() {
            let cross: Vec<serde_json::Value> = snap
                .cross_symbol_elites
                .iter()
                .map(|e| {
                    serde_json::json!({
                        "symbol": e.symbol,
                        "formula": e.formula,
                        "is_psr": e.is_psr,
                        "oos_psr": e.oos_psr,
                    })
                })
                .collect();
            obj["cross_symbol_elites"] = serde_json::json!(cross);
        }
    }

    obj
}

/// Determine whether the LLM oracle should be invoked this generation.
///
/// Returns `Some(reason)` if triggered, `None` otherwise.
/// Two trigger conditions (either activates):
/// 1. **Primary**: L0→L1 promotion rate drops below threshold (random exploration failing)
/// 2. **Secondary**: too_few_trades rate exceeds threshold (stuck in non-trading genome space)
///
/// Both are gated by minimum generation warmup and cooldown periods.
fn should_trigger_oracle(
    gen: usize,
    promo_tracker: &PromotionRateTracker,
    tft_tracker: &TftTracker,
    last_oracle_gen: usize,
    last_oracle_time: std::time::Instant,
    config: &LlmOracleConfig,
) -> Option<&'static str> {
    // Minimum generation warmup
    if gen < config.min_generation {
        return None;
    }

    // Generation-based cooldown (skip on first invocation when last_oracle_gen == 0)
    if last_oracle_gen > 0 && gen.saturating_sub(last_oracle_gen) < config.cooldown_gens {
        return None;
    }

    // Time-based cooldown (skip on first invocation)
    if last_oracle_gen > 0 && last_oracle_time.elapsed().as_secs() < config.cooldown_seconds {
        return None;
    }

    // Primary trigger: L0→L1 promotion rate drop
    if let Some(rate) = promo_tracker.avg_rate(0) {
        if rate < config.promotion_rate_threshold {
            return Some("promotion_rate");
        }
    }

    // Secondary trigger: high too_few_trades rate
    if gen >= config.tft_min_generation && tft_tracker.rate() > config.tft_rate_threshold {
        return Some("tft_rate");
    }

    None
}

/// Fetch top-performing formulas from other symbols (same exchange + mode)
/// for cross-symbol learning context in the LLM oracle prompt.
///
/// Returns empty vec on query failure (non-blocking — oracle still works).
async fn fetch_cross_symbol_elites(
    pool: &PgPool,
    exchange: &str,
    symbol: &str,
    mode_str: &str,
    feat_offset: usize,
    factor_names: &[String],
    limit: i64,
) -> Vec<llm_oracle::CrossSymbolElite> {
    let rows = match sqlx::query(
        "SELECT sg.symbol, sg.fitness, sg.best_genome, \
                (sg.metadata->>'oos_psr')::float AS oos_psr \
         FROM strategy_generations sg \
         WHERE sg.exchange = $1 \
           AND sg.mode = $2 \
           AND sg.symbol != $3 \
           AND sg.generation = ( \
               SELECT MAX(sg2.generation) \
               FROM strategy_generations sg2 \
               WHERE sg2.exchange = sg.exchange \
                 AND sg2.symbol = sg.symbol \
                 AND sg2.mode = sg.mode \
           ) \
           AND (sg.metadata->>'oos_psr')::float > 0 \
         ORDER BY (sg.metadata->>'oos_psr')::float DESC \
         LIMIT $4",
    )
    .bind(exchange)
    .bind(mode_str)
    .bind(symbol)
    .bind(limit)
    .fetch_all(pool)
    .await
    {
        Ok(rows) => rows,
        Err(e) => {
            warn!(
                "[{}:{}:{}] Cross-symbol elite query failed: {}",
                exchange, symbol, mode_str, e
            );
            return Vec::new();
        }
    };

    use sqlx::Row;
    let mut elites = Vec::new();
    for row in &rows {
        let other_symbol: String = row.get("symbol");
        let fitness: f64 = row.get("fitness");
        let oos_psr: f64 = row.get("oos_psr");

        let best_genome: Option<Vec<i32>> = row.try_get("best_genome").ok();
        let tokens = match best_genome {
            Some(ref genome) => {
                if genome.iter().any(|&x| x < 0) {
                    warn!(
                        "[{}] Skipping cross-symbol elite with negative token values",
                        other_symbol
                    );
                    continue;
                }
                genome.iter().map(|&x| x as usize).collect::<Vec<usize>>()
            }
            None => continue,
        };

        let formula = genome_decoder::decode_genome(&tokens, feat_offset, factor_names);
        elites.push(llm_oracle::CrossSymbolElite {
            symbol: other_symbol,
            formula,
            is_psr: fitness,
            oos_psr,
        });
    }

    elites
}

// ── P5: Ensemble Rebalance Loop ──────────────────────────────────────────

use backtest::ensemble::{self, EnsembleConfig, StrategyCandidate};
use backtest::ensemble_weights::{self, DynamicWeightConfig};
use backtest::hrp;

/// Periodic ensemble rebalance loop for one exchange.
///
/// P6b-F4: Daily ensemble walk-forward backtest scheduler.
///
/// Runs at 03:00 UTC each day. Computes aggregate performance statistics
/// from historical ensemble equity data and persists to ensemble_backtest_results.
async fn run_daily_backtest_loop(pool: PgPool, exchange: &str) {
    info!("[{}] Starting daily ensemble backtest scheduler", exchange);

    // Wait 5 minutes after startup (let ensemble run first)
    tokio::time::sleep(Duration::from_secs(300)).await;

    loop {
        // Compute sleep until next 03:00 UTC
        let now = chrono::Utc::now();
        let today_3am = now.date_naive().and_hms_opt(3, 0, 0).unwrap().and_utc();
        let next_run = if now < today_3am {
            today_3am
        } else {
            today_3am + chrono::Duration::days(1)
        };
        let sleep_secs = (next_run - now).num_seconds().max(0) as u64;
        info!(
            "[{}] Next ensemble backtest at {} (in {}h {}m)",
            exchange,
            next_run.format("%Y-%m-%d %H:%M UTC"),
            sleep_secs / 3600,
            (sleep_secs % 3600) / 60,
        );
        tokio::time::sleep(Duration::from_secs(sleep_secs)).await;

        // Run backtest
        match ensemble::run_ensemble_walk_forward(&pool, exchange).await {
            Ok(Some(result)) => {
                info!(
                    "[{}] Ensemble backtest: {} rebalances, return={:.2}%, sharpe={:.3}, maxDD={:.2}%",
                    exchange,
                    result.rebalance_count,
                    result.cumulative_return * 100.0,
                    result.annualized_sharpe,
                    result.max_drawdown * 100.0,
                );
                if let Err(e) = ensemble::persist_backtest_result(&pool, &result).await {
                    error!("[{}] Failed to persist backtest result: {}", exchange, e);
                }
            }
            Ok(None) => {
                info!("[{}] Insufficient ensemble history for backtest", exchange);
            }
            Err(e) => {
                error!("[{}] Ensemble backtest failed: {}", exchange, e);
            }
        }
    }
}

/// Loads top strategies, computes HRP weights with dynamic adjustments,
/// persists to DB, and publishes to Redis.
#[allow(clippy::too_many_arguments)]
async fn run_ensemble_loop(
    pool: PgPool,
    redis_url: &str,
    exchange: &str,
    resolution: &str,
    factor_config_path: &str,
    lookback_days: i64,
    threshold_config: ThresholdConfig,
    ensemble_cfg: EnsembleConfig,
    lfdr_config: backtest::hypothesis::LfdrConfig,
) -> anyhow::Result<()> {
    info!(
        "[{}] Starting ensemble rebalance loop (interval={}min)",
        exchange, ensemble_cfg.rebalance_interval_minutes
    );

    let default_interval = Duration::from_secs(ensemble_cfg.rebalance_interval_minutes * 60);
    let dw_config = DynamicWeightConfig::from_yaml(&ensemble_cfg.dynamic_weights);
    let factor_config = load_factor_config(factor_config_path);

    // Wait for evolution to produce initial generations before first rebalance
    tokio::time::sleep(Duration::from_secs(120)).await;

    let mut version = 0_i32;

    // Determine next version from DB
    let row = sqlx::query(
        "SELECT COALESCE(MAX(version), 0) as max_version FROM portfolio_ensembles WHERE exchange = $1",
    )
    .bind(exchange)
    .fetch_optional(&pool)
    .await;
    if let Ok(Some(r)) = row {
        version = sqlx::Row::get::<i32, _>(&r, "max_version");
    }

    // P6b-F3: Track previous regime for transition detection
    let mut prev_regime: Option<ensemble::VolRegime> = None;

    loop {
        let regime_info = match run_ensemble_rebalance(
            &pool,
            redis_url,
            exchange,
            resolution,
            lookback_days,
            &threshold_config,
            &ensemble_cfg,
            &dw_config,
            &factor_config,
            &lfdr_config,
            &mut version,
        )
        .await
        {
            Ok(info) => info,
            Err(e) => {
                error!("[{}] Ensemble rebalance error: {}", exchange, e);
                None
            }
        };

        // P6b-F3: Dynamic interval based on volatility regime
        let sleep_dur = if ensemble_cfg.regime_aware {
            if let Some(info) = regime_info {
                let interval_mins = match info.regime {
                    ensemble::VolRegime::Low => ensemble_cfg.regime_intervals[0],
                    ensemble::VolRegime::Normal => ensemble_cfg.regime_intervals[1],
                    ensemble::VolRegime::High => ensemble_cfg.regime_intervals[2],
                };

                // Detect regime transition
                if let Some(prev) = prev_regime {
                    if prev != info.regime {
                        info!(
                            "[{}] Regime transition: {} → {} (vol={:.1}%, interval={}min)",
                            exchange,
                            prev,
                            info.regime,
                            info.annualized_vol * 100.0,
                            interval_mins
                        );
                        // If volatility escalated, trigger immediate rebalance
                        if info.regime > prev {
                            prev_regime = Some(info.regime);
                            continue;
                        }
                    }
                }

                prev_regime = Some(info.regime);
                Duration::from_secs(interval_mins * 60)
            } else {
                default_interval
            }
        } else {
            default_interval
        };

        tokio::time::sleep(sleep_dur).await;
    }
}

/// Execute a single ensemble rebalance cycle.
///
/// Returns `Some(RegimeInfo)` on successful rebalance with regime detection,
/// or `None` if the rebalance was skipped (no candidates, insufficient data, etc.).
#[allow(clippy::too_many_arguments)]
async fn run_ensemble_rebalance(
    pool: &PgPool,
    redis_url: &str,
    exchange: &str,
    resolution: &str,
    lookback_days: i64,
    threshold_config: &ThresholdConfig,
    ensemble_cfg: &EnsembleConfig,
    dw_config: &DynamicWeightConfig,
    factor_config: &backtest_engine::config::FactorConfig,
    lfdr_config: &backtest::hypothesis::LfdrConfig,
    version: &mut i32,
) -> anyhow::Result<Option<ensemble::RegimeInfo>> {
    // 1. Load candidates
    let candidates = ensemble::load_candidates_from_db(pool, exchange).await?;
    if candidates.is_empty() {
        info!(
            "[{}] No strategy candidates found, skipping rebalance",
            exchange
        );
        return Ok(None);
    }

    // 2. Select eligible strategies
    let selected = ensemble::select_candidates_with_lfdr(candidates, ensemble_cfg, lfdr_config);
    if selected.is_empty() {
        info!(
            "[{}] No candidates meet ensemble thresholds, skipping rebalance",
            exchange
        );
        return Ok(None);
    }

    let n = selected.len();
    info!(
        "[{}] Ensemble rebalance: {} strategies selected",
        exchange, n
    );

    // 3. Load market data and extract returns for each selected strategy
    let mut backtester = backtest::Backtester::with_threshold_config(
        pool.clone(),
        factor_config.clone(),
        exchange.to_string(),
        resolution.to_string(),
        threshold_config.clone(),
    );

    let symbols: Vec<String> = selected.iter().map(|c| c.id.symbol.clone()).collect();
    backtester.load_data(&symbols, lookback_days).await?;

    let mut return_series: Vec<Vec<f64>> = Vec::with_capacity(n);
    let mut valid_candidates: Vec<&StrategyCandidate> = Vec::with_capacity(n);

    for candidate in &selected {
        let cache = match backtester.cache.get(&candidate.id.symbol) {
            Some(c) => c,
            None => {
                warn!(
                    "[{}] No cached data for {}, skipping",
                    exchange, candidate.id.symbol
                );
                continue;
            }
        };

        let mode: StrategyMode = candidate.id.mode.parse().unwrap_or(StrategyMode::LongOnly);

        match ensemble::extract_strategy_returns(
            &backtester.vm,
            &candidate.genome,
            cache,
            mode,
            &backtester.threshold_config,
            &candidate.id.symbol,
            exchange,
            ensemble_cfg.correlation_lookback_bars,
        ) {
            Some(returns) => {
                return_series.push(returns);
                valid_candidates.push(candidate);
            }
            None => {
                warn!(
                    "[{}] Failed to extract returns for {}, skipping",
                    exchange, candidate.id
                );
            }
        }
    }

    if valid_candidates.len() < 2 {
        if valid_candidates.len() == 1 {
            info!(
                "[{}] Only 1 valid strategy — trivial 100% allocation",
                exchange
            );
        } else {
            info!(
                "[{}] No valid strategies with returns, skipping rebalance",
                exchange
            );
            return Ok(None);
        }
    }

    // 4. Build T x N return matrix (align to shortest length)
    let min_len = return_series.iter().map(|r| r.len()).min().unwrap_or(0);
    if min_len < 30 {
        warn!(
            "[{}] Return series too short ({} bars), skipping",
            exchange, min_len
        );
        return Ok(None);
    }

    let n_valid = valid_candidates.len();
    let mut return_matrix = ndarray::Array2::<f64>::zeros((min_len, n_valid));
    for (j, series) in return_series.iter().enumerate() {
        let offset = series.len() - min_len;
        for i in 0..min_len {
            return_matrix[[i, j]] = series[offset + i];
        }
    }

    // 5. Run HRP allocation
    // P6a-F1: Use configurable covariance method (sample or EWMA)
    let hrp_result =
        match hrp::allocate_hrp_with_method(&return_matrix, ensemble_cfg.covariance_method) {
            Some(r) => r,
            None => {
                warn!("[{}] HRP allocation failed", exchange);
                return Ok(None);
            }
        };

    // 6. Apply dynamic weight adjustments
    let valid_owned: Vec<StrategyCandidate> =
        valid_candidates.iter().map(|c| (*c).clone()).collect();
    let mut adjustments = ensemble_weights::adjust_weights(
        &hrp_result.weights,
        &valid_owned,
        &hrp_result.correlation_matrix,
        dw_config,
    );

    // 6b. Load previous weights from DB (shared by deadzone + turnover)
    let prev_weights: Vec<(String, f64)> = sqlx::query_as::<_, (String, String, f64)>(
        "SELECT symbol, mode, final_weight FROM portfolio_ensemble_strategies pes \
         JOIN portfolio_ensembles pe ON pes.ensemble_id = pe.id \
         WHERE pe.exchange = $1 \
         ORDER BY pe.created_at DESC LIMIT 50",
    )
    .bind(exchange)
    .fetch_all(pool)
    .await
    .unwrap_or_default()
    .into_iter()
    .map(|(sym, mode, w)| (format!("{}_{}", sym, mode), w))
    .collect();

    // 6c. P6b-C1: Apply deadzone + L1 regularization
    if ensemble_cfg.deadzone.enabled && !prev_weights.is_empty() {
        // Build new weight vector keyed by "symbol_mode"
        let mut new_weights: Vec<(String, f64)> = adjustments
            .iter()
            .enumerate()
            .map(|(i, adj)| {
                let key = format!(
                    "{}_{}",
                    valid_candidates[i].id.symbol, valid_candidates[i].id.mode
                );
                (key, adj.final_weight)
            })
            .collect();

        let regime_multiplier = if ensemble_cfg.deadzone.vol_adaptive {
            // Preliminary regime detection for adaptive lambda
            let preliminary_returns: Vec<f64> = (0..min_len)
                .map(|i| {
                    adjustments
                        .iter()
                        .enumerate()
                        .map(|(j, adj)| return_matrix[[i, j]] * adj.final_weight)
                        .sum::<f64>()
                })
                .collect();
            let info = ensemble::detect_regime(
                &preliminary_returns,
                resolution,
                20,
                ensemble_cfg.regime_thresholds,
            );
            info.regime.deadzone_multiplier()
        } else {
            1.0
        };

        ensemble_weights::apply_deadzone_l1(
            &prev_weights,
            &mut new_weights,
            ensemble_cfg.deadzone.threshold,
            ensemble_cfg.deadzone.l1_lambda,
            regime_multiplier,
        );

        // Update adjustments with deadzoned weights
        for (i, (_, weight)) in new_weights.iter().enumerate() {
            adjustments[i].final_weight = *weight;
        }
    }

    // 7. Compute portfolio-level metrics
    let crowding_pairs = ensemble_weights::detect_crowding(
        &hrp_result.correlation_matrix,
        dw_config.crowding_corr_threshold,
    );

    // Average pairwise correlation
    let avg_corr = if n_valid > 1 {
        let mut sum = 0.0_f64;
        let mut count = 0;
        for i in 0..n_valid {
            for j in (i + 1)..n_valid {
                sum += hrp_result.correlation_matrix[[i, j]];
                count += 1;
            }
        }
        if count > 0 {
            sum / count as f64
        } else {
            0.0
        }
    } else {
        0.0
    };

    // Portfolio-level metrics from weighted returns
    let mut portfolio_returns = vec![0.0_f64; min_len];
    for i in 0..min_len {
        for (j, adj) in adjustments.iter().enumerate() {
            portfolio_returns[i] += return_matrix[[i, j]] * adj.final_weight;
        }
    }

    let port_mean = portfolio_returns.iter().sum::<f64>() / min_len as f64;
    let port_var = portfolio_returns
        .iter()
        .map(|r| (r - port_mean).powi(2))
        .sum::<f64>()
        / (min_len as f64 - 1.0);
    let port_std = port_var.sqrt();
    let port_sharpe = if port_std > 1e-10 {
        port_mean / port_std
    } else {
        0.0
    };

    // Max drawdown
    let mut equity = 1.0_f64;
    let mut peak = 1.0_f64;
    let mut max_dd = 0.0_f64;
    for &r in &portfolio_returns {
        equity *= 1.0 + r;
        if equity > peak {
            peak = equity;
        }
        let dd = (peak - equity) / peak;
        if dd > max_dd {
            max_dd = dd;
        }
    }

    // 7b. P6b-F3: Detect volatility regime
    let regime_info = ensemble::detect_regime(
        &portfolio_returns,
        resolution,
        20,
        ensemble_cfg.regime_thresholds,
    );

    // 7c. P6a-F2: Compute turnover vs previous rebalance weights
    let new_weights: Vec<(String, f64)> = adjustments
        .iter()
        .enumerate()
        .map(|(i, adj)| {
            let key = format!(
                "{}_{}",
                valid_candidates[i].id.symbol, valid_candidates[i].id.mode
            );
            (key, adj.final_weight)
        })
        .collect();

    // prev_weights loaded in step 6b (shared with deadzone)
    let turnover = ensemble_weights::compute_turnover(&prev_weights, &new_weights);
    let turnover_cost_val =
        ensemble_weights::turnover_cost(turnover, ensemble_cfg.turnover_cost_rate);

    // Deduct from shadow equity
    equity *= 1.0 - turnover_cost_val;

    // 8. Persist to DB
    *version += 1;
    let weights_json: serde_json::Value = adjustments
        .iter()
        .enumerate()
        .map(|(i, adj)| {
            serde_json::json!({
                "symbol": valid_candidates[i].id.symbol,
                "mode": valid_candidates[i].id.mode,
                "weight": adj.final_weight,
            })
        })
        .collect();

    let metadata = serde_json::json!({
        "turnover": turnover,
        "turnover_cost": turnover_cost_val,
        "turnover_cost_rate": ensemble_cfg.turnover_cost_rate,
        "covariance_method": format!("{:?}", ensemble_cfg.covariance_method),
        "regime": format!("{}", regime_info.regime),
        "annualized_vol": regime_info.annualized_vol,
    });

    let hrp_diagnostics = serde_json::json!({
        "leaf_order": hrp_result.leaf_order,
        "linkage_steps": hrp_result.linkage.len(),
        "return_bars": min_len,
    });

    let ensemble_id: uuid::Uuid = sqlx::query_scalar(
        "INSERT INTO portfolio_ensembles \
         (exchange, version, strategy_count, portfolio_oos_psr, portfolio_sharpe, \
          portfolio_max_drawdown, avg_pairwise_correlation, crowded_pair_count, \
          weights, hrp_diagnostics, metadata) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) \
         RETURNING id",
    )
    .bind(exchange)
    .bind(*version)
    .bind(n_valid as i32)
    .bind(port_sharpe) // Using Sharpe as proxy for portfolio PSR
    .bind(port_sharpe)
    .bind(max_dd)
    .bind(avg_corr)
    .bind(crowding_pairs.len() as i32)
    .bind(&weights_json)
    .bind(&hrp_diagnostics)
    .bind(&metadata)
    .fetch_one(pool)
    .await?;

    // Per-strategy detail rows
    for (i, adj) in adjustments.iter().enumerate() {
        let c = &valid_candidates[i];
        let strategy_id = format!(
            "{}_{}_gen{}",
            exchange.to_lowercase(),
            c.id.symbol.to_lowercase(),
            c.id.generation
        );

        let _ = sqlx::query(
            "INSERT INTO portfolio_ensemble_strategies \
             (ensemble_id, exchange, symbol, mode, generation, strategy_id, \
              hrp_weight, psr_factor, utilization_factor, crowding_penalty, final_weight, \
              oos_psr, is_fitness, utilization, genome) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)",
        )
        .bind(ensemble_id)
        .bind(exchange)
        .bind(&c.id.symbol)
        .bind(&c.id.mode)
        .bind(c.id.generation)
        .bind(&strategy_id)
        .bind(adj.hrp_weight)
        .bind(adj.psr_factor)
        .bind(adj.utilization_factor)
        .bind(adj.crowding_penalty)
        .bind(adj.final_weight)
        .bind(c.oos_psr)
        .bind(c.is_fitness)
        .bind(c.utilization)
        .bind(&c.genome)
        .execute(pool)
        .await
        .map_err(|e| {
            error!(
                "[{}] Failed to persist ensemble strategy {}: {}",
                exchange, c.id, e
            )
        });
    }

    // Shadow equity point
    let _ = sqlx::query(
        "INSERT INTO portfolio_ensemble_equity \
         (exchange, ensemble_version, timestamp, equity, period_return) \
         VALUES ($1, $2, NOW(), $3, $4) \
         ON CONFLICT (exchange, ensemble_version, timestamp) DO NOTHING",
    )
    .bind(exchange)
    .bind(*version)
    .bind(equity)
    .bind(port_mean * min_len as f64)
    .execute(pool)
    .await
    .map_err(|e| error!("[{}] Failed to persist equity point: {}", exchange, e));

    // 9. UPSERT deployed strategies (P6b-B1)
    let threshold_json = serde_json::to_value(threshold_config).unwrap_or_default();
    if let Err(e) = ensemble::upsert_deployed_strategies(
        pool,
        exchange,
        &valid_owned,
        &adjustments,
        *version,
        &threshold_json,
    )
    .await
    {
        error!("[{}] Failed to upsert deployed strategies: {}", exchange, e);
    }

    // 10. Publish to Redis
    if let Ok(client) = redis::Client::open(redis_url) {
        if let Ok(mut conn) = client.get_multiplexed_async_connection().await {
            let channel = format!("portfolio_ensemble:{}", exchange.to_lowercase());
            let payload = serde_json::json!({
                "exchange": exchange,
                "version": version,
                "strategy_count": n_valid,
                "sharpe": port_sharpe,
                "max_drawdown": max_dd,
                "avg_correlation": avg_corr,
                "crowded_pairs": crowding_pairs.len(),
                "weights": weights_json,
            });
            let _: Result<(), _> = conn.publish(&channel, payload.to_string()).await;
        }
    }

    info!(
        "[{}] Ensemble v{}: {} strategies, Sharpe={:.3}, MaxDD={:.3}, AvgCorr={:.3}, Crowded={}, Turnover={:.4}, Cost={:.6}, Regime={} (vol={:.1}%)",
        exchange,
        version,
        n_valid,
        port_sharpe,
        max_dd,
        avg_corr,
        crowding_pairs.len(),
        turnover,
        turnover_cost_val,
        regime_info.regime,
        regime_info.annualized_vol * 100.0,
    );

    Ok(Some(regime_info))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcts_config_deserialization_defaults() {
        // Verify MctsYamlConfig defaults match documentation
        let config = MctsYamlConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.budget, 1000);
        assert_eq!(config.seeds_per_round, 5);
        assert_eq!(config.interval, 50);
        assert!((config.exploration_c - 1.414).abs() < 1e-3);
        assert_eq!(config.max_length, 20);
        assert!(config.use_max_reward, "Extreme Bandit PUCT should default to true");
    }

    #[test]
    fn test_mcts_config_from_yaml() {
        let yaml = r#"
            enabled: true
            budget: 500
            seeds_per_round: 3
            use_max_reward: false
        "#;
        let config: MctsYamlConfig = serde_yaml::from_str(yaml).unwrap();
        assert!(config.enabled);
        assert_eq!(config.budget, 500);
        assert_eq!(config.seeds_per_round, 3);
        assert!(!config.use_max_reward);
        // Defaults for unspecified fields
        assert_eq!(config.interval, 50);
        assert_eq!(config.max_length, 20);
    }
}
