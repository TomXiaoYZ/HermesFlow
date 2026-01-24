use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Prediction market data (Polymarket, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionMarket {
    /// Market ID
    pub id: String,
    /// Data source (e.g., "Polymarket")
    pub source: String,
    /// Market title
    pub title: String,
    /// Market description
    pub description: Option<String>,
    /// Market category
    pub category: Option<String>,
    /// Market end date
    pub end_date: Option<DateTime<Utc>>,
    /// When the market was created
    pub created_at: DateTime<Utc>,
    /// Last update time
    pub updated_at: DateTime<Utc>,
    /// Whether the market is active
    pub active: bool,
    /// Market outcomes with current prices
    pub outcomes: Vec<MarketOutcome>,
    /// Additional metadata
    pub metadata: serde_json::Value,
}

/// Market outcome with price data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketOutcome {
    /// Outcome name
    pub outcome: String,
    /// Current price (probability)
    pub price: Decimal,
    /// 24h trading volume
    pub volume_24h: Option<Decimal>,
    /// Timestamp of this price
    pub timestamp: DateTime<Utc>,
}

impl PredictionMarket {
    /// Creates a new PredictionMarket instance
    pub fn new(id: String, source: String, title: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            source,
            title,
            description: None,
            category: None,
            end_date: None,
            created_at: now,
            updated_at: now,
            active: true,
            outcomes: vec![],
            metadata: serde_json::json!({}),
        }
    }

    /// Adds an outcome to the market
    pub fn add_outcome(&mut self, outcome: String, price: Decimal, volume_24h: Option<Decimal>) {
        self.outcomes.push(MarketOutcome {
            outcome,
            price,
            volume_24h,
            timestamp: Utc::now(),
        });
    }

    /// Gets the total probability (should be ~1.0 for valid markets)
    pub fn total_probability(&self) -> Decimal {
        self.outcomes.iter().map(|o| o.price).sum()
    }

    /// Checks if the market has ended
    pub fn is_ended(&self) -> bool {
        match self.end_date {
            Some(end) => end < Utc::now(),
            None => false,
        }
    }

    /// Gets the most likely outcome
    pub fn most_likely_outcome(&self) -> Option<&MarketOutcome> {
        self.outcomes.iter().max_by(|a, b| a.price.cmp(&b.price))
    }

    /// Calculates total 24h volume
    pub fn total_volume_24h(&self) -> Option<Decimal> {
        let volumes: Vec<Decimal> = self.outcomes.iter().filter_map(|o| o.volume_24h).collect();
        if volumes.is_empty() {
            None
        } else {
            Some(volumes.iter().sum())
        }
    }
}

impl MarketOutcome {
    /// Creates a new MarketOutcome instance
    pub fn new(outcome: String, price: Decimal) -> Self {
        Self {
            outcome,
            price,
            volume_24h: None,
            timestamp: Utc::now(),
        }
    }

    /// Converts price to percentage
    pub fn price_percentage(&self) -> Decimal {
        self.price * Decimal::from(100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_prediction_market_creation() {
        let market = PredictionMarket::new(
            "test-market-1".to_string(),
            "Polymarket".to_string(),
            "Will it rain tomorrow?".to_string(),
        );

        assert_eq!(market.id, "test-market-1");
        assert_eq!(market.source, "Polymarket");
        assert_eq!(market.title, "Will it rain tomorrow?");
        assert!(market.active);
    }

    #[test]
    fn test_add_outcome() {
        let mut market = PredictionMarket::new(
            "test-market-1".to_string(),
            "Polymarket".to_string(),
            "Will it rain tomorrow?".to_string(),
        );

        market.add_outcome("Yes".to_string(), dec!(0.6), Some(dec!(1000.0)));
        market.add_outcome("No".to_string(), dec!(0.4), Some(dec!(800.0)));

        assert_eq!(market.outcomes.len(), 2);
        assert_eq!(market.total_probability(), dec!(1.0));
    }

    #[test]
    fn test_most_likely_outcome() {
        let mut market = PredictionMarket::new(
            "test-market-1".to_string(),
            "Polymarket".to_string(),
            "Will it rain tomorrow?".to_string(),
        );

        market.add_outcome("Yes".to_string(), dec!(0.7), None);
        market.add_outcome("No".to_string(), dec!(0.3), None);

        let most_likely = market.most_likely_outcome().unwrap();
        assert_eq!(most_likely.outcome, "Yes");
    }

    #[test]
    fn test_total_volume() {
        let mut market = PredictionMarket::new(
            "test-market-1".to_string(),
            "Polymarket".to_string(),
            "Will it rain tomorrow?".to_string(),
        );

        market.add_outcome("Yes".to_string(), dec!(0.6), Some(dec!(1000.0)));
        market.add_outcome("No".to_string(), dec!(0.4), Some(dec!(800.0)));

        assert_eq!(market.total_volume_24h(), Some(dec!(1800.0)));
    }

    #[test]
    fn test_price_percentage() {
        let outcome = MarketOutcome::new("Yes".to_string(), dec!(0.75));
        assert_eq!(outcome.price_percentage(), dec!(75));
    }
}
