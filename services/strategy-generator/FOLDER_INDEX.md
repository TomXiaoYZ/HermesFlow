# strategy-generator - FOLDER_INDEX

> Genetic algorithm strategy evolution engine. Uses ALPS population structure, PSR fitness, walk-forward validation, LLM-guided mutation, MCTS symbolic regression, MAP-Elites subformula archive, and portfolio ensemble optimization.

## Module Map

```
src/
  main.rs              # Entry point: config load, DB init, evolution loop spawn, API server
                       #   P7-1A: MctsYamlConfig + LfdrConfig deserialization from YAML
                       #   P7-1B: Dedicated MCTS Rayon thread pool (2 threads default)
                       #   P7-1C: MCTS integration into evolution loop (inject seeds → ALPS L0)
                       #   P7-2B: CCIPCA diagnostic (lazy init, zero-copy ArrayView)
                       #   P7-3A: Removed hardcoded DATABASE_URL fallback
                       #   P7-5B: Genome diversity (Hamming) logging every 50 gens
                       #   P8-0E: UniformPolicy → LlmCachedPolicy; importance recompute every 500 gens
                       #   P8-1B: CCIPCA augmentation 75→80 features (PC0..PC4) after 200 obs
                       #   P8-2B: DiversityTriggerConfig + active L3/L4 intervention + elitist cull
                       #   P8-4C: Removed P7-3B 16MiB guard (sqlx 0.8 fixes RUSTSEC-2024-0363)
                       #   P9-1B: DiversityTrigger topology mutation (replaces brute-force threshold reset)
                       #   P9-1C: Ensemble rebalance gate (should_trigger_rebalance + OOS PSR queries)
                       #   P9-2A: SubformulaArchive init + MCTS ingest into MAP-Elites buckets
                       #   P9-3A: Causal verification wiring (bottom-5 suspicious, top-5 causal)
  genetic.rs           # Core GA: ALPS layers, selection, crossover, mutation, promotion
                       #   - ALPS: 5 layers [5,13,34,89,500 gens], 100 genomes/layer
                       #   - Dual-mode: long_only + long_short per (exchange, symbol)
                       #   - Operator pruning: 14/23 opcodes for new genomes
                       #   P7-5B: hamming_distance() + layer_diversity() metrics
                       #   P8-2C: layer_size(), generate_random_genomes(), cull_weakest()
                       #   P9-1B: DiversityTrigger zero-trade deadlock handling
  genome_decoder.rs    # Decodes integer genome → RPN formula string (human-readable)
  factor_loader.rs     # Loads factor definitions from YAML config
  llm_oracle.rs        # LLM-guided mutation (P2): Bedrock/Claude generates RPN formulas
                       #   - Triggered on stagnation (low promotion rate or high TFT rate)
                       #   - Cross-symbol learning: includes top formulas from other symbols
                       #   P9-3A: LLM downgraded to hypothesis generator (no longer final judge)
  api.rs               # HTTP API: evolution status, backtest results, strategy queries
  health.rs            # /health endpoint

  backtest/
    mod.rs             # Backtest orchestration: walk-forward, K-fold, PSR fitness
                       #   P6-1A: publication_delay temporal causality alignment
                       #   P6-3C: VM error log suppression (evolution phase → DEBUG)
                       #   P9-1A: quantized_position() — 0.25-step quantization + deadzone
                       #   P9-2B: bic_complexity_penalty() — BIC k*ln(n)/(2n) replaces linear
                       #   P9-2C: Walk-forward target_steps increased to 5
    data_frame.rs      # OHLCV data frame with factor columns
    portfolio.rs       # Simulated portfolio for backtest (P&L, drawdown, Sharpe)
    ensemble.rs        # P5: Portfolio ensemble selection (top strategies per exchange)
                       #   P6-1D: Non-linear decay routing buffer (active→decaying→retired)
                       #   P7-2A: select_candidates_with_lfdr() — lFDR filtering before selection
                       #   P9-1C: RebalanceTriggerConfig + should_trigger_rebalance()
                       #   P9-1C: spearman_rank_correlation() for signal divergence detection
    ensemble_weights.rs # P5: Dynamic weight adjustments (PSR reward, utilization, crowding)
                       #   P6-2A: Hysteresis dead-zone with per-asset thresholds
                       #   P7-3D: debug! tracing for dead-zone trigger/suppress decisions
                       #   P8-4B: Internal calculations converted to rust_decimal::Decimal
                       #   P9-1A: Dead-zone code activated (removed dead_code annotations)
    hrp.rs             # P5: Hierarchical Risk Parity allocation
    hypothesis.rs      # P6-1B: Local FDR hypothesis testing with n-gram Jaccard clustering
                       #   P7-2A: LfdrConfig #[derive(Deserialize)] for YAML config
                       #   [Feature gate: lfdr.enabled, wired into ensemble selection]
    incremental_pca.rs # P6-1C: CCIPCA incremental PCA, O(n·k), ndarray::Zip zero-alloc
                       #   P7-2B: update_view(ArrayView1) zero-copy method
                       #   P8-1A: project_features() augments feature tensor with PC columns
    factor_importance.rs # P7-5A: Permutation-based factor importance attribution
                       #   Shuffles each factor, measures PSR drop, returns sorted importances
                       #   P8-0A: bottom_n_summary() returns Bottom-10 noise factors
                       #   P9-3A: CausalVerificationResult + run_causal_verification()
                       #   P9-3A: partial_correlation() + pearson_correlation() — partial r_xy.z

  mcts/                # P6-4A→P7-1: MCTS Symbolic Regression (integrated in P7)
    mod.rs             #   Module entry: re-exports arena, state, search, policy
    arena.rs           #   Arena allocator: Vec<Node> + u32 indices (zero Rc/Arc)
                       #   P7-1A: debug_assert! overflow guard on alloc()
    state.rs           #   RPN formula state: partial tokens + stack depth, ActionSpace
                       #   P9: is_terminal min length 3 (prevents single-token formula dominance)
    search.rs          #   MCTS search: select→expand→simulate→backprop, DeceptionSuppressor
                       #   P6-4B: Extreme bandit PUCT (max_reward variant, P7 default=true)
                       #   P6-4D: 3/4-gram deception suppression via FNV-1a hashing
                       #   P7-1D: Integration tests (valid genomes, token conversion)
                       #   P9-2A: SubformulaArchive — MAP-Elites 5-bucket behavior archive
                       #   P9: Fixed double random_rollout bug (reuse first rollout tokens)
    policy.rs          #   Policy trait: UniformPolicy, HeuristicPolicy, LlmCachedPolicy
                       #   P6-4C: LLM semantic prior with HashMap cache
                       #   P8-0B: build_llm_prior_weights() — factor importance + elite operator prior
                       #   P8-0C: canonicalize_rpn() — commutative operand sorting for cache hits
                       #   P8-0D: LlmCachedPolicy wired to semantic prior (replaces UniformPolicy)
                       #   [Gate: mcts.enabled; wired into evolution loop via dedicated Rayon pool]
```

## Key Algorithms

### ALPS (Age-Layered Population Structure)
- 5 Fibonacci-aged layers: max ages [5, 13, 34, 89, 500]
- 100 genomes per layer = 500 total population
- Young genomes start in Layer 0, promoted up by age
- Prevents premature convergence

### Fitness: PSR (Probabilistic Sharpe Ratio)
- Bailey & Lopez de Prado, 2012
- Both IS and OOS evaluation
- Complexity penalty: BIC k*ln(n)/(2n) with high-risk operator weighting (P9-2B)
- IS PSR cap: 3.5 (prevents OOS overfitting)
- TFT sentinel: -15.0 for too-few-trades

### Walk-Forward Validation
- `initial_train: 2500`, `target_test_window: 1000`
- Resolution-aware embargo gaps (20/10/8 bars for 1d/1h/15m)
- `target_steps: 5` walk-forward windows (P9-2C, increased from 3)

### Multi-Timeframe (P3)
- 25 factors x 3 resolutions [1h, 4h, 1d] = 75 features
- Token layout: 0-24 = 1h, 25-49 = 4h, 50-74 = 1d

### Adaptive Thresholds (P4)
- Percentile-based thresholds with clamp ranges
- UtilizationTracker monitors long/short ratios per generation
- Adjusts every 50 generations based on utilization feedback

### Portfolio Ensemble (P5)
- HRP allocation for diversification
- Dynamic weights: PSR reward, utilization floor, crowding penalty
- Shadow equity tracking
- Rebalance interval: 30 minutes

### P6 Statistical Safeguards
- **Temporal Causality** (P6-1A): publication_delay per resolution prevents look-ahead bias
- **Local FDR** (P6-1B): n-gram Jaccard clustering → per-cluster lFDR (replaces global BH)
- **CCIPCA** (P6-1C): O(n·k) incremental PCA (avoids O(n²) covariance at 18k dimensions)
- **Decay Routing** (P6-1D): Smooth exponential weight decay (active→decaying→retired)
- **Hysteresis Dead-Zone** (P6-2A): Per-asset no-trade threshold to reduce micro-rebalancing

### P6→P7 MCTS Symbolic Regression
- Arena-allocated tree: contiguous Vec<Node> with u32 indices, zero-cost GC
- PUCT selector: configurable mean vs max reward (Extreme Bandit default in P7)
- LLM cached policy: prior P(next_token|partial_RPN) with HashMap cache
- Deception suppression: FNV-1a n-gram hashing with exponential decay penalty
- P7 Integration: dedicated Rayon pool, runs every `interval` generations, positive-PSR seeds inject into ALPS L0

### P7 Additions
- **lFDR Ensemble Filtering** (P7-2A): RPN n-gram Jaccard clustering applied before ensemble candidate selection
- **CCIPCA Diagnostic** (P7-2B): Zero-copy ArrayView, lazy-initialized per symbol, explained variance logging
- **Factor Importance** (P7-5A): Permutation importance — shuffle factor column, measure PSR drop
- **Genome Diversity** (P7-5B): Per-ALPS-layer Hamming distance, sampled max 50 pairs, logged every 50 gens
- **Security** (P7-3A): Removed hardcoded DB credentials
- **Dead-Zone Tracing** (P7-3D): debug! per-asset threshold/delta/triggered logging in hysteresis

### P8 Implemented
- **LLM-Guided MCTS Prior** (P8-0): FactorImportance → build_llm_prior_weights() → LlmCachedPolicy; canonical RPN hash
- **CCIPCA Active Remapping** (P8-1): project_features() augments 75→80 features (5 PC columns after 200 obs)
- **Diversity-Triggered Injection** (P8-2): L3/L4 Hamming diversity triggers random injection + elitist cull
- **VM Hot Path Optimization** (P8-3): Pre-execution shape guard, O(n) running-sum ts_mean/ts_sum, conditional NaN sanitization
- **sqlx 0.8 + Decimal** (P8-4): RUSTSEC-2024-0363 fix, f64→Decimal in ensemble_weights, removed 16MiB guard

### P9 Implemented
- **Quantized Position + Deadzone** (P9-1A): 0.25-step quantization + hysteresis deadzone filtering prevents fragment orders
- **DiversityTrigger Topology Mutation** (P9-1B): Replaces brute-force threshold reset with L0 cull + random injection + LLM rescue
- **Dynamic Ensemble Rebalance** (P9-1C): Signal divergence trigger via Spearman correlation + OOS improvement ratio gating
- **MAP-Elites SubformulaArchive** (P9-2A): 5 behavior buckets (Momentum, MeanRevert, Volatility, CrossAsset, Arithmetic) × 40 capacity
- **BIC Complexity Penalty** (P9-2B): `effective_k * ln(n) / (2*n)` replaces linear 0.05/token; high-risk ops weighted 1.5x
- **Walk-Forward 5 Steps** (P9-2C): Increased from 3 to 5 walk-forward validation steps
- **Causal Verification Pipeline** (P9-3A): Three-stage LLM hypothesis → partial correlation → lFDR confirmation
- **MCTS Min-Length 3** (P9): Prevents single-token formulas from dominating MCTS search
- **MCTS Single-Rollout Fix** (P9): Eliminated double random_rollout bug that prevented terminal token recording

## Dependencies
- `common`, `backtest-engine` (workspace crates)
- `aws-sdk-bedrockruntime` (LLM oracle)
- `ndarray` (P6-1C: CCIPCA matrix operations)
- `rust_decimal`, `rust_decimal_macros` (P8-4B: financial-grade precision)
- TimescaleDB (strategy persistence, backtest results)
- Redis (market data subscription)
