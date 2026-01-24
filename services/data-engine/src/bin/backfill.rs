use clap::Parser;
use data_engine::{
    collectors::MassiveConnector,
    config::{AppConfig, MassiveConfig},
    models::{Candle, MarketDataType, StandardMarketData},
    repository::{postgres::PostgresRepositories, MarketDataRepository},
};
use std::sync::Arc;
use tracing::{error, info, warn};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Data source (only 'massive' supported for now)
    #[arg(short, long, default_value = "massive")]
    source: String,

    /// Ticker symbol (e.g., AAPL)
    #[arg(short = 't', long)]
    symbol: String,

    /// Start date (YYYY-MM-DD)
    #[arg(long)]
    from: String,

    /// End date (YYYY-MM-DD)
    #[arg(long)]
    to: String,

    /// Resolution (multiplier) - e.g. 1
    #[arg(long, default_value_t = 1)]
    multiplier: i32,

    /// Timespan (minute, hour, day)
    #[arg(long, default_value = "day")]
    timespan: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // simple logging
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    // Load config
    let mut config = AppConfig::load().unwrap_or_default();

    // Manual override because config crate env loading seems flaky for password
    if config.postgres.password.is_empty() {
        if let Ok(pw) = std::env::var("DATA_ENGINE__POSTGRES__PASSWORD") {
            config.postgres.password = pw;
        }
    }
    // Manual override for database name too
    if let Ok(db) = std::env::var("DATA_ENGINE__POSTGRES__DATABASE") {
        config.postgres.database = db;
    }

    // Connect to DB (Required)
    println!(
        "Connecting to database at {}:{}...",
        config.postgres.host, config.postgres.port
    );
    println!(
        "DEBUG: Postgres User: {}, Password len: {}",
        config.postgres.username,
        config.postgres.password.len()
    );
    let repos = PostgresRepositories::new(&config.postgres).await?;

    if args.source == "massive" {
        // Use config if available, or try env var directly if config load failed partially
        let api_key = if let Some(massive_cfg) = &config.massive {
            massive_cfg.api_key.clone()
        } else {
            // Fallback
            std::env::var("DATA_ENGINE__MASSIVE__API_KEY").unwrap_or_default()
        };

        if api_key.is_empty() {
            error!("Massive API Key not found. Set DATA_ENGINE__MASSIVE__API_KEY.");
            return Ok(());
        }

        info!(
            "Initializing Massive Connector for {} (Rate Limit: 5/min default)",
            args.symbol
        );
        let config = MassiveConfig {
            enabled: true,
            api_key,
            rate_limit_per_min: 5,
            ws_url: "wss://socket.polygon.io/stocks".to_string(),
        };
        let connector = MassiveConnector::new(config);

        info!("Fetching history from {} to {}...", args.from, args.to);
        let candles = connector
            .fetch_history_candles(
                &args.symbol,
                args.multiplier,
                &args.timespan,
                &args.from,
                &args.to,
            )
            .await?;

        info!("Fetched {} candles. Saving to DB...", candles.len());
        let total_count = candles.len();

        let mut upserted = 0;
        for point in candles {
            // Map StandardMarketData to Candle
            let metadata: Option<serde_json::Value> = serde_json::from_str(&point.raw_data).ok();

            // Helper to convert i64 msec to DateTime<Utc>
            let seconds = point.timestamp / 1000;
            let nsec = ((point.timestamp % 1000) * 1_000_000) as u32;
            let time =
                chrono::DateTime::from_timestamp(seconds, nsec).unwrap_or(chrono::Utc::now());

            let amount = if let Some(ref meta) = metadata {
                if let Some(vwap_val) = meta.get("vwap").and_then(|v| v.as_f64()) {
                    use rust_decimal::prelude::FromPrimitive;
                    let vwap = rust_decimal::Decimal::from_f64(vwap_val).unwrap_or_default();
                    Some(vwap * point.quantity)
                } else {
                    None
                }
            } else {
                None
            };

            // Extract open from metadata if present
            let open_price = if let Some(ref meta) = metadata {
                if let Some(o) = meta.get("o").and_then(|v| v.as_f64()) {
                    use rust_decimal::prelude::FromPrimitive;
                    rust_decimal::Decimal::from_f64(o).unwrap_or(point.price)
                } else if let Some(o) = meta.get("open").and_then(|v| v.as_f64()) {
                    use rust_decimal::prelude::FromPrimitive;
                    rust_decimal::Decimal::from_f64(o).unwrap_or(point.price)
                } else {
                    point.price
                }
            } else {
                point.price
            };

            let candle = Candle {
                exchange: point.exchange.clone(), // Use exchange from StandardMarketData (e.g. "Polygon")
                symbol: point.symbol.clone(),
                resolution: format!(
                    "{}{}",
                    args.multiplier,
                    args.timespan.chars().next().unwrap_or('?')
                ), // e.g. "1d"
                open: open_price,
                high: point.high_24h.unwrap_or(point.price),
                low: point.low_24h.unwrap_or(point.price),
                close: point.price,
                volume: point.quantity,
                amount,
                liquidity: None,
                fdv: None,
                metadata: metadata.clone(),
                time,
            };

            if let Err(e) = repos.market_data.insert_candle(&candle).await {
                error!("Failed to insert candle for {}: {}", time, e);
            } else {
                upserted += 1;
            }
        }

        info!(
            "Backfill completed. Processed: {}, Upserted: {}",
            total_count, upserted
        );
    } else {
        error!("Unlock implemented for source: {}", args.source);
    }

    Ok(())
}
