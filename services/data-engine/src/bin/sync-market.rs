// CLI tool for running market data sync
// Usage: cargo run --bin sync-market -- --limit 10

use chrono::Utc;
use clap::Parser;
use sqlx::{PgPool, Row};
use std::error::Error;
use tracing::{error, info, warn};

#[derive(Parser)]
#[command(name = "sync-market")]
#[command(about = "Sync market data from watchlist", long_about = None)]
struct Args {
    /// Number of tasks to process
    #[arg(short, long, default_value = "10")]
    limit: i32,

    /// Specific exchange filter
    #[arg(short, long)]
    exchange: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    // Load environment
    dotenvy::dotenv().ok();

    // Connect to database
    let database_url = std::env::var("TIMESCALE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .expect("TIMESCALE_URL or DATABASE_URL must be set");

    info!("Connecting to database: {}", database_url);
    let pool = PgPool::connect(&database_url).await?;

    // Initialize Polygon connector
    use data_engine::collectors::polygon::{PolygonConfig, PolygonConnector};

    let polygon_config = PolygonConfig::from_env()?;
    let polygon = PolygonConnector::new(polygon_config);

    info!("Starting market data sync (limit: {})", args.limit);

    // Get pending tasks
    let tasks = if let Some(exchange) = &args.exchange {
        sqlx::query(
            r#"
            SELECT
                s.exchange,
                s.symbol,
                s.resolution,
                COALESCE(w.sync_from_date, '2023-01-01'::DATE) as sync_from_date,
                COALESCE(w.priority, 50) as priority
            FROM market_sync_status s
            INNER JOIN market_watchlist w ON s.exchange = w.exchange AND s.symbol = w.symbol
            WHERE s.status = 'pending'
              AND s.exchange = $1
              AND w.is_active = true
            ORDER BY w.priority DESC
            LIMIT $2
            "#,
        )
        .bind(exchange)
        .bind(args.limit)
        .fetch_all(&pool)
        .await?
    } else {
        sqlx::query(
            r#"
            SELECT
                s.exchange,
                s.symbol,
                s.resolution,
                COALESCE(w.sync_from_date, '2023-01-01'::DATE) as sync_from_date,
                COALESCE(w.priority, 50) as priority
            FROM market_sync_status s
            INNER JOIN market_watchlist w ON s.exchange = w.exchange AND s.symbol = w.symbol
            WHERE s.status = 'pending'
              AND w.is_active = true
            ORDER BY w.priority DESC
            LIMIT $1
            "#,
        )
        .bind(args.limit)
        .fetch_all(&pool)
        .await?
    };

    info!("Found {} pending tasks", tasks.len());

    if tasks.is_empty() {
        info!("No pending tasks!");
        return Ok(());
    }

    // Process each task
    for (i, task) in tasks.iter().enumerate() {
        let exchange: String = task.get("exchange");
        let symbol: String = task.get("symbol");
        let resolution: String = task.get("resolution");
        let sync_from_date: chrono::NaiveDate = task.get("sync_from_date");

        info!(
            "[{}/{}] Syncing {}/{} - {}",
            i + 1,
            tasks.len(),
            exchange,
            symbol,
            resolution
        );

        // Only handle Polygon for now
        if exchange != "Polygon" {
            warn!("Skipping {} - connector not implemented", exchange);
            continue;
        }

        // Mark as syncing
        sqlx::query(
            "UPDATE market_sync_status SET status = 'syncing' WHERE exchange = $1 AND symbol = $2 AND resolution = $3",
        )
        .bind(&exchange)
        .bind(&symbol)
        .bind(&resolution)
        .execute(&pool)
        .await?;

        // Fetch data
        let from_date = sync_from_date.format("%Y-%m-%d").to_string();
        let to_date = Utc::now().format("%Y-%m-%d").to_string();

        match polygon
            .fetch_history_candles(&symbol, &resolution, &from_date, &to_date)
            .await
        {
            Ok(candles) => {
                info!("  Fetched {} candles", candles.len());

                // Insert into database
                match insert_candles(&pool, &candles).await {
                    Ok(inserted) => {
                        info!("  Inserted {} candles", inserted);

                        // Update status
                        sqlx::query(
                            r#"
                            UPDATE market_sync_status
                            SET status = 'completed',
                                total_candles = $4,
                                last_sync_at = NOW(),
                                last_synced_time = $5
                            WHERE exchange = $1 AND symbol = $2 AND resolution = $3
                            "#,
                        )
                        .bind(&exchange)
                        .bind(&symbol)
                        .bind(&resolution)
                        .bind(inserted as i32)
                        .bind(if candles.is_empty() {
                            None
                        } else {
                            Some(candles.last().unwrap().time)
                        })
                        .execute(&pool)
                        .await?;
                    }
                    Err(e) => {
                        error!("  Insert failed: {}", e);

                        sqlx::query(
                            "UPDATE market_sync_status SET status = 'failed', error_message = $4 WHERE exchange = $1 AND symbol = $2 AND resolution = $3",
                        )
                        .bind(&exchange)
                        .bind(&symbol)
                        .bind(&resolution)
                        .bind(e.to_string())
                        .execute(&pool)
                        .await?;
                    }
                }
            }
            Err(e) => {
                error!("  Fetch failed: {}", e);

                sqlx::query(
                    "UPDATE market_sync_status SET status = 'failed', error_message = $4, retry_count = retry_count + 1 WHERE exchange = $1 AND symbol = $2 AND resolution = $3",
                )
                .bind(&exchange)
                .bind(&symbol)
                .bind(&resolution)
                .bind(e.to_string())
                .execute(&pool)
                .await?;
            }
        }

        // Rate limiting
        tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
    }

    info!("Sync completed!");

    // Show stats
    let stats = sqlx::query(
        r#"
        SELECT
            status,
            COUNT(*) as count
        FROM market_sync_status
        WHERE exchange = 'Polygon'
        GROUP BY status
        "#,
    )
    .fetch_all(&pool)
    .await?;

    info!("Sync Status:");
    for stat in stats {
        let status: String = stat.get("status");
        let count: i64 = stat.get("count");
        info!("  - {}: {}", status, count);
    }

    Ok(())
}

async fn insert_candles(
    pool: &PgPool,
    candles: &[data_engine::models::Candle],
) -> Result<usize, Box<dyn Error>> {
    if candles.is_empty() {
        return Ok(0);
    }

    let mut query_builder = sqlx::QueryBuilder::new(
        "INSERT INTO mkt_equity_candles (time, exchange, symbol, resolution, open, high, low, close, volume, metadata) "
    );

    query_builder.push_values(candles, |mut b, candle| {
        b.push_bind(candle.time)
            .push_bind(&candle.exchange)
            .push_bind(&candle.symbol)
            .push_bind(&candle.resolution)
            .push_bind(candle.open)
            .push_bind(candle.high)
            .push_bind(candle.low)
            .push_bind(candle.close)
            .push_bind(candle.volume)
            .push_bind(&candle.metadata);
    });

    query_builder.push(" ON CONFLICT (time, exchange, symbol, resolution) DO NOTHING");

    let query = query_builder.build();
    let result = query.execute(pool).await?;

    Ok(result.rows_affected() as usize)
}
