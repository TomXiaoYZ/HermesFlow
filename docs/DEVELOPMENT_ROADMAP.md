# HermesFlow Strategy Evolution — Development Roadmap

**Date**: 2026-02-20
**Current state**: P0 (OOS evaluation) and P1 (factor enrichment) deployed

---

## Stage Summary

| Stage | Name | Status | Goal |
|-------|------|--------|------|
| **P0** | Walk-Forward OOS Evaluation | Deployed | Fix OOS evaluation (35% → 94% success) |
| **P1** | Factor Enrichment (13 → 25) | Deployed | Reduce `too_few_trades` via richer signals |
| **P2** | LLM-Guided Mutation | Design complete | Accelerate convergence with domain-aware mutation |
| **P3** | Multi-Timeframe Factor Stacking | Planned | Add temporal depth (1h + 4h + 1d factors) |
| **P4** | Adaptive Threshold Tuning | Planned | Per-symbol threshold optimization |
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

## P2 — LLM-Guided Mutation (DESIGN COMPLETE)

**Design**: `docs/P2_ARCHITECTURE_DESIGN.md`
**Estimated files**: 2 new, 4 modified
**Estimated LoC**: ~500 new

### Prerequisites
- [ ] P1 TFT rate observation at 500 gens (validate need)
- [ ] LLM API access configured (Anthropic/OpenAI key in env)

### Implementation Plan

#### P2a: Genome Decoder (~100 LoC)
- [ ] `services/strategy-generator/src/genome_decoder.rs`
  - `token_to_name()` — token index → human name
  - `decode_genome()` — token array → RPN string
  - `encode_formula()` — RPN string → token array (with validation)
  - `validate_stack()` — verify stack correctness of a formula
- [ ] Unit tests: round-trip encode/decode, invalid formula rejection

#### P2b: LLM Oracle Core (~250 LoC)
- [ ] `services/strategy-generator/src/llm_oracle.rs`
  - `ElitePool` struct — collect top genomes across layers
  - `build_prompt()` — construct LLM prompt with elite formulas
  - `call_llm()` — HTTP request to LLM API (reqwest)
  - `parse_response()` — extract RPN formulas from JSON response
  - `generate_mutations()` — end-to-end: collect → prompt → parse → validate
- [ ] Add `reqwest` dependency to Cargo.toml
- [ ] Config struct in `config/generator.yaml`

#### P2c: Trigger Integration (~100 LoC)
- [ ] `services/strategy-generator/src/main.rs`
  - Trigger detection using `PromotionRateTracker`
  - Cooldown management per (symbol, mode)
  - Genome injection into AlpsGA Layer 0
- [ ] `services/strategy-generator/src/genetic.rs`
  - `inject_genomes(layer: usize, genomes: Vec<Genome>)` method
- [ ] Feature flag: `llm_oracle.enabled = false` by default

#### P2d: Monitoring (~50 LoC)
- [ ] Track in metadata: `llm_injection_count`, `llm_genome_survival_rate`, `llm_trigger_reason`
- [ ] Log: invocation count, parse success rate, genome validation rate
- [ ] Report: LLM genome fitness distribution vs random genome fitness

### Success Criteria
- TFT rate < 8% at 500 gens (with LLM)
- LLM genome survival to L1 > 30%
- No evolution regression (promotion rates maintained > 85%)

---

## P3 — Multi-Timeframe Factor Stacking (PLANNED)

**Goal**: Add temporal depth by computing factors at multiple resolutions (1h, 4h, 1d) and stacking them as separate features. Current: single 1h resolution.

### Rationale
- Short-term (1h): Captures intraday momentum, mean reversion
- Medium-term (4h): Smoothed trends, noise reduction
- Long-term (1d): Regime context, structural shifts
- Combined: Strategies can reference "is 1h momentum up while 1d trend is down" = mean reversion signal

### Design Sketch

#### Factor Expansion
- Current: 25 factors × 1 resolution = 25 features
- Proposed: 25 factors × 3 resolutions = 75 features
- `feat_offset` increases from 25 to 75
- Token space expands but operator set stays at 14

#### Implementation
1. **Candle resampling**: Resample 1h candles to 4h and 1d in `backtest/mod.rs`
2. **Factor computation**: Compute all 25 factors for each resolution
3. **Feature stacking**: Stack `[1h_factors, 4h_factors, 1d_factors]` along feature axis
4. **Genome compatibility**: Another `feat_offset` migration (25 → 75)

#### Risks
- **Search space explosion**: 75 features × 14 operators is much harder to search
- **Mitigation**: P2 LLM oracle becomes essential for guided exploration
- **Compute cost**: 3× factor computation per genome evaluation
- **Mitigation**: Cache resampled candles, factor arrays

#### Estimated Effort
- 3 files modified, ~300 LoC
- Requires P2 to be effective (blind search in 75-feature space is slow)

---

## P4 — Adaptive Threshold Tuning (PLANNED)

**Goal**: Optimize the VM signal thresholds (upper/lower) per symbol instead of using fixed values.

### Rationale
- Current: Fixed threshold pairs (e.g., `(0.0, 0.8)`, `(0.48, 0.8)`) define when the VM signal triggers buy/sell
- Different symbols have different signal distributions
- TSLA signals might be naturally higher-variance than DIA
- Optimal thresholds vary by symbol volatility and signal characteristics

### Design Sketch

#### Approach A: Encode thresholds in genome
- Add 2 tokens to each genome encoding the threshold values
- Let evolution optimize thresholds alongside the formula
- Pro: No extra infrastructure
- Con: Increases search space

#### Approach B: Post-hoc optimization
- After finding a promising formula, do a grid search over threshold pairs
- Only applied to genomes that survive to L2+
- Pro: Doesn't affect search space
- Con: Extra compute per elite evaluation

#### Approach C: Bayesian optimization
- Use Gaussian Process to model PSR as f(upper, lower) for a given genome
- Sample efficiently in threshold space
- Pro: Most sample-efficient
- Con: Complexity, external dependency

### Estimated Effort
- Approach B (recommended): ~200 LoC, 2 files modified
- Requires stable elite genomes (P1 convergence + P2 acceleration)

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
P0 (OOS eval) ──> P1 (factors) ──> P2 (LLM mutation) ──> P3 (multi-TF)
                       │                    │
                       │                    └──> P4 (thresholds)
                       │
                       └──> P5 (portfolio) ──> P6 (paper trading)
```

- **P2 depends on P1**: LLM oracle references the 25-factor vocabulary
- **P3 depends on P2**: 75-feature space requires guided search
- **P4 depends on P2**: Needs stable elite genomes to optimize thresholds against
- **P5 depends on P1+convergence**: Needs sufficient high-PSR strategies
- **P6 depends on P5**: Needs portfolio allocation for position sizing

---

## Recommended Implementation Order

1. **Now**: Let P1 run, collect data. Re-check TFT rates at 500 gens.
2. **P2a+P2b**: Implement genome decoder and LLM oracle core (can start immediately, no data dependency)
3. **P2c+P2d**: Wire triggers and monitoring (after 500 gens data validates the need)
4. **P4**: Adaptive thresholds (quick win once elites stabilize)
5. **P3**: Multi-timeframe stacking (major expansion, needs P2 working)
6. **P5**: Portfolio optimization (once we have 10+ high-PSR strategies)
7. **P6**: Paper trading (final validation step before live)
