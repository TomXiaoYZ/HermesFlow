use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
// use solana_sdk::transaction::Transaction;
// use solana_sdk::message::Message;
use std::str::FromStr;
use std::sync::Arc;
use anyhow::{Result, anyhow};
use tracing::info;
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use base64::{Engine as _, engine::general_purpose};

// Configuration constants (TODO: Move to config file)
const JUPITER_QUOTE_API: &str = "https://quote-api.jup.ag/v6/quote";
const JUPITER_SWAP_API: &str = "https://quote-api.jup.ag/v6/swap";
const SOL_MINT: &str = "So11111111111111111111111111111111111111112";

#[derive(Clone)]
pub struct SolanaTrader {
    rpc_client: Arc<RpcClient>,
    http_client: HttpClient,
    keypair: Arc<Keypair>, 
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuoteResponse {
    #[serde(rename = "outAmount")]
    pub out_amount: String,
    // Add other fields as needed
    #[serde(flatten)]
    extra: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SwapRequest {
    #[serde(rename = "quoteResponse")]
    pub quote_response: QuoteResponse,
    #[serde(rename = "userPublicKey")]
    pub user_public_key: String,
    #[serde(rename = "wrapAndUnwrapSol")]
    pub wrap_and_unwrap_sol: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SwapResponse {
    #[serde(rename = "swapTransaction")]
    pub swap_transaction: String, // base64 encoded transaction
}

impl SolanaTrader {
    pub fn new(rpc_url: &str, private_key_base58: &str) -> Result<Self> {
        let rpc_client = Arc::new(RpcClient::new(rpc_url.to_string()));
        let keypair_bytes = bs58::decode(private_key_base58).into_vec()
            .map_err(|e| anyhow!("Invalid private key base58: {}", e))?;
        let keypair = Keypair::from_bytes(&keypair_bytes)
            .map_err(|e| anyhow!("Invalid keypair bytes: {}", e))?;

        Ok(Self {
            rpc_client,
            http_client: HttpClient::new(),
            keypair: Arc::new(keypair),
        })
    }

    pub async fn get_balance(&self) -> Result<f64> {
        let balance_lamports = self.rpc_client.get_balance(&self.keypair.pubkey())
            .map_err(|e| anyhow!("Rpc error: {}", e))?;
        Ok(balance_lamports as f64 / 1e9)
    }

    pub async fn buy(&self, token_address: &str, amount_sol: f64, slippage_bps: u16) -> Result<String> {
        info!("Executing BUY: {} SOL -> {}", amount_sol, token_address);
        
        let amount_lamports = (amount_sol * 1e9) as u64;
        
        // 1. Get Quote
        let quote = self.get_quote(SOL_MINT, token_address, amount_lamports, slippage_bps).await?;
        info!("Quote received. Est Output: {}", quote.out_amount);

        // 2. Get Swap Transaction
        let swap_tx_base64 = self.get_swap_transaction(&quote).await?;
        
        // 3. Sign and Send
        self.sign_and_send_transaction(&swap_tx_base64).await
    }
    
    pub async fn sell(&self, token_address: &str, percentage: f64, slippage_bps: u16) -> Result<String> {
        info!("Executing SELL: {}% of {} -> SOL", percentage * 100.0, token_address);
        
        // 1. Check Token Balance
        let token_pubkey = Pubkey::from_str(token_address)?;
        let accounts = self.rpc_client.get_token_accounts_by_owner(
            &self.keypair.pubkey(),
            solana_client::rpc_request::TokenAccountsFilter::Mint(token_pubkey)
        ).map_err(|e| anyhow!("Failed to get token accounts: {}", e))?;

        let mut total_balance: u64 = 0;
        for account in accounts {
            // Need to parse account data to get amount. 
            // Simplified: assuming standard layout or using `get_token_account_balance` helper if we had the pubkey
            // For now, let's use a simplified approach or placeholder as parsing raw account data in rust requires more boilerplate
            // Using get_token_account_balance on the first account found for simplicity
            let pubkey = Pubkey::from_str(&account.pubkey)?;
            let balance = self.rpc_client.get_token_account_balance(&pubkey)?;
            total_balance += balance.amount.parse::<u64>().unwrap_or(0);
        }

        if total_balance == 0 {
            return Err(anyhow!("No token balance found for {}", token_address));
        }

        let amount_to_sell = (total_balance as f64 * percentage) as u64;
        if amount_to_sell == 0 {
            return Err(anyhow!("Sell amount is 0"));
        }

        // 2. Get Quote
        let quote = self.get_quote(token_address, SOL_MINT, amount_to_sell, slippage_bps).await?;

        // 3. Get Swap Transaction
        let swap_tx_base64 = self.get_swap_transaction(&quote).await?;

        // 4. Sign and Send
        self.sign_and_send_transaction(&swap_tx_base64).await
    }

    async fn get_quote(&self, input_mint: &str, output_mint: &str, amount: u64, slippage_bps: u16) -> Result<QuoteResponse> {
        let url = format!(
            "{}?inputMint={}&outputMint={}&amount={}&slippageBps={}",
            JUPITER_QUOTE_API, input_mint, output_mint, amount, slippage_bps
        );
        
        let resp = self.http_client.get(&url).send().await?
            .json::<QuoteResponse>().await?;
            
        Ok(resp)
    }

    async fn get_swap_transaction(&self, quote: &QuoteResponse) -> Result<String> {
        // let request = SwapRequest { ... }; // Unused, using serde_json macro below
        
        // Proper way:
        let request_json = serde_json::json!({
            "quoteResponse": quote,
            "userPublicKey": self.keypair.pubkey().to_string(),
            "wrapAndUnwrapSol": true
        });

        let resp = self.http_client.post(JUPITER_SWAP_API)
            .json(&request_json)
            .send().await?;
            
        if !resp.status().is_success() {
            let text = resp.text().await?;
            return Err(anyhow!("Jupiter Swap API error: {}", text));
        }

        let swap_resp: SwapResponse = resp.json().await?;
        Ok(swap_resp.swap_transaction)
    }

    async fn sign_and_send_transaction(&self, base64_tx: &str) -> Result<String> {
        // Decode base64 to bytes
        let tx_bytes = general_purpose::STANDARD.decode(base64_tx)
            .map_err(|e| anyhow!("Base64 decode failed: {}", e))?;

        // Deserialize transaction
        // Since Jupiter returns VersionedTransaction usually, we need solana-sdk support for it.
        // For simplicity, we assume we can treat it as a blob to sign or use bincode if we strictly typed it.
        // Actually, solana-client send_transaction usually takes a Transaction struct.
        // We need to use `bincode::deserialize` to get a `VersionedTransaction` 
        // But `solana-sdk` might not have `VersionedTransaction` easily exposed in older versions used here (1.16).
        // Let's check imports.
        
        use solana_sdk::transaction::VersionedTransaction;
        let mut tx: VersionedTransaction = bincode::deserialize(&tx_bytes)
            .map_err(|e| anyhow!("Tx deserialize failed: {}", e))?;

        // Sign
        // VersionedTransaction signing is a bit different.
        let _blockhash = self.rpc_client.get_latest_blockhash()?;
        // Actually Jupiter gives a signed-ready tx mostly, we just add our signature.
        
        // We need to sign it.
        let signature = self.keypair.try_sign_message(&tx.message.serialize())?;
        
        // We need to ADD signature to the list of signatures in the tx.
        // VersionedTransaction has `signatures`.
        if tx.signatures.is_empty() {
             tx.signatures.push(signature);
        } else {
            // Replace or add? Usually the fee payer is the first signer.
            // Assuming we are the only signer or the first one.
            tx.signatures[0] = signature;
        }

        // Send
        let config = solana_client::rpc_config::RpcSendTransactionConfig {
            skip_preflight: true,
            ..Default::default()
        };
        
        let signature = self.rpc_client.send_transaction_with_config(&tx, config)?;
        
        info!("Transaction sent: {}", signature);
        Ok(signature.to_string())
    }
}
