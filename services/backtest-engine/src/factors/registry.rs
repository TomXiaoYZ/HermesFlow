use super::traits::*;
use serde_json::Value;
use std::collections::HashMap;

/// Registry of all available factor factories
pub struct FactorRegistry {
    factories: HashMap<String, Box<dyn FactorFactory>>,
}

impl Default for FactorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl FactorRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            factories: HashMap::new(),
        };

        // Register all 33 factors
        registry.register("log_returns", Box::new(LogReturnsFactory));
        registry.register("liquidity_health", Box::new(LiquidityHealthFactory));
        registry.register("pressure", Box::new(BuySellPressureFactory));
        registry.register("fomo", Box::new(FOMOFactory));
        registry.register("pump_dev", Box::new(PumpDeviationFactory));
        registry.register("log_vol", Box::new(LogVolumeFactory));
        registry.register("vol_cluster", Box::new(VolatilityClusteringFactory));
        registry.register("mom_rev", Box::new(MomentumReversalFactory));
        registry.register("rsi", Box::new(RSIFactory));
        registry.register("ema_12", Box::new(EMA12Factory));
        registry.register("ema_26", Box::new(EMA26Factory));
        registry.register("ema_50", Box::new(EMA50Factory));
        registry.register("sma_200", Box::new(SMA200Factory));
        registry.register("macd_line", Box::new(MACDLineFactory));
        registry.register("macd_signal", Box::new(MACDSignalFactory));
        registry.register("macd_hist", Box::new(MACDHistFactory));
        registry.register("bb_bandwidth", Box::new(BBBandwidthFactory));
        registry.register("bb_percent_b", Box::new(BBPercentBFactory));
        registry.register("bb_position", Box::new(BBPositionFactory));
        registry.register("atr_pct", Box::new(ATRPercentFactory));
        registry.register("stoch_k", Box::new(StochKFactory));
        registry.register("stoch_d", Box::new(StochDFactory));
        registry.register("cci", Box::new(CCINormalizedFactory));
        registry.register("williams_r", Box::new(WilliamsRNormFactory));
        registry.register("vwap_dev", Box::new(VWAPDeviationFactory));
        registry.register("vwap_roll_dev", Box::new(VWAPRollingDevFactory));
        registry.register("obv_pct", Box::new(OBVPctChangeFactory));
        registry.register("mfi", Box::new(MFINormFactory));
        registry.register("hl_range", Box::new(HLRangeFactory));
        registry.register("close_pos", Box::new(ClosePositionFactory));
        registry.register("vol_trend", Box::new(VolumeTrendFactory));
        registry.register("momentum_10", Box::new(Momentum10Factory));
        registry.register("momentum_20", Box::new(Momentum20Factory));

        registry
    }

    fn register(&mut self, slug: &str, factory: Box<dyn FactorFactory>) {
        self.factories.insert(slug.to_string(), factory);
    }

    pub fn create(&self, slug: &str, params: &Value) -> Option<Box<dyn Factor>> {
        self.factories.get(slug).map(|f| f.create(params))
    }

    pub fn available_factors(&self) -> Vec<String> {
        self.factories.keys().cloned().collect()
    }
}

// ============================================================
// FACTORY IMPLEMENTATIONS (one per factor)
// ============================================================

macro_rules! simple_factory {
    ($name:ident, $factor:ty) => {
        pub struct $name;
        impl FactorFactory for $name {
            fn create(&self, _params: &Value) -> Box<dyn Factor> {
                Box::new(<$factor>::default())
            }
        }

        impl Default for $factor {
            fn default() -> Self {
                Self
            }
        }
    };
}

macro_rules! param_factory {
    ($factory_name:ident, $factor_type:ty, $field:ident, $default:expr) => {
        pub struct $factory_name;
        impl FactorFactory for $factory_name {
            fn create(&self, params: &Value) -> Box<dyn Factor> {
                let $field = params
                    .get(stringify!($field))
                    .and_then(|v| v.as_i64())
                    .unwrap_or($default) as usize;
                Box::new(<$factor_type>::new($field))
            }
        }
    };
}

// Meme indicators
simple_factory!(LogReturnsFactory, LogReturns);
simple_factory!(LiquidityHealthFactory, LiquidityHealth);
simple_factory!(BuySellPressureFactory, BuySellPressure);
simple_factory!(FOMOFactory, FOMO);
simple_factory!(LogVolumeFactory, LogVolume);
param_factory!(PumpDeviationFactory, PumpDeviation, window, 20);
param_factory!(
    VolatilityClusteringFactory,
    VolatilityClustering,
    window,
    20
);
param_factory!(MomentumReversalFactory, MomentumReversal, window, 20);
param_factory!(RSIFactory, RSI, period, 14);

// Moving averages
simple_factory!(EMA12Factory, EMA12);
simple_factory!(EMA26Factory, EMA26);
simple_factory!(EMA50Factory, EMA50);
simple_factory!(SMA200Factory, SMA200);

// MACD
simple_factory!(MACDLineFactory, MACDLine);
simple_factory!(MACDSignalFactory, MACDSignal);
simple_factory!(MACDHistFactory, MACDHist);

// Bollinger
simple_factory!(BBBandwidthFactory, BBBandwidth);
simple_factory!(BBPercentBFactory, BBPercentB);
simple_factory!(BBPositionFactory, BBPosition);

// ATR
simple_factory!(ATRPercentFactory, ATRPercent);

// Stochastic
simple_factory!(StochKFactory, StochK);
simple_factory!(StochDFactory, StochD);

// CCI
simple_factory!(CCINormalizedFactory, CCINormalized);

// Williams %R
simple_factory!(WilliamsRNormFactory, WilliamsRNorm);

// VWAP
simple_factory!(VWAPDeviationFactory, VWAPDeviation);
simple_factory!(VWAPRollingDevFactory, VWAPRollingDev);

// OBV
simple_factory!(OBVPctChangeFactory, OBVPctChange);

// MFI
simple_factory!(MFINormFactory, MFINorm);

// Additional
simple_factory!(HLRangeFactory, HLRange);
simple_factory!(ClosePositionFactory, ClosePosition);
simple_factory!(VolumeTrendFactory, VolumeTrend);
simple_factory!(Momentum10Factory, Momentum10);
simple_factory!(Momentum20Factory, Momentum20);
