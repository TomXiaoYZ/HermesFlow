#!/bin/bash
# Performance Benchmark Script for Data Engine

set -e

echo "📊 Data Engine - Performance Benchmarks"
echo "========================================"

# Run all benchmarks
echo ""
echo "🚀 Running parser benchmarks..."
cargo bench --bench parser_benchmarks

echo ""
echo "🚀 Running storage benchmarks..."
cargo bench --bench storage_benchmarks

echo ""
echo "=============================="
echo "✅ Benchmarks complete!"
echo "=============================="
echo ""
echo "📈 Results saved to: target/criterion/"
echo ""
echo "🔍 To view HTML reports:"
echo "   open target/criterion/report/index.html"
echo ""

