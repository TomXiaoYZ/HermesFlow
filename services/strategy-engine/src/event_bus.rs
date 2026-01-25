use common::events::{MarketDataUpdate, StrategyLog, TradeSignal, PortfolioUpdate};

use anyhow::Result;
use redis::Commands;
use std::thread;
use tokio::sync::mpsc;

pub struct EventBus {
    client: redis::Client,
}

impl EventBus {
    pub fn new(redis_url: &str) -> Result<Self> {
        let client = redis::Client::open(redis_url)?;
        Ok(Self { client })
    }

    pub async fn publish_signal(&self, signal: &TradeSignal) -> Result<()> {
        let client = self.client.clone();
        let signal_clone = signal.clone();

        // Use blocking task for Redis publish to avoid blocking async runtime
        tokio::task::spawn_blocking(move || -> Result<()> {
            let mut conn = client.get_connection()?;
            let json = serde_json::to_string(&signal_clone)?;
            let _: () = conn.publish("trade_signals", json)?;
            Ok(())
        })
        .await??;

        Ok(())
    }

    pub async fn publish_strategy_log(&self, log: &StrategyLog) -> Result<()> {
        let client = self.client.clone();
        let log_clone = log.clone();

        tokio::task::spawn_blocking(move || -> Result<()> {
            let mut conn = client.get_connection()?;
            let json = serde_json::to_string(&log_clone)?;
            let _: () = conn.publish("strategy_logs", json)?;
            Ok(())
        })
        .await??;
        Ok(())
    }

    pub async fn subscribe_market_data(
        &self,
        channel_name: &str,
    ) -> Result<mpsc::Receiver<MarketDataUpdate>> {
        let (tx, rx) = mpsc::channel(1000);
        let client = self.client.clone();
        let channel_name = channel_name.to_string();

        // Spawn a blocking thread for the Redis subscription loop
        thread::spawn(move || {
            match client.get_connection() {
                Ok(mut conn) => {
                    let mut pubsub = conn.as_pubsub();
                    if let Err(e) = pubsub.subscribe(&channel_name) {
                        tracing::error!("Failed to subscribe to {}: {}", channel_name, e);
                        return;
                    }

                    loop {
                        match pubsub.get_message() {
                            Ok(msg) => {
                                match msg.get_payload::<String>() {
                                    Ok(payload) => {
                                        match serde_json::from_str::<MarketDataUpdate>(&payload) {
                                            Ok(data) => {
                                                if let Err(_) = tx.blocking_send(data) {
                                                    // Receiver dropped, exit loop
                                                    break;
                                                }
                                            }
                                            Err(e) => tracing::error!(
                                                "Failed to deserialize market data: {}",
                                                e
                                            ),
                                        }
                                    }
                                    Err(e) => tracing::error!("Failed to get payload: {}", e),
                                }
                            }
                            Err(e) => {
                                tracing::error!("Redis subscription error: {}", e);
                                // Simple backoff or break? Let's break for now and maybe we need reconnection logic later
                                break;
                            }
                        }
                    }
                }
                Err(e) => tracing::error!("Failed to connect to Redis for subscription: {}", e),
            }
        });

        Ok(rx)
    }

    pub async fn subscribe_portfolio_updates(
        &self,
        channel_name: &str,
    ) -> Result<mpsc::Receiver<PortfolioUpdate>> {
        let (tx, rx) = mpsc::channel(100);
        let client = self.client.clone();
        let channel_name = channel_name.to_string();

        thread::spawn(move || {
            match client.get_connection() {
                Ok(mut conn) => {
                    let mut pubsub = conn.as_pubsub();
                    if let Err(e) = pubsub.subscribe(&channel_name) {
                        tracing::error!("Failed to subscribe to {}: {}", channel_name, e);
                        return;
                    }

                    loop {
                        match pubsub.get_message() {
                            Ok(msg) => {
                                match msg.get_payload::<String>() {
                                    Ok(payload) => {
                                        match serde_json::from_str::<PortfolioUpdate>(&payload) {
                                            Ok(data) => {
                                                if let Err(_) = tx.blocking_send(data) {
                                                    break;
                                                }
                                            }
                                            Err(e) => tracing::error!("Failed to deserialize portfolio data: {}", e),
                                        }
                                    }
                                    Err(e) => tracing::error!("Failed to get payload: {}", e),
                                }
                            }
                            Err(e) => {
                                tracing::error!("Redis subscription error: {}", e);
                                break;
                            }
                        }
                    }
                }
                Err(e) => tracing::error!("Failed to connect to Redis for subscription: {}", e),
            }
        });

        Ok(rx)
    }
}
