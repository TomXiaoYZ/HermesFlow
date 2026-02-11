-- Universal Market Watchlist
-- Supports all exchanges: Polygon, Binance, Bybit, Birdeye, OKX, etc.

CREATE TABLE IF NOT EXISTS market_watchlist (
    id SERIAL PRIMARY KEY,
    
    -- Asset identification
    exchange VARCHAR(50) NOT NULL,      -- 'Polygon', 'Binance', 'Bybit', 'Birdeye', etc
    symbol VARCHAR(50) NOT NULL,        -- 'AAPL', 'BTC', 'SOL/USDT', token address, etc
    asset_type VARCHAR(20) NOT NULL,    -- 'stock', 'crypto', 'token', 'forex'
    
    -- Display info
    name TEXT,
    base_currency VARCHAR(20),          -- For pairs: 'BTC' in BTC/USDT
    quote_currency VARCHAR(20),         -- For pairs: 'USDT' in BTC/USDT
    
    -- Timeframe configuration (which resolutions to sync)
    enabled_1m BOOLEAN DEFAULT false,
    enabled_5m BOOLEAN DEFAULT false,
    enabled_15m BOOLEAN DEFAULT false,
    enabled_30m BOOLEAN DEFAULT false,
    enabled_1h BOOLEAN DEFAULT true,
    enabled_4h BOOLEAN DEFAULT true,
    enabled_1d BOOLEAN DEFAULT true,
    enabled_1w BOOLEAN DEFAULT false,
    
    -- Control
    is_active BOOLEAN DEFAULT true,
    priority INTEGER DEFAULT 50,        -- 1-100, higher = sync first
    
    -- Sync configuration
    sync_from_date DATE DEFAULT '2023-01-01',
    last_synced_at TIMESTAMPTZ,
    
    -- Metadata (exchange-specific data in JSON)
    metadata JSONB DEFAULT '{}',        -- {market_cap, sector, liquidity, fdv, etc}
    
    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    notes TEXT,
    
    -- Unique constraint per exchange
    UNIQUE(exchange, symbol)
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_market_watchlist_exchange_active 
    ON market_watchlist(exchange, is_active) WHERE is_active = true;
CREATE INDEX IF NOT EXISTS idx_market_watchlist_asset_type 
    ON market_watchlist(asset_type);
CREATE INDEX IF NOT EXISTS idx_market_watchlist_priority 
    ON market_watchlist(priority DESC);
CREATE INDEX IF NOT EXISTS idx_market_watchlist_metadata 
    ON market_watchlist USING GIN(metadata);

-- Auto-update timestamp trigger
CREATE TRIGGER update_market_watchlist_timestamp
BEFORE UPDATE ON market_watchlist
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

-- Market sync status tracking
CREATE TABLE IF NOT EXISTS market_sync_status (
    id SERIAL PRIMARY KEY,
    
    -- Link to watchlist
    exchange VARCHAR(50) NOT NULL,
    symbol VARCHAR(50) NOT NULL,
    resolution VARCHAR(10) NOT NULL,   -- '1m', '5m', '15m', '1h', '4h', '1d', '1w'
    
    -- Sync progress
    last_synced_time TIMESTAMPTZ,      -- Last candle timestamp synced
    total_candles INTEGER DEFAULT 0,
    last_sync_at TIMESTAMPTZ,
    
    -- Status
    status VARCHAR(20) DEFAULT 'pending',  -- pending, syncing, completed, failed, paused
    error_message TEXT,
    retry_count INTEGER DEFAULT 0,
    
    -- Metadata
    sync_duration_ms INTEGER,          -- Last sync duration
    
    UNIQUE(exchange, symbol, resolution)
);

CREATE INDEX IF NOT EXISTS idx_market_sync_exchange_symbol 
    ON market_sync_status(exchange, symbol);
CREATE INDEX IF NOT EXISTS idx_market_sync_status 
    ON market_sync_status(status);
CREATE INDEX IF NOT EXISTS idx_market_sync_resolution 
    ON market_sync_status(resolution);

-- Comments for documentation
COMMENT ON TABLE market_watchlist IS 'Universal watchlist for all exchanges and asset types';
COMMENT ON COLUMN market_watchlist.exchange IS 'Exchange identifier: Polygon, Binance, Bybit, Birdeye, etc';
COMMENT ON COLUMN market_watchlist.symbol IS 'Exchange-specific symbol format';
COMMENT ON COLUMN market_watchlist.metadata IS 'Exchange-specific metadata: {market_cap, sector, liquidity, fdv, chain, etc}';
COMMENT ON COLUMN market_watchlist.priority IS 'Sync priority: 1-100, higher values sync first';

COMMENT ON TABLE market_sync_status IS 'Tracks sync progress per exchange/symbol/resolution';

-- Example usage:
-- 
-- Polygon (US Stocks):
-- INSERT INTO market_watchlist (exchange, symbol, asset_type, name, enabled_1h, enabled_1d, metadata, priority)
-- VALUES ('Polygon', 'AAPL', 'stock', 'Apple Inc.', true, true, '{"market_cap": 3000000000000, "sector": "Technology"}', 100);
--
-- Binance (Crypto):
-- INSERT INTO market_watchlist (exchange, symbol, asset_type, base_currency, quote_currency, enabled_15m, enabled_1h)
-- VALUES ('Binance', 'BTCUSDT', 'crypto', 'BTC', 'USDT', true, true);
--
-- Birdeye (Solana tokens):
-- INSERT INTO market_watchlist (exchange, symbol, asset_type, name, metadata)
-- VALUES ('Birdeye', 'So11111111111111111111111111111111111111112', 'token', 'Wrapped SOL', '{"chain": "solana", "liquidity": 5000000}');
