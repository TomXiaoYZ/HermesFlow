-- Migration 024: Add mode column to trade_orders for clean filtering
-- Enables filtering trades by strategy mode (long_only / long_short)
-- without parsing account_id

-- 1. Add mode column
ALTER TABLE trade_orders ADD COLUMN IF NOT EXISTS mode TEXT;

-- 2. Backfill from account_id (ibkr_long_only → long_only, ibkr_long_short → long_short)
UPDATE trade_orders SET mode = REPLACE(account_id, 'ibkr_', '')
WHERE account_id LIKE 'ibkr_%' AND mode IS NULL;

-- 3. Index for mode-filtered queries
CREATE INDEX IF NOT EXISTS idx_orders_mode ON trade_orders(mode, created_at DESC);

-- Register Migration
INSERT INTO schema_migrations (version, description)
VALUES ('024_trade_orders_mode', 'Add mode column to trade_orders for strategy mode filtering')
ON CONFLICT (version) DO NOTHING;
