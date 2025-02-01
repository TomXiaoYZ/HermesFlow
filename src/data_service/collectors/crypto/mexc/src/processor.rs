use std::collections::HashMap;
use serde_json::Value;
use crate::models::*;
use crate::Result;

/// 数据处理器
pub struct MexcDataProcessor {
    /// 缓存的交易对信息
    symbols: HashMap<String, Symbol>,
}

impl MexcDataProcessor {
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
        let channel_parts: Vec<&str> = message.channel.split('.').collect();
        if channel_parts.len() != 2 {
            return Ok(None);
        }

        match channel_parts[0] {
            "ticker" => self.process_ticker(message.data),
            "depth" => self.process_orderbook(message.data),
            "trade" => self.process_trade(message.data),
            _ => Ok(None),
        }
    }

    /// 处理Ticker数据
    fn process_ticker(&self, data: Value) -> Result<Option<ProcessedData>> {
        let ticker: Ticker = serde_json::from_value(data)
            .map_err(crate::MexcError::JsonError)?;

        Ok(Some(ProcessedData::Ticker(ticker)))
    }

    /// 处理深度数据
    fn process_orderbook(&self, data: Value) -> Result<Option<ProcessedData>> {
        let orderbook: Orderbook = serde_json::from_value(data)
            .map_err(crate::MexcError::JsonError)?;

        Ok(Some(ProcessedData::Orderbook(orderbook)))
    }

    /// 处理成交数据
    fn process_trade(&self, data: Value) -> Result<Option<ProcessedData>> {
        let trades: Vec<Trade> = serde_json::from_value(data)
            .map_err(crate::MexcError::JsonError)?;

        Ok(Some(ProcessedData::Trades(trades)))
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
    Trades(Vec<Trade>),
} 