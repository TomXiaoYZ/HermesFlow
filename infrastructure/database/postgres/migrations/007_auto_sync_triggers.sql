-- Auto-sync trigger for market_watchlist
-- When a new ticker is inserted, automatically queue it for sync

-- 1. Create pending sync entries when watchlist is inserted
CREATE OR REPLACE FUNCTION create_sync_tasks_on_watchlist_insert()
RETURNS TRIGGER AS $$
BEGIN
    -- Create sync tasks for all enabled resolutions
    IF NEW.enabled_1m THEN
        INSERT INTO market_sync_status (exchange, symbol, resolution, status)
        VALUES (NEW.exchange, NEW.symbol, '1m', 'pending')
        ON CONFLICT (exchange, symbol, resolution) DO NOTHING;
    END IF;
    
    IF NEW.enabled_5m THEN
        INSERT INTO market_sync_status (exchange, symbol, resolution, status)
        VALUES (NEW.exchange, NEW.symbol, '5m', 'pending')
        ON CONFLICT (exchange, symbol, resolution) DO NOTHING;
    END IF;
    
    IF NEW.enabled_15m THEN
        INSERT INTO market_sync_status (exchange, symbol, resolution, status)
        VALUES (NEW.exchange, NEW.symbol, '15m', 'pending')
        ON CONFLICT (exchange, symbol, resolution) DO NOTHING;
    END IF;
    
    IF NEW.enabled_30m THEN
        INSERT INTO market_sync_status (exchange, symbol, resolution, status)
        VALUES (NEW.exchange, NEW.symbol, '30m', 'pending')
        ON CONFLICT (exchange, symbol, resolution) DO NOTHING;
    END IF;
    
    IF NEW.enabled_1h THEN
        INSERT INTO market_sync_status (exchange, symbol, resolution, status)
        VALUES (NEW.exchange, NEW.symbol, '1h', 'pending')
        ON CONFLICT (exchange, symbol, resolution) DO NOTHING;
    END IF;
    
    IF NEW.enabled_4h THEN
        INSERT INTO market_sync_status (exchange, symbol, resolution, status)
        VALUES (NEW.exchange, NEW.symbol, '4h', 'pending')
        ON CONFLICT (exchange, symbol, resolution) DO NOTHING;
    END IF;
    
    IF NEW.enabled_1d THEN
        INSERT INTO market_sync_status (exchange, symbol, resolution, status)
        VALUES (NEW.exchange, NEW.symbol, '1d', 'pending')
        ON CONFLICT (exchange, symbol, resolution) DO NOTHING;
    END IF;
    
    IF NEW.enabled_1w THEN
        INSERT INTO market_sync_status (exchange, symbol, resolution, status)
        VALUES (NEW.exchange, NEW.symbol, '1w', 'pending')
        ON CONFLICT (exchange, symbol, resolution) DO NOTHING;
    END IF;
    
    -- Send notification for background worker
    PERFORM pg_notify('watchlist_insert', json_build_object(
        'exchange', NEW.exchange,
        'symbol', NEW.symbol,
        'priority', NEW.priority
    )::text);
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Attach trigger to market_watchlist
DROP TRIGGER IF EXISTS trigger_auto_sync_on_insert ON market_watchlist;
CREATE TRIGGER trigger_auto_sync_on_insert
    AFTER INSERT ON market_watchlist
    FOR EACH ROW
    WHEN (NEW.is_active = true)
    EXECUTE FUNCTION create_sync_tasks_on_watchlist_insert();

-- 2. Handle updates to enabled_* fields
CREATE OR REPLACE FUNCTION update_sync_tasks_on_watchlist_change()
RETURNS TRIGGER AS $$
BEGIN
    -- When enabling a new resolution, add sync task
    IF NEW.enabled_1m AND NOT OLD.enabled_1m THEN
        INSERT INTO market_sync_status (exchange, symbol, resolution, status)
        VALUES (NEW.exchange, NEW.symbol, '1m', 'pending')
        ON CONFLICT (exchange, symbol, resolution) DO UPDATE SET status = 'pending';
    END IF;
    
    IF NEW.enabled_15m AND NOT OLD.enabled_15m THEN
        INSERT INTO market_sync_status (exchange, symbol, resolution, status)
        VALUES (NEW.exchange, NEW.symbol, '15m', 'pending')
        ON CONFLICT (exchange, symbol, resolution) DO UPDATE SET status = 'pending';
    END IF;
    
    IF NEW.enabled_1h AND NOT OLD.enabled_1h THEN
        INSERT INTO market_sync_status (exchange, symbol, resolution, status)
        VALUES (NEW.exchange, NEW.symbol, '1h', 'pending')
        ON CONFLICT (exchange, symbol, resolution) DO UPDATE SET status = 'pending';
    END IF;
    
    IF NEW.enabled_4h AND NOT OLD.enabled_4h THEN
        INSERT INTO market_sync_status (exchange, symbol, resolution, status)
        VALUES (NEW.exchange, NEW.symbol, '4h', 'pending')
        ON CONFLICT (exchange, symbol, resolution) DO UPDATE SET status = 'pending';
    END IF;
    
    IF NEW.enabled_1d AND NOT OLD.enabled_1d THEN
        INSERT INTO market_sync_status (exchange, symbol, resolution, status)
        VALUES (NEW.exchange, NEW.symbol, '1d', 'pending')
        ON CONFLICT (exchange, symbol, resolution) DO UPDATE SET status = 'pending';
    END IF;
    
    IF NEW.enabled_1w AND NOT OLD.enabled_1w THEN
        INSERT INTO market_sync_status (exchange, symbol, resolution, status)
        VALUES (NEW.exchange, NEW.symbol, '1w', 'pending')
        ON CONFLICT (exchange, symbol, resolution) DO UPDATE SET status = 'pending';
    END IF;
    
    -- Notify worker of changes
    IF NEW.enabled_1m != OLD.enabled_1m OR 
       NEW.enabled_15m != OLD.enabled_15m OR
       NEW.enabled_1h != OLD.enabled_1h OR
       NEW.enabled_4h != OLD.enabled_4h OR
       NEW.enabled_1d != OLD.enabled_1d OR
       NEW.enabled_1w != OLD.enabled_1w THEN
        PERFORM pg_notify('watchlist_update', json_build_object(
            'exchange', NEW.exchange,
            'symbol', NEW.symbol
        )::text);
    END IF;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trigger_auto_sync_on_update ON market_watchlist;
CREATE TRIGGER trigger_auto_sync_on_update
    AFTER UPDATE ON market_watchlist
    FOR EACH ROW
    WHEN (NEW.is_active = true)
    EXECUTE FUNCTION update_sync_tasks_on_watchlist_change();

-- Helper function to get pending sync tasks
CREATE OR REPLACE FUNCTION get_pending_sync_tasks(limit_count INTEGER DEFAULT 100)
RETURNS TABLE (
    exchange VARCHAR,
    symbol VARCHAR,
    resolution VARCHAR,
    sync_from_date DATE,
    priority INTEGER
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        s.exchange,
        s.symbol,
        s.resolution,
        COALESCE(w.sync_from_date, '2023-01-01'::DATE) as sync_from_date,
        COALESCE(w.priority, 50) as priority
    FROM market_sync_status s
    INNER JOIN market_watchlist w ON s.exchange = w.exchange AND s.symbol = w.symbol
    WHERE s.status = 'pending'
      AND w.is_active = true
    ORDER BY w.priority DESC, s.exchange, s.symbol, s.resolution
    LIMIT limit_count;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION create_sync_tasks_on_watchlist_insert() IS 'Auto-creates sync tasks when new ticker added to watchlist';
COMMENT ON FUNCTION get_pending_sync_tasks(INTEGER) IS 'Returns pending sync tasks ordered by priority';
