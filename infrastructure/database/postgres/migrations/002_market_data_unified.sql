-- ============================================================
-- Unified Market Data Schema
-- Version: 2.0.0
-- Includes: Candles, Snapshots, Factors, Target Symbols
-- ============================================================

-- Enable TimescaleDB Extension (Idempotent)
CREATE EXTENSION IF NOT EXISTS timescaledb;

-- 1. Configuration: Target Symbols
CREATE TABLE IF NOT EXISTS data_engine_target_symbols (
    id SERIAL PRIMARY KEY,
    symbol VARCHAR(50) NOT NULL,            -- e.g., 'TSLA', 'BTC-USDT'
    exchange VARCHAR(50) NOT NULL,          -- e.g., 'BINANCE', 'IBKR'
    market_region VARCHAR(50) NOT NULL,     -- 'US', 'CN', 'HK', 'CRYPTO'
    asset_type VARCHAR(20) DEFAULT 'STK',   -- 'STK', 'CRYPTO', 'FUT', 'OPT'
    data_source VARCHAR(50) NOT NULL,       -- 'IBKR', 'CCXT', 'AKSHARE'
    is_active BOOLEAN DEFAULT TRUE,
    collect_options BOOLEAN DEFAULT FALSE,
    collection_frequency VARCHAR(20) DEFAULT 'REALTIME',
    priority INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(symbol, exchange, asset_type)
);

-- 2. Market Data: Candles (OHLCV)
CREATE TABLE IF NOT EXISTS mkt_equity_candles (
    time        TIMESTAMPTZ NOT NULL,
    exchange    VARCHAR(50) NOT NULL,
    symbol      VARCHAR(20) NOT NULL,
    resolution  VARCHAR(10) NOT NULL, -- '1m', '5m', '1h', '1d'
    
    open        DECIMAL(24,8) NOT NULL,
    high        DECIMAL(24,8) NOT NULL,
    low         DECIMAL(24,8) NOT NULL,
    close       DECIMAL(24,8) NOT NULL,
    volume      DECIMAL(24,8) NOT NULL,
    amount      DECIMAL(32,8),
    
    metadata    JSONB, -- VWAP, TradeCount, etc.
    created_at  TIMESTAMPTZ DEFAULT NOW(),
    
    -- Constraint for Upsert (Duplicate handling)
    UNIQUE(exchange, symbol, resolution, time)
);

-- Convert to Hypertable (Partition by time)
SELECT create_hypertable('mkt_equity_candles', 'time', if_not_exists => TRUE);

-- Enable Compression (Segment by symbol/exchange)
ALTER TABLE mkt_equity_candles SET (
    timescaledb.compress,
    timescaledb.compress_segmentby = 'exchange, symbol, resolution'
);

-- 3. Market Data: Snapshots (Tick/Quote)
-- Designed for high-frequency data, heavily compressed
CREATE TABLE IF NOT EXISTS mkt_equity_snapshots (
    time        TIMESTAMPTZ NOT NULL,
    exchange    VARCHAR(50) NOT NULL,
    symbol      VARCHAR(20) NOT NULL,
    
    price       DECIMAL(24,8),
    bid         DECIMAL(24,8),
    ask         DECIMAL(24,8),
    bid_size    DECIMAL(24,8),
    ask_size    DECIMAL(24,8),
    volume      DECIMAL(24,8), -- Cumulative volume
    
    -- Greeks (Optional for Options)
    iv          DECIMAL(10,4),
    delta       DECIMAL(10,4),
    
    received_at TIMESTAMPTZ DEFAULT NOW()
);

SELECT create_hypertable('mkt_equity_snapshots', 'time', if_not_exists => TRUE);

ALTER TABLE mkt_equity_snapshots SET (
    timescaledb.compress,
    timescaledb.compress_segmentby = 'exchange, symbol',
    timescaledb.compress_orderby = 'time DESC'
);

-- 4. Market Data: Factors (Derived Data)
-- Efficient storage for massive calculated factors
CREATE TABLE IF NOT EXISTS mkt_factors (
    time        TIMESTAMPTZ NOT NULL,
    exchange    VARCHAR(50) NOT NULL,
    symbol      VARCHAR(20) NOT NULL,
    resolution  VARCHAR(10) NOT NULL, -- '1m', '1d'
    group_name  VARCHAR(50) NOT NULL, -- e.g., 'MOMENTUM', 'VOLATILITY'
    
    -- Storing factors as JSONB for flexibility vs columns for strict schema
    -- Given 'lowest cost', defining columns is better for compression, 
    -- but JSONB is accepted for now as per discussion to support 'massive' variety.
    factors     JSONB, 
    
    UNIQUE(exchange, symbol, resolution, group_name, time)
);

SELECT create_hypertable('mkt_factors', 'time', if_not_exists => TRUE);

ALTER TABLE mkt_factors SET (
    timescaledb.compress,
    timescaledb.compress_segmentby = 'exchange, symbol, resolution, group_name'
);

-- Indices
CREATE INDEX IF NOT EXISTS idx_mkt_candles_lookup ON mkt_equity_candles (exchange, symbol, time DESC);
CREATE INDEX IF NOT EXISTS idx_mkt_snapshots_lookup ON mkt_equity_snapshots (exchange, symbol, time DESC);

-- Register Migration
INSERT INTO schema_migrations (version, description)
VALUES ('002_market_data_unified', 'Consolidated Market Data Schema (Candles, Snapshots, Factors)')
ON CONFLICT (version) DO NOTHING;
