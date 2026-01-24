use anyhow::Result;
use ibapi::contracts::Contract;
use ibapi::orders::{Action, Order as IbOrder};
use ibapi::Client;
use std::sync::Arc;
use tracing::info;

// Define local structs effectively mirroring data-engine's to avoid dependency hell for now,
// or ideally share a common-models crate.
// For migration speed, I will redefine them here.

#[derive(Debug, Clone)]
pub struct OrderRequest {
    pub symbol: String,
    pub action: String, // "BUY" or "SELL"
    pub quantity: f64,
    pub order_type: String, // "LMT", "MKT"
    pub price: Option<f64>,
}

#[derive(Clone)]
pub struct IBKRTrader {
    client: Arc<Client>,
    // account_id: String,
}

impl IBKRTrader {
    pub async fn new(host: &str, port: u32, client_id: u32) -> Result<Self> {
        let addr = format!("{}:{}", host, port);
        info!("Connecting to IBKR for trading at {}", addr);

        let client = Client::connect(&addr, client_id as i32)
            .map_err(|e| anyhow::anyhow!("IBKR Error: {}", e))?;

        Ok(Self {
            client: Arc::new(client),
            // account_id: "DU123456".to_string(),
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

        let order_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i32;

        order.order_id = order_id;

        // ibapi 0.1 returns an Iterator (lazy) that needs to be consumed for the order to be processed
        for _ in self
            .client
            .place_order(order_id, &contract, &order)
            .map_err(|e| anyhow::anyhow!("IBKR Place Order Error: {}", e))?
        {}

        info!("Order placed with ID: {}", order_id);

        Ok(order_id)
    }

    pub async fn cancel_order(&self, order_id: i32) -> Result<()> {
        info!("Cancelling order: {}", order_id);
        // ibapi 0.1 returns an Iterator
        for _ in self
            .client
            .cancel_order(order_id, "")
            .map_err(|e| anyhow::anyhow!("IBKR Cancel Error: {}", e))?
        {}
        Ok(())
    }

    fn create_contract(&self, symbol: &str) -> Contract {
        Contract::stock(symbol) // Removed .build() assuming stock() returns Contract itself or Builder that treats as Contract
    }
}
