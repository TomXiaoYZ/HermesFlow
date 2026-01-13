#!/bin/bash
# Quick start script for data-engine development

set -e

echo "🚀 Data Engine Quick Start"
echo "=========================="
echo ""

# Check prerequisites
echo "Checking prerequisites..."

if ! command -v cargo &> /dev/null; then
    echo "❌ Rust/Cargo not found. Please install from https://rustup.rs/"
    exit 1
fi
echo "✅ Rust/Cargo found"

if ! command -v docker &> /dev/null; then
    echo "⚠️  Docker not found. Skipping container checks."
else
    echo "✅ Docker found"
fi

# Build the project
echo ""
echo "Building project..."
cargo build --release

# Check if config exists
if [ ! -f "config/dev.toml" ]; then
    echo ""
    echo "⚠️  No dev.toml found. Copying from default.toml..."
    cp config/default.toml config/dev.toml
    echo "📝 Please edit config/dev.toml with your settings before running."
    echo ""
    echo "Required settings:"
    echo "  - Postgres host, database, username, password"
    echo "  - Redis URL"
    echo "  - (Optional) Twitter credentials"
    echo "  - (Optional) Polymarket API settings"
    exit 0
fi

# Run the application
echo ""
echo "Starting data-engine..."
echo "Press Ctrl+C to stop"
echo ""

export RUST_ENV=dev
cargo run --release

