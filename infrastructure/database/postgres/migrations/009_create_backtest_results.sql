CREATE TABLE IF NOT EXISTS backtest_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    strategy_id VARCHAR(255),
    genome INTEGER[], -- The strategy logic array
    token_address VARCHAR(255) NOT NULL,
    start_time TIMESTAMPTZ,
    end_time TIMESTAMPTZ,
    
    -- Metrics
    pnl_percent DOUBLE PRECISION,
    win_rate DOUBLE PRECISION,
    sharpe_ratio DOUBLE PRECISION,
    max_drawdown DOUBLE PRECISION,
    total_trades INTEGER,
    
    -- Visualization Data (JSON)
    equity_curve JSONB, -- [{time: ts, value: 1.05}, ...]
    trades JSONB,       -- [{entry_time, exit_time, pnl, direction}, ...]
    
    created_at TIMESTAMPTZ DEFAULT NOW(),
    metadata JSONB
);

CREATE INDEX idx_backtest_created_at ON backtest_results(created_at DESC);
