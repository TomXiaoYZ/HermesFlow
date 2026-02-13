use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use rust_decimal::Decimal;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::{debug, error, info, warn};
use url::Url;

use crate::error::Result;
use crate::models::{AssetType, DataSourceType, MarketDataType, StandardMarketData};
use crate::traits::ConnectorStats;

pub struct MassiveStreamer {
    api_key: String,
    url: String,
    stats: Arc<RwLock<ConnectorStats>>,
}

impl MassiveStreamer {
    pub fn new(api_key: String, url: String) -> Self {
        Self {
            api_key,
            url,
            stats: Arc::new(RwLock::new(ConnectorStats::default())),
        }
    }

    pub fn with_stats(api_key: String, url: String, stats: Arc<RwLock<ConnectorStats>>) -> Self {
        Self {
            api_key,
            url,
            stats,
        }
    }

    pub async fn connect(&self) -> Result<mpsc::Receiver<StandardMarketData>> {
        let (tx, rx) = mpsc::channel(10000);
        let api_key = self.api_key.clone();
        let url_str = self.url.clone();
        let stats = self.stats.clone();

        tokio::spawn(async move {
            let mut backoff = 1;
            loop {
                info!("Connecting to Massive WebSocket: {}", url_str);
                let url = Url::parse(&url_str).expect("Invalid URL");

                match connect_async(url).await {
                    Ok((ws_stream, _)) => {
                        info!("Connected to Massive WebSocket");
                        let (mut write, mut read) = ws_stream.split();

                        // 1. Authenticate
                        let auth_msg = serde_json::json!({
                            "action": "auth",
                            "params": api_key
                        });
                        if let Err(e) = write.send(Message::Text(auth_msg.to_string())).await {
                            error!("Failed to send auth: {}", e);
                            tokio::time::sleep(std::time::Duration::from_secs(backoff)).await;
                            continue;
                        }

                        // 2. Subscribe (Wait for auth success ideally, but Polygon allows pipelining sometimes,
                        // strictly we should wait for status.auth_success)
                        // For simplicity, we assume auth works or we catch error in loop
                        let sub_msg = serde_json::json!({
                            "action": "subscribe",
                            "params": "A.*" // Subscribe to all Second Aggregates
                        });
                        if let Err(e) = write.send(Message::Text(sub_msg.to_string())).await {
                            error!("Failed to send subscription: {}", e);
                        } else {
                            info!("Sent subscription request for A.*");
                        }

                        backoff = 1; // Reset backoff on successful connection logic entry

                        // 3. Read Loop
                        const READ_TIMEOUT: std::time::Duration =
                            std::time::Duration::from_secs(60);
                        loop {
                            let msg_opt =
                                match tokio::time::timeout(READ_TIMEOUT, read.next()).await {
                                    Ok(Some(msg_res)) => msg_res,
                                    Ok(None) => {
                                        warn!("Massive WebSocket stream ended");
                                        break;
                                    }
                                    Err(_) => {
                                        warn!(
                                        "Massive WebSocket read timed out after {}s, reconnecting",
                                        READ_TIMEOUT.as_secs()
                                    );
                                        break;
                                    }
                                };
                            match msg_opt {
                                Ok(Message::Text(text)) => {
                                    // Handle Pings/Heartbeats if any? Polygon sends generic messages
                                    if let Ok(values) = serde_json::from_str::<Vec<Value>>(&text) {
                                        for value in values {
                                            Self::process_message(value, &tx).await;
                                        }
                                        // Track stats for the batch
                                        let mut s = stats.write().await;
                                        s.messages_received += 1;
                                        s.last_message_at = Some(std::time::SystemTime::now());
                                    } else {
                                        debug!("Received non-array message: {}", text);
                                    }
                                }
                                Ok(Message::Ping(_)) => {
                                    // Tungstenite handles pong automatically usually
                                }
                                Ok(Message::Close(_)) => {
                                    warn!("Massive WebSocket closed");
                                    break;
                                }
                                Err(e) => {
                                    error!("WebSocket read error: {}", e);
                                    let mut s = stats.write().await;
                                    s.errors += 1;
                                    break;
                                }
                                _ => {}
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to connect to Massive WebSocket: {}", e);
                    }
                }

                // Reconnect logic
                info!("Reconnecting in {} seconds...", backoff);
                tokio::time::sleep(std::time::Duration::from_secs(backoff)).await;
                backoff = std::cmp::min(backoff * 2, 60);
            }
        });

        Ok(rx)
    }

    async fn process_message(value: Value, tx: &mpsc::Sender<StandardMarketData>) {
        // Check event type "ev"
        if let Some(ev) = value.get("ev").and_then(|v| v.as_str()) {
            match ev {
                "A" => {
                    // Aggregate (Second)
                    if let Some(data) = Self::parse_aggregate(value.clone()) {
                        if let Err(e) = tx.send(data).await {
                            warn!("Failed to send market data to channel: {}", e);
                        }
                    }
                }
                "status" => {
                    info!("Status message: {:?}", value);
                }
                _ => {
                    debug!("Unhandled event type: {}", ev);
                }
            }
        }
    }

    fn parse_aggregate(value: Value) -> Option<StandardMarketData> {
        // Schema: { "ev": "A", "sym": "MSFT", "v": 100, "o": 100.1, "c": 100.2, "h": 100.3, "l": 100.0, "vw": 100.15, "s": 1600000000000, "e": ... }
        let symbol = value.get("sym")?.as_str()?.to_string();
        let volume = value.get("v")?.as_f64()?;
        let close = value.get("c")?.as_f64()?;
        let high = value.get("h")?.as_f64()?;
        let low = value.get("l")?.as_f64()?;
        let _open = value.get("o")?.as_f64()?;
        let timestamp_ms = value.get("s")?.as_i64()?; // Start timestamp

        let price = Decimal::from_f64_retain(close).unwrap_or_default();
        let quantity = Decimal::from_f64_retain(volume).unwrap_or_default();

        // Construct StandardMarketData
        // We map "A" (Aggregate) to MarketDataType::Kline or Candle if we had it, but StandardMarketData
        // is designed for Ticks/Snapshots mostly. We can map it as a "Trade" (Ticker) style update
        // representing the close of that second.

        Some(StandardMarketData {
            source: DataSourceType::PolygonStock,
            exchange: "Polygon".to_string(),
            symbol,
            asset_type: AssetType::Spot,
            data_type: MarketDataType::Ticker, // Using Ticker to flow through existing pipelines easily
            price,
            quantity,
            timestamp: timestamp_ms,
            received_at: Utc::now().timestamp_millis(),
            bid: None,
            ask: None,
            volume_24h: None, // This is 1-sec volume
            high_24h: Some(Decimal::from_f64_retain(high).unwrap_or_default()),
            low_24h: Some(Decimal::from_f64_retain(low).unwrap_or_default()),
            open_interest: None,
            funding_rate: None,
            liquidity: None,
            fdv: None,
            sequence_id: None,
            raw_data: value.to_string(),
        })
    }
}
