use crate::config::PolymarketConfig;
use crate::error::Result;
use tokio::sync::broadcast::Receiver;

pub struct PolymarketCollector {
    config: PolymarketConfig,
}

impl PolymarketCollector {
    pub fn new(config: PolymarketConfig) -> Self {
        Self { config }
    }

    pub async fn start(&self, mut shutdown: Receiver<()>) -> Result<()> {
        // Stub implementation
        let _ = shutdown.recv().await;
        Ok(())
    }
}
