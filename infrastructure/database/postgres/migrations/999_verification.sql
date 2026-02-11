-- Database Verification Queries
-- Run these in AWS RDS Query Editor or via ECS Exec

-- 1. List all tables
SELECT table_name 
FROM information_schema.tables 
WHERE table_schema = 'public'
ORDER BY table_name;

-- 2. Check tweets table structure
SELECT column_name, data_type, is_nullable
FROM information_schema.columns 
WHERE table_name = 'tweets'
ORDER BY ordinal_position;

-- 3. Count tweets
SELECT COUNT(*) as total_tweets FROM tweets;

-- 4. Recent tweets
SELECT 
    username, 
    created_at, 
    LEFT(text, 100) as text_preview,
    retweet_count,
    favorite_count
FROM tweets 
ORDER BY created_at DESC 
LIMIT 10;

-- 5. Tweets by user
SELECT 
    username,
    COUNT(*) as tweet_count,
    MIN(created_at) as first_tweet,
    MAX(created_at) as last_tweet
FROM tweets 
GROUP BY username
ORDER BY tweet_count DESC;

-- 6. Twitter collection runs
SELECT 
    id,
    target,
    collected_at,
    scraped_count,
    upserted_count,
    LEFT(COALESCE(error, 'SUCCESS'), 100) as error_status
FROM twitter_collection_runs 
ORDER BY collected_at DESC 
LIMIT 20;

-- 7. Collection run statistics
SELECT 
    DATE_TRUNC('hour', collected_at) as hour,
    COUNT(*) as run_count,
    SUM(scraped_count) as total_scraped,
    SUM(upserted_count) as total_upserted,
    COUNT(CASE WHEN error IS NOT NULL THEN 1 END) as error_count
FROM twitter_collection_runs 
GROUP BY DATE_TRUNC('hour', collected_at)
ORDER BY hour DESC
LIMIT 24;

-- 8. Check prediction markets
SELECT COUNT(*) as total_markets FROM prediction_markets;

-- 9. Recent prediction markets
SELECT 
    id,
    source,
    title,
    active,
    created_at
FROM prediction_markets 
ORDER BY created_at DESC 
LIMIT 10;

-- 10. Market outcomes statistics
SELECT 
    COUNT(*) as total_outcomes,
    COUNT(DISTINCT market_id) as unique_markets,
    MIN(timestamp) as first_recorded,
    MAX(timestamp) as last_recorded
FROM market_outcomes;
