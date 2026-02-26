-- P6-1D: Non-linear decay routing buffer.
-- Replaces binary 'active'/'replaced' with smooth weight-decay state machine:
--   active → decaying → retired
-- Strategies in 'decaying' state have their ensemble weight multiplied
-- by an exponential decay factor each rebalance period.

ALTER TABLE deployed_strategies
    ADD COLUMN IF NOT EXISTS decay_state TEXT NOT NULL DEFAULT 'none'
        CHECK (decay_state IN ('none', 'decaying', 'retired')),
    ADD COLUMN IF NOT EXISTS decay_started_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS decay_factor DOUBLE PRECISION NOT NULL DEFAULT 1.0;

-- Index for efficient lookup of decaying strategies
CREATE INDEX IF NOT EXISTS idx_deployed_strategies_decay
    ON deployed_strategies (exchange, decay_state)
    WHERE decay_state = 'decaying';

COMMENT ON COLUMN deployed_strategies.decay_state IS
    'Decay lifecycle: none (normal active), decaying (weight reducing), retired (effectively zero weight)';
COMMENT ON COLUMN deployed_strategies.decay_started_at IS
    'Timestamp when strategy entered decaying state';
COMMENT ON COLUMN deployed_strategies.decay_factor IS
    'Current multiplicative decay factor (1.0 = no decay, approaches 0.0 over time)';
