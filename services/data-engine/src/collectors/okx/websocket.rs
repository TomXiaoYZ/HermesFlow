use super::config::OkxConfig;
use crate::error::{DataError, Result};
use crate::models::{AssetType, DataSourceType, MarketDataType, StandardMarketData};
use futures::{SinkExt, StreamExt};
use rust_decimal::Decimal;
use serde_json::Value;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{error, info, warn};
use url::Url;

pub struct OkxStreamer {
    url: String,
    symbols: Vec<String>, // e.g., ["BTC-USDT", "ETH-USDT"]
}

impl OkxStreamer {
    pub fn new(url: String, symbols: Vec<String>) -> Self {
        Self { url, symbols }
    }

    pub async fn connect(&self) -> Result<mpsc::Receiver<StandardMarketData>> {
        let (tx, rx) = mpsc::channel(10000);
        let url_str = self.url.clone();
        let symbols = self.symbols.clone();

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
                        // But wait, OKX sends plain string "ping" in some versions or frame?
                        // "server will send a ping message every 30s... client needs to reply pong"
                        // If it's a Text frame with content "ping", we reply "pong".

                        loop {
                            tokio::select! {
                                msg_res = read.next() => {
                                    match msg_res {
                                        Some(Ok(Message::Text(text))) => {
                                            if text == "ping" {
                                                if let Err(e) = write.send(Message::Text("pong".to_string())).await {
                                                    error!("Failed to send pong: {}", e);
                                                    break;
                                                }
                                                continue;
                                            }

                                            // Handle data
                                            if let Ok(value) = serde_json::from_str::<Value>(&text) {
                                                // Check for "data" field
                                                if let Some(data_array) = value.get("data").and_then(|v| v.as_array()) {
                                                     for d in data_array {
                                                         if let Some(md) = Self::parse_data(d) {
                                                             if tx.send(md).await.is_err() {
                                                                 return; // Receiver dropped
                                                             }
                                                         }
                                                     }
                                                }
                                                // Handle error/event status?
                                            }
                                        }
                                        Some(Ok(Message::Ping(_))) => {} // Tungstenite auto-pong
                                        Some(Ok(Message::Close(_))) => {
                                            warn!("OKX WebSocket closed");
                                            break;
                                        }
                                        Some(Err(e)) => {
                                            error!("WebSocket error: {}", e);
                                            break;
                                        }
                                        None => break,
                                        _ => {}
                                    }
                                }
                                // Implement keepalive/heartbeat sender if needed?
                                // OKX server initiates ping usually.
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
