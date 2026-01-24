use super::config::BinanceConfig;
use crate::error::{DataError, Result};
use crate::models::{AssetType, DataSourceType, MarketDataType, StandardMarketData};
use chrono::Utc;
use futures::{SinkExt, StreamExt};
use rust_decimal::Decimal;
use serde_json::Value;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tokio_tungstenite::tungstenite::Message;
use tracing::{error, info, warn};
use url::Url;

pub struct BinanceStreamer {
    url: String,
    symbols: Vec<String>,
}

impl BinanceStreamer {
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
                // Construct stream URL with params for immediate subscription
                // e.g. wss://stream.binance.com:9443/ws/btcusdt@aggTrade
                // Alternatively, connect to /ws and send SUBSCRIBE message.
                // Using /stream?streams=... allows multiplexing easily.
                
                // Format: lowercase symbols, append @aggTrade
                let streams: Vec<String> = symbols
                    .iter()
                    .map(|s| format!("{}@aggTrade", s.to_lowercase()))
                    .collect();
                
                let stream_query = streams.join("/");
                // Using combined streams endpoint: wss://stream.binance.com:9443/stream?streams=...
                // Only if url base ends with /stream, otherwise we assume /ws and send subscribe
                
                // For simplicity + robustness with custom config, we'll use /ws and SUBSCRIBE payload
                
                info!("Connecting to Binance WebSocket: {}", url_str);
                match connect_async(&url_str).await {
                    Ok((mut ws_stream, _)) => {
                        info!("Connected to Binance WebSocket");
                        // Publish to Redis
                            // Ensure the payload matches services/common/src/events.rs:MarketDataUpdate
                            // This block seems to be misplaced or incomplete as it refers to `normalized_symbol`, `price`, `quantity`, `event_time`, `redis_conn`
                            // which are not defined in this scope.
                            // Also, the `tokio::time::sleep` and `continue` are outside any conditional block.
                            // To make it syntactically correct, I'm commenting it out as it cannot be directly inserted.
                            /*
                            let payload = serde_json::json!({
                                "symbol": normalized_symbol,
                                "price": price,
                                "quantity": quantity,
                                "timestamp": event_time,
                                "source": "BinanceSpot", 
                                "data_type": "Trade" // Using Trade type for individual trade updates
                            });
                            
                            // Use standard market_data channel
                            if let Err(e) = redis_conn.publish::<_, _, ()>("market_data", payload.to_string()).await {
                                error!("Failed to publish trade: {}", e);
                            } tokio::time::sleep(tokio::time::Duration::from_secs(backoff)).await;
                             continue;
                        }
                            */
                        
                        // Subscribe
                        let subscribe_msg = serde_json::json!({
                            "method": "SUBSCRIBE",
                            "params": streams,
                            "id": Utc::now().timestamp_millis()
                        });
                        
                        if let Err(e) = ws_stream.send(Message::Text(subscribe_msg.to_string())).await {
                             error!("Failed to send subscribe: {}", e);
                             tokio::time::sleep(tokio::time::Duration::from_secs(backoff)).await;
                             continue;
                        }

                        backoff = 1;
                        
                        let (mut write, mut read) = ws_stream.split();
                        
                        // Keep-alive/Ping is handled by tungstenite protocol level usually for ping/pong frames
                        // Binance requires PONG response to Ping? 
                        // "The websocket server will send a ping frame every 3 minutes. If the websocket server does not receive a pong frame back from the connection within a 10 minute period, the connection will be disconnected."
                        // Tungstenite handles this by default (autoreply).

                        while let Some(msg_res) = read.next().await {
                            match msg_res {
                                Ok(Message::Text(text)) => {
                                    if let Ok(value) = serde_json::from_str::<Value>(&text) {
                                        // Handle Ping (sometimes sent as text in some exchanges, strict ping frame in others)
                                        // Binance uses standard frames, but payload might contain error/result
                                        if value.get("e").is_some() {
                                             if let Some(data) = Self::parse_message(value) {
                                                 if let Err(_) = tx.send(data).await {
                                                     break; // Receiver dropped, exit
                                                 }
                                             }
                                        }
                                    }
                                }
                                Ok(Message::Ping(_)) => {} // Auto-pong
                                Ok(Message::Close(_)) => {
                                    warn!("Binance WebSocket closed");
                                    break;
                                }
                                Err(e) => {
                                    error!("WebSocket error: {}", e);
                                    break;
                                }
                                _ => {}
                            }
                        }
                    }
                    Err(e) => error!("Failed to connect to Binance: {}", e),
                }

                warn!("Reconnecting to Binance in {} seconds...", backoff);
                tokio::time::sleep(tokio::time::Duration::from_secs(backoff)).await;
                backoff = std::cmp::min(backoff * 2, 60);
            }
        });

        Ok(rx)
    }

    fn parse_message(value: Value) -> Option<StandardMarketData> {
        // Handle "aggTrade"
        // { "e": "aggTrade", "s": "BTCUSDT", "p": "0.001", "q": "100", "T": 123456785, ... }
        
        // Note: When using /stream?streams=..., payload is wrapped in {"stream":"...", "data":...}
        // Since we simple /ws and SUBSCRIBE, payloads are direct event objects.
        
        let event_type = value.get("e")?.as_str()?;
        
        if event_type == "aggTrade" {
            let symbol = value.get("s")?.as_str()?.to_string();
            let price_str = value.get("p")?.as_str()?;
            let qty_str = value.get("q")?.as_str()?;
            let timestamp = value.get("T")?.as_i64()?;

            use rust_decimal::prelude::FromPrimitive;
            let price = Decimal::from_str_radix(price_str, 10).ok()?;
            let quantity = Decimal::from_str_radix(qty_str, 10).ok()?;

            return Some(StandardMarketData {
                source: DataSourceType::BinanceSpot, // Could be Futures if configured, hardcode Spot for now
                exchange: "Binance".to_string(),
                symbol,
                asset_type: AssetType::Crypto,
                data_type: MarketDataType::Trade,
                price,
                quantity,
                timestamp,
                received_at: Utc::now().timestamp_millis(),
                bid: None,
                ask: None,
                volume_24h: None, // Not in aggTrade
                high_24h: None,
                low_24h: None,
                open_interest: None,
                funding_rate: None,
                liquidity: None,
                fdv: None,
                sequence_id: value.get("a").and_then(|v| v.as_u64()), // AggTradeId
                raw_data: value.to_string(),
            });
        }
        
        None
    }
}
