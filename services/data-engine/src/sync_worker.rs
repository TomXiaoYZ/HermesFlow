// Market Data Sync Worker
// Automatically syncs historical data when new tickers are added to watchlist

use sqlx::{PgPool, postgres::PgListener};
use tokio::time::{interval, Duration};
use tracing::{info, warn, error};
use std::collections::HashMap;

use crate::collectors::{
    polygon::{PolygonConfig, PolygonConnector},
    // Add other connectors as needed
};

pub struct MarketSyncWorker {
    pool: PgPool,
    connectors: HashMap<String, Box<dyn ExchangeConnector>>,
}

trait ExchangeConnector: Send + Sync {
    fn exchange_name(&self) -> &str;
    async fn sync_historical(
        &self,
        symbol: &str,
        resolution: &str,
        from_date: &str,
        to_date: &str,
    ) -> Result<Vec<Candle>, Box<dyn std::error::Error + Send + Sync>>;
}

// Polygon connector implementation
struct PolygonExchangeConnector {
    connector: PolygonConnector,
}

impl ExchangeConnector for PolygonExchangeConnector {
    fn exchange_name(&self) -> &str {
        "Polygon"
    }
    
    async fn sync_historical(
        &self,
        symbol: &str,
        resolution: &str,
        from_date: &str,
        to_date: &str,
    ) -> Result<Vec<Candle>, Box<dyn std::error::Error + Send + Sync>> {
        self.connector.fetch_history_candles(symbol, resolution, from_date, to_date).await
    }
}

impl MarketSyncWorker {
    pub async fn new(pool: PgPool) -> Result<Self, Box<dyn std::error::Error>> {
        let mut connectors: HashMap<String, Box<dyn ExchangeConnector>> = HashMap::new();
        
        // Initialize Polygon connector if enabled
        if let Ok(polygon_config) = PolygonConfig::from_env() {
            info!("Initializing Polygon connector for auto-sync");
            let polygon_connector = PolygonConnector::new(polygon_config);
            connectors.insert(
                "Polygon".to_string(),
                Box::new(PolygonExchangeConnector { connector: polygon_connector })
            );
        }
        
        // TODO: Add other connectors (Binance, Bybit, Birdeye, etc.)
        
        Ok(Self { pool, connectors })
    }
    
    /// Main worker loop - polls for pending sync tasks
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("🚀 Market Sync Worker started");
        
        let mut interval = interval(Duration::from_secs(10)); // Check every 10 seconds
        
        loop {
            interval.tick().await;
            
            if let Err(e) = self.process_pending_tasks().await {
                error!("Error processing sync tasks: {}", e);
            }
        }
    }
    
    /// Listen to PostgreSQL notifications for immediate sync
    pub async fn listen_notifications(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut listener = PgListener::connect_with(&self.pool).await?;
        listener.listen_all(vec!["watchlist_insert", "watchlist_update"]).await?;
        
        info!("📡 Listening for watchlist changes...");
        
        loop {
            match listener.recv().await {
                Ok(notification) => {
                    info!("Received notification: {} - {}", notification.channel(), notification.payload());
                    
                    // Trigger immediate sync for new ticker
                    if let Err(e) = self.process_pending_tasks().await {
                        error!("Error processing notification: {}", e);
                    }
                }
                Err(e) => {
                    error!("Error receiving notification: {}", e);
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        }
    }
    
    async fn process_pending_tasks(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Get pending tasks from database
        let tasks = sqlx::query!(
            r#"
            SELECT 
                s.exchange,
                s.symbol,
                s.resolution,
                COALESCE(w.sync_from_date, '2023-01-01'::DATE) as "sync_from_date!",
                COALESCE(w.priority, 50) as "priority!"
            FROM market_sync_status s
            INNER JOIN market_watchlist w ON s.exchange = w.exchange AND s.symbol = w.symbol
            WHERE s.status = 'pending'
              AND w.is_active = true
            ORDER BY w.priority DESC, s.exchange, s.symbol, s.resolution
            LIMIT 10
            "#
        )
        .fetch_all(&self.pool)
        .await?;
        
        if tasks.is_empty() {
            return Ok(());
        }
        
        info!("📊 Processing {} pending sync tasks", tasks.len());
        
        for task in tasks {
            let exchange = task.exchange;
            let symbol = task.symbol;
            let resolution = task.resolution;
            
            // Get the appropriate connector
            let connector = match self.connectors.get(&exchange) {
                Some(c) => c,
                None => {
                    warn!("No connector available for exchange: {}", exchange);
                    continue;
                }
            };
            
            info!("🔄 Syncing {}/{} - {}", exchange, symbol, resolution);
            
            // Mark as syncing
            sqlx::query!(
                "UPDATE market_sync_status SET status = 'syncing' WHERE exchange = $1 AND symbol = $2 AND resolution = $3",
                &exchange, &symbol, &resolution
            )
            .execute(&self.pool)
            .await?;
            
            // Perform sync
            let from_date = task.sync_from_date.format("%Y-%m-%d").to_string();
            let to_date = chrono::Utc::now().format("%Y-%m-%d").to_string();
            
            match connector.sync_historical(&symbol, &resolution, &from_date, &to_date).await {
                Ok(candles) => {
                    info!("✅ Fetched {} candles for {}/{} - {}", candles.len(), exchange, symbol, resolution);
                    
                    // Insert candles into database
                    if let Err(e) = self.insert_candles(&candles).await {
                        error!("Failed to insert candles: {}", e);
                        
                        sqlx::query!(
                            "UPDATE market_sync_status SET status = 'failed', error_message = $4 WHERE exchange = $1 AND symbol = $2 AND resolution = $3",
                            &exchange, &symbol, &resolution, e.to_string()
                        )
                        .execute(&self.pool)
                        .await?;
                        
                        continue;
                    }
                    
                    // Mark as completed
                    sqlx::query!(
                        r#"
                        UPDATE market_sync_status 
                        SET status = 'completed',
                            total_candles = $4,
                            last_sync_at = NOW(),
                            last_synced_time = (
                                SELECT MAX(time) FROM mkt_equity_candles 
                                WHERE exchange = $1 AND symbol = $2 AND resolution = $3
                            )
                        WHERE exchange = $1 AND symbol = $2 AND resolution = $3
                        "#,
                        &exchange, &symbol, &resolution, candles.len() as i32
                    )
                    .execute(&self.pool)
                    .await?;
                    
                    info!("✅ Completed sync for {}/{} - {}", exchange, symbol, resolution);
                }
                Err(e) => {
                    error!("❌ Sync failed for {}/{} - {}: {}", exchange, symbol, resolution, e);
                    
                    sqlx::query!(
                        "UPDATE market_sync_status SET status = 'failed', error_message = $4, retry_count = retry_count + 1 WHERE exchange = $1 AND symbol = $2 AND resolution = $3",
                        &exchange, &symbol, &resolution, e.to_string()
                    )
                    .execute(&self.pool)
                    .await?;
                }
            }
            
            // Rate limiting between tasks
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        
        Ok(())
    }
    
    async fn insert_candles(&self, candles: &[Candle]) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        use crate::collectors::polygon::historical_sync::insert_candles_batch;
        insert_candles_batch(&self.pool, candles).await
    }
}

// Import Candle type
use crate::types::Candle;
