use crate::monitoring::quality::{CheckTier, DataMonitor, DataQualityConfig};
use crate::repository::postgres::PostgresRepositories;
use std::sync::Arc;
use tracing::{error, info};

pub struct DataQualityTask {
    monitor: DataMonitor,
    tier: CheckTier,
}

impl DataQualityTask {
    pub fn new(postgres: Arc<PostgresRepositories>, tier: CheckTier) -> Self {
        let config = DataQualityConfig::default();
        let monitor = DataMonitor::new(postgres.pool.clone(), config);
        Self { monitor, tier }
    }

    pub async fn run(&self) {
        info!(tier = %self.tier, "Starting data quality check");

        match self.monitor.run_checks(self.tier).await {
            Ok(()) => {
                info!(tier = %self.tier, "Data quality check passed");
            }
            Err(e) => {
                error!(tier = %self.tier, err = %e, "Data quality check failed");
            }
        }
    }
}
