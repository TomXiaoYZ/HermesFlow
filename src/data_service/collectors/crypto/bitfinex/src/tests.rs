#[cfg(test)]
mod tests {
    use super::*;
    use crate::collector::{BitfinexCollector, BitfinexCollectorConfig};
    use crate::models::{ExchangeStatus, OrderbookLevel, Symbol, TradeSide};
    use crate::processor::{BitfinexProcessor, BitfinexProcessorConfig};
    use crate::rest::{BitfinexRestClient, BitfinexRestConfig};
    use crate::websocket::{BitfinexWebsocketClient, BitfinexWebsocketConfig};
    use mockito::mock;
    use std::collections::HashMap;
    use std::time::Duration;
    use tokio::time::sleep;

    // REST客户端测试
    #[tokio::test]
    async fn test_rest_client() {
        // 模拟交易所信息接口
        let _m = mock("GET", "/conf/pub:info:pair")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"[
                {
                    "pair": "BTCUSD",
                    "price_precision": 2,
                    "initial_margin": "0.1",
                    "minimum_margin": "0.05",
                    "minimum_order_size": "0.001",
                    "maximum_order_size": "2000.0",
                    "minimum_price_increment": "0.01",
                    "is_trading": true
                }
            ]"#)
            .create();

        let config = BitfinexRestConfig {
            api_key: None,
            api_secret: None,
            timeout: 10,
        };

        let client = BitfinexRestClient::new(config);
        let info = client.get_symbols().await.unwrap();

        assert_eq!(info.name, "Bitfinex");
        assert_eq!(info.status, ExchangeStatus::Normal);
        assert_eq!(info.symbols.len(), 1);

        let symbol = &info.symbols[0];
        assert_eq!(symbol.base_currency, "BTC");
        assert_eq!(symbol.quote_currency, "USD");
        assert_eq!(symbol.price_precision, 2);
        assert_eq!(symbol.min_amount, 0.001);
        assert!(symbol.is_trading);
    }

    // WebSocket客户端测试
    #[tokio::test]
    async fn test_websocket_client() {
        let config = BitfinexWebsocketConfig {
            api_key: None,
            api_secret: None,
            ping_interval: 30,
        };

        let mut client = BitfinexWebsocketClient::new(config).await.unwrap();

        // 测试订阅
        client.subscribe_ticker("BTCUSD").await.unwrap();
        client.subscribe_orderbook("BTCUSD", "P0").await.unwrap();
        client.subscribe_trades("BTCUSD").await.unwrap();

        // 等待一些消息
        sleep(Duration::from_secs(1)).await;

        // 测试取消订阅
        for channel_id in client.channels.keys() {
            client.unsubscribe(*channel_id).await.unwrap();
        }

        // 关闭连接
        client.close().await.unwrap();
    }

    // 数据处理器测试
    #[tokio::test]
    async fn test_processor() {
        let config = BitfinexProcessorConfig {
            collector_config: BitfinexCollectorConfig::default(),
            ticker_cache_ttl: 1,
            orderbook_cache_ttl: 1,
            trades_cache_limit: 100,
            klines_cache_limit: 100,
        };

        let mut processor = BitfinexProcessor::new(config);

        // 启动处理器
        processor.start().await.unwrap();

        // 订阅交易对
        processor.subscribe("BTCUSD").await.unwrap();

        // 测试缓存机制
        let ticker = processor.get_ticker("BTCUSD").await.unwrap();
        assert!(ticker.is_some());

        let orderbook = processor.get_orderbook("BTCUSD").await.unwrap();
        assert!(orderbook.is_some());

        let trades = processor.get_trades("BTCUSD").await.unwrap();
        assert!(!trades.is_empty());

        let klines = processor.get_klines("BTCUSD", "1m").await.unwrap();
        assert!(!klines.is_empty());

        // 等待缓存过期
        sleep(Duration::from_secs(2)).await;

        // 测试数据更新
        let mut rx = processor.subscribe_updates();
        
        tokio::spawn(async move {
            while let Ok(event) = rx.recv().await {
                match event {
                    ProcessorEvent::TickerUpdate { symbol, data } => {
                        assert_eq!(symbol, "BTCUSD");
                        assert!(data.last_price > 0.0);
                    }
                    ProcessorEvent::OrderbookUpdate { symbol, data } => {
                        assert_eq!(symbol, "BTCUSD");
                        assert!(!data.bids.is_empty());
                        assert!(!data.asks.is_empty());
                    }
                    ProcessorEvent::TradeUpdate { symbol, data } => {
                        assert_eq!(symbol, "BTCUSD");
                        assert!(data.price > 0.0);
                        assert!(data.amount > 0.0);
                    }
                    ProcessorEvent::KlineUpdate { symbol, interval, data } => {
                        assert_eq!(symbol, "BTCUSD");
                        assert_eq!(interval, "1m");
                        assert!(data.close > 0.0);
                    }
                    ProcessorEvent::Error(error) => {
                        panic!("Unexpected error: {}", error);
                    }
                }
            }
        });

        // 等待一些更新
        sleep(Duration::from_secs(5)).await;

        // 取消订阅并停止处理器
        processor.unsubscribe("BTCUSD").await.unwrap();
        processor.stop().await.unwrap();
    }

    // 辅助函数：创建模拟的Ticker数据
    fn create_mock_ticker() -> Ticker {
        Ticker {
            symbol: "BTCUSD".to_string(),
            last_price: 50000.0,
            high_24h: 51000.0,
            low_24h: 49000.0,
            volume_24h: 1000.0,
            amount_24h: 50000000.0,
            price_change_24h: 2.5,
            timestamp: 1234567890000,
        }
    }

    // 辅助函数：创建模拟的订单簿数据
    fn create_mock_orderbook() -> Orderbook {
        Orderbook {
            symbol: "BTCUSD".to_string(),
            bids: vec![
                OrderbookLevel {
                    price: 49900.0,
                    amount: 1.0,
                    count: 5,
                },
                OrderbookLevel {
                    price: 49800.0,
                    amount: 2.0,
                    count: 3,
                },
            ],
            asks: vec![
                OrderbookLevel {
                    price: 50100.0,
                    amount: 1.5,
                    count: 4,
                },
                OrderbookLevel {
                    price: 50200.0,
                    amount: 2.5,
                    count: 2,
                },
            ],
            timestamp: 1234567890000,
        }
    }

    // 辅助函数：创建模拟的成交数据
    fn create_mock_trade() -> Trade {
        Trade {
            id: "123456".to_string(),
            symbol: "BTCUSD".to_string(),
            price: 50000.0,
            amount: 1.0,
            side: TradeSide::Buy,
            timestamp: 1234567890000,
        }
    }

    // 辅助函数：创建模拟的K线数据
    fn create_mock_kline() -> Kline {
        Kline {
            symbol: "BTCUSD".to_string(),
            open_time: 1234567890000,
            close_time: 1234567890060,
            open: 50000.0,
            high: 50100.0,
            low: 49900.0,
            close: 50050.0,
            volume: 100.0,
            amount: 5000000.0,
        }
    }
} 