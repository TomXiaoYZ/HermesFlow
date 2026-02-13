---
description: Testing standards and requirements
globs: ["**/*.rs", "**/*.ts", "**/*.tsx", "**/*.py"]
---

# Testing Standards

## Coverage Targets
| Component | Minimum |
|-----------|---------|
| Financial/trading logic | 100% |
| Repository implementations | 80% |
| API handlers | 80% |
| Utility functions | 80% |
| UI components | 70% |

## TDD Workflow
1. Write test first (RED)
2. Implement minimal code (GREEN)
3. Refactor while green (REFACTOR)
4. Verify with `cargo test --workspace`

## Test Commands
```bash
# Rust workspace
cargo test --workspace

# Single service
cargo test -p data-engine

# execution-engine (separate)
cd services/execution-engine && cargo test

# Frontend
cd services/web && npm test

# Python
cd services/futu-bridge && pytest
```

## Edge Cases to Test
- Empty/null inputs
- Boundary values (max numbers, zero, negative)
- Network failures and timeouts
- Database constraint violations
- Race conditions in async code
- Financial calculation overflow/underflow
- Invalid market data (NaN, negative prices)

## Anti-Patterns
- Tests depending on execution order
- Tests with side effects on shared state
- Testing implementation details vs behavior
- Ignoring async test isolation
- Using production data in tests
