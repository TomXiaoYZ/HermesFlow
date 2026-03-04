-- Add equity-proportional order limit to trading_accounts.
-- When cached_net_liq > 0, effective max = cached_net_liq * max_order_pct.
-- Falls back to static max_order_value when equity is unavailable.
ALTER TABLE trading_accounts
  ADD COLUMN IF NOT EXISTS max_order_pct NUMERIC NOT NULL DEFAULT 0.10;
