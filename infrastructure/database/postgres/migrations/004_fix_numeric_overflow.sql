-- Fix numeric overflow for large crypto values (FDV, etc.)
-- Previous schema used (18,8) or similar which caps at 10 Billion.
-- Crypto assets can exceed this easily (100 Trillion supply * price).
-- DECIMAL(40, 8) allows for 32 integer digits (10^32), plenty for any asset.

ALTER TABLE mkt_equity_candles 
    ALTER COLUMN fdv TYPE DECIMAL(40, 8),
    ALTER COLUMN liquidity TYPE DECIMAL(40, 8),
    ALTER COLUMN volume TYPE DECIMAL(40, 8),
    ALTER COLUMN amount TYPE DECIMAL(40, 8);
