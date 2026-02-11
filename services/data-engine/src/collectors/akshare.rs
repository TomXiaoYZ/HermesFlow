use async_trait::async_trait;
use chrono::Utc;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

use crate::error::{DataError, Result};
use crate::models::{AssetType, DataSourceType, MarketDataType, StandardMarketData};
use crate::traits::{ConnectorStats, DataSourceConnector};

use crate::config::AkShareConfig;
// use crate::error::{DataEngineError, Result}; // Already imported above

pub struct AkShareCollector {
    config: AkShareConfig,
    stats: ConnectorStats,
    running: bool,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct AkShareSpotResponse {
    #[serde(rename = "代码")]
    symbol: String,
    #[serde(rename = "名称")]
    name: String,
    #[serde(rename = "最新价")]
    price: Option<f64>,
    #[serde(rename = "涨跌幅")]
    change_percent: Option<f64>,
    #[serde(rename = "成交量")]
    volume: Option<f64>,
    #[serde(rename = "成交额")]
    turnover: Option<f64>,
    #[serde(rename = "最高")]
    high: Option<f64>,
    #[serde(rename = "最低")]
    low: Option<f64>,
    #[serde(rename = "今开")]
    open: Option<f64>,
    #[serde(rename = "昨收")]
    prev_close: Option<f64>,
}

impl AkShareCollector {
    pub fn new(config: AkShareConfig) -> Self {
        Self {
            config,
            stats: ConnectorStats::default(),
            running: false,
        }
    }

    #[allow(dead_code)]
    async fn fetch_snapshot(&self, client: &reqwest::Client) -> Result<Vec<StandardMarketData>> {
        let url = format!("{}/api/public/stock_zh_a_spot_em", self.config.aktools_url);

        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|e| DataError::ConnectionFailed {
                data_source: "AkShare".to_string(),
                reason: format!("Failed to fetch AkShare data: {}", e),
            })?;

        if !response.status().is_success() {
            return Err(DataError::ConnectionFailed {
                data_source: "AkShare".to_string(),
                reason: format!("AkShare API error: {}", response.status()),
            });
        }

        let raw_data: Vec<AkShareSpotResponse> =
            response.json().await.map_err(|e| DataError::ParseError {
                data_source: "AkShare".to_string(),
                message: format!("Failed to parse AkShare data: {}", e),
                raw_data: String::new(),
            })?;

        let mut market_data = Vec::with_capacity(raw_data.len());
        let now = Utc::now().timestamp_millis();

        for item in raw_data {
            if let Some(price) = item.price {
                // Skip invalid data
                if price <= 0.0 {
                    continue;
                }

                let price_dec = Decimal::from_f64_retain(price).unwrap_or_default();
                let volume_dec = item
                    .volume
                    .map(|v| Decimal::from_f64_retain(v).unwrap_or_default())
                    .unwrap_or_default();

                let high = item
                    .high
                    .map(|v| Decimal::from_f64_retain(v).unwrap_or_default());
                let low = item
                    .low
                    .map(|v| Decimal::from_f64_retain(v).unwrap_or_default());
                let _open = item
                    .open
                    .map(|v| Decimal::from_f64_retain(v).unwrap_or_default());
                let _prev_close = item
                    .prev_close
                    .map(|v| Decimal::from_f64_retain(v).unwrap_or_default());

                let mut data = StandardMarketData::new(
                    DataSourceType::AkShare,
                    item.symbol, // e.g., "600519"
                    AssetType::Spot,
                    MarketDataType::Ticker, // Use Ticker for snapshot
                    price_dec,
                    volume_dec,
                    now,
                );

                data.high_24h = high;
                data.low_24h = low;
                data.raw_data = item.name; // Store Chinese name in raw_data or elsewhere? Using raw_data for now.

                // Enrich with valid metadata if we want
                // data.bid/ask are not available in this specific bulk endpoint usually,
                // unless we query individual stocks.

                market_data.push(data);
            }
        }

        Ok(market_data)
    }
}

#[async_trait]
impl DataSourceConnector for AkShareCollector {
    fn source_type(&self) -> DataSourceType {
        DataSourceType::AkShare
    }

    fn supported_assets(&self) -> Vec<AssetType> {
        vec![AssetType::Spot]
    }

    async fn connect(&mut self) -> Result<mpsc::Receiver<StandardMarketData>> {
        self.running = true;
        let (tx, rx) = mpsc::channel(10000); // Large buffer for full market snapshot
        let config = self.config.clone();

        tokio::spawn(async move {
            let client = reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap_or_default();

            let mut interval =
                tokio::time::interval(Duration::from_secs(config.poll_interval_secs));

            info!(
                "AkShare collector started polling every {}s",
                config.poll_interval_secs
            );

            loop {
                interval.tick().await;

                // We create a new collector instance just to use the fetch method logic,
                // or refactor fetch_snapshot to be static/associated?
                // For simplicity, we implement logic here or create a temporary struct.
                // Let's refactor: fetch_snapshot can be associated function if we pass URL.

                let url = format!("{}/api/public/stock_zh_a_spot_em", config.aktools_url);
                match client.get(&url).send().await {
                    Ok(resp) => {
                        if resp.status().is_success() {
                            match resp.json::<Vec<AkShareSpotResponse>>().await {
                                Ok(data) => {
                                    let now = Utc::now().timestamp_millis();
                                    let mut count = 0;
                                    for item in data {
                                        if let Some(price) = item.price {
                                            if price <= 0.0 {
                                                continue;
                                            }

                                            let price_dec =
                                                Decimal::from_f64_retain(price).unwrap_or_default();
                                            let volume_dec = item
                                                .volume
                                                .map(|v| {
                                                    Decimal::from_f64_retain(v).unwrap_or_default()
                                                })
                                                .unwrap_or_default();

                                            // Optional fields
                                            let high = item.high.map(|v| {
                                                Decimal::from_f64_retain(v).unwrap_or_default()
                                            });
                                            let low = item.low.map(|v| {
                                                Decimal::from_f64_retain(v).unwrap_or_default()
                                            });

                                            let mut mk_data = StandardMarketData::new(
                                                DataSourceType::AkShare,
                                                item.symbol,
                                                AssetType::Spot,
                                                MarketDataType::Ticker,
                                                price_dec,
                                                volume_dec,
                                                now,
                                            );
                                            mk_data.high_24h = high;
                                            mk_data.low_24h = low;
                                            mk_data.raw_data = item.name;

                                            if tx.send(mk_data).await.is_err() {
                                                warn!("AkShare receiver dropped");
                                                return;
                                            }
                                            count += 1;
                                        }
                                    }
                                    info!("Fetched {} A-Share symbols", count);
                                }
                                Err(e) => error!("Failed to parse AkShare data: {}", e),
                            }
                        } else {
                            error!("AkShare API error: {}", resp.status());
                        }
                    }
                    Err(e) => error!("Failed to fetch AkShare data: {}", e),
                }
            }
        });

        Ok(rx)
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.running = false;
        Ok(())
    }

    async fn is_healthy(&self) -> bool {
        self.running
    }

    fn stats(&self) -> ConnectorStats {
        self.stats.clone()
    }
}
