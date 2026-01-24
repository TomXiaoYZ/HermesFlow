use crate::error::DataEngineError;
use crate::monitoring::quality::{DataMonitor, DataQualityConfig};
use crate::repository::postgres::PostgresRepositories;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};

pub struct DataQualityTask {
    monitor: DataMonitor,
}

impl DataQualityTask {
    pub fn new(postgres: Arc<PostgresRepositories>) -> Self {
        // Initialize DataMonitor with default config and the shared pool
        let config = DataQualityConfig::default();
        let monitor = DataMonitor::new(postgres.pool.clone(), config);
        Self { monitor }
    }

    pub async fn run(&self) {
        info!("🕵️ Starting Data Quality Check (5-Stage Validation)...");

        match self.monitor.run_checks().await {
            Ok(_) => {
                info!("✅ Data Quality Check Passed.");
            }
            Err(e) => {
                error!("❌ Data Quality Check Failed: {}", e);
            }
        }
    }
}
