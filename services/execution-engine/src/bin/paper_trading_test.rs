use chrono::Utc;
use common::events::{OrderSide, OrderType, TradeSignal};
use redis::Commands;
use uuid::Uuid;
// use std::{thread, time};

fn main() -> anyhow::Result<()> {
    let client = redis::Client::open("redis://127.0.0.1/")?;
    let mut conn = client.get_connection()?;

    println!(">>> Starting Paper Trading Test...");

    // 1. Simulate Strategy Engine publishing a signal
    let signal = TradeSignal {
        id: Uuid::new_v4(),
        symbol: "SOL/USDC".to_string(),
        side: OrderSide::Buy,
        quantity: 1.5,
        price: None,
        order_type: OrderType::Market,
        timestamp: Utc::now(),
        reason: "Test Signal".to_string(),
        strategy_id: "test-strat-001".to_string(),
        exchange: None,
        mode: None,
    };

    let json = serde_json::to_string(&signal)?;
    println!("[Strategy] Publishing Signal: {}", json);

    let _: () = conn.publish("trade_signals", json)?;

    // 2. Wait for Execution Engine (simulated) to pick it up
    // In a real test, we would run the actual execution-engine binary.
    // Here we just verify we can publish to the channel the engine listens to.

    println!("Signal published. If execution-engine is running, it should log 'Received signal'.");

    Ok(())
}
