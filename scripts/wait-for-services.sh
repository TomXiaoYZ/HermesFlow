#!/bin/bash
# 等待所有服务就绪

set -e

echo "等待服务就绪..."

# 等待PostgreSQL
until pg_isready -h postgres -p 5432 -U testuser; do
  echo "等待PostgreSQL..."
  sleep 2
done
echo "✅ PostgreSQL已就绪"

# 等待Redis
until redis-cli -h redis ping | grep -q PONG; do
  echo "等待Redis..."
  sleep 2
done
echo "✅ Redis已就绪"

# 等待ClickHouse
until curl -s http://clickhouse:8123/ping | grep -q Ok; do
  echo "等待ClickHouse..."
  sleep 2
done
echo "✅ ClickHouse已就绪"

# 等待Kafka
until echo "exit" | nc -z kafka 9092; do
  echo "等待Kafka..."
  sleep 2
done
echo "✅ Kafka已就绪"

echo "✅ 所有服务已就绪"

# 执行传入的命令
exec "$@"

