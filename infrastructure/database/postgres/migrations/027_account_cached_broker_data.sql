-- Migration 027: Add cached broker data columns to trading_accounts
-- The execution-engine writes real IBKR account values (net_liq, cash, buying_power)
-- every 30s, and the gateway reads them instead of computing from trade_executions.

ALTER TABLE trading_accounts ADD COLUMN IF NOT EXISTS cached_net_liq DECIMAL DEFAULT 0;
ALTER TABLE trading_accounts ADD COLUMN IF NOT EXISTS cached_cash DECIMAL DEFAULT 0;
ALTER TABLE trading_accounts ADD COLUMN IF NOT EXISTS cached_buying_power DECIMAL DEFAULT 0;
ALTER TABLE trading_accounts ADD COLUMN IF NOT EXISTS cache_updated_at TIMESTAMPTZ;
