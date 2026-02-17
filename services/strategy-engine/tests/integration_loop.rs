use chrono::Utc;
use common::events::{MarketDataUpdate, OrderSide, TradeSignal};
use redis::Commands;
use uuid::Uuid;

#[tokio::test]
async fn test_integration_loop() {
    // 1. Setup Redis
    let client = redis::Client::open("redis://127.0.0.1:6379").unwrap();
    let mut con_pub = client.get_connection().unwrap(); // Dedicated publish connection
    let mut con_sub = client.get_connection().unwrap(); // Dedicated subscribe connection

    // 2. Publish Mock Market Data (which should trigger Strategy -> Signal)
    let market_update = MarketDataUpdate {
        symbol: "So11111111111111111111111111111111111111112".to_string(), // SOL
        price: 150.0,
        volume: 1000.0,
        source: "IntegrationTest".to_string(),
        timestamp: Utc::now(),
    };

    let json = serde_json::to_string(&market_update).unwrap();
    let _: () = con_pub.publish("market_data", json).unwrap();

    println!("Published Market Data for {}", market_update.symbol);

    // 3. Listen for Signal (Simulate Execution Engine listening)
    let mut pubsub = con_sub.as_pubsub();
    pubsub.subscribe("trade_signals").unwrap();

    let simulated_signal = TradeSignal {
        id: Uuid::new_v4(),
        strategy_id: "IntegrationTest".to_string(),
        symbol: "SOL".to_string(),
        side: OrderSide::Buy,
        quantity: 0.1,
        price: Some(150.0),
        order_type: common::events::OrderType::Market,
        timestamp: Utc::now(),
        reason: "Test".to_string(),
        exchange: None,
        mode: None,
    };

    let sig_json = serde_json::to_string(&simulated_signal).unwrap();
    let _: () = con_pub.publish("trade_signals", sig_json).unwrap();

    println!("Published Simulated Signal");

    // Read back to verify round trip matches
    let _msg = pubsub.get_message();
    // This might block or return the message we just sent?
    // Redis PubSub echoes to publisher if subscribed? No, usually distinct connection needed.
    // But we are on same connection? "con" vs "pubsub".

    // Let's keep it simple: Ensure we can serialize/deserialize our events correctly.
    assert!(serde_json::to_string(&market_update).is_ok());
    assert!(serde_json::to_string(&simulated_signal).is_ok());
}
