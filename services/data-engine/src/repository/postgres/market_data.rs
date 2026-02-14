use crate::error::DataEngineError;
use crate::models::{Candle, StandardMarketData};
use crate::repository::MarketDataRepository;
use async_trait::async_trait;
use sqlx::PgPool;

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

        let received = chrono::DateTime::from_timestamp(
            data.received_at / 1000,
            ((data.received_at % 1000) * 1_000_000) as u32,
        )
        .unwrap_or(chrono::Utc::now());

        sqlx::query(
            r#"
            INSERT INTO mkt_equity_snapshots (
                exchange, symbol, price, bid, ask,
                bid_size, ask_size, volume, time, received_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (exchange, symbol, time) DO NOTHING
            "#,
        )
        .bind(&data.exchange)
        .bind(&data.symbol)
        .bind(data.price)
        .bind(data.bid)
        .bind(data.ask)
        .bind(data.bid_size)
        .bind(data.ask_size)
        .bind(data.quantity)
        .bind(ts)
        .bind(received)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            DataEngineError::DatabaseError(format!("Failed to insert equity snapshot: {}", e))
        })?;
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

    async fn get_watchlist_symbols(&self) -> Result<Vec<String>, DataEngineError> {
        let rows = sqlx::query(r#"SELECT symbol FROM market_watchlist WHERE is_active = true"#)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| {
                DataEngineError::DatabaseError(format!("Failed to fetch watchlist symbols: {}", e))
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
