use anyhow::{anyhow, Result};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
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
    /// Nonce used to generate seeds
    pub nonce: u64,
    /// Max order count
    pub order_num: u64,
    /// Within this range, 5 fraction_part is 1/10000
    pub depth: u64,
    /// Coin mint decimals
    pub coin_decimals: u64,
    /// Pc mint decimals
    pub pc_decimals: u64,
    /// AMM start state: 0 - not started, 1 - started
    pub state: u64,
    /// Reset flag
    pub reset_flag: u64,
    /// Min size for order
    pub min_size: u64,
    /// Vol max cut ratio
    pub vol_max_cut_ratio: u64,
    /// Amm owner
    pub amm_owner: Pubkey,
    /// Fee amount for liquidity provider
    pub fees_to_lp: u64,
    /// Pool fees (numerator/denominator)
    pub fees: u64,
    /// Pool coin token account
    pub pool_coin_token_account: Pubkey,
    /// Pool pc token account
    pub pool_pc_token_account: Pubkey,
    /// Coin mint address
    pub coin_mint: Pubkey,
    /// Pc mint address
    pub pc_mint: Pubkey,
    /// Lp mint address
    pub lp_mint: Pubkey,
    /// Open orders
    pub open_orders: Pubkey,
    /// Serum market
    pub market: Pubkey,
    /// Serum program id
    pub serum_program_id: Pubkey,
    /// Target orders
    pub target_orders: Pubkey,
    /// Withdraw queue
    pub withdraw_queue: Pubkey,
    /// Temp lp token account
    pub temp_lp_token_account: Pubkey,
    /// Owner lp token account
    pub amm_owner_lp_token_account: Pubkey,
    /// Pnl coin offset
    pub pnl_coin: u64,
    /// Pnl pc offset
    pub pnl_pc: u64,
    /// Pool total deposit coin in liquidity
    pub pool_coin_total: u64,
    /// Pool total deposit pc in liquidity  
    pub pool_pc_total: u64,
    /// Total issue lp amount
    pub pool_lp_supply: u64,
    /// Padding
    pub padding: [u64; 3],
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
    ("So11111111111111111111111111111111111111112", "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", "58oQChx4yWmvKdwLLZzBi4ChoCc2fqCUWBkwMihLYQo2"), // SOL/USDC
    ("So11111111111111111111111111111111111111112", "mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So", "ZfvDXXUhZDzDVsapffUyXHj9ByCoPjP4thL6YXcZ9ix"), // SOL/mSOL
    ("So11111111111111111111111111111111111111112", "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263", "Cz8hxuNmBTygnVDvt1YvPKmcWh5R3j1y3PnQ6YCkJkuA"), // SOL/BONK
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
    pub async fn find_pool(
        &self,
        input_mint: &Pubkey,
        output_mint: &Pubkey,
    ) -> Result<Pubkey> {
        info!(
            "Finding Raydium pool for {} -> {}",
            input_mint, output_mint
        );

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

    /// Build a Raydium swap instruction
    /// Note: This is a simplified version. Real Raydium instructions are more complex
    /// and require proper account setup and data serialization
    pub fn build_swap_instruction(
        &self,
        pool_address: &Pubkey,
        user_wallet: &Pubkey,
        user_source_token: &Pubkey,
        user_dest_token: &Pubkey,
        amount_in: u64,
        minimum_amount_out: u64,
    ) -> Result<Instruction> {
        let program_id = Pubkey::from_str(RAYDIUM_AMM_PROGRAM_ID)?;

        // Simplified instruction data (real Raydium uses borsh serialization)
        // Instruction discriminator for swap: 9 (this is simplified)
        let mut data = vec![9u8]; // Swap instruction
        data.extend_from_slice(&amount_in.to_le_bytes());
        data.extend_from_slice(&minimum_amount_out.to_le_bytes());

        // Simplified account list (real Raydium requires many more accounts)
        let accounts = vec![
            AccountMeta::new(*pool_address, false),
            AccountMeta::new(*user_wallet, true), // Signer
            AccountMeta::new(*user_source_token, false),
            AccountMeta::new(*user_dest_token, false),
            AccountMeta::new_readonly(system_program::id(), false),
            // Note: Real Raydium requires pool vaults, authority, etc.
        ];

        Ok(Instruction {
            program_id,
            accounts,
            data,
        })
    }

    /// Get pool reserves by deserializing Raydium AMM pool account data
    pub async fn get_pool_reserves(&self, pool_address: &Pubkey) -> Result<(u64, u64)> {
        info!("Fetching pool reserves for: {}", pool_address);

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

        // Deserialize the account data using Borsh
        let amm_info = AmmInfo::try_from_slice(&account.data)
            .map_err(|e| anyhow!("Failed to deserialize Raydium pool state: {}", e))?;

        // Extract reserves
        // pool_coin_total = base token reserve
        // pool_pc_total = quote token reserve (PC = Price Currency, usually USDC/USDT)
        let base_reserve = amm_info.pool_coin_total;
        let quote_reserve = amm_info.pool_pc_total;

        info!(
            "Pool {} reserves: coin={} ({} decimals), pc={} ({} decimals)",
            pool_address,
            base_reserve,
            amm_info.coin_decimals,
            quote_reserve,
            amm_info.pc_decimals
        );

        // Verify pool is active
        if amm_info.status == 0 {
            return Err(anyhow!("Pool {} is not initialized", pool_address));
        }

        if amm_info.state == 0 {
            warn!("Pool {} AMM state is 0 (not started)", pool_address);
        }

        Ok((base_reserve, quote_reserve))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_swap_output() {
        let trader = RaydiumTrader {
            rpc_client: Arc::new(RpcClient::new("https://api.mainnet-beta.solana.com".to_string())),
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
            rpc_client: Arc::new(RpcClient::new("https://api.mainnet-beta.solana.com".to_string())),
        };

        let sol_mint = Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap();
        let usdc_mint = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap();

        // Run async test
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(trader.find_pool(&sol_mint, &usdc_mint));

        assert!(result.is_ok());
    }
}
