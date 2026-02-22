# P3 Architecture Design: Multi-Timeframe Factor Stacking

> Status: **DESIGN** | Prerequisite: P2 (LLM Oracle) deployed and validated

## 1. Objective

Expand the strategy generator's feature space from 25 single-resolution (1h) factors to 75 multi-timeframe factors (25 factors x 3 resolutions: 1h, 4h, 1d). This gives the genetic algorithm and LLM Oracle access to cross-timeframe signals that single-resolution data cannot capture.

### Motivation

P2 validation revealed that the 25 1h-only factors are the binding constraint on OOS PSR improvement. The adaptive threshold ceiling fix (P2 Section 3) proved that signal stability across train/test windows matters more than formula complexity. Lower-frequency factors (4h, 1d) provide:

- **Trend context**: 1d SMA-200 deviation captures long-term trend that 1h bars alone cannot represent
- **Regime stability**: 4h volatility regime changes less frequently than 1h, producing more stable signals in OOS windows
- **Reduced noise**: Daily volume ratio is less noisy than hourly, giving the GA cleaner building blocks
- **Cross-timeframe alpha**: Formulas like `momentum_1h momentum_1d SUB` (intraday vs daily momentum divergence) are a well-known alpha source in quantitative finance

### Success Criteria

| Metric | P2 Baseline | P3 Target |
|--------|-------------|-----------|
| Average OOS PSR | 2.521 | > 3.0 |
| OOS PSR > 2 (strong) | 19/26 (73%) | > 85% |
| OOS PSR > 1 (significant) | 24/26 (92%) | > 95% |
| Symbols with fresh evolution needed | 0 (all restarted) | 26 (all restart) |

## 2. Current Architecture (P2 Baseline)

### Data Flow

```
mkt_equity_candles (1h)
  │
  └─ load_data() ─→ Array2<f64> (OHLCV, 1 x time)
       │
       └─ FeatureEngineer::compute_features_from_config()
            │
            └─ Array3<f64> (1, 25, time_1h)
                 │
                 └─ StackVM::execute(tokens, features)
                      │  feat_offset = 25
                      │  tokens 0-24 → feature slice
                      │  tokens 25+  → operators
                      │
                      └─ Array2<f64> signal → PSR fitness
```

### Key Components

| Component | File | Role |
|-----------|------|------|
| `FactorConfig` | `backtest-engine/src/config.rs:22-40` | Loads YAML, provides `feat_count()` and `feat_offset()` |
| `FeatureEngineer` | `backtest-engine/src/factors/engineer.rs:24-45` | Computes Array3 features from OHLCV + config |
| `StackVM` | `backtest-engine/src/vm/vm.rs:6-305` | RPN execution, `feat_offset` separates features from ops |
| `Backtester` | `strategy-generator/src/backtest/mod.rs:174-396` | Data loading, caching, evaluation |
| `genome_decoder` | `strategy-generator/src/genome_decoder.rs` | Bidirectional RPN codec (tokens <-> formula strings) |
| `llm_oracle` | `strategy-generator/src/llm_oracle.rs` | LLM prompt building with factor vocabulary |
| `main.rs` | `strategy-generator/src/main.rs` | Evolution loop, Oracle trigger, data orchestration |
| `genetic.rs` | `strategy-generator/src/genetic.rs` | ALPS GA, random genome generation |
| `factors-stock.yaml` | `config/factors-stock.yaml` | 25 factor definitions (id, name, normalization) |
| `generator.yaml` | `config/generator.yaml` | Exchange config (resolution, lookback, factor_config path) |

### Current Token Layout

```
Token:     0  1  2  ...  24  │  25  26  27  ...  47
Meaning:  ─── 25 features ──│── 23 operators (14 active) ──
          feat_offset = 25
```

## 3. P3 Architecture Design

### 3.1 Target Token Layout

```
Token:    0────24  25────49  50────74  │  75  76  77  ...  97
Meaning:  ─1h──── ──4h───── ──1d───── │ ── 23 operators (14 active) ──
          feat_offset = 75
```

- Tokens 0-24: 1h factors (same 25 as P2)
- Tokens 25-49: 4h factors (same 25 factor definitions, computed on 4h OHLCV)
- Tokens 50-74: 1d factors (same 25 factor definitions, computed on 1d OHLCV)
- Tokens 75+: Operators (unchanged)

### 3.2 Target Data Flow

```
mkt_equity_candles
  │
  ├─ load_resolution("1h") ─→ OHLCV_1h (1 x T_1h bars)
  ├─ load_resolution("4h") ─→ OHLCV_4h (1 x T_4h bars)
  └─ load_resolution("1d") ─→ OHLCV_1d (1 x T_1d bars)
       │
       ├─ compute_features(OHLCV_1h) ─→ Array3 (1, 25, T_1h)
       ├─ compute_features(OHLCV_4h) ─→ Array3 (1, 25, T_4h)
       │     └─ forward_fill_align(T_4h → T_1h)  ─→ Array3 (1, 25, T_1h)
       └─ compute_features(OHLCV_1d) ─→ Array3 (1, 25, T_1d)
             └─ forward_fill_align(T_1d → T_1h)  ─→ Array3 (1, 25, T_1h)
       │
       └─ concatenate(axis=1) ─→ Array3 (1, 75, T_1h)
            │
            └─ StackVM::execute(tokens, features)
                 feat_offset = 75
```

### 3.3 Temporal Alignment Strategy

The base resolution is **1h** (finest granularity). Lower-frequency features are forward-filled to the 1h time axis.

**Alignment rules:**
- For each 1h timestamp `t`, find the most recent 4h bar where `bar.time <= t`
- For each 1h timestamp `t`, find the most recent 1d bar where `bar.time <= t`
- Forward-fill: each lower-frequency bar's feature values persist until the next bar arrives

**Example (Polygon, trading hours only):**

```
1h bars:  09:30  10:30  11:30  12:30  13:30  14:30  15:30
4h bars:  09:30                       13:30
1d bars:  09:30

4h features at each 1h bar:
          [9:30] [9:30] [9:30] [9:30] [13:30] [13:30] [13:30]
          ^^^^^^ forward-fill from 09:30 bar until 13:30 bar arrives

1d features at each 1h bar:
          [9:30] [9:30] [9:30] [9:30] [9:30]  [9:30]  [9:30]
          ^^^^^^ all 1h bars within same day use same 1d features
```

**Implementation approach:**

Rather than timestamp matching (fragile with market hours, holidays, etc.), use a simpler index-based approach:

1. Compute features for each resolution independently on their native OHLCV data
2. For 4h→1h alignment: each 4h bar maps to approximately 4 consecutive 1h bars. Use timestamp lookup with binary search to find the mapping.
3. For 1d→1h alignment: each 1d bar maps to approximately 6.5 1h bars (Polygon) or 24 1h bars (crypto).

The alignment function:

```rust
/// Align lower-frequency features to the 1h time axis via forward-fill.
///
/// For each 1h timestamp, finds the most recent lower-freq bar and copies
/// its feature values. Bars before the first lower-freq timestamp get zeros.
fn forward_fill_align(
    lf_features: &Array3<f64>,    // (1, n_factors, T_lf)
    lf_timestamps: &[i64],        // T_lf timestamps
    hf_timestamps: &[i64],        // T_1h timestamps (target axis)
) -> Array3<f64>                  // (1, n_factors, T_1h)
```

### 3.4 Config Design

**Option chosen: Single YAML with resolution prefixes in the Backtester, not in the YAML.**

The same `factors-stock.yaml` (25 factors) is used for all three resolutions. The Backtester loads it once and applies it three times (once per resolution). This avoids duplicating 75 factor definitions and keeps the YAML simple.

The `generator.yaml` changes to declare multi-timeframe mode:

```yaml
exchanges:
  - exchange: Polygon
    resolution: "1h"          # base resolution (unchanged)
    lookback_days: 730
    factor_config: "config/factors-stock.yaml"
    multi_timeframe:           # NEW: P3 multi-timeframe config
      enabled: true
      resolutions: ["1h", "4h", "1d"]
    walk_forward:
      initial_train: 2500
      target_test_window: 1000
      min_test_window: 400
      target_steps: 3
```

When `multi_timeframe.enabled = false` (or absent), behavior is identical to P2.

### 3.5 Factor Naming Convention

For the LLM Oracle prompt and genome decoder, factors get timeframe suffixes:

| Token Range | Name Pattern | Example |
|-------------|-------------|---------|
| 0-24 | `{factor_name}_1h` | `return_1h`, `momentum_1h`, `spy_corr_1h` |
| 25-49 | `{factor_name}_4h` | `return_4h`, `momentum_4h`, `spy_corr_4h` |
| 50-74 | `{factor_name}_1d` | `return_1d`, `momentum_1d`, `spy_corr_1d` |

This makes LLM prompts self-documenting:
```
"return_1h return_1d SUB ABS"  → |hourly_return - daily_return|
"momentum_4h TS_MEAN momentum_1h SUB"  → 4h_trend - 1h_momentum (mean reversion)
```

### 3.6 StackVM Changes

The StackVM itself requires **zero code changes**. It already operates generically:
- Token < `feat_offset` → push `features[axis(1), token]`
- Token >= `feat_offset` → execute operator

Changing `feat_offset` from 25 to 75 is the only thing needed. The VM doesn't know or care about timeframes — it just indexes into the feature tensor.

### 3.7 TS Window Handling

Time-series operators (TS_MEAN, TS_STD, etc.) use a rolling window. Currently:
- 1h data → `ts_window = 10` (10 hours)
- 1d data → `ts_window = 20` (1 trading month)

In P3, the VM operates on 1h-aligned data, so `ts_window = 10` applies to ALL features. This means:
- TS_MEAN on a 1h factor = 10-hour rolling mean (same as P2)
- TS_MEAN on a 4h factor (forward-filled to 1h) = effectively ~2.5 bars of 4h data (since each 4h value repeats ~4 times)
- TS_MEAN on a 1d factor (forward-filled to 1h) = effectively ~1.5 days of 1d data

This is **intentional and desirable**: the VM discovers the appropriate temporal smoothing for each feature through evolution. A formula that needs long-term 1d smoothing will chain TS_MEAN operators.

### 3.8 Genome Invalidation

**All existing P2 genomes are incompatible with P3** because `feat_offset` changes from 25 to 75:

| P2 Token | P2 Meaning | P3 Meaning |
|----------|-----------|------------|
| 0 | return (1h) | return_1h (same) |
| 25 | ADD operator | momentum_4h (WRONG!) |
| 30 | SIGN operator | volatility_4h (WRONG!) |

The resume check in `main.rs` (lines 555-587) already handles this: if `stored_feat_offset != current_feat_offset`, it skips resume and starts fresh evolution. No migration is needed.

### 3.9 SPY Reference Data

Cross-asset factors (`spy_corr`, `spy_beta`, `spy_rel_strength`) need SPY data at each resolution. The current `load_reference_data()` loads SPY at the base resolution only.

P3 approach: load SPY at all 3 resolutions. Each resolution's feature computation uses its matching SPY data.

## 4. Implementation Plan

### Phase 1: Config & Data Loading

#### 4.1 Add MultiTimeframeConfig to generator.yaml

Add optional `multi_timeframe` section to `ExchangeConfig`:

```rust
// main.rs
#[derive(Debug, Clone, Deserialize)]
pub struct MultiTimeframeConfig {
    pub enabled: bool,
    pub resolutions: Vec<String>,  // e.g. ["1h", "4h", "1d"]
}

// In ExchangeConfig:
pub multi_timeframe: Option<MultiTimeframeConfig>,
```

**File**: `services/strategy-generator/src/main.rs`

#### 4.2 Expand Backtester for Multi-Resolution Loading

Add `load_data_multi_timeframe()` to Backtester:

```rust
/// Load candle data at multiple resolutions and stack features.
///
/// 1. For each resolution, query mkt_equity_candles
/// 2. Compute 25 features per resolution
/// 3. Forward-fill lower-freq features to 1h time axis
/// 4. Concatenate → Array3 (1, 75, T_1h)
pub async fn load_data_multi_timeframe(
    &mut self,
    symbols: &[String],
    days: i64,
    resolutions: &[String],  // ["1h", "4h", "1d"]
) -> anyhow::Result<()>
```

Key steps inside:
1. Query `mkt_equity_candles` for each (symbol, resolution) pair
2. Build `OhlcvData` structs per resolution (with matching SPY ref data)
3. Call `FeatureEngineer::compute_features_from_config()` per resolution → 3 x Array3 (1, 25, T_res)
4. Align 4h and 1d features to 1h timestamps via `forward_fill_align()`
5. Concatenate along axis(1) → Array3 (1, 75, T_1h)
6. Store in cache as before

**File**: `services/strategy-generator/src/backtest/mod.rs`

#### 4.3 Implement forward_fill_align()

New function in backtest module:

```rust
/// Forward-fill lower-frequency features to align with higher-frequency timestamps.
///
/// For each target timestamp t:
///   Find largest index j in lf_timestamps where lf_timestamps[j] <= t
///   Copy lf_features[:, :, j] to output[:, :, i]
///
/// Uses binary search for O(T_hf * log(T_lf)) complexity.
fn forward_fill_align(
    lf_features: &Array3<f64>,
    lf_timestamps: &[i64],
    hf_timestamps: &[i64],
) -> Array3<f64>
```

**File**: `services/strategy-generator/src/backtest/mod.rs`

### Phase 2: Feature Offset & Naming

#### 4.4 Create MultiTimeframeFactorConfig

New struct that wraps a single `FactorConfig` but reports `feat_offset = n_factors * n_resolutions`:

```rust
// backtest-engine/src/config.rs
pub struct MultiTimeframeFactorConfig {
    pub base_config: FactorConfig,
    pub resolutions: Vec<String>,
}

impl MultiTimeframeFactorConfig {
    pub fn feat_count(&self) -> usize {
        self.base_config.feat_count() * self.resolutions.len()
    }

    pub fn feat_offset(&self) -> usize {
        self.feat_count()
    }

    /// Generate factor names with timeframe suffixes.
    /// ["return_1h", "vwap_deviation_1h", ..., "return_4h", ..., "return_1d", ...]
    pub fn factor_names(&self) -> Vec<String> {
        let mut names = Vec::with_capacity(self.feat_count());
        for res in &self.resolutions {
            for factor in &self.base_config.active_factors {
                names.push(format!("{}_{}", factor.name, res));
            }
        }
        names
    }
}
```

**File**: `services/backtest-engine/src/config.rs`

#### 4.5 Update StackVM Initialization

The VM already takes `feat_offset` from config. With `MultiTimeframeFactorConfig.feat_offset() = 75`, no VM code changes needed. Just pass the new config:

```rust
let vm = StackVM::with_window(&mtf_config, ts_window);
// vm.feat_offset = 75
```

### Phase 3: Evolution Loop Integration

#### 4.6 Update main.rs Evolution Setup

In the per-symbol evolution function:

```rust
// Before (P2):
let feat_offset = factor_config.feat_offset();  // 25
let factor_names: Vec<String> = factor_config.active_factors
    .iter().map(|f| f.name.clone()).collect();

// After (P3, when multi_timeframe enabled):
let mtf_config = MultiTimeframeFactorConfig {
    base_config: factor_config.clone(),
    resolutions: vec!["1h".into(), "4h".into(), "1d".into()],
};
let feat_offset = mtf_config.feat_offset();  // 75
let factor_names = mtf_config.factor_names();
// ["return_1h", ..., "spy_rel_strength_1h",
//  "return_4h", ..., "spy_rel_strength_4h",
//  "return_1d", ..., "spy_rel_strength_1d"]
```

**File**: `services/strategy-generator/src/main.rs`

#### 4.7 Update Data Loading Call

```rust
// Before (P2):
backtester.load_data(&symbols, lookback_days).await?;

// After (P3):
if multi_timeframe_enabled {
    backtester.load_data_multi_timeframe(
        &symbols, lookback_days, &["1h", "4h", "1d"]
    ).await?;
} else {
    backtester.load_data(&symbols, lookback_days).await?;
}
```

### Phase 4: LLM Oracle & Genome Decoder

#### 4.8 Update genome_decoder.rs

The `token_to_name()` and `encode_formula()` functions already take `factor_names` as a parameter. With 75 entries in `factor_names` and `feat_offset = 75`, they work without code changes.

```
decode_genome([0, 50, 75], 75, factor_names_75)
→ "return_1h return_1d ADD"
```

**Update needed**: Test cases need updating to use 75 factors / feat_offset=75.

**File**: `services/strategy-generator/src/genome_decoder.rs` (tests only)

#### 4.9 Update llm_oracle.rs Prompt

The `build_prompt()` function already lists factor names dynamically from `ctx.factor_names`. With 75 names, the prompt naturally expands. However, the prompt should be enhanced to explain the timeframe structure:

```rust
// After "## Available Features\n"
if is_multi_timeframe {
    prompt.push_str("Features are organized by timeframe:\n");
    prompt.push_str("- _1h suffix: hourly factors (high-frequency, captures intraday patterns)\n");
    prompt.push_str("- _4h suffix: 4-hour factors (medium-frequency, captures session trends)\n");
    prompt.push_str("- _1d suffix: daily factors (low-frequency, captures multi-day regimes)\n");
    prompt.push_str("Cross-timeframe combinations (e.g., momentum_1h momentum_1d SUB) ");
    prompt.push_str("can capture timeframe divergences.\n\n");
}
```

**File**: `services/strategy-generator/src/llm_oracle.rs`

#### 4.10 Update Random Genome Generation

In `genetic.rs`, `generate_random_rpn()` takes `feat_offset` as parameter. With `feat_offset = 75`, it already generates valid tokens in range `[0, 75+22]`. The only consideration: with 75 features, random formulas are more likely to pick features (75/97 ≈ 77%) vs operators (22/97 ≈ 23%), which may produce longer formulas before achieving stack-correctness.

**Potential tuning**: Bias random generation toward operators slightly when `feat_offset > 50`. This is optional and can be tuned post-deployment.

### Phase 5: Backtest Evaluation

#### 4.11 Walk-Forward OOS Evaluation

No changes needed to evaluation logic. The walk-forward slicing operates on time indices, which are already aligned to 1h. The cached `features` Array3 has shape `(1, 75, T_1h)` instead of `(1, 25, T_1h)`, but the StackVM handles this transparently.

#### 4.12 Future Returns

Future returns remain computed from 1h OHLCV data (the base resolution). The signal is generated at 1h granularity; the execution model (open-to-open with 1-bar delay) is unchanged.

#### 4.13 Annualization Factor

Unchanged. The base resolution is still 1h, so `annualization_factor()` returns the same value.

## 5. File Modification List

### Must Change

| File | Changes | Scope |
|------|---------|-------|
| `config/generator.yaml` | Add `multi_timeframe` section | Config |
| `services/backtest-engine/src/config.rs` | Add `MultiTimeframeFactorConfig` struct | ~40 lines |
| `services/strategy-generator/src/backtest/mod.rs` | Add `load_data_multi_timeframe()`, `forward_fill_align()` | ~150 lines |
| `services/strategy-generator/src/main.rs` | MTF config setup, data loading dispatch, factor_names generation | ~60 lines |
| `services/strategy-generator/src/llm_oracle.rs` | Add timeframe explanation to prompt | ~15 lines |

### Tests to Update

| File | Changes |
|------|---------|
| `services/strategy-generator/src/genome_decoder.rs` | Add test cases with feat_offset=75, MTF factor names |
| `services/backtest-engine/src/config.rs` | Tests for `MultiTimeframeFactorConfig` |
| New: `services/strategy-generator/src/backtest/alignment_tests.rs` | Tests for `forward_fill_align()` |

### No Changes Required

| File | Reason |
|------|--------|
| `services/backtest-engine/src/vm/vm.rs` | VM is generic over feat_offset |
| `services/backtest-engine/src/vm/ops.rs` | Operators unchanged |
| `services/backtest-engine/src/factors/engineer.rs` | Already config-driven, called per resolution |
| `config/factors-stock.yaml` | Reused as-is for all 3 resolutions |
| `services/strategy-generator/src/genetic.rs` | `generate_random_rpn(feat_offset)` already parameterized |
| `services/strategy-generator/src/genome_decoder.rs` | Logic already parameterized (code unchanged, only tests) |

## 6. Memory & Performance Considerations

### Memory Impact

| Component | P2 | P3 | Delta |
|-----------|----|----|-------|
| Feature tensor per symbol | (1, 25, ~12000) = 300K f64 = 2.4 MB | (1, 75, ~12000) = 900K f64 = 7.2 MB | +4.8 MB |
| 13 symbols cached | ~31 MB | ~94 MB | +63 MB |
| DB query overhead | 1 query/symbol | 3 queries/symbol | 2x more queries |

Total additional memory: ~63 MB. Acceptable for a server process.

### DB Query Optimization

The 3 resolution queries per symbol can be parallelized:

```rust
let (rows_1h, rows_4h, rows_1d) = tokio::try_join!(
    fetch_candles(&pool, symbol, exchange, "1h", days),
    fetch_candles(&pool, symbol, exchange, "4h", days),
    fetch_candles(&pool, symbol, exchange, "1d", days),
)?;
```

### Feature Computation

Feature computation is CPU-bound (ndarray operations). For 3 resolutions, computation time roughly triples. However, this is a one-time cost per symbol at startup (cached thereafter).

### VM Execution

Per-genome VM execution time is unchanged. The VM pushes feature slices by index; the tensor is larger but each `index_axis` call is the same cost. The only potential impact is CPU cache pressure from the larger feature tensor, which is negligible.

## 7. Risks and Mitigations

### Risk 1: Temporal Alignment Bugs

**Risk**: Off-by-one errors in forward-fill alignment could introduce look-ahead bias (using future 4h/1d data at a 1h timestamp).

**Mitigation**:
- Use strict `<=` comparison: for 1h timestamp `t`, only use lower-freq bars where `bar.time <= t`
- Write comprehensive tests with known timestamps and verify alignment manually
- Log alignment statistics: "4h bars: T_4h, mapped to T_1h bars, coverage: X%"
- Add assertion: no NaN in aligned features (missing alignment → zero, not NaN)

### Risk 2: Insufficient Lower-Frequency Data

**Risk**: Some symbols may have sparse 4h or 1d data, producing mostly-zero features.

**Mitigation**:
- Data availability check: 1d has 202K rows across 18K symbols (avg ~11 rows/symbol for daily = ~3 years). For our 13 active symbols, this is sufficient.
- Minimum bar requirement: if `T_res < 50` for any resolution, fall back to P2 single-resolution mode for that symbol
- Log coverage: "Symbol SPY: 1h=12000, 4h=3100, 1d=750 bars"

### Risk 3: Feature Space Explosion Slows Convergence

**Risk**: 75 features (vs 25) means 3x more feature tokens to explore. Random genome generation may need more generations to find useful combinations.

**Mitigation**:
- LLM Oracle (P2) already handles this: it generates semantically meaningful formulas targeting specific timeframes
- ALPS layer structure provides age-based selection pressure that scales with feature space
- Monitor convergence rate: if L0→L1 promotion rate drops significantly, tune random generation bias
- Consider pre-seeding Layer 0 with known cross-timeframe patterns (e.g., `momentum_1h momentum_1d SUB`)

### Risk 4: All Existing Genomes Invalidated

**Risk**: Fresh start means losing ~78K generations of evolution progress.

**Mitigation**:
- This is expected and unavoidable — feat_offset changes make old tokens meaningless
- P2 baseline PSR values serve as comparison targets
- The resume check in main.rs already handles offset mismatch gracefully
- LLM Oracle cross-symbol learning will accelerate re-convergence using P3's richer feature space

### Risk 5: Forward-Filled Features Create Artificial Correlations

**Risk**: A 1d factor value repeated 6.5 times at 1h granularity creates plateau patterns. TS_MEAN on a forward-filled feature is just the feature itself (no variation within the fill period).

**Mitigation**:
- This is a feature, not a bug: the GA will learn that `TS_MEAN(momentum_1d)` is redundant and prefer raw `momentum_1d` or combine it with 1h factors
- The diversity of 3 resolutions means the GA has both smooth (1d) and noisy (1h) signals to combine
- Normalization (robust/z-score) still applies per-resolution before stacking

### Risk 6: LLM Prompt Becomes Too Long

**Risk**: 75 factor names + descriptions in the prompt may exceed model context or reduce quality.

**Mitigation**:
- Factor names are short (`return_1h`, `momentum_4h`): 75 names ≈ 150 extra tokens
- Grouping by timeframe (as designed in 4.9) keeps the prompt structured
- Current prompt + 75 factors ≈ ~1500 tokens total, well within model limits
- If needed, only list factor names (not descriptions) in the prompt

## 8. Rollback Strategy

P3 is gated by `multi_timeframe.enabled` in `generator.yaml`. To rollback:

1. Set `multi_timeframe.enabled: false` (or remove the section)
2. Restart strategy-generator
3. Evolution resumes with P2 behavior (25 features, feat_offset=25)
4. Old P2 genomes in DB remain valid and can be resumed

No database migrations are needed. The `strategy_generations` table already stores `feat_offset` in metadata JSON, so P2 and P3 genomes coexist safely.

## 9. Implementation Order

| Step | Description | Dependencies | Est. Lines |
|------|-------------|-------------|------------|
| 1 | `MultiTimeframeFactorConfig` in `config.rs` | None | 40 |
| 2 | `forward_fill_align()` + unit tests | None | 80 |
| 3 | `load_data_multi_timeframe()` in backtest | Steps 1, 2 | 150 |
| 4 | `generator.yaml` schema + parsing in `main.rs` | Step 1 | 40 |
| 5 | Evolution loop MTF integration in `main.rs` | Steps 3, 4 | 60 |
| 6 | LLM Oracle prompt update | Step 1 | 15 |
| 7 | Genome decoder tests update | Step 1 | 30 |
| 8 | Integration test: load → stack → evaluate 1 symbol | Steps 1-5 | 50 |
| 9 | Docker build + deploy + smoke test | Step 8 | - |
| 10 | Monitor first 1000 generations, compare to P2 baseline | Step 9 | - |

**Total new/modified code**: ~465 lines (excluding tests).
