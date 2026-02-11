use async_trait::async_trait;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use sqlx::PgPool;
// use tracing::error; // Removed unused import
use crate::error::DataEngineError;
use crate::models::{Candle, StandardMarketData};
use crate::repository::MarketDataRepository;

pub struct PostgresMarketDataRepository {
    pool: PgPool,
}

impl PostgresMarketDataRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MarketDataRepository for PostgresMarketDataRepository {
    async fn insert_snapshot(&self, data: &StandardMarketData) -> Result<(), DataEngineError> {
        let ts = chrono::DateTime::from_timestamp(
            data.timestamp / 1000,
            ((data.timestamp % 1000) * 1_000_000) as u32,
        )
        .unwrap_or(chrono::Utc::now());

        // Debug logging decreased to trace or if specific criteria met
        // tracing::info!("Insertion...");

        sqlx::query(
            r#"
            INSERT INTO mkt_equity_snapshots (
                exchange, symbol, price, bid, ask, 
                volume, vwap, high, low, timestamp
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#
        )
        // Wait, I need to match the SQL parameters exactly to the table columns from postgres.rs
        // postgres.rs insert_equity_snapshot used 13 args.
        // StandardMarketData has: price, bid, ask, high_24h, low_24h...
        // It does NOT have bid_size, ask_size, vwap, open, prev_close explicitly (except raw_data).
        // However, the COLLECTOR currently constructs StandardMarketData.
        // If the Collector puts data into StandardMarketData, where does it put 'open'?
        // 'open' is not in StandardMarketData!
        // This is a disconnect.
        // IBKRCollector currently constructs StandardMarketData AND calls insert_equity_snapshot separately?
        // No, in my refactor I want generic storage.
        // If StandardMarketData is the DTO, it must support all fields, or we accept data loss.
        // OR the Repository logic can extract from raw_data if needed (messy).
        // Solution: Update StandardMarketData to include 'open', 'prev_close', 'vwap'?
        // Or keep PostgresWriter's specialized methods but on the trait?
        // The Trait `insert_snapshot` takes `StandardMarketData`.
        // If `StandardMarketData` is insufficient, I should improve it.
        // "Best practice": Domain model should support the data we care about.
        // I'll add fields to `StandardMarketData` in Phase 4 (Model improvement),
        // or just map what we have now.
        // Currently IBKRCollector (Step 1020) maps:
        // price, quantity (volume), high_24h, low_24h, volume_24h.
        // It sets generic params.
        // It does NOT invoke `insert_equity_snapshot` in the `connect` loop for REALTIME bars?
        // Wait, calling `postgresql.insert_equity_candle` for historical.
        // For realtime, it sends to `tx` (channel).
        // Where does `rx` go? `main.rs` listens to `rx`.
        // `main.rs` loop calls `postgres.insert_equity_snapshot`?
        // I should check `main.rs`.
        // If `main.rs` extracts fields from `StandardMarketData`, then `StandardMarketData` MUST have them.
        // If `StandardMarketData` lacks `open`, then `main.rs` can't insert it.
        // So I'll check `main.rs` logic.
        .bind(&data.exchange) // $1
        .bind(&data.symbol)   // $2
        .bind(data.price)     // $3
        .bind(data.bid)       // $4
        .bind(data.ask)       // $5
        .bind(data.quantity.to_i64().unwrap_or(0)) // volume ($6)
        .bind(None::<Decimal>) // vwap ($7)
        .bind(data.high_24h) // high ($8)
        .bind(data.low_24h) // low ($9)
        .bind(ts) // timestamp ($10)
        .execute(&self.pool)
        .await
        .map_err(|e| DataEngineError::DatabaseError(format!("Failed to insert equity snapshot: {}", e)))?;
        Ok(())
    }

    async fn insert_candle(&self, data: &Candle) -> Result<(), DataEngineError> {
        sqlx::query(
            r#"
            INSERT INTO mkt_equity_candles (
                exchange, symbol, resolution, open, high, low, close, volume, amount, liquidity, fdv, metadata, time
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            ON CONFLICT (exchange, symbol, resolution, time) DO UPDATE SET
                open = EXCLUDED.open,
                high = EXCLUDED.high,
                low = EXCLUDED.low,
                close = EXCLUDED.close,
                volume = EXCLUDED.volume,
                amount = EXCLUDED.amount,
                liquidity = EXCLUDED.liquidity,
                fdv = EXCLUDED.fdv,
                metadata = EXCLUDED.metadata
            "#
        )
        .bind(&data.exchange)
        .bind(&data.symbol)
        .bind(&data.resolution)
        .bind(data.open)
        .bind(data.high)
        .bind(data.low)
        .bind(data.close)
        .bind(data.volume)
        .bind(data.amount)
        .bind(data.liquidity)
        .bind(data.fdv)
        .bind(&data.metadata)
        .bind(data.time)
        .execute(&self.pool)
        .await
        .map_err(|e| DataEngineError::DatabaseError(format!("Failed to insert candle: {}", e)))?;
        Ok(())
    }

    async fn insert_candles(&self, candles: &[Candle]) -> Result<(), DataEngineError> {
        if candles.is_empty() {
            return Ok(());
        }

        for chunk in candles.chunks(1000) {
            let mut query_builder = sqlx::QueryBuilder::new(
                "INSERT INTO mkt_equity_candles (exchange, symbol, resolution, open, high, low, close, volume, amount, liquidity, fdv, metadata, time) "
            );

            query_builder.push_values(chunk, |mut b, candle| {
                b.push_bind(&candle.exchange)
                    .push_bind(&candle.symbol)
                    .push_bind(&candle.resolution)
                    .push_bind(candle.open)
                    .push_bind(candle.high)
                    .push_bind(candle.low)
                    .push_bind(candle.close)
                    .push_bind(candle.volume)
                    .push_bind(candle.amount)
                    .push_bind(candle.liquidity)
                    .push_bind(candle.fdv)
                    .push_bind(&candle.metadata)
                    .push_bind(candle.time);
            });

            query_builder.push(
                " ON CONFLICT (exchange, symbol, resolution, time) DO UPDATE SET
                open = EXCLUDED.open,
                high = EXCLUDED.high,
                low = EXCLUDED.low,
                close = EXCLUDED.close,
                volume = EXCLUDED.volume,
                amount = EXCLUDED.amount,
                liquidity = EXCLUDED.liquidity,
                fdv = EXCLUDED.fdv,
                metadata = EXCLUDED.metadata",
            );

            query_builder
                .build()
                .execute(&self.pool)
                .await
                .map_err(|e| {
                    DataEngineError::DatabaseError(format!("Failed to batch insert candles: {}", e))
                })?;
        }

        Ok(())
    }

    async fn get_active_symbols(&self) -> Result<Vec<String>, DataEngineError> {
        let rows = sqlx::query(r#"SELECT DISTINCT symbol FROM mkt_equity_snapshots"#)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| {
                DataEngineError::DatabaseError(format!("Failed to fetch active symbols: {}", e))
            })?;

        Ok(rows
            .iter()
            .map(|row| {
                use sqlx::Row;
                row.get::<String, _>("symbol")
            })
            .collect())
    }

    async fn get_latest_candle_time(
        &self,
        exchange: &str,
        symbol: &str,
        resolution: &str,
    ) -> Result<Option<chrono::DateTime<chrono::Utc>>, DataEngineError> {
        let row = sqlx::query(
            r#"SELECT MAX(time) as max_time FROM mkt_equity_candles WHERE exchange = $1 AND symbol = $2 AND resolution = $3"#
        )
        .bind(exchange)
        .bind(symbol)
        .bind(resolution)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            DataEngineError::DatabaseError(format!("Failed to fetch latest candle time: {}", e))
        })?;

        if let Some(row) = row {
            use sqlx::Row;
            Ok(row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("max_time"))
        } else {
            Ok(None)
        }
    }
}
