#!/bin/bash
# Comprehensive Test Script for Data Engine

set -e

echo "🧪 Data Engine - Test Suite"
echo "============================"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✅ $2${NC}"
    else
        echo -e "${RED}❌ $2${NC}"
        exit 1
    fi
}

# 1. Code formatting check
echo ""
echo "📝 Checking code formatting..."
cargo fmt -- --check
print_status $? "Code formatting check"

# 2. Clippy lints
echo ""
echo "🔍 Running Clippy lints..."
cargo clippy --all-targets --all-features -- -D warnings
print_status $? "Clippy lints"

# 3. Unit tests
echo ""
echo "🧪 Running unit tests..."
cargo test --lib
print_status $? "Unit tests"

# 4. Integration tests
echo ""
echo "🔗 Running integration tests..."
cargo test --test '*'
print_status $? "Integration tests"

# 5. Doc tests
echo ""
echo "📚 Running doc tests..."
cargo test --doc
print_status $? "Doc tests"

# 6. Build release
echo ""
echo "🔨 Building release binary..."
cargo build --release
print_status $? "Release build"

# 7. Coverage (optional)
if command -v cargo-tarpaulin &> /dev/null; then
    echo ""
    echo "📊 Generating test coverage..."
    cargo tarpaulin --out Html --output-dir coverage --exclude-files 'benches/*' 'tests/*'
    print_status $? "Coverage report"
    echo "📈 Coverage report: coverage/index.html"
else
    echo ""
    echo -e "${YELLOW}⚠️  cargo-tarpaulin not installed, skipping coverage${NC}"
fi

# Summary
echo ""
echo "=============================="
echo -e "${GREEN}✅ All tests passed!${NC}"
echo "=============================="
echo ""
echo "📊 Test Summary:"
echo "   - Code formatting: ✅"
echo "   - Clippy lints:    ✅"
echo "   - Unit tests:      ✅"
echo "   - Integration:     ✅"
echo "   - Doc tests:       ✅"
echo "   - Release build:   ✅"
echo ""

