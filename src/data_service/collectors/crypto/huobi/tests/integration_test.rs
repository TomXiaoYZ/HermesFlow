use std::time::Duration;
use tokio::time::sleep;
use huobi::{HuobiCollector, HuobiCollectorConfig};
use common::MarketDataType;

#[tokio::test]
async fn test_huobi_collector() {
    // 创建收集器实例
    let config = HuobiCollectorConfig::default();
    let mut collector = HuobiCollector::new(config);

    // 启动收集器
    let mut rx = collector.start().await.expect("启动收集器失败");
    println!("收集器已启动");

    // 等待连接建立
    sleep(Duration::from_secs(1)).await;

    // 订阅 BTC/USDT 交易对的数据
    collector.subscribe("btcusdt").await.expect("订阅失败");
    println!("已订阅 BTC/USDT 交易对");

    // 接收并验证数据
    let received_trade = false;
    let received_depth = false;
    let received_kline = false;
    let mut received_ticker = false;

    // 等待接收不同类型的数据
    let timeout = Duration::from_secs(30);
    let start = std::time::Instant::now();

    while start.elapsed() < timeout && 
          (!received_trade || !received_depth || !received_kline || !received_ticker) {
        if let Ok(market_data) = rx.try_recv() {
            match market_data.data_type {
                MarketDataType::Trade(_) => {
                    println!("收到交易数据");
                    received_ticker = true;
                }
                MarketDataType::OrderBook { .. } => {
                    println!("收到深度数据");
                }
                MarketDataType::Candlestick(_) => {
                    println!("收到K线数据");
                }
            }
        }
        sleep(Duration::from_millis(100)).await;
    }

    // 取消订阅
    collector.unsubscribe("btcusdt").await.expect("取消订阅失败");
    println!("已取消订阅");

    // 停止收集器
    collector.stop().await.expect("停止收集器失败");
    println!("收集器已停止");

    // 验证是否收到所有类型的数据
    assert!(received_trade, "未收到交易数据");
    assert!(received_depth, "未收到深度数据");
    assert!(received_kline, "未收到K线数据");
}

#[tokio::test]
async fn test_error_handling() {
    // 测试错误的 WebSocket 地址
    let mut config = HuobiCollectorConfig::default();
    config.ws_endpoint = "wss://invalid.huobi.pro/ws".to_string();
    let mut collector = HuobiCollector::new(config);

    // 启动收集器应该失败
    let result = collector.start().await;
    assert!(result.is_err(), "使用无效地址应该失败");

    // 测试无效的交易对
    let config = HuobiCollectorConfig::default();
    let mut collector = HuobiCollector::new(config);
    let _ = collector.start().await;

    // 订阅无效的交易对
    let result = collector.subscribe("invalid_pair").await;
    assert!(result.is_err(), "订阅无效交易对应该失败");

    collector.stop().await.expect("停止收集器失败");
}

#[tokio::test]
async fn test_reconnection() {
    let config = HuobiCollectorConfig::default();
    let mut collector = HuobiCollector::new(config);

    // 启动收集器
    let _rx = collector.start().await.expect("启动收集器失败");
    println!("收集器已启动");

    // 订阅数据
    collector.subscribe("btcusdt").await.expect("订阅失败");
    println!("已订阅 BTC/USDT 交易对");

    // 等待一些数据
    sleep(Duration::from_secs(5)).await;

    // 模拟断开连接（通过重新启动）
    collector.stop().await.expect("停止收集器失败");
    println!("模拟断开连接");

    sleep(Duration::from_secs(1)).await;

    // 重新连接
    let mut rx = collector.start().await.expect("重新启动收集器失败");
    println!("重新连接成功");

    // 验证是否能继续接收数据
    let timeout = Duration::from_secs(10);
    let start = std::time::Instant::now();
    let mut received_data = false;

    while start.elapsed() < timeout && !received_data {
        if rx.try_recv().is_ok() {
            received_data = true;
            println!("重连后成功接收数据");
        }
        sleep(Duration::from_millis(100)).await;
    }

    assert!(received_data, "重连后应该能收到数据");

    collector.stop().await.expect("停止收集器失败");
}

#[tokio::test]
async fn test_multiple_symbols() {
    let config = HuobiCollectorConfig::default();
    let mut collector = HuobiCollector::new(config);

    // 启动收集器
    let mut rx = collector.start().await.expect("启动收集器失败");
    println!("收集器已启动");

    // 订阅多个交易对
    let symbols = vec!["btcusdt", "ethusdt", "dogeusdt"];
    for symbol in &symbols {
        collector.subscribe(symbol).await.expect("订阅失败");
        println!("已订阅 {}", symbol);
    }

    // 记录收到的每个交易对的数据
    let mut received_data = std::collections::HashSet::new();
    let timeout = Duration::from_secs(30);
    let start = std::time::Instant::now();

    while start.elapsed() < timeout && received_data.len() < symbols.len() {
        if let Ok(market_data) = rx.try_recv() {
            received_data.insert(market_data.symbol);
        }
        sleep(Duration::from_millis(100)).await;
    }

    // 验证是否收到所有交易对的数据
    for symbol in symbols {
        assert!(received_data.contains(symbol), "未收到{}的数据", symbol);
    }

    collector.stop().await.expect("停止收集器失败");
} 