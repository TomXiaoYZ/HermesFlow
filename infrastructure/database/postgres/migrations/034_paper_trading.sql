-- P6b-B2: Paper Trading Tables
-- Tracks simulated orders, executions, positions, and daily summaries
-- for strategies promoted from the ensemble to paper trading.

CREATE TABLE IF NOT EXISTS paper_trade_orders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id TEXT NOT NULL UNIQUE,
    exchange TEXT NOT NULL,
    symbol TEXT NOT NULL,
    side TEXT NOT NULL,
    quantity DECIMAL(24,8) NOT NULL,
    order_type TEXT NOT NULL DEFAULT 'market',
    limit_price DECIMAL(24,8),
    filled_qty DECIMAL(24,8) NOT NULL DEFAULT 0,
    avg_price DECIMAL(24,8) NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'submitted',
    strategy_id TEXT,
    mode TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_paper_orders_symbol
    ON paper_trade_orders(exchange, symbol, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_paper_orders_status
    ON paper_trade_orders(status)
    WHERE status IN ('submitted', 'partial');

CREATE TABLE IF NOT EXISTS paper_trade_executions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id TEXT NOT NULL REFERENCES paper_trade_orders(order_id),
    execution_id TEXT NOT NULL UNIQUE,
    price DECIMAL(24,8) NOT NULL,
    quantity DECIMAL(24,8) NOT NULL,
    commission DECIMAL(24,8) NOT NULL DEFAULT 0,
    executed_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS paper_positions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    exchange TEXT NOT NULL,
    symbol TEXT NOT NULL,
    quantity DECIMAL(24,8) NOT NULL DEFAULT 0,
    avg_cost DECIMAL(24,8) NOT NULL DEFAULT 0,
    unrealized_pnl DECIMAL(24,8) NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(exchange, symbol)
);

CREATE TABLE IF NOT EXISTS paper_daily_summary (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    exchange TEXT NOT NULL,
    date DATE NOT NULL,
    starting_equity DECIMAL(24,8) NOT NULL,
    ending_equity DECIMAL(24,8) NOT NULL,
    realized_pnl DECIMAL(24,8) NOT NULL DEFAULT 0,
    unrealized_pnl DECIMAL(24,8) NOT NULL DEFAULT 0,
    total_trades INTEGER NOT NULL DEFAULT 0,
    total_commission DECIMAL(24,8) NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(exchange, date)
);

CREATE INDEX IF NOT EXISTS idx_paper_daily_exchange
    ON paper_daily_summary(exchange, date DESC);
