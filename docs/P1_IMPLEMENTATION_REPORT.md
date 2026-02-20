# P1 Implementation Report: Factor Enrichment — 13 → 25 Factors

**Date**: 2026-02-20
**Commit**: `63d5464` (`feat: P1 factor enrichment — expand from 13 to 25 active factors`)
**Status**: Deployed & Verified

---

## 1. Summary

P1 expands the strategy-generator's factor set from 13 to 25 active factors for Polygon (US equity) evolution. The goal is to reduce `too_few_trades` OOS failures by giving the VM richer, more orthogonal raw material — including microstructure signals and cross-asset (SPY) reference factors.

All 25 factors compute without NaN/Inf across all 13 symbols. Docker deployment verified healthy. Genome compatibility handling correctly detects and skips old 13-factor genomes.

---

## 2. Factors Added (12 new, IDs 13-24)

### Tier A — Technical Enrichment (6 existing implementations, newly activated)

| ID | Name | Category | What it adds | Normalization |
|----|------|----------|-------------|---------------|
| 13 | `atr_pct` | Volatility | True Range as % of close (captures gaps) | robust |
| 14 | `obv_pct` | Volume | On-Balance Volume % change (directional volume) | robust |
| 15 | `mfi` | Volume | Money Flow Index (price-weighted volume momentum, 0-1) | none |
| 16 | `bb_percent_b` | Price level | Bollinger Bands %B (position within bands, 0-1) | none |
| 17 | `macd_hist` | Momentum | MACD histogram / close (momentum acceleration) | robust |
| 18 | `sma_200_diff` | Trend | (close - SMA200) / close (long-term trend context) | robust |

### Tier A — Microstructure (3 new implementations with unit tests)

| ID | Name | Category | Formula | Normalization |
|----|------|----------|---------|---------------|
| 19 | `amihud_illiq` | Microstructure | rolling_mean(\|return\| / dollar_volume, 20) | robust |
| 20 | `spread_proxy` | Microstructure | (high - low) / midprice, normalized by rolling mean | robust |
| 21 | `return_autocorr` | Microstructure | rolling_corr(ret[t], ret[t-1], 20), range [-1, 1] | none |

### Tier B — Cross-Asset SPY Reference (3 new implementations)

| ID | Name | Category | Formula | Normalization |
|----|------|----------|---------|---------------|
| 22 | `spy_corr` | Cross-asset | rolling_corr(ret, spy_ret, 60), range [-1, 1] | none |
| 23 | `spy_beta` | Cross-asset | cov(ret, spy_ret, 60) / var(spy_ret, 60) | robust |
| 24 | `spy_rel_strength` | Cross-asset | cumret(symbol, 20) - cumret(SPY, 20) | robust |

**Orthogonality by category**: Returns (1), Volume (3), Price level (2), Volatility (2), Momentum (4), Trend (1), Microstructure (3), Cross-asset (3), Regime (3), Range (1), Position (1), Strength (1).

---

## 3. Files Modified

| File | Lines Changed | Description |
|------|:---:|-------------|
| `services/backtest-engine/src/factors/indicators.rs` | +271 | 6 new methods (3 microstructure + 3 cross-asset) + 7 unit tests |
| `services/backtest-engine/src/factors/engineer.rs` | +33 | 12 new match arms in `compute_single_factor()` |
| `services/backtest-engine/src/factors/traits.rs` | +4 | `ref_close: Option<&Array2<f64>>` added to `OhlcvData` |
| `services/backtest-engine/src/factors/dynamic.rs` | +1 | Updated test OhlcvData construction |
| `services/backtest-engine/tests/parity_check.rs` | +1 | Updated test OhlcvData construction |
| `services/strategy-generator/src/backtest/mod.rs` | +57 | `ref_cache`, `load_reference_data()`, SPY alignment |
| `services/strategy-generator/src/main.rs` | +56/-11 | SPY loading, genome compat, `feat_offset` in metadata |
| `config/factors-stock.yaml` | +68/-3 | 12 new factor entries (IDs 13-24) |
| **Total** | **+480/-11** | |

**NOT modified**: `genetic.rs` (auto-adapts via `feat_offset`), walk-forward/OOS eval, DB schema, VM opcodes (all 23 retained for backward compatibility).

---

## 4. Key Design Decisions

### 4a. Cross-Asset Graceful Degradation
When `ref_close` is `None` (non-Polygon exchanges, or SPY load failure), cross-asset factors return `Array2::zeros()` — effectively neutral. This avoids conditional factor config per exchange while keeping the VM token space uniform.

### 4b. Genome Compatibility
Old genomes (13-factor `feat_offset=13`) encode operators starting at token 13. New genomes (25-factor `feat_offset=25`) encode operators starting at token 25. Tokens 13-24 would be misinterpreted as features instead of operators.

**Solution**:
- `feat_offset` is now persisted in generation metadata
- On resume, if stored `feat_offset` differs from current (or is absent), the genome is skipped
- Generation counter is preserved (e.g., resume at gen 51816), only genome tokens are discarded
- ALPS GA generates fresh random genomes in the 25-factor token space

### 4c. SPY Reference Data Loading
- `load_reference_data("SPY", lookback_days)` queries `mkt_equity_candles` for SPY
- Close prices stored in `ref_cache: HashMap<String, Array2<f64>>`
- Alignment: SPY array truncated from front to match each symbol's bar count
- Only loaded for Polygon exchange, and only for non-SPY symbols

---

## 5. Verification Results

### 5a. Build & Lint
```
cargo clippy --workspace -- -D warnings  ✅ Clean (0 warnings)
cargo fmt --all                           ✅ Applied
cargo test --workspace                    ✅ 202 tests passed, 0 failed
```

### 5b. New Unit Tests (7 added)
| Test | Result | What it validates |
|------|--------|------------------|
| `test_amihud_illiquidity` | ✅ | Non-negative, finite, higher volume → lower illiquidity |
| `test_spread_proxy` | ✅ | Non-negative, finite output |
| `test_return_autocorrelation` | ✅ | Trending → positive, alternating → low/negative |
| `test_rolling_correlation` | ✅ | Self-correlation → ~1.0 |
| `test_rolling_beta` | ✅ | Self-beta → ~1.0 |
| `test_relative_strength_vs` | ✅ | Outperformer → positive relative strength |
| (parity_check) | ✅ | Existing 13-factor parity still holds |

### 5c. Docker Deployment
```
Docker build:                             ✅ strategy-generator image built
Service status:                           ✅ healthy
Factor config loaded:                     ✅ "Loaded 25 factors from config/factors-stock.yaml" (×27 tasks)
SPY reference loaded:                     ✅ "Loaded 4049 bars of reference data for SPY" (×24 non-SPY tasks)
Genome compatibility:                     ✅ "Skipping genome resume: no feat_offset in metadata" (all symbols)
New generations persisted:                ✅ feat_offset=25 confirmed in DB metadata
No errors:                                ✅ Zero "Unknown factor", NaN, Inf, panic, or ERROR logs
```

### 5d. Live Evolution Log Sample
```
[Polygon:NVDA:long_only]  Gen 51825 IS: 1.7135 OOS: 1.4406 wf_steps: 2/2  ← Strong OOS
[Polygon:GLD:long_only]   Gen 51703 IS: 1.0131 OOS: 1.7618 wf_steps: 3/3  ← OOS > IS
[Polygon:TSLA:long_short] Gen 51713 IS: 1.3234 OOS: 1.5920 wf_steps: 2/2  ← Strong OOS
[Polygon:SPY:long_short]  Gen 51597 IS: 0.4702 OOS: 1.1127 wf_steps: 3/3  ← OOS > IS
[Polygon:AAPL:long_only]  Gen 51725 IS: 0.6929 OOS: 0.3409 wf_steps: 3/3  ← Moderate
[Polygon:META:long_only]  Gen 51581 IS: 1.2339 OOS: -15.00 wf_steps: 0/3  ← too_few_trades sentinel
```

### 5e. Observed `too_few_trades` at Deployment Time (Snapshot)
Note: These are early-generation results in the 25-factor space. The ALPS GA has only run ~1-10 new generations since restarting with `feat_offset=25`. Meaningful comparison vs 13-factor baseline requires ~50+ generations.

| Symbol | Mode | `too_few_trades` in WF? | Notes |
|--------|------|:-:|-------|
| META | long_only | 3/3 steps | Persistent problem symbol |
| META | long_short | 2/3 steps | Partial |
| IWM | long_short | 3/3 steps | |
| AMZN | long_short | 3/3 steps | |
| GOOGL | long_short | 2/3 steps | Partial |
| Others | * | 0/3 steps | Clean OOS evaluation |

---

## 6. What's Next (Recommended for Gemini Review)

1. **Baseline Comparison**: After ~50 generations in the 25-factor space, compare `too_few_trades` rate against the 13-factor baseline. Expected: reduction in `too_few_trades` from the richer signal space.

2. **Cross-Asset Factor Variance**: Monitor whether `spy_corr`, `spy_beta`, `spy_rel_strength` show non-trivial variance vs constant zeros. Current: SPY data loaded (4049 bars), factors are computing — full variance analysis after sufficient evolution.

3. **OOS PSR Distribution**: Compare the 25-factor OOS PSR distribution against 13-factor (from P0 report). Expected: broader signal diversity should improve both trade frequency and OOS stability.

4. **Factor Utilization Analysis**: After ~100 generations, analyze which of the 12 new factors appear most frequently in elite genomes. Candidates for pruning: factors that never appear in top-fitness genomes.

5. **Potential P2 Candidates** (from Gemini discussion):
   - Meta-layer / LLM-guided mutation
   - Adaptive factor weighting
   - Multi-timeframe factor stacking
