use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde_json::Value;
use async_trait::async_trait;
use tracing::{debug, error, warn};

use crate::error::BinanceError;
use crate::collectors::common::{
    DataProcessor, MarketData, DataQuality,
    MarketDataType, Trade, Kline, OrderBook, PriceLevel, Ticker,
};
use crate::types::{Symbol, OrderBook as BinanceOrderBook, Kline as BinanceKline, TradeEvent, TickerEvent, DepthEvent};
use crate::rest::RestClient;
use crate::websocket::WebSocketClient;

/// 市场数据缓存
#[derive(Debug, Default)]
struct MarketDataCache {
    /// 交易对信息缓存
    symbols: HashMap<String, Symbol>,
    /// 最新价格缓存
    latest_prices: HashMap<String, Decimal>,
    /// 订单簿缓存
    order_books: HashMap<String, BinanceOrderBook>,
    /// K线缓存
    klines: HashMap<String, Vec<BinanceKline>>,
}

/// Binance数据处理器
pub struct BinanceProcessor {
    /// REST API 客户端
    rest_client: RestClient,
    /// WebSocket 客户端
    ws_client: Option<WebSocketClient>,
    /// 数据缓存
    cache: Arc<RwLock<MarketDataCache>>,
    /// 配置的交易对
    symbols: Vec<String>,
}

impl BinanceProcessor {
    pub fn new(rest_client: RestClient, symbols: Vec<String>) -> Self {
        Self {
            rest_client,
            ws_client: None,
            cache: Arc::new(RwLock::new(MarketDataCache::default())),
            symbols,
        }
    }

    /// 初始化处理器
    pub async fn init(&mut self) -> Result<(), BinanceError> {
        // 获取并缓存交易对信息
        self.update_symbols().await?;
        
        // 获取并缓存初始市场数据
        for symbol in &self.symbols {
            self.update_order_book(symbol).await?;
            self.update_price(symbol).await?;
            self.update_klines(symbol, "1m", None, None, Some(100)).await?;
        }

        Ok(())
    }

    /// 更新交易对信息
    async fn update_symbols(&mut self) -> Result<(), BinanceError> {
        let symbols = self.rest_client.get_exchange_info().await?;
        let mut cache = self.cache.write().await;
        
        for symbol in symbols {
            if self.symbols.contains(&symbol.symbol) {
                cache.symbols.insert(symbol.symbol.clone(), symbol);
            }
        }

        Ok(())
    }

    /// 更新订单簿
    async fn update_order_book(&self, symbol: &str) -> Result<(), BinanceError> {
        let order_book = self.rest_client.get_order_book(symbol, Some(100)).await?;
        let mut cache = self.cache.write().await;
        cache.order_books.insert(symbol.to_string(), order_book);
        Ok(())
    }

    /// 更新最新价格
    async fn update_price(&self, symbol: &str) -> Result<(), BinanceError> {
        let price = self.rest_client.get_price(symbol).await?;
        let mut cache = self.cache.write().await;
        cache.latest_prices.insert(symbol.to_string(), price);
        Ok(())
    }

    /// 更新K线数据
    async fn update_klines(
        &self,
        symbol: &str,
        interval: &str,
        start_time: Option<i64>,
        end_time: Option<i64>,
        limit: Option<u32>,
    ) -> Result<(), BinanceError> {
        let klines = self.rest_client
            .get_klines(symbol, interval, start_time, end_time, limit)
            .await?;
        
        let mut cache = self.cache.write().await;
        cache.klines.insert(symbol.to_string(), klines);
        Ok(())
    }

    /// 解析交易数据
    fn parse_trade(data: &Value) -> Result<Trade, BinanceError> {
        let symbol = data["s"].as_str()
            .ok_or_else(|| BinanceError::ParseError("Missing symbol in trade data".to_string()))?
            .to_string();

        let trade_id = data["t"].as_i64()
            .ok_or_else(|| BinanceError::ParseError("Missing trade id".to_string()))?
            .to_string();

        let price = data["p"].as_str()
            .ok_or_else(|| BinanceError::ParseError("Missing price".to_string()))?;
        let price = Decimal::from_str_exact(price)
            .map_err(|e| BinanceError::ParseError(format!("Invalid price: {}", e)))?;

        let quantity = data["q"].as_str()
            .ok_or_else(|| BinanceError::ParseError("Missing quantity".to_string()))?;
        let quantity = Decimal::from_str_exact(quantity)
            .map_err(|e| BinanceError::ParseError(format!("Invalid quantity: {}", e)))?;

        let timestamp = data["T"].as_i64()
            .ok_or_else(|| BinanceError::ParseError("Missing timestamp".to_string()))?;
        let trade_time = DateTime::from_timestamp_millis(timestamp)
            .ok_or_else(|| BinanceError::ParseError("Invalid timestamp".to_string()))?;

        let is_buyer_maker = data["m"].as_bool()
            .ok_or_else(|| BinanceError::ParseError("Missing buyer maker flag".to_string()))?;

        Ok(Trade {
            exchange: "binance".to_string(),
            symbol,
            trade_id,
            price,
            quantity,
            side: if is_buyer_maker { crate::collectors::common::TradeSide::Sell } else { crate::collectors::common::TradeSide::Buy },
            trade_time,
            is_maker: is_buyer_maker,
            metadata: HashMap::new(),
        })
    }

    /// 解析K线数据
    fn parse_kline(data: &Value) -> Result<Kline, BinanceError> {
        let k = &data["k"];
        
        let symbol = k["s"].as_str()
            .ok_or_else(|| BinanceError::ParseError("Missing symbol in kline data".to_string()))?
            .to_string();

        let interval = k["i"].as_str()
            .ok_or_else(|| BinanceError::ParseError("Missing interval".to_string()))?
            .to_string();

        let start_time = k["t"].as_i64()
            .ok_or_else(|| BinanceError::ParseError("Missing start time".to_string()))?;
        let start_time = DateTime::from_timestamp_millis(start_time)
            .ok_or_else(|| BinanceError::ParseError("Invalid start time".to_string()))?;

        let close_time = k["T"].as_i64()
            .ok_or_else(|| BinanceError::ParseError("Missing close time".to_string()))?;
        let close_time = DateTime::from_timestamp_millis(close_time)
            .ok_or_else(|| BinanceError::ParseError("Invalid close time".to_string()))?;

        Ok(Kline {
            exchange: "binance".to_string(),
            symbol,
            interval,
            start_time,
            close_time,
            open: Decimal::from_str_exact(k["o"].as_str().unwrap_or("0")).unwrap_or_default(),
            high: Decimal::from_str_exact(k["h"].as_str().unwrap_or("0")).unwrap_or_default(),
            low: Decimal::from_str_exact(k["l"].as_str().unwrap_or("0")).unwrap_or_default(),
            close: Decimal::from_str_exact(k["c"].as_str().unwrap_or("0")).unwrap_or_default(),
            volume: Decimal::from_str_exact(k["v"].as_str().unwrap_or("0")).unwrap_or_default(),
            quote_volume: Decimal::from_str_exact(k["q"].as_str().unwrap_or("0")).unwrap_or_default(),
            trades_count: k["n"].as_i64().unwrap_or_default(),
            is_closed: k["x"].as_bool().unwrap_or_default(),
        })
    }

    /// 解析订单簿数据
    fn parse_orderbook(data: &Value) -> Result<OrderBook, BinanceError> {
        let symbol = data["s"].as_str()
            .ok_or_else(|| BinanceError::ParseError("Missing symbol in orderbook".to_string()))?
            .to_string();

        let timestamp = data["T"].as_i64()
            .ok_or_else(|| BinanceError::ParseError("Missing timestamp".to_string()))?;
        let timestamp = DateTime::from_timestamp_millis(timestamp)
            .ok_or_else(|| BinanceError::ParseError("Invalid timestamp".to_string()))?;

        let parse_level = |arr: &[Value]| -> Result<PriceLevel, BinanceError> {
            if arr.len() < 2 {
                return Err(BinanceError::ParseError("Invalid price level data".to_string()));
            }
            
            let price = Decimal::from_str_exact(arr[0].as_str().unwrap_or("0"))
                .map_err(|e| BinanceError::ParseError(format!("Invalid price: {}", e)))?;
            let quantity = Decimal::from_str_exact(arr[1].as_str().unwrap_or("0"))
                .map_err(|e| BinanceError::ParseError(format!("Invalid quantity: {}", e)))?;
            
            Ok(PriceLevel { price, quantity })
        };

        let bids = data["b"].as_array()
            .ok_or_else(|| BinanceError::ParseError("Missing bids".to_string()))?
            .iter()
            .filter_map(|v| v.as_array())
            .map(|arr| parse_level(arr))
            .collect::<Result<Vec<_>, _>>()?;

        let asks = data["a"].as_array()
            .ok_or_else(|| BinanceError::ParseError("Missing asks".to_string()))?
            .iter()
            .filter_map(|v| v.as_array())
            .map(|arr| parse_level(arr))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(OrderBook {
            exchange: "binance".to_string(),
            symbol,
            timestamp,
            bids,
            asks,
            update_id: data["u"].as_i64().unwrap_or_default(),
            metadata: HashMap::new(),
        })
    }

    /// 解析Ticker数据
    fn parse_ticker(data: &Value) -> Result<Ticker, BinanceError> {
        let symbol = data["s"].as_str()
            .ok_or_else(|| BinanceError::ParseError("Missing symbol in ticker".to_string()))?
            .to_string();

        let timestamp = data["E"].as_i64()
            .ok_or_else(|| BinanceError::ParseError("Missing timestamp".to_string()))?;
        let timestamp = DateTime::from_timestamp_millis(timestamp)
            .ok_or_else(|| BinanceError::ParseError("Invalid timestamp".to_string()))?;

        Ok(Ticker {
            exchange: "binance".to_string(),
            symbol,
            timestamp,
            last_price: Decimal::from_str_exact(data["c"].as_str().unwrap_or("0")).unwrap_or_default(),
            last_quantity: data["Q"].as_str()
                .and_then(|s| Decimal::from_str_exact(s).ok()),
            best_bid: Decimal::from_str_exact(data["b"].as_str().unwrap_or("0")).unwrap_or_default(),
            best_ask: Decimal::from_str_exact(data["a"].as_str().unwrap_or("0")).unwrap_or_default(),
            volume_24h: Decimal::from_str_exact(data["v"].as_str().unwrap_or("0")).unwrap_or_default(),
            quote_volume_24h: Decimal::from_str_exact(data["q"].as_str().unwrap_or("0")).unwrap_or_default(),
            high_24h: data["h"].as_str().and_then(|s| Decimal::from_str_exact(s).ok()),
            low_24h: data["l"].as_str().and_then(|s| Decimal::from_str_exact(s).ok()),
            open_24h: data["o"].as_str().and_then(|s| Decimal::from_str_exact(s).ok()),
            metadata: HashMap::new(),
        })
    }

    /// 验证数据质量
    fn validate_data(&self, data: &MarketData) -> DataQuality {
        let received_latency = (Utc::now().timestamp_millis() - data.timestamp.timestamp_millis()).max(0);
        
        let is_valid = match data.data_type {
            MarketDataType::Trade => data.raw_data["p"].is_string() && data.raw_data["q"].is_string(),
            MarketDataType::OrderBook => data.raw_data["b"].is_array() && data.raw_data["a"].is_array(),
            MarketDataType::Kline => data.raw_data["k"].is_object(),
            MarketDataType::Ticker => data.raw_data["c"].is_string(),
            _ => true,
        };

        DataQuality {
            latency: received_latency,
            is_gap: false, // 需要进一步实现gap检测
            gap_size: None,
            is_valid,
            error_type: if !is_valid { Some("Invalid data format".to_string()) } else { None },
            metadata: HashMap::new(),
        }
    }
}

#[async_trait]
impl DataProcessor for BinanceProcessor {
    type Error = BinanceError;

    async fn process(&self, mut data: MarketData) -> Result<MarketData, Self::Error> {
        // 根据数据类型进行解析和转换
        match data.data_type {
            MarketDataType::Trade => {
                let trade = Self::parse_trade(&data.raw_data)?;
                data.metadata.insert("trade_id".to_string(), trade.trade_id.clone());
                data.metadata.insert("is_maker".to_string(), trade.is_maker.to_string());
            }
            MarketDataType::Kline => {
                let kline = Self::parse_kline(&data.raw_data)?;
                data.metadata.insert("interval".to_string(), kline.interval.clone());
                data.metadata.insert("is_closed".to_string(), kline.is_closed.to_string());
            }
            MarketDataType::OrderBook => {
                let orderbook = Self::parse_orderbook(&data.raw_data)?;
                data.metadata.insert("update_id".to_string(), orderbook.update_id.to_string());
                data.metadata.insert("bids_count".to_string(), orderbook.bids.len().to_string());
                data.metadata.insert("asks_count".to_string(), orderbook.asks.len().to_string());
            }
            MarketDataType::Ticker => {
                let ticker = Self::parse_ticker(&data.raw_data)?;
                if let Some(vol) = ticker.volume_24h.to_string().parse().ok() {
                    data.metadata.insert("volume_24h".to_string(), vol);
                }
            }
            _ => {
                warn!("Unsupported data type: {:?}", data.data_type);
            }
        }

        Ok(data)
    }

    async fn validate(&self, data: &MarketData) -> Result<DataQuality, Self::Error> {
        Ok(self.validate_data(data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn create_processor() -> BinanceProcessor {
        BinanceProcessor::new(RestClient::new("https://api.binance.com", None, None), vec!["BTCUSDT".to_string()])
    }

    #[tokio::test]
    async fn test_parse_trade() {
        // 正常场景测试
        let data = json!({
            "e": "trade",
            "E": 1672515782136,
            "s": "BTCUSDT",
            "t": 123456,
            "p": "16500.50",
            "q": "0.12345",
            "T": 1672515782136,
            "m": true,
            "M": true
        });

        let trade = BinanceProcessor::parse_trade(&data).unwrap();
        assert_eq!(trade.symbol, "BTCUSDT");
        assert_eq!(trade.trade_id, "123456");
        assert_eq!(trade.price.to_string(), "16500.50");
        assert_eq!(trade.quantity.to_string(), "0.12345");
        assert_eq!(trade.is_maker, true);
        assert!(matches!(trade.side, crate::collectors::common::TradeSide::Sell));

        // 错误场景测试
        let invalid_data = json!({
            "e": "trade",
            "s": "BTCUSDT",  // 缺少必要字段
            "p": "16500.50",
            "q": "0.12345"
        });
        assert!(BinanceProcessor::parse_trade(&invalid_data).is_err());

        // 价格格式错误
        let invalid_price = json!({
            "e": "trade",
            "E": 1672515782136,
            "s": "BTCUSDT",
            "t": 123456,
            "p": "invalid",  // 无效的价格格式
            "q": "0.12345",
            "T": 1672515782136,
            "m": true
        });
        assert!(BinanceProcessor::parse_trade(&invalid_price).is_err());
    }

    #[tokio::test]
    async fn test_parse_kline() {
        // 正常场景测试
        let data = json!({
            "e": "kline",
            "E": 1672515782136,
            "s": "BTCUSDT",
            "k": {
                "t": 1672515780000,
                "T": 1672515839999,
                "s": "BTCUSDT",
                "i": "1m",
                "o": "16500.00",
                "h": "16505.00",
                "l": "16499.00",
                "c": "16503.50",
                "v": "10.5",
                "n": 100,
                "x": true,
                "q": "173275.25",
                "V": "5.2",
                "Q": "85850.25"
            }
        });

        let kline = BinanceProcessor::parse_kline(&data).unwrap();
        assert_eq!(kline.symbol, "BTCUSDT");
        assert_eq!(kline.interval, "1m");
        assert_eq!(kline.open.to_string(), "16500.00");
        assert_eq!(kline.high.to_string(), "16505.00");
        assert_eq!(kline.low.to_string(), "16499.00");
        assert_eq!(kline.close.to_string(), "16503.50");
        assert_eq!(kline.volume.to_string(), "10.5");
        assert_eq!(kline.trades_count, 100);
        assert!(kline.is_closed);

        // 缺少K线数据
        let missing_kline = json!({
            "e": "kline",
            "E": 1672515782136,
            "s": "BTCUSDT"
        });
        assert!(BinanceProcessor::parse_kline(&missing_kline).is_err());

        // 无效的时间戳
        let invalid_timestamp = json!({
            "e": "kline",
            "E": 1672515782136,
            "s": "BTCUSDT",
            "k": {
                "t": "invalid",
                "T": 1672515839999,
                "s": "BTCUSDT",
                "i": "1m",
                "o": "16500.00",
                "h": "16505.00",
                "l": "16499.00",
                "c": "16503.50"
            }
        });
        assert!(BinanceProcessor::parse_kline(&invalid_timestamp).is_err());
    }

    #[tokio::test]
    async fn test_parse_orderbook() {
        // 正常场景测试
        let data = json!({
            "e": "depthUpdate",
            "E": 1672515782136,
            "s": "BTCUSDT",
            "T": 1672515782136,
            "u": 12345,
            "b": [
                ["16500.50", "1.5"],
                ["16500.00", "2.3"]
            ],
            "a": [
                ["16501.00", "1.2"],
                ["16501.50", "0.8"]
            ]
        });

        let orderbook = BinanceProcessor::parse_orderbook(&data).unwrap();
        assert_eq!(orderbook.symbol, "BTCUSDT");
        assert_eq!(orderbook.update_id, 12345);
        assert_eq!(orderbook.bids.len(), 2);
        assert_eq!(orderbook.asks.len(), 2);
        assert_eq!(orderbook.bids[0].price.to_string(), "16500.50");
        assert_eq!(orderbook.bids[0].quantity.to_string(), "1.5");
        assert_eq!(orderbook.asks[0].price.to_string(), "16501.00");
        assert_eq!(orderbook.asks[0].quantity.to_string(), "1.2");

        // 空订单簿测试
        let empty_orderbook = json!({
            "e": "depthUpdate",
            "E": 1672515782136,
            "s": "BTCUSDT",
            "T": 1672515782136,
            "u": 12345,
            "b": [],
            "a": []
        });
        let result = BinanceProcessor::parse_orderbook(&empty_orderbook).unwrap();
        assert_eq!(result.bids.len(), 0);
        assert_eq!(result.asks.len(), 0);

        // 无效的价格格式
        let invalid_price = json!({
            "e": "depthUpdate",
            "E": 1672515782136,
            "s": "BTCUSDT",
            "T": 1672515782136,
            "u": 12345,
            "b": [
                ["invalid", "1.5"]  // 无效的价格格式
            ],
            "a": []
        });
        assert!(BinanceProcessor::parse_orderbook(&invalid_price).is_err());
    }

    #[tokio::test]
    async fn test_parse_ticker() {
        // 正常场景测试
        let data = json!({
            "e": "24hrTicker",
            "E": 1672515782136,
            "s": "BTCUSDT",
            "p": "100.00",
            "P": "0.60",
            "c": "16500.50",
            "Q": "0.12345",
            "o": "16400.50",
            "h": "16800.00",
            "l": "16300.00",
            "v": "5000.50",
            "q": "82508287.25",
            "O": 1672429382136,
            "C": 1672515782136,
            "b": "16500.00",
            "a": "16501.00"
        });

        let ticker = BinanceProcessor::parse_ticker(&data).unwrap();
        assert_eq!(ticker.symbol, "BTCUSDT");
        assert_eq!(ticker.last_price.to_string(), "16500.50");
        assert_eq!(ticker.best_bid.to_string(), "16500.00");
        assert_eq!(ticker.best_ask.to_string(), "16501.00");
        assert_eq!(ticker.volume_24h.to_string(), "5000.50");
        assert_eq!(ticker.quote_volume_24h.to_string(), "82508287.25");

        // 缺少必要字段
        let missing_fields = json!({
            "e": "24hrTicker",
            "E": 1672515782136,
            "s": "BTCUSDT"
        });
        assert!(BinanceProcessor::parse_ticker(&missing_fields).is_err());

        // 零值测试
        let zero_values = json!({
            "e": "24hrTicker",
            "E": 1672515782136,
            "s": "BTCUSDT",
            "p": "0",
            "P": "0",
            "c": "0",
            "Q": "0",
            "o": "0",
            "h": "0",
            "l": "0",
            "v": "0",
            "q": "0",
            "b": "0",
            "a": "0"
        });
        let ticker = BinanceProcessor::parse_ticker(&zero_values).unwrap();
        assert_eq!(ticker.last_price.to_string(), "0");
        assert_eq!(ticker.volume_24h.to_string(), "0");
    }

    #[tokio::test]
    async fn test_data_validation() {
        let processor = create_processor();
        
        // 测试有效的交易数据
        let valid_trade = MarketData {
            exchange: "binance".to_string(),
            symbol: "BTCUSDT".to_string(),
            data_type: MarketDataType::Trade,
            timestamp: Utc::now(),
            raw_data: json!({
                "p": "16500.50",
                "q": "1.5"
            }),
            metadata: HashMap::new(),
        };
        let quality = processor.validate_data(&valid_trade);
        assert!(quality.is_valid);
        assert!(quality.error_type.is_none());

        // 测试无效的交易数据
        let invalid_trade = MarketData {
            exchange: "binance".to_string(),
            symbol: "BTCUSDT".to_string(),
            data_type: MarketDataType::Trade,
            timestamp: Utc::now(),
            raw_data: json!({
                "p": 16500.50,  // 数字而不是字符串
                "q": "1.5"
            }),
            metadata: HashMap::new(),
        };
        let quality = processor.validate_data(&invalid_trade);
        assert!(!quality.is_valid);
        assert!(quality.error_type.is_some());

        // 测试延迟计算
        let old_data = MarketData {
            exchange: "binance".to_string(),
            symbol: "BTCUSDT".to_string(),
            data_type: MarketDataType::Trade,
            timestamp: Utc::now() - chrono::Duration::seconds(5),
            raw_data: json!({
                "p": "16500.50",
                "q": "1.5"
            }),
            metadata: HashMap::new(),
        };
        let quality = processor.validate_data(&old_data);
        assert!(quality.latency >= 5000); // 至少5秒的延迟
    }

    #[tokio::test]
    async fn test_process_data() {
        let processor = create_processor();
        
        // 测试处理交易数据
        let trade_data = MarketData {
            exchange: "binance".to_string(),
            symbol: "BTCUSDT".to_string(),
            data_type: MarketDataType::Trade,
            timestamp: Utc::now(),
            raw_data: json!({
                "e": "trade",
                "E": 1672515782136,
                "s": "BTCUSDT",
                "t": 123456,
                "p": "16500.50",
                "q": "0.12345",
                "T": 1672515782136,
                "m": true,
                "M": true
            }),
            metadata: HashMap::new(),
        };

        let processed_data = processor.process(trade_data).await.unwrap();
        assert!(processed_data.metadata.contains_key("trade_id"));
        assert!(processed_data.metadata.contains_key("is_maker"));
        assert_eq!(processed_data.metadata["trade_id"], "123456");
        assert_eq!(processed_data.metadata["is_maker"], "true");

        // 测试处理K线数据
        let kline_data = MarketData {
            exchange: "binance".to_string(),
            symbol: "BTCUSDT".to_string(),
            data_type: MarketDataType::Kline,
            timestamp: Utc::now(),
            raw_data: json!({
                "e": "kline",
                "E": 1672515782136,
                "s": "BTCUSDT",
                "k": {
                    "t": 1672515780000,
                    "T": 1672515839999,
                    "s": "BTCUSDT",
                    "i": "1m",
                    "o": "16500.00",
                    "h": "16505.00",
                    "l": "16499.00",
                    "c": "16503.50",
                    "v": "10.5",
                    "n": 100,
                    "x": true,
                    "q": "173275.25"
                }
            }),
            metadata: HashMap::new(),
        };

        let processed_data = processor.process(kline_data).await.unwrap();
        assert!(processed_data.metadata.contains_key("interval"));
        assert!(processed_data.metadata.contains_key("is_closed"));
        assert_eq!(processed_data.metadata["interval"], "1m");
        assert_eq!(processed_data.metadata["is_closed"], "true");

        // 测试处理未知数据类型
        let unknown_data = MarketData {
            exchange: "binance".to_string(),
            symbol: "BTCUSDT".to_string(),
            data_type: MarketDataType::Unknown,
            timestamp: Utc::now(),
            raw_data: json!({}),
            metadata: HashMap::new(),
        };

        let processed_data = processor.process(unknown_data).await.unwrap();
        assert!(processed_data.metadata.is_empty());
    }

    #[tokio::test]
    async fn test_symbol_info() {
        let mut processor = create_processor();
        
        // 测试更新交易对信息
        let info = json!({
            "baseAsset": "BTC",
            "quoteAsset": "USDT",
            "filters": [
                {
                    "filterType": "PRICE_FILTER",
                    "minPrice": "0.01",
                    "maxPrice": "1000000.00",
                    "tickSize": "0.01"
                }
            ]
        });

        processor.update_symbols().await.unwrap();
        assert_eq!(processor.cache.read().await.symbols.len(), 1);
        assert_eq!(processor.cache.read().await.symbols["BTCUSDT"], info);
    }
} 