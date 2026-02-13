-- Migration 016: Add missing columns and CHECK constraints for data integrity

-- Add missing columns to mkt_equity_snapshots
ALTER TABLE mkt_equity_snapshots ADD COLUMN IF NOT EXISTS bid_size DECIMAL(24,8);
ALTER TABLE mkt_equity_snapshots ADD COLUMN IF NOT EXISTS ask_size DECIMAL(24,8);
ALTER TABLE mkt_equity_snapshots ADD COLUMN IF NOT EXISTS received_at TIMESTAMPTZ;

-- Rename 'timestamp' to 'time' for consistency (matches hypertable chunk column)
-- Note: only rename if 'timestamp' exists and 'time' does not
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns
               WHERE table_name = 'mkt_equity_snapshots' AND column_name = 'timestamp')
       AND NOT EXISTS (SELECT 1 FROM information_schema.columns
               WHERE table_name = 'mkt_equity_snapshots' AND column_name = 'time')
    THEN
        ALTER TABLE mkt_equity_snapshots RENAME COLUMN "timestamp" TO "time";
    END IF;
END $$;

-- Clean up invalid data before adding constraints
DELETE FROM mkt_equity_snapshots WHERE price <= 0;

-- Increase decompression limit for DML on compressed hypertable chunks
SET timescaledb.max_tuples_decompressed_per_dml_transaction = 0;
DELETE FROM mkt_equity_candles WHERE open <= 0 OR high <= 0 OR low <= 0 OR close <= 0;
RESET timescaledb.max_tuples_decompressed_per_dml_transaction;

-- Snapshot CHECK constraints
ALTER TABLE mkt_equity_snapshots
    ADD CONSTRAINT chk_snapshot_price_positive CHECK (price > 0);

ALTER TABLE mkt_equity_snapshots
    ADD CONSTRAINT chk_snapshot_bid_ask CHECK (bid IS NULL OR ask IS NULL OR bid <= ask);

ALTER TABLE mkt_equity_snapshots
    ADD CONSTRAINT chk_snapshot_volume_nonneg CHECK (volume IS NULL OR volume >= 0);

-- Candle constraints
ALTER TABLE mkt_equity_candles
    ADD CONSTRAINT chk_candle_prices_positive CHECK (open > 0 AND high > 0 AND low > 0 AND close > 0);

ALTER TABLE mkt_equity_candles
    ADD CONSTRAINT chk_candle_ohlc_consistency CHECK (high >= GREATEST(open, close) AND low <= LEAST(open, close));

ALTER TABLE mkt_equity_candles
    ADD CONSTRAINT chk_candle_high_low CHECK (high >= low);

ALTER TABLE mkt_equity_candles
    ADD CONSTRAINT chk_candle_volume_nonneg CHECK (volume >= 0);
