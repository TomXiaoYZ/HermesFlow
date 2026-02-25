-- P6b-F4: Ensemble Walk-Forward Backtest
-- Stores results of simulated historical ensemble performance.
-- Replays recorded ensemble allocations over market returns
-- to validate out-of-sample portfolio behavior.

CREATE TABLE IF NOT EXISTS ensemble_backtest_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    exchange TEXT NOT NULL,
    backtest_start TIMESTAMPTZ NOT NULL,
    backtest_end TIMESTAMPTZ NOT NULL,
    rebalance_count INTEGER NOT NULL,
    cumulative_return DOUBLE PRECISION,
    annualized_sharpe DOUBLE PRECISION,
    max_drawdown DOUBLE PRECISION,
    avg_turnover DOUBLE PRECISION,
    total_turnover_cost DOUBLE PRECISION,
    avg_strategy_count DOUBLE PRECISION,
    per_period_json JSONB,
    metadata JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Index for latest backtest per exchange
CREATE INDEX IF NOT EXISTS idx_ensemble_backtest_exchange
    ON ensemble_backtest_results(exchange, created_at DESC);
