-- Migration 021: Per-symbol strategy evolution
-- Switch from universal cross-sectional ranking to per-symbol GA evolution

-- 1. Add symbol column to strategy_generations
ALTER TABLE strategy_generations ADD COLUMN IF NOT EXISTS symbol TEXT NOT NULL DEFAULT 'UNIVERSAL';

-- 2. Drop old primary key and recreate with symbol
ALTER TABLE strategy_generations DROP CONSTRAINT IF EXISTS strategy_generations_pkey;
ALTER TABLE strategy_generations ADD PRIMARY KEY (exchange, symbol, generation);

-- 3. Index for per-symbol queries
CREATE INDEX IF NOT EXISTS idx_strategy_gens_symbol
ON strategy_generations (exchange, symbol, generation DESC);

-- 4. Reduce watchlist to target 13 symbols
UPDATE market_watchlist SET is_active = false WHERE exchange = 'Polygon';

INSERT INTO market_watchlist (exchange, symbol, name, asset_type, is_active, priority,
    enabled_1h, enabled_4h, enabled_1d, sync_from_date)
VALUES
    ('Polygon', 'AAPL', 'Apple Inc', 'stock', true, 90, true, true, true, '2022-01-01'),
    ('Polygon', 'MSFT', 'Microsoft Corp', 'stock', true, 90, true, true, true, '2022-01-01'),
    ('Polygon', 'GOOGL', 'Alphabet Inc', 'stock', true, 90, true, true, true, '2022-01-01'),
    ('Polygon', 'AMZN', 'Amazon.com Inc', 'stock', true, 90, true, true, true, '2022-01-01'),
    ('Polygon', 'META', 'Meta Platforms Inc', 'stock', true, 90, true, true, true, '2022-01-01'),
    ('Polygon', 'NVDA', 'NVIDIA Corp', 'stock', true, 90, true, true, true, '2022-01-01'),
    ('Polygon', 'TSLA', 'Tesla Inc', 'stock', true, 90, true, true, true, '2022-01-01'),
    ('Polygon', 'SPY', 'S&P 500 ETF', 'stock', true, 80, true, true, true, '2022-01-01'),
    ('Polygon', 'QQQ', 'Nasdaq 100 ETF', 'stock', true, 80, true, true, true, '2022-01-01'),
    ('Polygon', 'DIA', 'Dow Jones ETF', 'stock', true, 80, true, true, true, '2022-01-01'),
    ('Polygon', 'IWM', 'Russell 2000 ETF', 'stock', true, 80, true, true, true, '2022-01-01'),
    ('Polygon', 'UVXY', 'ProShares Ultra VIX Short-Term Futures ETF', 'stock', true, 70, true, true, true, '2022-01-01'),
    ('Polygon', 'GLD', 'SPDR Gold Shares', 'stock', true, 70, true, true, true, '2022-01-01')
ON CONFLICT (exchange, symbol) DO UPDATE SET
    is_active = true, priority = EXCLUDED.priority, name = EXCLUDED.name;

-- 5. Deactivate non-target active_tokens
UPDATE active_tokens SET is_active = false
WHERE symbol NOT IN ('AAPL','MSFT','GOOGL','AMZN','META','NVDA','TSLA',
                     'SPY','QQQ','DIA','IWM','UVXY','GLD');
