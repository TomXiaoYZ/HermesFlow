# Contributing to HermesFlow

👋 Welcome! Before you write any code, you **MUST** read the Engineering Standards.

## 🚨 Critical Standards
See **[docs/STANDARDS.md](docs/STANDARDS.md)** for the "Law" of this repository.

### Quick Summary
- **No Bash Scripts**: Use `Makefile`.
- **No Global Python**: Use `.venv` via `make setup`.
- **No Secrets**: Use `.env` and `DATA_ENGINE__` overrides.
- **No Copy-Paste**: Use `infrastructure/python/hermes_common`.

## How to Start
1. `make setup`
2. `make up`

## How to Submit
1. `make lint` (Must pass)
2. `make test` (Must pass)
3. Submit PR.
