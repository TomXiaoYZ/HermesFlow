use crate::config::IbkrConfig;
use crate::error::Result;
use crate::models::trading::{AccountSummary, OrderRequest, Position};
use ibapi::contracts::Contract;
use ibapi::orders::{Order as IbOrder, Action};
use ibapi::Client;
use std::sync::Arc;
// use tokio::sync::Mutex; // Removed unused import
use tracing::{info, warn};

#[derive(Clone)]
pub struct IBKRTrader {
    client: Arc<Client>,
    account_id: String,
}

impl IBKRTrader {
    pub async fn new(config: &IbkrConfig) -> Result<Self> {
        let addr = format!("{}:{}", config.host, config.port);
        info!("Connecting to IBKR for trading at {}", addr);
        
        // Use client_id + 1 to avoid conflict with collector
        let client = Client::connect(&addr, config.client_id + 1).await
            .map_err(|e| crate::error::DataError::IbkrError(e))?;
        
        Ok(Self {
            client: Arc::new(client),
            account_id: "DU123456".to_string(), // TODO: Get from connection or config
        })
    }

    pub async fn place_order(&self, req: OrderRequest) -> Result<i32> {
        info!("Placing order: {:?}", req);
        
        let contract = self.create_contract(&req.symbol);
        let action = if req.action.to_uppercase() == "BUY" {
            Action::Buy
        } else {
            Action::Sell
        };
        
        let mut order = IbOrder::default();
        order.action = action;
        order.total_quantity = req.quantity;
        order.order_type = req.order_type.clone();
        
        if req.order_type.to_uppercase() == "LMT" {
            if let Some(price) = req.price {
                order.limit_price = Some(price);
            }
        }

        // Generate a temporary order ID (in production, use client.next_order_id())
        // For now using timestamp to avoid 0
        let order_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i32;
        
        order.order_id = order_id;

        self.client.place_order(order_id, &contract, &order).await
            .map_err(|e| crate::error::DataError::IbkrError(e))?;
            
        info!("Order placed with ID: {}", order_id);
        
        Ok(order_id)
    }

    pub async fn cancel_order(&self, order_id: i32) -> Result<()> {
        info!("Cancelling order: {}", order_id);
        self.client.cancel_order(order_id, "").await
            .map_err(|e| crate::error::DataError::IbkrError(e))?;
        Ok(())
    }

    pub async fn get_positions(&self) -> Result<Vec<Position>> {
        // TODO: Implement position request handling
        // This usually requires requesting positions and listening to a stream
        warn!("get_positions not fully implemented yet");
        Ok(vec![])
    }

    pub async fn get_account_summary(&self) -> Result<AccountSummary> {
        // TODO: Implement account summary request
        warn!("get_account_summary not fully implemented yet");
        Ok(AccountSummary {
            net_liquidation: 0.0,
            total_cash: 0.0,
            buying_power: 0.0,
            currency: "USD".to_string(),
        })
    }

    fn create_contract(&self, symbol: &str) -> Contract {
        Contract::stock(symbol).build()
    }
}
