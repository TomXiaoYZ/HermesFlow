use super::client::PolygonClient;
use super::config::PolygonConfig;
use super::types::{resolution_to_polygon_params, AggregateBar};
use crate::models::Candle;
use chrono::{TimeZone, Utc};
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use std::error::Error;
use tracing::{error, info};

pub struct PolygonConnector {
    client: PolygonClient,
}

impl PolygonConnector {
    pub fn new(config: PolygonConfig) -> Self {
        let client = PolygonClient::new(config);
        Self { client }
    }

    /// Fetch historical candles for a ticker with automatic date chunking
    ///
    /// # Arguments
    /// * `ticker` - Stock symbol (e.g., "AAPL")
    /// * `resolution` - Candle resolution (e.g., "1m", "15m", "1h", "1d")
    /// * `from_date` - Start date in YYYY-MM-DD format
    /// * `to_date` - End date in YYYY-MM-DD format
    pub async fn fetch_history_candles(
        &self,
        ticker: &str,
        resolution: &str,
        from_date: &str,
        to_date: &str,
    ) -> Result<Vec<Candle>, Box<dyn Error + Send + Sync>> {
        // Map resolution to Polygon API parameters
        let (multiplier, timespan) = resolution_to_polygon_params(resolution)?;

        info!(
            "Fetching Polygon history for {} ({}): {} to {}",
            ticker, resolution, from_date, to_date
        );

        // Parse dates
        let from = chrono::NaiveDate::parse_from_str(from_date, "%Y-%m-%d")
            .map_err(|e| format!("Invalid from_date format: {}", e))?;
        let to = chrono::NaiveDate::parse_from_str(to_date, "%Y-%m-%d")
            .map_err(|e| format!("Invalid to_date format: {}", e))?;

        // Calculate chunk size based on resolution to stay under 50k bar limit
        let chunk_days = self.calculate_chunk_days(resolution);

        let mut all_candles = Vec::new();
        let mut current_from = from;

        while current_from <= to {
            let current_to =
                std::cmp::min(current_from + chrono::Duration::days(chunk_days as i64), to);

            let from_str = current_from.format("%Y-%m-%d").to_string();
            let to_str = current_to.format("%Y-%m-%d").to_string();

            info!(
                "Fetching chunk: {} to {} ({} days)",
                from_str,
                to_str,
                (current_to - current_from).num_days()
            );

            match self
                .client
                .get_aggregates(ticker, multiplier, timespan.as_str(), &from_str, &to_str)
                .await
            {
                Ok(aggregates) => {
                    let candles: Vec<Candle> = aggregates
                        .into_iter()
                        .map(|agg| self.aggregate_to_candle(agg, ticker, resolution))
                        .collect();

                    info!("Fetched {} candles for chunk", candles.len());
                    all_candles.extend(candles);
                }
                Err(e) => {
                    error!("Failed to fetch chunk {} to {}: {}", from_str, to_str, e);
                    return Err(e);
                }
            }

            // Move to next chunk
            current_from = current_to + chrono::Duration::days(1);

            // Small delay between chunks to be nice to the API
            if current_from <= to {
                tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
            }
        }

        info!(
            "Completed fetching {} total candles for {} ({})",
            all_candles.len(),
            ticker,
            resolution
        );

        Ok(all_candles)
    }

    /// Calculate optimal chunk size in days based on resolution
    /// Goal: Stay well under 50k bar limit per request
    fn calculate_chunk_days(&self, resolution: &str) -> i32 {
        match resolution {
            "1m" => {
                // 1m: ~390 bars/day (6.5 hours trading) → 7 days = ~2,730 bars
                7
            }
            "5m" => {
                // 5m: ~78 bars/day → 30 days = ~2,340 bars
                30
            }
            "15m" => {
                // 15m: ~26 bars/day → 90 days = ~2,340 bars
                90
            }
            "30m" => {
                // 30m: ~13 bars/day → 180 days = ~2,340 bars
                180
            }
            "1h" => {
                // 1h: ~6.5 bars/day → 365 days = ~2,372 bars
                365
            }
            "4h" => {
                // 4h: ~1.6 bars/day → 1000 days = ~1,600 bars
                1000
            }
            "1d" => {
                // 1d: ~1 bar/day → 10000 days = ~10,000 bars
                10000
            }
            "1w" => {
                // 1w: ~0.2 bars/day → 50000 days (no chunking needed)
                50000
            }
            _ => {
                // Default conservative chunk
                30
            }
        }
    }

    /// Convert Polygon aggregate to Candle model
    fn aggregate_to_candle(&self, agg: AggregateBar, ticker: &str, resolution: &str) -> Candle {
        let time = Utc
            .timestamp_millis_opt(agg.timestamp)
            .single()
            .unwrap_or_else(|| Utc::now());

        Candle {
            exchange: "Polygon".to_string(),
            symbol: ticker.to_string(),
            resolution: resolution.to_string(),
            open: Decimal::from_f64(agg.open).unwrap_or_default(),
            high: Decimal::from_f64(agg.high).unwrap_or_default(),
            low: Decimal::from_f64(agg.low).unwrap_or_default(),
            close: Decimal::from_f64(agg.close).unwrap_or_default(),
            volume: Decimal::from_f64(agg.volume).unwrap_or_default(),
            amount: None,
            liquidity: None,
            fdv: None,
            metadata: agg.vwap.map(|vwap| {
                serde_json::json!({
                    "vwap": vwap,
                    "transactions": agg.transactions
                })
            }),
            time,
        }
    }

    /// Placeholder for WebSocket connection (Phase 3)
    pub async fn connect(
        &self,
    ) -> Result<
        tokio::sync::mpsc::Receiver<crate::models::StandardMarketData>,
        Box<dyn Error + Send + Sync>,
    > {
        let (_tx, rx) = tokio::sync::mpsc::channel(100);

        // TODO: Implement WebSocket connection in Phase 3
        tokio::spawn(async move {
            info!("Polygon WebSocket connector: Not yet implemented (Phase 3)");

            // Keep task alive
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
            }
        });

        Ok(rx)
    }

    pub async fn disconnect(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aggregate_to_candle() {
        let config = PolygonConfig {
            api_key: "test".to_string(),
            rest_base_url: "https://api.polygon.io".to_string(),
            ws_url: "wss://socket.polygon.io/stocks".to_string(),
            rate_limit_per_sec: 5,
            retry_max_attempts: 3,
            retry_delay_ms: 1000,
            ws_enabled: false,
        };

        let connector = PolygonConnector::new(config);

        let agg = AggregateBar {
            open: 150.25,
            high: 150.75,
            low: 150.10,
            close: 150.50,
            volume: 1000000.0,
            vwap: Some(150.45),
            timestamp: 1704067200000, // 2024-01-01 00:00:00 UTC
            transactions: Some(5000),
        };

        let candle = connector.aggregate_to_candle(agg, "AAPL", "1h");

        assert_eq!(candle.symbol, "AAPL");
        assert_eq!(candle.exchange, "Polygon");
        assert_eq!(candle.resolution, "1h");
        assert_eq!(candle.open, Decimal::from_f64(150.25).unwrap());
        assert_eq!(candle.close, Decimal::from_f64(150.50).unwrap());
        assert!(candle.metadata.is_some());
    }
}
