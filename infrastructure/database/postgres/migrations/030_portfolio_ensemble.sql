-- P5: Portfolio Ensemble & HRP Allocation
-- Stores rebalance snapshots, per-strategy weights, and shadow equity tracking.

-- One row per rebalance event
CREATE TABLE IF NOT EXISTS portfolio_ensembles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    exchange TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    strategy_count INTEGER NOT NULL,
    portfolio_oos_psr DOUBLE PRECISION,
    portfolio_sharpe DOUBLE PRECISION,
    portfolio_max_drawdown DOUBLE PRECISION,
    avg_pairwise_correlation DOUBLE PRECISION,
    crowded_pair_count INTEGER DEFAULT 0,
    weights JSONB NOT NULL,
    correlation_matrix JSONB,
    hrp_diagnostics JSONB,
    metadata JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(exchange, version)
);

-- Per-strategy detail within each ensemble
CREATE TABLE IF NOT EXISTS portfolio_ensemble_strategies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ensemble_id UUID NOT NULL REFERENCES portfolio_ensembles(id) ON DELETE CASCADE,
    exchange TEXT NOT NULL,
    symbol TEXT NOT NULL,
    mode TEXT NOT NULL,
    generation INTEGER NOT NULL,
    strategy_id TEXT NOT NULL,
    hrp_weight DOUBLE PRECISION NOT NULL,
    psr_factor DOUBLE PRECISION NOT NULL DEFAULT 1.0,
    utilization_factor DOUBLE PRECISION NOT NULL DEFAULT 1.0,
    crowding_penalty DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    final_weight DOUBLE PRECISION NOT NULL,
    oos_psr DOUBLE PRECISION,
    is_fitness DOUBLE PRECISION,
    utilization DOUBLE PRECISION,
    genome INTEGER[],
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Shadow portfolio equity tracking
CREATE TABLE IF NOT EXISTS portfolio_ensemble_equity (
    id BIGSERIAL PRIMARY KEY,
    exchange TEXT NOT NULL,
    ensemble_version INTEGER NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    equity DOUBLE PRECISION NOT NULL,
    period_return DOUBLE PRECISION,
    metadata JSONB,
    UNIQUE(exchange, ensemble_version, timestamp)
);

-- Indexes for common query patterns
CREATE INDEX IF NOT EXISTS idx_ensemble_strategies_ensemble_id
    ON portfolio_ensemble_strategies(ensemble_id);

CREATE INDEX IF NOT EXISTS idx_ensemble_equity_lookup
    ON portfolio_ensemble_equity(exchange, ensemble_version, timestamp DESC);
