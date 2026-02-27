# strategy-generator - FOLDER_INDEX

> Genetic algorithm strategy evolution engine. Uses ALPS population structure, PSR fitness, walk-forward validation, LLM-guided mutation, and portfolio ensemble optimization.

## Module Map

```
src/
  main.rs              # Entry point: config load, DB init, evolution loop spawn, API server
  genetic.rs           # Core GA: ALPS layers, selection, crossover, mutation, promotion
                       #   - ALPS: 5 layers [5,13,34,89,500 gens], 100 genomes/layer
                       #   - Dual-mode: long_only + long_short per (exchange, symbol)
                       #   - Operator pruning: 14/23 opcodes for new genomes
  genome_decoder.rs    # Decodes integer genome → RPN formula string (human-readable)
  factor_loader.rs     # Loads factor definitions from YAML config
  llm_oracle.rs        # LLM-guided mutation (P2): Bedrock/Claude generates RPN formulas
                       #   - Triggered on stagnation (low promotion rate or high TFT rate)
                       #   - Cross-symbol learning: includes top formulas from other symbols
  api.rs               # HTTP API: evolution status, backtest results, strategy queries
  health.rs            # /health endpoint

  backtest/
    mod.rs             # Backtest orchestration: walk-forward, K-fold, PSR fitness
                       #   P6-1A: publication_delay temporal causality alignment
                       #   P6-3C: VM error log suppression (evolution phase → DEBUG)
    data_frame.rs      # OHLCV data frame with factor columns
    portfolio.rs       # Simulated portfolio for backtest (P&L, drawdown, Sharpe)
    ensemble.rs        # P5: Portfolio ensemble selection (top strategies per exchange)
                       #   P6-1D: Non-linear decay routing buffer (active→decaying→retired)
    ensemble_weights.rs # P5: Dynamic weight adjustments (PSR reward, utilization, crowding)
                       #   P6-2A: Hysteresis dead-zone with per-asset thresholds
    hrp.rs             # P5: Hierarchical Risk Parity allocation
    hypothesis.rs      # P6-1B: Local FDR hypothesis testing with n-gram Jaccard clustering
                       #   [Feature gate: lfdr.enabled=false]
    incremental_pca.rs # P6-1C: CCIPCA incremental PCA, O(n·k), ndarray::Zip zero-alloc
                       #   [Feature gate: pca_enabled=false]

  mcts/                # P6-4A: LLM-Prior MCTS Symbolic Regression
    mod.rs             #   Module entry: re-exports arena, state, search, policy
    arena.rs           #   Arena allocator: Vec<Node> + u32 indices (zero Rc/Arc)
    state.rs           #   RPN formula state: partial tokens + stack depth, ActionSpace
    search.rs          #   MCTS search: select→expand→simulate→backprop, DeceptionSuppressor
                       #   P6-4B: Extreme bandit PUCT (max_reward variant)
                       #   P6-4D: 3/4-gram deception suppression via FNV-1a hashing
    policy.rs          #   Policy trait: UniformPolicy, HeuristicPolicy, LlmCachedPolicy
                       #   P6-4C: LLM semantic prior with HashMap cache
                       #   [Feature gate: mcts.enabled=false]
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
- Complexity penalty: 0.02/token above 8
- TFT sentinel: -15.0 for too-few-trades

### Walk-Forward Validation
- `initial_train: 2500`, `target_test_window: 1000`
- Resolution-aware embargo gaps (20/10/8 bars for 1d/1h/15m)
- `target_steps: 3` walk-forward windows

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

### P6 MCTS Symbolic Regression (Phase 4)
- Arena-allocated tree: contiguous Vec<Node> with u32 indices, zero-cost GC
- PUCT selector: configurable mean vs max reward (extreme bandit variant)
- LLM cached policy: prior P(next_token|partial_RPN) with HashMap cache
- Deception suppression: FNV-1a n-gram hashing with exponential decay penalty
- Integration: MCTS seeds inject into ALPS Layer 0 via `inject_genomes`

## Dependencies
- `common`, `backtest-engine` (workspace crates)
- `aws-sdk-bedrockruntime` (LLM oracle)
- `ndarray` (P6-1C: CCIPCA matrix operations)
- TimescaleDB (strategy persistence, backtest results)
- Redis (market data subscription)
