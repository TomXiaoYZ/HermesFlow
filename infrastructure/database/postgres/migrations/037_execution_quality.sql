-- P6-2C: Order execution quality metrics.
-- Tracks slippage, fill rate, and latency for every filled order.
-- Used for promotion criteria and ongoing monitoring.

CREATE TABLE IF NOT EXISTS execution_quality (
    id BIGSERIAL PRIMARY KEY,
    order_id TEXT NOT NULL,
    exchange TEXT NOT NULL,
    symbol TEXT NOT NULL,
    side TEXT NOT NULL CHECK (side IN ('buy', 'sell')),
    expected_price DOUBLE PRECISION NOT NULL,
    fill_price DOUBLE PRECISION NOT NULL,
    slippage_bps DOUBLE PRECISION NOT NULL,
    quantity DOUBLE PRECISION NOT NULL,
    fill_rate DOUBLE PRECISION NOT NULL,
    latency_ms BIGINT NOT NULL,
    broker TEXT NOT NULL,
    account_id TEXT,
    mode TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for per-strategy quality analysis
CREATE INDEX IF NOT EXISTS idx_execution_quality_strategy
    ON execution_quality (exchange, symbol, mode, created_at DESC);

-- Index for per-broker quality monitoring
CREATE INDEX IF NOT EXISTS idx_execution_quality_broker
    ON execution_quality (broker, created_at DESC);

COMMENT ON TABLE execution_quality IS
    'P6-2C: Per-fill execution quality metrics for slippage tracking and promotion criteria';
COMMENT ON COLUMN execution_quality.slippage_bps IS
    'Slippage in basis points: (fill_price - expected_price) / expected_price * 10000';
COMMENT ON COLUMN execution_quality.fill_rate IS
    'Fraction of requested quantity actually filled (0.0 to 1.0)';
COMMENT ON COLUMN execution_quality.latency_ms IS
    'Time from order submission to fill confirmation in milliseconds';
