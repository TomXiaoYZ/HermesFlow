-- Initialize database schema for data-engine
-- Run this manually if you want to set up the database before first run
-- Otherwise, the application will create these tables automatically

-- Create tweets table for Twitter/X social media data
CREATE TABLE IF NOT EXISTS tweets (
    id BIGINT PRIMARY KEY,
    username TEXT NOT NULL,
    text TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    received_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    user_id BIGINT,
    followers_count INTEGER,
    verified BOOLEAN DEFAULT false,
    retweet_count INTEGER DEFAULT 0,
    favorite_count INTEGER DEFAULT 0,
    reply_count INTEGER DEFAULT 0,
    quote_count INTEGER DEFAULT 0,
    is_retweet BOOLEAN DEFAULT false,
    is_reply BOOLEAN DEFAULT false,
    hashtags TEXT[],
    media_urls TEXT[],
    raw_data JSONB
);

-- Create indices for tweets
CREATE INDEX IF NOT EXISTS idx_tweets_username ON tweets(username);
CREATE INDEX IF NOT EXISTS idx_tweets_created_at ON tweets(created_at);
CREATE INDEX IF NOT EXISTS idx_tweets_followers ON tweets(followers_count) WHERE followers_count IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_tweets_engagement ON tweets(retweet_count + favorite_count + reply_count);
CREATE INDEX IF NOT EXISTS idx_tweets_hashtags ON tweets USING GIN(hashtags);

-- Create prediction_markets table for Polymarket and other prediction markets
CREATE TABLE IF NOT EXISTS prediction_markets (
    id TEXT PRIMARY KEY,
    source TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    category TEXT,
    end_date TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    active BOOLEAN DEFAULT true,
    metadata JSONB
);

-- Create indices for prediction_markets
CREATE INDEX IF NOT EXISTS idx_markets_source ON prediction_markets(source);
CREATE INDEX IF NOT EXISTS idx_markets_active ON prediction_markets(active);
CREATE INDEX IF NOT EXISTS idx_markets_end_date ON prediction_markets(end_date) WHERE end_date IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_markets_category ON prediction_markets(category) WHERE category IS NOT NULL;

-- Create market_outcomes table for tracking outcome prices over time
CREATE TABLE IF NOT EXISTS market_outcomes (
    id SERIAL PRIMARY KEY,
    market_id TEXT NOT NULL REFERENCES prediction_markets(id) ON DELETE CASCADE,
    outcome TEXT NOT NULL,
    price DECIMAL(18, 8) NOT NULL,
    volume_24h DECIMAL(18, 8),
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(market_id, outcome, timestamp)
);

-- Create indices for market_outcomes
CREATE INDEX IF NOT EXISTS idx_outcomes_market_id ON market_outcomes(market_id);
CREATE INDEX IF NOT EXISTS idx_outcomes_timestamp ON market_outcomes(timestamp);
CREATE INDEX IF NOT EXISTS idx_outcomes_market_outcome ON market_outcomes(market_id, outcome);

-- Create a view for latest market prices
CREATE OR REPLACE VIEW latest_market_prices AS
SELECT DISTINCT ON (market_id, outcome)
    market_id,
    outcome,
    price,
    volume_24h,
    timestamp
FROM market_outcomes
ORDER BY market_id, outcome, timestamp DESC;

-- Create a view for high-engagement tweets
CREATE OR REPLACE VIEW high_engagement_tweets AS
SELECT 
    id,
    username,
    text,
    created_at,
    retweet_count,
    favorite_count,
    reply_count,
    quote_count,
    (retweet_count + favorite_count + reply_count + quote_count) as total_engagement,
    hashtags
FROM tweets
WHERE (retweet_count + favorite_count + reply_count + quote_count) > 100
ORDER BY (retweet_count + favorite_count + reply_count + quote_count) DESC;

-- Create a function to update the updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create trigger for prediction_markets
DROP TRIGGER IF EXISTS update_prediction_markets_updated_at ON prediction_markets;
CREATE TRIGGER update_prediction_markets_updated_at
    BEFORE UPDATE ON prediction_markets
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Grant permissions (adjust as needed for your setup)
-- GRANT SELECT, INSERT, UPDATE ON tweets TO data_engine_user;
-- GRANT SELECT, INSERT, UPDATE ON prediction_markets TO data_engine_user;
-- GRANT SELECT, INSERT ON market_outcomes TO data_engine_user;
-- GRANT SELECT ON latest_market_prices TO data_engine_user;
-- GRANT SELECT ON high_engagement_tweets TO data_engine_user;

-- Display table statistics
SELECT 
    'tweets' as table_name,
    COUNT(*) as row_count,
    pg_size_pretty(pg_total_relation_size('tweets')) as total_size
FROM tweets
UNION ALL
SELECT 
    'prediction_markets',
    COUNT(*),
    pg_size_pretty(pg_total_relation_size('prediction_markets'))
FROM prediction_markets
UNION ALL
SELECT 
    'market_outcomes',
    COUNT(*),
    pg_size_pretty(pg_total_relation_size('market_outcomes'))
FROM market_outcomes;

