-- Migration 017: Add deduplication index for snapshots
-- Prevents duplicate ticks from replayed data with identical timestamps.
-- TimescaleDB requires the partitioning column (time) in unique indexes.

CREATE UNIQUE INDEX IF NOT EXISTS idx_snapshot_dedup
    ON mkt_equity_snapshots (exchange, symbol, time);
