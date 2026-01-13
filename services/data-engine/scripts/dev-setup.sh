#!/bin/bash
# Development Environment Setup Script for Data Engine

set -e

echo "🚀 Data Engine - Development Setup"
echo "=================================="

# Check Rust installation
echo ""
echo "📦 Checking Rust installation..."
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust not found. Please install Rust: https://rustup.rs/"
    exit 1
fi
echo "✅ Rust version: $(rustc --version)"

# Check required tools
echo ""
echo "🔧 Checking required tools..."

if ! command -v docker &> /dev/null; then
    echo "⚠️  Docker not found (optional for testing)"
else
    echo "✅ Docker: $(docker --version)"
fi

if ! command -v redis-cli &> /dev/null; then
    echo "⚠️  Redis CLI not found (optional for testing)"
else
    echo "✅ Redis CLI available"
fi

# Install cargo tools
echo ""
echo "🛠️  Installing cargo tools..."

if ! command -v cargo-tarpaulin &> /dev/null; then
    echo "Installing cargo-tarpaulin for coverage..."
    cargo install cargo-tarpaulin
fi

if ! command -v cargo-watch &> /dev/null; then
    echo "Installing cargo-watch for development..."
    cargo install cargo-watch
fi

# Create config directory if not exists
echo ""
echo "📁 Setting up configuration..."
mkdir -p config

# Set development environment
export RUST_ENV=dev
export RUST_LOG=debug

echo ""
echo "✅ Setup complete!"
echo ""
echo "📝 Next steps:"
echo "   1. Start Redis:      docker run -d -p 6379:6379 redis:7-alpine"
echo "   2. Start ClickHouse: docker run -d -p 9000:9000 clickhouse/clickhouse-server"
echo "   3. Run tests:        cargo test"
echo "   4. Run dev server:   cargo run"
echo "   5. Watch mode:       cargo watch -x run"
echo ""

