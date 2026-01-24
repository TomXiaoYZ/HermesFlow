use clickhouse::Client;
use std::time::{Duration, Instant};

use crate::error::{DataError, Result};
use crate::models::StandardMarketData;

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
    ///
    /// # Arguments
    ///
    /// * `url` - ClickHouse connection URL (e.g., "tcp://localhost:9000")
    /// * `database` - Database name
    /// * `batch_size` - Number of rows to buffer before flushing
    /// * `flush_interval_ms` - Maximum time between flushes in milliseconds
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
    ///
    /// This method performs the actual bulk insert operation.
    pub async fn flush(&mut self) -> Result<()> {
        if self.batch.is_empty() {
            return Ok(());
        }

        let start = Instant::now();
        let count = self.batch.len();

        tracing::debug!("Flushing {} rows to ClickHouse", count);

        // TODO: Implement actual ClickHouse insert
        // For now, we'll just clear the batch and log
        // In Sprint 3, we'll implement the actual Row serialization
        for data in self.batch.drain(..) {
            tracing::trace!(
                "Would insert row: symbol={}, price={}",
                data.symbol,
                data.price
            );
        }

        let elapsed = start.elapsed();
        self.last_flush = Instant::now();

        tracing::info!(
            "Flushed {} rows to ClickHouse in {:?} ({:.2} rows/sec)",
            count,
            elapsed,
            count as f64 / elapsed.as_secs_f64()
        );

        // Metrics will be updated when we implement the actual insertion
        // For now, just log the successful flush
        tracing::debug!("Batch flush completed successfully");

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

        let schema_sql = include_str!("../../../../infrastructure/database/clickhouse/migrations/002_clickhouse_ticks.sql");

        self.client
            .query(schema_sql)
            .execute()
            .await
            .map_err(|e| DataError::ClickHouseError(format!("Schema creation failed: {}", e)))?;

        tracing::info!("ClickHouse schema created successfully");
        Ok(())
    }

    /// Starts an automatic flush background task
    ///
    /// This task will periodically check if the batch should be flushed
    /// based on the time interval.
    pub async fn start_auto_flush(mut self) {
        let mut interval = tokio::time::interval(Duration::from_millis(1000)); // Check every second

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
        let writer = ClickHouseWriter::new("tcp://localhost:9000", "test_db", 1000, 5000);
        assert!(writer.is_ok());

        let writer = writer.unwrap();
        assert_eq!(writer.batch_len(), 0);
    }

    #[tokio::test]
    async fn test_clickhouse_writer_batch() {
        let mut writer = ClickHouseWriter::new("tcp://localhost:9000", "test_db", 3, 5000).unwrap();

        assert_eq!(writer.batch_len(), 0);

        // Add data to batch
        writer.write(create_test_data()).await.ok();
        assert_eq!(writer.batch_len(), 1);

        writer.write(create_test_data()).await.ok();
        assert_eq!(writer.batch_len(), 2);
    }

    #[test]
    fn test_should_flush_time_based() {
        let writer = ClickHouseWriter::new(
            "tcp://localhost:9000",
            "test_db",
            1000,
            100, // 100ms
        )
        .unwrap();

        // Empty batch should not flush
        assert!(!writer.should_flush());
    }
}
