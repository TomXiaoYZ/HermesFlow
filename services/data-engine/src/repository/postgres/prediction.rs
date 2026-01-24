use crate::error::DataEngineError;
use crate::models::{MarketOutcome, PredictionMarket};
use crate::repository::PredictionRepository;
use async_trait::async_trait;
use sqlx::PgPool;

pub struct PostgresPredictionRepository {
    pool: PgPool,
}

impl PostgresPredictionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PredictionRepository for PostgresPredictionRepository {
    async fn upsert_market(&self, market: &PredictionMarket) -> Result<(), DataEngineError> {
        sqlx::query(
            r#"
            INSERT INTO prediction_markets (
                id, source, title, description, category, end_date, active, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (id) DO UPDATE SET
                title = EXCLUDED.title,
                description = EXCLUDED.description,
                category = EXCLUDED.category,
                end_date = EXCLUDED.end_date,
                active = EXCLUDED.active,
                metadata = EXCLUDED.metadata,
                updated_at = NOW()
        "#,
        )
        .bind(&market.id)
        .bind(&market.source)
        .bind(&market.title)
        .bind(&market.description)
        .bind(&market.category)
        .bind(market.end_date)
        .bind(market.active)
        .bind(&market.metadata)
        .execute(&self.pool)
        .await
        .map_err(|e| DataEngineError::DatabaseError(format!("Failed to upsert market: {}", e)))?;
        Ok(())
    }

    async fn insert_outcome(
        &self,
        market_id: &str,
        outcome: &MarketOutcome,
    ) -> Result<(), DataEngineError> {
        sqlx::query(
            r#"
            INSERT INTO market_outcomes (market_id, outcome, price, volume_24h, timestamp)
            VALUES ($1, $2, $3, $4, NOW())
            ON CONFLICT (market_id, outcome, timestamp) DO NOTHING
        "#,
        )
        .bind(market_id)
        .bind(&outcome.outcome)
        .bind(outcome.price)
        .bind(outcome.volume_24h)
        .execute(&self.pool)
        .await
        .map_err(|e| DataEngineError::DatabaseError(format!("Failed to insert outcome: {}", e)))?;
        Ok(())
    }
}
