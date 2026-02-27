-- P7-4A: Shadow promotion guard
-- Prevents promotion from shadow → paper unless the strategy has
-- accumulated >= 7 distinct trading days of shadow signals.

-- Function to check shadow period eligibility
CREATE OR REPLACE FUNCTION check_shadow_promotion_eligibility()
RETURNS TRIGGER AS $$
DECLARE
    trading_days INTEGER;
BEGIN
    -- Only fire when status changes to 'paper'
    IF NEW.status = 'paper' AND (OLD.status IS NULL OR OLD.status = 'shadow') THEN
        SELECT COUNT(DISTINCT DATE(signal_time))
        INTO trading_days
        FROM shadow_signals
        WHERE exchange = NEW.exchange
          AND symbol = NEW.symbol
          AND mode = NEW.mode
          AND signal_time >= NOW() - INTERVAL '30 days';

        IF trading_days < 7 THEN
            RAISE EXCEPTION
                'Cannot promote to paper: only % trading days in shadow (minimum 7 required) for %:% (%)',
                trading_days, NEW.exchange, NEW.symbol, NEW.mode;
        END IF;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Attach trigger to strategy_generations table
DROP TRIGGER IF EXISTS trg_shadow_promotion_guard ON strategy_generations;
CREATE TRIGGER trg_shadow_promotion_guard
    BEFORE UPDATE ON strategy_generations
    FOR EACH ROW
    WHEN (NEW.status = 'paper')
    EXECUTE FUNCTION check_shadow_promotion_eligibility();

-- Add status column if not exists (idempotent)
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'strategy_generations' AND column_name = 'status'
    ) THEN
        ALTER TABLE strategy_generations ADD COLUMN status TEXT DEFAULT 'active';
    END IF;
END $$;
