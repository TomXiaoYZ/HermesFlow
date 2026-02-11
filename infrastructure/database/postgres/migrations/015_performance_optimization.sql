-- ============================================================
-- Performance Optimization Migration
-- Version: 050
-- Adds: Hypertable conversion, indexes, compression policies,
--        chunk tuning, continuous aggregates for 1h/1d
-- ============================================================

-- ============================================================
-- 1. CONVERT TO HYPERTABLES (idempotent)
-- ============================================================

-- mkt_equity_candles: 7-day chunks for OHLCV data
SELECT create_hypertable('mkt_equity_candles', 'time',
    migrate_data => true,
    chunk_time_interval => INTERVAL '7 days',
    if_not_exists => true);

-- mkt_equity_snapshots: 1-day chunks for high-frequency tick data
SELECT create_hypertable('mkt_equity_snapshots', 'timestamp',
    migrate_data => true,
    chunk_time_interval => INTERVAL '1 day',
    if_not_exists => true);

-- ============================================================
-- 2. INDEXES
-- ============================================================

-- Snapshots: symbol lookup index for queries by symbol
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_mkt_snapshots_symbol
ON mkt_equity_snapshots (symbol, "timestamp" DESC);

-- Watchlist: priority queue for sync task ordering
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_market_watchlist_priority_queue
ON market_watchlist (is_active, priority DESC, exchange)
WHERE is_active = true;

-- Active tokens: discovery workflow index
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_active_tokens_discovery
ON active_tokens (is_active, liquidity_usd DESC, last_updated DESC)
WHERE is_active = true;

-- ============================================================
-- 3. COMPRESSION POLICIES
-- ============================================================

-- Enable compression on candles
ALTER TABLE mkt_equity_candles SET (
    timescaledb.compress,
    timescaledb.compress_segmentby = 'exchange, symbol, resolution',
    timescaledb.compress_orderby = 'time DESC'
);

-- Enable compression on snapshots
ALTER TABLE mkt_equity_snapshots SET (
    timescaledb.compress,
    timescaledb.compress_segmentby = 'exchange, symbol',
    timescaledb.compress_orderby = 'timestamp DESC'
);

-- Auto-compress candles older than 7 days
SELECT add_compression_policy('mkt_equity_candles', INTERVAL '7 days',
    if_not_exists => true);

-- Auto-compress snapshots older than 3 days
SELECT add_compression_policy('mkt_equity_snapshots', INTERVAL '3 days',
    if_not_exists => true);

-- ============================================================
-- 4. DATA RETENTION POLICIES
-- ============================================================

-- Drop snapshot chunks older than 90 days
SELECT add_retention_policy('mkt_equity_snapshots', INTERVAL '90 days',
    if_not_exists => true);

-- ============================================================
-- 5. CONTINUOUS AGGREGATES
-- ============================================================

-- 1-hour candles from 1m data
CREATE MATERIALIZED VIEW IF NOT EXISTS mkt_equity_candles_1h
WITH (timescaledb.continuous) AS
SELECT
    time_bucket('1 hour', time) AS time,
    exchange,
    symbol,
    '1h'::text AS resolution,
    first(open, time) AS open,
    max(high) AS high,
    min(low) AS low,
    last(close, time) AS close,
    sum(volume) AS volume,
    last(liquidity, time) AS liquidity,
    last(fdv, time) AS fdv,
    sum(amount) AS amount
FROM mkt_equity_candles
WHERE resolution = '1m'
GROUP BY time_bucket('1 hour', time), exchange, symbol
WITH NO DATA;

SELECT add_continuous_aggregate_policy('mkt_equity_candles_1h',
    start_offset => INTERVAL '4 hours',
    end_offset => INTERVAL '15 minutes',
    schedule_interval => INTERVAL '15 minutes',
    if_not_exists => true);

-- 1-day candles from 1m data
CREATE MATERIALIZED VIEW IF NOT EXISTS mkt_equity_candles_1d
WITH (timescaledb.continuous) AS
SELECT
    time_bucket('1 day', time) AS time,
    exchange,
    symbol,
    '1d'::text AS resolution,
    first(open, time) AS open,
    max(high) AS high,
    min(low) AS low,
    last(close, time) AS close,
    sum(volume) AS volume,
    last(liquidity, time) AS liquidity,
    last(fdv, time) AS fdv,
    sum(amount) AS amount
FROM mkt_equity_candles
WHERE resolution = '1m'
GROUP BY time_bucket('1 day', time), exchange, symbol
WITH NO DATA;

SELECT add_continuous_aggregate_policy('mkt_equity_candles_1d',
    start_offset => INTERVAL '3 days',
    end_offset => INTERVAL '1 hour',
    schedule_interval => INTERVAL '1 hour',
    if_not_exists => true);
