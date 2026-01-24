-- ============================================================
-- Unified Trading System Schema
-- Version: 2.0.0
-- Includes: Orders, Executions, Positions, Accounts
-- ============================================================

-- 1. Orders (Standardized Execution)
-- Unified table for Spot, Futures, Options across all exchanges
CREATE TABLE IF NOT EXISTS trade_orders (
    id              BIGSERIAL PRIMARY KEY,
    order_id        TEXT UNIQUE NOT NULL, -- Internal UUID
    parent_order_id TEXT,                 -- For Algorithmic/Bracket orders
    
    exchange        VARCHAR(50) NOT NULL, -- 'BINANCE', 'IBKR'
    account_id      VARCHAR(50),          -- 'DU12345'
    
    symbol          VARCHAR(50) NOT NULL, -- 'BTC-USDT', 'AAPL'
    asset_type      VARCHAR(20) NOT NULL, -- 'STK', 'CRYPTO'
    
    side            VARCHAR(10) NOT NULL, -- 'BUY', 'SELL'
    order_type      VARCHAR(20) NOT NULL, -- 'LIMIT', 'MARKET', 'STOP'
    
    quantity        DECIMAL(24,8) NOT NULL,
    filled_qty      DECIMAL(24,8) DEFAULT 0,
    price           DECIMAL(24,8),        -- Limit Price
    avg_price       DECIMAL(24,8),        -- Executed Avg Price
    
    status          VARCHAR(20) NOT NULL DEFAULT 'NEW', -- NEW, PARTIALLY_FILLED, FILLED, CANCELED, REJECTED
    
    commission      DECIMAL(18,8),
    message         TEXT,                 -- Error or status message
    
    strategy_id     VARCHAR(50),          -- Which agent placed this?
    metadata        JSONB,                -- Exchange specific raw data (ClientOID etc)
    
    created_at      TIMESTAMPTZ DEFAULT NOW(),
    updated_at      TIMESTAMPTZ DEFAULT NOW()
);

-- Index for fast status lookup
CREATE INDEX IF NOT EXISTS idx_orders_status ON trade_orders(status, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_orders_exchange_symbol ON trade_orders(exchange, symbol);

-- 2. Executions (Fills)
-- Granular fill details
CREATE TABLE IF NOT EXISTS trade_executions (
    id              BIGSERIAL PRIMARY KEY,
    execution_id    TEXT UNIQUE NOT NULL, -- Exchange Trade ID
    order_id        TEXT REFERENCES trade_orders(order_id),
    
    price           DECIMAL(24,8) NOT NULL,
    quantity        DECIMAL(24,8) NOT NULL,
    commission      DECIMAL(18,8),
    commission_asset VARCHAR(20),
    
    trade_time      TIMESTAMPTZ NOT NULL,
    created_at      TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_executions_order ON trade_executions(order_id);

-- 3. Positions (Real-time View)
CREATE TABLE IF NOT EXISTS trade_positions (
    id              BIGSERIAL PRIMARY KEY,
    account_id      VARCHAR(50) NOT NULL,
    exchange        VARCHAR(50) NOT NULL,
    symbol          VARCHAR(50) NOT NULL,
    
    quantity        DECIMAL(24,8) NOT NULL, -- Negative for Short
    avg_price       DECIMAL(24,8) NOT NULL,
    
    current_price   DECIMAL(24,8),
    unrealized_pnl  DECIMAL(24,8),
    
    updated_at      TIMESTAMPTZ DEFAULT NOW(),
    
    UNIQUE(account_id, exchange, symbol)
);

-- 4. Accounts (Balance / Margin)
CREATE TABLE IF NOT EXISTS trade_accounts (
    account_id      VARCHAR(50) PRIMARY KEY,
    exchange        VARCHAR(50) NOT NULL,
    
    currency        VARCHAR(10) NOT NULL,
    total_balance   DECIMAL(24,8),
    available_balance DECIMAL(24,8),
    
    updated_at      TIMESTAMPTZ DEFAULT NOW()
);

-- Register Migration
INSERT INTO schema_migrations (version, description)
VALUES ('003_trading_system_unified', 'Consolidated Trading System Schema')
ON CONFLICT (version) DO NOTHING;
