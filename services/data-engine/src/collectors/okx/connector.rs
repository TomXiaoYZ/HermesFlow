use super::client::OkxClient;
use super::config::OkxConfig;
use super::websocket::OkxStreamer;
use crate::error::Result;
use crate::models::{AssetType, DataSourceType, StandardMarketData};
use crate::traits::{ConnectorStats, DataSourceConnector};
use async_trait::async_trait;
use tokio::sync::mpsc;

#[allow(dead_code)]
pub struct OkxConnector {
    config: OkxConfig,
    client: OkxClient,
    stats: ConnectorStats,
    running: bool,
}

impl OkxConnector {
    pub fn new(config: OkxConfig) -> Self {
        let client = OkxClient::new(config.clone());
        Self {
            config,
            client,
            stats: ConnectorStats::default(),
            running: false,
        }
    }
}

#[async_trait]
impl DataSourceConnector for OkxConnector {
    fn source_type(&self) -> DataSourceType {
        DataSourceType::OkxSpot // Default to Spot, could be dynamic
    }

    fn supported_assets(&self) -> Vec<AssetType> {
        vec![AssetType::Crypto]
    }

    async fn connect(&mut self) -> Result<mpsc::Receiver<StandardMarketData>> {
        self.running = true;

        // Symbols need to be passed from somewhere. Again, config doesn't have it by default unless updated.
        // Assuming symbols logic is handled externally or added to config.
        // I'll grab symbols from a theoretical place or pass empty if not present.
        // Wait, for Binance I assumed they are passed.
        // In main.rs, the 'DataSourceConfig' has symbols.
        // But 'OkxConfig' struct used here does NOT.
        // Just like Binance, I should probably pass symbols into the constructor or add it to the struct.
        // For now, I'll instantiate Streamer with empty symbols if none found, but that won't work.
        // I will add `symbols` field to `OkxConfig` too, to make it self-contained?
        // No, `AppConfig` has `data_sources` list. Maybe I should use that?
        // But main.rs passes `config.okx` (OkxConfig) to `new`.
        // I should stick to adding `symbols` to `OkxConfig` struct for consistency or handle it in main.rs logic?
        // Actually, main.rs logic uses `DataSourceConfig` to enable sources but `config.okx` for API keys.
        // This is a disconnect in my config design.
        // The `DataSourceConfig` list is generic. `OkxConfig` is specific.
        // Ideally `OkxConfig` should include the subscription list or `main.rs` should combine them.
        // Currently `main.rs` uses `AkShareConfig` etc which HAVE fields like `symbols` (IBKRConfig has it).
        // Let's check `OkxConfig` I wrote. It does NOT have symbols.
        // I should add `symbols: Vec<String>` to `OkxConfig` to match `IbkrConfig`.

        // But wait, I can't change `OkxConfig` definition easily without updating `config.rs` AGAIN.
        // Wait, `OkxConfig` is defined in `collectors/okx/config.rs`. I CAN update it easily.
        // So I will update `collectors/okx/config.rs` to include `symbols`.

        // For now, write this connector assuming `config.symbols` exists.

        let streamer = OkxStreamer::new(self.config.ws_url.clone(), self.config.symbols.clone());
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
