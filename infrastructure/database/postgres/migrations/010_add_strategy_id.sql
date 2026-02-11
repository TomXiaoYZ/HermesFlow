-- Add strategy_id to strategy_generations
ALTER TABLE strategy_generations ADD COLUMN IF NOT EXISTS strategy_id TEXT;

-- Index for lookup
CREATE INDEX IF NOT EXISTS idx_strategy_generations_id ON strategy_generations(strategy_id);
