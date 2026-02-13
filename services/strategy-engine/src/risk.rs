use common::events::{OrderSide, TradeSignal};
use reqwest::Client;
use serde::Deserialize;
use tracing::{info, warn};

#[derive(Debug, Clone)]
pub struct RiskConfig {
    pub min_liquidity_usd: f64,
    pub max_position_size_portion: f64, // e.g., 0.1 (10%)
    pub max_drawdown_limit: f64,        // e.g. 0.05 (5% daily)
    pub entry_amount_sol: f64,          // Default entry size in SOL (e.g. 0.1)
    pub check_honeypot: bool,           // Toggle for honeypot check
}

impl Default for RiskConfig {
    fn default() -> Self {
        Self {
            min_liquidity_usd: 1.0,
            max_position_size_portion: 0.5,
            max_drawdown_limit: 0.20,
            entry_amount_sol: 0.02, // 0.02 SOL default testing
            check_honeypot: true,
        }
    }
}

pub struct RiskEngine {
    config: RiskConfig,
    current_equity: f64, // Mock current equity (USD or SOL?) - Let's assume SOL for simplicity or track USD.
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
    data: [String; 2], // ["base64_data", "base64"]
                       // ... we just need data
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

    /// Calculate safe position size in SOL based on rules
    pub fn calculate_entry_size(&self) -> f64 {
        let max_size = self.current_equity * self.config.max_position_size_portion;
        // Limit to configured entry amount or max size
        let size = self.config.entry_amount_sol.min(max_size);

        if self.current_equity - size < 0.01 {
            // Leave dust for fees
            return 0.0;
        }
        size
    }

    /// Async check for signal validity
    pub async fn check(&self, signal: &TradeSignal, liquidity: Option<f64>) -> bool {
        // 1. Check Liquidity
        if let Some(liq) = liquidity {
            if liq < self.config.min_liquidity_usd {
                warn!(
                    "Risk Reject: Liquidity ${} < Min ${}",
                    liq, self.config.min_liquidity_usd
                );
                return false;
            }
        }

        // 2. Check Drawdown
        let dd = (self.daily_start_equity - self.current_equity) / self.daily_start_equity;
        if dd > self.config.max_drawdown_limit {
            warn!("Risk Reject: Daily Drawdown {:.2}% > Limit", dd * 100.0);
            return false;
        }

        // 3. Honeypot Check (Sell Simulation)
        if self.config.check_honeypot
            && signal.side == OrderSide::Buy
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
        // Skip for SOL
        if token_mint == "So11111111111111111111111111111111111111112" {
            return true;
        }

        // Simulate selling 1000 tokens (arbitrary unit) -> SOL
        // If route exists, likelihood of honeypot is lower (but not zero).
        // Real honeypot check involves simulation of transaction.
        // AlphaGPT used a "Quote Check" as a proxy.
        // URL: https://quote-api.jup.ag/v6/quote?inputMint=XXX&outputMint=SOL&amount=1000000&slippageBps=50

        let sol_mint = "So11111111111111111111111111111111111111112";
        let amount = 1_000_000; // 1M atoms of token (assuming 6 decimals = 1 token, or just small dust)

        let url = format!(
            "https://quote-api.jup.ag/v6/quote?inputMint={}&outputMint={}&amount={}&slippageBps=100",
            token_mint, sol_mint, amount
        );

        // Retry logic (3 attempts)
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
                        // 400/500 errors
                        warn!("Honeypot check HTTP error: {}", resp.status());
                    }
                    // If we got a response but it wasn't valid, maybe don't retry immediately if it's 400.
                    // But if it's 500, retry.
                    break;
                }
                Err(e) => {
                    warn!(
                        "Honeypot check network error (Attempt {}/3): {}",
                        attempts, e
                    );
                    if attempts >= 3 {
                        // Fallback to RPC Check if API fails completely
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
        let rpc_url = std::env::var("SOLANA_RPC_URL")
            .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());

        // JSON RPC Request
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
                            // Decode Base64
                            use base64::{engine::general_purpose, Engine as _};
                            if let Ok(data) = general_purpose::STANDARD.decode(base64_str) {
                                // SPL Token Mint Layout: 82 bytes
                                // Offset 46-49: Freeze Authority Option (u32)
                                // Offset 50-81: Freeze Authority Key (32 bytes)

                                if data.len() >= 82 {
                                    // Check offset 46 (Option tag)
                                    // 0 = None, 1 = Some
                                    let freeze_option = u32::from_le_bytes([
                                        data[46], data[47], data[48], data[49],
                                    ]);
                                    if freeze_option == 1 {
                                        // FREEZE AUTHORITY EXISTS!
                                        // Check if it's not null (sometimes it's 1 but key is zero? unlikely for standard SPL)
                                        // But safe bet: If Freeze Auth is set, REJECT.
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

        // If RPC also fails, we default to reject (conservative) or accept?
        // Since network is bad, maybe reject to be safe.
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use common::events::OrderType;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_position_sizing_logic() {
        let mut engine = RiskEngine::new();
        engine.update_equity(0.5);
        // Config: 0.5 portion, 0.02 entry default, current equity 0.5 SOL
        // Max portion size = 0.5 * 0.5 = 0.25 SOL
        // Entry default = 0.02 SOL
        // Min(0.25, 0.02) = 0.02
        // Remainder = 0.5 - 0.02 = 0.48 > 0.01. OK.

        let size = engine.calculate_entry_size();
        assert_eq!(size, 0.02);

        // Test Low Balance Cap
        engine.update_equity(0.015); // Very low
                                     // Max portion = 0.015 * 0.5 = 0.0075
                                     // Entry = 0.02
                                     // Min = 0.0075
                                     // Remainder = 0.015 - 0.0075 = 0.0075 < 0.01 (dust limit)
                                     // Should return 0.0
        let size = engine.calculate_entry_size();
        assert_eq!(size, 0.0);
    }

    #[tokio::test]
    async fn test_risk_check_liquidity() {
        let mut engine = RiskEngine::new();
        engine.set_check_honeypot(false); // Disable network call for unit test

        let signal = TradeSignal {
            id: Uuid::new_v4(),
            strategy_id: "test".to_string(),
            symbol: "TEST".to_string(),
            side: OrderSide::Buy,
            quantity: 1.0,
            price: Some(1.0),
            order_type: OrderType::Market,
            timestamp: Utc::now(),
            reason: "Testing".to_string(),
        };

        // Pass
        assert!(engine.check(&signal, Some(1000.0)).await);
        // Fail
        assert!(!engine.check(&signal, Some(0.5)).await);
    }
}
