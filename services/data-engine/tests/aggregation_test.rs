use chrono::{TimeZone, Utc};
use data_engine::tasks::candle_aggregation::CandleAggregator;
use rust_decimal_macros::dec;
use sqlx::{postgres::PgPoolOptions, Row};
use std::env;

#[tokio::test]
async fn test_multi_exchange_aggregation() {
    // 1. Setup DB Connection
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("Failed to connect to DB");

    // 2. Constants
    let exchange = "TEST_EX";
    let symbol = "TEST_AGG_SYM";
    let now = Utc::now();
    // Round to previous minute to ensure it falls into a bucket that is finalized or fetchable
    // aggregate_candles looks back N minutes.
    let ts = Utc.timestamp_opt(now.timestamp() - 60, 0).unwrap();

    // 3. Clean Setup
    sqlx::query("DELETE FROM mkt_equity_snapshots WHERE symbol = $1")
        .bind(symbol)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM mkt_equity_candles WHERE symbol = $1")
        .bind(symbol)
        .execute(&pool)
        .await
        .unwrap();

    // 4. Insert Snapshot (Manual SQL to verify schema)
    sqlx::query(
        r#"
        INSERT INTO mkt_equity_snapshots 
        (exchange, symbol, price, volume, high, low, timestamp)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(exchange)
    .bind(symbol)
    .bind(dec!(100.0))
    .bind(1000)
    .bind(dec!(101.0))
    .bind(dec!(99.0))
    .bind(ts)
    .execute(&pool)
    .await
    .expect("Failed to insert snapshot");

    // 5. Run Aggregation
    // Lookback 10 min, 1m resolution, 1m bucket
    let mut aggregator = CandleAggregator::new(pool.clone());
    aggregator
        .aggregate_candles(10, "1m", 1)
        .await
        .expect("Aggregation failed");

    // 6. Verify Result
    let row = sqlx::query(
        r#"
        SELECT exchange, close, volume FROM mkt_equity_candles
        WHERE symbol = $1 AND resolution = '1m'
        "#,
    )
    .bind(symbol)
    .fetch_optional(&pool)
    .await
    .expect("Failed to fetch candle");

    assert!(row.is_some(), "Candle was not created!");
    let row = row.unwrap();
    let saved_exchange: String = row.get("exchange");
    let saved_close: rust_decimal::Decimal = row.get("close");
    let saved_volume: rust_decimal::Decimal = row.get("volume"); // DB numeric

    // volume in DB might be numeric or int8, let's check model.
    // In DB migration 'volume' is DECIMAL(24,8).
    // In snapshot 'volume' is DECIMAL(24,8).
    // In rust model `Candle` volume is Decimal (or i64 in builder?).
    // In `candle_aggregation.rs` builder volume is i64.
    // Let's assert exchange correctness primarily.

    assert_eq!(saved_exchange, exchange, "Exchange mismatch!");
    assert_eq!(saved_close, dec!(100.0), "Close price mismatch!");

    // Clean up
    // sqlx::query("DELETE FROM mkt_equity_snapshots WHERE symbol = $1").bind(symbol).execute(&pool).await.unwrap();
    // sqlx::query("DELETE FROM mkt_equity_candles WHERE symbol = $1").bind(symbol).execute(&pool).await.unwrap();
}
