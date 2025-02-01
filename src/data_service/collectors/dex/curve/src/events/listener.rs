use crate::error::CurveError;
use crate::models::{Trade, AddLiquidity, RemoveLiquidity};
use ethers::{
    providers::{Provider, Http, Middleware, StreamExt, Ws},
    contract::Contract,
    core::types::{Address, Filter, Log, H256, U256},
    prelude::*,
};
use std::sync::Arc;
use std::str::FromStr;
use rust_decimal::Decimal;
use chrono::Utc;
use tokio::sync::broadcast::{self, Sender, Receiver};
use async_trait::async_trait;

const EVENTS_ABI: &str = include_str!("../../abi/CurveEvents.json");

/// 事件类型
#[derive(Debug, Clone)]
pub enum CurveEvent {
    /// 交易事件
    Trade(Trade),
    /// 添加流动性事件
    AddLiquidity(AddLiquidity),
    /// 移除流动性事件
    RemoveLiquidity(RemoveLiquidity),
}

/// 事件监听器特征
#[async_trait]
pub trait EventListener: Send + Sync {
    /// 开始监听事件
    async fn start(&self) -> Result<(), CurveError>;
    /// 停止监听事件
    async fn stop(&self) -> Result<(), CurveError>;
    /// 订阅事件
    fn subscribe(&self) -> Receiver<CurveEvent>;
}

/// Curve事件监听器
pub struct CurveEventListener {
    provider: Arc<Provider<Ws>>,
    pool_address: String,
    event_sender: Sender<CurveEvent>,
    contract: Contract<Provider<Ws>>,
}

impl CurveEventListener {
    /// 创建新的事件监听器实例
    pub async fn new(ws_url: &str, pool_address: &str) -> Result<Self, CurveError> {
        let provider = Provider::<Ws>::connect(ws_url)
            .await
            .map_err(|e| CurveError::ContractError(format!("WebSocket连接失败: {}", e)))?;
        let provider = Arc::new(provider);

        let address = Address::from_str(pool_address)
            .map_err(|e| CurveError::ContractError(format!("池子地址无效: {}", e)))?;

        let contract = Contract::new(address, EVENTS_ABI.parse().unwrap(), provider.clone());

        let (sender, _) = broadcast::channel(100); // 缓冲100个事件

        Ok(Self {
            provider,
            pool_address: pool_address.to_string(),
            event_sender: sender,
            contract,
        })
    }

    /// 解析交易事件
    async fn parse_trade_event(&self, log: &Log) -> Result<Trade, CurveError> {
        let buyer = log.topics[1]
            .as_bytes()
            .try_into()
            .map_err(|e| CurveError::ParseError(format!("解析买家地址失败: {}", e)))?;
        let buyer = Address::from(buyer);

        let parsed = self.contract
            .decode_event::<(i128, U256, i128, U256), H256>("TokenExchange", log.topics.clone(), log.data.clone())
            .map_err(|e| CurveError::ParseError(format!("解析交易事件失败: {}", e)))?;

        Ok(Trade {
            pool_address: self.pool_address.clone(),
            trader: format!("{:?}", buyer),
            token_in_index: parsed.0 as u8,
            token_out_index: parsed.2 as u8,
            amount_in: self.convert_to_decimal(&parsed.1)?,
            amount_out: self.convert_to_decimal(&parsed.3)?,
            fee: Decimal::from(0), // 从事件中无法获取具体费用
            timestamp: Utc::now(),
        })
    }

    /// 解析添加流动性事件
    async fn parse_add_liquidity_event(&self, log: &Log) -> Result<AddLiquidity, CurveError> {
        let provider = log.topics[1]
            .as_bytes()
            .try_into()
            .map_err(|e| CurveError::ParseError(format!("解析提供者地址失败: {}", e)))?;
        let provider = Address::from(provider);

        let parsed = self.contract
            .decode_event::<([U256; 3], [U256; 3], U256, U256), H256>("AddLiquidity", log.topics.clone(), log.data.clone())
            .map_err(|e| CurveError::ParseError(format!("解析添加流动性事件失败: {}", e)))?;

        let token_amounts = parsed.0
            .iter()
            .map(|amount| self.convert_to_decimal(amount))
            .collect::<Result<Vec<Decimal>, CurveError>>()?;

        let fees = parsed.1
            .iter()
            .map(|fee| self.convert_to_decimal(fee))
            .collect::<Result<Vec<Decimal>, CurveError>>()?;

        Ok(AddLiquidity {
            pool_address: self.pool_address.clone(),
            provider: format!("{:?}", provider),
            token_amounts,
            lp_token_amount: self.convert_to_decimal(&parsed.3)?,
            fee: fees.into_iter().sum(),
            timestamp: Utc::now(),
        })
    }

    /// 解析移除流动性事件
    async fn parse_remove_liquidity_event(&self, log: &Log) -> Result<RemoveLiquidity, CurveError> {
        let provider = log.topics[1]
            .as_bytes()
            .try_into()
            .map_err(|e| CurveError::ParseError(format!("解析提供者地址失败: {}", e)))?;
        let provider = Address::from(provider);

        let parsed = self.contract
            .decode_event::<([U256; 3], [U256; 3], U256), H256>("RemoveLiquidity", log.topics.clone(), log.data.clone())
            .map_err(|e| CurveError::ParseError(format!("解析移除流动性事件失败: {}", e)))?;

        let token_amounts = parsed.0
            .iter()
            .map(|amount| self.convert_to_decimal(amount))
            .collect::<Result<Vec<Decimal>, CurveError>>()?;

        let fees = parsed.1
            .iter()
            .map(|fee| self.convert_to_decimal(fee))
            .collect::<Result<Vec<Decimal>, CurveError>>()?;

        Ok(RemoveLiquidity {
            pool_address: self.pool_address.clone(),
            provider: format!("{:?}", provider),
            token_amounts,
            lp_token_amount: self.convert_to_decimal(&parsed.2)?,
            fee: fees.into_iter().sum(),
            timestamp: Utc::now(),
        })
    }

    // 辅助方法：将U256转换为Decimal
    fn convert_to_decimal(&self, value: &U256) -> Result<Decimal, CurveError> {
        let value_str = value.to_string();
        Decimal::from_str(&value_str)
            .map_err(|e| CurveError::ConversionError(format!("U256转Decimal失败: {}", e)))
    }
}

#[async_trait]
impl EventListener for CurveEventListener {
    async fn start(&self) -> Result<(), CurveError> {
        let address = Address::from_str(&self.pool_address)
            .map_err(|e| CurveError::ContractError(format!("池子地址无效: {}", e)))?;

        let filter = Filter::new()
            .address(address)
            .event("TokenExchange(address,int128,uint256,int128,uint256)")
            .event("AddLiquidity(address,uint256[3],uint256[3],uint256,uint256)")
            .event("RemoveLiquidity(address,uint256[3],uint256[3],uint256)");

        let mut stream = self.provider
            .subscribe_logs(&filter)
            .await
            .map_err(|e| CurveError::ContractError(format!("订阅事件失败: {}", e)))?;

        // 启动事件处理循环
        tokio::spawn({
            let event_sender = self.event_sender.clone();
            let this = self.clone();
            async move {
                while let Some(log) = stream.next().await {
                    let event = match log.topics[0] {
                        topic if topic == H256::from_str("0x8b3e96f2b889fa771c53c981b40daf005f63f637f1869f707052d15a3dd97140").unwrap() => {
                            // TokenExchange
                            match this.parse_trade_event(&log).await {
                                Ok(trade) => CurveEvent::Trade(trade),
                                Err(e) => {
                                    eprintln!("解析交易事件失败: {}", e);
                                    continue;
                                }
                            }
                        },
                        topic if topic == H256::from_str("0x26f55a85081d24974e85c6c00045d0f0453991e95873f52bff0d21af4079a768").unwrap() => {
                            // AddLiquidity
                            match this.parse_add_liquidity_event(&log).await {
                                Ok(add) => CurveEvent::AddLiquidity(add),
                                Err(e) => {
                                    eprintln!("解析添加流动性事件失败: {}", e);
                                    continue;
                                }
                            }
                        },
                        topic if topic == H256::from_str("0x7c363854ccf79623411f8995b362bce5eddff18c927edc6f5dbbb5e05819a82c").unwrap() => {
                            // RemoveLiquidity
                            match this.parse_remove_liquidity_event(&log).await {
                                Ok(remove) => CurveEvent::RemoveLiquidity(remove),
                                Err(e) => {
                                    eprintln!("解析移除流动性事件失败: {}", e);
                                    continue;
                                }
                            }
                        },
                        _ => continue,
                    };

                    if let Err(e) = event_sender.send(event) {
                        eprintln!("发送事件失败: {}", e);
                    }
                }
            }
        });

        Ok(())
    }

    async fn stop(&self) -> Result<(), CurveError> {
        // WebSocket provider会在drop时自动断开连接
        Ok(())
    }

    fn subscribe(&self) -> Receiver<CurveEvent> {
        self.event_sender.subscribe()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_event_listener() {
        let ws_url = env::var("ETH_WS_URL").expect("需要设置 ETH_WS_URL 环境变量");
        // 3pool地址
        let pool_address = "0xbebc44782c7db0a1a60cb6fe97d0b483032ff1c7";

        let listener = CurveEventListener::new(&ws_url, pool_address).await.unwrap();
        let mut receiver = listener.subscribe();

        listener.start().await.unwrap();

        // 等待30秒，看是否能收到事件
        tokio::spawn(async move {
            while let Ok(event) = receiver.recv().await {
                println!("收到事件: {:?}", event);
            }
        });

        sleep(Duration::from_secs(30)).await;
    }
} 