use std::collections::HashMap;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde_json::Value;
use tracing::{debug, error, warn};

use common::{MarketData, DataQuality, MarketDataType, Trade, Kline, OrderBook, PriceLevel, Ticker};
use crate::error::OkxError;
use crate::models;

/// OKX数据处理器
pub struct OkxProcessor {
    symbol_info: HashMap<String, Value>,
}

impl OkxProcessor {
    pub fn new() -> Self {
        Self {
            symbol_info: HashMap::new(),
        }
    }

    /// 更新交易对信息
    pub fn update_symbol_info(&mut self, symbol: String, info: Value) {
        self.symbol_info.insert(symbol, info);
    }

    /// 处理WebSocket消息
    pub async fn process_ws_message(&self, message: &str) -> Result<Option<MarketData>, OkxError> {
        let value: Value = serde_json::from_str(message)
            .map_err(|e| OkxError::ParseError(format!("Failed to parse WebSocket message: {}", e)))?;

        if let Some(event) = value.get("event") {
            match event.as_str() {
                Some("subscribe") => Ok(None),
                Some("unsubscribe") => Ok(None),
                Some("error") => {
                    error!("WebSocket error: {:?}", value);
                    Err(OkxError::WebSocketError(format!("WebSocket error: {:?}", value)))
                }
                _ => Ok(None),
            }
        } else if let Some(arg) = value.get("arg") {
            let channel = arg["channel"].as_str().unwrap_or("");
            match channel {
                "tickers" => self.process_ticker(&value),
                "trades" => self.process_trade(&value),
                "books" => self.process_order_book(&value),
                "candle1m" => self.process_kline(&value),
                _ => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    /// 处理Ticker数据
    fn process_ticker(&self, data: &Value) -> Result<Option<MarketData>, OkxError> {
        if let Some(ticker_data) = data["data"].as_array().and_then(|arr| arr.first()) {
            let ticker: models::Ticker = serde_json::from_value(ticker_data.clone())
                .map_err(|e| OkxError::ParseError(format!("Failed to parse ticker: {}", e)))?;

            let market_data = MarketData::Ticker(Ticker {
                symbol: ticker.inst_id.clone(),
                price: Decimal::from_str_exact(&ticker.last)
                    .map_err(|e| OkxError::ParseError(format!("Invalid price: {}", e)))?,
                volume: Decimal::from_str_exact(&ticker.vol_24h)
                    .map_err(|e| OkxError::ParseError(format!("Invalid volume: {}", e)))?,
                timestamp: DateTime::parse_from_rfc3339(&ticker.ts)
                    .map_err(|e| OkxError::ParseError(format!("Invalid timestamp: {}", e)))?
                    .with_timezone(&Utc),
                quality: DataQuality::Real,
            });

            Ok(Some(market_data))
        } else {
            Ok(None)
        }
    }

    /// 处理Trade数据
    fn process_trade(&self, data: &Value) -> Result<Option<MarketData>, OkxError> {
        if let Some(trade_data) = data["data"].as_array().and_then(|arr| arr.first()) {
            let trade: models::Trade = serde_json::from_value(trade_data.clone())
                .map_err(|e| OkxError::ParseError(format!("Failed to parse trade: {}", e)))?;

            let market_data = MarketData::Trade(Trade {
                symbol: trade.inst_id.clone(),
                id: trade.trade_id.clone(),
                price: Decimal::from_str_exact(&trade.px)
                    .map_err(|e| OkxError::ParseError(format!("Invalid price: {}", e)))?,
                quantity: Decimal::from_str_exact(&trade.sz)
                    .map_err(|e| OkxError::ParseError(format!("Invalid quantity: {}", e)))?,
                timestamp: DateTime::parse_from_rfc3339(&trade.ts)
                    .map_err(|e| OkxError::ParseError(format!("Invalid timestamp: {}", e)))?
                    .with_timezone(&Utc),
                is_buyer_maker: trade.side == "buy",
                quality: DataQuality::Real,
            });

            Ok(Some(market_data))
        } else {
            Ok(None)
        }
    }

    /// 处理OrderBook数据
    fn process_order_book(&self, data: &Value) -> Result<Option<MarketData>, OkxError> {
        if let Some(book_data) = data["data"].as_array().and_then(|arr| arr.first()) {
            let book: models::OrderBook = serde_json::from_value(book_data.clone())
                .map_err(|e| OkxError::ParseError(format!("Failed to parse order book: {}", e)))?;

            let parse_price_level = |level: &[String; 4]| -> Result<PriceLevel, OkxError> {
                Ok(PriceLevel {
                    price: Decimal::from_str_exact(&level[0])
                        .map_err(|e| OkxError::ParseError(format!("Invalid price: {}", e)))?,
                    quantity: Decimal::from_str_exact(&level[1])
                        .map_err(|e| OkxError::ParseError(format!("Invalid quantity: {}", e)))?,
                })
            };

            let asks = book.asks.iter()
                .map(parse_price_level)
                .collect::<Result<Vec<_>, _>>()?;

            let bids = book.bids.iter()
                .map(parse_price_level)
                .collect::<Result<Vec<_>, _>>()?;

            let market_data = MarketData::OrderBook(OrderBook {
                symbol: book_data["instId"].as_str()
                    .ok_or_else(|| OkxError::ParseError("Missing instId".to_string()))?
                    .to_string(),
                asks,
                bids,
                timestamp: DateTime::parse_from_rfc3339(&book.ts)
                    .map_err(|e| OkxError::ParseError(format!("Invalid timestamp: {}", e)))?
                    .with_timezone(&Utc),
                quality: DataQuality::Real,
            });

            Ok(Some(market_data))
        } else {
            Ok(None)
        }
    }

    /// 处理K线数据
    fn process_kline(&self, data: &Value) -> Result<Option<MarketData>, OkxError> {
        if let Some(kline_data) = data["data"].as_array().and_then(|arr| arr.first()) {
            let kline: models::Kline = serde_json::from_value(kline_data.clone())
                .map_err(|e| OkxError::ParseError(format!("Failed to parse kline: {}", e)))?;

            let market_data = MarketData::Kline(Kline {
                symbol: kline_data["instId"].as_str()
                    .ok_or_else(|| OkxError::ParseError("Missing instId".to_string()))?
                    .to_string(),
                timestamp: DateTime::parse_from_rfc3339(&kline.ts)
                    .map_err(|e| OkxError::ParseError(format!("Invalid timestamp: {}", e)))?
                    .with_timezone(&Utc),
                open: Decimal::from_str_exact(&kline.o)
                    .map_err(|e| OkxError::ParseError(format!("Invalid open price: {}", e)))?,
                high: Decimal::from_str_exact(&kline.h)
                    .map_err(|e| OkxError::ParseError(format!("Invalid high price: {}", e)))?,
                low: Decimal::from_str_exact(&kline.l)
                    .map_err(|e| OkxError::ParseError(format!("Invalid low price: {}", e)))?,
                close: Decimal::from_str_exact(&kline.c)
                    .map_err(|e| OkxError::ParseError(format!("Invalid close price: {}", e)))?,
                volume: Decimal::from_str_exact(&kline.vol)
                    .map_err(|e| OkxError::ParseError(format!("Invalid volume: {}", e)))?,
                quality: DataQuality::Real,
            });

            Ok(Some(market_data))
        } else {
            Ok(None)
        }
    }
} 