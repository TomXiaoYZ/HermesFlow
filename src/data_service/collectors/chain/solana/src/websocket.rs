use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::broadcast;
use solana_client::{
    rpc_client::RpcClient,
    rpc_config::{RpcTransactionConfig, RpcSignatureSubscribeConfig},
    pubsub_client::PubsubClient,
};
use solana_sdk::{
    commitment_config::CommitmentConfig,
    signature::Signature,
};
use solana_transaction_status::UiTransactionEncoding;
use crate::{
    config::SolConfig,
    error::{SolError, Result},
    models::{ChainEvent, TransactionInfo},
};

pub struct WebsocketClient {
    rpc_client: Arc<RpcClient>,
    config: Arc<SolConfig>,
    event_sender: broadcast::Sender<ChainEvent>,
}

impl WebsocketClient {
    pub fn new(
        config: Arc<SolConfig>,
        event_sender: broadcast::Sender<ChainEvent>,
    ) -> Result<Self> {
        let rpc_client = Arc::new(RpcClient::new_with_timeout(
            config.primary_url.clone(),
            std::time::Duration::from_secs(config.request_timeout_secs),
        ));

        Ok(Self {
            rpc_client,
            config,
            event_sender,
        })
    }

    pub async fn subscribe_slots(&self) -> Result<()> {
        let _commitment = CommitmentConfig::from_str(&self.config.commitment)
            .map_err(|e| SolError::ConfigError(format!("Invalid commitment: {}", e)))?;

        let ws_url = self.config.ws_url.clone();
        let event_sender = self.event_sender.clone();

        tokio::spawn(async move {
            let slot_subscribe = PubsubClient::slot_subscribe(&ws_url);
            if let Ok((_subscription, receiver)) = slot_subscribe {
                while let Ok(slot) = receiver.recv() {
                    let _ = event_sender.send(ChainEvent::SlotUpdate {
                        slot: slot.slot,
                        parent: Some(slot.parent),
                        status: "confirmed".to_string(),
                    });
                }
            }
        });

        Ok(())
    }

    pub async fn subscribe_transactions(&self) -> Result<()> {
        let commitment = CommitmentConfig::from_str(&self.config.commitment)
            .map_err(|e| SolError::ConfigError(format!("Invalid commitment: {}", e)))?;

        let ws_url = self.config.ws_url.clone();
        let event_sender = self.event_sender.clone();
        let rpc_client = self.rpc_client.clone();

        tokio::spawn(async move {
            let signature_subscribe = PubsubClient::signature_subscribe(
                &ws_url,
                &Signature::default(),
                Some(RpcSignatureSubscribeConfig {
                    commitment: Some(commitment),
                    enable_received_notification: Some(true),
                }),
            );

            if let Ok((_subscription, receiver)) = signature_subscribe {
                while let Ok(response) = receiver.recv() {
                    let signature_str = format!("{:?}", response.value);
                    let signature = match Signature::from_str(&signature_str) {
                        Ok(sig) => sig,
                        Err(_) => continue,
                    };

                    if let Ok(tx) = rpc_client.get_transaction_with_config(
                        &signature,
                        RpcTransactionConfig {
                            encoding: Some(UiTransactionEncoding::Base64),
                            commitment: Some(commitment),
                            max_supported_transaction_version: Some(0),
                        },
                    ) {
                        if let Some(meta) = tx.transaction.meta {
                            let recent_blockhash = format!("{:?}", signature);
                            let status = meta.status.clone();
                            let tx = TransactionInfo {
                                signature,
                                slot: tx.slot,
                                error: None,
                                block_time: tx.block_time,
                                status,
                                meta: Some(meta),
                                recent_blockhash,
                            };
                            let _ = event_sender.send(ChainEvent::NewTransaction(tx));
                        }
                    }
                }
            }
        });

        Ok(())
    }

    pub async fn start(&self) -> Result<()> {
        self.subscribe_slots().await?;
        self.subscribe_transactions().await?;
        Ok(())
    }

    pub async fn check_connection(&self) -> Result<()> {
        self.rpc_client
            .get_slot()
            .map_err(|e| SolError::ConnectionError(format!("Connection check failed: {}", e)))?;
        Ok(())
    }
} 