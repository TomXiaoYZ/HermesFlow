use anyhow::{anyhow, Result};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    transaction::Transaction,
};
use std::str::FromStr;
use std::sync::Arc;
use tracing::info;

// Raydium AMM V4 Program ID (主流 Memecoin 使用的版本)
const RAYDIUM_AMM_PROGRAM: &str = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";

// Raydium Authority (固定地址)
const RAYDIUM_AUTHORITY: &str = "5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1";

/// Raydium Swap 交易构建器（纯链上，无 HTTP API 依赖）
pub struct RaydiumSwap {
    rpc_client: Arc<RpcClient>,
}

impl RaydiumSwap {
    pub fn new(rpc_client: Arc<RpcClient>) -> Self {
        Self { rpc_client }
    }

    /// 查找 SOL/Token 的 Raydium 池子
    pub async fn find_pool(&self, token_mint: &Pubkey) -> Result<Pubkey> {
        // 这里需要链上查询所有 Raydium pool accounts
        // 简化版本：使用已知的 SOL/USDC 池作为路由中继
        // 真实实现需要遍历账户或使用离线池列表
        
        info!("查找 Token {} 的流动性池...", token_mint);
        
        // TODO: 实现池查找逻辑
        // 临时方案：返回错误提示不支持该 Token
        Err(anyhow!("Pool discovery not yet implemented. Token {} may not have a direct SOL pair.", token_mint))
    }

    /// 构建 Raydium Swap 指令
    pub fn build_swap_instruction(
        &self,
        pool_id: &Pubkey,
        user_wallet: &Pubkey,
        source_token: &Pubkey,
        dest_token: &Pubkey,
        amount_in: u64,
        minimum_amount_out: u64,
    ) -> Result<Instruction> {
        let program_id = Pubkey::from_str(RAYDIUM_AMM_PROGRAM)?;
        let authority = Pubkey::from_str(RAYDIUM_AUTHORITY)?;

        // Raydium Swap 指令数据格式 (简化版)
        let mut instruction_data = vec![9]; // Swap 指令标识
        instruction_data.extend_from_slice(&amount_in.to_le_bytes());
        instruction_data.extend_from_slice(&minimum_amount_out.to_le_bytes());

        // 构建账户列表
        let accounts = vec![
            AccountMeta::new_readonly(program_id, false),
            AccountMeta::new(*pool_id, false),
            AccountMeta::new_readonly(authority, false),
            AccountMeta::new(*user_wallet, true), // Signer
            AccountMeta::new(*source_token, false),
            AccountMeta::new(*dest_token, false),
        ];

        Ok(Instruction {
            program_id,
            accounts,
            data: instruction_data,
        })
    }

    /// 执行 Swap（完整流程）
    pub async fn execute_swap(
        &self,
        keypair: &Keypair,
        token_mint: &Pubkey,
        amount_sol: f64,
    ) -> Result<String> {
        info!("开始链上 Swap: {} SOL -> {}", amount_sol, token_mint);

        // 1. 查找池子
        let pool_id = self.find_pool(token_mint).await?;

        // 2. 构建指令
        let amount_lamports = (amount_sol * 1_000_000_000.0) as u64;
        let minimum_out = 0; // 0 表示接受任何数量（实际应该计算滑点）

        let swap_ix = self.build_swap_instruction(
            &pool_id,
            &keypair.pubkey(),
            &Pubkey::from_str("So11111111111111111111111111111111111111112")?, // SOL
            token_mint,
            amount_lamports,
            minimum_out,
        )?;

        // 3. 获取最新 blockhash
        let recent_blockhash = self.rpc_client.get_latest_blockhash()?;

        // 4. 创建并签名交易
        let mut transaction = Transaction::new_with_payer(&[swap_ix], Some(&keypair.pubkey()));
        transaction.sign(&[keypair], recent_blockhash);

        // 5. 发送交易
        let signature = self.rpc_client.send_and_confirm_transaction(&transaction)?;

        info!("Swap 成功! 签名: {}", signature);
        Ok(signature.to_string())
    }
}
