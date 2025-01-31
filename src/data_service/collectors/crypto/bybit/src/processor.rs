use std::collections::HashMap;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde_json::Value;
use tracing::{error, info, warn};

use common::{
    MarketData, DataQuality, MarketDataType,
    OrderBookLevel, Trade, Candlestick, Ticker,
};
use crate::error::{BybitError, ParseErrorKind};
use crate::models::{WebSocketResponse, InstrumentInfo};

/// 数据处理器
pub struct BybitProcessor {
    /// 交易对信息缓存
    symbol_info: HashMap<String, Value>,
}

impl BybitProcessor {
    /// 创建新的数据处理器实例
    pub fn new() -> Self {
        Self {
            symbol_info: HashMap::new(),
        }
    }

    /// 更新交易对信息
    pub fn update_symbol_info(&mut self, symbol: String, info: Value) {
        self.symbol_info.insert(symbol, info);
    }

    /// 处理 WebSocket 消息
    pub async fn process_ws_message(&self, message: &str) -> Result<Option<MarketData>, BybitError> {
        let response: WebSocketResponse = serde_json::from_str(message)
            .map_err(|e| BybitError::ParseError {
                kind: ParseErrorKind::JsonError,
                source: Some(Box::new(e)),
            })?;

        if let Some(topic) = response.topic {
            let parts: Vec<&str> = topic.split('.').collect();
            if parts.len() < 2 {
                return Ok(None);
            }

            let channel = parts[0];
            let symbol = parts[1];

            match channel {
                "orderbook" => self.process_orderbook(symbol, response.data),
                "trade" => self.process_trades(symbol, response.data),
                "tickers" => self.process_ticker(symbol, response.data),
                "kline" => self.process_kline(symbol, response.data),
                _ => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    /// 处理深度数据
    fn process_orderbook(&self, symbol: &str, data: Option<Value>) -> Result<Option<MarketData>, BybitError> {
        if let Some(Value::Object(obj)) = data {
            let ts = obj.get("ts")
                .and_then(|v| v.as_str())
                .ok_or_else(|| BybitError::ParseError {
                    kind: ParseErrorKind::MissingField("timestamp".to_string()),
                    source: None,
                })?;
            let timestamp = ts.parse::<i64>()
                .map_err(|e| BybitError::ParseError {
                    kind: ParseErrorKind::TimeParseError,
                    source: Some(Box::new(e)),
                })?;

            let mut bids = Vec::new();
            let mut asks = Vec::new();

            if let Some(Value::Array(bid_array)) = obj.get("bids") {
                for bid in bid_array {
                    if let Value::Array(level) = bid {
                        if level.len() >= 2 {
                            let price = level[0].as_str()
                                .ok_or_else(|| BybitError::ParseError {
                                    kind: ParseErrorKind::InvalidFieldType("bid price".to_string()),
                                    source: None,
                                })?
                                .parse::<Decimal>()
                                .map_err(|e| BybitError::ParseError {
                                    kind: ParseErrorKind::NumberParseError,
                                    source: Some(Box::new(e)),
                                })?;
                            let quantity = level[1].as_str()
                                .ok_or_else(|| BybitError::ParseError {
                                    kind: ParseErrorKind::InvalidFieldType("bid quantity".to_string()),
                                    source: None,
                                })?
                                .parse::<Decimal>()
                                .map_err(|e| BybitError::ParseError {
                                    kind: ParseErrorKind::NumberParseError,
                                    source: Some(Box::new(e)),
                                })?;
                            bids.push(OrderBookLevel { price, quantity });
                        }
                    }
                }
            }

            if let Some(Value::Array(ask_array)) = obj.get("asks") {
                for ask in ask_array {
                    if let Value::Array(level) = ask {
                        if level.len() >= 2 {
                            let price = level[0].as_str()
                                .ok_or_else(|| BybitError::ParseError {
                                    kind: ParseErrorKind::InvalidFieldType("ask price".to_string()),
                                    source: None,
                                })?
                                .parse::<Decimal>()
                                .map_err(|e| BybitError::ParseError {
                                    kind: ParseErrorKind::NumberParseError,
                                    source: Some(Box::new(e)),
                                })?;
                            let quantity = level[1].as_str()
                                .ok_or_else(|| BybitError::ParseError {
                                    kind: ParseErrorKind::InvalidFieldType("ask quantity".to_string()),
                                    source: None,
                                })?
                                .parse::<Decimal>()
                                .map_err(|e| BybitError::ParseError {
                                    kind: ParseErrorKind::NumberParseError,
                                    source: Some(Box::new(e)),
                                })?;
                            asks.push(OrderBookLevel { price, quantity });
                        }
                    }
                }
            }

            Ok(Some(MarketData {
                exchange: "bybit".to_string(),
                symbol: symbol.to_string(),
                timestamp: DateTime::<Utc>::from_timestamp(timestamp / 1000, (timestamp % 1000) as u32 * 1_000_000)
                    .ok_or_else(|| BybitError::ParseError {
                        kind: ParseErrorKind::TimeParseError,
                        source: None,
                    })?,
                data_type: MarketDataType::OrderBook { bids, asks },
                quality: DataQuality::Real,
            }))
        } else {
            Ok(None)
        }
    }

    /// 处理成交数据
    fn process_trades(&self, symbol: &str, data: Option<Value>) -> Result<Option<MarketData>, BybitError> {
        if let Some(Value::Array(trades)) = data {
            let mut processed_trades = Vec::new();

            for trade in trades {
                if let Value::Object(obj) = trade {
                    let ts = obj.get("ts")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| BybitError::ParseError {
                            kind: ParseErrorKind::MissingField("timestamp".to_string()),
                            source: None,
                        })?;
                    let timestamp = ts.parse::<i64>()
                        .map_err(|e| BybitError::ParseError {
                            kind: ParseErrorKind::TimeParseError,
                            source: Some(Box::new(e)),
                        })?;

                    let price = obj.get("price")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| BybitError::ParseError {
                            kind: ParseErrorKind::MissingField("price".to_string()),
                            source: None,
                        })?
                        .parse::<Decimal>()
                        .map_err(|e| BybitError::ParseError {
                            kind: ParseErrorKind::NumberParseError,
                            source: Some(Box::new(e)),
                        })?;

                    let quantity = obj.get("size")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| BybitError::ParseError {
                            kind: ParseErrorKind::MissingField("size".to_string()),
                            source: None,
                        })?
                        .parse::<Decimal>()
                        .map_err(|e| BybitError::ParseError {
                            kind: ParseErrorKind::NumberParseError,
                            source: Some(Box::new(e)),
                        })?;

                    let side = obj.get("side")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| BybitError::ParseError {
                            kind: ParseErrorKind::MissingField("side".to_string()),
                            source: None,
                        })?;

                    let trade_time = DateTime::<Utc>::from_timestamp(timestamp / 1000, (timestamp % 1000) as u32 * 1_000_000)
                        .ok_or_else(|| BybitError::ParseError {
                            kind: ParseErrorKind::TimeParseError,
                            source: None,
                        })?;

                    processed_trades.push(Trade {
                        price,
                        quantity,
                        side: side.to_string(),
                        timestamp: trade_time,
                    });
                }
            }

            if !processed_trades.is_empty() {
                Ok(Some(MarketData {
                    exchange: "bybit".to_string(),
                    symbol: symbol.to_string(),
                    timestamp: processed_trades[0].timestamp,
                    data_type: MarketDataType::Trade(processed_trades),
                    quality: DataQuality::Real,
                }))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    /// 处理行情数据
    fn process_ticker(&self, symbol: &str, data: Option<Value>) -> Result<Option<MarketData>, BybitError> {
        if let Some(Value::Object(obj)) = data {
            let ts = obj.get("ts")
                .and_then(|v| v.as_str())
                .ok_or_else(|| BybitError::ParseError {
                    kind: ParseErrorKind::MissingField("timestamp".to_string()),
                    source: None,
                })?;
            let timestamp = ts.parse::<i64>()
                .map_err(|e| BybitError::ParseError {
                    kind: ParseErrorKind::TimeParseError,
                    source: Some(Box::new(e)),
                })?;

            let last_price = obj.get("lastPrice")
                .and_then(|v| v.as_str())
                .ok_or_else(|| BybitError::ParseError {
                    kind: ParseErrorKind::MissingField("last price".to_string()),
                    source: None,
                })?
                .parse::<Decimal>()
                .map_err(|e| BybitError::ParseError {
                    kind: ParseErrorKind::NumberParseError,
                    source: Some(Box::new(e)),
                })?;

            let volume = obj.get("volume24h")
                .and_then(|v| v.as_str())
                .ok_or_else(|| BybitError::ParseError {
                    kind: ParseErrorKind::MissingField("volume".to_string()),
                    source: None,
                })?
                .parse::<Decimal>()
                .map_err(|e| BybitError::ParseError {
                    kind: ParseErrorKind::NumberParseError,
                    source: Some(Box::new(e)),
                })?;

            Ok(Some(MarketData {
                exchange: "bybit".to_string(),
                symbol: symbol.to_string(),
                timestamp: DateTime::<Utc>::from_timestamp(timestamp / 1000, (timestamp % 1000) as u32 * 1_000_000)
                    .ok_or_else(|| BybitError::ParseError {
                        kind: ParseErrorKind::TimeParseError,
                        source: None,
                    })?,
                data_type: MarketDataType::Ticker(Ticker {
                    price: last_price,
                    volume,
                }),
                quality: DataQuality::Real,
            }))
        } else {
            Ok(None)
        }
    }

    /// 处理K线数据
    fn process_kline(&self, symbol: &str, data: Option<Value>) -> Result<Option<MarketData>, BybitError> {
        if let Some(Value::Array(klines)) = data {
            if let Some(Value::Object(kline)) = klines.first() {
                let start_time = kline.get("start")
                    .and_then(|v| v.as_i64())
                    .ok_or_else(|| BybitError::ParseError {
                        kind: ParseErrorKind::MissingField("start time".to_string()),
                        source: None,
                    })?;

                let open = kline.get("open")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| BybitError::ParseError {
                        kind: ParseErrorKind::MissingField("open price".to_string()),
                        source: None,
                    })?
                    .parse::<Decimal>()
                    .map_err(|e| BybitError::ParseError {
                        kind: ParseErrorKind::NumberParseError,
                        source: Some(Box::new(e)),
                    })?;

                let high = kline.get("high")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| BybitError::ParseError {
                        kind: ParseErrorKind::MissingField("high price".to_string()),
                        source: None,
                    })?
                    .parse::<Decimal>()
                    .map_err(|e| BybitError::ParseError {
                        kind: ParseErrorKind::NumberParseError,
                        source: Some(Box::new(e)),
                    })?;

                let low = kline.get("low")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| BybitError::ParseError {
                        kind: ParseErrorKind::MissingField("low price".to_string()),
                        source: None,
                    })?
                    .parse::<Decimal>()
                    .map_err(|e| BybitError::ParseError {
                        kind: ParseErrorKind::NumberParseError,
                        source: Some(Box::new(e)),
                    })?;

                let close = kline.get("close")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| BybitError::ParseError {
                        kind: ParseErrorKind::MissingField("close price".to_string()),
                        source: None,
                    })?
                    .parse::<Decimal>()
                    .map_err(|e| BybitError::ParseError {
                        kind: ParseErrorKind::NumberParseError,
                        source: Some(Box::new(e)),
                    })?;

                let volume = kline.get("volume")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| BybitError::ParseError {
                        kind: ParseErrorKind::MissingField("volume".to_string()),
                        source: None,
                    })?
                    .parse::<Decimal>()
                    .map_err(|e| BybitError::ParseError {
                        kind: ParseErrorKind::NumberParseError,
                        source: Some(Box::new(e)),
                    })?;

                Ok(Some(MarketData {
                    exchange: "bybit".to_string(),
                    symbol: symbol.to_string(),
                    timestamp: DateTime::<Utc>::from_timestamp(start_time / 1000, (start_time % 1000) as u32 * 1_000_000)
                        .ok_or_else(|| BybitError::ParseError {
                            kind: ParseErrorKind::TimeParseError,
                            source: None,
                        })?,
                    data_type: MarketDataType::Candlestick(Candlestick {
                        open,
                        high,
                        low,
                        close,
                        volume,
                    }),
                    quality: DataQuality::Real,
                }))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_process_orderbook() {
        let processor = BybitProcessor::new();
        let data = json!({
            "ts": "1684856606001",
            "bids": [
                ["27789.0", "0.407312"],
                ["27788.9", "0.018674"]
            ],
            "asks": [
                ["27789.1", "0.405632"],
                ["27789.2", "0.021547"]
            ]
        });

        let result = processor.process_orderbook("BTCUSDT", Some(data));
        assert!(result.is_ok(), "Failed to process orderbook: {:?}", result);
        
        if let Ok(Some(market_data)) = result {
            match market_data.data_type {
                MarketDataType::OrderBook { bids, asks } => {
                    assert_eq!(bids.len(), 2);
                    assert_eq!(asks.len(), 2);
                },
                _ => panic!("Unexpected market data type"),
            }
        }
    }

    #[test]
    fn test_process_trades() {
        let processor = BybitProcessor::new();
        let data = json!([{
            "ts": "1684856606001",
            "price": "27789.0",
            "size": "0.407312",
            "side": "Buy"
        }]);

        let result = processor.process_trades("BTCUSDT", Some(data));
        assert!(result.is_ok(), "Failed to process trades: {:?}", result);
        
        if let Ok(Some(market_data)) = result {
            match market_data.data_type {
                MarketDataType::Trade(trades) => {
                    assert_eq!(trades.len(), 1);
                    assert_eq!(trades[0].side, "Buy");
                },
                _ => panic!("Unexpected market data type"),
            }
        }
    }

    #[test]
    fn test_process_ticker() {
        let processor = BybitProcessor::new();
        let data = json!({
            "ts": "1684856606001",
            "lastPrice": "27789.0",
            "volume24h": "1000.0"
        });

        let result = processor.process_ticker("BTCUSDT", Some(data));
        assert!(result.is_ok(), "Failed to process ticker: {:?}", result);
        
        if let Ok(Some(market_data)) = result {
            match market_data.data_type {
                MarketDataType::Ticker(ticker) => {
                    assert_eq!(ticker.price.to_string(), "27789.0");
                    assert_eq!(ticker.volume.to_string(), "1000.0");
                },
                _ => panic!("Unexpected market data type"),
            }
        }
    }

    #[test]
    fn test_process_kline() {
        let processor = BybitProcessor::new();
        let data = json!([{
            "start": 1684856606001,
            "open": "27789.0",
            "high": "27790.0",
            "low": "27788.0",
            "close": "27789.5",
            "volume": "100.0"
        }]);

        let result = processor.process_kline("BTCUSDT", Some(data));
        assert!(result.is_ok(), "Failed to process kline: {:?}", result);
        
        if let Ok(Some(market_data)) = result {
            match market_data.data_type {
                MarketDataType::Candlestick(candle) => {
                    assert_eq!(candle.open.to_string(), "27789.0");
                    assert_eq!(candle.high.to_string(), "27790.0");
                    assert_eq!(candle.low.to_string(), "27788.0");
                    assert_eq!(candle.close.to_string(), "27789.5");
                    assert_eq!(candle.volume.to_string(), "100.0");
                },
                _ => panic!("Unexpected market data type"),
            }
        }
    }

    #[test]
    fn test_error_handling() {
        let processor = BybitProcessor::new();
        
        // 测试缺失字段错误
        let data = json!({
            "ts": "1684856606001",
            // 缺少 lastPrice 字段
            "volume24h": "1000.0"
        });
        let result = processor.process_ticker("BTCUSDT", Some(data));
        assert!(matches!(result,
            Err(BybitError::ParseError {
                kind: ParseErrorKind::MissingField(_),
                ..
            })
        ));
        
        // 测试数值解析错误
        let data = json!({
            "ts": "1684856606001",
            "lastPrice": "invalid",
            "volume24h": "1000.0"
        });
        let result = processor.process_ticker("BTCUSDT", Some(data));
        assert!(matches!(result,
            Err(BybitError::ParseError {
                kind: ParseErrorKind::NumberParseError,
                ..
            })
        ));
        
        // 测试时间戳解析错误
        let data = json!({
            "ts": "invalid",
            "lastPrice": "27789.0",
            "volume24h": "1000.0"
        });
        let result = processor.process_ticker("BTCUSDT", Some(data));
        assert!(matches!(result,
            Err(BybitError::ParseError {
                kind: ParseErrorKind::TimeParseError,
                ..
            })
        ));
    }
} 