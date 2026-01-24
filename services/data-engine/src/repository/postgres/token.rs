use crate::error::DataEngineError;
use crate::repository::token::{ActiveToken, TokenRepository};
use async_trait::async_trait;
use sqlx::PgPool;

pub struct PostgresTokenRepository {
    pool: PgPool,
}

impl PostgresTokenRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TokenRepository for PostgresTokenRepository {
    async fn get_active_addresses(&self) -> Result<Vec<String>, DataEngineError> {
        let rows = sqlx::query_scalar::<_, String>(
            "SELECT address FROM active_tokens WHERE is_active = true ORDER BY liquidity_usd DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            DataEngineError::DatabaseError(format!("Failed to fetch active addresses: {}", e))
        })?;

        Ok(rows)
    }

    async fn get_active_tokens(&self) -> Result<Vec<ActiveToken>, DataEngineError> {
        let rows = sqlx::query_as::<_, ActiveToken>(
            r#"
            SELECT address, symbol, name, decimals, chain, 
                   liquidity_usd, fdv, market_cap, volume_24h, price_change_24h,
                   first_discovered, last_updated, is_active, metadata
            FROM active_tokens 
            WHERE is_active = true 
            ORDER BY liquidity_usd DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            DataEngineError::DatabaseError(format!("Failed to fetch active tokens: {}", e))
        })?;

        Ok(rows)
    }

    async fn upsert_token(&self, token: &ActiveToken) -> Result<(), DataEngineError> {
        sqlx::query(
            r#"
            INSERT INTO active_tokens (
                address, symbol, name, decimals, chain,
                liquidity_usd, fdv, market_cap, volume_24h, price_change_24h,
                is_active, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            ON CONFLICT (address) DO UPDATE SET
                symbol = EXCLUDED.symbol,
                name = EXCLUDED.name,
                liquidity_usd = EXCLUDED.liquidity_usd,
                fdv = EXCLUDED.fdv,
                market_cap = EXCLUDED.market_cap,
                volume_24h = EXCLUDED.volume_24h,
                price_change_24h = EXCLUDED.price_change_24h,
                is_active = EXCLUDED.is_active,
                metadata = EXCLUDED.metadata,
                last_updated = NOW()
            "#,
        )
        .bind(&token.address)
        .bind(&token.symbol)
        .bind(&token.name)
        .bind(token.decimals)
        .bind(&token.chain)
        .bind(token.liquidity_usd)
        .bind(token.fdv)
        .bind(token.market_cap)
        .bind(token.volume_24h)
        .bind(token.price_change_24h)
        .bind(token.is_active)
        .bind(&token.metadata)
        .execute(&self.pool)
        .await
        .map_err(|e| DataEngineError::DatabaseError(format!("Failed to upsert token: {}", e)))?;

        Ok(())
    }

    async fn upsert_tokens(&self, tokens: Vec<ActiveToken>) -> Result<(), DataEngineError> {
        for token in tokens {
            self.upsert_token(&token).await?;
        }
        Ok(())
    }

    async fn deactivate_stale(&self, hours: i64) -> Result<usize, DataEngineError> {
        let result = sqlx::query(
            r#"
            UPDATE active_tokens 
            SET is_active = false 
            WHERE is_active = true 
            AND last_updated < NOW() - INTERVAL '1 hour' * $1
            "#,
        )
        .bind(hours)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            DataEngineError::DatabaseError(format!("Failed to deactivate stale tokens: {}", e))
        })?;

        Ok(result.rows_affected() as usize)
    }

    async fn get_token(&self, address: &str) -> Result<Option<ActiveToken>, DataEngineError> {
        let row = sqlx::query_as::<_, ActiveToken>(
            r#"
            SELECT address, symbol, name, decimals, chain,
                   liquidity_usd, fdv, market_cap, volume_24h, price_change_24h,
                   first_discovered, last_updated, is_active, metadata
            FROM active_tokens 
            WHERE address = $1
            "#,
        )
        .bind(address)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DataEngineError::DatabaseError(format!("Failed to get token: {}", e)))?;

        Ok(row)
    }
}

// Implement sqlx::FromRow for ActiveToken
impl<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> for ActiveToken {
    fn from_row(row: &'r sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;
        Ok(ActiveToken {
            address: row.try_get("address")?,
            symbol: row.try_get("symbol")?,
            name: row.try_get("name")?,
            decimals: row.try_get("decimals")?,
            chain: row.try_get("chain")?,
            liquidity_usd: row.try_get("liquidity_usd")?,
            fdv: row.try_get("fdv")?,
            market_cap: row.try_get("market_cap")?,
            volume_24h: row.try_get("volume_24h")?,
            price_change_24h: row.try_get("price_change_24h")?,
            first_discovered: row.try_get("first_discovered")?,
            last_updated: row.try_get("last_updated")?,
            is_active: row.try_get("is_active")?,
            metadata: row.try_get("metadata")?,
        })
    }
}
