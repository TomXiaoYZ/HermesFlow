-- Create Continuous Aggregate for 15-minute OHLCV Candles
-- TimescaleDB will automatically and efficiently maintain this materialized view
-- by aggregating mkt_equity_snapshots into mkt_equity_candles

CREATE MATERIALIZED VIEW IF NOT EXISTS mkt_equity_candles_15m
WITH (timescaledb.continuous) AS
SELECT
    symbol,
    '15m' AS resolution,
    time_bucket('15 minutes', timestamp) AS time,
    FIRST(price, timestamp) AS open,
    MAX(price) AS high,
    MIN(price) AS low,
    LAST(price, timestamp) AS close,
    SUM(volume) AS volume,
    AVG(liquidity) AS liquidity,
    AVG(fdv) AS fdv,
    AVG(amount) AS amount
FROM mkt_equity_snapshots
GROUP BY symbol, time_bucket('15 minutes', timestamp);

-- Add refresh policy: Update every 5 minutes for the last hour of data
SELECT add_continuous_aggregate_policy('mkt_equity_candles_15m',
    start_offset => INTERVAL '1 hour',
    end_offset => INTERVAL '1 minute',
    schedule_interval => INTERVAL '5 minutes');

-- Insert aggregated data into mkt_equity_candles table
-- (This is a one-time backfill for existing data)
INSERT INTO mkt_equity_candles (symbol, resolution, time, open, high, low, close, volume, liquidity, fdv, amount)
SELECT symbol, resolution, time, open, high, low, close, volume, liquidity, fdv, amount
FROM mkt_equity_candles_15m
ON CONFLICT (symbol, resolution, time) DO NOTHING;

-- Optional: Create trigger to automatically insert new candle data
-- (Future enhancement - for now continuous aggregate handles it)
