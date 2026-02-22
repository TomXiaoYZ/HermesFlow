-- Backtest results retention: keep only latest per (token_address, mode).
-- Prevents unbounded growth from API handler or crash-recovery scenarios.

-- Index to accelerate the retention DELETE and per-symbol lookups
CREATE INDEX IF NOT EXISTS idx_backtest_results_token_mode
    ON backtest_results (token_address, mode, created_at DESC);

-- Scheduled cleanup function: for each (token_address, mode) group,
-- delete all rows except the most recent one.
CREATE OR REPLACE FUNCTION cleanup_stale_backtests() RETURNS void AS $$
BEGIN
    DELETE FROM backtest_results br
    USING (
        SELECT token_address, mode, MAX(created_at) AS keep_ts
        FROM backtest_results
        GROUP BY token_address, mode
    ) latest
    WHERE br.token_address = latest.token_address
      AND br.mode = latest.mode
      AND br.created_at < latest.keep_ts;
END;
$$ LANGUAGE plpgsql;

-- Strategy generations: safety net cleanup for rows older than 30 days.
-- The app already keeps a 1000-generation rolling window, but this catches
-- orphans from crashed processes that never ran their cleanup loop.
CREATE OR REPLACE FUNCTION cleanup_old_generations() RETURNS void AS $$
BEGIN
    DELETE FROM strategy_generations
    WHERE timestamp < NOW() - INTERVAL '30 days';
END;
$$ LANGUAGE plpgsql;
