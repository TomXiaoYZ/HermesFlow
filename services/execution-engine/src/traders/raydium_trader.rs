use anyhow::{anyhow, Result};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};
use std::str::FromStr;
use std::sync::Arc;
use tracing::{debug, info, warn};

// Raydium AMM Program ID
const RAYDIUM_AMM_PROGRAM_ID: &str = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";

/// Raydium AMM V4 Pool State
/// Based on https://github.com/raydium-io/raydium-amm/blob/master/program/src/state.rs
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct AmmInfo {
    /// Initialized status
    pub status: u64,
    pub nonce: u64,
    pub order_num: u64,
    pub depth: u64,
    pub coin_decimals: u64,
    pub pc_decimals: u64,
    pub state: u64,
    pub reset_flag: u64,
    pub min_size: u64,
    pub vol_max_cut_ratio: u64,

    /// Padding for intermediate u64/u128 fields (amount_wave, lot_sizes, fees, PnLs, swap amounts)
    /// Raydium V4 has ~256 bytes of params here before the first Pubkey.
    /// 256 bytes = 32 * u64
    pub padding_header: [u64; 32],

    pub pool_coin_token_account: Pubkey,
    pub pool_pc_token_account: Pubkey,
    pub coin_mint: Pubkey,
    pub pc_mint: Pubkey,
    pub lp_mint: Pubkey,
    pub open_orders: Pubkey,
    pub market: Pubkey,
    pub serum_program_id: Pubkey,
    pub target_orders: Pubkey,
    pub withdraw_queue: Pubkey,
    pub temp_lp_token_account: Pubkey,
    pub amm_owner: Pubkey,
    pub amm_owner_lp_token_account: Pubkey, // pnl_owner

    // Remaining fields (u64s)
    pub pool_coin_total: u64,
    pub pool_pc_total: u64,
}

// Raydium pool information
#[derive(Debug, Clone)]
pub struct PoolInfo {
    pub address: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub base_vault: Pubkey,
    pub quote_vault: Pubkey,
}

// Known pool addresses for major trading pairs
// This will be populated with real Raydium pool addresses
const KNOWN_POOLS: &[(&str, &str, &str)] = &[
    // (Base Token, Quote Token, Pool Address)
    (
        "So11111111111111111111111111111111111111112",
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
        "58oQChx4yWmvKdwLLZzBi4ChoCc2fqCUWBkwMihLYQo2",
    ), // SOL/USDC
    (
        "So11111111111111111111111111111111111111112",
        "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So",
        "ZfvDXXUhZDzDVsapffUyXHj9ByCoPjP4thL6YXcZ9ix",
    ), // SOL/mSOL
    (
        "So11111111111111111111111111111111111111112",
        "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263",
        "Cz8hxuNmBTygnVDvt1YvPKmcWh5R3j1y3PnQ6YCkJkuA",
    ), // SOL/BONK
       // Add more major pairs as needed
];

/// Raydium on-chain trader
#[derive(Clone)]
pub struct RaydiumTrader {
    rpc_client: Arc<RpcClient>,
}

impl RaydiumTrader {
    /// Create a new Raydium trader
    pub fn new(rpc_client: Arc<RpcClient>) -> Self {
        Self { rpc_client }
    }

    /// Find a Raydium pool for the given token pair
    pub async fn find_pool(&self, input_mint: &Pubkey, output_mint: &Pubkey) -> Result<Pubkey> {
        info!("Finding Raydium pool for {} -> {}", input_mint, output_mint);

        // Search in known pools first (hardcoded for speed)
        let input_str = input_mint.to_string();
        let output_str = output_mint.to_string();

        for (base, quote, pool) in KNOWN_POOLS {
            // Check both directions (base/quote and quote/base)
            if (*base == input_str && *quote == output_str)
                || (*quote == input_str && *base == output_str)
            {
                let pool_pubkey = Pubkey::from_str(pool)?;
                info!("Found known pool: {}", pool_pubkey);
                return Ok(pool_pubkey);
            }
        }

        Err(anyhow!(
            "No Raydium pool found for {} -> {}. Consider adding it to KNOWN_POOLS.",
            input_mint,
            output_mint
        ))
    }

    /// Calculate swap output amount using constant product formula (x * y = k)
    /// This is a simplified calculation - real Raydium uses more complex logic
    pub fn calculate_swap_output(
        &self,
        amount_in: u64,
        reserve_in: u64,
        reserve_out: u64,
        fee_numerator: u64,
        fee_denominator: u64,
    ) -> Result<u64> {
        if reserve_in == 0 || reserve_out == 0 {
            return Err(anyhow!("Invalid reserves: cannot be zero"));
        }

        // Calculate fee
        let amount_in_with_fee = amount_in
            .checked_mul(fee_denominator - fee_numerator)
            .ok_or_else(|| anyhow!("Overflow in fee calculation"))?
            / fee_denominator;

        // Constant product formula: (reserve_in + amount_in) * (reserve_out - amount_out) = k
        // amount_out = reserve_out - k / (reserve_in + amount_in)
        // amount_out = (amount_in * reserve_out) / (reserve_in + amount_in)
        let numerator = amount_in_with_fee
            .checked_mul(reserve_out)
            .ok_or_else(|| anyhow!("Overflow in output calculation"))?;

        let denominator = reserve_in
            .checked_add(amount_in_with_fee)
            .ok_or_else(|| anyhow!("Overflow in denominator calculation"))?;

        let amount_out = numerator / denominator;

        debug!(
            "Swap calculation: {} in -> {} out (reserves: {}/{})",
            amount_in, amount_out, reserve_in, reserve_out
        );

        Ok(amount_out)
    }

    /// Build a complete Raydium SwapBaseInV2 instruction (Standard V4)
    /// This requires 18 accounts to interact with Serum orderbook
    pub async fn build_swap_instruction(
        &self,
        amm_info: &AmmInfo,
        pool_address: &Pubkey,
        user_wallet: &Pubkey,
        user_source_token: &Pubkey,
        user_dest_token: &Pubkey,
        amount_in: u64,
        minimum_amount_out: u64,
    ) -> Result<Instruction> {
        let program_id = Pubkey::from_str(RAYDIUM_AMM_PROGRAM_ID)?;

        // Derive AMM authority PDA
        let (amm_authority, _bump) =
            Pubkey::find_program_address(&[b"amm authority".as_ref()], &program_id);

        // Fetch Serum Market Accounts
        // We need: Market, Bids, Asks, EventQueue, CoinVault, PCVault, VaultSigner
        let market_info = self
            .get_market_accounts(&amm_info.market, &amm_info.serum_program_id)
            .await?;

        info!(
            "Building SwapBaseIn (Standard V4) instruction: {} -> {} tokens",
            amount_in, minimum_amount_out
        );

        // Instruction discriminator for SwapBaseIn (index 9 in AmmInstruction enum)
        // Standard V4 swap uses 9, not 12. 12 is SwapBaseInV2 but might be less supported or have different reqs.
        // Let's us 9 (swap_base_in) which is the standard.
        let mut data = Vec::new();
        data.push(9u8); // swap_base_in discriminator
        data.extend_from_slice(&amount_in.to_le_bytes());
        data.extend_from_slice(&minimum_amount_out.to_le_bytes());

        // Accounts required for Raydium V4 Swap (18 total):
        // 0. Token Program
        // 1. AMM Account
        // 2. AMM Authority
        // 3. AMM Open Orders
        // 4. AMM Target Orders
        // 5. AMM Coin Vault
        // 6. AMM PC Vault
        // 7. Serum Program
        // 8. Serum Market
        // 9. Serum Bids
        // 10. Serum Asks
        // 11. Serum Event Queue
        // 12. Serum Coin Vault
        // 13. Serum PC Vault
        // 14. Serum Vault Signer
        // 15. User Source Token
        // 16. User Dest Token
        // 17. User Owner

        let accounts = vec![
            // 0. Token Program
            AccountMeta::new_readonly(spl_token::id(), false),
            // 1. AMM Account
            AccountMeta::new(*pool_address, false),
            // 2. AMM Authority
            AccountMeta::new_readonly(amm_authority, false),
            // 3. AMM Open Orders
            AccountMeta::new(amm_info.open_orders, false),
            // 4. AMM Target Orders
            AccountMeta::new(amm_info.target_orders, false),
            // 5. AMM Coin Vault
            AccountMeta::new(amm_info.pool_coin_token_account, false),
            // 6. AMM PC Vault
            AccountMeta::new(amm_info.pool_pc_token_account, false),
            // 7. Serum Program
            AccountMeta::new_readonly(amm_info.serum_program_id, false),
            // 8. Serum Market
            AccountMeta::new(amm_info.market, false),
            // 9. Serum Bids
            AccountMeta::new(market_info.bids, false),
            // 10. Serum Asks
            AccountMeta::new(market_info.asks, false),
            // 11. Serum Event Queue
            AccountMeta::new(market_info.event_queue, false),
            // 12. Serum Coin Vault
            AccountMeta::new(market_info.coin_vault, false),
            // 13. Serum PC Vault
            AccountMeta::new(market_info.pc_vault, false),
            // 14. Serum Vault Signer
            AccountMeta::new_readonly(market_info.vault_signer, false),
            // 15. User Source Token
            AccountMeta::new(*user_source_token, false),
            // 16. User Dest Token
            AccountMeta::new(*user_dest_token, false),
            // 17. User Owner
            AccountMeta::new_readonly(*user_wallet, true),
        ];

        Ok(Instruction {
            program_id,
            accounts,
            data,
        })
    }

    /// Get pool reserves by deserializing Raydium AMM pool account data
    pub async fn get_pool_reserves(&self, pool_address: &Pubkey) -> Result<(u64, u64)> {
        // Reuse get_amm_info logic
        let info = self.get_amm_info(pool_address).await?;
        Ok((info.pool_coin_total, info.pool_pc_total))
    }

    /// Get full AMM info by deserializing Raydium AMM pool account data
    /// This returns the complete AmmInfo struct which includes vaults and all pool state
    pub async fn get_amm_info(&self, pool_address: &Pubkey) -> Result<AmmInfo> {
        info!("Fetching AMM info for: {}", pool_address);

        // Fetch the pool account data from Solana RPC
        let account = self
            .rpc_client
            .get_account(pool_address)
            .map_err(|e| anyhow!("Failed to fetch pool account {}: {}", pool_address, e))?;

        // Verify account owner is Raydium AMM program
        let raydium_program_id = Pubkey::from_str(RAYDIUM_AMM_PROGRAM_ID)?;
        if account.owner != raydium_program_id {
            return Err(anyhow!(
                "Pool account {} is not owned by Raydium AMM program. Owner: {}",
                pool_address,
                account.owner
            ));
        }

        // DEBUG: Print FULL bytes in hex to debug layout
        let debug_hex: String = account
            .data
            .iter()
            .take(100)
            .map(|b| format!("{:02x}", b))
            .collect();
        info!("AMM Account Data Hex (First 100): {}", debug_hex);

        // Dynamic Parsing: Scan for USDC Mint to calibrate layout
        // Default layout assumption:
        // offset 400: coin_mint
        // offset 432: pc_mint (USDC)

        let data = &account.data;
        if data.len() < 752 {
            return Err(anyhow!("AMM Account data too short: {}", data.len()));
        }

        // Search for USDC Mint: EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
        let usdc_mint_pubkey =
            Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap();
        let usdc_bytes = usdc_mint_pubkey.to_bytes();

        let pc_mint_offset = if let Some(offset) = data.windows(32).position(|w| w == usdc_bytes) {
            info!("Found USDC Mint at offset: {}", offset);
            offset
        } else {
            warn!("USDC Mint not found in pool data. Assuming standard offset 432.");
            432
        };

        // Calculate offsets relative to pc_mint
        // Standard V4:
        // coin_mint (offset - 32)
        // pc_mint (offset)
        // lp_mint (offset + 32)
        // open_orders (offset + 64)
        // market (offset + 96)
        // serum_program_id (offset + 128)
        // target_orders (offset + 160)
        // withdraw_queue (offset + 192)
        // temp_lp_token_account (offset + 224)
        // amm_owner (offset + 256)
        // amm_owner_lp_token_account (offset + 288)

        let coin_mint_offset = pc_mint_offset - 32;
        let pool_pc_offset = coin_mint_offset - 32; // pool_pc_token_account
        let pool_coin_offset = pool_pc_offset - 32; // pool_coin_token_account

        // Parse Pubkeys
        let pool_coin_token_account =
            Pubkey::new_from_array(data[pool_coin_offset..pool_coin_offset + 32].try_into()?);
        let pool_pc_token_account =
            Pubkey::new_from_array(data[pool_pc_offset..pool_pc_offset + 32].try_into()?);
        let coin_mint =
            Pubkey::new_from_array(data[coin_mint_offset..coin_mint_offset + 32].try_into()?);
        let pc_mint = Pubkey::new_from_array(data[pc_mint_offset..pc_mint_offset + 32].try_into()?);

        let open_orders_offset = pc_mint_offset + 64;
        let open_orders =
            Pubkey::new_from_array(data[open_orders_offset..open_orders_offset + 32].try_into()?);

        let market_offset = pc_mint_offset + 96;
        let market = Pubkey::new_from_array(data[market_offset..market_offset + 32].try_into()?);

        let serum_prog_offset = pc_mint_offset + 128;
        let serum_program_id =
            Pubkey::new_from_array(data[serum_prog_offset..serum_prog_offset + 32].try_into()?);

        let target_orders_offset = pc_mint_offset + 160;
        let target_orders = Pubkey::new_from_array(
            data[target_orders_offset..target_orders_offset + 32].try_into()?,
        );

        let amm_owner_offset = pc_mint_offset + 256;
        let amm_owner =
            Pubkey::new_from_array(data[amm_owner_offset..amm_owner_offset + 32].try_into()?);

        // Parse Numerics (Header)
        // Status at 0, Coin Decimals at 32, Pc Decimals at 40
        let status = u64::from_le_bytes(data[0..8].try_into()?);
        let coin_decimals = u64::from_le_bytes(data[32..40].try_into()?);
        let pc_decimals = u64::from_le_bytes(data[40..48].try_into()?);

        info!("Fetching real reserves from token accounts...");
        let base_reserve = self.get_token_balance(&pool_coin_token_account).await?;
        let quote_reserve = self.get_token_balance(&pool_pc_token_account).await?;

        Ok(AmmInfo {
            status,
            nonce: 0,
            order_num: 0,
            depth: 0,
            coin_decimals,
            pc_decimals,
            state: 0,
            reset_flag: 0,
            min_size: 0,
            vol_max_cut_ratio: 0,
            padding_header: [0; 32],
            pool_coin_token_account,
            pool_pc_token_account,
            coin_mint,
            pc_mint,
            lp_mint: Pubkey::default(),
            open_orders,
            market,
            serum_program_id,
            target_orders,
            withdraw_queue: Pubkey::default(),
            temp_lp_token_account: Pubkey::default(),
            amm_owner,
            amm_owner_lp_token_account: Pubkey::default(),
            pool_coin_total: base_reserve,
            pool_pc_total: quote_reserve,
        })
    }

    /// Helper to get token account balance
    async fn get_token_balance(&self, token_account: &Pubkey) -> Result<u64> {
        let balance = self.rpc_client.get_token_account_balance(token_account)?;
        Ok(balance.amount.parse::<u64>()?)
    }

    /// Fetch Market Info (Bids, Asks, Queue, Vaults) from Serum Market account
    /// This is simplified for OpenBook/Serum V3 layout
    pub async fn get_market_accounts(
        &self,
        market_address: &Pubkey,
        serum_program_id: &Pubkey,
    ) -> Result<MarketAccounts> {
        info!("Fetching Market account: {}", market_address);
        let account = self
            .rpc_client
            .get_account(market_address)
            .map_err(|e| anyhow!("Failed to fetch market account: {}", e))?;

        if &account.owner != serum_program_id {
            return Err(anyhow!(
                "Market owner mismatch! Expected {}, got {}",
                serum_program_id,
                account.owner
            ));
        }

        let data = account.data;
        // Serum V3 Layout (approximate offsets for key fields)
        // 0-5 blob: padding string "sem..." or header
        // 13-45: account flags?
        // Let's use standard offsets for Serum V3
        // vault_signer_nonce: offset 4
        // base_mint: offset 53
        // quote_mint: offset 85
        // base_vault: offset 117
        // base_deposits_total: ...
        // base_fees_accrued: ...
        // quote_vault: offset 165
        // ...
        // bids: offset 285
        // asks: offset 317
        // event_queue: offset 349

        // Safety check
        if data.len() < 380 {
            return Err(anyhow!("Market account data too short"));
        }

        let base_vault = Pubkey::new_from_array(data[117..149].try_into()?);
        let quote_vault = Pubkey::new_from_array(data[165..197].try_into()?);
        let bids = Pubkey::new_from_array(data[285..317].try_into()?);
        let asks = Pubkey::new_from_array(data[317..349].try_into()?);
        let event_queue = Pubkey::new_from_array(data[349..381].try_into()?);

        // Serum V3 Layout (approximate offsets for key fields)
        // vault_signer_nonce is u64 at offset 4

        // Derive vault signer
        let nonce_u64 = u64::from_le_bytes(data[4..12].try_into()?);
        let (vault_signer, _) = Pubkey::find_program_address(
            &[market_address.as_ref(), &nonce_u64.to_le_bytes()],
            serum_program_id,
        );

        Ok(MarketAccounts {
            market: *market_address,
            bids,
            asks,
            event_queue,
            coin_vault: base_vault,
            pc_vault: quote_vault,
            vault_signer,
        })
    }
}

#[derive(Debug, Clone)]
pub struct MarketAccounts {
    pub market: Pubkey,
    pub bids: Pubkey,
    pub asks: Pubkey,
    pub event_queue: Pubkey,
    pub coin_vault: Pubkey,
    pub pc_vault: Pubkey,
    pub vault_signer: Pubkey,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_swap_output() {
        let trader = RaydiumTrader {
            rpc_client: Arc::new(RpcClient::new(
                "https://api.mainnet-beta.solana.com".to_string(),
            )),
        };

        // Test with 1 SOL input, 10 SOL reserve, 5000 USDC reserve
        // Fee: 0.25% (25/10000)
        let amount_in = 1_000_000_000; // 1 SOL (9 decimals)
        let reserve_in = 10_000_000_000; // 10 SOL
        let reserve_out = 5_000_000_000; // 5000 USDC (assuming 6 decimals normalized)
        let fee_numerator = 25;
        let fee_denominator = 10_000;

        let result = trader.calculate_swap_output(
            amount_in,
            reserve_in,
            reserve_out,
            fee_numerator,
            fee_denominator,
        );

        assert!(result.is_ok());
        let amount_out = result.unwrap();
        assert!(amount_out > 0);
        assert!(amount_out < reserve_out); // Output should be less than reserve
    }

    #[test]
    fn test_find_pool_sol_usdc() {
        let trader = RaydiumTrader {
            rpc_client: Arc::new(RpcClient::new(
                "https://api.mainnet-beta.solana.com".to_string(),
            )),
        };

        let sol_mint = Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap();
        let usdc_mint = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap();

        // Run async test
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(trader.find_pool(&sol_mint, &usdc_mint));

        assert!(result.is_ok());
    }
}
