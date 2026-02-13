-- Migration 005: Migrate unified_ticks to ReplacingMergeTree for deduplication
-- ReplacingMergeTree(ingested_at) keeps the latest row per ORDER BY key,
-- deduplicating replayed/retried inserts on merge.

-- Step 1: Create new table with ReplacingMergeTree engine
CREATE TABLE IF NOT EXISTS unified_ticks_new (
    source LowCardinality(String),
    exchange LowCardinality(String),
    symbol String,
    asset_type LowCardinality(String),
    data_type LowCardinality(String),
    price Decimal(32, 8),
    quantity Decimal(32, 8),
    timestamp DateTime64(3),
    received_at DateTime64(3),
    bid Nullable(Decimal(32, 8)),
    ask Nullable(Decimal(32, 8)),
    high_24h Nullable(Decimal(32, 8)),
    low_24h Nullable(Decimal(32, 8)),
    volume_24h Nullable(Decimal(32, 8)),
    open_interest Nullable(Decimal(32, 8)),
    funding_rate Nullable(Decimal(32, 8)),
    sequence_id Nullable(UInt64),
    raw_data String,
    ingested_at DateTime64(3) DEFAULT now64(3)
) ENGINE = ReplacingMergeTree(ingested_at)
PARTITION BY toYYYYMMDD(timestamp)
ORDER BY (source, symbol, timestamp, coalesce(sequence_id, 0))
SETTINGS index_granularity = 8192;

-- Step 2: Copy existing data
INSERT INTO unified_ticks_new SELECT * FROM unified_ticks;

-- Step 3: Swap tables
RENAME TABLE unified_ticks TO unified_ticks_old, unified_ticks_new TO unified_ticks;

-- Step 4: Recreate indexes on new table
CREATE INDEX IF NOT EXISTS idx_symbol_timestamp ON unified_ticks (symbol, timestamp) TYPE minmax GRANULARITY 4;
CREATE INDEX IF NOT EXISTS idx_data_type ON unified_ticks (data_type) TYPE set(10) GRANULARITY 4;

-- Step 5: Drop old table (uncomment after verifying data integrity)
-- DROP TABLE IF EXISTS unified_ticks_old;
