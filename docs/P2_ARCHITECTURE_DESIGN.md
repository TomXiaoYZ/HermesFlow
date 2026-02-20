# P2 Architecture Design: LLM-Guided Mutation

**Date**: 2026-02-20
**Status**: Design Phase
**Prerequisite**: P1 factor enrichment deployed (25-factor space operational)

---

## 1. Problem Statement

The ALPS GA explores the 25-factor × 14-operator search space via random mutation and crossover. While effective, the mutation operators are blind — they have no domain knowledge about which operator combinations produce meaningful financial signals. This leads to:

1. **High wastage**: ~50% of random genomes produce `too_few_trades` in early generations
2. **Slow convergence**: Random walks through genome space miss structured patterns (e.g., "momentum divergence" = `momentum ts_mean sub`)
3. **No learning transfer**: Each symbol/mode evolves independently; discoveries in NVDA don't inform AAPL

P2 adds an LLM "meta-layer" that analyzes elite genomes and generates semantically meaningful mutations, acting as a domain-aware mutation operator alongside the existing random operators.

---

## 2. Architecture Overview

```
┌─────────────────────────────────────────────────┐
│                  Evolution Loop                  │
│                                                  │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐  │
│  │  ALPS GA  │───>│ Evaluate │───>│  Select  │  │
│  │  Evolve   │    │  Fitness │    │  Promote │  │
│  └─────┬────┘    └──────────┘    └──────────┘  │
│        │                                         │
│        │ Every N gens (trigger)                  │
│        ▼                                         │
│  ┌─────────────────────────┐                    │
│  │   LLM Mutation Oracle   │                    │
│  │                         │                    │
│  │  1. Collect elite pool  │                    │
│  │  2. Decode to formulas  │                    │
│  │  3. Prompt LLM          │                    │
│  │  4. Parse new genomes   │                    │
│  │  5. Inject into Layer 0 │                    │
│  └─────────────────────────┘                    │
│                                                  │
└─────────────────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────────────────┐
│           LLM Provider (External)                │
│  Claude / GPT-4 / local Mistral via API          │
│  Input: elite genome formulas + fitness stats    │
│  Output: new RPN token sequences                 │
└─────────────────────────────────────────────────┘
```

---

## 3. Trigger Conditions

The LLM oracle is invoked **only when evolution stalls**, not every generation. This minimizes API cost and avoids disrupting healthy convergence.

### 3a. Primary Trigger: Promotion Rate Drop
```
IF avg_promotion_rate(L0→L1, window=50) < 0.70
   AND generation > 100  -- allow initial warmup
THEN trigger LLM mutation
```

**Rationale**: L0→L1 promotion rate reflects whether fresh random genomes can compete with L1 incumbents. A drop below 70% (baseline ~88%) signals that random exploration is no longer productive — the search space needs guided assistance.

### 3b. Secondary Trigger: TFT Stagnation
```
IF too_few_trades_rate(window=50) > 0.40
   AND generation > 200
THEN trigger LLM mutation
```

**Rationale**: Persistent high TFT rate despite evolution means the GA is stuck in non-trading regions of genome space.

### 3c. Cooldown
- Minimum **50 generations** between LLM invocations per (symbol, mode)
- Maximum **1 invocation per 10 minutes** per (symbol, mode) to bound API cost
- If LLM-generated genomes all score fitness < L0 median after 10 gens, back off to 100-gen cooldown

---

## 4. LLM Mutation Oracle Design

### 4a. Elite Pool Collection

```rust
struct ElitePool {
    /// Top 5 genomes from each ALPS layer (25 total)
    elites: Vec<(usize, Genome)>,  // (layer_idx, genome)
    /// Symbol-level context
    symbol: String,
    mode: String,
    /// Performance summary
    best_oos_psr: f64,
    current_tft_rate: f64,
    generation: usize,
}
```

### 4b. Genome → Formula Decoding

Convert RPN tokens back to human-readable formula for the LLM:

```
Tokens: [0, 3, 17, 2, 12, 30]
Decoded: "close high TS_MAX MUL TS_MEAN ADD"
Readable: "ADD(TS_MEAN(close), MUL(high, TS_MAX(close)))"
```

The decoder walks the token array, maps each token to its name (feature name or operator name), and optionally produces infix notation for readability.

### 4c. Prompt Template

```
You are a quantitative finance researcher designing alpha factors for US equities.

## Context
- Symbol: {symbol} ({mode} mode)
- Generation: {generation}
- Current best OOS PSR: {best_oos_psr}
- too_few_trades rate: {tft_rate}%
- Available features (25): {feature_list}
- Available operators: {operator_list}

## Current Elite Formulas (top 5 by OOS PSR)
1. {formula_1} → PSR: {psr_1}
2. {formula_2} → PSR: {psr_2}
...

## Task
Generate 10 new alpha factor formulas that:
1. Are different from the elites above
2. Combine features in financially meaningful ways
3. Produce trading signals (avoid formulas that output constants)
4. Use the RPN notation: feature names and operator names separated by spaces

Output as JSON array of strings, each string is an RPN formula.
Example: ["close momentum SUB ABS", "volume_ratio TS_MEAN close MUL"]
```

### 4d. Response Parsing & Validation

1. Parse JSON array of RPN strings from LLM response
2. Tokenize each formula: map feature/operator names back to token indices
3. **Validate stack correctness**: simulate stack depth, reject formulas that underflow or leave stack > 1
4. **Validate length**: reject formulas > 20 tokens
5. **Dedup**: reject formulas identical to existing elites
6. Convert valid formulas to `Genome` structs with `age=0, fitness=0.0`

### 4e. Injection Strategy

- Inject validated genomes into **Layer 0** (youngest layer)
- Replace random immigrants (the 25% immigration slot in the evolution step)
- Cap at **10 LLM-generated genomes per invocation** (out of 100 L0 population)
- Tag injected genomes with `source: "llm"` in a metadata field for tracking

---

## 5. Implementation Plan

### 5a. New Files

| File | Purpose |
|------|---------|
| `services/strategy-generator/src/llm_oracle.rs` | LLM mutation oracle: prompt building, API call, response parsing |
| `services/strategy-generator/src/genome_decoder.rs` | RPN token ↔ formula string conversion |

### 5b. Modified Files

| File | Changes |
|------|---------|
| `services/strategy-generator/src/main.rs` | Trigger detection, oracle invocation, genome injection |
| `services/strategy-generator/src/genetic.rs` | Add `inject_genomes(layer: usize, genomes: Vec<Genome>)` method to AlpsGA |
| `services/strategy-generator/Cargo.toml` | Add `reqwest` (HTTP client for LLM API) |
| `config/generator.yaml` | LLM oracle config (endpoint, model, trigger thresholds, cooldown) |

### 5c. Configuration

```yaml
llm_oracle:
  enabled: false  # Feature flag, default off
  provider: "anthropic"  # or "openai", "local"
  endpoint: "${LLM_ORACLE_ENDPOINT}"
  api_key: "${LLM_ORACLE_API_KEY}"
  model: "claude-sonnet-4-20250514"
  trigger:
    min_generation: 100
    promotion_rate_threshold: 0.70
    tft_rate_threshold: 0.40
    cooldown_gens: 50
    cooldown_seconds: 600
  injection:
    genomes_per_invocation: 10
    target_layer: 0
    max_retries: 2
```

### 5d. Genome Decoder Implementation

```rust
// genome_decoder.rs

/// Map token index to human-readable name.
pub fn token_to_name(token: usize, feat_offset: usize, factor_names: &[String]) -> String {
    if token < feat_offset {
        factor_names[token].clone()
    } else {
        let op_idx = token - feat_offset;
        match op_idx {
            0 => "ADD", 1 => "SUB", 2 => "MUL", 3 => "DIV",
            5 => "ABS", 6 => "SIGN", 10 => "DELAY1", 11 => "DELAY5",
            12 => "TS_MEAN", 13 => "TS_STD", 14 => "TS_RANK",
            16 => "TS_CORR", 17 => "TS_MIN", 18 => "TS_MAX",
            _ => "UNKNOWN",
        }.to_string()
    }
}

/// Convert genome tokens to RPN formula string.
pub fn decode_genome(tokens: &[usize], feat_offset: usize, factor_names: &[String]) -> String {
    tokens.iter()
        .map(|&t| token_to_name(t, feat_offset, factor_names))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Parse RPN formula string back to token indices.
pub fn encode_formula(formula: &str, feat_offset: usize, factor_names: &[String]) -> Option<Vec<usize>> {
    // Reverse lookup: name → token index
    // Validate stack correctness
    // Return None if invalid
}
```

### 5e. Cost Estimation

| Metric | Estimate |
|--------|----------|
| Prompt size | ~800 tokens (5 elite formulas + context) |
| Response size | ~400 tokens (10 RPN formulas in JSON) |
| Cost per invocation (Claude Sonnet) | ~$0.005 |
| Invocations per day (26 tasks × every 50 gens × ~10 gens/hour) | ~50 |
| **Daily cost** | **~$0.25** |

Negligible cost relative to the compute and market data expenses of running the evolution system.

---

## 6. Cross-Symbol Learning (P2.5 Extension)

### 6a. Concept

When the LLM generates a successful genome for NVDA (e.g., `close momentum SUB TS_MEAN volume_ratio MUL`), this formula may be relevant for other high-momentum stocks (TSLA, META). P2.5 extends the oracle to include cross-symbol elite sharing:

```
## Cross-Symbol Context
Successful formulas from similar symbols:
- NVDA: "close momentum SUB TS_MEAN volume_ratio MUL" → PSR 2.1
- TSLA: "return TS_STD close TS_MEAN DIV" → PSR 1.8

## Task
Adapt these patterns for {target_symbol}, considering:
- {target_symbol}'s sector, volatility profile, liquidity
```

### 6b. Similarity Clustering

Group symbols by factor correlation profile for cross-pollination:
- **Tech momentum**: NVDA, TSLA, META, AMZN, GOOGL, MSFT, AAPL
- **Index/ETF**: SPY, QQQ, DIA, IWM
- **Commodities**: GLD
- **Volatility**: UVXY

---

## 7. Evaluation Criteria (Success Metrics)

| Metric | Baseline (no LLM) | Target (with LLM) |
|--------|:--:|:--:|
| TFT rate at 500 gens | ~15% (projected) | <8% |
| TFT rate at 1000 gens | ~5% (projected) | <2% |
| Mean OOS PSR | 1.62 | >1.80 |
| L0→L1 promotion rate | 88% | >85% (maintained) |
| LLM genome survival to L1 | N/A | >30% |
| LLM genome in top-10 elite | N/A | >10% |

---

## 8. Risk Mitigation

| Risk | Impact | Mitigation |
|------|--------|------------|
| LLM generates invalid RPN | Wasted API call | Stack validation + retry (max 2) |
| LLM hallucinates operator names | Parse failure | Strict name→token mapping, reject unknowns |
| LLM output lacks diversity | Minimal improvement | Temperature tuning, diverse prompt sampling |
| API rate limits / outages | Evolution stalls | Feature flag off = pure GA (graceful degradation) |
| LLM-generated genomes overfit | High IS, low OOS | Walk-forward OOS evaluation catches this automatically |
| Cost escalation | Budget overrun | Per-symbol daily cap, cooldown enforcement |

---

## 9. Implementation Phases

| Phase | Scope | Dependency |
|-------|-------|------------|
| **P2a**: Genome decoder | `genome_decoder.rs` — token↔formula conversion + tests | None |
| **P2b**: LLM oracle core | `llm_oracle.rs` — prompt building, API call, response parsing | P2a |
| **P2c**: Trigger integration | Wire into `main.rs` evolution loop with feature flag | P2a, P2b |
| **P2d**: Monitoring & logging | Track LLM genome survival, injection stats in metadata | P2c |
| **P2e**: Cross-symbol learning | Cross-symbol elite sharing in prompts | P2c + data |
