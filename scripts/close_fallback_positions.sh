#!/usr/bin/env bash
# Close all positions left by the removed Fallback strategy.
# Run during US market hours (9:30 AM – 4:00 PM ET).
#
# Positions to close:
#   GXAI  +326 shares (sell to close)
#   USO   +6   shares (sell to close)
#   OLB   -1   share  (buy to cover short)
#
# These signals go through Redis -> execution-engine -> IBKR.

set -euo pipefail

REDIS_CMD="docker compose exec -T redis redis-cli"

echo "=== Closing Fallback positions ==="

# 1. Sell 326 GXAI (long position)
echo "[1/3] Selling 326 GXAI..."
$REDIS_CMD PUBLISH trade_signals "$(cat <<'JSON'
{"id":"00000000-0000-0000-0000-000000000001","symbol":"GXAI","side":"Sell","quantity":326.0,"price":null,"order_type":"Market","timestamp":"2026-02-18T00:00:00Z","reason":"Close Fallback position","strategy_id":"ManualClose","exchange":"polygon","mode":"long_only"}
JSON
)"

sleep 2

# 2. Sell 6 USO (long position)
echo "[2/3] Selling 6 USO..."
$REDIS_CMD PUBLISH trade_signals "$(cat <<'JSON'
{"id":"00000000-0000-0000-0000-000000000002","symbol":"USO","side":"Sell","quantity":6.0,"price":null,"order_type":"Market","timestamp":"2026-02-18T00:00:00Z","reason":"Close Fallback position","strategy_id":"ManualClose","exchange":"polygon","mode":"long_only"}
JSON
)"

sleep 2

# 3. Buy 1 OLB to cover (short position)
echo "[3/3] Buying 1 OLB to cover short..."
$REDIS_CMD PUBLISH trade_signals "$(cat <<'JSON'
{"id":"00000000-0000-0000-0000-000000000003","symbol":"OLB","side":"Buy","quantity":1.0,"price":null,"order_type":"Market","timestamp":"2026-02-18T00:00:00Z","reason":"Cover Fallback short","strategy_id":"ManualClose","exchange":"polygon","mode":"long_only"}
JSON
)"

echo ""
echo "=== Signals sent. Check execution-engine logs: ==="
echo "  docker compose logs execution-engine --tail 30"
echo ""
echo "After fills confirm, clean up DB records:"
echo "  docker compose exec -T timescaledb psql -U postgres -d hermesflow -c \\"
echo "    \"DELETE FROM trade_positions WHERE account_id = 'ibkr_long_only';\""
