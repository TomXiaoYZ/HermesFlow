-- 031: Replace hardcoded IBKR account IDs with placeholder values.
-- Actual account IDs are injected via environment variables at runtime
-- and reconciled by the execution-engine on startup.

UPDATE trading_accounts SET broker_account = 'PAPER_ACCOUNT_LO'  WHERE account_id = 'ibkr_long_only';
UPDATE trading_accounts SET broker_account = 'PAPER_ACCOUNT_LS'  WHERE account_id = 'ibkr_long_short';
