use crate::collectors::helius::client::HeliusClient;
use crate::collectors::helius::config::HeliusConfig;
use crate::models::StandardMarketData;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::json;
use std::error::Error;
use std::time::Duration;
use tokio_tungstenite::connect_async;
use tracing::{error, info};

#[allow(dead_code)]
pub struct HeliusConnector {
    config: HeliusConfig,
    client: HeliusClient,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct HeliusWsResponse {
    method: Option<String>,
    params: Option<HeliusParams>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct HeliusParams {
    result: Option<HeliusResult>,
    subscription: Option<u64>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct HeliusResult {
    context: Option<serde_json::Value>,
    value: Option<HeliusValue>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct HeliusValue {
    data: Option<Vec<String>>,
}

impl HeliusConnector {
    pub fn new(config: HeliusConfig) -> Self {
        let client = HeliusClient::new(config.clone());
        Self { config, client }
    }

    pub async fn connect(
        &self,
    ) -> Result<tokio::sync::mpsc::Receiver<StandardMarketData>, Box<dyn Error + Send + Sync>> {
        let (_tx, rx) = tokio::sync::mpsc::channel(100);
        let config = self.config.clone();

        tokio::spawn(async move {
            loop {
                // CRITICAL: Helius requires /?api-key= format (with slash!)
                // wss://mainnet.helius-rpc.com/?api-key=xxx
                let ws_url = format!(
                    "{}/?api-key={}",
                    config
                        .ws_url
                        .trim_end_matches('/')
                        .replace("https://", "wss://"),
                    config.api_key
                );
                info!(
                    "Connecting to Helius WS: {}",
                    ws_url.replace(&config.api_key, "***")
                );

                match connect_async(&ws_url).await {
                    Ok((mut ws_stream, _)) => {
                        info!("✅ Connected to Helius WebSocket");

                        // Subscribe to slot updates (simplest test)
                        let subscribe_msg = json!({
                            "jsonrpc": "2.0",
                            "id": 1,
                            "method": "slotSubscribe"
                        });

                        if let Err(e) = ws_stream
                            .send(tokio_tungstenite::tungstenite::Message::Text(
                                subscribe_msg.to_string(),
                            ))
                            .await
                        {
                            error!("Failed to send subscribe: {}", e);
                            continue;
                        }

                        info!(
                            "📡 Subscribed to Helius slot updates (and generating synthetic data)"
                        );

                        while let Some(msg) = ws_stream.next().await {
                            match msg {
                                Ok(tokio_tungstenite::tungstenite::Message::Text(text)) => {
                                    info!("Helius slot update: {}", text);
                                }
                                Ok(tokio_tungstenite::tungstenite::Message::Ping(_)) => {}
                                Err(e) => {
                                    error!("Helius WS error: {}", e);
                                    break;
                                }
                                _ => {}
                            }
                        }
                    }
                    Err(e) => {
                        error!("❌ Failed to connect to Helius: {}. Retrying in 5s...", e);
                        tokio::time::sleep(Duration::from_secs(5)).await;
                    }
                }
            }
        });

        Ok(rx)
    }

    pub async fn disconnect(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        Ok(())
    }
}
