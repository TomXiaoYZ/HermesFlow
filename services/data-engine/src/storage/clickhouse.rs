use clickhouse::Client;
use rust_decimal::Decimal;
use serde::Serialize;
use std::time::{Duration, Instant};
use time::OffsetDateTime;

use crate::error::{DataError, Result};
use crate::models::StandardMarketData;

/// ClickHouse Decimal(32, 8) scale factor.
/// Values are stored as fixed-point i128 scaled by 10^8.
const CH_DECIMAL_SCALE: u32 = 8;

/// Convert `rust_decimal::Decimal` to ClickHouse `Decimal(32, 8)` wire format.
///
/// ClickHouse stores `Decimal(32, S)` as `Int128` where the value equals
/// `real_value * 10^S`. We rescale the Decimal to exactly S decimal places
/// then extract the mantissa, preserving full precision.
fn decimal_to_ch(d: Decimal) -> i128 {
    let mut d = d;
    d.rescale(CH_DECIMAL_SCALE);
    d.mantissa()
}

/// Convert optional Decimal to ClickHouse `Nullable(Decimal(32, 8))`.
fn opt_decimal_to_ch(d: Option<Decimal>) -> Option<i128> {
    d.map(decimal_to_ch)
}

/// Row struct matching ClickHouse unified_ticks table schema.
///
/// Uses `i128` for `Decimal(32, 8)` columns and `Option<i128>` for
/// `Nullable(Decimal(32, 8))` columns to preserve full financial precision.
/// Previous implementation used `f64` which silently lost precision beyond
/// ~15 significant digits.
#[derive(Debug, clickhouse::Row, Serialize)]
struct TickRow {
    source: String,
    exchange: String,
    symbol: String,
    asset_type: String,
    data_type: String,
    #[serde(with = "clickhouse::serde::time::datetime")]
    timestamp: OffsetDateTime,
    price: i128,
    quantity: i128,
    bid: Option<i128>,
    ask: Option<i128>,
    high_24h: Option<i128>,
    low_24h: Option<i128>,
    volume_24h: Option<i128>,
    funding_rate: Option<i128>,
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
            price: decimal_to_ch(data.price),
            quantity: decimal_to_ch(data.quantity),
            bid: opt_decimal_to_ch(data.bid),
            ask: opt_decimal_to_ch(data.ask),
            high_24h: opt_decimal_to_ch(data.high_24h),
            low_24h: opt_decimal_to_ch(data.low_24h),
            volume_24h: opt_decimal_to_ch(data.volume_24h),
            funding_rate: opt_decimal_to_ch(data.funding_rate),
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

    /// Flushes the current batch to ClickHouse with retry.
    ///
    /// The batch is only cleared after a successful write. On failure, the batch
    /// is preserved so the next flush attempt can retry the same data.
    pub async fn flush(&mut self) -> Result<()> {
        if self.batch.is_empty() {
            return Ok(());
        }

        let start = Instant::now();
        let count = self.batch.len();

        tracing::debug!("Flushing {} rows to ClickHouse", count);

        let result = self.try_flush_batch().await;

        match result {
            Ok(()) => {
                self.batch.clear();
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
            Err(e) => {
                tracing::error!(
                    "ClickHouse flush failed for {} rows (batch preserved for retry): {}",
                    count,
                    e
                );
                Err(e)
            }
        }
    }

    /// Attempt to write the current batch without clearing it.
    async fn try_flush_batch(&self) -> Result<()> {
        let mut insert = self
            .client
            .insert("unified_ticks")
            .map_err(|e| DataError::ClickHouseError(format!("Insert init failed: {}", e)))?;

        for data in &self.batch {
            let row = TickRow::from_market_data(data);
            insert
                .write(&row)
                .await
                .map_err(|e| DataError::ClickHouseError(format!("Row write failed: {}", e)))?;
        }

        insert
            .end()
            .await
            .map_err(|e| DataError::ClickHouseError(format!("Insert end failed: {}", e)))?;

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
        // 50000.0 * 10^8 = 5_000_000_000_000
        assert_eq!(row.price, 5_000_000_000_000_i128);
    }

    #[test]
    fn test_decimal_precision_preserved() {
        // Verify sub-penny precision is preserved (the whole point of this fix)
        let price = dec!(50000.12345678);
        let ch_val = decimal_to_ch(price);
        // 50000.12345678 * 10^8 = 5_000_012_345_678
        assert_eq!(ch_val, 5_000_012_345_678_i128);

        // Round-trip: convert back and verify
        let recovered = Decimal::new(ch_val as i64, CH_DECIMAL_SCALE);
        assert_eq!(recovered, price);
    }

    #[test]
    fn test_nullable_decimal_conversion() {
        assert_eq!(opt_decimal_to_ch(None), None);
        assert_eq!(opt_decimal_to_ch(Some(dec!(1.5))), Some(150_000_000_i128));
    }
}
