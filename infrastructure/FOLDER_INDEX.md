# infrastructure - FOLDER_INDEX

> Infrastructure-as-code, database migrations, monitoring configuration, and deployment tooling.

## Directory Structure

```
infrastructure/
  database/
    postgres/
      migrations/              # 31 numbered SQL migrations (001-031 + 999_verification)
      schema.sql               # Full schema dump
      seeds/factors_seed.sql   # Factor definitions seed data
      functions/               # Stored functions (get_active_factors)
    clickhouse/
      migrations/              # 5 ClickHouse migrations (002-006)
      data/                    # ClickHouse data directory (volume-mounted)

  terraform/
    environments/
      dev/                     # Dev environment (main.tf, variables.tf, outputs.tf)
    modules/
      aks/                     # Azure Kubernetes Service
      acr/                     # Azure Container Registry
      database/                # Azure Database for PostgreSQL
      keyvault/                # Azure Key Vault (secret management)
      monitoring/              # Azure Monitor
      networking/              # VNet, subnets, NSGs

  prometheus/
    prometheus.yml             # Scrape config (data-engine, gateway, strategy-*)
    rules/
      data_integrity.yml       # Alert rules (stale data, gap detection, collector failures)

  grafana/
    provisioning/              # Data source + dashboard provisioning
    dashboards/                # Pre-built dashboard JSON

  alertmanager/
    alertmanager.yml           # Alert routing (→ Discord webhook)

  vector/
    vector.toml                # Log pipeline: Docker → ClickHouse

  aws/
    ecs/                       # ECS task definitions (ib-gateway, services)

  python/
    hermes_common/             # Shared Python utilities
      pyproject.toml
```

## Database Migration Summary

| Range | Domain |
|-------|--------|
| 001-006 | Core schema, market data, trading, active tokens, numeric fixes |
| 007 | Candle aggregates (TimescaleDB hypertable) |
| 008-010 | Factors, backtest results, strategy IDs |
| 011-017 | API metrics, watchlists, sync triggers, performance, constraints |
| 018 | Data quality incidents |
| 019-024 | Strategy partitioning, evolution, mode, ordering, trade orders |
| 025-028 | Trading accounts (config, capital, broker data, snapshots) |
| 029-031 | Backtest retention, portfolio ensemble, account parameterization |
