# HermesFlow Strategy Evolution — Development Roadmap

**Date**: 2026-02-22
**Current state**: P0–P4 deployed, P5–P6 planned

---

## Stage Summary

| Stage | Name | Status | Goal |
|-------|------|--------|------|
| **P0** | Walk-Forward OOS Evaluation | Deployed | Fix OOS evaluation (35% → 94% success) |
| **P1** | Factor Enrichment (13 → 25) | Deployed | Reduce `too_few_trades` via richer signals |
| **P2** | LLM-Guided Mutation | Deployed | Accelerate convergence with domain-aware mutation |
| **P3** | Multi-Timeframe Factor Stacking | Deployed | Add temporal depth (1h + 4h + 1d factors) |
| **P4** | Adaptive Threshold Tuning | Deployed | Per-symbol threshold optimization with utilization feedback |
| **P5** | Strategy Ensemble & Portfolio | Planned | Combine top strategies into portfolio allocation |
| **P6** | Live Paper Trading Integration | Planned | Forward-test top strategies against real market |

---

## P0 — Walk-Forward OOS Evaluation (COMPLETE)

**Commit**: `78b1c63`
**Report**: `docs/P0_IMPLEMENTATION_REPORT.md`

### Delivered
- 10 distinct sentinel values replacing opaque `-10.0`
- Walk-forward OOS with configurable K-fold steps
- Resolution-aware embargo gaps (20/10/8 bars for 1d/1h/15m)
- Aggregate OOS PSR with failure mode decomposition
- Metadata persistence for monitoring

### Metrics
- OOS evaluation success: 35% → 94.1%
- 100% of remaining failures identified as `too_few_trades`

---

## P1 — Factor Enrichment, 13 → 25 (COMPLETE)

**Commit**: `63d5464` (factors), `8b7110a` (promotion monitoring)
**Reports**: `docs/P1_IMPLEMENTATION_REPORT.md`, `docs/P1_TFT_COMPARISON_REPORT.md`

### Delivered
- 6 existing indicators activated (ATR%, OBV%, MFI, BB%B, MACD hist, SMA200 diff)
- 3 new microstructure factors (Amihud illiquidity, spread proxy, return autocorrelation)
- 3 cross-asset SPY reference factors (correlation, beta, relative strength)
- Genome compatibility handling (feat_offset detection)
- ALPS promotion rate monitoring (50-gen rolling window)

### Metrics (at ~270 gens)
- TFT rate: 27.3% (improving, Q1: 38.3% → Q3: 18.7%)
- OOS PSR quality: median 1.721 (vs 1.242 in 13-factor, +38.6%)
- ALPS promotion rates: 84-89% across all boundaries (healthy)

---

## P2 — LLM-Guided Mutation (COMPLETE)

**Commits**: `2e2e9fc` through `9d3a42b` (P2a–P2e)
**Reports**: `docs/P2_ARCHITECTURE_DESIGN.md`, `docs/P2_IMPLEMENTATION_REPORT.md`, `docs/P2_SUMMARY_FOR_GEMINI.md`
**Total LoC**: ~1,582

### Delivered (5 sub-phases)

| Sub-phase | Component | Files | Tests |
|-----------|-----------|-------|-------|
| P2a | Genome Decoder (`genome_decoder.rs`) — bidirectional RPN ↔ token conversion | 1 new | 12 |
| P2b | LLM Oracle Core (`llm_oracle.rs`) — multi-provider (Bedrock/Anthropic/OpenAI), prompt construction, validation | 1 new | 7 |
| P2c | Trigger Integration — dual triggers (promotion rate < 70%, TFT rate > 40%), cooldown, L0 injection | 2 modified | — |
| P2d | Monitoring + Frontend — Oracle Interactions panel, metadata JSONB audit trail | 3 modified | — |
| P2e | Cross-Symbol Learning — top OOS-PSR formulas from other symbols in LLM prompt | 1 modified | — |

### Results (as of gen ~78,000)
- TFT 100% stuck symbols: 11/26 (42%) → **0/26 (0%)**
- OOS PSR > 0: 13/26 (50%) → **26/26 (100%)**
- OOS PSR > 1: 7/26 (27%) → **24/26 (92%)**
- OOS PSR > 2: 4/26 (15%) → **19/26 (73%)**
- Average OOS PSR: ~0.5 → **2.521**

### Key Insight
Threshold ceiling fix (`[0.52, 0.70]` clamp) proved more impactful than 1,000+ Oracle-injected genomes. **Thresholds matter more than formulas** — this motivated P4.

---

## P3 — Multi-Timeframe Factor Stacking (COMPLETE)

**Commit**: `a3de5fe`
**Reports**: `docs/P3_ARCHITECTURE_DESIGN.md`, `docs/P3_IMPLEMENTATION_REPORT.md`, `docs/P3.5_TECHNICAL_REPORT.md`
**LoC**: ~678

### Delivered
- 3-resolution stacking: 1h, 4h, 1d → 25 × 3 = 75 total features
- Token layout: 0–24 = 1h, 25–49 = 4h, 50–74 = 1d, 75+ = operators
- `MultiTimeframeFactorConfig`: wraps single config, replicates across resolutions
- `forward_fill_align()`: temporal alignment at O(T log T) via binary search
- `load_data_multi_timeframe()`: parallel DB queries at 3 resolutions via `buffer_unordered(8)`
- LLM Oracle prompt updated with timeframe token explanations
- Feature gate: `config/generator.yaml` → `multi_timeframe.enabled`
- Genome compatibility: P2 (feat_offset=25) and P3 (feat_offset=75) genomes coexist

### Data Availability (TimescaleDB)
- 1h: 1,165,969 rows
- 4h: 315,737 rows
- 1d: 202,641 rows

### P3.5 Follow-ups (deployed)
- Z-score normalization fix for LongShort mode (commit `53338d4`)
- Unit tests for portfolio.rs and genetic.rs (commit `0053389`)
- Parallelized MTF data loading (commit `dc30047`)

---

## P4 — Adaptive Threshold Tuning (COMPLETE)

**Commit**: `8e84677`
**LoC**: ~895 (3 files)

### Delivered (3 sub-phases)

#### Phase 1: UtilizationTracker + WalkForwardStep Extension
- `WalkForwardStep` extended with `long_bars` and `short_bars` fields
- `psr_fitness_oos` returns 5-tuple: `(psr, trade_count, active_bars, long_bars, short_bars)`
- `UtilizationTracker` (rolling 50-gen window) tracks long/short ratios and total utilization
- Utilization metrics added to generation metadata JSONB

#### Phase 2: Per-Symbol Threshold Config
- `ThresholdConfig` with `resolve_upper` / `resolve_lower` methods supporting per-symbol overrides
- `config/generator.yaml` → `threshold_config` section with global defaults + per-symbol overrides
- All three adaptive threshold functions parameterized (no more hardcoded percentile/clamp values)
- NVDA LongOnly clamp relaxed to `[0.52, 0.85]` (was hitting 0.70 ceiling 18.8% of time)
- QQQ LongShort asymmetric percentiles (65/35 vs global 70/30)

#### Phase 3: Dynamic Threshold Adjustment
- `adjust_threshold_params` called every 50 generations per (symbol, mode)
- Relaxes thresholds when utilization < 30% (too few trades → lower percentile, widen clamps)
- Tightens thresholds when utilization > 80% (too many trades → raise percentile, narrow clamps)
- Corrects long/short asymmetry for LongShort mode (long_ratio > 0.8 or short_ratio > 0.8)
- Guardrails: percentile bounds (upper: 55–85, lower: 15–45), minimum 10 data points required

### Design Decision
The original plan discussed three approaches (genome encoding, grid search, Bayesian optimization). The implemented approach uses **utilization-feedback-driven dynamic adjustment** — a fourth approach that emerged from analyzing live DB data showing utilization as the key diagnostic metric. This proved more practical than the originally proposed approaches.

### Tests
- 13 new unit tests (109 total for strategy-generator)

---

## P5 — Strategy Ensemble & Portfolio (PLANNED)

**Goal**: Combine top-performing strategies from multiple symbols into a diversified portfolio with allocation weights.

### Rationale
- Individual strategies have varying Sharpe ratios and correlation profiles
- An ensemble of 5-10 uncorrelated strategies can achieve portfolio Sharpe > any individual
- Walk-forward validated strategies provide reliable OOS Sharpe estimates

### Design Sketch

1. **Strategy selection**: Top N strategies per symbol with OOS PSR > 1.5 and valid walk-forward
2. **Correlation matrix**: Compute pairwise return correlation across strategies
3. **Portfolio optimization**: Mean-variance optimization (or risk parity) for allocation weights
4. **Rebalancing**: Re-run selection + optimization weekly as new strategies evolve
5. **Output**: Portfolio definition file for `strategy-engine` (live execution)

### New Components
- `services/portfolio-optimizer/` — new Rust service or module within strategy-generator
- Portfolio optimization: minimum variance, max Sharpe, or risk parity
- Strategy correlation cache in TimescaleDB

### Estimated Effort
- New module: ~500-800 LoC
- Depends on: Sufficient elite strategies (P1+P2 convergence)

---

## P6 — Live Paper Trading Integration (PLANNED)

**Goal**: Forward-test evolved strategies against real-time market data before live execution with real capital.

### Rationale
- Walk-forward validation uses historical data — paper trading validates against live market microstructure
- Detects issues: slippage, partial fills, market impact, data lag, after-hours gaps
- Builds confidence before committing real capital

### Design Sketch

1. **Strategy deployment**: Export top walk-forward-validated strategies to `strategy-engine`
2. **Paper account**: Use IBKR paper trading account (already configured via `ib-gateway`)
3. **Signal generation**: `strategy-engine` computes VM signals from real-time data
4. **Paper execution**: `execution-engine` sends orders to paper account
5. **Performance tracking**: Compare paper PnL vs backtest expectations
6. **Promotion criteria**: Strategy paper-trades for 20+ trading days with Sharpe > 1.0 → eligible for live capital

### Integration Points
- `strategy-engine` ← strategy definitions from `strategy-generator`
- `execution-engine` ← signals from `strategy-engine`
- `data-engine` → real-time data to both engines

### Estimated Effort
- Mostly integration work, existing services handle the heavy lifting
- ~300 LoC of glue code + configuration
- Requires: P5 portfolio allocation for position sizing

---

## Dependency Graph

```
P0 (OOS eval) ──> P1 (factors) ──> P2 (LLM mutation) ──> P3 (multi-TF) ──> P4 (thresholds)
                       │                                                         │
                       └──> P5 (portfolio) ──────────────────────────────────> P6 (paper trading)
```

- **P2 depends on P1**: LLM oracle references the 25-factor vocabulary
- **P3 depends on P2**: 75-feature space requires guided search
- **P4 depends on P3**: Uses utilization feedback from walk-forward steps with MTF data
- **P5 depends on convergence**: Needs sufficient high-PSR strategies (10+)
- **P6 depends on P5**: Needs portfolio allocation for position sizing

---

## Completed Implementation Order

1. **P0**: Walk-forward OOS evaluation fix (commit `78b1c63`)
2. **P1**: Factor enrichment 13→25 (commit `63d5464`)
3. **P2**: LLM-guided mutation oracle, 5 sub-phases (commits `2e2e9fc`–`9d3a42b`)
4. **P3**: Multi-timeframe factor stacking (commit `a3de5fe`)
5. **P4**: Adaptive threshold tuning (commit `8e84677`)

## Remaining
6. **P5**: Portfolio optimization (once we have 10+ high-PSR strategies across symbols)
7. **P6**: Paper trading (final validation step before live capital)
