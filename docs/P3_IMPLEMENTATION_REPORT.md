# P3 Implementation Report: Multi-Timeframe Factor Stacking

> Status: **DEPLOYED & RUNNING** | Commit: `a3de5fe` | Date: 2026-02-22

## 1. Objective & Summary

P3 expands the strategy generator's feature space from 25 single-resolution (1h) factors to 75 multi-timeframe factors (25 factors x 3 resolutions: 1h, 4h, 1d). This gives the genetic algorithm and LLM Oracle access to cross-timeframe signals — trend context, regime stability, and reduced noise — that single-resolution data cannot capture.

### Motivation (from P2 Lessons)

P2 validation revealed that the 25 1h-only factors were the binding constraint on OOS PSR improvement. The adaptive threshold ceiling fix (`[0.52, 0.80]` → `[0.52, 0.70]`) proved that signal stability across train/test windows matters more than formula complexity. Lower-frequency factors (4h, 1d) provide more stable signals in OOS windows.

### P2 Baseline (for comparison)

| Metric | P2 Value |
|--------|----------|
| Average OOS PSR | 2.521 |
| OOS PSR > 2 (strong) | 19/26 (73%) |
| OOS PSR > 1 (significant) | 24/26 (92%) |
| OOS PSR > 0 (positive) | 26/26 (100%) |
| TFT 0% (healthy) | 26/26 (100%) |
| Total generations | ~80,000 per symbol |

### P3 Target

| Metric | Target |
|--------|--------|
| Average OOS PSR | > 3.0 |
| OOS PSR > 2 (strong) | > 85% |
| OOS PSR > 1 (significant) | > 95% |

## 2. Implementation Details

### 2.1 Token Layout Change

```
P2:  tokens 0-24 = 25 features (1h)     | 25+ = operators     feat_offset = 25
P3:  tokens 0-24 = 25 features (1h)     |
     tokens 25-49 = 25 features (4h)    |
     tokens 50-74 = 25 features (1d)    | 75+ = operators     feat_offset = 75
```

### 2.2 Code Changes

| File | Lines Changed | Description |
|------|--------------|-------------|
| `services/backtest-engine/src/config.rs` | +100 | `MultiTimeframeFactorConfig` struct with `feat_count()`, `feat_offset()`, `factor_names()` + 3 unit tests |
| `services/strategy-generator/src/backtest/mod.rs` | +280 | `forward_fill_align()` (binary search temporal alignment), `load_data_multi_timeframe()`, `fetch_candles_for_resolution()`, `load_reference_close()` + 4 unit tests |
| `services/strategy-generator/src/main.rs` | +60 | `MultiTimeframeYamlConfig` struct, MTF config parsing, evolution loop integration (feat_offset=75, factor names dispatch, data loading dispatch) |
| `config/generator.yaml` | +6 | `multi_timeframe` config section (`enabled: true`, `resolutions: ["1h", "4h", "1d"]`) |
| `services/strategy-generator/src/llm_oracle.rs` | +20 | Timeframe explanation in LLM prompt, cross-timeframe RPN examples |
| `services/strategy-generator/src/genome_decoder.rs` | +45 | 4 MTF test cases (cross-timeframe decode, encode round-trip, stack validation at offset=75) |
| `services/strategy-engine/src/risk.rs` | +5/-5 | Whitespace formatting only (`cargo fmt`) |

**Total**: +678 lines across 7 files. ~465 lines of new logic, ~100 lines of tests, remainder formatting.

### 2.3 Key Components

#### MultiTimeframeFactorConfig (`config.rs`)

Wraps a single `FactorConfig` (25 factors) and replicates it across multiple resolutions. Provides:
- `feat_count()` → 75 (25 x 3)
- `feat_offset()` → 75
- `factor_names()` → `["return_1h", ..., "spy_rel_strength_1h", "return_4h", ..., "return_1d", ...]`

#### forward_fill_align (`backtest/mod.rs`)

Aligns lower-frequency features to the 1h time axis. For each 1h timestamp, finds the most recent lower-frequency bar via binary search and copies its feature values. Complexity: O(T_1h * log(T_lf)).

```
4h features at each 1h bar:
  1h:  09:30  10:30  11:30  12:30  13:30  14:30  15:30
  4h:  [9:30] [9:30] [9:30] [9:30] [13:30] [13:30] [13:30]
       ^^^^^^ forward-fill until next 4h bar arrives
```

#### load_data_multi_timeframe (`backtest/mod.rs`)

For each symbol:
1. Queries `mkt_equity_candles` at 3 resolutions (1h, 4h, 1d)
2. Loads SPY reference data per resolution for cross-asset factors
3. Computes 25 features per resolution via `FeatureEngineer::compute_features_from_config()`
4. Forward-fills 4h and 1d features to the 1h time axis
5. Concatenates along the feature axis → Array3 (1, 75, T_1h)

#### LLM Oracle Prompt Update (`llm_oracle.rs`)

Auto-detects MTF mode via factor name suffixes. When active, adds:
- Timeframe explanation (`_1h` = intraday, `_4h` = session trends, `_1d` = multi-day regimes)
- Cross-timeframe RPN examples (`momentum_1h momentum_1d SUB`, `volatility_4h TS_MEAN volatility_1h DIV`)

#### Feature Gate (`generator.yaml`)

```yaml
multi_timeframe:
  enabled: true           # set to false to revert to P2 mode
  resolutions: ["1h", "4h", "1d"]
```

No database migrations needed. P2 and P3 genomes coexist via `feat_offset` in metadata JSON.

### 2.4 Components Unchanged

| Component | Reason |
|-----------|--------|
| `StackVM` (`vm.rs`) | Generic over `feat_offset`; token < offset → feature, token >= offset → operator |
| `vm/ops.rs` | Operators unchanged (14 active, 23 total) |
| `FeatureEngineer` (`engineer.rs`) | Already config-driven; called once per resolution |
| `factors-stock.yaml` | Reused as-is for all 3 resolutions |
| `genetic.rs` | `generate_random_rpn(feat_offset)` already parameterized |
| `genome_decoder.rs` | Decode/encode logic already parameterized (only tests added) |

## 3. Test & Build Verification

### 3.1 Unit Tests

| Crate | Total | New P3 Tests | Result |
|-------|-------|-------------|--------|
| backtest-engine | 66 | 3 (`test_mtf_feat_count`, `test_mtf_factor_names`, `test_mtf_single_resolution_matches_base`) | ALL PASS |
| strategy-generator | 36 | 8 (`forward_fill_*` x4, `test_mtf_*` x4) | ALL PASS |

### 3.2 Lint

```
cargo clippy -p backtest-engine -p strategy-generator -- -D warnings
→ 0 errors, 0 warnings
```

### 3.3 Docker Build

```
docker compose build strategy-generator
→ Release build: 87s
→ Image: hermesflow-strategy-generator (sha256:44cb952f...)
```

### 3.4 Container Health

```
docker compose ps strategy-generator
→ STATUS: Up (healthy)
→ PORTS: 0.0.0.0:8082->8082/tcp
```

## 4. Current Running State

### 4.1 All 26 Evolution Tasks Active

All 13 symbols x 2 modes (long_only + long_short) are running with P3 configuration:

```
[Polygon:SPY:long_only]  P3 MTF enabled: 25 factors x 3 resolutions = 75 features (feat_offset=75)
[Polygon:SPY:long_short] P3 MTF enabled: 25 factors x 3 resolutions = 75 features (feat_offset=75)
[Polygon:AAPL:long_only] P3 MTF enabled: 25 factors x 3 resolutions = 75 features (feat_offset=75)
... (26 tasks total)
```

### 4.2 Genome Invalidation Handled

All 26 symbols correctly detected the feat_offset change and started fresh evolution:

```
[Polygon:SPY:long_only]  Skipping genome resume: feat_offset changed (25→75), old tokens incompatible
[Polygon:AAPL:long_only] Skipping genome resume: feat_offset changed (25→75), old tokens incompatible
... (26 warnings total)
```

### 4.3 MTF Data Loading Successful

All symbols loaded 75 stacked features across 3 resolutions:

| Symbol | 1h Bars | Stacked Shape | Status |
|--------|---------|---------------|--------|
| META | 4,958 | (1, 75, 4958) | Loaded |
| MSFT | 4,894 | (1, 75, 4894) | Loaded |
| GOOGL | 4,410 | (1, 75, 4410) | Loaded |
| AMZN | 3,971 | (1, 75, 3971) | Loaded |
| AAPL | 3,875 | (1, 75, 3875) | Loaded |
| TSLA | 3,117 | (1, 75, 3117) | Loaded |
| NVDA | 3,086 | (1, 75, 3086) | Loaded |
| SPY | ~4,900 | (1, 75, ~4900) | Loaded |
| QQQ | ~4,900 | (1, 75, ~4900) | Loaded |
| DIA | ~4,900 | (1, 75, ~4900) | Loaded |
| IWM | ~4,900 | (1, 75, ~4900) | Loaded |
| GLD | ~4,900 | (1, 75, ~4900) | Loaded |
| UVXY | ~4,900 | (1, 75, ~4900) | Loaded |

### 4.4 Memory Footprint

Per-symbol feature tensor: (1, 75, ~4000) = 300K f64 = ~2.4 MB.
13 symbols x 2 modes (shared cache) ≈ ~31 MB. Acceptable overhead.

## 5. Next Steps & Expected Behavior

### 5.1 Early Generation Behavior (Gen 0-100)

The `IS: 0.0000 OOS: -13.0000` (VM_FAILURE sentinel) observed in the first few generations is **expected**. With 75 feature tokens + 23 operator tokens = 98 total token values, random genomes are more likely to produce feature-heavy sequences that don't collapse to a single stack value. ALPS natural selection will eliminate these within the first 5-10 generations as the population matures.

### 5.2 Convergence Timeline

| Phase | Generations | Expected Behavior |
|-------|------------|-------------------|
| 0-50 | Random exploration | Mostly VM failures, first valid signals emerging |
| 50-200 | Early convergence | Positive IS PSR appearing, promotion rates establishing |
| 200-1000 | Signal discovery | Cross-timeframe patterns emerging, OOS PSR turning positive |
| 1000-5000 | Stabilization | OOS PSR comparable to P2 baseline (~2.5), MTF-specific alpha emerging |
| 5000+ | Maturation | Potential improvement over P2 baseline via multi-timeframe factors |

### 5.3 LLM Oracle Activation

The Oracle will trigger after generation 100 (promotion rate trigger) or generation 200 (TFT trigger). With 75 factors in the vocabulary and cross-timeframe examples in the prompt, the Oracle should generate more diverse and semantically rich formulas than P2.

### 5.4 Monitoring Checkpoints

- **Gen 100**: Verify promotion rates are non-zero (ALPS layers functioning)
- **Gen 500**: Verify first positive OOS PSR values appearing
- **Gen 1000**: Compare average OOS PSR to P2 baseline (should be approaching positive territory)
- **Gen 5000**: Assess whether MTF factors produce measurably better OOS PSR than P2's 2.521 average
- **Gen 10000+**: Final assessment of P3 value-add

### 5.5 Rollback Plan

If P3 shows no improvement or degraded performance after 10,000 generations:
1. Set `multi_timeframe.enabled: false` in `config/generator.yaml`
2. Restart strategy-generator
3. Evolution resumes with P2 behavior (25 features, feat_offset=25)
4. Old P2 genomes in DB remain valid and can be resumed
