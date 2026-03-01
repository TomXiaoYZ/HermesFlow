# HermesFlow Strategy Evolution — Development Roadmap

**Date**: 2026-03-01
**Current state**: P0–P7 deployed, P8 designed (3 phases)

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
| **P6** | Full-Stack Evolution Upgrade | Deployed | Temporal causality, lFDR, CCIPCA, MCTS, shadow trading, decay routing |
| **P7** | Statistical Barriers + MCTS Integration | Deployed | MCTS evolution loop wiring, lFDR filtering, shadow promotion guard |
| **P8** | Semantic Prior + Active Reduction | Designed | LLM-guided MCTS, CCIPCA augmentation, diversity trigger, VM optimization |

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

## P6 — Full-Stack Evolution Upgrade (COMPLETE)

**Commits**: `d1908e1` (MCTS), `ee09590` (dead_code), multiple prior commits
**Goal**: Harden the evolution engine, add statistical safeguards, and integrate MCTS symbolic regression.

### Delivered

| ID | Component | Description |
|----|-----------|-------------|
| P6-1A | Publication Delay | Temporal causality alignment per resolution (0/300/900s for 1h/4h/1d) |
| P6-1B | Local FDR | n-gram Jaccard clustering → per-cluster lFDR hypothesis testing |
| P6-1C | CCIPCA | O(n·k) incremental PCA (diagnostic mode, active remapping deferred to P8) |
| P6-1D | Decay Routing | Non-linear decay buffer (active→decaying→retired) for strategy lifecycle |
| P6-2A | Hysteresis Dead-Zone | Per-asset no-trade threshold with L1 regularization to reduce micro-rebalancing |
| P6-2B | Shadow Trading | Shadow signal table, shadow status columns, execution quality metrics |
| P6-4A | MCTS Engine | Arena-allocated MCTS: contiguous Vec<Node>, u32 indices, zero-cost GC |
| P6-4B | Extreme Bandit PUCT | Configurable mean vs max reward variant |
| P6-4C | LLM Cached Policy | HashMap cache with uniform fallback (ready for P8 activation) |
| P6-4D | Deception Suppressor | FNV-1a n-gram hashing with exponential decay penalty |
| E2 | Protected-Op Penalty | Fitness penalty when VM protection_ratio > 15% |
| G2 | rayon CPU Isolation | Separate Rayon thread pool for CPU-bound fitness evaluation |
| F1 | EWMA Covariance | Exponentially weighted covariance (λ=0.94) for HRP allocation |
| F2 | Turnover Cost | Per-venue cost modeling in shadow equity tracking |
| F3 | Regime-Aware Rebalance | Volatility-driven frequency: Low/Normal/High → 240/60/15 min |
| C1 | Turnover Deadzone + L1 | Dynamic lambda with vol-regime scaling |

### New DB Migrations
- `035`: Strategy decay routing (decay_state, decay_factor columns)
- `036`: Shadow trading signals table + shadow status columns
- `037`: Execution quality metrics table

---

## P7 — Statistical Barriers + MCTS Integration (COMPLETE)

**Commit**: `a342a49`
**Goal**: Wire MCTS into the evolution loop, add statistical filtering, and establish shadow-to-live promotion safeguards.

### Delivered

| ID | Component | Description |
|----|-----------|-------------|
| P7-1A | MCTS Config | `MctsYamlConfig` + `LfdrConfig` deserialization from YAML |
| P7-1B | MCTS Thread Pool | Dedicated Rayon pool (2 threads default, configurable via `MCTS_THREADS`) |
| P7-1C | MCTS Evolution Integration | Seeds inject into ALPS L0 every N generations (configurable interval) |
| P7-1D | MCTS Integration Tests | Valid genome validation, token conversion tests |
| P7-2A | lFDR Ensemble Filtering | `select_candidates_with_lfdr()` — RPN n-gram clustering before ensemble selection |
| P7-2B | CCIPCA Diagnostic | Zero-copy ArrayView, lazy-initialized per symbol, explained variance logging |
| P7-3A | Security | Removed hardcoded DATABASE_URL fallback |
| P7-3B | Payload Guard | 16MiB sqlx payload size guard before DB persist |
| P7-3D | Dead-Zone Tracing | debug! per-asset threshold/delta/triggered logging |
| P7-4A | Shadow Promotion Guard | 7-trading-day trigger before shadow→live promotion |
| P7-4C | Auto-Demotion | Consecutive underperformance tracking for live→shadow demotion |
| P7-5A | Factor Importance | Permutation importance — shuffle factor column, measure PSR drop |
| P7-5B | Genome Diversity | Per-ALPS-layer Hamming distance monitoring every 50 generations |

### New DB Migrations
- `038`: Shadow promotion guard (7-trading-day trigger)
- `039`: Auto-demotion logic (consecutive underperformance tracking)

---

## P8 — Semantic Prior + Active Reduction (DESIGNED)

**Design doc**: `docs/P8_ARCHITECTURE_DESIGN.md`
**Goal**: Transform blind MCTS to semantic search, activate CCIPCA feature reduction, close the diversity feedback loop.

### Five-Phase Architecture

| Phase | Name | Priority | Description |
|-------|------|----------|-------------|
| 0 | LLM-Guided MCTS Policy Prior | HIGHEST | Wire FactorImportance → LlmCachedPolicy; replace UniformPolicy |
| 1 | CCIPCA Active Token Remapping | HIGH | project_features() augments 75→80 features (5 PC columns) |
| 2 | ALPS Diversity-Triggered Injection | HIGH | L3/L4 Hamming diversity triggers emergency MCTS + Oracle |
| 3 | VM Hot Path Optimization | MEDIUM | Pre-execution shape guard, ndarray::Zip TS ops, conditional NaN sanitization |
| 4 | sqlx 0.8 Migration + Financial Precision | MEDIUM | RUSTSEC-2024-0363 fix, f64→Decimal in ensemble_weights/shadow paths |

### Gemini Fact-Check
- 4/5 recommendations valid; Actix-web claim debunked (all services use Axum)
- unsafe `uget()` deferred to P9 (no evidence of runtime panics)

---

## Dependency Graph

```
P0 (OOS eval) ──> P1 (factors) ──> P2 (LLM mutation) ──> P3 (multi-TF) ──> P4 (thresholds)
                       │                                                         │
                       └──> P5 (portfolio) ──> P6 (hardening+MCTS) ──> P7 (barriers) ──> P8 (semantic)
```

- **P2 depends on P1**: LLM oracle references the 25-factor vocabulary
- **P3 depends on P2**: 75-feature space requires guided search
- **P4 depends on P3**: Uses utilization feedback from walk-forward steps with MTF data
- **P5 depends on convergence**: Needs sufficient high-PSR strategies (10+)
- **P6 depends on P5**: Enhances evolution, portfolio, and adds MCTS engine
- **P7 depends on P6**: Wires MCTS into evolution loop, adds statistical barriers
- **P8 depends on P7**: Activates dead_code components (LlmCachedPolicy, FactorImportance, CCIPCA active)

---

## Completed Implementation Order

1. **P0**: Walk-forward OOS evaluation fix (commit `78b1c63`)
2. **P1**: Factor enrichment 13→25 (commit `63d5464`)
3. **P2**: LLM-guided mutation oracle, 5 sub-phases (commits `2e2e9fc`–`9d3a42b`)
4. **P3**: Multi-timeframe factor stacking (commit `a3de5fe`)
5. **P4**: Adaptive threshold tuning (commit `8e84677`)
6. **P5**: HRP portfolio ensemble with dynamic weights (commit `5fde72b`)
7. **P6**: Full-stack evolution upgrade — MCTS, lFDR, CCIPCA, shadow trading (commit `d1908e1`)
8. **P7**: Statistical barriers + MCTS integration (commit `a342a49`)

## Remaining
9. **P8**: Semantic prior + active reduction — LLM-guided MCTS, CCIPCA augmentation, diversity trigger, VM optimization, sqlx 0.8
