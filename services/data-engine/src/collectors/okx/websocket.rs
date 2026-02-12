use crate::error::Result;
use crate::models::{AssetType, DataSourceType, MarketDataType, StandardMarketData};
use crate::traits::ConnectorStats;
use futures::{SinkExt, StreamExt};
use rust_decimal::Decimal;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{error, info, warn};

pub struct OkxStreamer {
    url: String,
    symbols: Vec<String>, // e.g., ["BTC-USDT", "ETH-USDT"]
    stats: Arc<RwLock<ConnectorStats>>,
}

impl OkxStreamer {
    pub fn new(url: String, symbols: Vec<String>) -> Self {
        Self {
            url,
            symbols,
            stats: Arc::new(RwLock::new(ConnectorStats::default())),
        }
    }

    pub fn with_stats(
        url: String,
        symbols: Vec<String>,
        stats: Arc<RwLock<ConnectorStats>>,
    ) -> Self {
        Self {
            url,
            symbols,
            stats,
        }
    }

    pub async fn connect(&self) -> Result<mpsc::Receiver<StandardMarketData>> {
        let (tx, rx) = mpsc::channel(10000);
        let url_str = self.url.clone();
        let symbols = self.symbols.clone();
        let stats = self.stats.clone();

        tokio::spawn(async move {
            let mut backoff = 1;

            loop {
                info!("Connecting to OKX WebSocket: {}", url_str);
                match connect_async(&url_str).await {
                    Ok((mut ws_stream, _)) => {
                        info!("Connected to OKX WebSocket");

                        // Construct subscription channel args
                        // "args": [{"channel": "trades", "instId": "BTC-USDT"}, ...]
                        let args: Vec<Value> = symbols
                            .iter()
                            .map(|s| {
                                serde_json::json!({
                                    "channel": "trades",
                                    "instId": s
                                })
                            })
                            .collect();

                        let subscribe_msg = serde_json::json!({
                            "op": "subscribe",
                            "args": args
                        });

                        if let Err(e) = ws_stream
                            .send(Message::Text(subscribe_msg.to_string()))
                            .await
                        {
                            error!("Failed to send subscribe: {}", e);
                            tokio::time::sleep(tokio::time::Duration::from_secs(backoff)).await;
                            continue;
                        }

                        backoff = 1;
                        let (mut write, mut read) = ws_stream.split();

                        // OKX requires responding to "ping" with "pong" string
                        // OKX server sends ping every 30s; timeout at 60s to detect silent death.
                        const READ_TIMEOUT: std::time::Duration =
                            std::time::Duration::from_secs(60);

                        loop {
                            let msg_res =
                                match tokio::time::timeout(READ_TIMEOUT, read.next()).await {
                                    Ok(Some(msg_res)) => msg_res,
                                    Ok(None) => {
                                        warn!("OKX WebSocket stream ended");
                                        break;
                                    }
                                    Err(_) => {
                                        warn!(
                                            "OKX WebSocket read timed out after {}s, reconnecting",
                                            READ_TIMEOUT.as_secs()
                                        );
                                        break;
                                    }
                                };

                            match msg_res {
                                Ok(Message::Text(text)) => {
                                    if text == "ping" {
                                        if let Err(e) =
                                            write.send(Message::Text("pong".to_string())).await
                                        {
                                            error!("Failed to send pong: {}", e);
                                            break;
                                        }
                                        continue;
                                    }

                                    // Handle data
                                    if let Ok(value) = serde_json::from_str::<Value>(&text) {
                                        // Check for "data" field
                                        if let Some(data_array) =
                                            value.get("data").and_then(|v| v.as_array())
                                        {
                                            let count = data_array.len() as u64;
                                            for d in data_array {
                                                if let Some(md) = Self::parse_data(d) {
                                                    if tx.send(md).await.is_err() {
                                                        return; // Receiver dropped
                                                    }
                                                }
                                            }
                                            // Track stats
                                            let mut s = stats.write().await;
                                            s.messages_received += count;
                                            s.messages_processed += count;
                                            s.last_message_at = Some(std::time::SystemTime::now());
                                        }
                                    }
                                }
                                Ok(Message::Ping(_)) => {} // Tungstenite auto-pong
                                Ok(Message::Close(_)) => {
                                    warn!("OKX WebSocket closed");
                                    break;
                                }
                                Err(e) => {
                                    error!("WebSocket error: {}", e);
                                    let mut s = stats.write().await;
                                    s.errors += 1;
                                    break;
                                }
                                _ => {}
                            }
                        }
                    }
                    Err(e) => error!("Failed to connect to OKX: {}", e),
                }

                warn!("Reconnecting to OKX in {} seconds...", backoff);
                tokio::time::sleep(tokio::time::Duration::from_secs(backoff)).await;
                backoff = std::cmp::min(backoff * 2, 60);
            }
        });

        Ok(rx)
    }

    fn parse_data(d: &Value) -> Option<StandardMarketData> {
        // {"instId":"BTC-USDT","px":"20000","sz":"0.1","ts":"1630000000000", ...}
        let inst_id = d.get("instId")?.as_str()?;
        let px = d.get("px")?.as_str()?;
        let sz = d.get("sz")?.as_str()?;
        let ts = d.get("ts")?.as_str()?; // timestamp string

        use rust_decimal::prelude::FromStr;
        let price = Decimal::from_str(px).ok()?;
        let quantity = Decimal::from_str(sz).ok()?;
        let timestamp = ts.parse::<i64>().ok()?;

        Some(StandardMarketData {
            source: DataSourceType::OkxSpot, // Assume Spot for now
            exchange: "OKX".to_string(),
            symbol: inst_id.to_string(),
            asset_type: AssetType::Crypto,
            data_type: MarketDataType::Trade,
            price,
            quantity,
            timestamp,
            received_at: chrono::Utc::now().timestamp_millis(),
            bid: None,
            ask: None,
            volume_24h: None,
            high_24h: None,
            low_24h: None,
            open_interest: None,
            funding_rate: None,
            liquidity: None,
            fdv: None,
            sequence_id: d
                .get("tradeId")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse().ok()),
            raw_data: d.to_string(),
        })
    }
}
