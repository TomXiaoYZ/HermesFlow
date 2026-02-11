use redis::AsyncCommands;
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::time::Duration;
use tracing::{error, info, warn};

mod api;
mod backtest;
mod genetic;

use backtest_engine::config::FactorConfig; // Standard one for portfolio sim? No, wait.
                                           // Backtester here is the local one.
use backtest::Backtester;
use genetic::GeneticAlgorithm;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Initialize Logging
    tracing_subscriber::fmt::init();
    info!("Starting Strategy Generator (Evolutionary Optimizer)...");

    // Spawn health check server
    tokio::spawn(common::health::start_health_server(
        "strategy-generator",
        8084,
    ));

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
            // Fallback to default 6 factors
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

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    let client = redis::Client::open(redis_url.clone())?;
    let mut redis_conn = client.get_async_connection().await?;

    // Start Heartbeat Task
    common::heartbeat::spawn_heartbeat("strategy-generator", &redis_url);

    // 2.1 Spawn API Server
    let pool_api = pool.clone();
    let config_api = factor_config.clone();
    tokio::spawn(async move {
        api::start_api_server(pool_api, config_api).await;
    });

    // 3. Initialize Components
    let mut backtester = Backtester::new(pool.clone(), factor_config.clone());
    let mut ga = GeneticAlgorithm::new(100); // Population 100

    // 4. Load Data
    // Fetch top active symbols from DB
    use sqlx::Row;
    let rows = sqlx::query("SELECT address FROM active_tokens WHERE is_active = true")
        .fetch_all(&pool)
        .await
        .unwrap_or_default();

    let mut symbols: Vec<String> = rows.into_iter().map(|r| r.get("address")).collect();

    if symbols.is_empty() {
        warn!("No active tokens found in DB. Falling back to default list.");
        symbols = vec![
            "So11111111111111111111111111111111111111112".to_string(), // SOL
            "JUPyiwrYJFskUPiHa7hkeR8VUtkPHCLkdP9KcJQUE85".to_string(), // JUP
            "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263".to_string(), // BONK
            "EKpQGSJmxSWojVRHgWN2EWH18dPBfJCs8J6QW2K2pump".to_string(), // PENGU
        ];
    } else {
        info!(
            "Fetched {} active symbols from DB: {:?}",
            symbols.len(),
            symbols
        );
    }

    info!("Loading historical data for {} symbols...", symbols.len());
    if let Err(e) = backtester.load_data(&symbols, 7).await {
        error!("Failed to load historical data: {}", e);
        warn!("Backtester will have no data - all fitness will be 0.0");
    }

    // 4.1 Load Strategy State (Generation & Population)
    // Check DB for last generation
    let last_gen_row = sqlx::query(
        "SELECT generation, best_genome FROM strategy_generations ORDER BY generation DESC LIMIT 1",
    )
    .fetch_optional(&pool)
    .await;

    if let Ok(Some(row)) = last_gen_row {
        if let Ok(max_gen) = row.try_get::<i32, _>("generation") {
            info!("Resuming evolution from generation {}", max_gen);
            ga.generation = max_gen as usize + 1;
        }

        // Load the best genome to preserve logic!
        if let Ok(best_tokens) = row.try_get::<Vec<i32>, _>("best_genome") {
            info!("Restoring best genome from DB: {:?}", best_tokens);
            // Replace the first random genome with the saved best one
            // We need to cast i32 back to i32 (it matches).
            // Genome struct expects Vec<i32> usually.
            // Let's check Genome definition. Assuming it has `tokens: Vec<i32>`.
            // Modify GA to accept injection? Or just direct access.
            if !ga.population.is_empty() {
                ga.population[0].tokens = best_tokens.into_iter().map(|x| x as usize).collect();
                // Fitness will be re-evaluated in the loop
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
            info!(
                "Generation {} Best Fitness: {:.4} (Tokens: {:?})",
                gen, best.fitness, best.tokens
            );

            // Payload for Redis/Frontend
            let payload = serde_json::json!({
                "strategy_id": "alphagpt_evo_v2",
                "timestamp": chrono::Utc::now().timestamp(),
                "formula": best.tokens,
                "generation": gen, // Explicitly include generation for easier parsing
                "fitness": best.fitness,
                "best_tokens": best.tokens, // Include duplicate field for compatibility
                "meta": {
                    "name": format!("Evo-Gen{}-{:.2}", gen, best.fitness),
                    "description": format!("Evolved Strategy. Sharpe: {:.2}", best.fitness)
                }
            });

            let payload_str = payload.to_string();

            // 1. Publish to PubSub (Realtime)
            let _: () = redis_conn
                .publish("strategy_updates", &payload_str)
                .await
                .unwrap_or_else(|e| {
                    error!("Failed to publish strategy: {}", e);
                });

            // 2. Persist to Redis (History/Status API)
            let _: () = redis_conn
                .set("strategy:status", &payload_str)
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
                .set("strategy:population", &pop_payload)
                .await
                .unwrap_or(());

            // 3. Persist to DB (Permanent History)
            let tokens_i32: Vec<i32> = best.tokens.iter().map(|&x| x as i32).collect();
            let strategy_id = format!("alphagpt_gen_{}", gen);
            let _ = sqlx::query(
                "INSERT INTO strategy_generations (generation, fitness, best_genome, metadata, strategy_id) VALUES ($1, $2, $3, $4, $5) ON CONFLICT (generation) DO UPDATE SET strategy_id = $5"
            )
            .bind(gen as i32)
            .bind(best.fitness)
            .bind(&tokens_i32)
            .bind(&payload)
            .bind(&strategy_id)
            .execute(&pool)
            .await
            .map_err(|e| error!("Failed to persist generation: {}", e));

            // PubSub & Logging ...

            // 4. Persistence: Run Detailed Backtest & Save to backtest_results
            if gen.is_multiple_of(5) {
                // Universal Portfolio Simulation
                let tokens_i32: Vec<i32> = best.tokens.iter().map(|&x| x as i32).collect();

                match backtester.run_portfolio_simulation(&tokens_i32, 30).await {
                    Ok(sim_result) => {
                        // Extract metrics
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

                        // Insert
                        // handle genome array
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

            // Cleanup old generations (Keep last 1000)
            if gen.is_multiple_of(10) && gen > 1000 {
                let cutoff = gen as i32 - 1000;
                let _ = sqlx::query("DELETE FROM strategy_generations WHERE generation < $1")
                    .bind(cutoff)
                    .execute(&pool)
                    .await
                    .map_err(|e| error!("Failed to cleanup old generations: {}", e));
            }
        }

        // Sleep between generations to avoid pegging CPU in loop?
        // Evolution is CPU intensive. We should run continuously but maybe yield to runtime.
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}
