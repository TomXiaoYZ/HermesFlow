use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// OHLCV Candle structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candle {
    pub symbol: String,
    pub resolution: String, // '1m', '1h', etc.
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: i64,
    pub timestamp: DateTime<Utc>,
    pub received_at: Option<DateTime<Utc>>,
}

impl Candle {
    pub fn new(
        symbol: String,
        resolution: String,
        open: Decimal,
        high: Decimal,
        low: Decimal,
        close: Decimal,
        volume: i64,
        timestamp: DateTime<Utc>,
    ) -> Self {
        Self {
            symbol,
            resolution,
            open,
            high,
            low,
            close,
            volume,
            timestamp,
            received_at: Some(Utc::now()),
        }
    }
}
