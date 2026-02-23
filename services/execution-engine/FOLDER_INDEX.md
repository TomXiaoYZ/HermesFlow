# execution-engine - FOLDER_INDEX

> Trade execution service. Subscribes to trade signals via Redis, executes orders across Solana/Raydium, IBKR (dual-gateway), and Futu brokers. Performs pre-trade risk checks and post-trade reconciliation.

**Note**: Excluded from Cargo workspace due to Solana SDK `tokio ~1.14` conflict. Build separately.

## Module Map

```
src/
  main.rs              # Entry point: broker init (Solana, IBKR x2, Futu), Redis sub, health server
                       #   - IBKR dual-gateway: ib-gateway (long_only) + ib-gateway-ls (long_short)
                       #   - Solana: optional (only if SOLANA_PRIVATE_KEY set)
  lib.rs               # Crate re-exports
  command_listener.rs  # Redis subscriber: listens for trade signals, dispatches to traders
  risk.rs              # Pre-trade risk checks: max order value, max positions, daily loss limit
                       #   - Per-account limits from trading_accounts table
  reconciliation.rs    # Post-trade reconciliation: DB vs broker position sync
  health.rs            # /health endpoint (port 8083)

  traders/
    mod.rs             # Trader trait + registry
    ibkr_trader.rs     # IBKR execution via TWS API (TCP)
                       #   - Two instances: long_only + long_short accounts
                       #   - Account summaries cached to DB every 30s
    solana_trader.rs   # Solana on-chain execution (Raydium/Jupiter)
                       #   - skip_preflight configurable via env
    raydium_trader.rs  # Raydium AMM-specific execution
    futu_trader.rs     # Futu execution via futu-bridge HTTP API

  bin/
    list_accounts.rs       # CLI: List IBKR accounts
    live_test_raydium.rs   # CLI: Live test Raydium execution
    paper_trading_test.rs  # CLI: Paper trading integration test
```

## Execution Flow

```
Redis Pub/Sub (trade signals) → command_listener
  → risk.rs (pre-trade checks)
  → traders/{ibkr,solana,raydium,futu}_trader
  → reconciliation.rs (post-trade DB sync)
```

## IBKR Dual-Gateway Architecture

| Account | Gateway Container | Host Env Var | Port |
|---------|-------------------|-------------|------|
| long_only | ib-gateway | `IBKR_HOST` | `IBKR_PORT` (4004) |
| long_short | ib-gateway-ls | `IBKR_HOST_LS` | `IBKR_PORT_LS` (4004) |

## Dependencies
- TimescaleDB (trading_accounts, trade_orders, positions)
- Redis (signal subscription)
- External: IB Gateway (TCP), Futu Bridge (HTTP), Solana RPC
