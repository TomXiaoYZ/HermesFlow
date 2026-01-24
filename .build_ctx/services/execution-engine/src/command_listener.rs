use common::events::{TradeSignal, OrderUpdate};
use anyhow::Result;
use redis::Commands;
use std::sync::Arc;
// use tokio::sync::broadcast;
use tracing::{info, error, warn};
use crate::traders::solana_trader::SolanaTrader;
use crate::traders::ibkr_trader::IBKRTrader;

pub struct CommandListener {
    client: redis::Client,
    pub solana_trader: Option<Arc<SolanaTrader>>,
    pub ibkr_trader: Option<Arc<IBKRTrader>>,
}

impl CommandListener {
    pub fn new(redis_url: &str) -> Result<Self> {
        let client = redis::Client::open(redis_url)?;
        Ok(Self { 
            client,
            solana_trader: None,
            ibkr_trader: None,
        })
    }
    
    pub fn set_traders(&mut self, solana: Option<Arc<SolanaTrader>>, ibkr: Option<Arc<IBKRTrader>>) {
        self.solana_trader = solana;
        self.ibkr_trader = ibkr;
    }

    pub fn publish_update(&self, update: &OrderUpdate) -> Result<()> {
        let mut conn = self.client.get_connection()?;
        let json = serde_json::to_string(update)?;
        let _: () = conn.publish("order_updates", json)?;
        Ok(())
    }

    pub async fn listen_for_signals(&self) -> Result<()> {
        let mut conn = self.client.get_connection()?;
        let mut pubsub = conn.as_pubsub();
        pubsub.subscribe("trade_signals")?;

        info!("CommandListener: Subscribed to trade_signals");

        loop {
            let msg = pubsub.get_message()?;
            let payload: String = msg.get_payload()?;
            
            if let Ok(signal) = serde_json::from_str::<TradeSignal>(&payload) {
                info!("Received signal: {:?}", signal);
                
                // Route to Trader
                // Simple heuristic: if symbol is Solana address (length > 30), use SolanaTrader
                // Else use IBKRTrader
                if signal.symbol.len() > 30 || signal.symbol == "SOL" {
                     if let Some(trader) = &self.solana_trader {
                         let trader = trader.clone();
                         let sig_clone = signal.clone();
                         
                         // Spawning async task for execution to not block listener
                         tokio::spawn(async move {
                             let res = match sig_clone.side {
                                 common::events::OrderSide::Buy => {
                                     // Buy logic: quantity in SOL? Or quantity in Tokens?
                                     // Signal says 'quantity' of Asset.
                                     // If Buy SOL -> Token: Input is SOL amount.
                                     // If we are given quantity of generic asset, we might need a price to valid input amount.
                                     // For now, assume quantity is Input Amount (SOL for buys).
                                     trader.buy(&sig_clone.symbol, sig_clone.quantity, 100).await
                                 },
                                 common::events::OrderSide::Sell => {
                                     // Sell logic: quantity is % to sell? Or strict amount?
                                     // SolanaTrader.sell takes percentage (0.0-1.0).
                                     // Let's assume signal.quantity 1.0 = 100%.
                                     trader.sell(&sig_clone.symbol, sig_clone.quantity, 100).await
                                 }
                             };
                             
                             match res {
                                 Ok(tx) => info!("Execution Success: {}", tx),
                                 Err(e) => error!("Execution Failed: {}", e),
                             }
                         });
                     } else {
                         error!("No Solana Trader configured");
                     }
                } else {
                    // IBKR
                     if let Some(_) = &self.ibkr_trader {
                         // TODO: IBKR exec
                         info!("Routing to IBKR Trader (Not Implemented)");
                     }
                }
            }
        }
    }
}
