---
description: Security rules for a financial trading platform handling real money
globs: ["**/*"]
---

# Security Rules

## CRITICAL: This Platform Handles Real Money

## Before ANY Commit
- [ ] No hardcoded secrets (API keys, passwords, private keys, tokens)
- [ ] All user inputs validated
- [ ] SQL injection prevention (SQLx parameterized queries only)
- [ ] No `unwrap()` on external/user data
- [ ] Error messages don't leak internal details
- [ ] Rate limiting on public endpoints
- [ ] Authentication/authorization verified

## Secret Management
- NEVER hardcode secrets in source code
- Use environment variables or secret manager
- Validate required secrets exist at startup
- Rotate any secrets that may have been exposed
- Never log secrets or include in error messages

## Financial Operations
- Verify wallet signatures before transactions
- Check balances before execution
- Atomic transaction handling
- Double-spend prevention
- Slippage protection
- Maximum order size limits
- Audit log every trade execution

## Rust-Specific
- No `unsafe` without documented justification
- No `unwrap()` / `expect()` on external data
- Bounds checking on numerical operations
- Constant-time comparison for secrets

## If Security Issue Found
1. STOP all other work immediately
2. Assess severity and impact
3. Fix CRITICAL issues before any other changes
4. Rotate exposed secrets
5. Audit for similar patterns across codebase
