use serde::{Deserialize, Serialize};
use web3::types::{Block, Transaction, H256, U64};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockInfo {
    pub number: u64,
    pub hash: H256,
    pub parent_hash: H256,
    pub timestamp: u64,
    pub transactions: Vec<H256>,
    pub gas_used: U64,
    pub gas_limit: U64,
    pub base_fee_per_gas: Option<U64>,
}

impl From<Block<H256>> for BlockInfo {
    fn from(block: Block<H256>) -> Self {
        Self {
            number: block.number.unwrap_or_default().as_u64(),
            hash: block.hash.unwrap_or_default(),
            parent_hash: block.parent_hash,
            timestamp: block.timestamp.as_u64(),
            transactions: block.transactions,
            gas_used: block.gas_used,
            gas_limit: block.gas_limit,
            base_fee_per_gas: block.base_fee_per_gas,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionInfo {
    pub hash: H256,
    pub block_hash: Option<H256>,
    pub block_number: Option<u64>,
    pub from: String,
    pub to: Option<String>,
    pub value: U64,
    pub gas_price: Option<U64>,
    pub max_fee_per_gas: Option<U64>,
    pub max_priority_fee_per_gas: Option<U64>,
    pub input: Vec<u8>,
}

impl From<Transaction> for TransactionInfo {
    fn from(tx: Transaction) -> Self {
        Self {
            hash: tx.hash,
            block_hash: tx.block_hash,
            block_number: tx.block_number.map(|n| n.as_u64()),
            from: format!("{:?}", tx.from),
            to: tx.to.map(|addr| format!("{:?}", addr)),
            value: tx.value,
            gas_price: tx.gas_price,
            max_fee_per_gas: tx.max_fee_per_gas,
            max_priority_fee_per_gas: tx.max_priority_fee_per_gas,
            input: tx.input.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainStats {
    pub latest_block: u64,
    pub pending_transactions: usize,
    pub gas_price: U64,
    pub network_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChainEvent {
    NewBlock(BlockInfo),
    NewTransaction(TransactionInfo),
    Reorg {
        old_block: BlockInfo,
        new_block: BlockInfo,
    },
    PendingTransaction(TransactionInfo),
} 