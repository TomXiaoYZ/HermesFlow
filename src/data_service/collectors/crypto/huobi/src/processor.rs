use std::collections::HashMap;
use chrono::{TimeZone, Utc};
use serde_json::Value;
use tracing::warn;

use common::{
    MarketData, DataQuality, MarketDataType,
    OrderBookLevel, Trade as CommonTrade, Candlestick,
    Side, Exchange,
};

use crate::error::HuobiError;
use crate::types::{
    KlineResponse, DepthResponse, TradeResponse,
    TickerResponse, SubscriptionResponse,
};

/// 数据处理器
pub struct HuobiProcessor {
    symbol_info: HashMap<String, Value>,
}

impl HuobiProcessor {
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
    pub async fn process_ws_message(&self, message: &str) -> Result<Option<MarketData>, HuobiError> {
        let value: Value = serde_json::from_str(message)?;

        // 处理 Ping/Pong 消息
        if value.get("ping").is_some() || value.get("pong").is_some() {
            return Ok(None);
        }

        // 处理订阅响应
        if value.get("subbed").is_some() {
            let response: SubscriptionResponse = serde_json::from_value(value.clone())?;
            if response.status != "ok" {
                warn!("订阅失败: {:?}", response);
            }
            return Ok(None);
        }

        // 提取频道和时间戳
        let ch = value["ch"].as_str().ok_or_else(|| {
            HuobiError::ParseError("Missing channel information".to_string())
        })?;
        let ts = value["ts"].as_i64().ok_or_else(|| {
            HuobiError::ParseError("Missing timestamp".to_string())
        })?;

        // 根据频道类型处理不同的数据
        if ch.contains(".kline.") {
            self.process_kline(ch, ts, value.clone()).map(Some)
        } else if ch.contains(".depth.") {
            self.process_depth(ch, ts, value.clone()).map(Some)
        } else if ch.contains(".trade.") {
            self.process_trade(ch, ts, value.clone()).map(Some)
        } else if ch.contains(".detail") {
            self.process_ticker(ch, ts, value.clone()).map(Some)
        } else {
            warn!("未知的频道类型: {}", ch);
            Ok(None)
        }
    }

    /// 处理K线数据
    fn process_kline(&self, ch: &str, ts: i64, value: Value) -> Result<MarketData, HuobiError> {
        let response: KlineResponse = serde_json::from_value(value)?;
        let symbol = extract_symbol(ch)?;

        let candlestick = Candlestick {
            timestamp: Utc.timestamp_millis_opt(response.tick.id * 1000).unwrap(),
            open: response.tick.open,
            high: response.tick.high,
            low: response.tick.low,
            close: response.tick.close,
            volume: response.tick.amount,
            turnover: response.tick.vol,
            trade_count: response.tick.count as u64,
        };

        Ok(MarketData {
            exchange: Exchange::Huobi,
            symbol: symbol.to_string(),
            timestamp: Utc.timestamp_millis_opt(ts).unwrap(),
            data_type: MarketDataType::Candlestick(candlestick),
            quality: DataQuality::Real,
        })
    }

    /// 处理深度数据
    fn process_depth(&self, ch: &str, ts: i64, value: Value) -> Result<MarketData, HuobiError> {
        let response: DepthResponse = serde_json::from_value(value)?;
        let symbol = extract_symbol(ch)?;

        let bids: Vec<OrderBookLevel> = response.tick.bids
            .into_iter()
            .map(|bid| OrderBookLevel {
                price: bid[0],
                amount: bid[1],
            })
            .collect();

        let asks: Vec<OrderBookLevel> = response.tick.asks
            .into_iter()
            .map(|ask| OrderBookLevel {
                price: ask[0],
                amount: ask[1],
            })
            .collect();

        Ok(MarketData {
            exchange: Exchange::Huobi,
            symbol: symbol.to_string(),
            timestamp: Utc.timestamp_millis_opt(ts).unwrap(),
            data_type: MarketDataType::OrderBook { bids, asks },
            quality: DataQuality::Real,
        })
    }

    /// 处理成交数据
    fn process_trade(&self, ch: &str, ts: i64, value: Value) -> Result<MarketData, HuobiError> {
        let response: TradeResponse = serde_json::from_value(value)?;
        let symbol = extract_symbol(ch)?;

        let trades: Vec<CommonTrade> = response.tick.data
            .into_iter()
            .map(|trade| CommonTrade {
                id: trade.id.to_string(),
                timestamp: Utc.timestamp_millis_opt(trade.ts).unwrap(),
                price: trade.price,
                amount: trade.amount,
                side: match trade.direction.as_str() {
                    "buy" => Side::Buy,
                    "sell" => Side::Sell,
                    _ => Side::Unknown,
                },
            })
            .collect();

        Ok(MarketData {
            exchange: Exchange::Huobi,
            symbol: symbol.to_string(),
            timestamp: Utc.timestamp_millis_opt(ts).unwrap(),
            data_type: MarketDataType::Trade(trades),
            quality: DataQuality::Real,
        })
    }

    /// 处理行情数据
    fn process_ticker(&self, ch: &str, ts: i64, value: Value) -> Result<MarketData, HuobiError> {
        let response: TickerResponse = serde_json::from_value(value)?;
        let symbol = extract_symbol(ch)?;

        let ticker = response.tick;
        let bids = vec![OrderBookLevel {
            price: ticker.bid,
            amount: ticker.bid_size,
        }];
        let asks = vec![OrderBookLevel {
            price: ticker.ask,
            amount: ticker.ask_size,
        }];

        Ok(MarketData {
            exchange: Exchange::Huobi,
            symbol: symbol.to_string(),
            timestamp: Utc.timestamp_millis_opt(ts).unwrap(),
            data_type: MarketDataType::OrderBook { bids, asks },
            quality: DataQuality::Real,
        })
    }
}

/// 从频道名称中提取交易对
fn extract_symbol(ch: &str) -> Result<&str, HuobiError> {
    ch.split('.')
        .nth(1)
        .ok_or_else(|| HuobiError::ParseError("Invalid channel format".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_process_kline() {
        let processor = HuobiProcessor::new();
        let message = json!({
            "ch": "market.btcusdt.kline.1min",
            "ts": 1630000000000_i64,
            "tick": {
                "id": 1630000000,
                "amount": "1.23",
                "count": 100,
                "open": "40000.0",
                "close": "41000.0",
                "low": "39800.0",
                "high": "41200.0",
                "vol": "49200.0"
            }
        });

        let result = processor.process_ws_message(&message.to_string()).await.unwrap();
        assert!(result.is_some());
        if let Some(MarketData { data_type, .. }) = result {
            assert!(matches!(data_type, MarketDataType::Candlestick(_)));
        }
    }

    #[tokio::test]
    async fn test_process_depth() {
        let processor = HuobiProcessor::new();
        let message = json!({
            "ch": "market.btcusdt.depth.step0",
            "ts": 1630000000000_i64,
            "tick": {
                "ts": 1630000000000_i64,
                "version": 100,
                "bids": [["40000.0", "1.0"]],
                "asks": [["40100.0", "1.0"]]
            }
        });

        let result = processor.process_ws_message(&message.to_string()).await.unwrap();
        assert!(result.is_some());
        if let Some(MarketData { data_type, .. }) = result {
            assert!(matches!(data_type, MarketDataType::OrderBook { .. }));
        }
    }

    #[tokio::test]
    async fn test_process_trade() {
        let processor = HuobiProcessor::new();
        let message = json!({
            "ch": "market.btcusdt.trade.detail",
            "ts": 1630000000000_i64,
            "tick": {
                "id": 1630000000,
                "ts": 1630000000000_i64,
                "data": [{
                    "id": 1,
                    "ts": 1630000000000_i64,
                    "amount": "0.1",
                    "price": "40000.0",
                    "direction": "buy"
                }]
            }
        });

        let result = processor.process_ws_message(&message.to_string()).await.unwrap();
        assert!(result.is_some());
        if let Some(MarketData { data_type, .. }) = result {
            assert!(matches!(data_type, MarketDataType::Trade(_)));
        }
    }
} 