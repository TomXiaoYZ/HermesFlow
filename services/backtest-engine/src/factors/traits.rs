use crate::factors::atr::ATR;
use crate::factors::bollinger::BollingerBands;
use crate::factors::cci::CCI;
use crate::factors::indicators::MemeIndicators;
use crate::factors::macd::MACD;
use crate::factors::mfi::MFI;
use crate::factors::moving_averages::MovingAverages;
use crate::factors::obv::OBV;
use crate::factors::stochastic::Stochastic;
use crate::factors::vwap::VWAP;
use crate::factors::williams_r::WilliamsR;
use crate::vm::ops::ts_delay;
use ndarray::Array2;
use std::collections::HashMap;

/// Factor computation context - all input data needed
#[derive(Clone)]
pub struct FactorContext {
    pub close: Array2<f64>,
    pub open: Array2<f64>,
    pub high: Array2<f64>,
    pub low: Array2<f64>,
    pub volume: Array2<f64>,
    pub liquidity: Array2<f64>,
    pub fdv: Array2<f64>,
    /// Cache for intermediate results (e.g., "ema_12" -> computed EMA)
    pub cache: HashMap<String, Array2<f64>>,
}

/// Trait that all factors must implement
pub trait Factor: Send + Sync {
    /// Unique identifier (matches DB slug)
    fn slug(&self) -> &str;

    /// Compute the factor given context
    fn compute(&self, ctx: &mut FactorContext) -> Array2<f64>;

    /// List of factor slugs this depends on (for dependency resolution)
    fn dependencies(&self) -> Vec<&str> {
        vec![]
    }
}

/// Factory trait for creating factor instances
pub trait FactorFactory: Send + Sync {
    fn create(&self, params: &serde_json::Value) -> Box<dyn Factor>;
}

// ============================================================
// CONCRETE FACTOR IMPLEMENTATIONS
// ============================================================

// Meme Indicators
pub struct LogReturns;
impl Factor for LogReturns {
    fn slug(&self) -> &str {
        "log_returns"
    }
    fn compute(&self, ctx: &mut FactorContext) -> Array2<f64> {
        let prev = ts_delay(&ctx.close, 1);
        (&ctx.close / (&prev + 1e-9)).mapv(f64::ln)
    }
}

pub struct LiquidityHealth;
impl Factor for LiquidityHealth {
    fn slug(&self) -> &str {
        "liquidity_health"
    }
    fn compute(&self, ctx: &mut FactorContext) -> Array2<f64> {
        MemeIndicators::liquidity_health(&ctx.liquidity, &ctx.fdv)
    }
}

pub struct BuySellPressure;
impl Factor for BuySellPressure {
    fn slug(&self) -> &str {
        "pressure"
    }
    fn compute(&self, ctx: &mut FactorContext) -> Array2<f64> {
        MemeIndicators::buy_sell_imbalance(&ctx.close, &ctx.open, &ctx.high, &ctx.low)
    }
}

pub struct FOMO;
impl Factor for FOMO {
    fn slug(&self) -> &str {
        "fomo"
    }
    fn compute(&self, ctx: &mut FactorContext) -> Array2<f64> {
        MemeIndicators::fomo_acceleration(&ctx.volume)
    }
}

pub struct PumpDeviation {
    window: usize,
}

impl PumpDeviation {
    pub fn new(window: usize) -> Self {
        Self { window }
    }
}

impl Factor for PumpDeviation {
    fn slug(&self) -> &str {
        "pump_dev"
    }
    fn compute(&self, ctx: &mut FactorContext) -> Array2<f64> {
        MemeIndicators::pump_deviation(&ctx.close, self.window)
    }
}

pub struct LogVolume;
impl Factor for LogVolume {
    fn slug(&self) -> &str {
        "log_vol"
    }
    fn compute(&self, ctx: &mut FactorContext) -> Array2<f64> {
        ctx.volume.mapv(|v| (v + 1.0).ln())
    }
}

pub struct VolatilityClustering {
    window: usize,
}

impl VolatilityClustering {
    pub fn new(window: usize) -> Self {
        Self { window }
    }
}

impl Factor for VolatilityClustering {
    fn slug(&self) -> &str {
        "vol_cluster"
    }
    fn compute(&self, ctx: &mut FactorContext) -> Array2<f64> {
        let returns = ctx.cache.get("log_returns").cloned().unwrap_or_else(|| {
            let prev = ts_delay(&ctx.close, 1);
            (&ctx.close / (&prev + 1e-9)).mapv(f64::ln)
        });
        MemeIndicators::volatility_clustering(&returns, self.window)
    }
    fn dependencies(&self) -> Vec<&str> {
        vec!["log_returns"]
    }
}

pub struct MomentumReversal {
    window: usize,
}

impl MomentumReversal {
    pub fn new(window: usize) -> Self {
        Self { window }
    }
}

impl Factor for MomentumReversal {
    fn slug(&self) -> &str {
        "mom_rev"
    }
    fn compute(&self, ctx: &mut FactorContext) -> Array2<f64> {
        MemeIndicators::momentum_reversal(&ctx.close, self.window)
    }
}

pub struct RSI {
    period: usize,
}

impl RSI {
    pub fn new(period: usize) -> Self {
        Self { period }
    }
}

impl Factor for RSI {
    fn slug(&self) -> &str {
        "rsi"
    }
    fn compute(&self, ctx: &mut FactorContext) -> Array2<f64> {
        MemeIndicators::relative_strength(&ctx.close, self.period)
    }
}

// Moving Averages
pub struct EMA12;
impl Factor for EMA12 {
    fn slug(&self) -> &str {
        "ema_12"
    }
    fn compute(&self, ctx: &mut FactorContext) -> Array2<f64> {
        let ema = MovingAverages::ema(&ctx.close, 12);
        (&ctx.close - &ema) / (&ctx.close + 1e-9)
    }
}

pub struct EMA26;
impl Factor for EMA26 {
    fn slug(&self) -> &str {
        "ema_26"
    }
    fn compute(&self, ctx: &mut FactorContext) -> Array2<f64> {
        let ema = MovingAverages::ema(&ctx.close, 26);
        (&ctx.close - &ema) / (&ctx.close + 1e-9)
    }
}

pub struct EMA50;
impl Factor for EMA50 {
    fn slug(&self) -> &str {
        "ema_50"
    }
    fn compute(&self, ctx: &mut FactorContext) -> Array2<f64> {
        let ema = MovingAverages::ema(&ctx.close, 50);
        (&ctx.close - &ema) / (&ctx.close + 1e-9)
    }
}

pub struct SMA200;
impl Factor for SMA200 {
    fn slug(&self) -> &str {
        "sma_200"
    }
    fn compute(&self, ctx: &mut FactorContext) -> Array2<f64> {
        let sma = MovingAverages::sma(&ctx.close, 200);
        (&ctx.close - &sma) / (&ctx.close + 1e-9)
    }
}

// MACD
pub struct MACDLine;
impl Factor for MACDLine {
    fn slug(&self) -> &str {
        "macd_line"
    }
    fn compute(&self, ctx: &mut FactorContext) -> Array2<f64> {
        let (line, _, _) = MACD::macd(&ctx.close);
        &line / (&ctx.close + 1e-9)
    }
}

pub struct MACDSignal;
impl Factor for MACDSignal {
    fn slug(&self) -> &str {
        "macd_signal"
    }
    fn compute(&self, ctx: &mut FactorContext) -> Array2<f64> {
        let (_, signal, _) = MACD::macd(&ctx.close);
        &signal / (&ctx.close + 1e-9)
    }
}

pub struct MACDHist;
impl Factor for MACDHist {
    fn slug(&self) -> &str {
        "macd_hist"
    }
    fn compute(&self, ctx: &mut FactorContext) -> Array2<f64> {
        let (_, _, hist) = MACD::macd(&ctx.close);
        &hist / (&ctx.close + 1e-9)
    }
}

// Bollinger
pub struct BBBandwidth;
impl Factor for BBBandwidth {
    fn slug(&self) -> &str {
        "bb_bandwidth"
    }
    fn compute(&self, ctx: &mut FactorContext) -> Array2<f64> {
        BollingerBands::bandwidth(&ctx.close, 20)
    }
}

pub struct BBPercentB;
impl Factor for BBPercentB {
    fn slug(&self) -> &str {
        "bb_percent_b"
    }
    fn compute(&self, ctx: &mut FactorContext) -> Array2<f64> {
        BollingerBands::percent_b(&ctx.close, 20)
    }
}

pub struct BBPosition;
impl Factor for BBPosition {
    fn slug(&self) -> &str {
        "bb_position"
    }
    fn compute(&self, ctx: &mut FactorContext) -> Array2<f64> {
        let (_, middle, _) = BollingerBands::bollinger(&ctx.close);
        (&ctx.close - &middle) / (&middle + 1e-9)
    }
}

// ATR
pub struct ATRPercent;
impl Factor for ATRPercent {
    fn slug(&self) -> &str {
        "atr_pct"
    }
    fn compute(&self, ctx: &mut FactorContext) -> Array2<f64> {
        ATR::atr_percent(&ctx.high, &ctx.low, &ctx.close)
    }
}

// Stochastic
pub struct StochK;
impl Factor for StochK {
    fn slug(&self) -> &str {
        "stoch_k"
    }
    fn compute(&self, ctx: &mut FactorContext) -> Array2<f64> {
        let (k, _) = Stochastic::stochastic(&ctx.high, &ctx.low, &ctx.close);
        (&k - 50.0) / 50.0 // Normalize to [-1, 1]
    }
}

pub struct StochD;
impl Factor for StochD {
    fn slug(&self) -> &str {
        "stoch_d"
    }
    fn compute(&self, ctx: &mut FactorContext) -> Array2<f64> {
        let (_, d) = Stochastic::stochastic(&ctx.high, &ctx.low, &ctx.close);
        (&d - 50.0) / 50.0
    }
}

// CCI
pub struct CCINormalized;
impl Factor for CCINormalized {
    fn slug(&self) -> &str {
        "cci"
    }
    fn compute(&self, ctx: &mut FactorContext) -> Array2<f64> {
        CCI::cci_normalized(&ctx.high, &ctx.low, &ctx.close, 20)
    }
}

// Williams %R
pub struct WilliamsRNorm;
impl Factor for WilliamsRNorm {
    fn slug(&self) -> &str {
        "williams_r"
    }
    fn compute(&self, ctx: &mut FactorContext) -> Array2<f64> {
        WilliamsR::williams_r_normalized(&ctx.high, &ctx.low, &ctx.close, 14)
    }
}

// VWAP
pub struct VWAPDeviation;
impl Factor for VWAPDeviation {
    fn slug(&self) -> &str {
        "vwap_dev"
    }
    fn compute(&self, ctx: &mut FactorContext) -> Array2<f64> {
        VWAP::vwap_deviation(&ctx.high, &ctx.low, &ctx.close, &ctx.volume)
    }
}

pub struct VWAPRollingDev;
impl Factor for VWAPRollingDev {
    fn slug(&self) -> &str {
        "vwap_roll_dev"
    }
    fn compute(&self, ctx: &mut FactorContext) -> Array2<f64> {
        let vwap = VWAP::vwap_rolling(&ctx.high, &ctx.low, &ctx.close, &ctx.volume, 20);
        (&ctx.close - &vwap) / (&vwap + 1e-9)
    }
}

// OBV
pub struct OBVPctChange;
impl Factor for OBVPctChange {
    fn slug(&self) -> &str {
        "obv_pct"
    }
    fn compute(&self, ctx: &mut FactorContext) -> Array2<f64> {
        OBV::obv_pct_change(&ctx.close, &ctx.volume)
    }
}

// MFI
pub struct MFINorm;
impl Factor for MFINorm {
    fn slug(&self) -> &str {
        "mfi"
    }
    fn compute(&self, ctx: &mut FactorContext) -> Array2<f64> {
        MFI::mfi_normalized(&ctx.high, &ctx.low, &ctx.close, &ctx.volume, 14)
    }
}

// Additional
pub struct HLRange;
impl Factor for HLRange {
    fn slug(&self) -> &str {
        "hl_range"
    }
    fn compute(&self, ctx: &mut FactorContext) -> Array2<f64> {
        (&ctx.high - &ctx.low) / (&ctx.close + 1e-9)
    }
}

pub struct ClosePosition;
impl Factor for ClosePosition {
    fn slug(&self) -> &str {
        "close_pos"
    }
    fn compute(&self, ctx: &mut FactorContext) -> Array2<f64> {
        let pos = (&ctx.close - &ctx.low) / (&ctx.high - &ctx.low + 1e-9);
        pos.mapv(|v| v.clamp(0.0, 1.0))
    }
}

pub struct VolumeTrend;
impl Factor for VolumeTrend {
    fn slug(&self) -> &str {
        "vol_trend"
    }
    fn compute(&self, ctx: &mut FactorContext) -> Array2<f64> {
        let prev = ts_delay(&ctx.volume, 1);
        (&ctx.volume - &prev) / (&prev + 1.0)
    }
}

pub struct Momentum10;
impl Factor for Momentum10 {
    fn slug(&self) -> &str {
        "momentum_10"
    }
    fn compute(&self, ctx: &mut FactorContext) -> Array2<f64> {
        let prev = ts_delay(&ctx.close, 10);
        (&ctx.close - &prev) / (&prev + 1e-9)
    }
}

pub struct Momentum20;
impl Factor for Momentum20 {
    fn slug(&self) -> &str {
        "momentum_20"
    }
    fn compute(&self, ctx: &mut FactorContext) -> Array2<f64> {
        let prev = ts_delay(&ctx.close, 20);
        (&ctx.close - &prev) / (&prev + 1e-9)
    }
}
