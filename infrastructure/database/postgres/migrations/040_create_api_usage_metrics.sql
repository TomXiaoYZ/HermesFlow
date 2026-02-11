-- Create API Usage Metrics table
CREATE TABLE IF NOT EXISTS api_usage_metrics (
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    provider TEXT NOT NULL,
    endpoint TEXT,
    request_count BIGINT NOT NULL,
    metadata JSONB
);

-- Convert to hypertable for TimescaleDB efficiency
SELECT create_hypertable('api_usage_metrics', 'timestamp', if_not_exists => TRUE);

-- Create index for query performance
CREATE INDEX IF NOT EXISTS idx_api_usage_metrics_provider_timestamp ON api_usage_metrics (provider, timestamp DESC);
