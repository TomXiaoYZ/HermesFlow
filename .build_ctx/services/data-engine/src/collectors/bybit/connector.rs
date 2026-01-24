use super::client::BybitClient;
use super::config::BybitConfig;
use super::websocket::BybitStreamer;
use crate::error::Result;
use crate::models::{AssetType, DataSourceType, StandardMarketData};
use crate::traits::{ConnectorStats, DataSourceConnector};
use async_trait::async_trait;
use tokio::sync::mpsc;

pub struct BybitConnector {
    config: BybitConfig,
    client: BybitClient,
    stats: ConnectorStats,
    running: bool,
}

impl BybitConnector {
    pub fn new(config: BybitConfig) -> Self {
        let client = BybitClient::new(config.clone());
        Self {
            config,
            client,
            stats: ConnectorStats::default(),
            running: false,
        }
    }
}

#[async_trait]
impl DataSourceConnector for BybitConnector {
    fn source_type(&self) -> DataSourceType {
        DataSourceType::BybitSpot
    }

    fn supported_assets(&self) -> Vec<AssetType> {
        vec![AssetType::Crypto]
    }

    async fn connect(&mut self) -> Result<mpsc::Receiver<StandardMarketData>> {
        self.running = true;
        let streamer = BybitStreamer::new(self.config.ws_url.clone(), self.config.symbols.clone());
        streamer.connect().await
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
