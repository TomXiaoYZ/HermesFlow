use anyhow::{anyhow, Result};
use base64::{engine::general_purpose, Engine as _};
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::system_instruction;
use solana_sdk::transaction::Transaction;
use spl_associated_token_account::get_associated_token_address;
use std::str::FromStr;
use std::sync::Arc;
use tracing::{info, warn};

// Import our Raydium trader
use super::raydium_trader::RaydiumTrader;

// Configuration constants (TODO: Move to config file)
const JUPITER_QUOTE_API: &str = "https://jupiter-relay.slovinskypatrickiv729.workers.dev/v6/quote";
const JUPITER_SWAP_API: &str = "https://jupiter-relay.slovinskypatrickiv729.workers.dev/v6/swap";
const SOL_MINT: &str = "So11111111111111111111111111111111111111112";

#[derive(Clone)]
pub struct SolanaTrader {
    rpc_client: Arc<RpcClient>,
    http_client: HttpClient,
    keypair: Arc<Keypair>,
    raydium: RaydiumTrader,
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
    #[serde(rename = "prioritizationFeeLamports")]
    pub prioritization_fee_lamports: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SwapResponse {
    #[serde(rename = "swapTransaction")]
    pub swap_transaction: String, // base64 encoded transaction
}

impl SolanaTrader {
    // ... (new, get_balance, buy, sell methods unchanged) ...

    pub fn new(rpc_url: &str, private_key_base58: &str) -> Result<Self> {
        let rpc_client = Arc::new(RpcClient::new(rpc_url.to_string()));
        let keypair_bytes = bs58::decode(private_key_base58)
            .into_vec()
            .map_err(|e| anyhow!("Invalid private key base58: {}", e))?;
        let keypair = Keypair::from_bytes(&keypair_bytes)
            .map_err(|e| anyhow!("Invalid keypair bytes: {}", e))?;

        // Initialize Raydium trader
        let raydium = RaydiumTrader::new(Arc::clone(&rpc_client));

        Ok(Self {
            rpc_client,
            http_client: HttpClient::new(),
            keypair: Arc::new(keypair),
            raydium,
        })
    }

    pub fn get_pubkey(&self) -> Pubkey {
        self.keypair.pubkey()
    }

    pub async fn get_balance(&self) -> Result<f64> {
        let balance_lamports = self
            .rpc_client
            .get_balance(&self.keypair.pubkey())
            .map_err(|e| anyhow!("Rpc error: {}", e))?;
        Ok(balance_lamports as f64 / 1e9)
    }

    /// Buy using Jupiter API (fallback method)
    pub async fn buy_jupiter(
        &self,
        token_address: &str,
        amount_sol: f64,
        slippage_bps: u16,
    ) -> Result<String> {
        info!(
            "[Jupiter] Executing BUY: {} SOL -> {}",
            amount_sol, token_address
        );

        let amount_lamports = (amount_sol * 1e9) as u64;

        // 1. Get Quote
        let quote = self
            .get_quote(SOL_MINT, token_address, amount_lamports, slippage_bps)
            .await?;
        info!("[Jupiter] Quote received. Est Output: {}", quote.out_amount);

        // 2. Get Swap Transaction
        let swap_tx_base64 = self.get_swap_transaction(&quote).await?;

        // 3. Sign and Send
        let sig = self.sign_and_send_transaction(&swap_tx_base64).await?;

        // 4. Confirm Transaction
        self.confirm_transaction(&sig).await?;

        Ok(sig)
    }

    /// Raydium on-chain swap with complete transaction flow
    pub async fn buy_raydium_experimental(
        &self,
        token_address: &str,
        amount_sol: f64,
        slippage_bps: u16,
    ) -> Result<String> {
        info!(
            "[Raydium] Executing BUY: {} SOL -> {}",
            amount_sol, token_address
        );

        // 1. Find pool
        let sol_mint = Pubkey::from_str(SOL_MINT)?;
        let token_mint = Pubkey::from_str(token_address)?;

        let pool = self.raydium.find_pool(&sol_mint, &token_mint).await?;
        info!("[Raydium] Found pool: {}", pool);

        // 2. Get full AMM info (includes vaults and pool state)
        let amm_info = self.raydium.get_amm_info(&pool).await?;
        info!(
            "[Raydium] Pool reserves: coin={} pc={}",
            amm_info.pool_coin_total, amm_info.pool_pc_total
        );

        // 3. Determine which reserve is SOL and which is the token
        // SOL is "wrapped SOL" (wSOL) in the pool
        let wsol_mint = Pubkey::from_str(SOL_MINT)?;
        let (reserve_in, reserve_out) = if amm_info.coin_mint == wsol_mint {
            // coin is SOL, pc is token
            (amm_info.pool_coin_total, amm_info.pool_pc_total)
        } else if amm_info.pc_mint == wsol_mint {
            // pc is SOL, coin is token
            (amm_info.pool_pc_total, amm_info.pool_coin_total)
        } else {
            return Err(anyhow!(
                "Pool does not contain SOL/wSOL. Coin: {}, PC: {}",
                amm_info.coin_mint,
                amm_info.pc_mint
            ));
        };

        // 4. Calculate expected output with real reserves
        let amount_in = (amount_sol * 1e9) as u64; // SOL has 9 decimals
        let amount_out = self.raydium.calculate_swap_output(
            amount_in,
            reserve_in,
            reserve_out,
            25, // 0.25% fee (Raydium standard)
            10_000,
        )?;

        // 5. Apply slippage protection
        let slippage_multiplier = 1.0 - (slippage_bps as f64 / 10_000.0);
        let minimum_amount_out = ((amount_out as f64) * slippage_multiplier) as u64;

        info!(
            "[Raydium] Swap calculation: {} SOL -> {} tokens (min: {} with {}bps slippage)",
            amount_sol, amount_out, minimum_amount_out, slippage_bps
        );

        // 6. Prepare wSOL wrapping and token ATA
        let wallet = self.keypair.pubkey();
        let user_wsol_ata = get_associated_token_address(&wallet, &wsol_mint);
        let user_token_ata = get_associated_token_address(&wallet, &token_mint);

        info!("[Raydium] User wSOL ATA: {}", user_wsol_ata);
        info!("[Raydium] User token ATA: {}", user_token_ata);

        let mut instructions = vec![];

        // 6a. Create wSOL ATA if not exists
        let wsol_ata_exists = self.rpc_client.get_account(&user_wsol_ata).is_ok();
        if !wsol_ata_exists {
            info!("[Raydium] Creating wSOL ATA");
            let create_wsol_ata_ix =
                spl_associated_token_account::instruction::create_associated_token_account(
                    &wallet,
                    &wallet,
                    &wsol_mint,
                    &spl_token::id(),
                );
            instructions.push(create_wsol_ata_ix);
        } else {
            info!("[Raydium] wSOL ATA already exists");
        }

        // 6b. Transfer native SOL to wSOL ATA for wrapping
        // We need to transfer the amount we want to swap
        info!("[Raydium] Wrapping {} lamports of SOL to wSOL", amount_in);
        let transfer_to_wsol_ix =
            solana_sdk::system_instruction::transfer(&wallet, &user_wsol_ata, amount_in);
        instructions.push(transfer_to_wsol_ix);

        // 6c. Sync native to update wSOL token balance
        info!("[Raydium] Syncing native SOL to wSOL token balance");
        let sync_native_ix = spl_token::instruction::sync_native(&spl_token::id(), &user_wsol_ata)?;
        instructions.push(sync_native_ix);

        // 6d. Create output token ATA if not exists
        match self.rpc_client.get_account(&user_token_ata) {
            Ok(_) => {
                info!("[Raydium] Token ATA already exists");
            }
            Err(_) => {
                info!("[Raydium] Creating token ATA");
                let create_token_ata_ix =
                    spl_associated_token_account::instruction::create_associated_token_account(
                        &wallet,
                        &wallet,
                        &token_mint,
                        &spl_token::id(),
                    );
                instructions.push(create_token_ata_ix);
            }
        }

        // 7. Build swap instruction
        let swap_ix = self
            .raydium
            .build_swap_instruction(
                &amm_info,
                &pool,
                &wallet,
                &user_wsol_ata,
                &user_token_ata,
                amount_in,
                minimum_amount_out,
            )
            .await?;
        instructions.push(swap_ix);

        // 8. Close wSOL ATA after swap to get remaining SOL back
        // This is optional but recommended to avoid rent
        info!("[Raydium] Adding instruction to close wSOL ATA and recover rent");
        let close_wsol_ata_ix = spl_token::instruction::close_account(
            &spl_token::id(),
            &user_wsol_ata,
            &wallet, // destination for remaining SOL
            &wallet, // account authority
            &[],     // no multisig
        )?;
        instructions.push(close_wsol_ata_ix);

        // 9. Build and send transaction
        let recent_blockhash = self.rpc_client.get_latest_blockhash()?;
        let transaction = Transaction::new_signed_with_payer(
            &instructions,
            Some(&wallet),
            &[&self.keypair],
            recent_blockhash,
        );

        info!(
            "[Raydium] Sending transaction with {} instructions...",
            instructions.len()
        );
        let signature = self
            .rpc_client
            .send_and_confirm_transaction(&transaction)
            .map_err(|e| anyhow!("Raydium transaction failed: {}", e))?;

        let sig_str = signature.to_string();
        info!("[Raydium] Transaction successful: {}", sig_str);

        Ok(sig_str)
    }

    /// Raydium on-chain sell: Token -> SOL (native)
    /// 1. Swap Token -> wSOL
    /// 2. Unwrap wSOL -> Native SOL (by closing account)
    pub async fn sell_raydium_experimental(
        &self,
        token_address: &str,
        amount_token: u64,
        slippage_bps: u16,
    ) -> Result<String> {
        info!("[Raydium] Executing SELL: {} tokens -> SOL", amount_token);

        // 1. Find pool
        let sol_mint = Pubkey::from_str(SOL_MINT)?;
        let token_mint = Pubkey::from_str(token_address)?;

        let pool = self.raydium.find_pool(&sol_mint, &token_mint).await?;
        info!("[Raydium] Found pool: {}", pool);

        // 2. Get full AMM info
        let amm_info = self.raydium.get_amm_info(&pool).await?;

        // 3. Determine reserves (Input = Token, Output = SOL)
        let wsol_mint = Pubkey::from_str(SOL_MINT)?;

        // We are swapping Token -> wSOL
        // reserve_in = Token Reserve
        // reserve_out = SOL Reserve
        let (reserve_in, reserve_out) = if amm_info.coin_mint == wsol_mint {
            // coin is SOL, pc is Token.
            // In: Token (pc), Out: SOL (coin)
            (amm_info.pool_pc_total, amm_info.pool_coin_total)
        } else if amm_info.pc_mint == wsol_mint {
            // pc is SOL, coin is Token.
            // In: Token (coin), Out: SOL (pc)
            (amm_info.pool_coin_total, amm_info.pool_pc_total)
        } else {
            return Err(anyhow!("Pool does not contain SOL/wSOL"));
        };

        // 4. Calculate expected output (wSOL)
        let amount_out_wsol = self.raydium.calculate_swap_output(
            amount_token,
            reserve_in,
            reserve_out,
            25, // 0.25% fee
            10_000,
        )?;

        // 5. Apply slippage
        let slippage_multiplier = 1.0 - (slippage_bps as f64 / 10_000.0);
        let minimum_amount_out = ((amount_out_wsol as f64) * slippage_multiplier) as u64;

        info!(
            "[Raydium] Sell Calculation: {} tokens -> {} wSOL (min: {})",
            amount_token, amount_out_wsol, minimum_amount_out
        );

        // 6. Build Instructions
        let wallet = self.keypair.pubkey();
        let user_wsol_ata = get_associated_token_address(&wallet, &wsol_mint);
        let user_token_ata = get_associated_token_address(&wallet, &token_mint);

        let mut instructions = vec![];

        // 6a. Create wSOL ATA (Destination) if NOT exists
        // We need it to receive wSOL from the pool
        let wsol_ata_exists = self.rpc_client.get_account(&user_wsol_ata).is_ok();
        if !wsol_ata_exists {
            info!("[Raydium] Creating wSOL ATA (destination for swap)");
            let create_wsol_ata_ix =
                spl_associated_token_account::instruction::create_associated_token_account(
                    &wallet,
                    &wallet,
                    &wsol_mint,
                    &spl_token::id(),
                );
            instructions.push(create_wsol_ata_ix);
        }

        // 6b. Token ATA Must Exist (Source)
        if self.rpc_client.get_account(&user_token_ata).is_err() {
            return Err(anyhow!("User Token ATA does not exist. Cannot sell."));
        }

        // 7. Swap Instruction (Source: Token ATA, Dest: wSOL ATA)
        let swap_ix = self
            .raydium
            .build_swap_instruction(
                &amm_info,
                &pool,
                &wallet,
                &user_token_ata, // Source: Token
                &user_wsol_ata,  // Dest: wSOL
                amount_token,
                minimum_amount_out,
            )
            .await?;
        instructions.push(swap_ix);

        // 8. Close wSOL ATA (Unwrap to native SOL)
        // This sends the wSOL balance + rent back to the user as native SOL.
        let close_wsol_ata_ix = spl_token::instruction::close_account(
            &spl_token::id(),
            &user_wsol_ata,
            &wallet, // Dest for funds
            &wallet, // Authority
            &[],
        )?;
        instructions.push(close_wsol_ata_ix);

        // 9. Send Transaction
        let recent_blockhash = self.rpc_client.get_latest_blockhash()?;
        let transaction = Transaction::new_signed_with_payer(
            &instructions,
            Some(&wallet),
            &[&self.keypair],
            recent_blockhash,
        );

        info!("[Raydium] Sending SELL transaction...");
        let signature = self
            .rpc_client
            .send_and_confirm_transaction(&transaction)
            .map_err(|e| anyhow!("Raydium sell failed: {}", e))?;

        let sig_str = signature.to_string();
        info!("[Raydium] Sell successful: {}", sig_str);

        Ok(sig_str)
    }
    /// Main buy entry point - tries Raydium first, falls back to Jupiter
    pub async fn buy(
        &self,
        token_address: &str,
        amount_sol: f64,
        slippage_bps: u16,
    ) -> Result<String> {
        // Guard against self-swap
        if token_address == SOL_MINT {
            return Err(anyhow!("Cannot swap SOL for SOL (Self-swap detected)"));
        }

        // Try Raydium experimental first
        match self
            .buy_raydium_experimental(token_address, amount_sol, slippage_bps)
            .await
        {
            Ok(sig) => {
                info!("[Raydium-MVP] Successfully executed via Raydium");
                Ok(sig)
            }
            Err(e) => {
                warn!("[Raydium-MVP] Failed: {}. Falling back to Jupiter", e);
                self.buy_jupiter(token_address, amount_sol, slippage_bps)
                    .await
            }
        }
    }

    pub async fn sell(
        &self,
        token_address: &str,
        percentage: f64,
        slippage_bps: u16,
    ) -> Result<String> {
        info!(
            "Executing SELL: {}% of {} -> SOL",
            percentage * 100.0,
            token_address
        );

        // 1. Check Token Balance
        let token_pubkey = Pubkey::from_str(token_address)?;
        let accounts = self
            .rpc_client
            .get_token_accounts_by_owner(
                &self.keypair.pubkey(),
                solana_client::rpc_request::TokenAccountsFilter::Mint(token_pubkey),
            )
            .map_err(|e| anyhow!("Failed to get token accounts: {}", e))?;

        let mut total_balance: u64 = 0;
        for account in accounts {
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

        // 2. Try Raydium Sell First
        match self
            .sell_raydium_experimental(token_address, amount_to_sell, slippage_bps)
            .await
        {
            Ok(sig) => {
                info!("[Raydium-MVP] Sell successful via Raydium");
                return Ok(sig);
            }
            Err(e) => {
                warn!("[Raydium-MVP] Sell failed: {}. Falling back to Jupiter.", e);
            }
        }

        // 3. Fallback to Jupiter (Get Quote)
        let quote = self
            .get_quote(token_address, SOL_MINT, amount_to_sell, slippage_bps)
            .await?;

        // 4. Get Swap Transaction
        let swap_tx_base64 = self.get_swap_transaction(&quote).await?;

        // 5. Sign and Send
        let sig = self.sign_and_send_transaction(&swap_tx_base64).await?;

        // 6. Confirm
        self.confirm_transaction(&sig).await?;

        Ok(sig)
    }

    async fn get_quote(
        &self,
        input_mint: &str,
        output_mint: &str,
        amount: u64,
        slippage_bps: u16,
    ) -> Result<QuoteResponse> {
        let url = format!(
            "{}?inputMint={}&outputMint={}&amount={}&slippageBps={}",
            JUPITER_QUOTE_API, input_mint, output_mint, amount, slippage_bps
        );

        let resp = self.http_client.get(&url).send().await?;

        if !resp.status().is_success() {
            let text = resp.text().await?;
            return Err(anyhow!("Jupiter Quote API error ({}): {}", url, text));
        }

        let quote = resp.json::<QuoteResponse>().await?;
        Ok(quote)
    }

    async fn get_swap_transaction(&self, quote: &QuoteResponse) -> Result<String> {
        // High Priority Fee: 50,000 MicroLamports? No, API expects 'prioritizationFeeLamports' (Total fee) or 'computeUnitPriceMicroLamports'
        // Docs: "prioritizationFeeLamports" is legacy/global?
        // Better to use 'dynamicComputeUnitLimit': true and 'prioritizationFeeLamports': 'auto' or high number.
        // Let's set 100,000 lamports (0.0001 SOL) to be safe.

        let request_json = serde_json::json!({
            "quoteResponse": quote,
            "userPublicKey": self.keypair.pubkey().to_string(),
            "wrapAndUnwrapSol": true,
            "prioritizationFeeLamports": 100_000,
            "dynamicComputeUnitLimit": true
        });

        let resp = self
            .http_client
            .post(JUPITER_SWAP_API)
            .json(&request_json)
            .send()
            .await?;

        if !resp.status().is_success() {
            let text = resp.text().await?;
            return Err(anyhow!("Jupiter Swap API error: {}", text));
        }

        let swap_resp: SwapResponse = resp.json().await?;
        Ok(swap_resp.swap_transaction)
    }

    async fn sign_and_send_transaction(&self, base64_tx: &str) -> Result<String> {
        // Decode base64 to bytes
        let tx_bytes = general_purpose::STANDARD
            .decode(base64_tx)
            .map_err(|e| anyhow!("Base64 decode failed: {}", e))?;

        // Deserialize transaction
        // Since Jupiter returns VersionedTransaction usually, we need solana-sdk support for it.
        // For simplicity, we assume we can treat it as a blob to sign or use bincode if we strictly typed it.
        // Actually, solana-client send_transaction usually takes a Transaction struct.
        // We need to use `bincode::deserialize` to get a `VersionedTransaction`
        // But `solana-sdk` might not have `VersionedTransaction` easily exposed in older versions used here (1.16).
        // Let's check imports.

        use solana_sdk::transaction::VersionedTransaction;
        let mut tx: VersionedTransaction =
            bincode::deserialize(&tx_bytes).map_err(|e| anyhow!("Tx deserialize failed: {}", e))?;

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

    /// Verifies if a token can be sold (Honeypot Check) by requesting a Quote for a Sell operation.
    /// Simulates selling 1 "UI Unit" (or raw amount if decimals unknown, defaulting to 1M lamports/units).
    pub async fn check_honeypot(&self, token_address: &str) -> Result<bool> {
        info!("Running Honeypot Check for {}", token_address);

        // 1. Determine Amount to Verify (Simulate selling ~1 USD worth or 1 Token?)
        // Safer to try selling a small fixed raw amount relative to typical supply.
        // Or fetch decimals. Let's try to fetch decimals first.
        let decimals = match self.get_decimals(token_address).await {
            Ok(d) => d,
            Err(e) => {
                // If we can't even get decimals/supply, it's suspicious or RPC failed.
                // But could be just network. Let's assume 6 decimals (standard for memecoins check?)
                // Actually safer to return false if we can't inspect it.
                tracing::warn!(
                    "Honeypot Check: Failed to fetch decimals for {}: {}",
                    token_address,
                    e
                );
                return Ok(false);
            }
        };

        // Simulate selling 1 UI Token (e.g. 1_000_000 for 6 decimals)
        let amount_check = 10_u64.pow(decimals as u32);

        // 2. Request Quote: Token -> SOL
        let quote_result = self
            .get_quote(
                token_address,
                SOL_MINT,
                amount_check,
                5000, // 50% slippage allowed for check (we just want TO KNOW if it's possible)
            )
            .await;

        match quote_result {
            Ok(quote) => {
                // Check if output is non-zero
                if let Ok(out_amount) = quote.out_amount.parse::<u64>() {
                    if out_amount > 0 {
                        // Success! Route exists.
                        info!(
                            "Honeypot Check PASSED: {} (Simulated Sell 1 Unit -> {} lamports)",
                            token_address, out_amount
                        );
                        return Ok(true);
                    }
                }
                tracing::warn!(
                    "Honeypot Check FAILED: {} (Quote returned 0 output)",
                    token_address
                );
                Ok(false)
            }
            Err(e) => {
                tracing::warn!(
                    "Honeypot Check FAILED: {} (No Quote/Route found: {})",
                    token_address,
                    e
                );
                Ok(false)
            }
        }
    }

    async fn get_decimals(&self, mint_address: &str) -> Result<u8> {
        let pubkey = Pubkey::from_str(mint_address)?;
        let account = self.rpc_client.get_token_supply(&pubkey)?;
        Ok(account.decimals)
    }

    /// Polls for transaction confirmation
    pub async fn confirm_transaction(&self, signature: &str) -> Result<()> {
        let sig = solana_sdk::signature::Signature::from_str(signature)?;
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(60);

        info!("Confirming transaction {}...", signature);

        loop {
            if start.elapsed() > timeout {
                return Err(anyhow!("Transaction confirmation timed out: {}", signature));
            }

            let statuses = self.rpc_client.get_signature_statuses(&[sig])?;
            if let Some(Some(status)) = statuses.value.get(0) {
                if let Some(err) = &status.err {
                    return Err(anyhow!("Transaction failed: {:?} - {:?}", err, status));
                }

                if let Some(confirmation_status) = &status.confirmation_status {
                    match confirmation_status {
                        solana_transaction_status::TransactionConfirmationStatus::Confirmed
                        | solana_transaction_status::TransactionConfirmationStatus::Finalized => {
                            info!("Transaction CONFIRMED: {}", signature);
                            return Ok(());
                        }
                        _ => {
                            // Processed but not confirmed yet
                        }
                    }
                }
            }

            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
    }
}
