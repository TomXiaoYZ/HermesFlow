use crate::error::DataEngineError;
use crate::repository::MetricsRepository;
use async_trait::async_trait;
use sqlx::PgPool;
use tracing::error;

pub struct PostgresMetricsRepository {
    pool: PgPool,
}

impl PostgresMetricsRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MetricsRepository for PostgresMetricsRepository {
    async fn insert_api_usage(&self, provider: &str, count: i64) -> Result<(), DataEngineError> {
        sqlx::query(
            r#"
            INSERT INTO api_usage_metrics (provider, request_count, timestamp)
            VALUES ($1, $2, NOW())
            "#,
        )
        .bind(provider)
        .bind(count)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to insert api usage metric: {}", e);
            DataEngineError::DatabaseError(format!("Failed to insert api usage: {}", e))
        })?;

        Ok(())
    }
}
