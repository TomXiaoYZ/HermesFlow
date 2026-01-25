-- Fix numeric overflow for active_tokens table
-- Previous schema used NUMERIC(18,8) which caps at ~10 Billion.
-- Crypto assets can exceed this easily (meme coins with trillion supply * price).
-- DECIMAL(40, 8) allows for 32 integer digits (10^32), sufficient for any asset.

ALTER TABLE active_tokens
    ALTER COLUMN liquidity_usd TYPE DECIMAL(40, 8),
    ALTER COLUMN fdv TYPE DECIMAL(40, 8),
    ALTER COLUMN market_cap TYPE DECIMAL(40, 8),
    ALTER COLUMN volume_24h TYPE DECIMAL(40, 8);
