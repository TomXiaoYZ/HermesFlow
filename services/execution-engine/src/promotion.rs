//! P6b-B3: Promotion criteria for paper → live trading.
//!
//! Daily check evaluates paper trading performance to determine
//! whether a strategy qualifies for live trading promotion.
//! Promotion requires human approval — this module only flags eligibility.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio_postgres::Client;
use tracing::{info, warn};

/// Criteria thresholds for paper → live promotion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromotionCriteria {
    /// Minimum days of paper trading required.
    pub min_paper_days: i64,
    /// Minimum annualized Sharpe ratio.
    pub min_sharpe: f64,
    /// Maximum allowed deviation between paper and backtest PnL (fraction).
    pub max_pnl_deviation: f64,
    /// Maximum drawdown allowed (fraction).
    pub max_drawdown: f64,
    /// Minimum order fill rate (fraction).
    pub min_fill_rate: f64,
}

impl Default for PromotionCriteria {
    fn default() -> Self {
        Self {
            min_paper_days: 20,
            min_sharpe: 1.0,
            max_pnl_deviation: 0.20,
            max_drawdown: 0.15,
            min_fill_rate: 0.90,
        }
    }
}

/// Result of a promotion eligibility check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromotionResult {
    pub exchange: String,
    pub eligible: bool,
    pub paper_days: i64,
    pub sharpe: f64,
    pub max_drawdown: f64,
    pub fill_rate: f64,
    pub pnl_deviation: f64,
    pub failures: Vec<String>,
    pub checked_at: DateTime<Utc>,
}

/// Paper trading performance summary for promotion evaluation.
#[derive(Debug, Clone)]
struct PaperPerformance {
    days: i64,
    total_return: f64,
    sharpe: f64,
    max_drawdown: f64,
    total_orders: i64,
    filled_orders: i64,
}

/// Check if an exchange's paper trading results meet promotion criteria.
pub async fn check_promotion_eligibility(
    db: &Client,
    exchange: &str,
    criteria: &PromotionCriteria,
) -> anyhow::Result<PromotionResult> {
    let perf = load_paper_performance(db, exchange).await?;
    let mut failures = Vec::new();

    // 1. Minimum paper trading duration
    if perf.days < criteria.min_paper_days {
        failures.push(format!(
            "Insufficient paper days: {} < {}",
            perf.days, criteria.min_paper_days
        ));
    }

    // 2. Minimum Sharpe ratio
    if perf.sharpe < criteria.min_sharpe {
        failures.push(format!(
            "Sharpe too low: {:.3} < {:.3}",
            perf.sharpe, criteria.min_sharpe
        ));
    }

    // 3. Maximum drawdown
    if perf.max_drawdown > criteria.max_drawdown {
        failures.push(format!(
            "Drawdown too high: {:.1}% > {:.1}%",
            perf.max_drawdown * 100.0,
            criteria.max_drawdown * 100.0
        ));
    }

    // 4. Fill rate
    let fill_rate = if perf.total_orders > 0 {
        perf.filled_orders as f64 / perf.total_orders as f64
    } else {
        0.0
    };
    if fill_rate < criteria.min_fill_rate {
        failures.push(format!(
            "Fill rate too low: {:.1}% < {:.1}%",
            fill_rate * 100.0,
            criteria.min_fill_rate * 100.0
        ));
    }

    // 5. PnL deviation vs backtest (compare paper total_return against ensemble backtest)
    let backtest_return = load_backtest_return(db, exchange).await.unwrap_or(0.0);
    let pnl_deviation = if backtest_return.abs() > 1e-10 {
        ((perf.total_return - backtest_return) / backtest_return).abs()
    } else {
        0.0
    };
    if pnl_deviation > criteria.max_pnl_deviation {
        failures.push(format!(
            "PnL deviation too high: {:.1}% > {:.1}%",
            pnl_deviation * 100.0,
            criteria.max_pnl_deviation * 100.0
        ));
    }

    let eligible = failures.is_empty();

    let result = PromotionResult {
        exchange: exchange.to_string(),
        eligible,
        paper_days: perf.days,
        sharpe: perf.sharpe,
        max_drawdown: perf.max_drawdown,
        fill_rate,
        pnl_deviation,
        failures,
        checked_at: Utc::now(),
    };

    if eligible {
        info!(
            "[{}] PROMOTION ELIGIBLE: {}d paper, Sharpe={:.3}, DD={:.1}%, FillRate={:.1}%",
            exchange,
            perf.days,
            perf.sharpe,
            perf.max_drawdown * 100.0,
            fill_rate * 100.0,
        );
    } else {
        warn!(
            "[{}] Not eligible for promotion: {:?}",
            exchange, result.failures
        );
    }

    Ok(result)
}

/// Load paper trading performance from daily summaries.
async fn load_paper_performance(
    db: &Client,
    exchange: &str,
) -> anyhow::Result<PaperPerformance> {
    // Count paper trading days
    let days_row = db
        .query_one(
            "SELECT COUNT(DISTINCT date) as days FROM paper_daily_summary WHERE exchange = $1",
            &[&exchange],
        )
        .await;
    let days: i64 = days_row
        .map(|r| r.get::<_, i64>("days"))
        .unwrap_or(0);

    // Load daily equity series for Sharpe and drawdown calculation
    // Cast DECIMAL to float8 in SQL to avoid rust_decimal dependency
    let rows = db
        .query(
            "SELECT date, starting_equity::float8 as starting_equity, \
                    ending_equity::float8 as ending_equity \
             FROM paper_daily_summary \
             WHERE exchange = $1 \
             ORDER BY date ASC",
            &[&exchange],
        )
        .await
        .unwrap_or_default();

    let mut daily_returns: Vec<f64> = Vec::new();
    let mut equity_curve: Vec<f64> = Vec::new();

    for row in &rows {
        let start: f64 = row.get("starting_equity");
        let end: f64 = row.get("ending_equity");

        if start > 1e-10 {
            daily_returns.push((end / start) - 1.0);
        }
        equity_curve.push(end);
    }

    // Compute Sharpe
    let n = daily_returns.len();
    let sharpe = if n >= 2 {
        let mean = daily_returns.iter().sum::<f64>() / n as f64;
        let var = daily_returns
            .iter()
            .map(|r| (r - mean).powi(2))
            .sum::<f64>()
            / (n as f64 - 1.0);
        let std = var.sqrt();
        if std > 1e-10 {
            mean / std * 252.0_f64.sqrt()
        } else {
            0.0
        }
    } else {
        0.0
    };

    // Compute max drawdown
    let mut peak = 0.0_f64;
    let mut max_dd = 0.0_f64;
    for &eq in &equity_curve {
        if eq > peak {
            peak = eq;
        }
        if peak > 1e-10 {
            let dd = (peak - eq) / peak;
            if dd > max_dd {
                max_dd = dd;
            }
        }
    }

    // Total return
    let total_return = if !equity_curve.is_empty() && equity_curve[0] > 1e-10 {
        (equity_curve.last().unwrap_or(&0.0) / equity_curve[0]) - 1.0
    } else {
        0.0
    };

    // Order fill statistics
    let orders_row = db
        .query_one(
            "SELECT COUNT(*) as total, \
                    COUNT(*) FILTER (WHERE status = 'filled') as filled \
             FROM paper_trade_orders WHERE exchange = $1",
            &[&exchange],
        )
        .await;
    let (total_orders, filled_orders) = orders_row
        .map(|r| (r.get::<_, i64>("total"), r.get::<_, i64>("filled")))
        .unwrap_or((0, 0));

    Ok(PaperPerformance {
        days,
        total_return,
        sharpe,
        max_drawdown: max_dd,
        total_orders,
        filled_orders,
    })
}

/// Load the latest ensemble backtest cumulative return for comparison.
async fn load_backtest_return(db: &Client, exchange: &str) -> anyhow::Result<f64> {
    let row = db
        .query_one(
            "SELECT cumulative_return FROM ensemble_backtest_results \
             WHERE exchange = $1 ORDER BY created_at DESC LIMIT 1",
            &[&exchange],
        )
        .await?;
    Ok(row.get::<_, f64>("cumulative_return"))
}
