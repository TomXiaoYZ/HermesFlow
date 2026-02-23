# HermesFlow Strategy Evolution — Development Roadmap

**Date**: 2026-02-23
**Current state**: P0–P5 deployed, P6 designed (3 phases)

---

## Stage Summary

| Stage | Name | Status | Goal |
|-------|------|--------|------|
| **P0** | Walk-Forward OOS Evaluation | Deployed | Fix OOS evaluation (35% → 94% success) |
| **P1** | Factor Enrichment (13 → 25) | Deployed | Reduce `too_few_trades` via richer signals |
| **P2** | LLM-Guided Mutation | Deployed | Accelerate convergence with domain-aware mutation |
| **P3** | Multi-Timeframe Factor Stacking | Deployed | Add temporal depth (1h + 4h + 1d factors) |
| **P4** | Adaptive Threshold Tuning | Deployed | Per-symbol threshold optimization with utilization feedback |
| **P5** | Strategy Ensemble & Portfolio | Deployed | HRP portfolio ensemble with dynamic weights and crowding detection |
| **P6a** | Foundation Hardening | Designed | Atomic blocks, protected-op penalty, EWMA covariance, rayon isolation |
| **P6b** | Paper Trading + Deadzone | Designed | Strategy deployment pipeline, paper accounts, turnover deadzone |
| **P6c** | MCTS + Execution | Designed | MCTS symbolic regression, execution optimization, multi-exchange |

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

## P5 — Strategy Ensemble & Portfolio (COMPLETE)

**Commit**: `5fde72b`
**Report**: `docs/P5_IMPLEMENTATION_REPORT.md`
**LoC**: ~1,200

### Delivered

#### HRP Portfolio Allocation (Pure-Math Implementation)
- 5-stage Hierarchical Risk Parity pipeline: correlation → distance → single-linkage clustering → optimal leaf ordering → recursive bisection
- No external optimization libraries — implemented from first principles using `ndarray`
- Pearson correlation matrix with minimum 30-bar data requirement

#### Dynamic Weight Adjustment
- Momentum tilt: recent Sharpe ratio bias (α = 0.3)
- Volatility scaling: inverse realized vol weighting
- Drawdown dampening: reduce allocation to strategies in drawdown > 10%
- Crowding detection: penalize strategies with correlation > 0.7

#### Ensemble Selection & Rebalancing
- Top-N strategy selection per (exchange, symbol, mode) with OOS PSR > 1.5
- Shadow equity tracking across rebalance cycles
- 30-minute rebalance interval (configurable)
- Full persistence to `portfolio_ensembles` TimescaleDB table

### Tests
- 45 unit tests covering HRP math, dynamic weights, crowding detection, and rebalance logic
- 353/353 workspace tests pass

---

## P6 — Full-Stack Evolution Upgrade + Live Paper Trading (DESIGNED)

**Design doc**: `docs/plans/2026-02-23-p6-design.md`
**Goal**: Harden the evolution engine, validate strategies via paper trading, and upgrade search from heuristic GP to MCTS symbolic regression.

### Three-Phase Architecture

```
P6a (Foundation Hardening) → P6b (Paper Trading) → P6c (MCTS + Execution)
```

### P6a — Foundation Hardening (8 items)

| ID | Component | Service | Description |
|----|-----------|---------|-------------|
| E1 | Atomic Semantic Blocks | strategy-generator, backtest-engine | `block_mask` protecting LLM-generated RPN fragments from destructive crossover/mutation |
| E2 | Protected-Op Penalty | backtest-engine, strategy-generator | Fitness penalty when protection_ratio > 15% to combat deceptive pseudo-signals |
| E3 | SIMD Vectorization* | backtest-engine | AVX2/NEON batch processing for RPN VM arithmetic (conditional on profiling) |
| G1 | ndarray Zero-Copy | strategy-generator, backtest-engine | ArrayView + stride manipulation for MTF alignment without memory allocation |
| G2 | rayon CPU Isolation | strategy-generator | Separate thread pool for CPU-bound fitness evaluation, freeing Tokio for I/O |
| G3 | rkyv Deserialization* | common, data-engine, strategy-engine | Zero-copy Redis deserialization (conditional on profiling) |
| F1 | EWMA Covariance | strategy-generator | Exponentially weighted moving average (λ=0.94) replacing equal-weight correlation |
| F2 | Portfolio Turnover Cost | strategy-generator | Per-venue cost modeling in shadow equity tracking |

### P6b — Paper Trading + Deadzone (6 items)

| ID | Component | Service | Description |
|----|-----------|---------|-------------|
| B1 | Strategy Deployment Pipeline | strategy-engine, strategy-generator | `deployed_strategies` table + VM signal generation from live data |
| B2 | Paper Account Integration | execution-engine | Paper mode for IBKR, Futu, Solana devnet with isolated tracking tables |
| B3 | Promotion Criteria | execution-engine | 5-criterion gate (20d, Sharpe>1, <20% deviation, <15% drawdown, >90% fills) |
| C1 | Turnover Deadzone + L1 | strategy-generator | Dynamic lambda with vol-regime scaling to suppress micro-rebalancing |
| F3 | Regime-Aware Rebalance | strategy-generator | Volatility-driven frequency: Low/Normal/High → 240/60/15 min |
| F4 | Ensemble Walk-Forward | strategy-generator | Portfolio-level backtest validation (scheduled daily 03:00 UTC) |

### P6c — MCTS + Execution (4 items)

| ID | Component | Service | Description |
|----|-----------|---------|-------------|
| A1 | MCTS Engine | strategy-generator | 2000-3000 LoC new module: UCT selection, LLM policy prior, risk-seeking gradients |
| A2 | Search Space Constraints | strategy-generator | 30-token limit, legal action masks, discrete constant set |
| D1 | Execution Optimization | execution-engine | 3 levels: profiling → crossbeam SPSC → core_affinity + io_uring (Linux) |
| D2 | Multi-Exchange Portfolio | strategy-generator | Cross-venue HRP with daily normalization, per-exchange capital limits |

### New DB Migrations
- `031_deployed_strategies.sql` — strategy deployment tracking (P6b)
- `032_paper_trading.sql` — paper trade orders, executions, positions, daily summary (P6b)
- `033_ensemble_backtest.sql` — ensemble-level backtest results (P6b)

### Multi-Exchange Support
Execution venues (not data sources): IBKR (US stocks), Binance/OKX/Bybit (crypto), Futu (HK stocks), Longbridge (future), Solana (DeFi). Adding new exchanges requires only a config entry in `capital_limits`.

---

## Dependency Graph

```
P0 (OOS eval) ──> P1 (factors) ──> P2 (LLM mutation) ──> P3 (multi-TF) ──> P4 (thresholds)
                       │                                                         │
                       └──> P5 (portfolio) ──> P6a (hardening) ──> P6b (paper) ──> P6c (MCTS)
```

- **P2 depends on P1**: LLM oracle references the 25-factor vocabulary
- **P3 depends on P2**: 75-feature space requires guided search
- **P4 depends on P3**: Uses utilization feedback from walk-forward steps with MTF data
- **P5 depends on convergence**: Needs sufficient high-PSR strategies (10+)
- **P6a depends on P5**: Enhances evolution and portfolio components built in P5
- **P6b depends on P6a**: Paper trading needs turnover cost model (F2) and EWMA covariance (F1)
- **P6c depends on P6b**: MCTS needs enhanced VM (E1, E2); execution optimization needs paper trading baseline (B2)

---

## Completed Implementation Order

1. **P0**: Walk-forward OOS evaluation fix (commit `78b1c63`)
2. **P1**: Factor enrichment 13→25 (commit `63d5464`)
3. **P2**: LLM-guided mutation oracle, 5 sub-phases (commits `2e2e9fc`–`9d3a42b`)
4. **P3**: Multi-timeframe factor stacking (commit `a3de5fe`)
5. **P4**: Adaptive threshold tuning (commit `8e84677`)
6. **P5**: HRP portfolio ensemble with dynamic weights (commit `5fde72b`)

## Remaining
7. **P6a**: Foundation hardening — atomic blocks, protected-op penalty, EWMA covariance, rayon isolation
8. **P6b**: Paper trading — strategy deployment, paper accounts, turnover deadzone, regime-aware rebalancing
9. **P6c**: MCTS + execution — symbolic regression engine, execution optimization, multi-exchange portfolio
