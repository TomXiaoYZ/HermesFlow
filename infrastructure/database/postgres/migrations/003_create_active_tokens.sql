-- Migration: Create active_tokens table for dynamic token tracking
-- Description: Stores currently active trading tokens discovered from Birdeye trending API
-- Author: HermesFlow Team
-- Date: 2026-01-23

CREATE TABLE IF NOT EXISTS active_tokens (
    address TEXT PRIMARY KEY,
    symbol TEXT NOT NULL,
    name TEXT,
    decimals INTEGER NOT NULL DEFAULT 9,
    chain TEXT NOT NULL DEFAULT 'solana',
    liquidity_usd DECIMAL(18,2),
    fdv DECIMAL(18,2),
    market_cap DECIMAL(18,2),
    volume_24h DECIMAL(18,2),
    price_change_24h DECIMAL(8,4),
    first_discovered TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_updated TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_active BOOLEAN NOT NULL DEFAULT true,
    metadata JSONB
);

-- Index for querying active tokens efficiently
CREATE INDEX IF NOT EXISTS idx_active_tokens_active 
ON active_tokens(is_active) 
WHERE is_active = true;

-- Index for sorting by update time
CREATE INDEX IF NOT EXISTS idx_active_tokens_updated 
ON active_tokens(last_updated DESC);

-- Index for filtering by liquidity
CREATE INDEX IF NOT EXISTS idx_active_tokens_liquidity 
ON active_tokens(liquidity_usd DESC) 
WHERE is_active = true;

-- Function to automatically update last_updated timestamp
CREATE OR REPLACE FUNCTION update_active_tokens_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.last_updated = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to call the function on UPDATE
DROP TRIGGER IF EXISTS trigger_update_active_tokens_timestamp ON active_tokens;
CREATE TRIGGER trigger_update_active_tokens_timestamp
BEFORE UPDATE ON active_tokens
FOR EACH ROW
EXECUTE FUNCTION update_active_tokens_timestamp();
