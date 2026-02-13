use clap::Parser;
use clickhouse::Client;
use data_engine::{
    config::AppConfig,
    models::{AssetType, DataSourceType, MarketDataType, StandardMarketData},
    repository::{postgres::PostgresRepositories, MarketDataRepository},
    storage::ClickHouseWriter,
};
use rust_decimal::Decimal;
use serde::Deserialize;
use tracing::{error, info, warn};

#[derive(Parser, Debug)]
#[command(
    name = "replay-dead-letters",
    about = "Replay dead letter records from ClickHouse back into the main pipeline"
)]
struct Args {
    /// Filter by source (e.g., "BinanceSpot", "Jupiter"). Omit for all sources.
    #[arg(short, long)]
    source: Option<String>,

    /// Filter by symbol (e.g., "BTCUSDT"). Omit for all symbols.
    #[arg(long)]
    symbol: Option<String>,

    /// Filter by storage target (e.g., "postgres", "clickhouse"). Omit for all.
    #[arg(long)]
    target: Option<String>,

    /// Start time filter (RFC3339, e.g., "2025-01-01T00:00:00Z"). Omit for no lower bound.
    #[arg(long)]
    from: Option<String>,

    /// End time filter (RFC3339, e.g., "2025-12-31T23:59:59Z"). Omit for no upper bound.
    #[arg(long)]
    to: Option<String>,

    /// Maximum number of records to replay
    #[arg(long, default_value_t = 1000)]
    limit: u64,

    /// Dry-run mode — show what would be replayed without actually inserting
    #[arg(long, default_value_t = false)]
    dry_run: bool,

    /// Also re-insert into ClickHouse unified_ticks (by default only Postgres)
    #[arg(long, default_value_t = false)]
    include_clickhouse: bool,
}

/// Parse a DataSourceType from its string representation.
fn parse_source_type(s: &str) -> DataSourceType {
    match s {
        "BinanceSpot" => DataSourceType::BinanceSpot,
        "BinanceFutures" => DataSourceType::BinanceFutures,
        "BinancePerp" => DataSourceType::BinancePerp,
        "OkxSpot" => DataSourceType::OkxSpot,
        "OkxFutures" => DataSourceType::OkxFutures,
        "OkxPerp" => DataSourceType::OkxPerp,
        "BybitSpot" => DataSourceType::BybitSpot,
        "BybitFutures" => DataSourceType::BybitFutures,
        "BybitPerp" => DataSourceType::BybitPerp,
        "FutuStock" => DataSourceType::FutuStock,
        "BitgetSpot" => DataSourceType::BitgetSpot,
        "BitgetFutures" => DataSourceType::BitgetFutures,
        "GmgnDex" => DataSourceType::GmgnDex,
        "UniswapV3" => DataSourceType::UniswapV3,
        "Birdeye" => DataSourceType::Birdeye,
        "DexScreener" => DataSourceType::DexScreener,
        "Helius" => DataSourceType::Helius,
        "Jupiter" => DataSourceType::Jupiter,
        "IbkrStock" => DataSourceType::IbkrStock,
        "IbkrOption" => DataSourceType::IbkrOption,
        "PolygonStock" => DataSourceType::PolygonStock,
        "AlpacaStock" => DataSourceType::AlpacaStock,
        "TwitterSentiment" => DataSourceType::TwitterSentiment,
        "NewsApiSentiment" => DataSourceType::NewsApiSentiment,
        "PolymarketGamma" => DataSourceType::PolymarketGamma,
        "AkShare" => DataSourceType::AkShare,
        "FredMacro" => DataSourceType::FredMacro,
        other => DataSourceType::Other(other.to_string()),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();

    info!(
        "Dead letter replay CLI starting (dry_run={}, limit={})",
        args.dry_run, args.limit
    );

    // ── Load config ──────────────────────────────────────────────────
    let mut config = AppConfig::load().unwrap_or_default();
    if config.postgres.password.is_empty() {
        if let Ok(pw) = std::env::var("DATA_ENGINE__POSTGRES__PASSWORD") {
            config.postgres.password = pw;
        }
    }
    if let Ok(db) = std::env::var("DATA_ENGINE__POSTGRES__DATABASE") {
        config.postgres.database = db;
    }

    // ── Connect to ClickHouse ────────────────────────────────────────
    let ch_url = std::env::var("CLICKHOUSE_URL").unwrap_or_else(|_| config.clickhouse.url.clone());
    let ch_user =
        std::env::var("CLICKHOUSE_USER").unwrap_or_else(|_| config.clickhouse.username.clone());
    let ch_pass =
        std::env::var("CLICKHOUSE_PASSWORD").unwrap_or_else(|_| config.clickhouse.password.clone());
    let ch_db =
        std::env::var("CLICKHOUSE_DATABASE").unwrap_or_else(|_| config.clickhouse.database.clone());

    let mut ch_client = Client::default()
        .with_url(&ch_url)
        .with_database(&ch_db)
        .with_user(&ch_user);
    if !ch_pass.is_empty() {
        ch_client = ch_client.with_password(&ch_pass);
    }

    // ── Build query ──────────────────────────────────────────────────
    let mut conditions = vec!["replayed_at IS NULL".to_string()];
    if let Some(ref src) = args.source {
        conditions.push(format!("source = '{}'", src.replace('\'', "''")));
    }
    if let Some(ref sym) = args.symbol {
        conditions.push(format!("symbol = '{}'", sym.replace('\'', "''")));
    }
    if let Some(ref tgt) = args.target {
        conditions.push(format!("storage_target = '{}'", tgt.replace('\'', "''")));
    }
    if let Some(ref from) = args.from {
        conditions.push(format!(
            "created_at >= parseDateTimeBestEffort('{}')",
            from.replace('\'', "''")
        ));
    }
    if let Some(ref to) = args.to {
        conditions.push(format!(
            "created_at <= parseDateTimeBestEffort('{}')",
            to.replace('\'', "''")
        ));
    }

    let where_clause = conditions.join(" AND ");
    let query = format!(
        "SELECT id, source, exchange, symbol, price, quantity, timestamp, \
         storage_target, error, raw_data \
         FROM dead_letters WHERE {} ORDER BY created_at ASC LIMIT {}",
        where_clause, args.limit
    );

    info!("Query: {}", query);

    // ── Fetch dead letters ───────────────────────────────────────────
    // Use raw query to avoid Decimal deserialization issues
    let rows_result = ch_client
        .query(&format!(
            "SELECT toString(id) as id_str, source, exchange, symbol, \
             toFloat64(price) as price_f64, toFloat64(quantity) as qty_f64, \
             toUnixTimestamp64Milli(timestamp) as ts_millis, \
             storage_target, error, raw_data \
             FROM dead_letters WHERE {} ORDER BY created_at ASC LIMIT {}",
            where_clause, args.limit
        ))
        .fetch_all::<RawDeadLetter>()
        .await;

    let rows = match rows_result {
        Ok(r) => r,
        Err(e) => {
            error!("Failed to query dead_letters: {}", e);
            return Err(e.into());
        }
    };

    info!("Found {} dead letter records to replay", rows.len());
    if rows.is_empty() {
        info!("Nothing to replay.");
        return Ok(());
    }

    // ── Connect to Postgres ──────────────────────────────────────────
    let repos = if !args.dry_run {
        info!(
            "Connecting to Postgres at {}:{}...",
            config.postgres.host, config.postgres.port
        );
        Some(PostgresRepositories::new(&config.postgres).await?)
    } else {
        None
    };

    // ── Optional ClickHouse writer ───────────────────────────────────
    let mut ch_writer = if !args.dry_run && args.include_clickhouse {
        Some(ClickHouseWriter::new_with_auth(
            &ch_url, &ch_db, &ch_user, &ch_pass, 100, 5000,
        )?)
    } else {
        None
    };

    // ── Replay loop ──────────────────────────────────────────────────
    let mut success_count = 0u64;
    let mut fail_count = 0u64;

    for row in &rows {
        let source_type = parse_source_type(&row.source);
        let ts_millis = row.ts_millis;

        let price = Decimal::try_from(row.price_f64).unwrap_or_default();
        let quantity = Decimal::try_from(row.qty_f64).unwrap_or_default();

        let market_data = StandardMarketData {
            source: source_type,
            exchange: row.exchange.clone(),
            symbol: row.symbol.clone(),
            asset_type: AssetType::Spot, // Best guess; dead letter doesn't store this
            data_type: MarketDataType::Trade,
            price,
            quantity,
            timestamp: ts_millis,
            received_at: chrono::Utc::now().timestamp_millis(),
            raw_data: row.raw_data.clone(),
            ..Default::default()
        };

        if args.dry_run {
            info!(
                "[DRY-RUN] Would replay: source={} symbol={} price={} ts={} target={}",
                row.source, row.symbol, price, ts_millis, row.storage_target
            );
            success_count += 1;
            continue;
        }

        // Replay into Postgres (snapshot insert)
        let insert_ok = if let Some(ref repos) = repos {
            match repos.market_data.insert_snapshot(&market_data).await {
                Ok(()) => true,
                Err(e) => {
                    warn!(
                        "Failed to replay {} {} into Postgres: {}",
                        row.source, row.symbol, e
                    );
                    false
                }
            }
        } else {
            true
        };

        // Optionally replay into ClickHouse
        let ch_ok = if let Some(ref mut writer) = ch_writer {
            match writer.write(market_data.clone()).await {
                Ok(()) => true,
                Err(e) => {
                    warn!(
                        "Failed to replay {} {} into ClickHouse: {}",
                        row.source, row.symbol, e
                    );
                    false
                }
            }
        } else {
            true
        };

        if insert_ok && ch_ok {
            // Mark as replayed in ClickHouse
            let status = if args.include_clickhouse {
                "replayed_pg_ch"
            } else {
                "replayed_pg"
            };
            if let Err(e) = mark_replayed(&ch_client, &row.id_str, status).await {
                warn!(
                    "Failed to mark dead letter {} as replayed: {}",
                    row.id_str, e
                );
            }
            success_count += 1;
        } else {
            fail_count += 1;
        }
    }

    // Flush any remaining ClickHouse batch
    if let Some(ref mut writer) = ch_writer {
        if let Err(e) = writer.flush().await {
            error!("Failed to flush ClickHouse replay batch: {}", e);
        }
    }

    info!(
        "Replay complete: {} succeeded, {} failed out of {} total",
        success_count,
        fail_count,
        rows.len()
    );

    Ok(())
}

/// Raw dead letter row using simple types to avoid Decimal deserialization issues.
#[derive(Debug, clickhouse::Row, Deserialize)]
#[allow(dead_code)]
struct RawDeadLetter {
    id_str: String,
    source: String,
    exchange: String,
    symbol: String,
    price_f64: f64,
    qty_f64: f64,
    ts_millis: i64,
    storage_target: String,
    error: String,
    raw_data: String,
}

/// Mark a dead letter record as replayed by updating replayed_at and replay_status.
async fn mark_replayed(client: &Client, id_str: &str, status: &str) -> Result<(), String> {
    let query = format!(
        "ALTER TABLE dead_letters UPDATE replayed_at = now(), replay_status = '{}' \
         WHERE toString(id) = '{}' AND replayed_at IS NULL",
        status.replace('\'', "''"),
        id_str.replace('\'', "''"),
    );
    client
        .query(&query)
        .execute()
        .await
        .map_err(|e| format!("ALTER TABLE UPDATE failed: {}", e))?;
    Ok(())
}
