-- Fix initial_capital to match IBKR paper account starting balance ($1M)
UPDATE trading_accounts SET initial_capital = 1000000
WHERE account_id IN ('ibkr_long_only', 'ibkr_long_short');

-- Daily net-liquidation snapshots for day-over-day PnL comparison
CREATE TABLE IF NOT EXISTS account_daily_snapshots (
    account_id  VARCHAR(50)   NOT NULL,
    snapshot_date DATE        NOT NULL,
    net_liquidation DECIMAL(24,8) NOT NULL,
    cash_balance    DECIMAL(24,8),
    buying_power    DECIMAL(24,8),
    created_at  TIMESTAMPTZ   DEFAULT NOW(),
    PRIMARY KEY (account_id, snapshot_date)
);

-- Seed today's snapshot from current cached values so daily PnL works immediately
INSERT INTO account_daily_snapshots (account_id, snapshot_date, net_liquidation, cash_balance, buying_power)
SELECT account_id, CURRENT_DATE, COALESCE(cached_net_liq, initial_capital), cached_cash, cached_buying_power
FROM trading_accounts
WHERE cached_net_liq IS NOT NULL
ON CONFLICT (account_id, snapshot_date) DO NOTHING;
