use crate::error::DataEngineError;
use sqlx::PgPool;
use tracing::info;

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
        self.create_active_tokens_table().await?;
        self.create_strategy_table().await?;
        self.run_trading_system_migrations().await?;
        self.run_market_data_migrations().await?;
        self.create_metrics_table().await?;
        self.create_watchlist_table().await?;
        self.create_factors_table().await?;
        self.create_backtest_results_table().await?;
        info!("All DB migrations completed successfully");
        Ok(())
    }

    async fn create_watchlist_table(&self) -> Result<(), DataEngineError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS market_watchlist (
                exchange TEXT NOT NULL,
                symbol TEXT NOT NULL,
                name TEXT,
                asset_type TEXT NOT NULL DEFAULT 'stock',
                is_active BOOLEAN DEFAULT true,
                created_at TIMESTAMPTZ DEFAULT NOW(),
                PRIMARY KEY (exchange, symbol)
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            DataEngineError::DatabaseError(format!(
                "Failed to create market_watchlist table: {}",
                e
            ))
        })?;

        // Ensure asset_type column exists
        sqlx::query(
            r#"
            ALTER TABLE market_watchlist 
            ADD COLUMN IF NOT EXISTS asset_type TEXT NOT NULL DEFAULT 'stock'
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            DataEngineError::DatabaseError(format!(
                "Failed to add asset_type to market_watchlist: {}",
                e
            ))
        })?;

        // Create trigger for auto-sync on insert if needed
        // For now, simpler to just have the table.
        Ok(())
    }

    async fn create_strategy_table(&self) -> Result<(), DataEngineError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS strategy_generations (
                generation INTEGER PRIMARY KEY,
                fitness DOUBLE PRECISION,
                best_genome INTEGER[],
                timestamp TIMESTAMPTZ DEFAULT NOW(),
                metadata JSONB
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            DataEngineError::DatabaseError(format!(
                "Failed to create strategy_generations table: {}",
                e
            ))
        })?;
        Ok(())
    }

    async fn create_active_tokens_table(&self) -> Result<(), DataEngineError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS active_tokens (
                address TEXT PRIMARY KEY,
                symbol TEXT NOT NULL,
                name TEXT NOT NULL,
                decimals INTEGER NOT NULL,
                chain TEXT NOT NULL,
                liquidity_usd DECIMAL(40, 8),
                fdv DECIMAL(40, 8),
                market_cap DECIMAL(40, 8),
                volume_24h DECIMAL(40, 8),
                price_change_24h DECIMAL(40, 8),
                first_discovered TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                last_updated TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                is_active BOOLEAN DEFAULT true,
                metadata JSONB
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            DataEngineError::DatabaseError(format!("Failed to create active_tokens table: {}", e))
        })?;

        // Index for liquidity sorting
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_active_tokens_liquidity ON active_tokens(liquidity_usd DESC)")
            .execute(&self.pool)
            .await
            .map_err(|e| DataEngineError::DatabaseError(format!("Failed index: {}", e)))?;

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
        .execute(&self.pool)
        .await
        .map_err(|e| {
            DataEngineError::DatabaseError(format!("Failed to create tweets table: {}", e))
        })?;

        // Indices for tweets
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_tweets_username ON tweets(username)")
            .execute(&self.pool)
            .await
            .map_err(|e| {
                DataEngineError::DatabaseError(format!("Failed to create index: {}", e))
            })?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_tweets_created_at ON tweets(created_at)")
            .execute(&self.pool)
            .await
            .map_err(|e| {
                DataEngineError::DatabaseError(format!("Failed to create index: {}", e))
            })?;

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
        .execute(&self.pool)
        .await
        .map_err(|e| {
            DataEngineError::DatabaseError(format!(
                "Failed to create prediction_markets table: {}",
                e
            ))
        })?;

        // Market outcomes
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS market_outcomes (
                id SERIAL PRIMARY KEY,
                market_id TEXT NOT NULL REFERENCES prediction_markets(id) ON DELETE CASCADE,
                outcome TEXT NOT NULL,
                price DECIMAL(40, 8) NOT NULL,
                volume_24h DECIMAL(40, 8),
                timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                UNIQUE(market_id, outcome, timestamp)
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            DataEngineError::DatabaseError(format!("Failed to create market_outcomes table: {}", e))
        })?;

        // Indices
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_markets_source ON prediction_markets(source)")
            .execute(&self.pool)
            .await
            .map_err(|e| DataEngineError::DatabaseError(format!("Failed index: {}", e)))?;
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_outcomes_market_id ON market_outcomes(market_id)",
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DataEngineError::DatabaseError(format!("Failed index: {}", e)))?;

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
        .map_err(|e| {
            DataEngineError::DatabaseError(format!(
                "Failed to create twitter_collection_runs: {}",
                e
            ))
        })?;
        Ok(())
    }

    async fn run_market_data_migrations(&self) -> Result<(), DataEngineError> {
        // Ensure mkt_equity_candles table exists (if not created by manual SQL)
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS mkt_equity_candles (
                exchange TEXT NOT NULL,
                symbol TEXT NOT NULL,
                resolution TEXT NOT NULL,
                time TIMESTAMPTZ NOT NULL,
                open DECIMAL(40, 8) NOT NULL,
                high DECIMAL(40, 8) NOT NULL,
                low DECIMAL(40, 8) NOT NULL,
                close DECIMAL(40, 8) NOT NULL,
                volume DECIMAL(40, 8) NOT NULL,
                amount DECIMAL(40, 8),
                metadata JSONB,
                PRIMARY KEY (exchange, symbol, resolution, time)
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            DataEngineError::DatabaseError(format!(
                "Failed to ensure mkt_equity_candles exists: {}",
                e
            ))
        })?;

        // Add liquidity column if not exists
        sqlx::query(
            r#"
            ALTER TABLE mkt_equity_candles 
            ADD COLUMN IF NOT EXISTS liquidity DECIMAL(40, 8)
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            DataEngineError::DatabaseError(format!("Failed to add liquidity column: {}", e))
        })?;

        // Add fdv column if not exists
        sqlx::query(
            r#"
            ALTER TABLE mkt_equity_candles 
            ADD COLUMN IF NOT EXISTS fdv DECIMAL(40, 8)
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DataEngineError::DatabaseError(format!("Failed to add fdv column: {}", e)))?;

        // Ensure mkt_equity_snapshots table exists (basic version)
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS mkt_equity_snapshots (
                symbol TEXT NOT NULL,
                price DECIMAL(40, 8) NOT NULL,
                bid DECIMAL(40, 8),
                ask DECIMAL(40, 8),
                volume BIGINT,
                vwap DECIMAL(40, 8),
                high DECIMAL(40, 8),
                low DECIMAL(40, 8),
                timestamp TIMESTAMPTZ NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            DataEngineError::DatabaseError(format!(
                "Failed to ensure mkt_equity_snapshots exists: {}",
                e
            ))
        })?;

        Ok(())
    }

    async fn run_trading_system_migrations(&self) -> Result<(), DataEngineError> {
        // trade_orders
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS trade_orders (
                id              BIGSERIAL PRIMARY KEY,
                order_id        TEXT UNIQUE NOT NULL,
                parent_order_id TEXT,
                exchange        VARCHAR(50) NOT NULL,
                account_id      VARCHAR(50),
                symbol          VARCHAR(50) NOT NULL,
                asset_type      VARCHAR(20) NOT NULL,
                side            VARCHAR(10) NOT NULL,
                order_type      VARCHAR(20) NOT NULL,
                quantity        DECIMAL(24,8) NOT NULL,
                filled_qty      DECIMAL(24,8) DEFAULT 0,
                price           DECIMAL(24,8),
                avg_price       DECIMAL(24,8),
                status          VARCHAR(20) NOT NULL DEFAULT 'NEW',
                commission      DECIMAL(18,8),
                message         TEXT,
                strategy_id     VARCHAR(50),
                metadata        JSONB,
                created_at      TIMESTAMPTZ DEFAULT NOW(),
                updated_at      TIMESTAMPTZ DEFAULT NOW()
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            DataEngineError::DatabaseError(format!("Failed to create trade_orders table: {}", e))
        })?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_orders_status ON trade_orders(status, created_at DESC)",
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DataEngineError::DatabaseError(format!("Failed index: {}", e)))?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_orders_exchange_symbol ON trade_orders(exchange, symbol)")
            .execute(&self.pool)
            .await
            .map_err(|e| DataEngineError::DatabaseError(format!("Failed index: {}", e)))?;

        // trade_executions
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS trade_executions (
                id              BIGSERIAL PRIMARY KEY,
                execution_id    TEXT UNIQUE NOT NULL,
                order_id        TEXT REFERENCES trade_orders(order_id),
                price           DECIMAL(24,8) NOT NULL,
                quantity        DECIMAL(24,8) NOT NULL,
                commission      DECIMAL(18,8),
                commission_asset VARCHAR(20),
                trade_time      TIMESTAMPTZ NOT NULL,
                created_at      TIMESTAMPTZ DEFAULT NOW()
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            DataEngineError::DatabaseError(format!(
                "Failed to create trade_executions table: {}",
                e
            ))
        })?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_executions_order ON trade_executions(order_id)",
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DataEngineError::DatabaseError(format!("Failed index: {}", e)))?;

        // trade_positions
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS trade_positions (
                id              BIGSERIAL PRIMARY KEY,
                account_id      VARCHAR(50) NOT NULL,
                exchange        VARCHAR(50) NOT NULL,
                symbol          VARCHAR(50) NOT NULL,
                quantity        DECIMAL(24,8) NOT NULL,
                avg_price       DECIMAL(24,8) NOT NULL,
                current_price   DECIMAL(24,8),
                unrealized_pnl  DECIMAL(24,8),
                updated_at      TIMESTAMPTZ DEFAULT NOW(),
                UNIQUE(account_id, exchange, symbol)
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            DataEngineError::DatabaseError(format!("Failed to create trade_positions table: {}", e))
        })?;

        // trade_accounts
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS trade_accounts (
                account_id      VARCHAR(50) PRIMARY KEY,
                exchange        VARCHAR(50) NOT NULL,
                currency        VARCHAR(10) NOT NULL,
                total_balance   DECIMAL(24,8),
                available_balance DECIMAL(24,8),
                updated_at      TIMESTAMPTZ DEFAULT NOW()
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            DataEngineError::DatabaseError(format!("Failed to create trade_accounts table: {}", e))
        })?;

        info!("Trading system tables created successfully");
        Ok(())
    }

    async fn create_metrics_table(&self) -> Result<(), DataEngineError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS api_usage_metrics (
                timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                provider TEXT NOT NULL,
                endpoint TEXT,
                request_count BIGINT NOT NULL,
                metadata JSONB
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            DataEngineError::DatabaseError(format!(
                "Failed to create api_usage_metrics table: {}",
                e
            ))
        })?;

        // Convert to hypertable for TimescaleDB efficiency
        // We use a separate query or handle error if it's not timescaledb or already exists
        let _ = sqlx::query(
            "SELECT create_hypertable('api_usage_metrics', 'timestamp', if_not_exists => TRUE)",
        )
        .execute(&self.pool)
        .await;

        // Create index for query performance
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_api_usage_metrics_provider_timestamp ON api_usage_metrics (provider, timestamp DESC)")
            .execute(&self.pool)
            .await
            .map_err(|e| DataEngineError::DatabaseError(format!("Failed to create index: {}", e)))?;

        Ok(())
    }

    async fn create_factors_table(&self) -> Result<(), DataEngineError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS factors (
                id SERIAL PRIMARY KEY,
                name VARCHAR(200) NOT NULL,
                slug VARCHAR(200) NOT NULL UNIQUE,
                category VARCHAR(100) NOT NULL,
                rust_function VARCHAR(500),
                formula TEXT NOT NULL,
                latex_formula TEXT,
                description TEXT NOT NULL,
                interpretation TEXT,
                parameters JSONB DEFAULT '[]',
                examples JSONB,
                output_range TEXT,
                normalization VARCHAR(50),
                computation_cost VARCHAR(20),
                min_bars_required INTEGER DEFAULT 0,
                tags TEXT[],
                refs JSONB,
                is_active BOOLEAN DEFAULT true,
                version INTEGER DEFAULT 1,
                created_at TIMESTAMPTZ DEFAULT NOW(),
                updated_at TIMESTAMPTZ DEFAULT NOW()
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            DataEngineError::DatabaseError(format!("Failed to create factors table: {}", e))
        })?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_factors_category ON factors(category)")
            .execute(&self.pool)
            .await
            .map_err(|e| DataEngineError::DatabaseError(format!("Failed index: {}", e)))?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_factors_slug ON factors(slug)")
            .execute(&self.pool)
            .await
            .map_err(|e| DataEngineError::DatabaseError(format!("Failed index: {}", e)))?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_factors_active ON factors(is_active)")
            .execute(&self.pool)
            .await
            .map_err(|e| DataEngineError::DatabaseError(format!("Failed index: {}", e)))?;

        Ok(())
    }

    async fn create_backtest_results_table(&self) -> Result<(), DataEngineError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS backtest_results (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                strategy_id VARCHAR(255),
                genome INTEGER[],
                token_address VARCHAR(255) NOT NULL,
                start_time TIMESTAMPTZ,
                end_time TIMESTAMPTZ,
                pnl_percent DOUBLE PRECISION,
                win_rate DOUBLE PRECISION,
                sharpe_ratio DOUBLE PRECISION,
                max_drawdown DOUBLE PRECISION,
                total_trades INTEGER,
                equity_curve JSONB,
                trades JSONB,
                created_at TIMESTAMPTZ DEFAULT NOW(),
                metadata JSONB
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            DataEngineError::DatabaseError(format!(
                "Failed to create backtest_results table: {}",
                e
            ))
        })?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_backtest_created_at ON backtest_results(created_at DESC)",
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DataEngineError::DatabaseError(format!("Failed index: {}", e)))?;

        Ok(())
    }
}
