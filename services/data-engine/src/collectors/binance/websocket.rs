use crate::error::Result;
use crate::models::{AssetType, DataSourceType, MarketDataType, StandardMarketData};
use chrono::Utc;
use futures::{SinkExt, StreamExt};
use rust_decimal::Decimal;
use serde_json::Value;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_tungstenite::client_async_tls_with_config;
use tokio_tungstenite::tungstenite::Message;
use tracing::{error, info, warn};
use url::Url;

pub struct BinanceStreamer {
    url: String,
    symbols: Vec<String>,
}

/// Connect a raw TCP stream, optionally tunneling through an HTTP CONNECT proxy.
/// Reads HTTPS_PROXY env var. Falls back to direct connection if unset.
async fn connect_tcp(
    host: &str,
    port: u16,
) -> std::result::Result<TcpStream, Box<dyn std::error::Error + Send + Sync>> {
    if let Ok(proxy_url) = std::env::var("HTTPS_PROXY") {
        if let Ok(proxy) = Url::parse(&proxy_url) {
            if proxy.scheme() == "http" || proxy.scheme() == "https" {
                let proxy_host = proxy.host_str().unwrap_or("127.0.0.1");
                let proxy_port = proxy.port().unwrap_or(7897);
                let proxy_addr = format!("{}:{}", proxy_host, proxy_port);

                info!(
                    "Connecting to {}:{} via HTTP proxy {}",
                    host, port, proxy_addr
                );
                let mut stream = TcpStream::connect(&proxy_addr).await?;

                // HTTP CONNECT tunnel
                let req = format!(
                    "CONNECT {}:{} HTTP/1.1\r\nHost: {}:{}\r\n\r\n",
                    host, port, host, port
                );
                stream.write_all(req.as_bytes()).await?;

                let mut buf = [0u8; 4096];
                let n = stream.read(&mut buf).await?;
                let response = String::from_utf8_lossy(&buf[..n]);
                if !response.contains("200") {
                    return Err(format!(
                        "Proxy CONNECT to {}:{} failed: {}",
                        host,
                        port,
                        response.trim()
                    )
                    .into());
                }

                return Ok(stream);
            }
        }
    }

    // Direct connection
    Ok(TcpStream::connect(format!("{}:{}", host, port)).await?)
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
                let streams: Vec<String> = symbols
                    .iter()
                    .map(|s| format!("{}@aggTrade", s.to_lowercase()))
                    .collect();

                info!("Connecting to Binance WebSocket: {}", url_str);

                // Parse target host and port from WebSocket URL
                let ws_result = match Url::parse(&url_str) {
                    Ok(parsed) => {
                        let host = parsed.host_str().unwrap_or("stream.binance.com");
                        let port = parsed.port().unwrap_or(if parsed.scheme() == "wss" {
                            443
                        } else {
                            80
                        });

                        match connect_tcp(host, port).await {
                            Ok(tcp_stream) => {
                                client_async_tls_with_config(&url_str, tcp_stream, None, None).await
                            }
                            Err(e) => {
                                error!("Failed to establish TCP connection: {}", e);
                                Err(tokio_tungstenite::tungstenite::Error::Io(
                                    std::io::Error::new(
                                        std::io::ErrorKind::ConnectionRefused,
                                        e.to_string(),
                                    ),
                                ))
                            }
                        }
                    }
                    Err(e) => {
                        error!("Invalid WebSocket URL: {}", e);
                        Err(tokio_tungstenite::tungstenite::Error::Url(
                            tokio_tungstenite::tungstenite::error::UrlError::NoHostName,
                        ))
                    }
                };

                match ws_result {
                    Ok((mut ws_stream, _)) => {
                        info!("Connected to Binance WebSocket");

                        // Subscribe
                        let subscribe_msg = serde_json::json!({
                            "method": "SUBSCRIBE",
                            "params": streams,
                            "id": Utc::now().timestamp_millis()
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

                        let (_write, mut read) = ws_stream.split();

                        while let Some(msg_res) = read.next().await {
                            match msg_res {
                                Ok(Message::Text(text)) => {
                                    if let Ok(value) = serde_json::from_str::<Value>(&text) {
                                        if value.get("e").is_some() {
                                            if let Some(data) = Self::parse_message(value) {
                                                if tx.send(data).await.is_err() {
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
        let event_type = value.get("e")?.as_str()?;

        if event_type == "aggTrade" {
            let symbol = value.get("s")?.as_str()?.to_string();
            let price_str = value.get("p")?.as_str()?;
            let qty_str = value.get("q")?.as_str()?;
            let timestamp = value.get("T")?.as_i64()?;

            let price = Decimal::from_str_radix(price_str, 10).ok()?;
            let quantity = Decimal::from_str_radix(qty_str, 10).ok()?;

            return Some(StandardMarketData {
                source: DataSourceType::BinanceSpot,
                exchange: "Binance".to_string(),
                symbol,
                asset_type: AssetType::Crypto,
                data_type: MarketDataType::Trade,
                price,
                quantity,
                timestamp,
                received_at: Utc::now().timestamp_millis(),
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
                sequence_id: value.get("a").and_then(|v| v.as_u64()),
                raw_data: value.to_string(),
            });
        }

        None
    }
}
