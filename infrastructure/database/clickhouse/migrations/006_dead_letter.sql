-- Dead letter persistence table
-- Stores records that could not be written to their target storage
-- after all retry attempts have been exhausted.
CREATE TABLE IF NOT EXISTS dead_letters (
    id UUID DEFAULT generateUUIDv4(),
    source LowCardinality(String),
    exchange String,
    symbol String,
    price Decimal(32, 8),
    quantity Decimal(32, 8),
    timestamp DateTime64(3),
    storage_target LowCardinality(String),
    error String,
    raw_data String,
    created_at DateTime DEFAULT now(),
    replayed_at Nullable(DateTime),
    replay_status LowCardinality(Nullable(String))
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(created_at)
ORDER BY (source, created_at)
TTL created_at + INTERVAL 90 DAY;
