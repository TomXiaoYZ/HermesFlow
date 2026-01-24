-- Data Quality Validation Script

-- 1. Token Distribution
SELECT 
    'Total Active Tokens' as metric,
    COUNT(*) as value
FROM active_tokens 
WHERE is_active = true;

-- 2. Data Freshness
SELECT 
    'Tokens Updated Last 1h' as metric,
    COUNT(*) as value
FROM active_tokens 
WHERE is_active = true 
AND last_updated > NOW() - INTERVAL '1 hour';

-- 3. Liquidity Distribution
SELECT 
    symbol,
    ROUND(liquidity_usd::numeric, 0) as liquidity,
    ROUND(fdv::numeric, 0) as fdv
FROM active_tokens 
WHERE is_active = true 
ORDER BY liquidity_usd DESC 
LIMIT 10;

-- 4. Helius Transaction Data (Placeholder validation)
-- Currently we just ensure connection is up, but tracking table might not be populated yet
-- as we only subscribe to slots. 
-- Future: SELECT COUNT(*) FROM whale_transactions...
