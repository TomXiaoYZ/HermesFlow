-- Materialized view for 1-minute OHLCV aggregates
-- This is an optimization for fast historical data queries at 1-minute resolution

CREATE MATERIALIZED VIEW IF NOT EXISTS unified_ticks_1m
ENGINE = AggregatingMergeTree()
PARTITION BY toYYYYMM(timestamp)
ORDER BY (source, symbol, timestamp)
POPULATE  -- Backfill existing data
AS SELECT
    source,
    exchange,
    symbol,
    asset_type,
    toStartOfMinute(timestamp) AS timestamp,
    
    -- OHLCV aggregations
    argMin(price, timestamp) AS open,   -- First price in minute
    max(price) AS high,
    min(price) AS low,
    argMax(price, timestamp) AS close,  -- Last price in minute
    sum(quantity) AS volume,
    
    -- Additional statistics
    count() AS ticks,
    avg(price) AS avg_price,
    
    -- Min/max timestamps in the minute
    min(timestamp) AS first_tick_at,
    max(timestamp) AS last_tick_at
FROM unified_ticks
WHERE data_type = 'Trade'  -- Only aggregate trades
GROUP BY source, exchange, symbol, asset_type, timestamp;

-- Create a table for storing the materialized view results
CREATE TABLE IF NOT EXISTS market_ohlcv_1m (
    source LowCardinality(String),
    exchange LowCardinality(String),
    symbol String,
    asset_type LowCardinality(String),
    timestamp DateTime,
    
    open Decimal(32, 8),
    high Decimal(32, 8),
    low Decimal(32, 8),
    close Decimal(32, 8),
    volume Decimal(32, 8),
    
    ticks UInt32,
    avg_price Decimal(32, 8),
    first_tick_at DateTime64(3),
    last_tick_at DateTime64(3)
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(timestamp)
ORDER BY (source, symbol, timestamp)
SETTINGS index_granularity = 8192;

-- Note: In production, you would typically use a separate aggregating table
-- and query it with the appropriate aggregate functions (e.g., -State, -Merge)






