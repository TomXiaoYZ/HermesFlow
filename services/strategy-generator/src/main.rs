use redis::AsyncCommands;
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::time::Duration;
use tracing::{error, info, warn};

mod backtest;
mod genetic;

use backtest::Backtester;
use genetic::GeneticAlgorithm;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Initialize Logging
    tracing_subscriber::fmt::init();
    info!("Starting Strategy Generator (Evolutionary Optimizer)...");

    // 2. Connect to Infrastructure
    let db_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:hermesflow@localhost:5432/hermesflow".to_string());
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    let client = redis::Client::open(redis_url)?;
    let mut redis_conn = client.get_async_connection().await?;

    // 3. Initialize Components
    let mut backtester = Backtester::new(pool.clone());
    let mut ga = GeneticAlgorithm::new(100); // Population 100

    // 4. Load Data
    // Fetch top active symbols from DB
    use sqlx::Row;
    let rows = sqlx::query("SELECT symbol FROM active_tokens WHERE is_active = true")
        .fetch_all(&pool)
        .await
        .unwrap_or_default();

    let mut symbols: Vec<String> = rows.into_iter().map(|r| r.get("symbol")).collect();

    if symbols.is_empty() {
        warn!("No active tokens found in DB. Falling back to default list.");
        symbols = vec![
            "So11111111111111111111111111111111111111112".to_string(), // SOL
            "JUPyiwrYJFskUPiHa7hkeR8VUtkPHCLkdP9KcJQUE85".to_string(), // JUP
            "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263".to_string(), // BONK
            "EKpQGSJmxSWojVRHgWN2EWH18dPBfJCs8J6QW2K2pump".to_string(), // PENGU
        ];
    } else {
        info!("Fetched {} active symbols from DB", symbols.len());
    }

    info!("Loading historical data for {} symbols...", symbols.len());
    let _ = backtester.load_data(&symbols, 7).await; // Load 7 days

    // 4.1 Load Strategy State
    // Check DB for last generation
    let last_gen_row = sqlx::query("SELECT MAX(generation) as max_gen FROM strategy_generations")
        .fetch_one(&pool)
        .await;

    if let Ok(row) = last_gen_row {
        if let Some(max_gen) = row.try_get::<i32, _>("max_gen").ok() {
            info!("Resuming evolution from generation {}", max_gen);
            ga.generation = max_gen as usize + 1;
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
            let population_view = ga.population.iter().map(|g| {
                serde_json::json!({
                    "fitness": g.fitness,
                    "tokens": g.tokens,
                    "generation": gen
                })
            }).collect::<Vec<_>>();
            
            let pop_payload = serde_json::to_string(&population_view).unwrap_or_default();
            let _: () = redis_conn
                .set("strategy:population", &pop_payload)
                .await
                .unwrap_or(());
            
            // 3. Persist to DB (Permanent History)
            let tokens_i32: Vec<i32> = best.tokens.iter().map(|&x| x as i32).collect();
            let _ = sqlx::query(
                "INSERT INTO strategy_generations (generation, fitness, best_genome, metadata) VALUES ($1, $2, $3, $4) ON CONFLICT (generation) DO NOTHING"
            )
            .bind(gen as i32)
            .bind(best.fitness)
            .bind(&tokens_i32)
            .bind(&payload)
            .execute(&pool)
            .await
            .map_err(|e| error!("Failed to persist generation: {}", e));

            // Publish Log every generation for visibility
            if best.fitness > 0.0001 || gen % 1 == 0 {
                let log_payload = serde_json::json!({
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "strategy_id": "EvolutionaryKernel",
                    "symbol": "SYSTEM",
                    "action": if best.fitness > 0.0 { "Discovery" } else { "Evolving" },
                    "message": format!("Gen {}: Best Strategy Fitness: {:.4} (Evaluating...)", gen, best.fitness)
                });
                let log_str = log_payload.to_string();

                // 1. PubSub
                let _: () = redis_conn
                    .publish("strategy_logs", &log_str)
                    .await
                    .unwrap_or(());

                // 2. Persist List (History)
                let _: () = redis_conn
                    .lpush("system:logs", &log_str)
                    .await
                    .unwrap_or(());
                
                // Trim to 100
                let _: () = redis_conn
                    .ltrim("system:logs", 0, 99)
                    .await
                    .unwrap_or(());
            }
        }

        // Sleep between generations to avoid pegging CPU in loop?
        // Evolution is CPU intensive. We should run continuously but maybe yield to runtime.
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}
