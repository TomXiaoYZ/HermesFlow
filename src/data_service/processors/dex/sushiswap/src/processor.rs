use std::collections::HashMap;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde_json::Value;
use tracing::{debug, error, warn};

use common::{
    MarketData, DataQuality, MarketDataType, Trade, OrderBook, OrderBookLevel,
    DataProcessor, Exchange, Side, LiquidityEvent, Position,
};
use crate::error::SushiSwapError;
use crate::models::{Pool, Token, PoolData};
use crate::metrics;

/// SushiSwap数据处理器
pub struct SushiSwapProcessor {
    pools: HashMap<String, Pool>,
    tokens: HashMap<String, Token>,
    config: HashMap<String, String>,
}

impl SushiSwapProcessor {
    pub fn new(config: HashMap<String, String>) -> Self {
        Self {
            pools: HashMap::new(),
            tokens: HashMap::new(),
            config,
        }
    }

    /// 更新池子信息
    pub fn update_pool(&mut self, pool_id: String, pool: Pool) {
        self.pools.insert(pool_id, pool);
    }

    /// 更新代币信息
    pub fn update_token(&mut self, token_id: String, token: Token) {
        self.tokens.insert(token_id, token);
    }

    /// 处理Graph API数据
    pub async fn process_graph_data(&self, data: &Value) -> Result<Option<MarketData>, SushiSwapError> {
        if let Some(pool_data) = data.get("pair") {
            let pool_id = pool_data["id"].as_str()
                .ok_or_else(|| SushiSwapError::ParseError("Missing pool ID".to_string()))?;
            
            let token0_address = pool_data["token0"]["id"].as_str()
                .ok_or_else(|| SushiSwapError::ParseError("Missing token0 address".to_string()))?;
            let token1_address = pool_data["token1"]["id"].as_str()
                .ok_or_else(|| SushiSwapError::ParseError("Missing token1 address".to_string()))?;
            
            let symbol = format!("{}-{}", token0_address, token1_address).to_uppercase();

            let market_data = MarketData {
                exchange: Exchange::SushiSwap,
                symbol,
                timestamp: Utc::now(),
                data_type: MarketDataType::LiquidityEvent(LiquidityEvent {
                    pool_id: pool_id.to_string(),
                    token0_address: token0_address.to_string(),
                    token1_address: token1_address.to_string(),
                    token0_amount: Decimal::from_str_exact(pool_data["reserve0"].as_str()
                        .ok_or_else(|| SushiSwapError::ParseError("Missing reserve0".to_string()))?)
                        .map_err(|e| SushiSwapError::ParseError(format!("Invalid reserve0: {}", e)))?,
                    token1_amount: Decimal::from_str_exact(pool_data["reserve1"].as_str()
                        .ok_or_else(|| SushiSwapError::ParseError("Missing reserve1".to_string()))?)
                        .map_err(|e| SushiSwapError::ParseError(format!("Invalid reserve1: {}", e)))?,
                    liquidity: Decimal::from_str_exact(pool_data["totalSupply"].as_str()
                        .ok_or_else(|| SushiSwapError::ParseError("Missing total supply".to_string()))?)
                        .map_err(|e| SushiSwapError::ParseError(format!("Invalid total supply: {}", e)))?,
                    sqrt_price_x96: Decimal::ZERO, // SushiSwap V1不使用此字段
                    tick: 0, // SushiSwap V1不使用此字段
                    fee_tier: 30, // SushiSwap V1固定0.3%手续费
                }),
                quality: DataQuality::Real,
            };

            Ok(Some(market_data))
        } else {
            Ok(None)
        }
    }

    /// 处理合约事件数据
    pub async fn process_event_data(&self, event: &Value) -> Result<Option<MarketData>, SushiSwapError> {
        if let Some(event_name) = event["event"].as_str() {
            match event_name {
                "Swap" => self.process_swap_event(event),
                "Mint" => self.process_mint_event(event),
                "Burn" => self.process_burn_event(event),
                _ => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    /// 处理Swap事件
    fn process_swap_event(&self, event: &Value) -> Result<Option<MarketData>, SushiSwapError> {
        let data = event["data"].as_object()
            .ok_or_else(|| SushiSwapError::ParseError("Missing event data".to_string()))?;

        let pool_address = event["address"].as_str()
            .ok_or_else(|| SushiSwapError::ParseError("Missing pool address".to_string()))?;

        let market_data = MarketData {
            exchange: Exchange::SushiSwap,
            symbol: pool_address.to_string(),
            timestamp: DateTime::from_timestamp_millis(
                event["timestamp"].as_i64()
                    .ok_or_else(|| SushiSwapError::ParseError("Missing timestamp".to_string()))?)
                .ok_or_else(|| SushiSwapError::ParseError("Invalid timestamp".to_string()))?
                .with_timezone(&Utc),
            data_type: MarketDataType::Trade(vec![Trade {
                id: event["transactionHash"].as_str()
                    .ok_or_else(|| SushiSwapError::ParseError("Missing transaction hash".to_string()))?
                    .to_string(),
                price: Decimal::from_str_exact(data["price"].as_str()
                    .ok_or_else(|| SushiSwapError::ParseError("Missing price".to_string()))?)
                    .map_err(|e| SushiSwapError::ParseError(format!("Invalid price: {}", e)))?,
                quantity: Decimal::from_str_exact(data["amount"].as_str()
                    .ok_or_else(|| SushiSwapError::ParseError("Missing amount".to_string()))?)
                    .map_err(|e| SushiSwapError::ParseError(format!("Invalid amount: {}", e)))?,
                timestamp: Utc::now(),
                side: if data["amount0Out"].as_str().unwrap_or("0") != "0" { Side::Buy } else { Side::Sell },
            }]),
            quality: DataQuality::Real,
        };

        Ok(Some(market_data))
    }

    /// 处理Mint事件
    fn process_mint_event(&self, event: &Value) -> Result<Option<MarketData>, SushiSwapError> {
        let data = event["data"].as_object()
            .ok_or_else(|| SushiSwapError::ParseError("Missing event data".to_string()))?;

        let pool_address = event["address"].as_str()
            .ok_or_else(|| SushiSwapError::ParseError("Missing pool address".to_string()))?;

        let market_data = MarketData {
            exchange: Exchange::SushiSwap,
            symbol: pool_address.to_string(),
            timestamp: DateTime::from_timestamp_millis(
                event["timestamp"].as_i64()
                    .ok_or_else(|| SushiSwapError::ParseError("Missing timestamp".to_string()))?)
                .ok_or_else(|| SushiSwapError::ParseError("Invalid timestamp".to_string()))?
                .with_timezone(&Utc),
            data_type: MarketDataType::Position(Position {
                id: event["transactionHash"].as_str()
                    .ok_or_else(|| SushiSwapError::ParseError("Missing transaction hash".to_string()))?
                    .to_string(),
                token0_amount: Decimal::from_str_exact(data["amount0"].as_str()
                    .ok_or_else(|| SushiSwapError::ParseError("Missing amount0".to_string()))?)
                    .map_err(|e| SushiSwapError::ParseError(format!("Invalid amount0: {}", e)))?,
                token1_amount: Decimal::from_str_exact(data["amount1"].as_str()
                    .ok_or_else(|| SushiSwapError::ParseError("Missing amount1".to_string()))?)
                    .map_err(|e| SushiSwapError::ParseError(format!("Invalid amount1: {}", e)))?,
                liquidity: Decimal::from_str_exact(data["liquidity"].as_str()
                    .ok_or_else(|| SushiSwapError::ParseError("Missing liquidity".to_string()))?)
                    .map_err(|e| SushiSwapError::ParseError(format!("Invalid liquidity: {}", e)))?,
                tick_lower: 0, // SushiSwap V1不使用此字段
                tick_upper: 0, // SushiSwap V1不使用此字段
                position_type: "mint".to_string(),
            }),
            quality: DataQuality::Real,
        };

        Ok(Some(market_data))
    }

    /// 处理Burn事件
    fn process_burn_event(&self, event: &Value) -> Result<Option<MarketData>, SushiSwapError> {
        let data = event["data"].as_object()
            .ok_or_else(|| SushiSwapError::ParseError("Missing event data".to_string()))?;

        let pool_address = event["address"].as_str()
            .ok_or_else(|| SushiSwapError::ParseError("Missing pool address".to_string()))?;

        let market_data = MarketData {
            exchange: Exchange::SushiSwap,
            symbol: pool_address.to_string(),
            timestamp: DateTime::from_timestamp_millis(
                event["timestamp"].as_i64()
                    .ok_or_else(|| SushiSwapError::ParseError("Missing timestamp".to_string()))?)
                .ok_or_else(|| SushiSwapError::ParseError("Invalid timestamp".to_string()))?
                .with_timezone(&Utc),
            data_type: MarketDataType::Position(Position {
                id: event["transactionHash"].as_str()
                    .ok_or_else(|| SushiSwapError::ParseError("Missing transaction hash".to_string()))?
                    .to_string(),
                token0_amount: Decimal::from_str_exact(data["amount0"].as_str()
                    .ok_or_else(|| SushiSwapError::ParseError("Missing amount0".to_string()))?)
                    .map_err(|e| SushiSwapError::ParseError(format!("Invalid amount0: {}", e)))?,
                token1_amount: Decimal::from_str_exact(data["amount1"].as_str()
                    .ok_or_else(|| SushiSwapError::ParseError("Missing amount1".to_string()))?)
                    .map_err(|e| SushiSwapError::ParseError(format!("Invalid amount1: {}", e)))?,
                liquidity: Decimal::from_str_exact(data["liquidity"].as_str()
                    .ok_or_else(|| SushiSwapError::ParseError("Missing liquidity".to_string()))?)
                    .map_err(|e| SushiSwapError::ParseError(format!("Invalid liquidity: {}", e)))?,
                tick_lower: 0, // SushiSwap V1不使用此字段
                tick_upper: 0, // SushiSwap V1不使用此字段
                position_type: "burn".to_string(),
            }),
            quality: DataQuality::Real,
        };

        Ok(Some(market_data))
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
            MarketDataType::Position(position) => {
                if !self.validate_amount(position.token0_amount) {
                    score -= 20.0;
                }
                if !self.validate_amount(position.token1_amount) {
                    score -= 20.0;
                }
                if !self.validate_amount(position.liquidity) {
                    score -= 20.0;
                }
            }
            MarketDataType::LiquidityEvent(event) => {
                if !self.validate_amount(event.token0_amount) {
                    score -= 20.0;
                }
                if !self.validate_amount(event.token1_amount) {
                    score -= 20.0;
                }
                if !self.validate_amount(event.liquidity) {
                    score -= 20.0;
                }
            }
            _ => {}
        }

        score.max(0.0)
    }
}

#[async_trait]
impl DataProcessor for SushiSwapProcessor {
    async fn process(&self, data: MarketData) -> Result<MarketData, Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();

        // 记录处理延迟
        metrics::record_rest_latency(
            "sushiswap",
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
        metrics::update_data_quality("sushiswap", "market_data", quality_score);

        // 记录验证延迟
        metrics::record_rest_latency(
            "sushiswap",
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
    async fn test_process_swap() {
        let processor = SushiSwapProcessor::new(HashMap::new());
        
        let event = serde_json::json!({
            "event": "Swap",
            "address": "0x1234567890abcdef",
            "timestamp": Utc::now().timestamp_millis(),
            "transactionHash": "0xabcdef1234567890",
            "data": {
                "price": "1000.5",
                "amount": "1.5",
                "amount0Out": "1.5",
                "amount1Out": "0"
            }
        });

        let result = processor.process_event_data(&event).await.unwrap().unwrap();
        
        if let MarketDataType::Trade(trades) = result.data_type {
            assert_eq!(trades[0].price, dec!(1000.5));
            assert_eq!(trades[0].quantity, dec!(1.5));
            assert_eq!(trades[0].side, Side::Buy);
        } else {
            panic!("Wrong market data type");
        }
    }

    #[tokio::test]
    async fn test_process_mint() {
        let processor = SushiSwapProcessor::new(HashMap::new());
        
        let event = serde_json::json!({
            "event": "Mint",
            "address": "0x1234567890abcdef",
            "timestamp": Utc::now().timestamp_millis(),
            "transactionHash": "0xabcdef1234567890",
            "data": {
                "amount0": "100",
                "amount1": "200",
                "liquidity": "1000"
            }
        });

        let result = processor.process_event_data(&event).await.unwrap().unwrap();
        
        if let MarketDataType::Position(position) = result.data_type {
            assert_eq!(position.token0_amount, dec!(100));
            assert_eq!(position.token1_amount, dec!(200));
            assert_eq!(position.liquidity, dec!(1000));
            assert_eq!(position.position_type, "mint");
        } else {
            panic!("Wrong market data type");
        }
    }

    #[tokio::test]
    async fn test_validate_data() {
        let processor = SushiSwapProcessor::new(HashMap::new());
        
        let good_data = MarketData {
            exchange: Exchange::SushiSwap,
            symbol: "0x1234567890abcdef".to_string(),
            timestamp: Utc::now(),
            data_type: MarketDataType::Trade(vec![Trade {
                id: "0xabcdef1234567890".to_string(),
                timestamp: Utc::now(),
                price: dec!(1000),
                quantity: dec!(1),
                side: Side::Buy,
            }]),
            quality: DataQuality::Real,
        };

        let quality = processor.validate(&good_data).await.unwrap();
        assert_eq!(quality, DataQuality::Real);

        let bad_data = MarketData {
            exchange: Exchange::SushiSwap,
            symbol: "0x1234567890abcdef".to_string(),
            timestamp: Utc::now(),
            data_type: MarketDataType::Trade(vec![Trade {
                id: "0xabcdef1234567890".to_string(),
                timestamp: Utc::now(),
                price: dec!(0),
                quantity: dec!(0),
                side: Side::Buy,
            }]),
            quality: DataQuality::Real,
        };

        let quality = processor.validate(&bad_data).await.unwrap();
        assert_eq!(quality, DataQuality::History);
    }
} 