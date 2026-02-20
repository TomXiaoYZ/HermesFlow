# HermesFlow Strategy Evolution System — Complete Status Report

> Generated: 2026-02-20 | System: Running | Generation: ~49,000+

---

## Table of Contents

1. [System Architecture Overview](#1-system-architecture-overview)
2. [Genetic Algorithm: ALPS Implementation](#2-genetic-algorithm-alps-implementation)
3. [Factor System (13 Stock Factors)](#3-factor-system-13-stock-factors)
4. [VM (Stack Machine) & Operator Set](#4-vm-stack-machine--operator-set)
5. [Backtest Engine & Signal Logic](#5-backtest-engine--signal-logic)
6. [Fitness Function: PSR (Probabilistic Sharpe Ratio)](#6-fitness-function-psr)
7. [K-Fold Temporal Cross-Validation](#7-k-fold-temporal-cross-validation)
8. [Current Evolution Results (Live Data)](#8-current-evolution-results-live-data)
9. [Best Genome Examples (Decoded Formulas)](#9-best-genome-examples-decoded-formulas)
10. [Identified Problems & Anomalies](#10-identified-problems--anomalies)
11. [Configuration Summary](#11-configuration-summary)
12. [Database Schema](#12-database-schema)
13. [Open Questions for Direction](#13-open-questions-for-direction)

---

## 1. System Architecture Overview

### Tech Stack
- **Strategy Generator**: Rust (Tokio async), runs 26 parallel evolution loops (13 symbols x 2 modes)
- **Backtest Engine**: Rust library, called in-process by generator
- **VM**: Custom stack-based RPN (Reverse Polish Notation) calculator
- **Storage**: PostgreSQL/TimescaleDB for generations + backtest results
- **Pub/Sub**: Redis for real-time strategy updates
- **Frontend**: Next.js dashboard (EvolutionExplorer component)
- **Data Source**: Polygon.io (US equities, 1-hour candles)

### Evolution Loop (per symbol, per mode)
```
Every 5 seconds:
  1. Adaptive K-fold computation (K = clamp(data_len / 300, 3, 8))
  2. Evaluate all 500 genomes via K-fold temporal CV → IS fitness (PSR z-score)
  3. ALPS evolution: age increment → promote over-aged → replenish layer 0 → evolve layers
  4. Evaluate best genome: compute OOS PSR + per-fold PSRs
  5. Persist to DB: generation record + metadata (oos_psr, fold_psrs, stagnation)
  6. Every 5 gen: run detailed backtest simulation → persist equity curve + trades
  7. Every 10 gen (after gen 100): cleanup old generations (keep window [gen-1000, gen+100])
```

### Target Universe
13 US equities via Polygon: `AAPL, MSFT, GOOGL, AMZN, META, NVDA, TSLA, SPY, QQQ, DIA, IWM, UVXY, GLD`

### Dual-Mode Evolution
Each symbol runs two independent GA evolutions:
- **long_only**: Position space {0, 1} — only long positions
- **long_short**: Position space {-1, 0, 1} — can go long, flat, or short

---

## 2. Genetic Algorithm: ALPS Implementation

### ALPS (Age-Layered Population Structure)

Replaces flat GA with 5 Fibonacci-aged layers to combat stagnation:

| Layer | Max Age | Pop Size | Role |
|-------|---------|----------|------|
| 0 | 5 | 100 | Fresh exploration (always replenished with random genomes) |
| 1 | 13 | 100 | Young promising candidates |
| 2 | 34 | 100 | Maturing strategies |
| 3 | 89 | 100 | Experienced, well-tested |
| 4 | 500 | 100 | Elite archive (over-aged discarded) |

**Total population: 500 genomes per (symbol, mode)**

### ALPS Evolution Phases (per generation)

1. **Age all genomes**: `genome.age += 1` for every genome in every layer
2. **Promote over-aged** (bottom-up): If `genome.age > layer.max_age`, promote to next layer
   - If next layer has space: insert directly
   - If next layer full: replace worst genome if promoted is fitter
   - If top layer (4): discard over-aged (age > 500) to prevent stagnation
3. **Replenish layer 0**: Fill to 100 with fresh random genomes
4. **Evolve each layer independently**:
   - Sort by fitness, deduplicate (replace duplicate genomes with random)
   - **Elitism**: Keep top 5% (min 2 genomes)
   - **Fill remaining** via:
     - 40% — Crossover + Mutation (tournament select 2 parents, k=3)
     - 35% — Mutation only (clone 1 parent, mutate)
     - 25% — Immigration (fresh random genome)
5. **Update global best genome**: Scan all layers, track all-time best by IS fitness

### Crossover Operator
- **Type**: Single-point crossover at valid RPN cut points
- **Cut point selection**: Positions where stack depth == 1 (valid sub-expression boundary)
- **Child**: prefix of parent1 + suffix of parent2
- **Max length enforcement**: Truncate to 20 tokens
- **Age inheritance**: `child.age = max(parent1.age, parent2.age)`

### Mutation Operators (5 independent, can stack)

| Operator | Probability | Description |
|----------|-------------|-------------|
| Point mutation | 40% | Change any token to same-arity token (feature→feature, op→same-arity op) |
| Operator mutation | 20% | Find random operator, swap for same-arity operator |
| Growth mutation | 8% | Append (Feature + BinaryOp), only if genome < 20 tokens |
| Shrink mutation | 20% | Remove random unary operator, only if genome > 3 tokens |
| Subtree replacement | 10% | Replace entire genome with new random formula (age reset to 0) |

**Expected mutations per call**: ~0.98 (independently sampled, multiple can apply)

### Tournament Selection
- Tournament size: k=3
- Selects best fitness among 3 random individuals from current layer

### Genome Structure
```rust
struct Genome {
    tokens: Vec<usize>,    // RPN formula (max 20 tokens)
    fitness: f64,          // IS fitness (PSR z-score from K-fold CV)
    age: usize,            // Generations survived (for ALPS layer management)
}
```

### Random Genome Generation
- Target length: random [3, 12] tokens
- Max stack depth: 5
- Features: random from [0, feat_offset)
- Operators: from pruned set (14 active operators)
- Ensures valid RPN: stack depth exactly 1 at end

---

## 3. Factor System (13 Stock Factors)

### Configuration (`config/factors-stock.yaml`)

Resolution: **1 hour** | Lookback: **730 days (2 years)**

| ID | Name | Formula | Normalization | Window |
|----|------|---------|---------------|--------|
| 0 | return | `ln(close[t] / close[t-1])` | robust | 1-period |
| 1 | vwap_deviation | `(close - VWAP) / VWAP` | robust | 20-bar rolling |
| 2 | volume_ratio | `volume / SMA(volume, 20)`, clamped [0, 10] | robust | 20-bar SMA |
| 3 | mean_reversion | `(close - SMA(close, 20)) / SMA(close, 20)` | robust | 20-bar SMA |
| 4 | adv_ratio | `(close * volume) / SMA(close * volume, 20)`, clamped [0, 10] | robust | 20-bar SMA |
| 5 | volatility | `sqrt(SMA(ln_return^2, 20))` | robust | 20-bar rolling |
| 6 | momentum | `(close[t] - close[t-20]) / close[t-20]` | robust | 20-bar lookback |
| 7 | relative_strength | `(RSI(14) - 50) / 50` → [-1, 1] | robust | 14-bar RSI |
| 8 | close_position | `(close - low) / (high - low)`, clamped [0, 1] | **none** | instantaneous |
| 9 | intraday_range | `(high - low) / close` | robust | instantaneous |
| 10 | vol_regime | `(short_vol_20 - long_mean_60) / long_std_60`, clamped [-3, 3] | robust | 20/60 dual window |
| 11 | trend_strength | Linear regression slope / price × window, clamped [-5, 5] | robust | 20-bar regression |
| 12 | momentum_regime | `2 × frac(positive_returns, 20) - 1` → [-1, 1] | **none** | 20-bar rolling |

### Robust Normalization (applied to 11/13 factors)
```
median = median(time_series)
MAD = median(|time_series - median|)
normalized = (value - median) / (MAD + 1e-6)
output = clamp(normalized, -5.0, 5.0)
```

### Data Layout
- Input: OHLCV from Polygon (1h candles, 730 days ≈ 11,388 bars for 6.5h trading day)
- Factor tensor: `Array3<f64>` shape `(batch=1_per_symbol, 13_factors, time_steps)`
- Each factor is computed independently, then normalized, then stacked

---

## 4. VM (Stack Machine) & Operator Set

### StackVM Architecture
- **Input**: RPN token sequence + 3D feature tensor
- **Output**: `Array2<f64>` shape `(batch, time)` — raw signal per time step
- **feat_offset**: 13 (number of factors; tokens 0-12 are features, 13+ are operators)
- **ts_window**: 10 (for 1h resolution; used by TS_MEAN, TS_STD, etc.)

### Complete Opcode Table (23 total)

| OpIdx | Name | Arity | Status | Description |
|-------|------|-------|--------|-------------|
| 0 | ADD | 2 | **active** | x + y |
| 1 | SUB | 2 | **active** | x - y |
| 2 | MUL | 2 | **active** | x × y |
| 3 | DIV | 2 | **active** | x / (y + 1e-9) |
| 4 | NEG | 1 | legacy | -x |
| 5 | ABS | 1 | **active** | \|x\| |
| 6 | SIGN | 1 | **active** | sign(x) ∈ {-1, 0, 1} |
| 7 | GATE | 3 | legacy | if cond > 0 then x else y |
| 8 | SIGNED_POWER | 1 | legacy | sign(x) × sqrt(\|x\|) |
| 9 | DECAY_LINEAR | 1 | legacy | Linearly-weighted MA |
| 10 | DELAY1 | 1 | **active** | ts_delay(x, 1) |
| 11 | DELAY5 | 1 | **active** | ts_delay(x, 5) |
| 12 | TS_MEAN | 1 | **active** | Rolling mean (window=10) |
| 13 | TS_STD | 1 | **active** | Rolling std (window=10) |
| 14 | TS_RANK | 1 | **active** | Normalized rank in window [0,1] |
| 15 | TS_SUM | 1 | legacy | Rolling sum |
| 16 | TS_CORR | 2 | **active** | Rolling correlation |
| 17 | TS_MIN | 1 | **active** | Rolling minimum |
| 18 | TS_MAX | 1 | **active** | Rolling maximum |
| 19 | LOG | 1 | legacy | ln(\|x\|), 0 if \|x\| < 1e-9 |
| 20 | SQRT | 1 | legacy | sqrt(\|x\|) |
| 21 | TS_ARGMAX | 1 | legacy | Position of max in window [0,1] |
| 22 | TS_DELTA | 1 | legacy | x[t] - x[t-1] |

**14 active** (used for new genome generation): ABS, SIGN, DELAY1, DELAY5, TS_MEAN, TS_STD, TS_RANK, TS_MIN, TS_MAX, ADD, SUB, MUL, DIV, TS_CORR

**9 legacy** (retained for backward compatibility with stored genomes): NEG, GATE, SIGNED_POWER, DECAY_LINEAR, TS_SUM, LOG, SQRT, TS_ARGMAX, TS_DELTA

### Pruning Rationale
- NEG: Redundant (can use SUB or MUL with negative features)
- GATE: Ternary operator, overly complex for GA to optimize
- SIGNED_POWER, DECAY_LINEAR: Specialized, domain-error-prone
- TS_SUM: Redundant with TS_MEAN × window
- LOG, SQRT: Domain errors on negative inputs
- TS_ARGMAX, TS_DELTA: Unstable/legacy

### NaN/Inf Protection (3 layers)
1. **Operator-level**: Division adds 1e-9 epsilon; LOG returns 0 for tiny inputs; SQRT takes abs first
2. **VM post-op**: After every operator, replace NaN→0.0, +Inf→1.0, -Inf→-1.0
3. **Return-level**: If final stack ≠ 1 element, return None → fitness = -10.0

---

## 5. Backtest Engine & Signal Logic

### Signal Generation
```
raw_signal = VM.execute(genome_tokens, feature_tensor)    // shape (batch, time)
sigmoid_signal = 1 / (1 + exp(-raw_signal))               // map to (0, 1)
```

### Adaptive Threshold Computation
```
signals_sorted = sort(sigmoid_signal)
adaptive_upper = percentile(sigmoid_signal, 70th), clamped [0.52, 0.80]
adaptive_lower = percentile(sigmoid_signal, 30th), clamped [0.20, 0.48]
```

### Position Logic

**Long-Only**: `position = sigmoid > adaptive_upper ? 1.0 : 0.0`

**Long-Short**:
```
position = sigmoid > adaptive_upper ? 1.0
         : sigmoid < adaptive_lower ? -1.0
         : 0.0
```

### Transaction Cost Model

| Parameter | Polygon (Stocks) |
|-----------|-----------------|
| Base fee | 0.01% (1 bp) |
| Max impact slippage | 5% |
| Short borrow premium | 50% extra on entry cost |
| Impact formula | `min(trade_size / liquidity, 0.05)` |

```
turnover[t] = |position[t] - position[t-1]|
cost[t] = turnover[t] × (base_fee + impact)
// Short entry: cost × 1.5
net_pnl[t] = position[t] × return[t] - cost[t]
```

### Equity Curve
```
equity[0] = 1.0
equity[t] = equity[t-1] × (1.0 + net_pnl[t])
// Break if equity <= 0 (bankrupt)
```

### Sharpe Ratio Annualization
- 1h stock data: `sqrt(252 × 6.5)` ≈ sqrt(1638) ≈ 40.47

### Minimum Activity Requirements
- `min_trades = max(3, trading_days / 10)` — at least 1 trade per ~10 days
- `min_active_bars = 5% of fold length`
- Strategies that don't trade enough get penalized

### Complexity Penalty (Parsimony)
```
penalty = 0.02 × max(0, token_count - 8) × scale
scale = (1000 / data_length).clamp(0.2, 1.0)
```
Incentivizes shorter formulas; extra 0.02 penalty per token beyond 8.

### Large Drawdown Penalty
```
If fold loss < -2%: penalty += 0.5 per excess occurrence
```

---

## 6. Fitness Function: PSR (Probabilistic Sharpe Ratio)

### Bailey & Lopez de Prado (2012)

PSR z-score measures the probability that the true Sharpe ratio is greater than 0, accounting for non-normality of returns:

```
PSR = (sharpe - 0) / SE(sharpe)

SE(sharpe) = sqrt((1 - skew × sharpe + (kurtosis - 1)/4 × sharpe²) / N)

where:
  sharpe = mean_return / std_return (annualized)
  skew = third standardized moment of returns
  kurtosis = fourth standardized moment of returns
  N = number of observations
```

### IS Fitness Computation
```
For each of K folds (IS portion):
  1. Run backtest on fold → get returns series
  2. Compute sharpe, skew, kurtosis, N
  3. Compute PSR z-score, clamp to [-5, 5]

fitness = mean(fold_PSRs) - 0.5 × std(fold_PSRs) - complexity_penalty
```

The `- 0.5 × std(fold_PSRs)` term penalizes inconsistency across folds.

### OOS PSR Computation
- Separate backtest on held-out data (last 30% of time series)
- Independent PSR z-score computation
- Clamped to [-5, 5]
- Value of -10.0 means: strategy produced no valid trades or had NaN/Inf output on OOS data

### IS-OOS Gap Monitoring
```
If IS_fitness > 1.0 AND OOS_PSR < 0.0 AND (IS_fitness - OOS_PSR) > 2.0:
  → Log WARNING: IS-OOS divergence (overfitting signal)
```

---

## 7. K-Fold Temporal Cross-Validation

### Adaptive K
```
K = clamp(data_length / 300, 3, 8)
```
For 730 days × ~6.5h/day of 1h bars ≈ 11,388 bars: K = clamp(11388/300, 3, 8) = **8 folds**

### Embargo (Resolution-Aware Gap)
- 1h resolution: 10-bar embargo between folds
- Purpose: Prevent TS operators from carrying information across fold boundaries
- Embargo bars are excluded from both adjacent folds

### Data Split
- **IS (In-Sample)**: First 70% of data — used for K-fold CV fitness
- **OOS (Out-of-Sample)**: Last 30% — used for OOS PSR evaluation only

### Fold PSR Values
- Each fold produces an independent PSR z-score
- Value of -10.0 for a fold means: no valid trades or NaN/Inf output in that fold
- Currently most strategies have only 3-4 out of 8 folds producing valid PSR scores

---

## 8. Current Evolution Results (Live Data)

### Long-Only Mode

| Symbol | Gen | IS Fitness | OOS PSR | PnL | Sharpe | MaxDD | WinRate | Valid Folds |
|--------|-----|-----------|---------|-----|--------|-------|---------|-------------|
| **GLD** | 49,127 | 3.41 | -10.0 | +15.4% | 2.95 | 0.7% | 65.5% | 3/8 |
| **QQQ** | 49,131 | 3.04 | **2.69** | +32.9% | 1.85 | 3.2% | 68.2% | 3/8 |
| **META** | 49,015 | 3.07 | **5.0** | +18.9% | 1.34 | 1.0% | 39.7% | 3/8 |
| **TSLA** | 49,237 | 3.01 | **5.0** | +72.9% | 1.14 | 7.2% | 48.7% | 4/8 |
| **NVDA** | 49,236 | 3.01 | -10.0 | +27.7% | 1.69 | 8.1% | 43.8% | 3/8 |
| **IWM** | 49,145 | 2.99 | -10.0 | +2.0% | 0.28 | 5.4% | 29.8% | 3/8 |
| **MSFT** | 49,017 | 2.92 | -10.0 | +31.8% | 1.29 | 7.1% | 17.5% | 3/8 |
| **QQQ** | 49,131 | 3.04 | **2.69** | +32.9% | 1.85 | 3.2% | 68.2% | 3/8 |
| **AAPL** | 49,149 | 2.56 | -10.0 | +33.9% | 0.81 | 4.1% | 36.6% | 3/8 |
| **DIA** | 49,129 | 2.36 | -10.0 | +12.2% | 1.08 | 3.3% | 35.9% | 3/8 |
| **UVXY** | 49,090 | 2.52 | -10.0 | +58.8% | 1.95 | 4.0% | 20.5% | 3/8 |
| **AMZN** | 49,120 | 2.03 | -10.0 | +34.7% | 1.34 | 4.9% | 36.7% | 3/8 |
| **GOOGL** | 49,071 | 2.02 | -10.0 | +29.0% | 0.81 | 3.2% | 49.0% | 3/8 |
| **SPY** | 49,129 | 2.04 | -10.0 | +16.3% | 1.14 | 4.7% | 50.0% | 3/8 |

**Long-Only Summary**: Only 3/13 symbols have positive OOS PSR (META, TSLA, QQQ). Most have OOS = -10.0 (failed to generalize).

### Long-Short Mode

| Symbol | Gen | IS Fitness | OOS PSR | PnL | Sharpe | MaxDD | WinRate | Valid Folds |
|--------|-----|-----------|---------|-----|--------|-------|---------|-------------|
| **NVDA** | 49,145 | 2.12 | **3.44** | **+1435.8%** | **3.20** | 17.4% | 44.6% | **8/8** |
| **AAPL** | 49,038 | 1.57 | **3.36** | +411.4% | 2.71 | 9.8% | 54.3% | **8/8** |
| **TSLA** | 49,135 | 3.09 | **5.0** | +115.3% | 1.54 | 3.4% | 61.1% | 4/8 |
| **GLD** | 49,025 | 2.62 | **1.66** | +9.2% | 1.32 | 3.3% | 76.0% | 3/8 |
| **META** | 48,934 | 2.81 | **3.04** | +126.6% | 1.82 | 21.5% | 43.9% | 4/8 |
| **MSFT** | 48,942 | 2.28 | **3.89** | +18.9% | 0.60 | 23.3% | 61.0% | 4/8 |
| **GOOGL** | 48,981 | 2.43 | -10.0 | +151.6% | 1.27 | 28.4% | 63.2% | 3/8 |
| **DIA** | 49,033 | 1.96 | -10.0 | +13.7% | 1.26 | 1.2% | 60.9% | 3/8 |
| **IWM** | 49,017 | 1.49 | -10.0 | +23.8% | 0.62 | 25.2% | 64.7% | 3/8 |
| **QQQ** | 49,021 | 1.93 | -10.0 | +6.7% | 1.40 | 2.0% | 78.3% | 3/8 |
| **UVXY** | 49,015 | 2.65 | -10.0 | +47.6% | 0.79 | 42.8% | 43.3% | 3/8 |
| **AMZN** | 49,015 | 3.43 | -10.0 | **0.0%** | 0.0 | 0.0% | 0.0% | 3/8 |
| **SPY** | 49,030 | 1.97 | -10.0 | **0.0%** | 0.0 | 0.0% | 0.0% | 4/8 |

**Long-Short Summary**: 6/13 symbols have positive OOS PSR. NVDA and AAPL stand out with 8/8 fold consistency. AMZN and SPY produce 0 trades (dead strategies despite high IS fitness — severe overfitting).

---

## 9. Best Genome Examples (Decoded Formulas)

Token encoding: Features are indices 0-12, operators are index 13+ (feat_offset=13).

### NVDA Long-Short (Best performer: Sharpe 3.20, PnL +1435.8%)
```
Tokens: [11, 27, 11, 30, 7, 14, 14, 11, 24, 7, 14, 14]
Length: 12

Decoded (feat_offset=13):
  11 = trend_strength (feature)
  27 = 13+14 = TS_RANK
  11 = trend_strength (feature)
  30 = 13+17 = TS_MIN
  7  = relative_strength (feature)
  14 = 13+1 = SUB (binary)
  14 = 13+1 = SUB (binary)
  11 = trend_strength (feature)
  24 = 13+11 = DELAY5
  7  = relative_strength (feature)
  14 = 13+1 = SUB (binary)
  14 = 13+1 = SUB (binary)

Formula: ((TsRank(trend_strength) - (TsMin(trend_strength) - RSI)) - (Delay5(trend_strength) - RSI))
```

**Observation**: Heavily uses `trend_strength` (factor 11) and `relative_strength/RSI` (factor 7). Compares rank of trend vs min of trend, adjusted by RSI momentum.

### AAPL Long-Short (Sharpe 2.71, PnL +411.4%)
```
Tokens: [3, 30, 10, 13, 10, 12, 30, 24, 16, 13, 30, 10, 12, 30, 16, 13, 31, 24, 2, 13]
Length: 20 (maximum)

Key features used: mean_reversion (3), momentum (6, possibly via index shifting), TS_MIN, TS_MAX
```

### TSLA Long-Short (OOS PSR 5.0, PnL +115.3%)
```
Tokens: [11, 7, 13, 18, 31, 1, 15, 2, 14, 24, 4, 14, 24, 3, 29, 9, 31, 19, 29]
Length: 19
```

### GLD Long-Short (Win Rate 76%, Sharpe 1.32)
```
Tokens: [3, 4, 30, 19, 29, 27, 27, 6, 16, 6, 16]
Length: 11 (compact)
```

---

## 10. Identified Problems & Anomalies

### Problem 1: Massive IS-OOS Divergence (Most Critical)

**Symptom**: Most strategies have IS fitness 2-3+ but OOS PSR = -10.0

**Data**: Out of 26 strategy slots (13 symbols × 2 modes):
- Long-Only: Only 3/13 have OOS PSR > 0 (META, TSLA, QQQ)
- Long-Short: Only 6/13 have OOS PSR > 0 (NVDA, AAPL, TSLA, GLD, META, MSFT)
- **17/26 strategies (65%) show complete OOS failure** despite high IS fitness

**Logs show constant warnings**:
```
WARN: [Polygon:NVDA:long_only] Gen 49254 — IS-OOS divergence (IS=3.013, OOS=-10.000, gap=13.013)
WARN: [Polygon:AMZN:long_short] Gen 49033 — IS-OOS divergence (IS=3.430, OOS=-10.000, gap=13.430)
```

**Possible causes**:
- OOS PSR of -10.0 might be a "no trades" penalty, not a real negative PSR
- The 70/30 IS/OOS split may be too aggressive for 1h data
- Adaptive thresholds computed on IS data don't generalize to OOS distribution
- Strategies are fitting to IS-specific volatility regimes that don't exist in OOS period

### Problem 2: Dead Strategies (AMZN, SPY Long-Short)

**Symptom**: AMZN and SPY long_short have 0 PnL, 0 trades, 0 win rate despite IS fitness 3.43 and 1.97 respectively.

**Possible causes**:
- Adaptive thresholds are too tight for these symbols in the OOS period
- The genome produces signals that cluster around the neutral zone (sigmoid ≈ 0.5) in OOS
- Symbol-specific data quality issues

### Problem 3: Low Valid Fold Count

**Symptom**: Most strategies have only 3/8 folds with valid PSR (non -10.0). Even the best IS fitness strategies fail on 5 of 8 time windows.

**Implication**: The strategies are not robust across time periods. They may be exploiting specific market regimes (e.g., 2024 bull run) rather than finding structural patterns.

### Problem 4: Convergence at ~49,000 Generations

**Symptom**: Evolution has been running for ~49,000 generations. ALPS should provide continuous exploration via layer 0 immigration (25% random per layer), but:
- Best genomes have ages 200-500 (near the top-layer cap)
- No significant improvement in OOS metrics recently
- IS fitness is stable (not improving)

**Question**: Has the search space been exhausted with 14 operators × 13 features? Or is the fitness landscape deceptive (IS fitness plateau doesn't correspond to OOS improvement)?

### Problem 5: Long-Only Underperformance

**Symptom**: Long-Only mode systematically underperforms Long-Short. Only 3/13 have positive OOS PSR vs 6/13 for Long-Short.

**Possible explanation**: Long-only can only profit from upward moves. In a 2-year lookback that likely includes both bull and bear periods, being unable to short during downturns limits the strategy's ability to generate consistent positive Sharpe across all folds.

### Problem 6: MSFT Low Win Rate (17.5%)

The MSFT long_only strategy has 17.5% win rate but +31.8% PnL. This is a "few big wins, many small losses" profile that is psychologically difficult to execute in real trading and may indicate fragility.

---

## 11. Configuration Summary

### Generator Config (`config/generator.yaml`)
```yaml
exchanges:
  - exchange: Polygon
    resolution: "1h"
    lookback_days: 730
    factor_config: "config/factors-stock.yaml"
```

### Key Parameters (Hardcoded)

| Parameter | Value | Location |
|-----------|-------|----------|
| ALPS layers | 5 | genetic.rs:135 |
| Layer max ages | [5, 13, 34, 89, 500] | genetic.rs:135 |
| Pop per layer | 100 | genetic.rs:136 |
| Total population | 500 | derived |
| Max genome length | 20 tokens | genetic.rs:402 |
| Random genome length | [3, 12] | genetic.rs:79 |
| Max stack depth | 5 | passed to generate_random_rpn |
| Tournament size | 3 | genetic.rs:172 |
| Elitism rate | 5% (min 2) | genetic.rs:334 |
| Crossover rate | 40% | genetic.rs:349 |
| Mutation-only rate | 35% | genetic.rs:354 |
| Immigration rate | 25% | genetic.rs:359 |
| K-fold range | [3, 8] | main.rs:472 |
| Target bars/fold | 300 | main.rs:471 |
| IS/OOS split | 70/30 | backtest |
| Embargo (1h) | 10 bars | backtest |
| Parsimony threshold | 8 tokens | backtest |
| Parsimony penalty | 0.02/token | backtest |
| Drawdown penalty trigger | -2% | backtest |
| Drawdown penalty value | 0.5 per excess | backtest |
| TS_WINDOW (1h) | 10 bars | vm.rs |
| Adaptive upper threshold | 70th pct, [0.52, 0.80] | backtest |
| Adaptive lower threshold | 30th pct, [0.20, 0.48] | backtest |
| Min activity | max(3, days/10) trades | backtest |
| Generation interval | 5 seconds | main.rs:651 |
| Backtest freq | Every 5 generations | main.rs:573 |
| Cleanup freq | Every 10 gen after gen 100 | main.rs:635 |
| Retention window | [gen-1000, gen+100] | main.rs:643 |
| Base fee (stocks) | 0.01% (1 bp) | backtest |
| Short premium | 1.5× | backtest |
| Max impact | 5% | backtest |

### Fitness Formula
```
fold_psr[i] = clamp(PSR_zscore(fold_i_returns), -5, 5)
IS_fitness = mean(fold_psrs) - 0.5 × std(fold_psrs) - complexity_penalty
OOS_PSR = PSR_zscore(oos_returns), clamped to [-5, 5], or -10 if no trades
```

---

## 12. Database Schema

### strategy_generations
```sql
PRIMARY KEY (exchange, symbol, mode, generation)

Columns:
  exchange TEXT NOT NULL DEFAULT 'Birdeye'
  symbol TEXT NOT NULL DEFAULT 'UNIVERSAL'
  mode TEXT NOT NULL DEFAULT 'long_only'   -- 'long_only' or 'long_short'
  generation INTEGER
  fitness DOUBLE PRECISION                 -- IS PSR z-score
  best_genome INTEGER[]                    -- VM opcode array
  strategy_id TEXT                         -- e.g. 'polygon_AAPL_long_only_gen_49000'
  timestamp TIMESTAMPTZ DEFAULT NOW()
  metadata JSONB                           -- {oos_psr, fold_psrs, stagnation, age, ...}
```

### backtest_results
```sql
PRIMARY KEY (id UUID)

Columns:
  strategy_id VARCHAR(255)
  genome INTEGER[]
  token_address VARCHAR(255) NOT NULL      -- symbol name
  mode TEXT NOT NULL DEFAULT 'long_only'
  pnl_percent DOUBLE PRECISION
  sharpe_ratio DOUBLE PRECISION
  max_drawdown DOUBLE PRECISION
  win_rate DOUBLE PRECISION
  total_trades INTEGER
  equity_curve JSONB                       -- [{time, value}, ...]
  trades JSONB                             -- [{entry, exit, pnl, direction}, ...]
  metrics JSONB                            -- {sortino, calmar, profit_factor, ...}
  created_at TIMESTAMPTZ DEFAULT NOW()
```

### Retention Policy
- Generations: Keep [gen-1000, gen+100] window per (exchange, symbol, mode)
- Backtests: Keep only latest per (symbol, mode)

---

## 13. Open Questions for Direction

### Strategic Questions

1. **Should we continue evolving?** At ~49K generations with 500 population × 14 operators × 13 features, has the search space been adequately explored? Or should we expand the search space first?

2. **How to improve OOS generalization?** 65% of strategies fail OOS. Is this a fundamental limitation of the approach, or can we fix it with:
   - Larger OOS portion (currently 30%)?
   - Walk-forward validation instead of fixed split?
   - Ensemble methods (combining multiple genomes)?
   - Regularization in the fitness function?

3. **Should we add more factors?** Currently 13 factors. Candidates include:
   - Sector relative strength
   - Earnings/fundamental signals
   - Cross-asset correlations (SPY vs individual stocks)
   - Higher-frequency microstructure (bid-ask spread, order flow)
   - Macro regime indicators

4. **Should we increase operator set?** Re-enable some pruned operators (e.g., TS_DELTA, DECAY_LINEAR)? Or add new ones (e.g., conditional operators, normalization operators)?

5. **Position sizing**: Currently binary (0/1 or -1/0/1). Should we evolve continuous position sizes? Or add a separate position sizing layer?

6. **Multi-symbol strategies**: Currently each symbol evolves independently. Should we evolve cross-sectional strategies that allocate across multiple symbols?

7. **Regime detection**: Should we evolve separate strategies for different market regimes (high vol vs low vol, trending vs mean-reverting)?

### Technical Questions

8. **OOS = -10.0 diagnosis**: Is -10.0 always "no trades" or can it also mean "genuine negative PSR"? Need to distinguish these cases.

9. **Adaptive threshold leakage**: Are the adaptive thresholds (70th/30th percentile) computed on IS data and then applied to OOS without recalculation? This could cause threshold mismatch.

10. **ALPS age cap**: Is 500 generations the right cap for the elite layer? Some best genomes are at age 450-490, suggesting they're about to be discarded.

11. **Fold failure rate**: Why do 5/8 folds fail? Is it because the strategy genuinely doesn't work in those time periods, or is there a data issue (insufficient bars, regime change)?

12. **Complexity ceiling**: Most best genomes are 11-20 tokens. Is 20 tokens enough expressiveness, or should the limit be higher?

---

*End of Report*
