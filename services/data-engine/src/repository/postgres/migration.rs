use sqlx::PgPool;
use tracing::{info, warn};
use crate::error::DataEngineError;

pub struct MigrationManager {
    pool: PgPool,
}

impl MigrationManager {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Runs all migrations
    pub async fn run_migrations(&self) -> Result<(), DataEngineError> {
        self.create_base_schema().await?;
        self.create_twitter_runs_table().await?;
        self.run_trading_system_migrations().await?;
        self.run_market_data_migrations().await?;
        info!("All DB migrations completed successfully");
        Ok(())
    }

    /// Creates base tables (tweets, prediction_markets)
    async fn create_base_schema(&self) -> Result<(), DataEngineError> {
        // Tweets table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS tweets (
                id BIGINT PRIMARY KEY,
                username TEXT NOT NULL,
                text TEXT NOT NULL,
                created_at TIMESTAMPTZ NOT NULL,
                received_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                user_id BIGINT,
                followers_count INTEGER,
                verified BOOLEAN DEFAULT false,
                retweet_count INTEGER DEFAULT 0,
                favorite_count INTEGER DEFAULT 0,
                reply_count INTEGER DEFAULT 0,
                quote_count INTEGER DEFAULT 0,
                is_retweet BOOLEAN DEFAULT false,
                is_reply BOOLEAN DEFAULT false,
                hashtags TEXT[],
                media_urls TEXT[],
                raw_data JSONB
            )
            "#,
        )
        .execute(&self.pool).await
        .map_err(|e| DataEngineError::DatabaseError(format!("Failed to create tweets table: {}", e)))?;

        // Indices for tweets
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_tweets_username ON tweets(username)")
            .execute(&self.pool).await
            .map_err(|e| DataEngineError::DatabaseError(format!("Failed to create index: {}", e)))?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_tweets_created_at ON tweets(created_at)")
            .execute(&self.pool).await
            .map_err(|e| DataEngineError::DatabaseError(format!("Failed to create index: {}", e)))?;

        // Prediction markets
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS prediction_markets (
                id TEXT PRIMARY KEY,
                source TEXT NOT NULL,
                title TEXT NOT NULL,
                description TEXT,
                category TEXT,
                end_date TIMESTAMPTZ,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                active BOOLEAN DEFAULT true,
                metadata JSONB
            )
            "#,
        )
        .execute(&self.pool).await
        .map_err(|e| DataEngineError::DatabaseError(format!("Failed to create prediction_markets table: {}", e)))?;

        // Market outcomes
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS market_outcomes (
                id SERIAL PRIMARY KEY,
                market_id TEXT NOT NULL REFERENCES prediction_markets(id) ON DELETE CASCADE,
                outcome TEXT NOT NULL,
                price DECIMAL(18, 8) NOT NULL,
                volume_24h DECIMAL(18, 8),
                timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                UNIQUE(market_id, outcome, timestamp)
            )
            "#,
        )
        .execute(&self.pool).await
        .map_err(|e| DataEngineError::DatabaseError(format!("Failed to create market_outcomes table: {}", e)))?;

        // Indices
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_markets_source ON prediction_markets(source)")
            .execute(&self.pool).await.map_err(|e| DataEngineError::DatabaseError(format!("Failed index: {}", e)))?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_outcomes_market_id ON market_outcomes(market_id)")
            .execute(&self.pool).await.map_err(|e| DataEngineError::DatabaseError(format!("Failed index: {}", e)))?;
        
        info!("Base schema created successfully");
        Ok(())
    }

    async fn create_twitter_runs_table(&self) -> Result<(), DataEngineError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS twitter_collection_runs (
                id SERIAL PRIMARY KEY,
                target TEXT NOT NULL,
                collected_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                scraped_count INTEGER NOT NULL,
                upserted_count INTEGER NOT NULL,
                error TEXT
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DataEngineError::DatabaseError(format!("Failed to create twitter_collection_runs: {}", e)))?;
        Ok(())
    }

    async fn run_market_data_migrations(&self) -> Result<(), DataEngineError> {
        // Reuse checking logic from postgres.rs
        let migration_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM information_schema.tables WHERE table_name = 'schema_migrations')"
        )
        .fetch_one(&self.pool).await.unwrap_or(false);

        if migration_exists {
            let applied: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = '005_market_data')"
            ).fetch_one(&self.pool).await.unwrap_or(false);
            if applied { return Ok(()); }
        }

        let sql = r#"
            CREATE TABLE IF NOT EXISTS mkt_equity_snapshots (
                id BIGSERIAL PRIMARY KEY,
                symbol VARCHAR(20) NOT NULL,
                price DECIMAL(18,4) NOT NULL,
                bid DECIMAL(18,4),
                ask DECIMAL(18,4),
                bid_size INTEGER,
                ask_size INTEGER,
                volume BIGINT,
                vwap DECIMAL(18,4),
                high DECIMAL(18,4),
                low DECIMAL(18,4),
                open DECIMAL(18,4),
                prev_close DECIMAL(18,4),
                timestamp TIMESTAMPTZ NOT NULL,
                received_at TIMESTAMPTZ DEFAULT NOW()
            );
            CREATE INDEX IF NOT EXISTS idx_mkt_equity_snapshots_symbol_time 
                ON mkt_equity_snapshots(symbol, timestamp DESC);

            CREATE TABLE IF NOT EXISTS mkt_equity_candles (
                id BIGSERIAL PRIMARY KEY,
                symbol VARCHAR(20) NOT NULL,
                resolution VARCHAR(10) NOT NULL,
                open DECIMAL(18,4) NOT NULL,
                high DECIMAL(18,4) NOT NULL,
                low DECIMAL(18,4) NOT NULL,
                close DECIMAL(18,4) NOT NULL,
                volume BIGINT NOT NULL,
                timestamp TIMESTAMPTZ NOT NULL,
                received_at TIMESTAMPTZ DEFAULT NOW(),
                UNIQUE(symbol, resolution, timestamp)
            );
            CREATE INDEX IF NOT EXISTS idx_mkt_equity_candles_lookup 
                ON mkt_equity_candles(symbol, resolution, timestamp DESC);
            
            CREATE TABLE IF NOT EXISTS schema_migrations (
                version VARCHAR(50) PRIMARY KEY,
                description TEXT,
                applied_at TIMESTAMPTZ DEFAULT NOW()
            );
        "#;
        
        // Execute raw sql
        if let Err(e) = sqlx::raw_sql(sql).execute(&self.pool).await {
             warn!("Market data migration warning: {}", e);
        }
        
        let _ = sqlx::query("INSERT INTO schema_migrations (version, description) VALUES ('005_market_data', 'Market data tables') ON CONFLICT (version) DO NOTHING")
            .execute(&self.pool).await;

        Ok(())
    }

    async fn run_trading_system_migrations(&self) -> Result<(), DataEngineError> {
         let migration_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM information_schema.tables WHERE table_name = 'schema_migrations')"
        ).fetch_one(&self.pool).await.unwrap_or(false);

        if migration_exists {
            let applied: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = '004_trading_tables')"
            ).fetch_one(&self.pool).await.unwrap_or(false);
            if applied { return Ok(()); }
        }

        let sql = r#"
            CREATE TABLE IF NOT EXISTS orders (
                id UUID PRIMARY KEY,
                ib_order_id INTEGER,
                symbol VARCHAR(50) NOT NULL,
                action VARCHAR(10) NOT NULL,
                quantity DECIMAL(18, 8) NOT NULL,
                order_type VARCHAR(10) NOT NULL,
                status VARCHAR(20) NOT NULL,
                created_at TIMESTAMPTZ DEFAULT NOW(),
                updated_at TIMESTAMPTZ DEFAULT NOW()
            );

            CREATE TABLE IF NOT EXISTS trades (
                id UUID PRIMARY KEY,
                order_id UUID REFERENCES orders(id),
                symbol VARCHAR(50) NOT NULL,
                quantity DECIMAL(18, 8) NOT NULL,
                price DECIMAL(18, 8) NOT NULL,
                commission DECIMAL(18, 8),
                executed_at TIMESTAMPTZ DEFAULT NOW()
            );
            
             CREATE TABLE IF NOT EXISTS schema_migrations (
                version VARCHAR(50) PRIMARY KEY,
                description TEXT,
                applied_at TIMESTAMPTZ DEFAULT NOW()
            );
        "#;

        if let Err(e) = sqlx::raw_sql(sql).execute(&self.pool).await {
            warn!("Trading migration warning: {}", e);
        }

        let _ = sqlx::query("INSERT INTO schema_migrations (version, description) VALUES ('004_trading_tables', 'Basic trading tables') ON CONFLICT (version) DO NOTHING")
            .execute(&self.pool).await;

        Ok(())
    }
}
