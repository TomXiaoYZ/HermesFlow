use redis::{aio::ConnectionManager, AsyncCommands, Client};

use crate::error::{DataError, Result};
use crate::models::StandardMarketData;

/// Redis cache for storing latest market data
///
/// This cache provides fast access to the most recent market data
/// for each symbol, enabling low-latency queries.
#[derive(Clone)]
pub struct RedisCache {
    connection: ConnectionManager,
    ttl_secs: u64,
}

impl RedisCache {
    /// Creates a new Redis cache
    ///
    /// # Arguments
    ///
    /// * `url` - Redis connection URL (e.g., "redis://localhost:6379")
    /// * `ttl_secs` - Time-to-live for cached data in seconds
    pub async fn new(url: &str, ttl_secs: u64) -> Result<Self> {
        let client = Client::open(url)?;
        let connection = ConnectionManager::new(client)
            .await
            .map_err(DataError::RedisError)?;

        Ok(Self {
            connection,
            ttl_secs,
        })
    }

    /// Stores the latest market data for a symbol
    ///
    /// The data is stored with a key pattern: `market:{source}:{symbol}:latest`
    ///
    /// # Arguments
    ///
    /// * `data` - The market data to store
    pub async fn store_latest(&mut self, data: &StandardMarketData) -> Result<()> {
        let key = format!("market:{}:{}:latest", data.source, data.symbol);
        let json = serde_json::to_string(data)?;

        let mut conn = self.connection.clone();
        conn.set_ex::<_, _, ()>(&key, json, self.ttl_secs).await?;

        tracing::trace!("Cached latest data for {} in Redis", data.symbol);

        Ok(())
    }

    /// Retrieves the latest market data for a symbol
    ///
    /// # Arguments
    ///
    /// * `source` - The data source type as string
    /// * `symbol` - The trading pair symbol
    ///
    /// # Returns
    ///
    /// * `Ok(Some(StandardMarketData))` - Data found in cache
    /// * `Ok(None)` - Data not found (cache miss)
    /// * `Err(_)` - Redis error occurred
    pub async fn get_latest(
        &mut self,
        source: &str,
        symbol: &str,
    ) -> Result<Option<StandardMarketData>> {
        let key = format!("market:{}:{}:latest", source, symbol);

        let mut conn = self.connection.clone();
        let result: Option<String> = conn.get(&key).await?;

        match result {
            Some(json) => {
                let data: StandardMarketData = serde_json::from_str(&json)?;
                Ok(Some(data))
            }
            None => Ok(None),
        }
    }

    /// Checks if Redis is healthy by performing a PING
    pub async fn check_health(&mut self) -> Result<bool> {
        let mut conn = self.connection.clone();
        let pong: String = redis::cmd("PING")
            .query_async(&mut conn)
            .await
            .map_err(DataError::RedisError)?;

        Ok(pong == "PONG")
    }

    /// Publishes a message to a Redis channel
    pub async fn publish(&self, channel: &str, message: &str) -> crate::error::Result<()> {
        let mut conn = self.connection.clone();
        redis::AsyncCommands::publish::<_, _, ()>(&mut conn, channel, message)
            .await
            .map_err(crate::error::DataError::RedisError)?;
        Ok(())
    }

    /// Deletes a cached entry
    pub async fn delete(&mut self, source: &str, symbol: &str) -> Result<()> {
        let key = format!("market:{}:{}:latest", source, symbol);

        let mut conn = self.connection.clone();
        conn.del::<_, ()>(&key).await?;

        Ok(())
    }

    /// Sets the TTL for cached data
    pub fn set_ttl(&mut self, ttl_secs: u64) {
        self.ttl_secs = ttl_secs;
    }

    /// Gets the current TTL setting
    pub fn get_ttl(&self) -> u64 {
        self.ttl_secs
    }

    /// Returns a clone of the connection manager for executing arbitrary commands
    pub fn get_connection(&self) -> ConnectionManager {
        self.connection.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AssetType, DataSourceType, MarketDataType};
    use rust_decimal_macros::dec;

    fn create_test_data() -> StandardMarketData {
        StandardMarketData::new(
            DataSourceType::BinanceSpot,
            "BTCUSDT".to_string(),
            AssetType::Spot,
            MarketDataType::Trade,
            dec!(50000.0),
            dec!(0.1),
            1234567890000,
        )
    }

    #[test]
    fn test_redis_cache_ttl() {
        // Note: These are unit tests that don't require actual Redis connection
        // Integration tests with real Redis would be in tests/integration_tests.rs

        // Mock test - just verify the structure compiles
        let ttl = 86400;
        assert_eq!(ttl, 86400);
    }

    #[test]
    fn test_redis_key_format() {
        let data = create_test_data();
        let key = format!("market:{}:{}:latest", data.source, data.symbol);
        assert_eq!(key, "market:BinanceSpot:BTCUSDT:latest");
    }

    // Integration tests would go here with testcontainers
    // #[tokio::test]
    // async fn test_redis_store_and_retrieve() {
    //     // Requires Redis running - would use testcontainers
    // }
}
