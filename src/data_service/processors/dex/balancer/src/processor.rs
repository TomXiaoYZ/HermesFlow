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
use crate::error::BalancerError;
use crate::models::{Pool, Token, PoolData};
use crate::metrics;

/// Balancer数据处理器
pub struct BalancerProcessor {
    pools: HashMap<String, Pool>,
    tokens: HashMap<String, Token>,
    config: HashMap<String, String>,
}

impl BalancerProcessor {
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
    pub async fn process_graph_data(&self, data: &Value) -> Result<Option<MarketData>, BalancerError> {
        if let Some(pool_data) = data.get("pool") {
            let pool_id = pool_data["id"].as_str()
                .ok_or_else(|| BalancerError::ParseError("Missing pool ID".to_string()))?;
            
            let tokens: Vec<String> = pool_data["tokens"]
                .as_array()
                .ok_or_else(|| BalancerError::ParseError("Missing tokens array".to_string()))?
                .iter()
                .map(|token| token["address"].as_str()
                    .ok_or_else(|| BalancerError::ParseError("Missing token address".to_string()))
                    .map(|s| s.to_string()))
                .collect::<Result<Vec<_>, _>>()?;

            let weights: Vec<Decimal> = pool_data["weights"]
                .as_array()
                .ok_or_else(|| BalancerError::ParseError("Missing weights array".to_string()))?
                .iter()
                .map(|weight| Decimal::from_str_exact(weight.as_str()
                    .ok_or_else(|| BalancerError::ParseError("Invalid weight format".to_string()))?)
                    .map_err(|e| BalancerError::ParseError(format!("Invalid weight: {}", e))))
                .collect::<Result<Vec<_>, _>>()?;

            let balances: Vec<Decimal> = pool_data["balances"]
                .as_array()
                .ok_or_else(|| BalancerError::ParseError("Missing balances array".to_string()))?
                .iter()
                .map(|balance| Decimal::from_str_exact(balance.as_str()
                    .ok_or_else(|| BalancerError::ParseError("Invalid balance format".to_string()))?)
                    .map_err(|e| BalancerError::ParseError(format!("Invalid balance: {}", e))))
                .collect::<Result<Vec<_>, _>>()?;

            let symbol = tokens.join("-");

            let market_data = MarketData {
                exchange: Exchange::Balancer,
                symbol,
                timestamp: Utc::now(),
                data_type: MarketDataType::LiquidityEvent(LiquidityEvent {
                    pool_id: pool_id.to_string(),
                    token0_address: tokens[0].clone(),
                    token1_address: tokens[1].clone(),
                    token0_amount: balances[0],
                    token1_amount: balances[1],
                    liquidity: pool_data["totalLiquidity"]
                        .as_str()
                        .ok_or_else(|| BalancerError::ParseError("Missing total liquidity".to_string()))
                        .and_then(|s| Decimal::from_str_exact(s)
                            .map_err(|e| BalancerError::ParseError(format!("Invalid total liquidity: {}", e))))?,
                    sqrt_price_x96: Decimal::ZERO, // Balancer不使用此字段
                    tick: 0, // Balancer不使用此字段
                    fee_tier: pool_data["swapFee"]
                        .as_str()
                        .ok_or_else(|| BalancerError::ParseError("Missing swap fee".to_string()))
                        .and_then(|s| Decimal::from_str_exact(s)
                            .map_err(|e| BalancerError::ParseError(format!("Invalid swap fee: {}", e))))
                        .map(|fee| (fee * Decimal::from(10000)).to_u64().unwrap_or(0))?,
                }),
                quality: DataQuality::Real,
            };

            Ok(Some(market_data))
        } else {
            Ok(None)
        }
    }

    /// 处理合约事件数据
    pub async fn process_event_data(&self, event: &Value) -> Result<Option<MarketData>, BalancerError> {
        if let Some(event_name) = event["event"].as_str() {
            match event_name {
                "Swap" => self.process_swap_event(event),
                "PoolBalanceChanged" => self.process_balance_change_event(event),
                "PoolCreated" => self.process_pool_created_event(event),
                _ => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    /// 处理Swap事件
    fn process_swap_event(&self, event: &Value) -> Result<Option<MarketData>, BalancerError> {
        let data = event["data"].as_object()
            .ok_or_else(|| BalancerError::ParseError("Missing event data".to_string()))?;

        let pool_address = event["address"].as_str()
            .ok_or_else(|| BalancerError::ParseError("Missing pool address".to_string()))?;

        let market_data = MarketData {
            exchange: Exchange::Balancer,
            symbol: pool_address.to_string(),
            timestamp: DateTime::from_timestamp_millis(
                event["timestamp"].as_i64()
                    .ok_or_else(|| BalancerError::ParseError("Missing timestamp".to_string()))?)
                .ok_or_else(|| BalancerError::ParseError("Invalid timestamp".to_string()))?
                .with_timezone(&Utc),
            data_type: MarketDataType::Trade(vec![Trade {
                id: event["transactionHash"].as_str()
                    .ok_or_else(|| BalancerError::ParseError("Missing transaction hash".to_string()))?
                    .to_string(),
                price: Decimal::from_str_exact(data["price"].as_str()
                    .ok_or_else(|| BalancerError::ParseError("Missing price".to_string()))?)
                    .map_err(|e| BalancerError::ParseError(format!("Invalid price: {}", e)))?,
                quantity: Decimal::from_str_exact(data["amount"].as_str()
                    .ok_or_else(|| BalancerError::ParseError("Missing amount".to_string()))?)
                    .map_err(|e| BalancerError::ParseError(format!("Invalid amount: {}", e)))?,
                timestamp: Utc::now(),
                side: if data["tokenInIndex"].as_u64().unwrap_or(0) == 0 { Side::Buy } else { Side::Sell },
            }]),
            quality: DataQuality::Real,
        };

        Ok(Some(market_data))
    }

    /// 处理池子余额变化事件
    fn process_balance_change_event(&self, event: &Value) -> Result<Option<MarketData>, BalancerError> {
        let data = event["data"].as_object()
            .ok_or_else(|| BalancerError::ParseError("Missing event data".to_string()))?;

        let pool_address = event["address"].as_str()
            .ok_or_else(|| BalancerError::ParseError("Missing pool address".to_string()))?;

        let market_data = MarketData {
            exchange: Exchange::Balancer,
            symbol: pool_address.to_string(),
            timestamp: DateTime::from_timestamp_millis(
                event["timestamp"].as_i64()
                    .ok_or_else(|| BalancerError::ParseError("Missing timestamp".to_string()))?)
                .ok_or_else(|| BalancerError::ParseError("Invalid timestamp".to_string()))?
                .with_timezone(&Utc),
            data_type: MarketDataType::Position(Position {
                id: event["transactionHash"].as_str()
                    .ok_or_else(|| BalancerError::ParseError("Missing transaction hash".to_string()))?
                    .to_string(),
                token0_amount: Decimal::from_str_exact(data["balance0"].as_str()
                    .ok_or_else(|| BalancerError::ParseError("Missing balance0".to_string()))?)
                    .map_err(|e| BalancerError::ParseError(format!("Invalid balance0: {}", e)))?,
                token1_amount: Decimal::from_str_exact(data["balance1"].as_str()
                    .ok_or_else(|| BalancerError::ParseError("Missing balance1".to_string()))?)
                    .map_err(|e| BalancerError::ParseError(format!("Invalid balance1: {}", e)))?,
                liquidity: Decimal::from_str_exact(data["liquidity"].as_str()
                    .ok_or_else(|| BalancerError::ParseError("Missing liquidity".to_string()))?)
                    .map_err(|e| BalancerError::ParseError(format!("Invalid liquidity: {}", e)))?,
                tick_lower: 0, // Balancer不使用此字段
                tick_upper: 0, // Balancer不使用此字段
                position_type: "balance_change".to_string(),
            }),
            quality: DataQuality::Real,
        };

        Ok(Some(market_data))
    }

    /// 处理池子创建事件
    fn process_pool_created_event(&self, event: &Value) -> Result<Option<MarketData>, BalancerError> {
        let data = event["data"].as_object()
            .ok_or_else(|| BalancerError::ParseError("Missing event data".to_string()))?;

        let pool_address = event["address"].as_str()
            .ok_or_else(|| BalancerError::ParseError("Missing pool address".to_string()))?;

        let market_data = MarketData {
            exchange: Exchange::Balancer,
            symbol: pool_address.to_string(),
            timestamp: DateTime::from_timestamp_millis(
                event["timestamp"].as_i64()
                    .ok_or_else(|| BalancerError::ParseError("Missing timestamp".to_string()))?)
                .ok_or_else(|| BalancerError::ParseError("Invalid timestamp".to_string()))?
                .with_timezone(&Utc),
            data_type: MarketDataType::Position(Position {
                id: event["transactionHash"].as_str()
                    .ok_or_else(|| BalancerError::ParseError("Missing transaction hash".to_string()))?
                    .to_string(),
                token0_amount: Decimal::ZERO,
                token1_amount: Decimal::ZERO,
                liquidity: Decimal::ZERO,
                tick_lower: 0, // Balancer不使用此字段
                tick_upper: 0, // Balancer不使用此字段
                position_type: "pool_created".to_string(),
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
impl DataProcessor for BalancerProcessor {
    async fn process(&self, data: MarketData) -> Result<MarketData, Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();

        // 记录处理延迟
        metrics::record_rest_latency(
            "balancer",
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
        metrics::update_data_quality("balancer", "market_data", quality_score);

        // 记录验证延迟
        metrics::record_rest_latency(
            "balancer",
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
        let processor = BalancerProcessor::new(HashMap::new());
        
        let event = serde_json::json!({
            "event": "Swap",
            "address": "0x1234567890abcdef",
            "timestamp": Utc::now().timestamp_millis(),
            "transactionHash": "0xabcdef1234567890",
            "data": {
                "price": "1000.5",
                "amount": "1.5",
                "tokenInIndex": 0
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
    async fn test_process_balance_change() {
        let processor = BalancerProcessor::new(HashMap::new());
        
        let event = serde_json::json!({
            "event": "PoolBalanceChanged",
            "address": "0x1234567890abcdef",
            "timestamp": Utc::now().timestamp_millis(),
            "transactionHash": "0xabcdef1234567890",
            "data": {
                "balance0": "100",
                "balance1": "200",
                "liquidity": "1000"
            }
        });

        let result = processor.process_event_data(&event).await.unwrap().unwrap();
        
        if let MarketDataType::Position(position) = result.data_type {
            assert_eq!(position.token0_amount, dec!(100));
            assert_eq!(position.token1_amount, dec!(200));
            assert_eq!(position.liquidity, dec!(1000));
            assert_eq!(position.position_type, "balance_change");
        } else {
            panic!("Wrong market data type");
        }
    }

    #[tokio::test]
    async fn test_validate_data() {
        let processor = BalancerProcessor::new(HashMap::new());
        
        let good_data = MarketData {
            exchange: Exchange::Balancer,
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
            exchange: Exchange::Balancer,
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