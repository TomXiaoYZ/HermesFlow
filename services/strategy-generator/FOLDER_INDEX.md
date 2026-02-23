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
    data_frame.rs      # OHLCV data frame with factor columns
    portfolio.rs       # Simulated portfolio for backtest (P&L, drawdown, Sharpe)
    ensemble.rs        # P5: Portfolio ensemble selection (top strategies per exchange)
    ensemble_weights.rs # P5: Dynamic weight adjustments (PSR reward, utilization, crowding)
    hrp.rs             # P5: Hierarchical Risk Parity allocation
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

## Dependencies
- `common`, `backtest-engine` (workspace crates)
- `aws-sdk-bedrockruntime` (LLM oracle)
- TimescaleDB (strategy persistence, backtest results)
- Redis (market data subscription)
