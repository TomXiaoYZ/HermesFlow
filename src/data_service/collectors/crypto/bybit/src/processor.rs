use std::collections::HashMap;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde_json::Value;
use tracing::{debug, error, warn};

use common::{
    MarketData, DataQuality, MarketDataType, Trade, Kline, OrderBook, PriceLevel, Ticker,
    DataProcessor, Exchange, OrderBookLevel, Candlestick, Side,
};
use crate::error::BybitError;
use crate::models;
use crate::metrics;

/// Bybit数据处理器
pub struct BybitProcessor {
    symbol_info: HashMap<String, Value>,
    config: HashMap<String, String>,
}

impl BybitProcessor {
    pub fn new(config: HashMap<String, String>) -> Self {
        Self {
            symbol_info: HashMap::new(),
            config,
        }
    }

    /// 更新交易对信息
    pub fn update_symbol_info(&mut self, symbol: String, info: Value) {
        self.symbol_info.insert(symbol, info);
    }

    /// 处理WebSocket消息
    pub async fn process_ws_message(&self, message: &str) -> Result<Option<MarketData>, BybitError> {
        let value: Value = serde_json::from_str(message)
            .map_err(|e| BybitError::ParseError(format!("Failed to parse WebSocket message: {}", e)))?;

        if let Some(topic) = value.get("topic") {
            match topic.as_str() {
                Some("tickers") => self.process_ticker(&value),
                Some("trades") => self.process_trade(&value),
                Some("orderbook") => self.process_order_book(&value),
                Some("kline") => self.process_kline(&value),
                _ => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    /// 处理Ticker数据
    fn process_ticker(&self, data: &Value) -> Result<Option<MarketData>, BybitError> {
        if let Some(ticker_data) = data["data"].as_object() {
            let market_data = MarketData {
                exchange: Exchange::Bybit,
                symbol: ticker_data["symbol"].as_str()
                    .ok_or_else(|| BybitError::ParseError("Missing symbol".to_string()))?
                    .to_string(),
                timestamp: DateTime::parse_from_rfc3339(ticker_data["timestamp"].as_str()
                    .ok_or_else(|| BybitError::ParseError("Missing timestamp".to_string()))?)
                    .map_err(|e| BybitError::ParseError(format!("Invalid timestamp: {}", e)))?
                    .with_timezone(&Utc),
                data_type: MarketDataType::Trade(vec![Trade {
                    id: ticker_data["tradeId"].as_str()
                        .ok_or_else(|| BybitError::ParseError("Missing trade ID".to_string()))?
                        .to_string(),
                    price: Decimal::from_str_exact(ticker_data["lastPrice"].as_str()
                        .ok_or_else(|| BybitError::ParseError("Missing price".to_string()))?)
                        .map_err(|e| BybitError::ParseError(format!("Invalid price: {}", e)))?,
                    quantity: Decimal::from_str_exact(ticker_data["lastQty"].as_str()
                        .ok_or_else(|| BybitError::ParseError("Missing quantity".to_string()))?)
                        .map_err(|e| BybitError::ParseError(format!("Invalid quantity: {}", e)))?,
                    timestamp: DateTime::parse_from_rfc3339(ticker_data["timestamp"].as_str()
                        .ok_or_else(|| BybitError::ParseError("Missing timestamp".to_string()))?)
                        .map_err(|e| BybitError::ParseError(format!("Invalid timestamp: {}", e)))?
                        .with_timezone(&Utc),
                    side: if ticker_data["side"].as_str() == Some("Buy") { Side::Buy } else { Side::Sell },
                }]),
                quality: DataQuality::Real,
            };

            Ok(Some(market_data))
        } else {
            Ok(None)
        }
    }

    /// 处理Trade数据
    fn process_trade(&self, data: &Value) -> Result<Option<MarketData>, BybitError> {
        if let Some(trades) = data["data"].as_array() {
            let trades = trades.iter().map(|trade| {
                Ok(Trade {
                    id: trade["tradeId"].as_str()
                        .ok_or_else(|| BybitError::ParseError("Missing trade ID".to_string()))?
                        .to_string(),
                    price: Decimal::from_str_exact(trade["price"].as_str()
                        .ok_or_else(|| BybitError::ParseError("Missing price".to_string()))?)
                        .map_err(|e| BybitError::ParseError(format!("Invalid price: {}", e)))?,
                    quantity: Decimal::from_str_exact(trade["size"].as_str()
                        .ok_or_else(|| BybitError::ParseError("Missing size".to_string()))?)
                        .map_err(|e| BybitError::ParseError(format!("Invalid size: {}", e)))?,
                    timestamp: DateTime::parse_from_rfc3339(trade["time"].as_str()
                        .ok_or_else(|| BybitError::ParseError("Missing time".to_string()))?)
                        .map_err(|e| BybitError::ParseError(format!("Invalid time: {}", e)))?
                        .with_timezone(&Utc),
                    side: if trade["side"].as_str() == Some("Buy") { Side::Buy } else { Side::Sell },
                })
            }).collect::<Result<Vec<_>, _>>()?;

            let market_data = MarketData {
                exchange: Exchange::Bybit,
                symbol: data["symbol"].as_str()
                    .ok_or_else(|| BybitError::ParseError("Missing symbol".to_string()))?
                    .to_string(),
                timestamp: Utc::now(),
                data_type: MarketDataType::Trade(trades),
                quality: DataQuality::Real,
            };

            Ok(Some(market_data))
        } else {
            Ok(None)
        }
    }

    /// 处理OrderBook数据
    fn process_order_book(&self, data: &Value) -> Result<Option<MarketData>, BybitError> {
        if let Some(book_data) = data["data"].as_object() {
            let parse_level = |price: &str, size: &str| -> Result<OrderBookLevel, BybitError> {
                Ok(OrderBookLevel {
                    price: Decimal::from_str_exact(price)
                        .map_err(|e| BybitError::ParseError(format!("Invalid price: {}", e)))?,
                    quantity: Decimal::from_str_exact(size)
                        .map_err(|e| BybitError::ParseError(format!("Invalid size: {}", e)))?,
                })
            };

            let bids = book_data["bids"].as_array()
                .ok_or_else(|| BybitError::ParseError("Missing bids".to_string()))?
                .iter()
                .map(|level| {
                    let level = level.as_array()
                        .ok_or_else(|| BybitError::ParseError("Invalid bid level format".to_string()))?;
                    parse_level(
                        level[0].as_str().ok_or_else(|| BybitError::ParseError("Invalid bid price".to_string()))?,
                        level[1].as_str().ok_or_else(|| BybitError::ParseError("Invalid bid size".to_string()))?,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;

            let asks = book_data["asks"].as_array()
                .ok_or_else(|| BybitError::ParseError("Missing asks".to_string()))?
                .iter()
                .map(|level| {
                    let level = level.as_array()
                        .ok_or_else(|| BybitError::ParseError("Invalid ask level format".to_string()))?;
                    parse_level(
                        level[0].as_str().ok_or_else(|| BybitError::ParseError("Invalid ask price".to_string()))?,
                        level[1].as_str().ok_or_else(|| BybitError::ParseError("Invalid ask size".to_string()))?,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;

            let market_data = MarketData {
                exchange: Exchange::Bybit,
                symbol: data["symbol"].as_str()
                    .ok_or_else(|| BybitError::ParseError("Missing symbol".to_string()))?
                    .to_string(),
                timestamp: DateTime::parse_from_rfc3339(book_data["timestamp"].as_str()
                    .ok_or_else(|| BybitError::ParseError("Missing timestamp".to_string()))?)
                    .map_err(|e| BybitError::ParseError(format!("Invalid timestamp: {}", e)))?
                    .with_timezone(&Utc),
                data_type: MarketDataType::OrderBook { bids, asks },
                quality: DataQuality::Real,
            };

            Ok(Some(market_data))
        } else {
            Ok(None)
        }
    }

    /// 处理K线数据
    fn process_kline(&self, data: &Value) -> Result<Option<MarketData>, BybitError> {
        if let Some(kline_data) = data["data"].as_object() {
            let market_data = MarketData {
                exchange: Exchange::Bybit,
                symbol: data["symbol"].as_str()
                    .ok_or_else(|| BybitError::ParseError("Missing symbol".to_string()))?
                    .to_string(),
                timestamp: DateTime::parse_from_rfc3339(kline_data["timestamp"].as_str()
                    .ok_or_else(|| BybitError::ParseError("Missing timestamp".to_string()))?)
                    .map_err(|e| BybitError::ParseError(format!("Invalid timestamp: {}", e)))?
                    .with_timezone(&Utc),
                data_type: MarketDataType::Candlestick(Candlestick {
                    timestamp: DateTime::parse_from_rfc3339(kline_data["timestamp"].as_str()
                        .ok_or_else(|| BybitError::ParseError("Missing timestamp".to_string()))?)
                        .map_err(|e| BybitError::ParseError(format!("Invalid timestamp: {}", e)))?
                        .with_timezone(&Utc),
                    open: Decimal::from_str_exact(kline_data["open"].as_str()
                        .ok_or_else(|| BybitError::ParseError("Missing open price".to_string()))?)
                        .map_err(|e| BybitError::ParseError(format!("Invalid open price: {}", e)))?,
                    high: Decimal::from_str_exact(kline_data["high"].as_str()
                        .ok_or_else(|| BybitError::ParseError("Missing high price".to_string()))?)
                        .map_err(|e| BybitError::ParseError(format!("Invalid high price: {}", e)))?,
                    low: Decimal::from_str_exact(kline_data["low"].as_str()
                        .ok_or_else(|| BybitError::ParseError("Missing low price".to_string()))?)
                        .map_err(|e| BybitError::ParseError(format!("Invalid low price: {}", e)))?,
                    close: Decimal::from_str_exact(kline_data["close"].as_str()
                        .ok_or_else(|| BybitError::ParseError("Missing close price".to_string()))?)
                        .map_err(|e| BybitError::ParseError(format!("Invalid close price: {}", e)))?,
                    volume: Decimal::from_str_exact(kline_data["volume"].as_str()
                        .ok_or_else(|| BybitError::ParseError("Missing volume".to_string()))?)
                        .map_err(|e| BybitError::ParseError(format!("Invalid volume: {}", e)))?,
                }),
                quality: DataQuality::Real,
            };

            Ok(Some(market_data))
        } else {
            Ok(None)
        }
    }

    // 验证价格范围
    fn validate_price(&self, price: Decimal) -> bool {
        price > Decimal::ZERO
    }

    // 验证数量范围
    fn validate_amount(&self, amount: Decimal) -> bool {
        amount > Decimal::ZERO
    }

    // 验证时间戳
    fn validate_timestamp(&self, timestamp: i64) -> bool {
        let now = Utc::now().timestamp_millis();
        let diff = (now - timestamp).abs();
        // 允许5秒的时间差
        diff <= 5000
    }

    // 计算数据质量分数
    fn calculate_quality_score(&self, data: &MarketData) -> f64 {
        let mut score = 100.0;
        
        match &data.data_type {
            MarketDataType::Trade(trades) => {
                for trade in trades {
                    if !self.validate_price(trade.price) {
                        score -= 20.0;
                    }
                    if !self.validate_amount(trade.quantity) {
                        score -= 20.0;
                    }
                    if !self.validate_timestamp(trade.timestamp.timestamp_millis()) {
                        score -= 10.0;
                    }
                }
            }
            MarketDataType::OrderBook { bids, asks } => {
                // 验证买卖盘价格顺序
                if !bids.is_empty() && !asks.is_empty() {
                    if bids[0].price >= asks[0].price {
                        score -= 50.0;
                    }
                }
                
                // 验证价格和数量
                for level in bids.iter().chain(asks.iter()) {
                    if !self.validate_price(level.price) {
                        score -= 10.0;
                    }
                    if !self.validate_amount(level.quantity) {
                        score -= 10.0;
                    }
                }
            }
            MarketDataType::Candlestick(k) => {
                // 验证K线数据的合理性
                if k.high < k.low || k.open < k.low || k.close < k.low || 
                   k.high < k.open || k.high < k.close {
                    score -= 50.0;
                }
                if !self.validate_timestamp(k.timestamp.timestamp_millis()) {
                    score -= 10.0;
                }
            }
        }

        score.max(0.0)
    }

    // 标准化交易对格式（Bybit特有的格式转换）
    fn normalize_symbol(&self, symbol: &str) -> String {
        symbol.to_uppercase()
    }
}

#[async_trait]
impl DataProcessor for BybitProcessor {
    async fn process(&self, mut data: MarketData) -> Result<MarketData, Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();

        // 标准化交易所名称
        data.exchange = Exchange::Bybit;

        // 标准化交易对格式
        data.symbol = self.normalize_symbol(&data.symbol);

        // 根据数据类型进行处理
        match &mut data.data_type {
            MarketDataType::Trade(trades) => {
                for trade in trades {
                    // 标准化交易方向
                    if trade.side == Side::Unknown {
                        trade.side = Side::Buy; // 默认设置为买入
                    }
                }
            }
            MarketDataType::OrderBook { bids, asks } => {
                // 排序买卖盘（买盘降序，卖盘升序）
                bids.sort_by(|a, b| b.price.cmp(&a.price));
                asks.sort_by(|a, b| a.price.cmp(&b.price));

                // 移除价格为0的档位
                bids.retain(|level| level.price > Decimal::ZERO);
                asks.retain(|level| level.price > Decimal::ZERO);
            }
            MarketDataType::Candlestick(_) => {
                // K线数据已经在之前的解析阶段标准化
            }
        }

        // 记录处理延迟
        metrics::record_rest_latency(
            "bybit",
            "data_processing",
            start_time,
        );

        Ok(data)
    }

    async fn validate(&self, data: &MarketData) -> Result<DataQuality, Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();

        // 计算数据质量分数
        let quality_score = self.calculate_quality_score(data);
        
        // 更新监控指标
        metrics::update_data_quality("bybit", "market_data", quality_score);

        // 记录验证延迟
        metrics::record_rest_latency(
            "bybit",
            "data_validation",
            start_time,
        );

        // 根据质量分数确定数据质量级别
        let quality = if quality_score >= 90.0 {
            DataQuality::Real
        } else if quality_score >= 60.0 {
            DataQuality::Delay
        } else {
            DataQuality::History
        };

        Ok(quality)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[tokio::test]
    async fn test_process_trade() {
        let processor = BybitProcessor::new(HashMap::new());
        
        let trade = Trade {
            id: "1".to_string(),
            timestamp: Utc::now(),
            price: dec!(50000),
            quantity: dec!(1),
            side: Side::Unknown,
            quality: DataQuality::Real,
        };

        let input = MarketData {
            exchange: Exchange::Bybit,
            symbol: "BTCUSDT".to_string(),
            timestamp: Utc::now(),
            data_type: MarketDataType::Trade(vec![trade]),
            quality: DataQuality::Real,
        };

        let processed = processor.process(input).await.unwrap();
        
        if let MarketDataType::Trade(trades) = processed.data_type {
            assert_eq!(trades[0].side, Side::Buy);
            assert_eq!(processed.symbol, "BTCUSDT");
        } else {
            panic!("Wrong market data type");
        }
    }

    #[tokio::test]
    async fn test_process_orderbook() {
        let processor = BybitProcessor::new(HashMap::new());
        
        let input = MarketData {
            exchange: Exchange::Bybit,
            symbol: "BTCUSDT".to_string(),
            timestamp: Utc::now(),
            data_type: MarketDataType::OrderBook {
                bids: vec![
                    OrderBookLevel {
                        price: dec!(49000),
                        quantity: dec!(1),
                    },
                    OrderBookLevel {
                        price: dec!(50000),
                        quantity: dec!(1),
                    },
                ],
                asks: vec![
                    OrderBookLevel {
                        price: dec!(51000),
                        quantity: dec!(1),
                    },
                    OrderBookLevel {
                        price: dec!(50500),
                        quantity: dec!(1),
                    },
                ],
            },
            quality: DataQuality::Real,
        };

        let processed = processor.process(input).await.unwrap();
        
        if let MarketDataType::OrderBook { bids, asks } = processed.data_type {
            assert_eq!(bids[0].price, dec!(50000));
            assert_eq!(asks[0].price, dec!(50500));
            assert_eq!(processed.symbol, "BTCUSDT");
        } else {
            panic!("Wrong market data type");
        }
    }

    #[tokio::test]
    async fn test_validate_data() {
        let processor = BybitProcessor::new(HashMap::new());
        
        let good_data = MarketData {
            exchange: Exchange::Bybit,
            symbol: "BTCUSDT".to_string(),
            timestamp: Utc::now(),
            data_type: MarketDataType::Trade(vec![Trade {
                id: "1".to_string(),
                timestamp: Utc::now(),
                price: dec!(50000),
                quantity: dec!(1),
                side: Side::Buy,
                quality: DataQuality::Real,
            }]),
            quality: DataQuality::Real,
        };

        let quality = processor.validate(&good_data).await.unwrap();
        assert_eq!(quality, DataQuality::Real);

        let bad_data = MarketData {
            exchange: Exchange::Bybit,
            symbol: "BTCUSDT".to_string(),
            timestamp: Utc::now(),
            data_type: MarketDataType::Trade(vec![Trade {
                id: "1".to_string(),
                timestamp: Utc::now(),
                price: dec!(0),
                quantity: dec!(0),
                side: Side::Buy,
                quality: DataQuality::Real,
            }]),
            quality: DataQuality::Real,
        };

        let quality = processor.validate(&bad_data).await.unwrap();
        assert_eq!(quality, DataQuality::History);
    }

    #[tokio::test]
    async fn test_symbol_normalization() {
        let processor = BybitProcessor::new(HashMap::new());
        
        assert_eq!(processor.normalize_symbol("BTCUSDT"), "BTCUSDT");
        assert_eq!(processor.normalize_symbol("ethusdt"), "ETHUSDT");
    }
} 