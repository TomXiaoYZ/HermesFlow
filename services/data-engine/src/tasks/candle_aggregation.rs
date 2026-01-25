use crate::error::DataEngineError;
use chrono::{DateTime, Duration, Timelike, Utc};
use sqlx::postgres::PgPool;
use sqlx::Row;
use std::collections::HashMap;
use tracing::{error, info, warn};

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
    pub async fn aggregate_candles(&mut self, lookback_minutes: i64) -> Result<(), DataEngineError> {
        info!("Starting candle aggregation (lookback: {}min)...", lookback_minutes);

        let end_time = Utc::now();
        let start_time = end_time - Duration::minutes(lookback_minutes);

        // Fetch all snapshots in the time range
        let snapshots = sqlx::query(
            r#"
            SELECT symbol, timestamp, price, volume, high, low
            FROM mkt_equity_snapshots
            WHERE timestamp >= $1 AND timestamp < $2
            ORDER BY symbol, timestamp ASC
            "#,
        )
        .bind(start_time)
        .bind(end_time)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| DataEngineError::DatabaseError(format!("Failed to fetch snapshots: {}", e)))?;

        if snapshots.is_empty() {
            warn!("No snapshots found in the last {} minutes", lookback_minutes);
            return Ok(());
        }

        // Group by symbol and 15-min buckets
        let mut candles: HashMap<(String, DateTime<Utc>), CandleBuilder> = HashMap::new();

        for row in snapshots {
            let symbol: String = row.get("symbol");
            let timestamp: DateTime<Utc> = row.get("timestamp");
            let price: rust_decimal::Decimal = row.get("price");
            let volume: Option<i64> = row.try_get("volume").ok();
            let high: Option<rust_decimal::Decimal> = row.try_get("high").ok();
            let low: Option<rust_decimal::Decimal> = row.try_get("low").ok();

            // Round down to 15-minute bucket
            let bucket = round_to_15min(timestamp);
            let key = (symbol.clone(), bucket);

            let builder = candles.entry(key).or_insert_with(|| CandleBuilder::new(bucket));
            builder.add_tick(price, volume, high, low);
        }

        // Insert candles
        let mut inserted = 0;
        for ((symbol, time), builder) in candles {
            let candle = builder.finalize();
            
            let result = sqlx::query(
                r#"
                INSERT INTO mkt_equity_candles 
                (exchange, symbol, resolution, time, open, high, low, close, volume, liquidity, fdv, amount)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
                ON CONFLICT (exchange, symbol, resolution, time) DO UPDATE
                SET high = GREATEST(mkt_equity_candles.high, EXCLUDED.high),
                    low = LEAST(mkt_equity_candles.low, EXCLUDED.low),
                    close = EXCLUDED.close,
                    volume = mkt_equity_candles.volume + EXCLUDED.volume
                "#,
            )
            .bind("solana")
            .bind(&symbol)
            .bind("15m")
            .bind(time)
            .bind(candle.open)
            .bind(candle.high)
            .bind(candle.low)
            .bind(candle.close)
            .bind(candle.volume)
            .bind(rust_decimal::Decimal::ZERO)  // liquidity
            .bind(rust_decimal::Decimal::ZERO)  // fdv
            .bind(rust_decimal::Decimal::ZERO)  // amount
            .execute(&self.pool)
            .await;

            match result {
                Ok(_) => inserted += 1,
                Err(e) => error!("Failed to insert candle for {}: {}", symbol, e),
            }
        }

        info!("Candle aggregation complete: {} candles inserted/updated", inserted);
        self.last_aggregation = Some(end_time);
        Ok(())
    }
}

struct CandleBuilder {
    time: DateTime<Utc>,
    prices: Vec<rust_decimal::Decimal>,
    high: Option<rust_decimal::Decimal>,
    low: Option<rust_decimal::Decimal>,
    volume: i64,
}

impl CandleBuilder {
    fn new(time: DateTime<Utc>) -> Self {
        Self {
            time,
            prices: Vec::new(),
            high: None,
            low: None,
            volume: 0,
        }
    }

    fn add_tick(
        &mut self,
        price: rust_decimal::Decimal,
        volume: Option<i64>,
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
        let open = self.prices.first().copied().unwrap_or(rust_decimal::Decimal::ZERO);
        let close = self.prices.last().copied().unwrap_or(rust_decimal::Decimal::ZERO);
        let high = self.high.unwrap_or(open);
        let low = self.low.unwrap_or(open);

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
    volume: i64,
}

fn round_to_15min(dt: DateTime<Utc>) -> DateTime<Utc> {
    let minutes = dt.minute();
    let rounded_minutes = (minutes / 15) * 15;
    dt.date_naive()
        .and_hms_opt(dt.hour(), rounded_minutes, 0)
        .unwrap()
        .and_utc()
}
