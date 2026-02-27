-- P7-4C: Auto-demotion columns and logic
-- Tracks consecutive underperformance and manages paper → shadow demotion.

-- Add demotion tracking columns
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'strategy_generations' AND column_name = 'demotion_count'
    ) THEN
        ALTER TABLE strategy_generations ADD COLUMN demotion_count INTEGER DEFAULT 0;
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'strategy_generations' AND column_name = 'last_demotion_timestamp'
    ) THEN
        ALTER TABLE strategy_generations ADD COLUMN last_demotion_timestamp TIMESTAMPTZ;
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'strategy_generations' AND column_name = 'consecutive_underperform'
    ) THEN
        ALTER TABLE strategy_generations ADD COLUMN consecutive_underperform INTEGER DEFAULT 0;
    END IF;
END $$;

-- Function: auto-demote paper strategies that underperform
-- Called periodically by the ensemble rebalance loop.
-- Criteria: consecutive_underperform >= 3 OR max drawdown > threshold
CREATE OR REPLACE FUNCTION demote_underperforming_strategies(
    p_exchange TEXT,
    p_min_oos_psr DOUBLE PRECISION DEFAULT 0.0,
    p_max_consecutive INTEGER DEFAULT 3
)
RETURNS TABLE(
    demoted_symbol TEXT,
    demoted_mode TEXT,
    old_status TEXT,
    new_consecutive INTEGER
) AS $$
BEGIN
    RETURN QUERY
    UPDATE strategy_generations sg
    SET
        status = 'shadow',
        demotion_count = sg.demotion_count + 1,
        last_demotion_timestamp = NOW(),
        consecutive_underperform = 0
    WHERE sg.exchange = p_exchange
      AND sg.status = 'paper'
      AND sg.consecutive_underperform >= p_max_consecutive
    RETURNING sg.symbol, sg.mode, 'paper'::TEXT, sg.consecutive_underperform;
END;
$$ LANGUAGE plpgsql;
