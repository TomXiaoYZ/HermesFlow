-- Migration 019: Add exchange column to strategy_generations for multi-exchange support
-- Allows running separate crypto and stock strategy generators in parallel

-- 1. Add exchange column with default for existing data
ALTER TABLE strategy_generations ADD COLUMN IF NOT EXISTS exchange TEXT NOT NULL DEFAULT 'Birdeye';

-- 2. Drop old primary key (generation only)
ALTER TABLE strategy_generations DROP CONSTRAINT strategy_generations_pkey;

-- 3. Create new composite primary key (exchange + generation)
ALTER TABLE strategy_generations ADD PRIMARY KEY (exchange, generation);

-- 4. Index for fast resume query (latest generation per exchange)
CREATE INDEX IF NOT EXISTS idx_strategy_gen_exchange_gen
    ON strategy_generations (exchange, generation DESC);
