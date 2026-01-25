-- System Logs Table for Vector
CREATE TABLE IF NOT EXISTS system_logs (
    timestamp DateTime64(3),
    container_name String,
    image String,
    level String,
    message String,
    raw_json String
) ENGINE = MergeTree()
ORDER BY (timestamp, container_name)
TTL toDateTime(timestamp) + INTERVAL 15 DAY;
