#!/bin/bash
set -e

echo "🔧 Formatting all Rust services..."

SERVICES=(
    "services/data-engine"
    "services/strategy-engine"
    "services/execution-engine"
    "services/strategy-generator"
    "services/backtest-engine"
    "services/risk-engine"
    "services/common"
)

for service in "${SERVICES[@]}"; do
    if [ -f "$service/Cargo.toml" ]; then
        echo "📦 Formatting $service..."
        (cd "$service" && cargo fmt)
    fi
done

echo "✅ All Rust services formatted!"
