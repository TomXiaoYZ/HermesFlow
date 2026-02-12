use crate::error::DataEngineError;
use crate::models::{MarketOutcome, PredictionMarket};
use crate::repository::PredictionRepository;
use async_trait::async_trait;
use sqlx::{PgPool, Row};

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
        .bind(outcome.volume)
        .execute(&self.pool)
        .await
        .map_err(|e| DataEngineError::DatabaseError(format!("Failed to insert outcome: {}", e)))?;
        Ok(())
    }

    async fn list_markets(
        &self,
        active_only: bool,
        category: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<PredictionMarket>, DataEngineError> {
        let rows = if active_only {
            if let Some(cat) = category {
                sqlx::query(
                    r#"
                    SELECT id, source, title, description, category, end_date,
                           created_at, updated_at, active, metadata
                    FROM prediction_markets
                    WHERE active = true AND category = $1
                    ORDER BY updated_at DESC
                    LIMIT $2 OFFSET $3
                    "#,
                )
                .bind(cat)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
            } else {
                sqlx::query(
                    r#"
                    SELECT id, source, title, description, category, end_date,
                           created_at, updated_at, active, metadata
                    FROM prediction_markets
                    WHERE active = true
                    ORDER BY updated_at DESC
                    LIMIT $1 OFFSET $2
                    "#,
                )
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await
            }
        } else if let Some(cat) = category {
            sqlx::query(
                r#"
                SELECT id, source, title, description, category, end_date,
                       created_at, updated_at, active, metadata
                FROM prediction_markets
                WHERE category = $1
                ORDER BY updated_at DESC
                LIMIT $2 OFFSET $3
                "#,
            )
            .bind(cat)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query(
                r#"
                SELECT id, source, title, description, category, end_date,
                       created_at, updated_at, active, metadata
                FROM prediction_markets
                ORDER BY updated_at DESC
                LIMIT $1 OFFSET $2
                "#,
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
        }
        .map_err(|e| DataEngineError::DatabaseError(format!("Failed to list markets: {}", e)))?;

        let mut markets = Vec::with_capacity(rows.len());
        for row in rows {
            let market_id: String = row.get("id");

            // Fetch latest outcomes for this market
            let outcome_rows = sqlx::query(
                r#"
                SELECT DISTINCT ON (outcome) outcome, price, volume_24h, timestamp
                FROM market_outcomes
                WHERE market_id = $1
                ORDER BY outcome, timestamp DESC
                "#,
            )
            .bind(&market_id)
            .fetch_all(&self.pool)
            .await
            .unwrap_or_default();

            let outcomes: Vec<MarketOutcome> = outcome_rows
                .iter()
                .map(|r| MarketOutcome {
                    outcome: r.get("outcome"),
                    price: r.get("price"),
                    volume: r.try_get("volume_24h").ok(),
                    timestamp: r.get("timestamp"),
                })
                .collect();

            markets.push(PredictionMarket {
                id: market_id,
                source: row.get("source"),
                title: row.get("title"),
                description: row.try_get("description").ok(),
                category: row.try_get("category").ok(),
                end_date: row.try_get("end_date").ok().flatten(),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
                active: row.get("active"),
                outcomes,
                metadata: row
                    .try_get("metadata")
                    .ok()
                    .unwrap_or(serde_json::json!({})),
            });
        }

        Ok(markets)
    }

    async fn get_market(
        &self,
        market_id: &str,
    ) -> Result<Option<PredictionMarket>, DataEngineError> {
        let row_opt = sqlx::query(
            r#"
            SELECT id, source, title, description, category, end_date,
                   created_at, updated_at, active, metadata
            FROM prediction_markets
            WHERE id = $1
            "#,
        )
        .bind(market_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DataEngineError::DatabaseError(format!("Failed to get market: {}", e)))?;

        let row = match row_opt {
            Some(r) => r,
            None => return Ok(None),
        };

        // Fetch latest outcomes
        let outcome_rows = sqlx::query(
            r#"
            SELECT DISTINCT ON (outcome) outcome, price, volume_24h, timestamp
            FROM market_outcomes
            WHERE market_id = $1
            ORDER BY outcome, timestamp DESC
            "#,
        )
        .bind(market_id)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        let outcomes: Vec<MarketOutcome> = outcome_rows
            .iter()
            .map(|r| MarketOutcome {
                outcome: r.get("outcome"),
                price: r.get("price"),
                volume: r.try_get("volume_24h").ok(),
                timestamp: r.get("timestamp"),
            })
            .collect();

        Ok(Some(PredictionMarket {
            id: row.get("id"),
            source: row.get("source"),
            title: row.get("title"),
            description: row.try_get("description").ok(),
            category: row.try_get("category").ok(),
            end_date: row.try_get("end_date").ok().flatten(),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            active: row.get("active"),
            outcomes,
            metadata: row
                .try_get("metadata")
                .ok()
                .unwrap_or(serde_json::json!({})),
        }))
    }

    async fn get_outcome_history(
        &self,
        market_id: &str,
        limit: i64,
    ) -> Result<Vec<MarketOutcome>, DataEngineError> {
        let rows = sqlx::query(
            r#"
            SELECT outcome, price, volume_24h, timestamp
            FROM market_outcomes
            WHERE market_id = $1
            ORDER BY timestamp DESC
            LIMIT $2
            "#,
        )
        .bind(market_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            DataEngineError::DatabaseError(format!("Failed to get outcome history: {}", e))
        })?;

        Ok(rows
            .iter()
            .map(|r| MarketOutcome {
                outcome: r.get("outcome"),
                price: r.get("price"),
                volume: r.try_get("volume_24h").ok(),
                timestamp: r.get("timestamp"),
            })
            .collect())
    }
}
