-- 025: Trading accounts table for per-account configuration and risk limits
-- Replaces global env-var risk parameters with per-account DB config

CREATE TABLE IF NOT EXISTS trading_accounts (
    account_id      TEXT PRIMARY KEY,
    label           TEXT NOT NULL,
    broker          TEXT NOT NULL DEFAULT 'IBKR',
    broker_account  TEXT,
    mode            TEXT NOT NULL,
    is_enabled      BOOLEAN NOT NULL DEFAULT true,
    max_order_value DECIMAL NOT NULL DEFAULT 2000,
    max_positions   INTEGER NOT NULL DEFAULT 5,
    max_daily_loss  DECIMAL NOT NULL DEFAULT 500,
    created_at      TIMESTAMPTZ DEFAULT NOW(),
    updated_at      TIMESTAMPTZ DEFAULT NOW()
);

INSERT INTO trading_accounts (account_id, label, broker, broker_account, mode)
VALUES
    ('ibkr_long_only',  'Long Only',  'IBKR', 'DU7413927',  'long_only'),
    ('ibkr_long_short', 'Long Short', 'IBKR', 'DUP964037', 'long_short')
ON CONFLICT (account_id) DO NOTHING;
