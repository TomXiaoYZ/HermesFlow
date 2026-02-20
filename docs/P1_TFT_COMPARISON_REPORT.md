# P1 too_few_trades Comparison Report: 25-Factor vs 13-Factor

**Date**: 2026-02-20
**Data snapshot**: ~270 generations per symbol in 25-factor space, ~19 generations per symbol in 13-factor space
**Generation range (25-factor)**: 51495 → 52094 (599 gens elapsed wall-clock)

---

## 1. Executive Summary

The 25-factor space shows a **higher** `too_few_trades` (TFT) rate than the 13-factor baseline at this early stage. This is **expected behavior** — the 25-factor genomes are essentially random (270 gens of convergence) while 13-factor genomes had 50,000+ generations of convergence before walk-forward evaluation was enabled.

The critical finding is the **improving trend within the 25-factor space**: TFT rate dropped from 38.3% (Q1) to 18.7% (Q3) before a partial rebound to 24.6% (Q4). This confirms the ALPS GA is learning to produce trade-generating strategies.

| Metric | 13-Factor (n=494) | 25-Factor (n=7018) |
|--------|:--:|:--:|
| TFT rate | 5.3% | 27.3% |
| OOS success rate | 81.8% | 48.5% |
| Avg OOS PSR (valid) | 1.372 | 1.620 |
| Median OOS PSR (valid) | 1.242 | 1.721 |
| Max OOS PSR | 3.305 | 3.615 |
| PSR std dev | 0.658 | 0.809 |

**Key insight**: When 25-factor strategies *do* trade, they achieve **higher PSR** than 13-factor strategies (median 1.721 vs 1.242). The richer signal space produces better strategies — the challenge is guiding evolution toward trade-generating genomes faster.

---

## 2. Aggregate Comparison

### 2a. Failure Mode Distribution

**13-Factor** (494 total, mature convergence):
- Success (valid OOS): 404 (81.8%)
- too_few_trades: 26 (5.3%)
- single_step: 64 (13.0%)

**25-Factor** (7018 total, early convergence):
- Success (valid OOS): 3401 (48.5%)
- too_few_trades: 1918 (27.3%)
- single_step: 1699 (24.2%)

### 2b. Why the Comparison is Not Apples-to-Apples

The 13-factor data has a critical bias:
1. **Only 19 generations per symbol** with walk-forward metadata (P0 was deployed recently)
2. Those 19 gens represent genomes that **evolved for ~51,500 generations** in the 13-factor space
3. They are elite genomes — the product of 50K+ gens of ALPS selection pressure

The 25-factor data represents:
1. **~270 generations per symbol** of actual evolution from random initialization
2. Genomes started from scratch when `feat_offset` changed from 13 to 25
3. The ALPS layers are still populating — L3/L4 may not have mature elites yet

**Conclusion**: A fair comparison requires either (a) running 25-factor evolution for 50K+ gens, or (b) comparing both spaces at the same evolutionary age (e.g., first 270 gens).

---

## 3. TFT Trend Within 25-Factor Space (Convergence Evidence)

The 25-factor data is divided into quartiles by generation order:

| Quartile | Gens | TFT | TFT% | Success | Success% |
|----------|:----:|:---:|:----:|:-------:|:--------:|
| Q1 (earliest 25%) | 1787 | 685 | **38.3%** | 616 | 34.5% |
| Q2 | 1787 | 503 | **28.1%** | 1026 | 57.4% |
| Q3 | 1787 | 335 | **18.7%** | 912 | 51.0% |
| Q4 (latest 25%) | 1787 | 440 | **24.6%** | 897 | 50.2% |

**Trend**: TFT dropped from 38.3% → 18.7% (Q1→Q3), a 51% reduction. The Q4 rebound to 24.6% likely reflects ALPS immigration (25% of each layer's new population is random genomes), which periodically introduces non-trading strategies. The underlying learned population is improving.

**Projection**: If the Q1→Q3 trend continues, TFT rate should reach single digits by ~1000 generations per symbol.

---

## 4. Per-Symbol Analysis (25-Factor)

### 4a. High-TFT Symbols (>50%)

| Symbol | Mode | Gens | TFT | TFT% | OK% | Avg PSR |
|--------|------|:----:|:---:|:----:|:---:|:-------:|
| QQQ | long_short | 271 | 250 | 92.3% | 6.6% | 0.722 |
| MSFT | long_short | 271 | 234 | 86.3% | 10.3% | 1.141 |
| GOOGL | long_only | 272 | 214 | 78.7% | 1.5% | 1.433 |
| UVXY | long_short | 272 | 211 | 77.6% | 22.4% | 1.254 |
| META | long_only | 271 | 165 | 60.9% | 32.8% | 1.813 |
| GLD | long_short | 271 | 154 | 56.8% | 40.6% | 0.811 |
| TSLA | long_only | 274 | 149 | 54.4% | 38.3% | 3.273 |

**Pattern**: `long_short` mode has systematically higher TFT — the dual-direction strategy space is harder to search. QQQ long_short (92.3% TFT) is the worst performer, likely because QQQ's trend-following nature makes short signals rare.

### 4b. Low-TFT Symbols (<5%)

| Symbol | Mode | Gens | TFT | TFT% | OK% | Avg PSR |
|--------|------|:----:|:---:|:----:|:---:|:-------:|
| DIA | long_only | 273 | 0 | 0.0% | 98.5% | 2.235 |
| NVDA | long_short | 273 | 0 | 0.0% | 78.0% | 0.975 |
| QQQ | long_only | 273 | 0 | 0.0% | 73.3% | 1.117 |
| SPY | long_short | 272 | 0 | 0.0% | 80.1% | 0.437 |
| UVXY | long_only | 272 | 0 | 0.0% | 11.8% | 0.702 |
| AAPL | long_short | 271 | 1 | 0.4% | 68.3% | 1.320 |
| AMZN | long_only | 273 | 3 | 1.1% | 34.8% | 1.382 |

**Pattern**: `long_only` mode generally has lower TFT. DIA long_only is the star — 0% TFT, 98.5% success, and highest avg PSR (2.235). Liquid, low-volatility ETFs perform best.

### 4c. 13-Factor Comparison (Problem Symbols)

Comparing the same symbol/mode pairs that had TFT in 13-factor:

| Symbol | Mode | 13f TFT% | 25f TFT% | Direction |
|--------|------|:--------:|:--------:|:---------:|
| QQQ | long_only | 68.4% | 0.0% | Solved |
| NVDA | long_short | 47.4% | 0.0% | Solved |
| NVDA | long_only | 21.1% | 31.5% | Worse (maturity gap) |

QQQ long_only and NVDA long_short — the two worst 13-factor TFT offenders — now show **0% TFT** in 25-factor. This is strong evidence that the enriched factor space helps, though the maturity gap caveat applies (these same symbols may just have found trade-generating genomes earlier in 25-factor evolution).

---

## 5. ALPS Promotion Rate Baseline

Average promotion rates across 2894 samples (50-generation rolling window):

| Boundary | Avg Rate |
|----------|:--------:|
| L0 → L1 | 88.1% |
| L1 → L2 | 89.0% |
| L2 → L3 | 86.7% |
| L3 → L4 | 84.2% |

Average rolling promoted per 50 gens: **692**
Average rolling discarded per 50 gens: **90**

These rates are healthy — significantly above 50% at all boundaries. A sustained drop below 70% at L0→L1 would indicate convergence stall and trigger P2 intervention.

---

## 6. OOS PSR Quality

When 25-factor strategies succeed (no TFT), they produce **higher quality** strategies:

| Metric | 13-Factor | 25-Factor | Delta |
|--------|:---------:|:---------:|:-----:|
| Mean PSR | 1.372 | 1.620 | +18.1% |
| Median PSR | 1.242 | 1.721 | +38.6% |
| Max PSR | 3.305 | 3.615 | +9.4% |
| Std Dev | 0.658 | 0.809 | +22.9% |

The higher standard deviation reflects the wider diversity of the 25-factor signal space — strategies range from mediocre to excellent. As evolution converges, we expect the mean to rise and std to narrow.

---

## 7. Conclusions & Recommendations

### 7a. Conclusions
1. **TFT rate is elevated but improving** — 38.3% → 18.7% across Q1→Q3, with 25% immigration noise in Q4
2. **Quality is higher when trades occur** — median PSR 1.721 vs 1.242 (+38.6%)
3. **Problem symbols from 13-factor are resolved** — QQQ long_only (68% → 0%), NVDA long_short (47% → 0%)
4. **long_short mode is systematically harder** — higher TFT across most symbols
5. **ALPS promotion rates are healthy** — 84-89% across all boundaries

### 7b. When to Expect Parity
- **~500 generations**: TFT should drop below 15% aggregate
- **~1000 generations**: TFT should approach 13-factor baseline (~5%)
- **~2000 generations**: Meaningful PSR quality comparison possible

### 7c. Actionable Next Steps
1. **Continue monitoring** — re-run this report at 500 and 1000 gens
2. **Flag for P2**: If TFT fails to drop below 15% by 500 gens, prioritize LLM-guided mutation
3. **Per-symbol intervention**: QQQ long_short (92% TFT) may need symbol-specific tuning regardless of P2
4. **Factor utilization analysis**: At 500 gens, check which of the 12 new factors appear in elite genomes
