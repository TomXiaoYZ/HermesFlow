use std::sync::Arc;
use futures::StreamExt;
use tokio::sync::broadcast;
use web3::{
    futures::FutureExt,
    transports::WebSocket,
    Web3,
};
use crate::{
    config::EthConfig,
    error::{EthError, Result},
    models::{BlockInfo, ChainEvent, TransactionInfo},
};

pub struct WebsocketClient {
    web3: Web3<WebSocket>,
    config: Arc<EthConfig>,
    event_sender: broadcast::Sender<ChainEvent>,
}

impl WebsocketClient {
    pub async fn new(
        config: Arc<EthConfig>,
        event_sender: broadcast::Sender<ChainEvent>,
    ) -> Result<Self> {
        let transport = WebSocket::new(&config.ws_url)
            .await
            .map_err(|e| EthError::ConnectionError(format!("WebSocket connection failed: {}", e)))?;

        let web3 = Web3::new(transport);

        Ok(Self {
            web3,
            config,
            event_sender,
        })
    }

    pub async fn subscribe_new_blocks(&self) -> Result<()> {
        let mut sub = self.web3
            .eth_subscribe()
            .subscribe_new_heads()
            .await
            .map_err(|e| EthError::WebsocketError(format!("Block subscription failed: {}", e)))?;

        let event_sender = self.event_sender.clone();

        tokio::spawn(async move {
            while let Some(block) = sub.next().await {
                if let Ok(block) = block {
                    let block_info = BlockInfo::from(block);
                    let _ = event_sender.send(ChainEvent::NewBlock(block_info));
                }
            }
        });

        Ok(())
    }

    pub async fn subscribe_pending_transactions(&self) -> Result<()> {
        let mut sub = self.web3
            .eth_subscribe()
            .subscribe_pending_transactions()
            .await
            .map_err(|e| EthError::WebsocketError(format!("Transaction subscription failed: {}", e)))?;

        let web3 = self.web3.clone();
        let event_sender = self.event_sender.clone();

        tokio::spawn(async move {
            while let Some(tx_hash) = sub.next().await {
                if let Ok(hash) = tx_hash {
                    if let Ok(Some(tx)) = web3.eth().transaction(hash).await {
                        let tx_info = TransactionInfo::from(tx);
                        let _ = event_sender.send(ChainEvent::PendingTransaction(tx_info));
                    }
                }
            }
        });

        Ok(())
    }

    pub async fn start(&self) -> Result<()> {
        self.subscribe_new_blocks().await?;
        self.subscribe_pending_transactions().await?;
        Ok(())
    }

    pub async fn check_connection(&self) -> Result<()> {
        self.web3
            .eth()
            .block_number()
            .await
            .map_err(|e| EthError::ConnectionError(format!("Connection check failed: {}", e)))?;
        Ok(())
    }
} 