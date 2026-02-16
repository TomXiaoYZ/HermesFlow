-- Migration 020: Add covering index for candle history API queries
-- The history endpoint filters (symbol, resolution, time range, exchange) but
-- existing index idx_mkt_candles_lookup is (exchange, symbol, time DESC)
-- which misses resolution and has suboptimal column order for the query pattern.
-- Note: TimescaleDB hypertables do not support CONCURRENTLY.

CREATE INDEX IF NOT EXISTS idx_mkt_candles_history_lookup
ON mkt_equity_candles (exchange, symbol, resolution, time DESC);
