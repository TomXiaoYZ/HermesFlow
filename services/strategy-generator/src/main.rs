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

use backtest::Backtester;
use backtest_engine::config::FactorConfig;
use genetic::GeneticAlgorithm;

/// Per-exchange evolution config, loaded from config/generator.yaml.
#[derive(Debug, Deserialize, Clone)]
pub struct ExchangeConfig {
    pub exchange: String,
    pub resolution: String,
    pub lookback_days: i64,
    pub factor_config: String,
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

    // Spawn one evolution task per exchange
    let mut handles = Vec::new();
    for ec in exchange_configs {
        let pool = pool.clone();
        let redis_url = redis_url.clone();
        let handle = tokio::spawn(async move {
            if let Err(e) = run_evolution(pool, &redis_url, ec).await {
                error!("Evolution loop failed: {}", e);
            }
        });
        handles.push(handle);
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
        },
        ExchangeConfig {
            exchange: "Polygon".to_string(),
            resolution: "1d".to_string(),
            lookback_days: 365,
            factor_config: "config/factors-stock.yaml".to_string(),
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

/// Run the evolution loop for a single exchange. Each call is a long-running task.
async fn run_evolution(
    pool: PgPool,
    redis_url: &str,
    config: ExchangeConfig,
) -> anyhow::Result<()> {
    let exchange = &config.exchange;
    let resolution = &config.resolution;
    let exchange_lower = exchange.to_lowercase();

    info!(
        "[{}] Starting evolution: resolution={}, lookback={}d, factors={}",
        exchange, resolution, config.lookback_days, config.factor_config
    );

    let factor_config = load_factor_config(&config.factor_config);
    let feat_offset = factor_config.feat_offset();
    info!("[{}] Feature offset: {}", exchange, feat_offset);

    let client = redis::Client::open(redis_url.to_string())?;
    let mut redis_conn = client.get_async_connection().await?;

    let redis_key_status = format!("strategy:{}:status", exchange_lower);
    let redis_key_population = format!("strategy:{}:population", exchange_lower);
    let redis_channel = format!("strategy_updates:{}", exchange_lower);

    let mut backtester = Backtester::new(
        pool.clone(),
        factor_config,
        exchange.clone(),
        resolution.clone(),
    );
    let pop_size = if exchange == "Polygon" { 200 } else { 100 };
    let mut ga = GeneticAlgorithm::new(pop_size, feat_offset);

    // Load symbols
    use sqlx::Row;
    let mut symbols: Vec<String> = if exchange == "Polygon" {
        sqlx::query(
            "SELECT symbol FROM market_watchlist WHERE exchange = 'Polygon' AND is_active = true",
        )
        .fetch_all(&pool)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|r| r.get("symbol"))
        .collect()
    } else {
        sqlx::query("SELECT address FROM active_tokens WHERE is_active = true")
            .fetch_all(&pool)
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

    info!(
        "[{}] Loading {} days of data for {} symbols...",
        exchange,
        config.lookback_days,
        symbols.len()
    );
    if let Err(e) = backtester.load_data(&symbols, config.lookback_days).await {
        error!("[{}] Failed to load data: {}", exchange, e);
    }

    // Resume from last generation
    let last_gen_row = sqlx::query(
        "SELECT generation, best_genome FROM strategy_generations WHERE exchange = $1 ORDER BY generation DESC LIMIT 1",
    )
    .bind(exchange)
    .fetch_optional(&pool)
    .await;

    if let Ok(Some(row)) = last_gen_row {
        if let Ok(max_gen) = row.try_get::<i32, _>("generation") {
            info!("[{}] Resuming from generation {}", exchange, max_gen);
            ga.generation = max_gen as usize + 1;
        }
        if let Ok(best_tokens) = row.try_get::<Vec<i32>, _>("best_genome") {
            if !ga.population.is_empty() {
                ga.population[0].tokens = best_tokens.into_iter().map(|x| x as usize).collect();
            }
        }
    }

    // Evolution loop
    loop {
        let gen = ga.generation;

        for genome in ga.population.iter_mut() {
            backtester.evaluate(genome);
        }
        ga.evolve();

        if let Some(best) = &ga.best_genome {
            let oos_ic = backtester.evaluate_oos(best);
            info!(
                "[{}] Gen {} Fitness: {:.4} OOS IC: {:.4}",
                exchange, gen, best.fitness, oos_ic
            );

            let payload = serde_json::json!({
                "strategy_id": format!("{}_alphagpt_evo_v2", exchange_lower),
                "timestamp": chrono::Utc::now().timestamp(),
                "formula": best.tokens,
                "generation": gen,
                "fitness": best.fitness,
                "oos_ic": oos_ic,
                "best_tokens": best.tokens,
                "exchange": exchange,
                "resolution": resolution,
                "meta": {
                    "name": format!("Evo-Gen{}-{:.2}", gen, best.fitness),
                    "description": format!("Evolved Strategy. IC: {:.4}", best.fitness)
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

            let population_view: Vec<_> = ga.population.iter().map(|g| {
                serde_json::json!({"fitness": g.fitness, "tokens": g.tokens, "generation": gen})
            }).collect();
            let pop_str = serde_json::to_string(&population_view).unwrap_or_default();
            let _: () = redis_conn
                .set(&redis_key_population, &pop_str)
                .await
                .unwrap_or(());

            // DB persist
            let tokens_i32: Vec<i32> = best.tokens.iter().map(|&x| x as i32).collect();
            let strategy_id = format!("{}_alphagpt_gen_{}", exchange_lower, gen);
            let _ = sqlx::query(
                "INSERT INTO strategy_generations (exchange, generation, fitness, best_genome, metadata, strategy_id) \
                 VALUES ($1, $2, $3, $4, $5, $6) \
                 ON CONFLICT (exchange, generation) DO UPDATE SET fitness = $3, best_genome = $4, metadata = $5, strategy_id = $6"
            )
            .bind(exchange)
            .bind(gen as i32)
            .bind(best.fitness)
            .bind(&tokens_i32)
            .bind(&payload)
            .bind(&strategy_id)
            .execute(&pool)
            .await
            .map_err(|e| error!("[{}] DB persist failed: {}", exchange, e));

            // Portfolio simulation every 5 generations
            if gen.is_multiple_of(5) {
                let tokens_i32: Vec<i32> = best.tokens.iter().map(|&x| x as i32).collect();
                match backtester
                    .run_portfolio_simulation(&tokens_i32, config.lookback_days)
                    .await
                {
                    Ok(sim) => {
                        let m = sim["metrics"].as_object().unwrap();
                        info!(
                            "[{}] Portfolio: PnL={:.2}%, Sharpe={:.2}",
                            exchange,
                            m["total_return"].as_f64().unwrap_or(0.0) * 100.0,
                            m.get("sharpe_ratio")
                                .and_then(|v| v.as_f64())
                                .unwrap_or(0.0)
                        );

                        let _ = sqlx::query(
                            "INSERT INTO backtest_results (strategy_id, genome, token_address, pnl_percent, win_rate, total_trades, sharpe_ratio, max_drawdown, equity_curve, created_at) \
                             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW())"
                        )
                        .bind(&strategy_id)
                        .bind(&tokens_i32)
                        .bind(sim["symbol"].as_str().unwrap_or("UNIVERSAL"))
                        .bind(m["total_return"].as_f64().unwrap_or(0.0))
                        .bind(m["win_rate"].as_f64().unwrap_or(0.0))
                        .bind(m["total_trades"].as_i64().unwrap_or(0) as i32)
                        .bind(m.get("sharpe_ratio").and_then(|v| v.as_f64()).unwrap_or(0.0))
                        .bind(m.get("max_drawdown").and_then(|v| v.as_f64()).unwrap_or(0.0))
                        .bind(&sim["equity_curve"])
                        .execute(&pool)
                        .await
                        .map_err(|e| error!("[{}] Backtest result persist failed: {}", exchange, e));
                    }
                    Err(e) => error!("[{}] Portfolio sim failed: {}", exchange, e),
                }
            }

            // Cleanup old generations
            if gen.is_multiple_of(10) && gen > 1000 {
                let _ = sqlx::query(
                    "DELETE FROM strategy_generations WHERE exchange = $1 AND generation < $2",
                )
                .bind(exchange)
                .bind(gen as i32 - 1000)
                .execute(&pool)
                .await;
            }
        }

        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}
