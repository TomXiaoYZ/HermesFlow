use clickhouse::Client;
use rust_decimal::prelude::ToPrimitive;
use serde::Serialize;
use std::time::{Duration, Instant};
use time::OffsetDateTime;

use crate::error::{DataError, Result};
use crate::models::StandardMarketData;

/// Row struct matching ClickHouse unified_ticks table schema.
/// Uses clickhouse::Row derive for efficient serialization.
#[derive(Debug, clickhouse::Row, Serialize)]
struct TickRow {
    source: String,
    exchange: String,
    symbol: String,
    asset_type: String,
    data_type: String,
    #[serde(with = "clickhouse::serde::time::datetime")]
    timestamp: OffsetDateTime,
    price: f64,
    quantity: f64,
    bid: f64,
    ask: f64,
    high_24h: f64,
    low_24h: f64,
    volume_24h: f64,
    funding_rate: f64,
    raw_data: String,
}

impl TickRow {
    fn from_market_data(data: &StandardMarketData) -> Self {
        let ts_secs = data.timestamp / 1000;
        let timestamp =
            OffsetDateTime::from_unix_timestamp(ts_secs).unwrap_or(OffsetDateTime::now_utc());

        Self {
            source: data.source.as_str().to_string(),
            exchange: data.exchange.clone(),
            symbol: data.symbol.clone(),
            asset_type: format!("{:?}", data.asset_type),
            data_type: format!("{:?}", data.data_type),
            timestamp,
            price: data.price.to_f64().unwrap_or(0.0),
            quantity: data.quantity.to_f64().unwrap_or(0.0),
            bid: data.bid.and_then(|d| d.to_f64()).unwrap_or(0.0),
            ask: data.ask.and_then(|d| d.to_f64()).unwrap_or(0.0),
            high_24h: data.high_24h.and_then(|d| d.to_f64()).unwrap_or(0.0),
            low_24h: data.low_24h.and_then(|d| d.to_f64()).unwrap_or(0.0),
            volume_24h: data.volume_24h.and_then(|d| d.to_f64()).unwrap_or(0.0),
            funding_rate: data.funding_rate.and_then(|d| d.to_f64()).unwrap_or(0.0),
            raw_data: data.raw_data.clone(),
        }
    }
}

/// ClickHouse writer for batch inserting market data
///
/// This writer buffers market data and performs batch inserts to ClickHouse
/// for optimal performance. It supports automatic flushing based on batch size
/// and time intervals.
pub struct ClickHouseWriter {
    client: Client,
    batch: Vec<StandardMarketData>,
    batch_size: usize,
    flush_interval: Duration,
    last_flush: Instant,
}

impl ClickHouseWriter {
    /// Creates a new ClickHouse writer
    pub fn new(
        url: &str,
        database: &str,
        batch_size: usize,
        flush_interval_ms: u64,
    ) -> Result<Self> {
        let client = Client::default().with_url(url).with_database(database);

        Ok(Self {
            client,
            batch: Vec::with_capacity(batch_size),
            batch_size,
            flush_interval: Duration::from_millis(flush_interval_ms),
            last_flush: Instant::now(),
        })
    }

    /// Returns a reference to the underlying ClickHouse client
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Adds market data to the batch
    ///
    /// If the batch reaches the configured size, it will be automatically flushed.
    pub async fn write(&mut self, data: StandardMarketData) -> Result<()> {
        self.batch.push(data);

        if self.batch.len() >= self.batch_size {
            self.flush().await?;
        }

        Ok(())
    }

    /// Flushes the current batch to ClickHouse
    pub async fn flush(&mut self) -> Result<()> {
        if self.batch.is_empty() {
            return Ok(());
        }

        let start = Instant::now();
        let count = self.batch.len();

        tracing::debug!("Flushing {} rows to ClickHouse", count);

        let mut insert = self
            .client
            .insert("unified_ticks")
            .map_err(|e| DataError::ClickHouseError(format!("Insert init failed: {}", e)))?;

        for data in self.batch.drain(..) {
            let row = TickRow::from_market_data(&data);
            insert
                .write(&row)
                .await
                .map_err(|e| DataError::ClickHouseError(format!("Row write failed: {}", e)))?;
        }

        insert
            .end()
            .await
            .map_err(|e| DataError::ClickHouseError(format!("Insert end failed: {}", e)))?;

        let elapsed = start.elapsed();
        self.last_flush = Instant::now();

        tracing::info!(
            "Flushed {} rows to ClickHouse in {:?} ({:.0} rows/sec)",
            count,
            elapsed,
            if elapsed.as_secs_f64() > 0.0 {
                count as f64 / elapsed.as_secs_f64()
            } else {
                count as f64
            }
        );

        Ok(())
    }

    /// Checks if the batch should be flushed based on time interval
    pub fn should_flush(&self) -> bool {
        !self.batch.is_empty() && self.last_flush.elapsed() >= self.flush_interval
    }

    /// Returns the current batch size
    pub fn batch_len(&self) -> usize {
        self.batch.len()
    }

    /// Creates the unified_ticks table schema if it doesn't exist
    pub async fn create_schema(&self) -> Result<()> {
        tracing::info!("Creating ClickHouse schema");

        let schema_sql = include_str!(
            "../../../../infrastructure/database/clickhouse/migrations/002_clickhouse_ticks.sql"
        );

        self.client
            .query(schema_sql)
            .execute()
            .await
            .map_err(|e| DataError::ClickHouseError(format!("Schema creation failed: {}", e)))?;

        tracing::info!("ClickHouse schema created successfully");
        Ok(())
    }

    /// Starts an automatic flush background task
    pub async fn start_auto_flush(mut self) {
        let mut interval = tokio::time::interval(Duration::from_millis(1000));

        loop {
            interval.tick().await;

            if self.should_flush() {
                if let Err(e) = self.flush().await {
                    tracing::error!("Auto-flush error: {}", e);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AssetType, DataSourceType, MarketDataType};
    use rust_decimal_macros::dec;

    fn create_test_data() -> StandardMarketData {
        StandardMarketData::new(
            DataSourceType::BinanceSpot,
            "BTCUSDT".to_string(),
            AssetType::Spot,
            MarketDataType::Trade,
            dec!(50000.0),
            dec!(0.1),
            chrono::Utc::now().timestamp_millis(),
        )
    }

    #[test]
    fn test_clickhouse_writer_creation() {
        let writer = ClickHouseWriter::new("http://localhost:8123", "test_db", 1000, 5000);
        assert!(writer.is_ok());

        let writer = writer.unwrap();
        assert_eq!(writer.batch_len(), 0);
    }

    #[tokio::test]
    async fn test_clickhouse_writer_batch() {
        let mut writer =
            ClickHouseWriter::new("http://localhost:8123", "test_db", 3, 5000).unwrap();

        assert_eq!(writer.batch_len(), 0);

        writer.write(create_test_data()).await.ok();
        assert_eq!(writer.batch_len(), 1);

        writer.write(create_test_data()).await.ok();
        assert_eq!(writer.batch_len(), 2);
    }

    #[test]
    fn test_should_flush_time_based() {
        let writer = ClickHouseWriter::new(
            "http://localhost:8123",
            "test_db",
            1000,
            100, // 100ms
        )
        .unwrap();

        // Empty batch should not flush
        assert!(!writer.should_flush());
    }

    #[test]
    fn test_tick_row_conversion() {
        let data = create_test_data();
        let row = TickRow::from_market_data(&data);
        assert_eq!(row.symbol, "BTCUSDT");
        assert_eq!(row.exchange, "Binance");
        assert!((row.price - 50000.0).abs() < 0.01);
    }
}
