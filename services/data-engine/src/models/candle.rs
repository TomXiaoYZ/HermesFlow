use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

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
    pub amount: Option<Decimal>,    // Turnover/Notional
    pub liquidity: Option<Decimal>, // DEX Liquidity (USD)
    pub fdv: Option<Decimal>,       // Fully Diluted Valuation (USD)
    pub metadata: Option<serde_json::Value>,
    pub time: DateTime<Utc>,
}

impl Candle {
    /// Validates candle data for basic sanity.
    pub fn validate(&self) -> Result<(), String> {
        if self.symbol.is_empty() {
            return Err("symbol is empty".into());
        }
        if self.high < self.low {
            return Err(format!(
                "high {} < low {} for {}",
                self.high, self.low, self.symbol
            ));
        }
        if self.high < self.open || self.high < self.close {
            return Err(format!(
                "high {} < open {} or close {} for {}",
                self.high, self.open, self.close, self.symbol
            ));
        }
        if self.low > self.open || self.low > self.close {
            return Err(format!(
                "low {} > open {} or close {} for {}",
                self.low, self.open, self.close, self.symbol
            ));
        }
        if self.volume < Decimal::ZERO {
            return Err(format!("volume is negative: {}", self.volume));
        }
        // Timestamp sanity: not before 2000-01-01
        let min_time = chrono::DateTime::parse_from_rfc3339("2000-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        if self.time < min_time {
            return Err(format!("candle time too old: {}", self.time));
        }
        Ok(())
    }

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
