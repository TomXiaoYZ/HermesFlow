# data-engine - FOLDER_INDEX

> Market data aggregation service. Collects from 12+ data sources, normalizes to `StandardMarketData`, persists to TimescaleDB/ClickHouse, broadcasts via Redis Pub/Sub.

## Module Map

```
src/
  main.rs                    # Entry point: config load, DB/Redis/CH init, collector spawn, Axum server
  lib.rs                     # Crate re-exports
  config.rs                  # 12-Factor config (env → toml cascade)
  error.rs                   # DataError + DataEngineError (thiserror)
  health.rs                  # /health endpoint
  collector_spawner.rs       # Spawns collector tasks per exchange/symbol
  sync_worker.rs             # Background sync worker for historical data

  collectors/
    mod.rs                   # Collector trait + registry
    circuit_breaker.rs       # Circuit breaker for failing collectors
    binance/                 # Binance spot (REST + WebSocket)
      client.rs              #   HTTP client
      config.rs              #   Exchange-specific config
      connector.rs           #   DataCollector impl
      websocket.rs           #   WS stream handler
    birdeye/                 # Birdeye (Solana token data)
      client.rs / config.rs / connector.rs
      meta_collector.rs      #   Token metadata enrichment
    bybit/                   # Bybit derivatives (REST + WebSocket)
      client.rs / config.rs / connector.rs / websocket.rs
    dexscreener/             # DexScreener (DEX aggregated data)
      client.rs / config.rs / connector.rs
    futu/                    # Futu (HK stocks via futu-bridge)
      client.rs / config.rs / connector.rs
    helius/                  # Helius (Solana RPC + DAS API)
      client.rs / config.rs / connector.rs
    jupiter/                 # Jupiter (Solana DEX aggregator)
      client.rs / config.rs / connector.rs
    massive/                 # Massive (Polygon WebSocket proxy)
      client.rs / connector.rs / websocket.rs
    okx/                     # OKX exchange (REST + WebSocket)
      client.rs / config.rs / connector.rs / websocket.rs
    polygon/                 # Polygon.io (US stocks)
      client.rs / config.rs / connector.rs
      historical_sync.rs     #   Backfill historical OHLCV
      types.rs               #   Polygon-specific types
    akshare.rs               # AKShare (China A-shares)
    ibkr.rs                  # IBKR market data
    polymarket.rs            # Polymarket (prediction markets)
    twitter.rs               # Twitter sentiment

  models/
    mod.rs                   # Model re-exports
    market_data.rs           # StandardMarketData (unified type, rust_decimal)
    market_data_type.rs      # MarketDataType enum
    asset_type.rs            # AssetType enum (Crypto, Stock, etc.)
    data_source_type.rs      # DataSourceType enum
    candle.rs                # OHLCV candle model
    prediction_data.rs       # Prediction market data
    social_data.rs           # Social sentiment data
    token_metadata.rs        # Token/asset metadata
    trading.rs               # Trading-related models

  monitoring/
    mod.rs                   # Monitoring module exports
    quality.rs               # 7-stage data quality pipeline
    health.rs                # Collector health tracking
    metrics.rs               # Prometheus metrics
    logging.rs               # Structured logging
    dead_letter.rs           # Dead letter queue for failed ingestion
    market_schedule.rs       # Market hours / trading calendar

  repository/
    mod.rs                   # Repository traits (MarketDataRepository, etc.)
    token.rs                 # Token repository trait
    token_export.rs          # Token export utilities
    postgres/
      mod.rs                 # PostgreSQL implementations
      market_data.rs         #   Market data CRUD
      metrics.rs             #   Metrics persistence
      migration.rs           #   Schema migration runner
      prediction.rs          #   Prediction data persistence
      social.rs              #   Social data persistence
      token.rs               #   Token metadata persistence
      trading.rs             #   Trading data persistence

  server/
    mod.rs                   # Axum server setup
    routes.rs                # Route definitions
    handlers/
      mod.rs                 # Handler re-exports
      agent.rs               #   Agent management endpoints
      config.rs              #   Config query endpoints
      data.rs                #   Market data endpoints
      history.rs             #   Historical data endpoints
      jobs.rs                #   Background job endpoints
      prediction.rs          #   Prediction data endpoints
      trading.rs             #   Trading data endpoints

  storage/
    mod.rs                   # Storage layer exports
    redis.rs                 # Redis cache + pub/sub
    clickhouse.rs            # ClickHouse OLAP client

  tasks/
    mod.rs                   # Task scheduler
    manager.rs               # Task lifecycle management
    candle_aggregation.rs    # OHLCV aggregation (1m→5m→15m→1h→4h→1d→1w)
    data_quality.rs          # Periodic quality audit task
    historical_sync.rs       # Historical backfill task
    polygon_sync.rs          # Polygon-specific sync
    token_discovery.rs       # Auto-discover new tokens/symbols

  trading/
    mod.rs                   # Trading utilities
    ibkr_trader.rs           # IBKR trading integration

  traits/
    mod.rs                   # Trait re-exports
    connector.rs             # DataCollector trait
    parser.rs                # DataParser trait

  registry/
    mod.rs                   # Registry exports
    parser_registry.rs       # Parser registration

  utils/
    mod.rs                   # Utility functions

  bin/
    backfill.rs              # CLI: Historical data backfill
    replay-dead-letters.rs   # CLI: Replay failed ingestion
    sync-market.rs           # CLI: One-shot market sync
```

## Key Patterns
- **Collector Trait** (`traits/connector.rs`): All 12+ data sources implement `DataCollector`
- **StandardMarketData**: Unified type using `rust_decimal::Decimal` for financial precision
- **7-Stage Quality Pipeline** (`monitoring/quality.rs`): Active count, freshness, gap detection, liquidity guard, price spike, cross-source divergence, volume anomaly
  - P6-3A: Poisson-based dynamic staleness detection (per-symbol EWMA tick arrival rate λ_i)
  - P7-0A: Corrected alert semantics ("symbol-exchange pairs" instead of "symbols")
  - P7-0B: EWMA tick_rates reset on market open transition (prevents Monday false positives)
- **Market Calendar** (`monitoring/market_schedule.rs`): P6-3B timezone-aware NYSE calendar, auto-suspend flow alerts during off-hours
  - P7-0B: Extended holidays to 2028, check_holiday_coverage() at startup, DST edge-case tests
- **Tiered Scheduling**: Critical (30s), warning (5m), full audit (1h)

## Dependencies
- `common` (workspace crate)
- TimescaleDB, Redis, ClickHouse
