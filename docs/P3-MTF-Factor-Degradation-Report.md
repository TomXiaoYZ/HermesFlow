# P3 Multi-Timeframe Factor Expansion: Strategy Degradation Analysis Report

**Date:** 2026-02-23
**Context:** HermesFlow quantitative trading platform
**Purpose:** Technical consultation request for Gemini advisor — detailed evaluation of strategy performance degradation after P3 multi-timeframe factor expansion

---

## 1. Problem Statement

After P3 phase introduced multi-timeframe stacking (25 base factors x 3 resolutions = 75 features), **all symbols across both long_only and long_short modes show significant strategy quality degradation** compared to the original 25-factor single-timeframe configuration.

Observed symptoms:
- Lower PSR fitness values across all symbols (Polygon exchange)
- Higher `too_few_trades` (TFT) rate in the population
- Slower convergence in ALPS layer promotions
- LLM Oracle triggered more frequently (due to stagnation detection)
- Strategies that previously performed well on specific symbols no longer emerge

**Core question: Is this expected? If so, what is the optimal remediation path that preserves the multi-timeframe information advantage while restoring search efficiency?**

---

## 2. System Architecture Context

### 2.1 Factor Configuration

25 base factors defined in `config/factors-stock.yaml`:

| ID | Factor | Description | Normalization |
|----|--------|-------------|---------------|
| 0 | return | Log return (1-period) | robust |
| 1 | vwap_deviation | (close - VWAP) / VWAP | robust |
| 2 | volume_ratio | Volume / SMA(volume, 20) | robust |
| 3 | mean_reversion | Deviation from 20-period MA | robust |
| 4 | adv_ratio | Dollar volume / ADV(20) | robust |
| 5 | volatility | 20-period realized volatility | robust |
| 6 | momentum | 20-period price momentum | robust |
| 7 | relative_strength | RSI(14) | robust |
| 8 | close_position | (close - low) / (high - low) | none |
| 9 | intraday_range | (high - low) / close | robust |
| 10 | vol_regime | Volatility regime z-score | robust |
| 11 | trend_strength | Linear regression slope | robust |
| 12 | momentum_regime | Momentum regime (trending vs choppy) | none |
| 13 | atr_pct | ATR as % of close | robust |
| 14 | obv_pct | OBV percent change | robust |
| 15 | mfi | Money Flow Index | none |
| 16 | bb_percent_b | Bollinger %B (0-1) | none |
| 17 | macd_hist | MACD histogram / close | robust |
| 18 | sma_200_diff | (close - SMA200) / close | robust |
| 19 | amihud_illiq | Amihud illiquidity ratio | robust |
| 20 | spread_proxy | Implicit bid-ask spread | robust |
| 21 | return_autocorr | Return autocorrelation | none |
| 22 | spy_corr | Rolling correlation with SPY | none |
| 23 | spy_beta | Rolling beta vs SPY | robust |
| 24 | spy_rel_strength | Relative strength vs SPY | robust |

### 2.2 Multi-Timeframe Token Layout (P3)

```yaml
# config/generator.yaml
multi_timeframe:
  enabled: true
  resolutions: ["1h", "4h", "1d"]
```

Token mapping after MTF expansion (`feat_offset = 75`):

```
Tokens  0-24:  1h resolution  (return_1h, vwap_deviation_1h, ..., spy_rel_strength_1h)
Tokens 25-49:  4h resolution  (return_4h, vwap_deviation_4h, ..., spy_rel_strength_4h)
Tokens 50-74:  1d resolution  (return_1d, vwap_deviation_1d, ..., spy_rel_strength_1d)
Tokens 75+:    Operators       (ADD, SUB, MUL, DIV, ABS, SIGN, DELAY1, DELAY5,
                                TS_MEAN, TS_STD, TS_RANK, TS_MIN, TS_MAX, TS_CORR)
```

### 2.3 ALPS Population Structure

```rust
const ALPS_LAYER_MAX_AGES: [usize; 5] = [5, 13, 34, 89, 500];
const ALPS_LAYER_POP_SIZE: usize = 100;
const ALPS_NUM_LAYERS: usize = 5;
// Total population: 500 genomes
```

---

## 3. Root Cause Analysis

### 3.1 Uniform Random Sampling in Expanded Feature Space

The genetic algorithm uses **uniform random sampling** across all features for both genome initialization and mutation:

**Genome initialization** (`genetic.rs:102, 147-150`):
```rust
let features: Vec<usize> = (0..feat_offset).collect();  // ALL tokens 0..feat_offset
// ...
"FEAT" => {
    tokens.push(features[rng.gen_range(0..features.len())]);  // Uniform random
    stack_depth += 1;
}
```

**Point mutation** (`genetic.rs:494-510`, 40% of breeding operations):
```rust
if old < feat_offset {
    genome.tokens[idx] = rng.gen_range(0..feat_offset);  // Uniform random across ALL features
}
```

**Impact quantification:**

| Metric | 25-token (pre-P3) | 75-token (post-P3) | Degradation |
|--------|-------------------|---------------------|-------------|
| Per-selection probability of hitting specific useful factor | 1/25 = 4.0% | 1/75 = 1.33% | **-67%** |
| Mutation probability of improving a feature token | ~40-60% | ~13-20% | **-67%** |
| Expected useful features per random genome (len 3-12) | ~0.3 | ~0.1 | **-67%** |
| Layer 0 signal-to-noise ratio | baseline | ~1/3 baseline | **critical** |
| Effective search space | O(25^k) | O(75^k) | **exponential** |

For a typical genome of length 7 (midpoint of 3-12 range), the probability of generating a genome with at least 2 useful features from the "correct" timeframe:
- 25-token: ~45%
- 75-token: ~8%

### 3.2 Cross-Timeframe Feature Redundancy

Many factors are highly correlated across timeframes. For example:
- `return_1h` vs `return_4h` vs `return_1d` — same underlying price movement at different granularities
- `volatility_1h` vs `volatility_4h` — nested windows of the same realized vol
- `momentum_1h` vs `momentum_4h` — similar directional signals with lag

The GA has **no mechanism to recognize or penalize this redundancy**. A genome containing `[return_1h, return_4h, ADD]` adds noise rather than signal, but from the GA's perspective it's a valid combination.

### 3.3 Insufficient Complexity Penalty

Current penalty (`backtest/mod.rs:1644-1651`):

```rust
let token_len = genome.tokens.len();
let penalty_scale = (1000.0 / (len as f64).max(1000.0)).clamp(0.2, 1.0);
let complexity_penalty = if token_len > 8 {
    (token_len - 8) as f64 * 0.02 * penalty_scale
} else {
    0.0
};
let fitness = mean_psr - 0.5 * std_psr - complexity_penalty;
```

For a 12-token genome: `(12-8) * 0.02 * penalty_scale ≈ 0.04-0.08`

Against a typical successful PSR fitness of 1.0-3.0, this penalty is **only 2-4%** — insufficient to create selection pressure against bloated genomes that accumulate redundant cross-timeframe features.

### 3.4 No Feature Selection or Pruning Mechanism

Current mutation operators:
- **Point mutation (40%)**: Replaces token with uniform random — no feature importance weighting
- **Operator mutation (20%)**: Only changes operators, not features
- **Growth mutation (8%)**: Adds random feature + operator — increases bloat
- **Shrink mutation (20%)**: Removes unary operators only — **does not remove useless features**
- **Subtree replacement (10%)**: Full random regeneration — same uniform sampling problem

**Missing operators:**
- No "feature pruning" mutation that removes low-contribution feature tokens
- No weighted sampling based on historical feature success
- No cross-timeframe deduplication

### 3.5 Crossover Dilution

Single-point crossover (`genetic.rs:461-487`):
```rust
fn crossover(parent1: &Genome, parent2: &Genome, feat_offset: usize) -> Genome {
    let cut1 = cuts1[rng.gen_range(0..cuts1.len())];
    let cut2 = cuts2[rng.gen_range(0..cuts2.len())];
    let mut tokens = parent1.tokens[..cut1].to_vec();
    tokens.extend_from_slice(&parent2.tokens[cut2..]);
    if tokens.len() > 20 { tokens.truncate(20); }
    // ...
}
```

With 75 features, crossover is more likely to combine **incompatible timeframe features** from different parents (e.g., parent1 uses 1h features, parent2 uses 1d features), producing offspring that mix timeframes randomly rather than meaningfully.

### 3.6 TFT Cascade Effect

`too_few_trades` threshold (`backtest/mod.rs:2015-2025`):
```rust
let min_trades = 3_u32.max((trading_days / 10.0) as u32);
if trade_count < min_trades || (active_bars as f64) < (n as f64 * 0.05) {
    return SENTINEL_TOO_FEW_TRADES;  // -15.0
}
```

Noisier alpha signals from random 75-feature combinations produce weaker, more diffuse signals that fail to trigger enough trades. This creates a cascade:
1. More genomes get TFT sentinel (-15.0)
2. Population fitness variance increases
3. Tournament selection becomes less effective (winners are often just "less bad")
4. Promotion rate from Layer 0→1 drops
5. LLM Oracle triggers more frequently, but oracle-injected genomes also face the 75-token mutation dilution

---

## 4. Quantitative Impact Model

### 4.1 Search Efficiency Comparison

Assuming ~15 of 75 features are actually useful for a given symbol:

| Scenario | Useful/Total | P(useful per mutation) | Effective mutations per 100 | Expected gens to find good 5-token formula |
|----------|-------------|----------------------|---------------------------|------------------------------------------|
| 25-token | 15/25 | 60% | 60 | ~50-100 |
| 75-token | 15/75 | 20% | 20 | ~150-300+ |
| 75-token + weighted sampling (proposed) | 15/75 weighted | ~50% | 50 | ~60-120 |

### 4.2 Population Dynamics Under 75 Tokens

With 500 total genomes and uniform sampling:
- **Layer 0 (100 genomes, max age 5)**: Rapid turnover, mostly noise with 75 tokens
- **Layer 1 (100, age 13)**: Receives few promotions from noisy Layer 0
- **Layers 2-4**: Increasingly starved of quality candidates
- **Best genome**: Likely stuck at Layer 1-2 level quality

### 4.3 LLM Oracle Interaction

Current trigger thresholds:
```yaml
promotion_rate_threshold: 0.70   # L0→L1 promotion rate below 70% triggers oracle
tft_rate_threshold: 0.40         # TFT rate above 40% triggers oracle
tft_min_generation: 200          # Only after 200 generations
cooldown_gens: 50
cooldown_seconds: 600
```

Under 75 tokens, TFT rate likely exceeds 40% much earlier and more consistently, causing:
1. Frequent oracle invocations (every 50 generations after cooldown)
2. Oracle-generated formulas injected into Layer 0 face the same mutation dilution
3. Oracle effectiveness reduced — its carefully crafted formulas get corrupted by uniform random mutations within a few generations

---

## 5. Questions for Gemini Advisor

### 5.1 Architecture-Level Questions

**Q1: Feature Selection Strategy**
Given that 75 features contain significant redundancy (25 base factors x 3 correlated timeframes), what is the recommended approach for integrating feature selection into the ALPS genetic programming framework?

Options under consideration:
- **A) Pre-filter**: Compute Information Coefficient (IC) per feature per symbol, only include features with |IC| > threshold in the token pool
- **B) Weighted sampling**: Track feature usage frequency in top-performing genomes, bias sampling toward historically useful features
- **C) Hierarchical encoding**: Encode genome to first select timeframe, then select factor (reduces effective search to 3 + 25 instead of 75)
- **D) Adaptive feature pool**: Start with 25 tokens (best timeframe per symbol), gradually introduce cross-timeframe tokens as evolution progresses

Which approach best preserves the multi-timeframe information advantage while restoring search efficiency? Can multiple approaches be combined?

**Q2: Population Sizing**
With 75 tokens, should the ALPS population be scaled up? Current: 100 per layer x 5 layers = 500. Literature suggests population should scale with search space dimensionality. What is the recommended scaling formula for RPN-based genetic programming with 75 features and 14 operators?

**Q3: Complexity Penalty Calibration**
Current penalty: 0.02 per token above length 8. With 75 features, should this be:
- Increased (e.g., 0.05-0.10)?
- Made feature-count-aware (penalize diversity of timeframes used)?
- Replaced with a different regularization approach (e.g., MDL — Minimum Description Length)?

### 5.2 Genetic Operator Design Questions

**Q4: Feature Pruning Mutation**
We lack a mutation operator that removes useless features. Proposed design:
```
Feature Prune Mutation (15% probability):
1. For each feature token in genome:
   a. Temporarily remove it (and its dependent operator)
   b. Re-evaluate fitness on a fast proxy (e.g., IC against returns)
   c. If fitness unchanged or improved, keep the removal
2. Return pruned genome
```
Is this approach sound? What are the risks (computational cost, premature convergence)?

**Q5: Cross-Timeframe Crossover**
Should crossover be modified to be "timeframe-aware"? For example:
- Only combine parents that use the same primary timeframe
- Or: preserve parent1's timeframe structure, only swap the factor identities within the same timeframe from parent2

**Q6: Tournament Selection Pressure**
Current tournament size k=3. With 75 tokens producing noisier fitness landscapes, should k be increased to 5 or 7 to strengthen selection pressure? What are the tradeoffs with premature convergence?

### 5.3 Multi-Timeframe Strategy Questions

**Q7: Optimal Timeframe Combination**
The current setup uses [1h, 4h, 1d]. Is this the right set of resolutions for US equities (Polygon data)? Would [1h, 1d] (2 timeframes = 50 tokens) provide a better information-to-noise ratio than [1h, 4h, 1d] (75 tokens)?

The correlation between 1h and 4h factors is likely higher than between 1h and 1d. Removing the 4h resolution would:
- Reduce token space from 75 to 50 (-33%)
- Reduce cross-timeframe redundancy
- Still capture intraday (1h) and daily (1d) regime signals

**Q8: Phased Evolution**
Should multi-timeframe evolution be phased?
- Phase 1: Evolve with 25 tokens (single best timeframe per symbol)
- Phase 2: Introduce cross-timeframe tokens as "enhancement" mutations only
- Phase 3: Full 75-token evolution with population seeded from Phase 1-2 winners

This avoids cold-starting the 75-dimensional search but adds implementation complexity. Is the tradeoff worthwhile?

### 5.4 Fitness Function Questions

**Q9: Feature Diversity Penalty**
Should the fitness function penalize genomes that use features from multiple timeframes without a clear information gain? For example:
```
diversity_penalty = n_unique_timeframes_used * 0.05
```
This would create pressure toward timeframe-coherent strategies while still allowing cross-timeframe combinations that genuinely improve PSR.

**Q10: PSR Sensitivity to Feature Count**
The PSR (Bailey & Lopez de Prado 2012) z-score adjusts for skewness and kurtosis but NOT for the number of features used. With 75 features, the risk of data-snooping is higher. Should we adopt:
- **Deflated Sharpe Ratio (DSR)** which adjusts for the number of trials?
- **Haircut Sharpe Ratio** which accounts for multiple testing?
- Both would reduce fitness scores for strategies that emerged from a larger search space, providing a natural penalty for the 75-token regime.

### 5.5 P5 Interaction Questions

**Q11: HRP Sensitivity to Degraded Alpha**
P5's portfolio ensemble (HRP + dynamic weights) depends on the quality of underlying single-strategy alphas. If P3's 75-token expansion has degraded individual strategy quality:
- Does HRP's diversification benefit compensate for weaker individual alphas?
- Or does "garbage in → garbage out" apply, making P5 optimization futile until P3 search efficiency is fixed?

**Q12: Crowding Detection Under Degraded Strategies**
If the 75-token GA produces more homogeneous strategies (because the few survivors all discovered the same dominant signal), does the crowding penalty (`corr > 0.7`) become over-triggered? Should the crowding threshold be dynamically adjusted based on population diversity?

---

## 6. Proposed Remediation Priority

Based on our analysis, the recommended fix order (balancing impact vs. implementation complexity):

### Tier 1: Quick Wins (config changes + minor code changes)

1. **Increase complexity penalty** from 0.02 to 0.06 per token above 8
2. **Increase tournament size** from k=3 to k=5
3. **Consider reducing to 2 timeframes** [1h, 1d] = 50 tokens (requires validation)
4. **Increase Layer 0 population** from 100 to 150-200

### Tier 2: Moderate Changes (new genetic operators)

5. **Weighted feature sampling**: Track top-genome feature frequencies, bias sampling 70% toward historically successful features, 30% exploration
6. **Feature pruning mutation**: New 15% probability mutation that removes low-contribution feature tokens
7. **Timeframe-aware crossover**: Preserve timeframe coherence during recombination

### Tier 3: Architecture Changes

8. **Phased evolution**: Single-timeframe → cross-timeframe enhancement
9. **Deflated Sharpe Ratio**: Adjust PSR for multiple testing
10. **Adaptive feature pool**: Per-symbol, per-generation feature importance tracking

---

## 7. Appendix: Gemini P5 Evaluation Response

> (Gemini's complete evaluation preserved here for reference)

Gemini 对 P5 阶段的评价总结为以下核心观点:

1. **HRP 选型正确**: 遗传算法产生的策略池高度共线，HRP 不需要协方差矩阵求逆，避免了 Markowitz 在奇异矩阵附近的数值不稳定
2. **乘性因子设计优雅**: `Final Weight = HRP_weight * PSR_factor * Util_factor * (1 - Crowding_penalty)` 保留了风险平价约束
3. **防前瞻偏差设计严密**: 复用 psr_fitness 逻辑确保组合优化器看到的收益矩阵与策略演化环境一致
4. **代码工程成熟**: 零 I/O 纯函数模块 + 25 个边界条件单元测试
5. **建议引入 EWMA 协方差矩阵** 替代等权窗口以捕捉短期相关性突变
6. **P6 需要调仓死区 (Rebalance Deadband)** 避免微小权重变化产生不必要的交易成本

**我们的补充观点**: P5 的组合优化建立在底层单策略质量之上。当前 P3 因子扩展导致的 GA 搜索效率问题直接影响输入到 P5 的策略质量。**修复 P3 搜索效率问题的优先级应高于推进 P6**，否则组合优化是在低质量策略池上做优化，效果受限。

---

*Report generated for Gemini advisor technical consultation.*
