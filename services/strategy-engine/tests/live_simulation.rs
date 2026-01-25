use backtest_engine::vm::vm::StackVM;
use chrono::Utc;
use common::events::{MarketDataUpdate, OrderSide, TradeSignal};
use std::env;
use strategy_engine::event_bus::EventBus;
use strategy_engine::market_data_manager::MarketDataManager;
use strategy_engine::risk::RiskEngine;
use tokio::time::{sleep, Duration};
use uuid::Uuid;

#[tokio::test]
async fn test_live_strategy_loop() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_test_writer().init();
    // 1. Setup
    let redis_url = "redis://127.0.0.1:6379";
    let event_bus = EventBus::new(redis_url)?;
    let mut market_manager = MarketDataManager::new();
    let mut vm = StackVM::new();
    let risk_engine = RiskEngine::new();

    // Subscribe to signals to verify output
    let mut signal_rx = event_bus.subscribe_market_data("trade_signals").await?; // Using generic subscribe for now, returns raw string in rx?
                                                                                 // Wait, subscribe_market_data returns Receiver<MarketDataUpdate>.
                                                                                 // I need a Receiver for TradeSignal.
                                                                                 // EventBus doesn't have subscribe_signals yet.
                                                                                 // I need to add it or just raw redis subscribe here.

    let client = redis::Client::open(redis_url)?;
    let mut conn = client.get_connection()?;
    // Set timeout to prevent hanging
    conn.set_read_timeout(Some(std::time::Duration::from_secs(5)))?;
    let mut pubsub = conn.as_pubsub();
    pubsub.subscribe("trade_signals")?;

    // 2. Simulate Market Data Stream (Pump)
    let symbol = "SOL_TEST";
    let base_price = 100.0;

    // We need enough data to trigger features (min 20)
    for i in 0..30 {
        let price = base_price + (i as f64) * 0.5; // Up 0.5 each step
        let update = MarketDataUpdate {
            symbol: symbol.to_string(),
            price,
            volume: 1000.0,
            timestamp: Utc::now(),
            source: "test".to_string(),
        };
        // println!("Loop {}: Price {}", i, price);

        if let Some(features) = market_manager.on_update(update.clone()) {
            let tokens = vec![0];
            if let Some(result) = vm.execute(&tokens, &features) {
                if let Some(last_val) = result.last() {
                    // println!("Loop {}: VM Last Val {}", i, last_val);

                    // Logic: Ret > 0.0001 -> Buy, < -0.0001 -> Sell
                    // Observed: -1.8 (Sell side)
                    let side = if *last_val > 0.0001 {
                        Some(OrderSide::Buy)
                    } else if *last_val < -0.0001 {
                        Some(OrderSide::Sell)
                    } else {
                        None
                    };

                    if let Some(side_enum) = side {
                        let signal = TradeSignal {
                            id: Uuid::new_v4(),
                            strategy_id: "test".to_string(),
                            symbol: symbol.to_string(),
                            side: side_enum,
                            quantity: 1.0,
                            price: None,
                            order_type: common::events::OrderType::Market,
                            timestamp: Utc::now(),
                            reason: "UnitTest".to_string(),
                        };

                        if risk_engine.check(&signal, Some(1_000_000.0)).await {
                            // println!("Loop {}: Publishing Signal...", i);
                            event_bus.publish_signal(&signal).await?;
                            // println!("Loop {}: Signal Published!", i);
                        }
                    }
                }
            }
        }
        sleep(Duration::from_millis(10)).await;
    }

    // 3. Assert Signal Received from Redis
    // Allow retries or timeout
    let msg = pubsub.get_message()?;
    let payload: String = msg.get_payload()?;
    let signal: TradeSignal = serde_json::from_str(&payload)?;

    assert_eq!(signal.symbol, symbol);
    // We expect Sell now due to negative z-score of returns
    assert_eq!(signal.side, OrderSide::Sell);

    println!("SUCCESS: Received {:?} signal for {}", signal.side, symbol);

    Ok(())
}
