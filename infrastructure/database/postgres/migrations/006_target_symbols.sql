-- Refined schema for Data Engine target symbols
-- Supports multi-exchange routing and option collection toggles

CREATE TABLE IF NOT EXISTS data_engine_target_symbols (
    id SERIAL PRIMARY KEY,
    
    -- Core Identity
    symbol VARCHAR(50) NOT NULL,            -- e.g., 'TSLA', 'BTC-USDT', '600519'
    exchange VARCHAR(50) NOT NULL,          -- e.g., 'SMART' (IBKR), 'BINANCE', 'SSE' (Shanghai)
    market_region VARCHAR(50) NOT NULL,     -- e.g., 'US', 'CN', 'HK', 'CRYPTO_GLOBAL'
    asset_type VARCHAR(20) DEFAULT 'STK',   -- 'STK', 'CRYPTO', 'FUT'
    
    -- Routing & Source
    data_source VARCHAR(50) NOT NULL,       -- e.g., 'IBKR', 'CCXT', 'TUSHARE'
    currency VARCHAR(10) DEFAULT 'USD',
    
    -- Collection Configuration
    is_active BOOLEAN DEFAULT TRUE,
    collect_options BOOLEAN DEFAULT FALSE,  -- Enable underlying option chain collection
    collection_frequency VARCHAR(20) DEFAULT 'REALTIME', -- 'REALTIME', '1M', '1D'
    
    -- Metadata
    description VARCHAR(255),
    priority INTEGER DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    -- Constraint to prevent duplicates
    UNIQUE(symbol, exchange, asset_type)
);

-- Indices for performance
CREATE INDEX IF NOT EXISTS idx_de_targets_active_source 
    ON data_engine_target_symbols(is_active, data_source);

-- Initial Seed Data: US Tech Stocks (IBKR)
INSERT INTO data_engine_target_symbols 
(symbol, exchange, market_region, asset_type, data_source, collect_options, description) 
VALUES 
('TSLA', 'SMART', 'US', 'STK', 'IBKR', true, 'Tesla Inc.'),
('AAPL', 'SMART', 'US', 'STK', 'IBKR', true, 'Apple Inc.'),
('NVDA', 'SMART', 'US', 'STK', 'IBKR', true, 'Nvidia Corp.')
ON CONFLICT (symbol, exchange, asset_type) DO NOTHING;
