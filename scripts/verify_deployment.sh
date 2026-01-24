#!/bin/bash
set -e

echo "Checking container status..."
docker ps --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"

echo "\nChecking Redis connection..."
docker exec hermesflow-redis-1 redis-cli ping

echo "\nChecking Web UI..."
curl -I http://localhost:3000 || echo "Web UI not accessible"

echo "\nChecking for Market Data in Redis..."
# Timeout after 5 seconds
timeout 5s docker exec hermesflow-redis-1 redis-cli subscribe market_data | head -n 4 || echo "No market data received"

echo "\nDone."
