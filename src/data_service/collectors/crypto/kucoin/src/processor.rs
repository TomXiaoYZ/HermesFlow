use std::collections::HashMap;
use serde_json::Value;
use crate::models::*;
use crate::Result;

/// 数据处理器
pub struct KucoinDataProcessor {
    /// 缓存的交易对信息
    symbols: HashMap<String, Symbol>,
}

impl KucoinDataProcessor {
    /// 创建新的数据处理器
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
        }
    }

    /// 更新交易对信息
    pub fn update_symbols(&mut self, symbols: Vec<Symbol>) {
        self.symbols.clear();
        for symbol in symbols {
            self.symbols.insert(symbol.symbol.clone(), symbol);
        }
    }

    /// 处理WebSocket消息
    pub fn process_message(&self, message: ResponseMessage) -> Result<Option<ProcessedData>> {
        // 解析频道类型
        let channel_parts: Vec<&str> = message.channel.split(':').collect();
        if channel_parts.len() != 2 {
            return Ok(None);
        }

        let channel_type = match channel_parts[0] {
            "/market/ticker" => "ticker",
            "/market/level2" => "depth",
            "/market/match" => "trade",
            _ => return Ok(None),
        };

        let symbol = channel_parts[1];

        match channel_type {
            "ticker" => self.process_ticker(symbol.to_string(), message.data),
            "depth" => self.process_orderbook(symbol.to_string(), message.data),
            "trade" => self.process_trade(symbol.to_string(), message.data),
            _ => Ok(None),
        }
    }

    /// 处理Ticker数据
    fn process_ticker(&self, symbol: String, data: Value) -> Result<Option<ProcessedData>> {
        let ticker = Ticker {
            exchange: "kucoin".to_string(),
            symbol: symbol.clone(),
            last_price: data["price"].as_str().unwrap_or("0").parse().unwrap_or(0.0),
            high_24h: data["high"].as_str().unwrap_or("0").parse().unwrap_or(0.0),
            low_24h: data["low"].as_str().unwrap_or("0").parse().unwrap_or(0.0),
            volume_24h: data["vol"].as_str().unwrap_or("0").parse().unwrap_or(0.0),
            amount_24h: data["volValue"].as_str().unwrap_or("0").parse().unwrap_or(0.0),
        };

        Ok(Some(ProcessedData::Ticker(ticker)))
    }

    /// 处理深度数据
    fn process_orderbook(&self, symbol: String, data: Value) -> Result<Option<ProcessedData>> {
        let timestamp = data["timestamp"].as_u64().unwrap_or(0);
        
        let bids: Vec<(f64, f64)> = data["bids"].as_array()
            .unwrap_or(&vec![])
            .chunks(2)
            .map(|chunk| {
                let price = chunk[0].as_str().unwrap_or("0").parse().unwrap_or(0.0);
                let size = chunk[1].as_str().unwrap_or("0").parse().unwrap_or(0.0);
                (price, size)
            })
            .collect();

        let asks: Vec<(f64, f64)> = data["asks"].as_array()
            .unwrap_or(&vec![])
            .chunks(2)
            .map(|chunk| {
                let price = chunk[0].as_str().unwrap_or("0").parse().unwrap_or(0.0);
                let size = chunk[1].as_str().unwrap_or("0").parse().unwrap_or(0.0);
                (price, size)
            })
            .collect();

        let orderbook = Orderbook {
            exchange: "kucoin".to_string(),
            symbol,
            timestamp,
            bids,
            asks,
        };

        Ok(Some(ProcessedData::Orderbook(orderbook)))
    }

    /// 处理成交数据
    fn process_trade(&self, symbol: String, data: Value) -> Result<Option<ProcessedData>> {
        let trade = Trade {
            exchange: "kucoin".to_string(),
            symbol,
            trade_id: data["tradeId"].as_str().unwrap_or("").to_string(),
            price: data["price"].as_str().unwrap_or("0").parse().unwrap_or(0.0),
            size: data["size"].as_str().unwrap_or("0").parse().unwrap_or(0.0),
            timestamp: data["time"].as_u64().unwrap_or(0),
            side: data["side"].as_str().unwrap_or("").to_string(),
        };

        Ok(Some(ProcessedData::Trade(trade)))
    }
}

/// 处理后的数据
#[derive(Debug, Clone)]
pub enum ProcessedData {
    /// Ticker数据
    Ticker(Ticker),
    /// 深度数据
    Orderbook(Orderbook),
    /// 成交数据
    Trade(Trade),
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_process_ticker() {
        let processor = KucoinDataProcessor::new();
        let message = ResponseMessage {
            channel: "/market/ticker:BTC-USDT".to_string(),
            data: json!({
                "price": "50000.5",
                "high": "51000.0",
                "low": "49000.0",
                "vol": "100.5",
                "volValue": "5025025.25"
            }),
        };

        let result = processor.process_message(message).unwrap().unwrap();
        match result {
            ProcessedData::Ticker(ticker) => {
                assert_eq!(ticker.exchange, "kucoin");
                assert_eq!(ticker.symbol, "BTC-USDT");
                assert_eq!(ticker.last_price, 50000.5);
                assert_eq!(ticker.high_24h, 51000.0);
                assert_eq!(ticker.low_24h, 49000.0);
                assert_eq!(ticker.volume_24h, 100.5);
                assert_eq!(ticker.amount_24h, 5025025.25);
            }
            _ => panic!("Expected Ticker data"),
        }
    }

    #[test]
    fn test_process_orderbook() {
        let processor = KucoinDataProcessor::new();
        let message = ResponseMessage {
            channel: "/market/level2:BTC-USDT".to_string(),
            data: json!({
                "timestamp": 1234567890,
                "bids": ["50000.5", "1.5", "49999.5", "2.0"],
                "asks": ["50001.0", "1.0", "50002.0", "2.5"]
            }),
        };

        let result = processor.process_message(message).unwrap().unwrap();
        match result {
            ProcessedData::Orderbook(orderbook) => {
                assert_eq!(orderbook.exchange, "kucoin");
                assert_eq!(orderbook.symbol, "BTC-USDT");
                assert_eq!(orderbook.timestamp, 1234567890);
                assert_eq!(orderbook.bids, vec![(50000.5, 1.5), (49999.5, 2.0)]);
                assert_eq!(orderbook.asks, vec![(50001.0, 1.0), (50002.0, 2.5)]);
            }
            _ => panic!("Expected Orderbook data"),
        }
    }

    #[test]
    fn test_process_trade() {
        let processor = KucoinDataProcessor::new();
        let message = ResponseMessage {
            channel: "/market/match:BTC-USDT".to_string(),
            data: json!({
                "tradeId": "123456",
                "price": "50000.5",
                "size": "1.5",
                "time": 1234567890,
                "side": "buy"
            }),
        };

        let result = processor.process_message(message).unwrap().unwrap();
        match result {
            ProcessedData::Trade(trade) => {
                assert_eq!(trade.exchange, "kucoin");
                assert_eq!(trade.symbol, "BTC-USDT");
                assert_eq!(trade.trade_id, "123456");
                assert_eq!(trade.price, 50000.5);
                assert_eq!(trade.size, 1.5);
                assert_eq!(trade.timestamp, 1234567890);
                assert_eq!(trade.side, "buy");
            }
            _ => panic!("Expected Trade data"),
        }
    }

    #[test]
    fn test_invalid_channel() {
        let processor = KucoinDataProcessor::new();
        let message = ResponseMessage {
            channel: "invalid_channel".to_string(),
            data: json!({}),
        };

        let result = processor.process_message(message).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_update_symbols() {
        let mut processor = KucoinDataProcessor::new();
        let symbols = vec![
            Symbol {
                exchange: "kucoin".to_string(),
                symbol: "BTC-USDT".to_string(),
                base_currency: "BTC".to_string(),
                quote_currency: "USDT".to_string(),
                price_precision: 1,
                size_precision: 8,
                min_size: 0.0001,
                min_funds: 5.0,
            }
        ];

        processor.update_symbols(symbols.clone());
        assert_eq!(processor.symbols.len(), 1);
        assert_eq!(
            processor.symbols.get("BTC-USDT").unwrap().base_currency,
            "BTC"
        );
    }
}
