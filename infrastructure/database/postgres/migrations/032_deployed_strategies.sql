-- P6b-B1: Deployment Pipeline
-- Tracks which strategies are currently active for paper/live trading.
-- UPSERT'd by ensemble rebalance in strategy-generator.
-- Polled by strategy-engine for loading active formulas.

CREATE TABLE IF NOT EXISTS deployed_strategies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    exchange TEXT NOT NULL,
    symbol TEXT NOT NULL,
    mode TEXT NOT NULL,
    genome INTEGER[] NOT NULL,
    generation INTEGER NOT NULL,
    threshold_config JSONB,
    oos_psr DOUBLE PRECISION,
    is_fitness DOUBLE PRECISION,
    utilization DOUBLE PRECISION,
    final_weight DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    ensemble_version INTEGER NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    deployed_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(exchange, symbol, mode)
);

-- Index for strategy-engine polling active strategies
CREATE INDEX IF NOT EXISTS idx_deployed_strategies_active
    ON deployed_strategies(exchange, status)
    WHERE status = 'active';

-- Index for historical lookups
CREATE INDEX IF NOT EXISTS idx_deployed_strategies_updated
    ON deployed_strategies(updated_at DESC);
