-- ============================================================
-- Consolidated Database Schema
-- Version: 4.0.0 (Auto-generated)
-- Date: 2026-02-11
--
-- This file is auto-generated from all migration files.
-- Do NOT edit manually. Instead, create a new migration.
--
-- Includes: Core Schema, Market Data, Active Tokens, Trading System,
--           Factors Library, Backtest Results, Watchlists, API Metrics,
--           Performance Optimizations
-- ============================================================

-- ============================================================
-- 0. Core Extensions & Setup
-- ============================================================
CREATE EXTENSION IF NOT EXISTS timescaledb;

-- Utility function to auto-update updated_at columns
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE 'plpgsql';

-- ============================================================
-- 1. Social Data: Tweets
-- ============================================================
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
);

CREATE INDEX IF NOT EXISTS idx_tweets_username ON tweets(username);
CREATE INDEX IF NOT EXISTS idx_tweets_created_at ON tweets(created_at);
CREATE INDEX IF NOT EXISTS idx_tweets_followers ON tweets(followers_count) WHERE followers_count IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_tweets_hashtags ON tweets USING GIN(hashtags);

CREATE TABLE IF NOT EXISTS twitter_collection_runs (
    id SERIAL PRIMARY KEY,
    target TEXT NOT NULL,
    collected_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    scraped_count INTEGER NOT NULL,
    upserted_count INTEGER NOT NULL,
    error TEXT
);

-- ============================================================
-- 2. Prediction Markets
-- ============================================================
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
);

CREATE INDEX IF NOT EXISTS idx_markets_source ON prediction_markets(source);
CREATE INDEX IF NOT EXISTS idx_markets_active ON prediction_markets(active);
CREATE INDEX IF NOT EXISTS idx_markets_end_date ON prediction_markets(end_date) WHERE end_date IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_markets_category ON prediction_markets(category) WHERE category IS NOT NULL;

DROP TRIGGER IF EXISTS update_prediction_markets_updated_at ON prediction_markets;
CREATE TRIGGER update_prediction_markets_updated_at
    BEFORE UPDATE ON prediction_markets
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TABLE IF NOT EXISTS market_outcomes (
    id SERIAL PRIMARY KEY,
    market_id TEXT NOT NULL REFERENCES prediction_markets(id) ON DELETE CASCADE,
    outcome TEXT NOT NULL,
    price DECIMAL(18, 8) NOT NULL,
    volume_24h DECIMAL(18, 8),
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(market_id, outcome, timestamp)
);

CREATE INDEX IF NOT EXISTS idx_outcomes_market_id ON market_outcomes(market_id);
CREATE INDEX IF NOT EXISTS idx_outcomes_timestamp ON market_outcomes(timestamp);
CREATE INDEX IF NOT EXISTS idx_outcomes_market_outcome ON market_outcomes(market_id, outcome);

-- ============================================================
-- 3. Active Tokens (Crypto token discovery)
-- ============================================================
CREATE TABLE IF NOT EXISTS active_tokens (
    address TEXT PRIMARY KEY,
    symbol TEXT NOT NULL,
    name TEXT,
    decimals INTEGER NOT NULL DEFAULT 9,
    chain TEXT NOT NULL DEFAULT 'solana',
    liquidity_usd DECIMAL(40, 8),
    fdv DECIMAL(40, 8),
    market_cap DECIMAL(40, 8),
    volume_24h DECIMAL(40, 8),
    price_change_24h DECIMAL(8,4),
    first_discovered TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_updated TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_active BOOLEAN NOT NULL DEFAULT true,
    metadata JSONB
);

CREATE INDEX IF NOT EXISTS idx_active_tokens_active ON active_tokens(is_active) WHERE is_active = true;
CREATE INDEX IF NOT EXISTS idx_active_tokens_updated ON active_tokens(last_updated DESC);
CREATE INDEX IF NOT EXISTS idx_active_tokens_liquidity ON active_tokens(liquidity_usd DESC) WHERE is_active = true;

CREATE OR REPLACE FUNCTION update_active_tokens_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.last_updated = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trigger_update_active_tokens_timestamp ON active_tokens;
CREATE TRIGGER trigger_update_active_tokens_timestamp
    BEFORE UPDATE ON active_tokens
    FOR EACH ROW
    EXECUTE FUNCTION update_active_tokens_timestamp();

-- Legacy target symbols table
CREATE TABLE IF NOT EXISTS data_engine_target_symbols (
    id SERIAL PRIMARY KEY,
    symbol VARCHAR(50) NOT NULL,
    exchange VARCHAR(50) NOT NULL,
    market_region VARCHAR(50) NOT NULL,
    asset_type VARCHAR(20) DEFAULT 'STK',
    data_source VARCHAR(50) NOT NULL,
    is_active BOOLEAN DEFAULT TRUE,
    collect_options BOOLEAN DEFAULT FALSE,
    collection_frequency VARCHAR(20) DEFAULT 'REALTIME',
    priority INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(symbol, exchange, asset_type)
);

-- ============================================================
-- 4. Market Data: Candles (OHLCV)
-- ============================================================
CREATE TABLE IF NOT EXISTS mkt_equity_candles (
    time        TIMESTAMPTZ NOT NULL,
    exchange    VARCHAR(50) NOT NULL,
    symbol      VARCHAR(20) NOT NULL,
    resolution  VARCHAR(10) NOT NULL,

    open        DECIMAL(24,8) NOT NULL,
    high        DECIMAL(24,8) NOT NULL,
    low         DECIMAL(24,8) NOT NULL,
    close       DECIMAL(24,8) NOT NULL,
    volume      DECIMAL(40,8) NOT NULL,
    amount      DECIMAL(40,8),

    metadata    JSONB,
    liquidity   DECIMAL(40,8),
    fdv         DECIMAL(40,8),

    created_at  TIMESTAMPTZ DEFAULT NOW(),

    UNIQUE(exchange, symbol, resolution, time)
);

SELECT create_hypertable('mkt_equity_candles', 'time',
    chunk_time_interval => INTERVAL '7 days',
    if_not_exists => true);

ALTER TABLE mkt_equity_candles SET (
    timescaledb.compress,
    timescaledb.compress_segmentby = 'exchange, symbol, resolution',
    timescaledb.compress_orderby = 'time DESC'
);

SELECT add_compression_policy('mkt_equity_candles', INTERVAL '7 days',
    if_not_exists => true);

CREATE INDEX IF NOT EXISTS idx_mkt_candles_lookup ON mkt_equity_candles (exchange, symbol, time DESC);

-- ============================================================
-- 5. Market Data: Snapshots (High Frequency Tick/Quote)
-- ============================================================
CREATE TABLE IF NOT EXISTS mkt_equity_snapshots (
    time        TIMESTAMPTZ NOT NULL,
    exchange    VARCHAR(50) NOT NULL DEFAULT 'birdeye',
    symbol      VARCHAR(20) NOT NULL,

    price       DECIMAL(24,8),
    bid         DECIMAL(24,8),
    ask         DECIMAL(24,8),
    bid_size    DECIMAL(24,8),
    ask_size    DECIMAL(24,8),
    volume      DECIMAL(24,8),

    vwap        DECIMAL(24,8),
    high        DECIMAL(24,8),
    low         DECIMAL(24,8),

    iv          DECIMAL(10,4),
    delta       DECIMAL(10,4),

    timestamp   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    received_at TIMESTAMPTZ DEFAULT NOW()
);

SELECT create_hypertable('mkt_equity_snapshots', 'timestamp',
    chunk_time_interval => INTERVAL '1 day',
    if_not_exists => true);

ALTER TABLE mkt_equity_snapshots SET (
    timescaledb.compress,
    timescaledb.compress_segmentby = 'exchange, symbol',
    timescaledb.compress_orderby = 'timestamp DESC'
);

SELECT add_compression_policy('mkt_equity_snapshots', INTERVAL '3 days',
    if_not_exists => true);

SELECT add_retention_policy('mkt_equity_snapshots', INTERVAL '90 days',
    if_not_exists => true);

CREATE INDEX IF NOT EXISTS idx_mkt_snapshots_lookup_v2 ON mkt_equity_snapshots (exchange, symbol, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_mkt_snapshots_symbol ON mkt_equity_snapshots (symbol, timestamp DESC);

-- ============================================================
-- 6. Market Data: Factors (Derived/Computed)
-- ============================================================
CREATE TABLE IF NOT EXISTS mkt_factors (
    time        TIMESTAMPTZ NOT NULL,
    exchange    VARCHAR(50) NOT NULL,
    symbol      VARCHAR(20) NOT NULL,
    resolution  VARCHAR(10) NOT NULL,
    group_name  VARCHAR(50) NOT NULL,
    factors     JSONB,
    UNIQUE(exchange, symbol, resolution, group_name, time)
);

SELECT create_hypertable('mkt_factors', 'time', if_not_exists => TRUE);

ALTER TABLE mkt_factors SET (
    timescaledb.compress,
    timescaledb.compress_segmentby = 'exchange, symbol, resolution, group_name'
);

-- ============================================================
-- 7. Trading System
-- ============================================================

-- Orders
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
);

CREATE INDEX IF NOT EXISTS idx_orders_status ON trade_orders(status, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_orders_exchange_symbol ON trade_orders(exchange, symbol);

-- Executions (Fills)
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
);

CREATE INDEX IF NOT EXISTS idx_executions_order ON trade_executions(order_id);

-- Positions
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
);

-- Accounts
CREATE TABLE IF NOT EXISTS trade_accounts (
    account_id      VARCHAR(50) PRIMARY KEY,
    exchange        VARCHAR(50) NOT NULL,
    currency        VARCHAR(10) NOT NULL,
    total_balance   DECIMAL(24,8),
    available_balance DECIMAL(24,8),
    updated_at      TIMESTAMPTZ DEFAULT NOW()
);

-- ============================================================
-- 8. Strategy Engine
-- ============================================================
CREATE TABLE IF NOT EXISTS strategy_generations (
    generation INTEGER PRIMARY KEY,
    fitness DOUBLE PRECISION,
    best_genome INTEGER[],
    timestamp TIMESTAMPTZ DEFAULT NOW(),
    metadata JSONB,
    strategy_id TEXT
);

CREATE INDEX IF NOT EXISTS idx_strategy_generations_id ON strategy_generations(strategy_id);

-- ============================================================
-- 9. Factors Library (Documentation/Registry)
-- ============================================================
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
);

CREATE INDEX IF NOT EXISTS idx_factors_category ON factors(category);
CREATE INDEX IF NOT EXISTS idx_factors_slug ON factors(slug);
CREATE INDEX IF NOT EXISTS idx_factors_tags ON factors USING GIN(tags);
CREATE INDEX IF NOT EXISTS idx_factors_active ON factors(is_active);

COMMENT ON TABLE factors IS 'Technical analysis factor library with formulas and documentation';

-- ============================================================
-- 10. Backtest Results
-- ============================================================
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
);

CREATE INDEX IF NOT EXISTS idx_backtest_created_at ON backtest_results(created_at DESC);

-- ============================================================
-- 11. API Usage Metrics
-- ============================================================
CREATE TABLE IF NOT EXISTS api_usage_metrics (
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    provider TEXT NOT NULL,
    endpoint TEXT,
    request_count BIGINT NOT NULL,
    metadata JSONB
);

SELECT create_hypertable('api_usage_metrics', 'timestamp', if_not_exists => TRUE);
CREATE INDEX IF NOT EXISTS idx_api_usage_metrics_provider_timestamp ON api_usage_metrics (provider, timestamp DESC);

-- ============================================================
-- 12. Universal Market Watchlist
-- ============================================================
CREATE TABLE IF NOT EXISTS market_watchlist (
    id SERIAL PRIMARY KEY,
    exchange VARCHAR(50) NOT NULL,
    symbol VARCHAR(50) NOT NULL,
    asset_type VARCHAR(20) NOT NULL,
    name TEXT,
    base_currency VARCHAR(20),
    quote_currency VARCHAR(20),
    enabled_1m BOOLEAN DEFAULT false,
    enabled_5m BOOLEAN DEFAULT false,
    enabled_15m BOOLEAN DEFAULT false,
    enabled_30m BOOLEAN DEFAULT false,
    enabled_1h BOOLEAN DEFAULT true,
    enabled_4h BOOLEAN DEFAULT true,
    enabled_1d BOOLEAN DEFAULT true,
    enabled_1w BOOLEAN DEFAULT false,
    is_active BOOLEAN DEFAULT true,
    priority INTEGER DEFAULT 50,
    sync_from_date DATE DEFAULT '2023-01-01',
    last_synced_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    notes TEXT,
    UNIQUE(exchange, symbol)
);

CREATE INDEX IF NOT EXISTS idx_market_watchlist_exchange_active ON market_watchlist(exchange, is_active) WHERE is_active = true;
CREATE INDEX IF NOT EXISTS idx_market_watchlist_asset_type ON market_watchlist(asset_type);
CREATE INDEX IF NOT EXISTS idx_market_watchlist_priority ON market_watchlist(priority DESC);
CREATE INDEX IF NOT EXISTS idx_market_watchlist_metadata ON market_watchlist USING GIN(metadata);

CREATE TRIGGER update_market_watchlist_timestamp
    BEFORE UPDATE ON market_watchlist
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Sync status tracking
CREATE TABLE IF NOT EXISTS market_sync_status (
    id SERIAL PRIMARY KEY,
    exchange VARCHAR(50) NOT NULL,
    symbol VARCHAR(50) NOT NULL,
    resolution VARCHAR(10) NOT NULL,
    last_synced_time TIMESTAMPTZ,
    total_candles INTEGER DEFAULT 0,
    last_sync_at TIMESTAMPTZ,
    status VARCHAR(20) DEFAULT 'pending',
    error_message TEXT,
    retry_count INTEGER DEFAULT 0,
    sync_duration_ms INTEGER,
    UNIQUE(exchange, symbol, resolution)
);

CREATE INDEX IF NOT EXISTS idx_market_sync_exchange_symbol ON market_sync_status(exchange, symbol);
CREATE INDEX IF NOT EXISTS idx_market_sync_status ON market_sync_status(status);
CREATE INDEX IF NOT EXISTS idx_market_sync_resolution ON market_sync_status(resolution);

-- ============================================================
-- 13. Performance Indexes (from migration 050)
-- ============================================================
CREATE INDEX IF NOT EXISTS idx_active_tokens_discovery
    ON active_tokens (is_active, liquidity_usd DESC, last_updated DESC)
    WHERE is_active = true;

-- ============================================================
-- 14. Views
-- ============================================================
CREATE OR REPLACE VIEW latest_market_prices AS
SELECT DISTINCT ON (market_id, outcome)
    market_id,
    outcome,
    price,
    volume_24h,
    timestamp
FROM market_outcomes
ORDER BY market_id, outcome, timestamp DESC;
