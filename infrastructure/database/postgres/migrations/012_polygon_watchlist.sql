-- Polygon.io Watchlist Configuration Table
-- Manages which US stocks to track and what timeframes to sync

CREATE TABLE IF NOT EXISTS polygon_watchlist (
    ticker VARCHAR(10) PRIMARY KEY,
    name TEXT,
    market_cap DECIMAL(20, 2),
    sector VARCHAR(100),
    
    -- Timeframe flags (which resolutions to sync)
    enabled_1m BOOLEAN DEFAULT false,
    enabled_15m BOOLEAN DEFAULT false,
    enabled_1h BOOLEAN DEFAULT true,
    enabled_4h BOOLEAN DEFAULT true,
    enabled_1d BOOLEAN DEFAULT true,
    enabled_1w BOOLEAN DEFAULT false,
    
    -- Control
    is_active BOOLEAN DEFAULT true,
    priority INTEGER DEFAULT 50,
    
    -- Sync metadata
    last_synced_at TIMESTAMPTZ,
    sync_from_date DATE DEFAULT '2023-01-01',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    notes TEXT
);

-- Performance indexes
CREATE INDEX IF NOT EXISTS idx_polygon_watchlist_active 
    ON polygon_watchlist(is_active) WHERE is_active = true;
CREATE INDEX IF NOT EXISTS idx_polygon_watchlist_market_cap 
    ON polygon_watchlist(market_cap DESC);
CREATE INDEX IF NOT EXISTS idx_polygon_watchlist_priority 
    ON polygon_watchlist(priority DESC);

-- Sync status tracking
CREATE TABLE IF NOT EXISTS polygon_sync_status (
    id SERIAL PRIMARY KEY,
    ticker VARCHAR(10) NOT NULL,
    resolution VARCHAR(10) NOT NULL,
    
    last_synced_time TIMESTAMPTZ,
    total_candles INTEGER DEFAULT 0,
    last_sync_at TIMESTAMPTZ,
    
    status VARCHAR(20) DEFAULT 'pending',
    error_message TEXT,
    
    UNIQUE(ticker, resolution)
);

CREATE INDEX IF NOT EXISTS idx_polygon_sync_ticker_resolution 
    ON polygon_sync_status(ticker, resolution);
CREATE INDEX IF NOT EXISTS idx_polygon_sync_status 
    ON polygon_sync_status(status);

-- Comments
COMMENT ON TABLE polygon_watchlist IS 'Configuration for Polygon.io US stock data collection';
COMMENT ON COLUMN polygon_watchlist.priority IS 'Sync priority: 1-100, higher = sync first';
COMMENT ON COLUMN polygon_watchlist.sync_from_date IS 'Start date for historical backfill';
