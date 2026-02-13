use crate::error::Result;
use crate::models::{AssetType, DataSourceType, MarketDataType, StandardMarketData};
use futures::{SinkExt, StreamExt};
use rust_decimal::Decimal;
use serde_json::Value;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{error, info, warn};

pub struct BybitStreamer {
    url: String,
    symbols: Vec<String>,
}

impl BybitStreamer {
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
                info!("Connecting to Bybit WebSocket: {}", url_str);
                match connect_async(&url_str).await {
                    Ok((mut ws_stream, _)) => {
                        info!("Connected to Bybit WebSocket");

                        // Subscribe to public trade channel
                        // "op": "subscribe", "args": ["publicTrade.BTCUSDT", ...]
                        let args: Vec<String> = symbols
                            .iter()
                            .map(|s| format!("publicTrade.{}", s))
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

                        // Bybit heartbeat: Send {"op": "ping"} every 20s
                        let (mut write, mut read) = ws_stream.split();
                        let mut ping_interval =
                            tokio::time::interval(tokio::time::Duration::from_secs(20));

                        loop {
                            tokio::select! {
                                _ = ping_interval.tick() => {
                                    let ping = serde_json::json!({"op": "ping"});
                                    if let Err(e) = write.send(Message::Text(ping.to_string())).await {
                                        error!("Failed to send ping: {}", e);
                                        break;
                                    }
                                }
                                msg_res = read.next() => {
                                    match msg_res {
                                        Some(Ok(Message::Text(text))) => {
                                            if let Ok(value) = serde_json::from_str::<Value>(&text) {
                                                // Check for "topic" and "data"
                                                if let Some(topic) = value.get("topic").and_then(|v| v.as_str()) {
                                                    if topic.starts_with("publicTrade") {
                                                        if let Some(data_array) = value.get("data").and_then(|v| v.as_array()) {
                                                             for d in data_array {
                                                                 if let Some(md) = Self::parse_data(d) {
                                                                     if tx.send(md).await.is_err() {
                                                                         return;
                                                                     }
                                                                 }
                                                             }
                                                        }
                                                    }
                                                }
                                                // Handle op response: {"success": true, "ret_msg": "subscribe", ...}
                                            }
                                        }
                                        Some(Ok(Message::Close(_))) => {
                                            warn!("Bybit WebSocket closed");
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
                            }
                        }
                    }
                    Err(e) => error!("Failed to connect to Bybit: {}", e),
                }

                warn!("Reconnecting to Bybit in {} seconds...", backoff);
                tokio::time::sleep(tokio::time::Duration::from_secs(backoff)).await;
                backoff = std::cmp::min(backoff * 2, 60);
            }
        });

        Ok(rx)
    }

    fn parse_data(d: &Value) -> Option<StandardMarketData> {
        // {"T":1672304486868,"s":"BTCUSDT","S":"Buy","v":"0.001","p":"16578.50", "i": "..."}
        let symbol = d.get("s")?.as_str()?;
        let price_str = d.get("p")?.as_str()?;
        let volume_str = d.get("v")?.as_str()?;
        let timestamp = d.get("T")?.as_i64()?;

        use rust_decimal::prelude::FromStr;
        let price = Decimal::from_str(price_str).ok()?;
        let quantity = Decimal::from_str(volume_str).ok()?;

        Some(StandardMarketData {
            source: DataSourceType::BybitSpot,
            // NO, I need to add Bybit type.
            // I'll add BybitSpot/Futures to data_source_type.rs first.
            // For now, I'll assume Spot/Unknown.
            exchange: "Bybit".to_string(),
            symbol: symbol.to_string(),
            asset_type: AssetType::Crypto,
            data_type: MarketDataType::Trade,
            price,
            quantity,
            timestamp,
            received_at: chrono::Utc::now().timestamp_millis(),
            bid: None,
            bid_size: None,
            ask: None,
            ask_size: None,
            volume_24h: None,
            high_24h: None,
            low_24h: None,
            open_interest: None,
            funding_rate: None,
            liquidity: None,
            fdv: None,
            sequence_id: d
                .get("i")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse().ok()), // Trade ID
            raw_data: d.to_string(),
        })
    }
}
