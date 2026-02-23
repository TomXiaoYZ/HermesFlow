# backtest-engine - FOLDER_INDEX

> Shared library crate for factor computation and VM-based strategy execution. Used by both `strategy-generator` (offline backtesting) and `strategy-engine` (real-time evaluation).

## Module Map

```
src/
  lib.rs               # Crate re-exports
  main.rs              # Standalone binary (unused in production)
  backtest.rs          # Backtest runner: apply strategy genome to OHLCV data
  config.rs            # Backtest configuration (commission, slippage, etc.)

  factors/
    mod.rs             # Factor module exports + FactorRegistry
    traits.rs          # Factor trait definition
    registry.rs        # Factor registration by ID (0-24)
    engineer.rs        # FactorEngineer: computes all factors for a data frame
    dynamic.rs         # Dynamic factor loading from YAML config

    # Individual factor implementations (25 base factors):
    atr.rs             # Average True Range (ATR%)
    bollinger.rs       # Bollinger Band %B
    cci.rs             # Commodity Channel Index
    indicators.rs      # Generic indicator helpers
    macd.rs            # MACD histogram
    mfi.rs             # Money Flow Index
    moving_averages.rs # SMA-200 diff, trend strength
    obv.rs             # On-Balance Volume %
    stochastic.rs      # Stochastic oscillator
    vwap.rs            # VWAP deviation
    williams_r.rs      # Williams %R

  vm/
    mod.rs             # VM module exports
    ops.rs             # 23 opcodes: arithmetic, comparison, logic, stack ops
    vm.rs              # Stack-based VM: executes RPN genome → signal value
    engine.rs          # VM engine: batch evaluation over data frames
```

## Factor IDs (0-24)

Defined in `config/factors-stock.yaml`:
```
0: return           1: vwap_deviation    2: volume_ratio
3: mean_reversion   4: adv_ratio         5: volatility
6: momentum         7: relative_strength 8: close_position
9: intraday_range  10: vol_regime       11: trend_strength
12: momentum_regime 13: atr_pct         14: obv_pct
15: mfi            16: bb_percent_b     17: macd_hist
18: sma_200_diff   19: amihud_illiq     20: spread_proxy
21: return_autocorr 22: spy_corr        23: spy_beta
24: spy_rel_strength
```

## VM Opcodes (23)

Arithmetic: ADD, SUB, MUL, DIV, ABS, NEG, SQRT
Comparison: GT, LT, EQ, MAX, MIN
Logic: AND, OR, NOT, IF
Stack: DUP, SWAP, NOP
Special: LAG, SMA, EMA, ZSCORE

## Dependencies
- `ndarray`, `ndarray-stats` (numerical computation)
- `serde`, `serde_json`, `serde_yaml` (serialization)
- No workspace crate deps (standalone library used by strategy-engine and strategy-generator)
