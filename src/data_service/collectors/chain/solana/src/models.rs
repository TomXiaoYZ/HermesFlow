use serde::{Deserialize, Serialize};
use solana_sdk::{
    clock::Slot,
    hash::Hash,
    signature::Signature,
    transaction::TransactionError,
};
use solana_transaction_status::UiTransactionStatusMeta;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockInfo {
    pub slot: Slot,
    pub blockhash: Hash,
    pub parent_slot: Slot,
    pub transactions: Vec<TransactionInfo>,
    pub block_time: Option<i64>,
    pub block_height: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionInfo {
    pub signature: Signature,
    pub slot: Slot,
    pub error: Option<String>,
    pub block_time: Option<i64>,
    pub status: Result<(), TransactionError>,
    pub meta: Option<UiTransactionStatusMeta>,
    pub recent_blockhash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfo {
    pub pubkey: String,
    pub lamports: u64,
    pub owner: String,
    pub executable: bool,
    pub rent_epoch: u64,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramInfo {
    pub program_id: String,
    pub owner: String,
    pub executable: bool,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainStats {
    pub slot: Slot,
    pub block_height: u64,
    pub transaction_count: u64,
    pub epoch: u64,
    pub leader: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChainEvent {
    NewBlock(BlockInfo),
    NewTransaction(TransactionInfo),
    AccountUpdate {
        slot: Slot,
        account: AccountInfo,
    },
    ProgramUpdate {
        slot: Slot,
        program: ProgramInfo,
    },
    SlotUpdate {
        slot: Slot,
        parent: Option<Slot>,
        status: String,
    },
} 