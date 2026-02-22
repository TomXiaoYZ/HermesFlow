use common::events::{OrderSide, TradeSignal};
use reqwest::Client;
use serde::Deserialize;
use std::env;
use tracing::{info, warn};

#[derive(Debug, Clone)]
pub struct RiskConfig {
    pub min_liquidity_usd: f64,
    pub max_position_size_portion: f64, // e.g., 0.1 (10%) — crypto
    pub max_drawdown_limit: f64,        // e.g. 0.20 (20% daily)
    pub entry_amount_sol: f64,          // Default entry size in SOL (crypto)
    pub check_honeypot: bool,           // Toggle for honeypot check
    pub trade_size_pct: f64,            // % of equity per stock trade (e.g. 0.005 = 0.5%)
}

impl Default for RiskConfig {
    fn default() -> Self {
        Self {
            min_liquidity_usd: 1.0,
            max_position_size_portion: 0.5,
            max_drawdown_limit: 0.20,
            entry_amount_sol: 0.02,
            check_honeypot: env::var("CHECK_HONEYPOT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(false),
            trade_size_pct: env::var("TRADE_SIZE_PCT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0.005),
        }
    }
}

pub struct RiskEngine {
    config: RiskConfig,
    current_equity: f64,
    daily_start_equity: f64,
    http_client: Client,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct JupiterQuote {
    #[serde(rename = "outAmount")]
    out_amount: String,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct RpcResponse<T> {
    result: Option<RpcResult<T>>,
    error: Option<serde_json::Value>,
}

#[derive(Deserialize, Debug)]
struct RpcResult<T> {
    value: T,
}

#[derive(Deserialize, Debug)]
struct AccountInfo {
    data: [String; 2],
}

/// Check if a symbol looks like a US stock ticker.
/// Matches 1-5 uppercase ASCII letters, optionally followed by a dot and 1-2
/// uppercase letters (e.g. AAPL, A, BRK.A, BRK.B, PBR.A).
pub fn is_stock_symbol(symbol: &str) -> bool {
    if symbol.is_empty() || symbol.len() > 7 {
        return false;
    }
    let parts: Vec<&str> = symbol.splitn(2, '.').collect();
    let base = parts[0];
    if base.is_empty() || base.len() > 5 || !base.chars().all(|c| c.is_ascii_uppercase()) {
        return false;
    }
    if let Some(suffix) = parts.get(1) {
        !suffix.is_empty() && suffix.len() <= 2 && suffix.chars().all(|c| c.is_ascii_uppercase())
    } else {
        true
    }
}

impl Default for RiskEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl RiskEngine {
    pub fn new() -> Self {
        Self {
            config: RiskConfig::default(),
            current_equity: 0.0,
            daily_start_equity: 0.0,
            http_client: Client::new(),
        }
    }

    pub fn update_equity(&mut self, equity: f64) {
        self.current_equity = equity;
    }

    pub fn set_check_honeypot(&mut self, enabled: bool) {
        self.config.check_honeypot = enabled;
    }

    /// Calculate safe position size in SOL based on rules (for crypto)
    pub fn calculate_entry_size(&self) -> f64 {
        let max_size = self.current_equity * self.config.max_position_size_portion;
        let size = self.config.entry_amount_sol.min(max_size);

        if self.current_equity - size < 0.01 {
            return 0.0;
        }
        size
    }

    /// Calculate stock position size in shares as a percentage of account equity.
    /// trade_size_pct (e.g. 0.005 = 0.5%) determines the USD value per trade,
    /// then we floor-divide by price to get whole shares.
    pub fn calculate_stock_entry_shares(&self, current_price: f64) -> f64 {
        if current_price <= 0.0 || self.current_equity <= 0.0 {
            return 0.0;
        }
        let trade_value = self.current_equity * self.config.trade_size_pct;
        (trade_value / current_price).floor()
    }

    /// Async check for signal validity
    pub async fn check(&self, signal: &TradeSignal, liquidity: Option<f64>) -> bool {
        let is_stock = is_stock_symbol(&signal.symbol);

        // 1. Check Liquidity (skip for stocks — they have exchange-level liquidity)
        if !is_stock {
            if let Some(liq) = liquidity {
                if liq < self.config.min_liquidity_usd {
                    warn!(
                        "Risk Reject: Liquidity ${} < Min ${}",
                        liq, self.config.min_liquidity_usd
                    );
                    return false;
                }
            }
        }

        // 2. Check Drawdown
        if self.daily_start_equity > 0.0 {
            let dd = (self.daily_start_equity - self.current_equity) / self.daily_start_equity;
            if dd > self.config.max_drawdown_limit {
                warn!("Risk Reject: Daily Drawdown {:.2}% > Limit", dd * 100.0);
                return false;
            }
        }

        // 3. Honeypot Check — skip for stocks (only relevant for crypto tokens)
        if self.config.check_honeypot
            && signal.side == OrderSide::Buy
            && !is_stock
            && !self.check_honeypot(&signal.symbol).await
        {
            warn!(
                "Risk Reject: Honeypot detected/Simulation failed for {}",
                signal.symbol
            );
            return false;
        }

        true
    }

    async fn check_honeypot(&self, token_mint: &str) -> bool {
        if token_mint == "So11111111111111111111111111111111111111112" {
            return true;
        }

        let sol_mint = "So11111111111111111111111111111111111111112";
        let amount = 1_000_000;

        let url = format!(
            "https://quote-api.jup.ag/v6/quote?inputMint={}&outputMint={}&amount={}&slippageBps=100",
            token_mint, sol_mint, amount
        );

        let mut attempts = 0;
        loop {
            attempts += 1;
            match self.http_client.get(&url).send().await {
                Ok(resp) => {
                    if resp.status().is_success() {
                        if let Ok(_quote) = resp.json::<JupiterQuote>().await {
                            return true;
                        }
                    } else {
                        warn!("Honeypot check HTTP error: {}", resp.status());
                    }
                    break;
                }
                Err(e) => {
                    warn!(
                        "Honeypot check network error (Attempt {}/3): {}",
                        attempts, e
                    );
                    if attempts >= 3 {
                        warn!("Jupiter API failed 3 times. Trying RPC Fallback...");
                        return self.check_honeypot_rpc(token_mint).await;
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(500 * attempts)).await;
                }
            }
        }
        false
    }

    async fn check_honeypot_rpc(&self, token_mint: &str) -> bool {
        let rpc_url = env::var("SOLANA_RPC_URL")
            .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getAccountInfo",
            "params": [
                token_mint,
                {
                    "encoding": "base64"
                }
            ]
        });

        match self.http_client.post(&rpc_url).json(&body).send().await {
            Ok(resp) => {
                if let Ok(rpc_resp) = resp.json::<RpcResponse<AccountInfo>>().await {
                    if let Some(result) = rpc_resp.result {
                        if let Some(base64_str) = result.value.data.first() {
                            use base64::{engine::general_purpose, Engine as _};
                            if let Ok(data) = general_purpose::STANDARD.decode(base64_str) {
                                if data.len() >= 82 {
                                    let freeze_option = u32::from_le_bytes([
                                        data[46], data[47], data[48], data[49],
                                    ]);
                                    if freeze_option == 1 {
                                        warn!(
                                            "Risk Reject: Freeze Authority detected for {}",
                                            token_mint
                                        );
                                        return false;
                                    }
                                    info!(
                                        "RPC Check Passed: No Freeze Authority for {}",
                                        token_mint
                                    );
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                warn!("RPC Check failed: {}", e);
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use common::events::OrderType;
    use uuid::Uuid;

    #[test]
    fn test_is_stock_symbol() {
        assert!(is_stock_symbol("AAPL"));
        assert!(is_stock_symbol("MSFT"));
        assert!(is_stock_symbol("A"));
        // Dot-suffix share classes (e.g. BRK.A, BRK.B, PBR.A)
        assert!(is_stock_symbol("BRK.A"));
        assert!(is_stock_symbol("BRK.B"));
        assert!(is_stock_symbol("PBR.A"));
        // Negative cases
        assert!(!is_stock_symbol(
            "So11111111111111111111111111111111111111112"
        ));
        assert!(!is_stock_symbol("sol"));
        assert!(!is_stock_symbol(""));
        assert!(!is_stock_symbol("TOOLONGSYMBOL"));
        assert!(!is_stock_symbol("A.BCD")); // suffix too long
        assert!(!is_stock_symbol(".A")); // empty base
    }

    #[tokio::test]
    async fn test_position_sizing_logic() {
        let mut engine = RiskEngine::new();
        engine.update_equity(0.5);

        let size = engine.calculate_entry_size();
        assert_eq!(size, 0.02);

        engine.update_equity(0.015);
        let size = engine.calculate_entry_size();
        assert_eq!(size, 0.0);
    }

    #[test]
    fn test_stock_entry_shares_pct_based() {
        let mut engine = RiskEngine::new();
        // Default trade_size_pct = 0.005 (0.5%)
        // Equity = $1,000,000 → trade_value = $5,000
        engine.update_equity(1_000_000.0);
        let shares = engine.calculate_stock_entry_shares(180.0);
        assert_eq!(shares, 27.0); // floor(5000/180) = 27

        // Equity = $100,000 → trade_value = $500
        engine.update_equity(100_000.0);
        let shares = engine.calculate_stock_entry_shares(180.0);
        assert_eq!(shares, 2.0); // floor(500/180) = 2

        // Zero equity → 0 shares
        engine.update_equity(0.0);
        let shares = engine.calculate_stock_entry_shares(180.0);
        assert_eq!(shares, 0.0);

        // Zero price → 0 shares
        engine.update_equity(1_000_000.0);
        let shares = engine.calculate_stock_entry_shares(0.0);
        assert_eq!(shares, 0.0);
    }

    #[tokio::test]
    async fn test_risk_check_liquidity() {
        let mut engine = RiskEngine::new();
        engine.set_check_honeypot(false);

        let signal = TradeSignal {
            id: Uuid::new_v4(),
            strategy_id: "test".to_string(),
            symbol: "TEST_TOKEN_LONG_NAME_XXXX".to_string(),
            side: OrderSide::Buy,
            quantity: 1.0,
            price: Some(1.0),
            order_type: OrderType::Market,
            timestamp: Utc::now(),
            reason: "Testing".to_string(),
            exchange: None,
            mode: None,
        };

        assert!(engine.check(&signal, Some(1000.0)).await);
        assert!(!engine.check(&signal, Some(0.5)).await);
    }

    #[tokio::test]
    async fn test_stock_risk_checks() {
        let mut engine = RiskEngine::new();
        engine.set_check_honeypot(false);
        engine.update_equity(1_000_000.0);

        // Stock signal — should skip liquidity & honeypot checks
        let signal = TradeSignal {
            id: Uuid::new_v4(),
            strategy_id: "test".to_string(),
            symbol: "AAPL".to_string(),
            side: OrderSide::Buy,
            quantity: 10.0,
            price: Some(180.0),
            order_type: OrderType::Market,
            timestamp: Utc::now(),
            reason: "Testing".to_string(),
            exchange: Some("polygon".to_string()),
            mode: Some("long_only".to_string()),
        };

        // Stock entry with equity → approved (no fixed limits)
        assert!(engine.check(&signal, None).await);
    }
}
