use redis::AsyncCommands;
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::time::Duration;
use tracing::{error, info, warn};

mod api;
mod backtest;
mod genetic;

use backtest::Backtester;
use backtest_engine::config::FactorConfig;
use genetic::GeneticAlgorithm;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Initialize Logging
    tracing_subscriber::fmt::init();
    info!("Starting Strategy Generator (Evolutionary Optimizer)...");

    // Load exchange/resolution config
    let exchange = env::var("GENERATOR_EXCHANGE").unwrap_or_else(|_| "Birdeye".to_string());
    let resolution = env::var("GENERATOR_RESOLUTION").unwrap_or_else(|_| "15m".to_string());

    // Configurable ports for multi-instance support
    let health_port: u16 = env::var("GENERATOR_HEALTH_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8084);
    let api_port: u16 = env::var("GENERATOR_API_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8082);

    let service_name = format!("strategy-generator-{}", exchange.to_lowercase());
    info!(
        "Asset mode: exchange={}, resolution={}, api_port={}, health_port={}",
        exchange, resolution, api_port, health_port
    );

    // Spawn health check server
    let health_name = service_name.clone();
    tokio::spawn(async move {
        common::health::start_health_server(&health_name, health_port).await;
    });

    // 2. Connect to Infrastructure
    let db_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:hermesflow@localhost:5432/hermesflow".to_string());
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    // Load Factor Config
    let config_path =
        env::var("FACTOR_CONFIG").unwrap_or_else(|_| "config/factors.yaml".to_string());
    info!("Loading factor configuration from: {}", config_path);

    let factor_config = match FactorConfig::from_file(&config_path) {
        Ok(cfg) => {
            info!("Loaded {} active factors", cfg.active_factors.len());
            cfg
        }
        Err(e) => {
            warn!(
                "Failed to load factor config: {}. Falling back to default AlphaGPT 6-factor mode.",
                e
            );
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
    };

    let feat_offset = factor_config.feat_offset();
    info!("Feature offset (feat_count): {}", feat_offset);

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    let client = redis::Client::open(redis_url.clone())?;
    let mut redis_conn = client.get_async_connection().await?;

    // Start Heartbeat Task
    common::heartbeat::spawn_heartbeat(&service_name, &redis_url);

    // Exchange-namespaced Redis keys
    let exchange_lower = exchange.to_lowercase();
    let redis_key_status = format!("strategy:{}:status", exchange_lower);
    let redis_key_population = format!("strategy:{}:population", exchange_lower);
    let redis_channel = format!("strategy_updates:{}", exchange_lower);

    // 2.1 Spawn API Server
    let pool_api = pool.clone();
    let config_api = factor_config.clone();
    let exchange_api = exchange.clone();
    let resolution_api = resolution.clone();
    tokio::spawn(async move {
        api::start_api_server(pool_api, config_api, exchange_api, resolution_api, api_port).await;
    });

    // 3. Initialize Components
    let mut backtester = Backtester::new(
        pool.clone(),
        factor_config.clone(),
        exchange.clone(),
        resolution.clone(),
    );
    let mut ga = GeneticAlgorithm::new(100, feat_offset);

    // 4. Load Data — exchange-aware symbol loading
    use sqlx::Row;
    let mut symbols: Vec<String> = if exchange == "Polygon" {
        let rows = sqlx::query(
            "SELECT symbol FROM market_watchlist WHERE exchange = 'Polygon' AND is_active = true",
        )
        .fetch_all(&pool)
        .await
        .unwrap_or_default();
        rows.into_iter().map(|r| r.get("symbol")).collect()
    } else {
        let rows = sqlx::query("SELECT address FROM active_tokens WHERE is_active = true")
            .fetch_all(&pool)
            .await
            .unwrap_or_default();
        rows.into_iter().map(|r| r.get("address")).collect()
    };

    if symbols.is_empty() {
        if exchange == "Polygon" {
            warn!("No active stocks found in DB. Falling back to default list.");
            symbols = vec![
                "AAPL".to_string(),
                "NVDA".to_string(),
                "MSFT".to_string(),
                "GOOGL".to_string(),
            ];
        } else {
            warn!("No active tokens found in DB. Falling back to default list.");
            symbols = vec![
                "So11111111111111111111111111111111111111112".to_string(),
                "JUPyiwrYJFskUPiHa7hkeR8VUtkPHCLkdP9KcJQUE85".to_string(),
                "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263".to_string(),
                "EKpQGSJmxSWojVRHgWN2EWH18dPBfJCs8J6QW2K2pump".to_string(),
            ];
        }
    } else {
        info!(
            "Fetched {} active symbols from DB: {:?}",
            symbols.len(),
            symbols
        );
    }

    // Configurable lookback
    let lookback_days: i64 = env::var("GENERATOR_LOOKBACK_DAYS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(if exchange == "Polygon" { 365 } else { 7 });

    info!(
        "Loading historical data for {} symbols ({} days lookback)...",
        symbols.len(),
        lookback_days
    );
    if let Err(e) = backtester.load_data(&symbols, lookback_days).await {
        error!("Failed to load historical data: {}", e);
        warn!("Backtester will have no data - all fitness will be 0.0");
    }

    // 4.1 Load Strategy State — filtered by exchange
    let last_gen_row = sqlx::query(
        "SELECT generation, best_genome FROM strategy_generations WHERE exchange = $1 ORDER BY generation DESC LIMIT 1",
    )
    .bind(&exchange)
    .fetch_optional(&pool)
    .await;

    if let Ok(Some(row)) = last_gen_row {
        if let Ok(max_gen) = row.try_get::<i32, _>("generation") {
            info!("Resuming evolution from generation {}", max_gen);
            ga.generation = max_gen as usize + 1;
        }

        if let Ok(best_tokens) = row.try_get::<Vec<i32>, _>("best_genome") {
            info!("Restoring best genome from DB: {:?}", best_tokens);
            if !ga.population.is_empty() {
                ga.population[0].tokens = best_tokens.into_iter().map(|x| x as usize).collect();
            }
        }
    }

    // 5. Evolution Loop
    loop {
        let gen = ga.generation;
        info!("Running Generation {}...", gen);

        // Evaluate Fitness
        for genome in ga.population.iter_mut() {
            backtester.evaluate(genome);
        }

        // Log Best
        ga.evolve();

        if let Some(best) = &ga.best_genome {
            // OOS IC for monitoring (not used in selection)
            let oos_ic = backtester.evaluate_oos(best);

            info!(
                "Generation {} Best Fitness: {:.4} (OOS IC: {:.4}, Tokens: {:?})",
                gen, best.fitness, oos_ic, best.tokens
            );

            // Payload for Redis/Frontend
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
                    "description": format!("Evolved Strategy. Sharpe: {:.2}", best.fitness)
                }
            });

            let payload_str = payload.to_string();

            // 1. Publish to PubSub (Realtime) — exchange-namespaced channel
            let _: () = redis_conn
                .publish(&redis_channel, &payload_str)
                .await
                .unwrap_or_else(|e| {
                    error!("Failed to publish strategy: {}", e);
                });

            // 2. Persist to Redis — exchange-namespaced keys
            let _: () = redis_conn
                .set(&redis_key_status, &payload_str)
                .await
                .unwrap_or(());

            // 2.1 Publish Full Population (Leaderboard / Pool)
            let population_view = ga
                .population
                .iter()
                .map(|g| {
                    serde_json::json!({
                        "fitness": g.fitness,
                        "tokens": g.tokens,
                        "generation": gen
                    })
                })
                .collect::<Vec<_>>();

            let pop_payload = serde_json::to_string(&population_view).unwrap_or_default();
            let _: () = redis_conn
                .set(&redis_key_population, &pop_payload)
                .await
                .unwrap_or(());

            // 3. Persist to DB — with exchange column
            let tokens_i32: Vec<i32> = best.tokens.iter().map(|&x| x as i32).collect();
            let strategy_id = format!("{}_alphagpt_gen_{}", exchange_lower, gen);
            let _ = sqlx::query(
                "INSERT INTO strategy_generations (exchange, generation, fitness, best_genome, metadata, strategy_id) VALUES ($1, $2, $3, $4, $5, $6) ON CONFLICT (exchange, generation) DO UPDATE SET fitness = $3, best_genome = $4, metadata = $5, strategy_id = $6"
            )
            .bind(&exchange)
            .bind(gen as i32)
            .bind(best.fitness)
            .bind(&tokens_i32)
            .bind(&payload)
            .bind(&strategy_id)
            .execute(&pool)
            .await
            .map_err(|e| error!("Failed to persist generation: {}", e));

            // 4. Persistence: Run Detailed Backtest & Save to backtest_results
            if gen.is_multiple_of(5) {
                let tokens_i32: Vec<i32> = best.tokens.iter().map(|&x| x as i32).collect();

                match backtester
                    .run_portfolio_simulation(&tokens_i32, lookback_days)
                    .await
                {
                    Ok(sim_result) => {
                        let metrics = sim_result["metrics"].as_object().unwrap();
                        let total_ret = metrics["total_return"].as_f64().unwrap_or(0.0);
                        let win_rate = metrics["win_rate"].as_f64().unwrap_or(0.0);
                        let trades_count = metrics["total_trades"].as_i64().unwrap_or(0);
                        let sharpe = metrics
                            .get("sharpe_ratio")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.0);
                        let drawdown = metrics
                            .get("max_drawdown")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.0);
                        let equity = &sim_result["equity_curve"];

                        info!(
                            "Portfolio simulation completed: PnL={:.4}%, Sharpe={:.2}, Trades={}",
                            total_ret * 100.0,
                            sharpe,
                            trades_count
                        );

                        let _ = sqlx::query(
                            r#"
                               INSERT INTO backtest_results (
                                   strategy_id, genome, token_address,
                                   pnl_percent, win_rate, total_trades,
                                   sharpe_ratio, max_drawdown,
                                   equity_curve, created_at
                               ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW())
                               "#,
                        )
                        .bind(&strategy_id)
                        .bind(&tokens_i32)
                        .bind(sim_result["symbol"].as_str().unwrap_or("UNIVERSAL"))
                        .bind(total_ret)
                        .bind(win_rate)
                        .bind(trades_count as i32)
                        .bind(sharpe)
                        .bind(drawdown)
                        .bind(equity)
                        .execute(&pool)
                        .await
                        .map_err(|e| error!("Failed to insert backtest result: {}", e));
                    }
                    Err(e) => {
                        error!("Detailed simulation failed for persistence: {}", e);
                    }
                }
            }

            // Cleanup old generations (Keep last 1000 per exchange)
            if gen.is_multiple_of(10) && gen > 1000 {
                let cutoff = gen as i32 - 1000;
                let _ = sqlx::query(
                    "DELETE FROM strategy_generations WHERE exchange = $1 AND generation < $2",
                )
                .bind(&exchange)
                .bind(cutoff)
                .execute(&pool)
                .await
                .map_err(|e| error!("Failed to cleanup old generations: {}", e));
            }
        }

        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}
