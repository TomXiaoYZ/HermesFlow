use chrono::Utc;
use common::events::TradeSignal;
use std::env;
use std::sync::Arc;
use tokio_postgres::Client as PgClient;
use tracing::warn;

#[derive(Debug, Clone)]
pub struct RiskResult {
    pub approved: bool,
    pub reason: String,
}

impl RiskResult {
    fn approve() -> Self {
        Self {
            approved: true,
            reason: String::new(),
        }
    }

    fn reject(reason: String) -> Self {
        Self {
            approved: false,
            reason,
        }
    }
}

pub struct StockRiskEngine {
    max_order_value_usd: f64,
    max_positions: usize,
    max_daily_loss_usd: f64,
}

impl Default for StockRiskEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl StockRiskEngine {
    pub fn new() -> Self {
        Self {
            max_order_value_usd: env::var("RISK_MAX_ORDER_VALUE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(2000.0),
            max_positions: env::var("RISK_MAX_POSITIONS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(5),
            max_daily_loss_usd: env::var("RISK_MAX_DAILY_LOSS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(500.0),
        }
    }

    pub async fn check_pre_trade(
        &self,
        signal: &TradeSignal,
        db: &Option<Arc<PgClient>>,
    ) -> RiskResult {
        // 1. Order value check
        let order_value = signal.quantity * signal.price.unwrap_or(0.0);
        if order_value > self.max_order_value_usd {
            return RiskResult::reject(format!(
                "Order value ${:.2} exceeds max ${:.2}",
                order_value, self.max_order_value_usd
            ));
        }

        // 2. Zero quantity check
        if signal.quantity <= 0.0 {
            return RiskResult::reject("Zero or negative quantity".to_string());
        }

        if let Some(client) = db {
            // 3. Open position count (mode-aware)
            let account_id = match signal.mode.as_deref() {
                Some(m) => format!("ibkr_{}", m),
                None => "default".to_string(),
            };
            let open_count = count_open_positions(client, "IBKR", &account_id).await;
            if open_count >= self.max_positions {
                return RiskResult::reject(format!(
                    "Already holding {} positions (max {})",
                    open_count, self.max_positions
                ));
            }

            // 4. Daily realized P&L check
            let daily_pnl = calculate_daily_pnl(client).await;
            if daily_pnl < -self.max_daily_loss_usd {
                return RiskResult::reject(format!(
                    "Daily loss ${:.2} exceeds max ${:.2}",
                    daily_pnl.abs(),
                    self.max_daily_loss_usd
                ));
            }

            // 5. Duplicate signal check (same symbol, same side, within 60s)
            let side_str = signal.side.to_string();
            let recent = has_recent_order(client, &signal.symbol, &side_str, 60).await;
            if recent {
                return RiskResult::reject(format!(
                    "Duplicate {} signal for {} within 60s",
                    side_str, signal.symbol
                ));
            }
        }

        RiskResult::approve()
    }
}

async fn count_open_positions(client: &PgClient, exchange: &str, account_id: &str) -> usize {
    let result = client
        .query_one(
            "SELECT COUNT(*)::BIGINT FROM trade_positions WHERE exchange = $1 AND account_id = $2 AND quantity != 0",
            &[&exchange, &account_id],
        )
        .await;

    match result {
        Ok(row) => {
            let count: i64 = row.get(0);
            count as usize
        }
        Err(e) => {
            warn!("Failed to count positions: {}", e);
            0
        }
    }
}

async fn calculate_daily_pnl(client: &PgClient) -> f64 {
    let today = Utc::now().date_naive();
    let result = client
        .query_one(
            "SELECT COALESCE(SUM(
                CASE WHEN side = 'Sell' THEN filled_qty * avg_price
                     ELSE -filled_qty * avg_price
                END
            ), 0)::FLOAT8 FROM trade_orders
            WHERE status = 'Filled' AND DATE(created_at) = $1",
            &[&today],
        )
        .await;

    match result {
        Ok(row) => row.get::<_, f64>(0),
        Err(e) => {
            warn!("Failed to calculate daily PnL: {}", e);
            0.0
        }
    }
}

async fn has_recent_order(client: &PgClient, symbol: &str, side: &str, secs: i64) -> bool {
    let cutoff = Utc::now() - chrono::Duration::seconds(secs);
    let result = client
        .query_one(
            "SELECT COUNT(*)::BIGINT FROM trade_orders WHERE symbol = $1 AND side = $2 AND created_at > $3",
            &[&symbol, &side, &cutoff],
        )
        .await;

    match result {
        Ok(row) => {
            let count: i64 = row.get(0);
            count > 0
        }
        Err(e) => {
            warn!("Failed to check recent orders: {}", e);
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use common::events::{OrderSide, OrderType};
    use uuid::Uuid;

    #[tokio::test]
    async fn test_order_value_limit() {
        let engine = StockRiskEngine::new();
        let signal = TradeSignal {
            id: Uuid::new_v4(),
            strategy_id: "test".to_string(),
            symbol: "AAPL".to_string(),
            side: OrderSide::Buy,
            quantity: 100.0,
            price: Some(180.0),
            order_type: OrderType::Market,
            timestamp: Utc::now(),
            reason: "test".to_string(),
            exchange: Some("polygon".to_string()),
            mode: None,
        };

        // 100 * 180 = 18000 > 2000 default max → reject
        let result = engine.check_pre_trade(&signal, &None).await;
        assert!(!result.approved);

        // 5 * 180 = 900 < 2000 → approve
        let small_signal = TradeSignal {
            quantity: 5.0,
            ..signal
        };
        let result = engine.check_pre_trade(&small_signal, &None).await;
        assert!(result.approved);
    }

    #[tokio::test]
    async fn test_zero_quantity_rejected() {
        let engine = StockRiskEngine::new();
        let signal = TradeSignal {
            id: Uuid::new_v4(),
            strategy_id: "test".to_string(),
            symbol: "AAPL".to_string(),
            side: OrderSide::Buy,
            quantity: 0.0,
            price: Some(180.0),
            order_type: OrderType::Market,
            timestamp: Utc::now(),
            reason: "test".to_string(),
            exchange: Some("polygon".to_string()),
            mode: None,
        };

        let result = engine.check_pre_trade(&signal, &None).await;
        assert!(!result.approved);
    }

    #[tokio::test]
    async fn test_sell_signal_basic_checks() {
        // Sell signals (short entries) should still pass basic pre-trade checks
        // (order value, zero quantity) without a DB — dedup requires DB
        let engine = StockRiskEngine::new();

        // Valid sell signal — should approve without DB
        let signal = TradeSignal {
            id: Uuid::new_v4(),
            strategy_id: "test".to_string(),
            symbol: "AAPL".to_string(),
            side: OrderSide::Sell,
            quantity: 5.0,
            price: Some(180.0),
            order_type: OrderType::Market,
            timestamp: Utc::now(),
            reason: "test short entry".to_string(),
            exchange: Some("polygon".to_string()),
            mode: Some("long_short".to_string()),
        };

        let result = engine.check_pre_trade(&signal, &None).await;
        assert!(result.approved);

        // Over-limit sell signal — should reject
        let big_signal = TradeSignal {
            quantity: 100.0,
            ..signal
        };
        let result = engine.check_pre_trade(&big_signal, &None).await;
        assert!(!result.approved);
        assert!(result.reason.contains("exceeds max"));
    }
}
