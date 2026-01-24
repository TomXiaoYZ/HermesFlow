use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use crate::error::DataEngineError;
use crate::models::{Order, Trade};
use crate::repository::TradingRepository;

pub struct PostgresTradingRepository {
    pool: PgPool,
}

impl PostgresTradingRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TradingRepository for PostgresTradingRepository {
    async fn insert_order(&self, order: &Order) -> Result<Uuid, DataEngineError> {
        sqlx::query(r#"
            INSERT INTO orders (id, ib_order_id, symbol, action, quantity, order_type, status)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#)
        .bind(order.id)
        .bind(order.ib_order_id)
        .bind(&order.symbol)
        .bind(&order.action)
        .bind(order.quantity)
        .bind(&order.order_type)
        .bind(&order.status)
        .execute(&self.pool)
        .await
        .map_err(|e| DataEngineError::DatabaseError(format!("Failed to insert order: {}", e)))?;
        
        Ok(order.id)
    }

    async fn insert_trade(&self, trade: &Trade) -> Result<Uuid, DataEngineError> {
        sqlx::query(r#"
            INSERT INTO trades (id, order_id, symbol, quantity, price, commission)
            VALUES ($1, $2, $3, $4, $5, $6)
        "#)
        .bind(trade.id)
        .bind(trade.order_id)
        .bind(&trade.symbol)
        .bind(trade.quantity)
        .bind(trade.price)
        .bind(trade.commission)
        .execute(&self.pool)
        .await
        .map_err(|e| DataEngineError::DatabaseError(format!("Failed to insert trade: {}", e)))?;
        
        Ok(trade.id)
    }
}
