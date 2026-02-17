-- Migration 023: Index to support timestamp-based ordering of generations
-- Fixes: orphaned high-generation-number rows causing incorrect API results
-- when using ORDER BY generation DESC. Switching to ORDER BY timestamp DESC
-- requires this composite index for efficient queries.

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_strategy_gens_timestamp
    ON strategy_generations (exchange, symbol, mode, timestamp DESC);
