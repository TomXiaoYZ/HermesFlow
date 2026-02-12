use crate::error::DataEngineError;
use chrono::{DateTime, Duration, TimeZone, Utc};
use sqlx::postgres::PgPool;
use sqlx::Row;
use std::collections::HashMap;
use tracing::{error, info};

/// Candle Aggregator - Periodically converts snapshots to candles
pub struct CandleAggregator {
    pool: PgPool,
    last_aggregation: Option<DateTime<Utc>>,
}

impl CandleAggregator {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            last_aggregation: None,
        }
    }

    /// Run candle aggregation for the last N minutes of data
    pub async fn aggregate_candles(
        &mut self,
        lookback_minutes: i64,
        resolution_str: &str,
        bucket_minutes: i64,
    ) -> Result<(), DataEngineError> {
        info!(
            "Starting candle aggregation (Res: {}, lookback: {}min)...",
            resolution_str, lookback_minutes
        );

        let end_time = Utc::now();
        let start_time = end_time - Duration::minutes(lookback_minutes);

        // Fetch all snapshots in the time range
        // Note: For larger aggregations (e.g. 1d), looking back just N minutes might be insufficient if snapshots are sparse.
        // Ideally we should look back enough to cover the bucket.
        let snapshots = sqlx::query(
            r#"
            SELECT exchange, symbol, timestamp, price, volume, high, low
            FROM mkt_equity_snapshots
            WHERE timestamp >= $1 AND timestamp < $2
            ORDER BY timestamp ASC
            "#,
        )
        .bind(start_time)
        .bind(end_time)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DataEngineError::DatabaseError(format!("Failed to fetch snapshots: {}", e)))?;

        if snapshots.is_empty() {
            return Ok(());
        }

        // Group by exchange, symbol and bucket
        // Key: (Exchange, Symbol, TimeBucket)
        let mut candles: HashMap<(String, String, DateTime<Utc>), CandleBuilder> = HashMap::new();

        for row in snapshots {
            let exchange: String = row.get("exchange");
            let symbol: String = row.get("symbol");
            let timestamp: DateTime<Utc> = row.get("timestamp");
            let price: rust_decimal::Decimal = row.get("price");
            let volume: Option<rust_decimal::Decimal> = row.try_get("volume").ok();
            let high: Option<rust_decimal::Decimal> = row.try_get("high").ok();
            let low: Option<rust_decimal::Decimal> = row.try_get("low").ok();

            // Round logic based on resolution
            let bucket = round_to_minutes(timestamp, bucket_minutes);
            let key = (exchange, symbol, bucket);

            let builder = candles
                .entry(key)
                .or_insert_with(|| CandleBuilder::new(bucket));
            builder.add_tick(price, volume, high, low);
        }

        // Batch insert candles (much faster than individual INSERTs)
        let candle_rows: Vec<_> = candles
            .into_iter()
            .map(|((exchange, symbol, time), builder)| {
                let c = builder.finalize();
                (exchange, symbol, time, c)
            })
            .collect();

        let total = candle_rows.len();
        let mut inserted = 0;

        for chunk in candle_rows.chunks(500) {
            let mut query_builder = sqlx::QueryBuilder::new(
                "INSERT INTO mkt_equity_candles (exchange, symbol, resolution, time, open, high, low, close, volume, liquidity, fdv, amount) "
            );

            query_builder.push_values(chunk, |mut b, (exchange, symbol, time, candle)| {
                b.push_bind(exchange)
                    .push_bind(symbol)
                    .push_bind(resolution_str)
                    .push_bind(time)
                    .push_bind(candle.open)
                    .push_bind(candle.high)
                    .push_bind(candle.low)
                    .push_bind(candle.close)
                    .push_bind(candle.volume)
                    .push_bind(rust_decimal::Decimal::ZERO)
                    .push_bind(rust_decimal::Decimal::ZERO)
                    .push_bind(rust_decimal::Decimal::ZERO);
            });

            query_builder.push(
                " ON CONFLICT (exchange, symbol, resolution, time) DO UPDATE SET
                high = GREATEST(mkt_equity_candles.high, EXCLUDED.high),
                low = LEAST(mkt_equity_candles.low, EXCLUDED.low),
                close = EXCLUDED.close,
                volume = EXCLUDED.volume",
            );

            match query_builder.build().execute(&self.pool).await {
                Ok(result) => inserted += result.rows_affected() as usize,
                Err(e) => error!(
                    "Failed to batch insert candles for {}: {}",
                    resolution_str, e
                ),
            }
        }

        info!(
            "Candle aggregation ({}) complete: {}/{} candles inserted/updated",
            resolution_str, inserted, total
        );
        self.last_aggregation = Some(end_time);
        Ok(())
    }
}

// Helper to replace generic round_to_15min
fn round_to_minutes(dt: DateTime<Utc>, minutes_bucket: i64) -> DateTime<Utc> {
    // If bucket is 1440 (1 day), align to Beijing Time (UTC+8)
    if minutes_bucket >= 1440 {
        // Shift to Beijing Time
        let beijing_offset = chrono::FixedOffset::east_opt(8 * 3600).unwrap();
        let beijing_dt = dt.with_timezone(&beijing_offset);

        // Truncate to day start in Beijing Time
        let day_start = beijing_dt.date_naive().and_hms_opt(0, 0, 0).unwrap();

        // Convert back to UTC
        let day_start_beijing = beijing_offset.from_local_datetime(&day_start).unwrap();
        return day_start_beijing.with_timezone(&Utc);
    }

    // Total minutes from epoch or just naive rounding?
    // Using naive minute rounding is safe for small buckets (1, 5, 15, 60) within the hour/day
    let timestamp = dt.timestamp();
    let seconds_bucket = minutes_bucket * 60;
    let rounded_timestamp = (timestamp / seconds_bucket) * seconds_bucket;
    Utc.timestamp_opt(rounded_timestamp, 0).unwrap()
}

struct CandleBuilder {
    _time: DateTime<Utc>,
    prices: Vec<rust_decimal::Decimal>,
    high: Option<rust_decimal::Decimal>,
    low: Option<rust_decimal::Decimal>,
    volume: rust_decimal::Decimal,
}

impl CandleBuilder {
    fn new(time: DateTime<Utc>) -> Self {
        Self {
            _time: time,
            prices: Vec::new(),
            high: None,
            low: None,
            volume: rust_decimal::Decimal::ZERO,
        }
    }

    fn add_tick(
        &mut self,
        price: rust_decimal::Decimal,
        volume: Option<rust_decimal::Decimal>,
        high: Option<rust_decimal::Decimal>,
        low: Option<rust_decimal::Decimal>,
    ) {
        self.prices.push(price);
        if let Some(v) = volume {
            self.volume += v;
        }

        // Update high/low
        self.high = match (self.high, high) {
            (Some(a), Some(b)) => Some(a.max(b).max(price)),
            (Some(a), None) => Some(a.max(price)),
            (None, Some(b)) => Some(b.max(price)),
            (None, None) => Some(price),
        };

        self.low = match (self.low, low) {
            (Some(a), Some(b)) => Some(a.min(b).min(price)),
            (Some(a), None) => Some(a.min(price)),
            (None, Some(b)) => Some(b.min(price)),
            (None, None) => Some(price),
        };
    }

    fn finalize(self) -> Candle {
        let open = self
            .prices
            .first()
            .copied()
            .unwrap_or(rust_decimal::Decimal::ZERO);
        let close = self
            .prices
            .last()
            .copied()
            .unwrap_or(rust_decimal::Decimal::ZERO);
        let high = self.high.unwrap_or(open);
        let low = self.low.unwrap_or(open);

        // Ensure OHLC consistency: high >= max(open,close) and low <= min(open,close)
        let high = high.max(open).max(close);
        let low = low.min(open).min(close);

        Candle {
            open,
            high,
            low,
            close,
            volume: self.volume,
        }
    }
}

struct Candle {
    open: rust_decimal::Decimal,
    high: rust_decimal::Decimal,
    low: rust_decimal::Decimal,
    close: rust_decimal::Decimal,
    volume: rust_decimal::Decimal,
}
