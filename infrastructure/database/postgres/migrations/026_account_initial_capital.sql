-- Migration 026: Add initial_capital to trading_accounts
-- Enables cash balance computation from trade execution history:
-- cash = initial_capital - Σ(buy_cost) + Σ(sell_proceeds) - Σ(commissions)

ALTER TABLE trading_accounts ADD COLUMN IF NOT EXISTS initial_capital DECIMAL NOT NULL DEFAULT 100000;

UPDATE trading_accounts SET initial_capital = 100000 WHERE account_id = 'ibkr_long_only';
UPDATE trading_accounts SET initial_capital = 100000 WHERE account_id = 'ibkr_long_short';
