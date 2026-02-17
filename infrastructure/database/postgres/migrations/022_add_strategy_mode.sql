-- Migration 022: Add strategy mode (long_only / long_short) for dual evolution
-- Runs two parallel GA evolutions per symbol: one long-only, one long-short

-- 1. Add mode column with backward-compatible default
ALTER TABLE strategy_generations
    ADD COLUMN IF NOT EXISTS mode TEXT NOT NULL DEFAULT 'long_only';

-- 2. Migrate existing strategy_id format: polygon_AAPL_gen_500 -> polygon_AAPL_long_only_gen_500
UPDATE strategy_generations
SET strategy_id = REPLACE(strategy_id, '_gen_', '_long_only_gen_')
WHERE strategy_id NOT LIKE '%_long_only_gen_%'
  AND strategy_id NOT LIKE '%_long_short_gen_%';

-- 3. Update PK to include mode
ALTER TABLE strategy_generations DROP CONSTRAINT IF EXISTS strategy_generations_pkey;
ALTER TABLE strategy_generations ADD PRIMARY KEY (exchange, symbol, mode, generation);

-- 4. Index for mode-filtered queries
DROP INDEX IF EXISTS idx_strategy_gens_symbol;
CREATE INDEX IF NOT EXISTS idx_strategy_gens_symbol_mode
    ON strategy_generations (exchange, symbol, mode, generation DESC);

-- 5. Add mode to backtest_results
ALTER TABLE backtest_results
    ADD COLUMN IF NOT EXISTS mode TEXT NOT NULL DEFAULT 'long_only';

UPDATE backtest_results
SET strategy_id = REPLACE(strategy_id, '_gen_', '_long_only_gen_'), mode = 'long_only'
WHERE strategy_id IS NOT NULL
  AND strategy_id NOT LIKE '%_long_only_gen_%'
  AND strategy_id NOT LIKE '%_long_short_gen_%';

CREATE INDEX IF NOT EXISTS idx_backtest_results_mode
    ON backtest_results (token_address, mode, created_at DESC);
