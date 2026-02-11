use super::client::FutuClient;
use super::config::FutuConfig;
use crate::error::Result;
use crate::models::{AssetType, DataSourceType, StandardMarketData};
use crate::traits::{ConnectorStats, DataSourceConnector};
use async_trait::async_trait;
use tokio::sync::mpsc;
use tracing::warn;

#[allow(dead_code)]
pub struct FutuConnector {
    config: FutuConfig,
    client: FutuClient,
    stats: ConnectorStats,
    running: bool,
}

impl FutuConnector {
    pub fn new(config: FutuConfig) -> Self {
        let client = FutuClient::new(config.clone());
        Self {
            config,
            client,
            stats: ConnectorStats::default(),
            running: false,
        }
    }
}

#[async_trait]
impl DataSourceConnector for FutuConnector {
    fn source_type(&self) -> DataSourceType {
        DataSourceType::FutuStock
    }

    fn supported_assets(&self) -> Vec<AssetType> {
        vec![AssetType::Stock]
    }

    async fn connect(&mut self) -> Result<mpsc::Receiver<StandardMarketData>> {
        self.running = true;
        let (_tx, _rx) = mpsc::channel(100);

        // Placeholder loop: Try to connect but essentially do nothing for now
        // since we lack Protobuf definitions to decode actual messages.

        warn!("Futu Connector started in Placeholder mode (Waiting for Protobuf definitions)");

        if let Err(e) = self.client.connect().await {
            warn!("Futu Connection failed/postponed: {}", e);
        }

        Ok(_rx)
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.running = false;
        Ok(())
    }

    async fn is_healthy(&self) -> bool {
        self.running && self.client.is_connected()
    }

    fn stats(&self) -> ConnectorStats {
        self.stats.clone()
    }
}
