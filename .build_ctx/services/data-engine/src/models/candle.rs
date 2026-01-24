use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candle {
    pub exchange: String, // e.g. "Polygon", "Binance"
    pub symbol: String,
    pub resolution: String, // '1m', '1h', '1d'
    pub open: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub close: Decimal,
    pub volume: Decimal,
    pub amount: Option<Decimal>, // Turnover/Notional
    pub liquidity: Option<Decimal>, // DEX Liquidity (USD)
    pub fdv: Option<Decimal>, // Fully Diluted Valuation (USD)
    pub metadata: Option<serde_json::Value>,
    pub time: DateTime<Utc>,
}

impl Candle {
    pub fn new(
        exchange: String,
        symbol: String,
        resolution: String,
        open: Decimal,
        high: Decimal,
        low: Decimal,
        close: Decimal,
        volume: Decimal,
        time: DateTime<Utc>,
    ) -> Self {
        Self {
            exchange,
            symbol,
            resolution,
            open,
            high,
            low,
            close,
            volume,
            amount: None,
            liquidity: None,
            fdv: None,
            metadata: None,
            time,
        }
    }
}
