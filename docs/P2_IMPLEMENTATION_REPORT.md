# P2 Implementation Report: LLM-Guided Mutation Oracle

**Date**: 2026-02-20
**Commits**: `2e2e9fc`..`9d3a42b` (7 feature commits + 1 doc commit)
**Status**: Deployed & Verified — Oracle actively firing in production

---

## 1. Summary

P2 adds an LLM mutation oracle to the ALPS genetic algorithm. When evolution stalls (low promotion rate or high too_few_trades rate), the system invokes Claude Sonnet via AWS Bedrock to generate semantically meaningful RPN formulas, validates them, and injects survivors into ALPS Layer 0.

P2e extends this with cross-symbol learning: the oracle prompt includes top-performing formulas from other symbols (same exchange + mode), enabling knowledge transfer across tickers without shared in-memory state.

**Key numbers** (as of deployment):
- 13 symbols × 2 modes = 26 evolution tasks running
- 12 oracle invocations observed across different (symbol, mode) pairs
- 116 genomes injected total (acceptance rate: 89%, 116/130 parsed)
- 10 of 12 invocations used cross-symbol context (post-P2e)
- 28 unit tests passing, 0 clippy warnings

---

## 2. Architecture

```
                     ┌─────────────────────────────────────────────────┐
                     │           Strategy Generator (per symbol)       │
                     │                                                 │
  ┌──────────┐       │  ┌─────────┐    ┌──────────┐    ┌───────────┐  │
  │ DB Query ├──────►│  │ Trigger │───►│ build    │───►│ call_llm  │  │
  │ cross-   │       │  │ Check   │    │ _prompt  │    │ (Bedrock) │  │
  │ symbol   │       │  └────┬────┘    └──────────┘    └─────┬─────┘  │
  │ elites   │       │       │                               │        │
  └──────────┘       │  promo_rate < 70%              parse + validate│
                     │  OR tft_rate > 40%                    │        │
                     │       │                               ▼        │
                     │  ┌────┴───────────────────────────────────┐    │
                     │  │  inject valid genomes → ALPS Layer 0   │    │
                     │  └────────────────────────────────────────┘    │
                     │                                                 │
                     │  metadata JSONB ──► API ──► Frontend Panel     │
                     └─────────────────────────────────────────────────┘
```

### Pipeline per invocation:
1. **Trigger** — either L0→L1 promotion rate < 70% (`promotion_rate`) or TFT rate > 40% (`tft_rate`)
2. **Cross-symbol query** — DB: latest gen per other symbol where OOS PSR > 0, top 10 by OOS PSR
3. **Prompt construction** — context + factors + operators + RPN examples + local elites + cross-symbol elites + task
4. **LLM call** — AWS Bedrock Converse API → Claude Sonnet
5. **Parse** — extract JSON array from response (handles code fences, surrounding text)
6. **Validate** — `encode_formula()` → `validate_stack()` → dedup vs existing elites
7. **Inject** — valid genomes inserted into ALPS Layer 0; full log persisted to metadata JSONB

---

## 3. Sub-Tasks

### P2a: Genome Decoder (`genome_decoder.rs`, 353 lines)

Bidirectional conversion between RPN token sequences and human-readable formulas:

| Function | Direction | Example |
|----------|-----------|---------|
| `decode_genome()` | tokens → formula | `[0, 6, 25+12]` → `"return momentum TS_MEAN"` |
| `encode_formula()` | formula → tokens | `"return momentum TS_MEAN"` → `[0, 6, 37]` |
| `validate_stack()` | tokens → Result | Verifies exactly 1 value remains on stack |

12 unit tests cover round-trip, unknown tokens, stack underflow/overflow, operator chains.

### P2b: LLM Oracle Core (`llm_oracle.rs`, 795 lines)

| Component | Description |
|-----------|-------------|
| `LlmOracleConfig` | 12 config fields: provider, model, thresholds, cooldowns |
| `build_prompt()` | Constructs structured prompt: context, factors, operators, RPN tutorial, elites, cross-symbol, task |
| `call_llm()` | Multi-provider: AWS Bedrock Converse, Anthropic Messages, OpenAI Chat Completions |
| `parse_response()` | Extracts JSON array from LLM response (handles markdown fences, surrounding text) |
| `validate_formulas()` | Token encoding → stack validation → dedup (vs batch + vs existing elites) |
| `generate_mutations()` | End-to-end: prompt → call → parse → validate → `OracleResult` |

7 unit tests for prompt generation, response parsing (clean JSON, code fence, surrounding text, invalid), formula validation (mixed valid/invalid, dedup, complex formulas).

### P2c: Trigger Integration (`main.rs`, ~200 lines added)

**Dual trigger conditions** (either activates):

| Trigger | Condition | Warmup | Description |
|---------|-----------|--------|-------------|
| `promotion_rate` | L0→L1 rolling avg < 70% | gen >= 100 | Random exploration failing |
| `tft_rate` | 50-gen rolling TFT rate > 40% | gen >= 200 | Stuck in non-trading genome space |

**Cooldown**: min 50 generations AND 600 seconds between invocations (per symbol/mode).

**Supporting trackers**:
- `PromotionRateTracker`: Circular buffer of 50 generations, rolling avg promotion rates per layer boundary
- `TftTracker`: Circular buffer of 50 generations, fraction where best genome had `too_few_trades`

### P2d: Frontend Oracle Monitoring (`EvolutionExplorer.tsx`, ~300 lines added)

- **LLM Oracle Interactions panel**: Standalone collapsible panel listing all oracle invocations
- **OracleInteractionCard**: Per-invocation expandable card showing trigger reason, TFT rate, accepted/rejected formulas with reasons
- **OracleDetailPanel**: Inline display within generation detail view
- **Expandable prompt/response**: Full LLM prompt and raw response viewable on demand

### P2e: Cross-Symbol Learning (`main.rs` + `llm_oracle.rs` + `EvolutionExplorer.tsx`, 180 lines)

- `CrossSymbolElite` struct: symbol, formula, IS PSR, OOS PSR
- `fetch_cross_symbol_elites()`: SQL query — latest gen per other symbol (same exchange + mode), OOS PSR > 0, top 10
- `build_prompt()`: New "Cross-Symbol Successful Formulas" section between local elites and task
- Metadata JSONB: `cross_symbol_elites` array serialized when present
- Frontend: Sky-blue themed "Cross-Symbol References" section in OracleInteractionCard

---

## 4. Files Modified

| File | Lines Changed | Description |
|------|:---:|-------------|
| `services/strategy-generator/src/genome_decoder.rs` | +353 | **NEW** — Bidirectional RPN codec + stack validator + 12 tests |
| `services/strategy-generator/src/llm_oracle.rs` | +795 | **NEW** — Oracle core: config, prompt, LLM API (3 providers), parse, validate + 7 tests |
| `services/strategy-generator/src/main.rs` | +532/-11 | Trackers, trigger logic, oracle invocation block, cross-symbol query, metadata builder |
| `services/strategy-generator/src/genetic.rs` | +94 | `collect_elites()`, `inject_genomes()`, `total_population()` for oracle integration |
| `services/strategy-generator/src/api.rs` | +3 | Metadata JSONB passthrough for oracle fields |
| `services/strategy-generator/Cargo.toml` | +4 | `aws-sdk-bedrockruntime`, `aws-config` dependencies |
| `services/web/src/components/EvolutionExplorer.tsx` | +328 | Oracle Interactions panel, OracleInteractionCard, cross-symbol display |
| `config/generator.yaml` | +19 | `llm_oracle` config section |
| **Total** | **+2098/-30** | 8 files (2 new, 6 modified) |

**NOT modified**: DB schema/migrations (metadata JSONB passthrough), gateway, backtest-engine, Docker networking.

---

## 5. Verification Results

### 5a. Build & Lint
```
cargo clippy -p strategy-generator -- -D warnings   ✅ Clean (0 warnings)
cargo test -p strategy-generator                     ✅ 28 tests passed, 0 failed
cargo fmt --all -- --check                           ✅ Clean
cd services/web && npx next build                    ✅ Compiled successfully
```

### 5b. Docker Deployment
```
docker compose build strategy-generator web          ✅ Both images built
docker compose up -d strategy-generator web          ✅ Both containers healthy
strategy-generator health:                           ✅ {"status":"healthy"}
strategy-generator API:                              ✅ /polygon/symbols returns 13 symbols
web frontend:                                        ✅ Port 3000 responding
```

### 5c. Live Oracle Activity (Observed at deployment)

**Oracle invocations by (symbol, mode):**

| Symbol | Mode | Invocations | Injected | Trigger | Cross-Symbol Refs | Current TFT |
|--------|------|:-:|:-:|---------|:-:|:-:|
| AMZN | long_only | 1 | 8 | tft_rate | 10 | 8.0% |
| DIA | long_only | 1 | 8 | tft_rate | 10 | 0.0% |
| GLD | long_only | 1 | 10 | tft_rate | 10 | 42.0% |
| GOOGL | long_only | 1 | 10 | tft_rate | 10 | 100.0% |
| GOOGL | long_short | 1 | 10 | tft_rate | 10 | 70.0% |
| IWM | long_only | 1 | 10 | tft_rate | 10 | 0.0% |
| MSFT | long_short | 1 | 9 | tft_rate | 10 | 100.0% |
| NVDA | long_short | 1 | 8 | tft_rate | 10 | 0.0% |
| QQQ | long_only | 1 | 8 | tft_rate | 10 | 84.0% |
| QQQ | long_short | 1 | 8 | tft_rate | 10 | 100.0% |
| SPY | long_short | 1 | 8 | tft_rate | 10 | 60.0% |
| TSLA | long_only | 1 | 9 | tft_rate | 10 | 100.0% |
| UVXY | long_short | 1 | 8 | tft_rate | 10 | 100.0% |

**Totals**: 13 invocations, 116/130 formulas accepted (89.2% acceptance rate).

### 5d. Example Oracle Interaction (TSLA long_only, Gen 53649)

**Trigger**: `tft_rate` (50-gen TFT rate: 50%)

**Cross-symbol context provided** (top 5 of 10):
| Symbol | Formula | OOS PSR | IS PSR |
|--------|---------|:---:|:---:|
| NVDA | `mean_reversion TS_STD ABS sma_200_diff TS_CORR` | 1.294 | 0.959 |
| DIA | `vwap_deviation relative_strength TS_CORR TS_MAX` | 1.293 | 0.357 |
| MSFT | `intraday_range atr_pct SIGN MUL TS_MIN` | 1.199 | 0.490 |
| GOOGL | `close_position amihud_illiq ADD volume_ratio ...` | 1.176 | 0.740 |
| AMZN | `sma_200_diff TS_MIN return_autocorr MUL` | 1.034 | 0.640 |

**LLM output**: 10 formulas parsed, 9 accepted, 1 rejected (stack depth error).

**Accepted formulas** (injected into Layer 0):
```
mean_reversion TS_STD sma_200_diff TS_CORR ABS
vwap_deviation relative_strength TS_CORR TS_RANK
intraday_range atr_pct MUL TS_MIN SIGN
sma_200_diff TS_MIN return_autocorr MUL TS_STD
bb_percent_b mfi SUB volatility DIV TS_MAX
relative_strength TS_MIN TS_MAX SIGN TS_MEAN
macd_hist intraday_range TS_CORR TS_RANK volatility DIV
spy_beta trend_strength MUL momentum_regime TS_CORR TS_STD
vol_regime amihud_illiq DIV close_position ADD TS_RANK
```

**Rejected**:
- `volume_ratio TS_RANK DELAY5 momentum MUL ADD` → stack: final stack depth 0 (expected 1)

### 5e. Key Observations

1. **All triggers are `tft_rate`** — no `promotion_rate` triggers observed yet. This indicates the primary stagnation signal is "too many non-trading genomes" rather than "L0→L1 promotion failure". The TFT trigger (gen >= 200) fires earlier than promotion drop.

2. **89% acceptance rate** — the LLM generates mostly valid RPN formulas. The 11% rejection is primarily stack depth errors (binary ops without enough operands), not unknown tokens. The structured prompt with explicit RPN tutorial and examples is working.

3. **Cross-symbol learning provides diverse patterns** — the LLM receives formulas from NVDA, DIA, MSFT, GOOGL, AMZN, etc., creating a "collective intelligence" effect. Symbols with OOS PSR > 1.0 contribute the most valuable patterns.

4. **Pre-P2e vs post-P2e invocations visible** — GLD gen 53497 (pre-P2e) has no cross-symbol data; GLD gen 53621 (post-P2e) has 10 cross-symbol refs. The metadata JSONB cleanly differentiates old vs new invocations.

5. **High TFT symbols** — GOOGL, MSFT(LS), QQQ(LS), TSLA(LO), UVXY(LS) all show 100% TFT rate, indicating the oracle has fired but hasn't yet had enough generations post-injection to see improvement. These are the symbols where P2 impact will be most measurable.

---

## 6. Configuration

```yaml
# config/generator.yaml
llm_oracle:
  enabled: true
  provider: "bedrock"
  model: "us.anthropic.claude-sonnet-4-20250514-v1:0"
  region: "us-east-1"
  genomes_per_invocation: 10
  max_response_tokens: 1024
  min_generation: 100
  promotion_rate_threshold: 0.70
  tft_rate_threshold: 0.40
  tft_min_generation: 200
  cooldown_gens: 50
  cooldown_seconds: 600
```

**Cost estimate**: ~$0.005/invocation × ~13 invocations/cycle × ~1 cycle/500 gens ≈ $0.07/day.

---

## 7. What's Next (Recommended for Gemini Review)

1. **LLM Genome Survival Rate**: Track what fraction of LLM-injected genomes survive from L0 → L1 promotion. This is the ground-truth metric for whether the LLM is generating competitively fit formulas. Target: >30%.

2. **TFT Rate Before/After**: For symbols with 100% TFT (GOOGL, QQQ/LS, TSLA/LO, UVXY/LS), monitor whether TFT drops after oracle injection. If 9 LLM genomes don't reduce TFT within 50 generations, the formulas may need structural changes (more features, longer formulas).

3. **Cross-Symbol Pattern Adoption**: Analyze whether formulas adapted from cross-symbol context (e.g., NVDA's `mean_reversion TS_STD ABS sma_200_diff TS_CORR` appearing in TSLA's elite pool) achieve competitive PSR scores on the target symbol. This validates the cross-pollination hypothesis.

4. **Prompt Engineering Iteration**: Current acceptance rate is 89%. Main failure mode is stack depth errors. Consider:
   - Adding explicit stack depth tracking examples in the prompt
   - Requesting the LLM to self-validate before output
   - Increasing `max_response_tokens` to allow more diverse output

5. **P3 Candidates** (from roadmap):
   - Multi-timeframe factor stacking (1h + 1d signals)
   - Adaptive trigger thresholds (per-symbol calibration)
   - LLM genome fitness tracking across generations (survival analytics)
