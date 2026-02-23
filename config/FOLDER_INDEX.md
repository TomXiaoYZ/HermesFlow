# config - FOLDER_INDEX

> Runtime configuration files for the strategy evolution pipeline.

## Files

| File | Purpose | Used By |
|------|---------|---------|
| `factors.yaml` | Factor definitions for **crypto** backtest/strategy engines | backtest-engine, strategy-engine |
| `factors-stock.yaml` | 25 factor definitions for **stock** (Polygon) evolution | strategy-generator |
| `generator.yaml` | Strategy generator master config | strategy-generator |

## generator.yaml Sections

| Section | Purpose |
|---------|---------|
| `exchanges` | Per-exchange config: resolution, lookback, factor_config, multi_timeframe, walk_forward |
| `threshold_config` | P4 adaptive thresholds: percentile/clamp for long_only + long_short, per-symbol overrides |
| `ensemble` | P5 portfolio ensemble: HRP, dynamic weights, crowding, rebalance interval |
| `llm_oracle` | P2 LLM mutation: provider, model, trigger thresholds, cooldown |
