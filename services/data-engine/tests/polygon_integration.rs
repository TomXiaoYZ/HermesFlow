/// Example integration test for Polygon connector
///
/// NOTE: Requires valid POLYGON_API_KEY environment variable
/// Run with: cargo test --test polygon_integration -- --ignored
///
use data_engine::collectors::polygon::{sync_polygon_history, PolygonConfig, PolygonConnector};

#[tokio::test]
#[ignore] // Requires API key
async fn test_fetch_aapl_daily() {
    // Load config from environment
    let config = PolygonConfig::from_env().expect("POLYGON_API_KEY must be set");
    let connector = PolygonConnector::new(config);

    // Fetch 1 month of daily data
    let candles = connector
        .fetch_history_candles("AAPL", "1d", "2024-01-01", "2024-01-31")
        .await
        .expect("Failed to fetch candles");

    println!("Fetched {} daily candles for AAPL", candles.len());

    assert!(!candles.is_empty());
    assert_eq!(candles[0].symbol, "AAPL");
    assert_eq!(candles[0].exchange, "Polygon");
    assert_eq!(candles[0].resolution, "1d");

    // Verify OHLC data
    for candle in &candles {
        assert!(candle.high >= candle.low);
        assert!(candle.high >= candle.open);
        assert!(candle.high >= candle.close);
        assert!(candle.low <= candle.open);
        assert!(candle.low <= candle.close);
        assert!(candle.volume > rust_decimal::Decimal::ZERO);
    }
}

#[tokio::test]
#[ignore] // Requires API key
async fn test_fetch_hourly_with_chunking() {
    let config = PolygonConfig::from_env().expect("POLYGON_API_KEY must be set");
    let connector = PolygonConnector::new(config);

    // Fetch 3 months of hourly data (will trigger chunking)
    let candles = connector
        .fetch_history_candles("MSFT", "1h", "2024-01-01", "2024-03-31")
        .await
        .expect("Failed to fetch candles");

    println!("Fetched {} hourly candles for MSFT", candles.len());

    assert!(!candles.is_empty());

    // Verify chronological order
    for i in 1..candles.len() {
        assert!(candles[i].time >= candles[i - 1].time);
    }
}

#[tokio::test]
#[ignore] // Requires API key + database
async fn test_historical_sync() {
    // This test requires running TimescaleDB
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:password@localhost:5432/hermesflow".to_string());

    let pool = sqlx::PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    let config = PolygonConfig::from_env().expect("POLYGON_API_KEY must be set");
    let connector = PolygonConnector::new(config);

    let tickers = vec!["AAPL".to_string()];
    let resolutions = vec!["1h".to_string(), "1d".to_string()];

    sync_polygon_history(
        &pool,
        &connector,
        tickers,
        resolutions,
        "2024-01-01",
        "2024-01-07", // 1 week
    )
    .await
    .expect("Sync failed");

    // Verify data was inserted
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM mkt_equity_candles WHERE exchange = 'Polygon' AND symbol = 'AAPL'",
    )
    .fetch_one(&pool)
    .await
    .expect("Query failed");

    println!("Inserted {} candles into database", count);
    assert!(count > 0);
}
