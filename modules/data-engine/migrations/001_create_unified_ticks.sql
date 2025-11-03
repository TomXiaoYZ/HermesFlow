-- Unified ticks table for all market data sources
-- This table stores all market data (trades, tickers, klines, etc.) from all sources

CREATE TABLE IF NOT EXISTS unified_ticks (
    -- Identifiers
    source LowCardinality(String),       -- 'BinanceSpot', 'OkxFutures', etc.
    exchange LowCardinality(String),     -- 'Binance', 'OKX', 'IBKR', etc.
    symbol String,                        -- 'BTCUSDT', 'ETHUSDT', etc.
    asset_type LowCardinality(String),   -- 'Spot', 'Perpetual', 'Future', 'Option', 'Stock'
    data_type LowCardinality(String),    -- 'Trade', 'Ticker', 'Kline', 'OrderBook', 'FundingRate'
    
    -- Market data (using Decimal for precision)
    price Decimal(32, 8),                 -- Price with 8 decimal precision
    quantity Decimal(32, 8),              -- Volume/quantity
    timestamp DateTime64(3),              -- Exchange timestamp (millisecond precision)
    received_at DateTime64(3),            -- System received timestamp
    
    -- Optional fields (nullable)
    bid Nullable(Decimal(32, 8)),         -- Best bid price
    ask Nullable(Decimal(32, 8)),         -- Best ask price
    high_24h Nullable(Decimal(32, 8)),    -- 24-hour high
    low_24h Nullable(Decimal(32, 8)),     -- 24-hour low
    volume_24h Nullable(Decimal(32, 8)),  -- 24-hour volume
    open_interest Nullable(Decimal(32, 8)), -- Open interest (futures)
    funding_rate Nullable(Decimal(32, 8)),  -- Funding rate (perpetuals)
    
    -- Metadata
    sequence_id Nullable(UInt64),         -- Message sequence ID for ordering
    raw_data String,                      -- Original message (for debugging)
    
    -- Ingestion metadata
    ingested_at DateTime64(3) DEFAULT now64(3)  -- When row was inserted
) ENGINE = MergeTree()
PARTITION BY toYYYYMMDD(timestamp)        -- Daily partitions for efficient querying and retention
ORDER BY (source, symbol, timestamp)       -- Sorted by source, symbol, and time
SETTINGS index_granularity = 8192;

-- Create index for fast symbol+timestamp lookups
CREATE INDEX IF NOT EXISTS idx_symbol_timestamp ON unified_ticks (symbol, timestamp) TYPE minmax GRANULARITY 4;

-- Create index for fast data_type filtering
CREATE INDEX IF NOT EXISTS idx_data_type ON unified_ticks (data_type) TYPE set(10) GRANULARITY 4;

-- Comments for documentation
-- ALTER TABLE unified_ticks COMMENT COLUMN source 'Data source type (e.g., BinanceSpot, OkxFutures)';
-- ALTER TABLE unified_ticks COMMENT COLUMN exchange 'Exchange name (e.g., Binance, OKX)';
-- ALTER TABLE unified_ticks COMMENT COLUMN symbol 'Trading pair symbol (e.g., BTCUSDT)';
-- ALTER TABLE unified_ticks COMMENT COLUMN asset_type 'Asset classification (Spot, Perpetual, Future, etc.)';
-- ALTER TABLE unified_ticks COMMENT COLUMN data_type 'Data classification (Trade, Ticker, Kline, etc.)';






