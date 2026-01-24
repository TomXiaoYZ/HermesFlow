#!/bin/bash
# Start development dependencies with Docker

set -e

echo "🐳 Starting development dependencies..."
echo "========================================"

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo "❌ Docker is not running. Please start Docker Desktop."
    exit 1
fi

# Start Redis
echo ""
echo "📦 Starting Redis..."
docker run -d \
    --name hermesflow-redis \
    -p 6379:6379 \
    --rm \
    redis:7-alpine

if [ $? -eq 0 ]; then
    echo "✅ Redis started on port 6379"
else
    echo "⚠️  Redis container may already exist"
fi

# Start ClickHouse
echo ""
echo "📦 Starting ClickHouse..."
docker run -d \
    --name hermesflow-clickhouse \
    -p 9000:9000 \
    -p 8123:8123 \
    --rm \
    --ulimit nofile=262144:262144 \
    clickhouse/clickhouse-server:latest

if [ $? -eq 0 ]; then
    echo "✅ ClickHouse started on ports 9000 (native) and 8123 (http)"
else
    echo "⚠️  ClickHouse container may already exist"
fi

# Wait for services to be ready
echo ""
echo "⏳ Waiting for services to be ready..."
sleep 5

# Test Redis
echo ""
echo "🔍 Testing Redis connection..."
if redis-cli ping > /dev/null 2>&1; then
    echo "✅ Redis is ready"
else
    echo "⚠️  Redis not responding yet, may need more time"
fi

# Test ClickHouse
echo ""
echo "🔍 Testing ClickHouse connection..."
if curl -s http://localhost:8123/ping > /dev/null 2>&1; then
    echo "✅ ClickHouse is ready"
else
    echo "⚠️  ClickHouse not responding yet, may need more time"
fi

echo ""
echo "=============================="
echo "✅ Development environment ready!"
echo "=============================="
echo ""
echo "📝 Service URLs:"
echo "   Redis:      redis://localhost:6379"
echo "   ClickHouse: tcp://localhost:9000 (native)"
echo "   ClickHouse: http://localhost:8123 (http)"
echo ""
echo "🛑 To stop services:"
echo "   docker stop hermesflow-redis hermesflow-clickhouse"
echo ""

