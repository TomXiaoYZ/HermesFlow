use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::Utc;
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use super::{AccountSummary, BrokerOrderType, BrokerPosition, OrderParams, OrderResult, Trader};

/// FutuTrader communicates with the futu-bridge Python sidecar via HTTP.
#[derive(Clone)]
pub struct FutuTrader {
    http: HttpClient,
    base_url: String,
}

#[derive(Debug, Serialize)]
struct PlaceOrderRequest {
    symbol: String,
    side: String,
    quantity: f64,
    order_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    limit_price: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct PlaceOrderResponse {
    order_id: String,
    status: String,
    broker: String,
}

#[derive(Debug, Deserialize)]
struct PositionItem {
    symbol: String,
    quantity: f64,
    avg_cost: f64,
    market_value: f64,
    unrealized_pnl: f64,
}

#[derive(Debug, Deserialize)]
struct AccountResponse {
    net_liquidation: f64,
    cash: f64,
    buying_power: f64,
    currency: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct HealthResponse {
    status: String,
    opend_connected: bool,
}

impl FutuTrader {
    pub async fn new(bridge_url: &str) -> Result<Self> {
        let http = HttpClient::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        let base_url = bridge_url.trim_end_matches('/').to_string();

        // Verify connectivity
        let health_url = format!("{}/health", base_url);
        let resp = http
            .get(&health_url)
            .send()
            .await
            .map_err(|e| anyhow!("Futu bridge unreachable at {}: {}", base_url, e))?;

        let health: HealthResponse = resp
            .json()
            .await
            .map_err(|e| anyhow!("Futu bridge health check parse error: {}", e))?;

        if !health.opend_connected {
            warn!("Futu bridge is running but OpenD is not connected (degraded mode)");
        }

        info!(
            "Futu bridge connected at {} (opend={})",
            base_url, health.opend_connected
        );

        Ok(Self { http, base_url })
    }

    fn map_order_type(ot: &BrokerOrderType) -> &'static str {
        match ot {
            BrokerOrderType::Market => "Market",
            BrokerOrderType::Limit => "Limit",
            BrokerOrderType::MarketOnClose => "MarketOnClose",
        }
    }

    async fn place_order(
        &self,
        symbol: &str,
        quantity: f64,
        side: &str,
        params: &OrderParams,
    ) -> Result<OrderResult> {
        let req = PlaceOrderRequest {
            symbol: symbol.to_string(),
            side: side.to_string(),
            quantity,
            order_type: Self::map_order_type(&params.order_type).to_string(),
            limit_price: params.limit_price,
        };

        let url = format!("{}/api/order", self.base_url);

        info!(
            "Futu {}: {} x{} (type={})",
            side, symbol, quantity, req.order_type
        );

        let resp = self
            .http
            .post(&url)
            .json(&req)
            .send()
            .await
            .map_err(|e| anyhow!("Futu bridge request failed: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Futu bridge returned {}: {}", status, body));
        }

        let result: PlaceOrderResponse = resp
            .json()
            .await
            .map_err(|e| anyhow!("Futu bridge response parse error: {}", e))?;

        Ok(OrderResult {
            order_id: result.order_id,
            status: result.status,
            filled_qty: 0.0,
            avg_price: 0.0,
            broker: "Futu".to_string(),
            timestamp: Utc::now(),
        })
    }
}

#[async_trait]
impl Trader for FutuTrader {
    fn broker_name(&self) -> &str {
        "Futu"
    }

    async fn buy(&self, symbol: &str, quantity: f64, params: &OrderParams) -> Result<OrderResult> {
        self.place_order(symbol, quantity, "Buy", params).await
    }

    async fn sell(&self, symbol: &str, quantity: f64, params: &OrderParams) -> Result<OrderResult> {
        self.place_order(symbol, quantity, "Sell", params).await
    }

    async fn cancel_order(&self, order_id: &str) -> Result<()> {
        let url = format!("{}/api/order/{}", self.base_url, order_id);

        info!("Futu cancelling order {}", order_id);

        let resp = self
            .http
            .delete(&url)
            .send()
            .await
            .map_err(|e| anyhow!("Futu bridge cancel request failed: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Futu bridge cancel returned {}: {}", status, body));
        }

        Ok(())
    }

    async fn get_positions(&self) -> Result<Vec<BrokerPosition>> {
        let url = format!("{}/api/positions", self.base_url);

        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| anyhow!("Futu bridge positions request failed: {}", e))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Futu bridge positions error: {}", body));
        }

        let items: Vec<PositionItem> = resp
            .json()
            .await
            .map_err(|e| anyhow!("Futu positions parse error: {}", e))?;

        Ok(items
            .into_iter()
            .map(|p| BrokerPosition {
                symbol: p.symbol,
                quantity: p.quantity,
                avg_cost: p.avg_cost,
                market_value: p.market_value,
                unrealized_pnl: p.unrealized_pnl,
                account: String::new(),
            })
            .collect())
    }

    async fn get_account_summary(&self) -> Result<AccountSummary> {
        let url = format!("{}/api/account", self.base_url);

        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| anyhow!("Futu bridge account request failed: {}", e))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow!("Futu bridge account error: {}", body));
        }

        let acct: AccountResponse = resp
            .json()
            .await
            .map_err(|e| anyhow!("Futu account parse error: {}", e))?;

        Ok(AccountSummary {
            net_liquidation: acct.net_liquidation,
            cash: acct.cash,
            buying_power: acct.buying_power,
            currency: acct.currency,
        })
    }
}
