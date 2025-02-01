use web3::{
    Web3,
    transports::Http,
    types::{Address, U256, FilterBuilder, BlockNumber, Log},
    contract::{Contract, Options},
};
use rust_decimal::Decimal;
use std::str::FromStr;
use crate::error::UniswapV3Error;
use crate::models::{Pool, PoolData, TickData, PoolEvent};
use hex;
use tracing::{info, error, debug};
use tracing_subscriber::{self, EnvFilter};

const UNISWAP_V3_POOL_ABI: &[u8] = include_bytes!("./abi/IUniswapV3Pool.json");
const ERC20_ABI: &[u8] = include_bytes!("./abi/IERC20.json");

/// Uniswap V3数据采集器
pub struct UniswapV3Collector {
    web3: Web3<Http>,
}

impl UniswapV3Collector {
    /// 创建新的采集器实例
    pub fn new(endpoint: &str) -> Result<Self, UniswapV3Error> {
        info!("创建 Uniswap V3 采集器，endpoint: {}", endpoint);
        let transport = Http::new(endpoint)
            .map_err(|e| {
                error!("创建 HTTP 传输失败: {}", e);
                UniswapV3Error::NetworkError(e.to_string())
            })?;
        
        Ok(Self {
            web3: Web3::new(transport),
        })
    }

    /// 获取池子信息
    pub async fn get_pool(&self, pool_address: &str) -> Result<PoolData, UniswapV3Error> {
        info!("获取池子信息，地址: {}", pool_address);
        let pool_address = Address::from_str(pool_address)
            .map_err(|e| {
                error!("解析池子地址失败: {}", e);
                UniswapV3Error::ParseError(e.to_string())
            })?;

        let pool_contract = Contract::from_json(
            self.web3.eth(),
            pool_address,
            UNISWAP_V3_POOL_ABI
        ).map_err(|e| {
            error!("创建池子合约失败: {}", e);
            UniswapV3Error::ContractError(e.to_string())
        })?;

        debug!("获取池子基本信息");
        // 获取基本信息
        let token0: Address = pool_contract.query("token0", (), None, Options::default(), None)
            .await
            .map_err(|e| {
                error!("获取 token0 失败: {}", e);
                UniswapV3Error::ContractError(e.to_string())
            })?;

        let token1: Address = pool_contract.query("token1", (), None, Options::default(), None)
            .await
            .map_err(|e| UniswapV3Error::ContractError(e.to_string()))?;

        let fee: U256 = pool_contract.query("fee", (), None, Options::default(), None)
            .await
            .map_err(|e| UniswapV3Error::ContractError(e.to_string()))?;

        let liquidity: U256 = pool_contract.query("liquidity", (), None, Options::default(), None)
            .await
            .map_err(|e| UniswapV3Error::ContractError(e.to_string()))?;

        let slot0: (U256, i32, U256, U256, U256, U256, bool) = pool_contract.query("slot0", (), None, Options::default(), None)
            .await
            .map_err(|e| UniswapV3Error::ContractError(e.to_string()))?;

        // 创建 ERC20 合约实例
        let token0_contract = Contract::from_json(
            self.web3.eth(),
            token0,
            ERC20_ABI
        ).map_err(|e| UniswapV3Error::ContractError(e.to_string()))?;

        let token1_contract = Contract::from_json(
            self.web3.eth(),
            token1,
            ERC20_ABI
        ).map_err(|e| UniswapV3Error::ContractError(e.to_string()))?;

        // 获取代币信息
        let token0_decimals: U256 = token0_contract.query("decimals", (), None, Options::default(), None)
            .await
            .map_err(|e| UniswapV3Error::ContractError(e.to_string()))?;

        let token1_decimals: U256 = token1_contract.query("decimals", (), None, Options::default(), None)
            .await
            .map_err(|e| UniswapV3Error::ContractError(e.to_string()))?;

        // 计算价格
        let sqrt_price_x96 = slot0.0;
        let tick = slot0.1;
        
        // Q96 = 2^96
        let q96 = U256::from(1) << 96;
        let price_u256 = sqrt_price_x96.pow(U256::from(2)) / (q96.pow(U256::from(2)));
        let mut price = Decimal::from(price_u256.as_u128());

        // 调整价格以考虑代币的小数位数
        let token0_decimals = token0_decimals.as_u64() as i32;
        let token1_decimals = token1_decimals.as_u64() as i32;
        let decimal_adjustment = token0_decimals - token1_decimals;
        
        if decimal_adjustment > 0 {
            price = price * Decimal::from(10i64.pow(decimal_adjustment as u32));
        } else if decimal_adjustment < 0 {
            price = price / Decimal::from(10i64.pow((-decimal_adjustment) as u32));
        }

        // 计算 token1/token0 的价格
        let price = if price != Decimal::from(0) {
            Decimal::from(1) / price
        } else {
            Decimal::from(0)
        };

        // 使用 tick 来计算价格作为备选方案
        let tick_price = if tick != 0 {
            let base_price = 1.0001f64.powf(tick as f64);
            Decimal::from_str(&format!("{:.10}", base_price)).unwrap_or_default()
        } else {
            price
        };

        // 使用两种方法计算的价格中的非零值，或默认为0
        let final_price = if price != Decimal::from(0) {
            price
        } else if tick_price != Decimal::from(0) {
            tick_price
        } else {
            Decimal::from(0)
        };

        Ok(PoolData {
            pool: Pool {
                address: format!("0x{}", hex::encode(pool_address.as_bytes())),
                token0: format!("0x{}", hex::encode(token0.as_bytes())),
                token1: format!("0x{}", hex::encode(token1.as_bytes())),
                fee: fee.as_u64() as u32,
            },
            token0_decimals: token0_decimals as u8,
            token1_decimals: token1_decimals as u8,
            liquidity: liquidity.as_u128(),
            sqrt_price_x96: sqrt_price_x96.as_u128(),
            tick,
            price: final_price,
        })
    }

    /// 获取最近的交易
    pub async fn get_recent_swaps(&self, pool_address: &str) -> Result<Vec<PoolEvent>, UniswapV3Error> {
        info!("获取最近交易，池子地址: {}", pool_address);
        let pool_address = Address::from_str(pool_address)
            .map_err(|e| {
                error!("解析池子地址失败: {}", e);
                UniswapV3Error::ParseError(e.to_string())
            })?;

        // 获取当前区块号
        let current_block = self.web3.eth().block_number()
            .await
            .map_err(|e| {
                error!("获取当前区块号失败: {}", e);
                UniswapV3Error::ContractError(e.to_string())
            })?;

        // 创建过滤器，获取最近1000个区块的事件
        let from_block = current_block.as_u64().saturating_sub(1000);
        debug!("获取区块范围: {} -> latest", from_block);
        
        let filter = FilterBuilder::default()
            .address(vec![pool_address])
            .from_block(BlockNumber::Number(from_block.into()))
            .to_block(BlockNumber::Latest)
            .topics(
                Some(vec![
                    "0xc42079f94a6350d7e6235f29174924f928cc2ac818eb64fed8004e115fbcca67".parse().unwrap()
                ]),
                None,
                None,
                None,
            )
            .build();

        // 获取事件日志
        let logs = self.web3.eth().logs(filter)
            .await
            .map_err(|e| {
                error!("获取事件日志失败: {}", e);
                UniswapV3Error::ContractError(e.to_string())
            })?;

        debug!("找到 {} 条交易记录", logs.len());

        // 解析事件日志
        let mut swaps = Vec::new();
        for (i, log) in logs.iter().enumerate() {
            debug!("解析第 {} 条交易记录", i + 1);
            if let Some(swap) = self.parse_swap_event(log.clone()).await? {
                swaps.push(swap);
            }
        }

        info!("成功解析 {} 条交易记录", swaps.len());
        Ok(swaps)
    }

    /// 解析 Swap 事件
    async fn parse_swap_event(&self, log: Log) -> Result<Option<PoolEvent>, UniswapV3Error> {
        if log.topics.len() < 3 {
            return Ok(None);
        }

        // 解析事件参数
        let sender = format!("0x{}", hex::encode(&log.topics[1].0));
        let recipient = format!("0x{}", hex::encode(&log.topics[2].0));

        if log.data.0.len() < 5 * 32 {
            return Ok(None);
        }

        // 获取池子合约地址
        let pool_address = log.address;
        
        // 创建池子合约实例
        let pool_contract = Contract::from_json(
            self.web3.eth(),
            pool_address,
            UNISWAP_V3_POOL_ABI
        ).map_err(|e| UniswapV3Error::ContractError(e.to_string()))?;

        // 获取代币地址
        let token0: Address = pool_contract.query("token0", (), None, Options::default(), None)
            .await
            .map_err(|e| UniswapV3Error::ContractError(e.to_string()))?;

        let token1: Address = pool_contract.query("token1", (), None, Options::default(), None)
            .await
            .map_err(|e| UniswapV3Error::ContractError(e.to_string()))?;

        // 创建代币合约实例
        let token0_contract = Contract::from_json(
            self.web3.eth(),
            token0,
            ERC20_ABI
        ).map_err(|e| UniswapV3Error::ContractError(e.to_string()))?;

        let token1_contract = Contract::from_json(
            self.web3.eth(),
            token1,
            ERC20_ABI
        ).map_err(|e| UniswapV3Error::ContractError(e.to_string()))?;

        // 获取代币小数位数
        let token0_decimals: U256 = token0_contract.query("decimals", (), None, Options::default(), None)
            .await
            .map_err(|e| UniswapV3Error::ContractError(e.to_string()))?;

        let token1_decimals: U256 = token1_contract.query("decimals", (), None, Options::default(), None)
            .await
            .map_err(|e| UniswapV3Error::ContractError(e.to_string()))?;

        // 解析数据字段
        let amount0 = i128::from_be_bytes((&log.data.0[16..32]).try_into().unwrap());
        let amount1 = i128::from_be_bytes((&log.data.0[48..64]).try_into().unwrap());
        let sqrt_price_x96 = U256::from_big_endian(&log.data.0[64..96]);
        let liquidity = U256::from_big_endian(&log.data.0[96..128]);
        let tick = i32::from_be_bytes((&log.data.0[128..132]).try_into().unwrap());

        // 调整数值到正确的范围
        let amount0_dec = if amount0 >= 0 {
            Decimal::from(amount0 as i64)
        } else {
            -Decimal::from((-amount0) as i64)
        };

        let amount1_dec = if amount1 >= 0 {
            Decimal::from(amount1 as i64)
        } else {
            -Decimal::from((-amount1) as i64)
        };

        // 根据实际代币小数位数调整金额
        let amount0_dec = amount0_dec / Decimal::from(10i64.pow(token0_decimals.as_u32()));
        let amount1_dec = amount1_dec / Decimal::from(10i64.pow(token1_decimals.as_u32()));

        let sqrt_price_x96_dec = Decimal::from_str(&sqrt_price_x96.to_string())
            .unwrap_or_default();
        let liquidity_dec = Decimal::from_str(&liquidity.to_string())
            .unwrap_or_default();

        Ok(Some(PoolEvent::Swap {
            sender,
            recipient,
            amount0: amount0_dec,
            amount1: amount1_dec,
            sqrt_price_x96: sqrt_price_x96_dec,
            liquidity: liquidity_dec,
            tick,
        }))
    }

    /// 获取流动性分布
    pub async fn get_liquidity_distribution(&self, pool_address: &str) -> Result<Vec<TickData>, UniswapV3Error> {
        info!("获取流动性分布，池子地址: {}", pool_address);
        let pool_address = Address::from_str(pool_address)
            .map_err(|e| {
                error!("解析池子地址失败: {}", e);
                UniswapV3Error::ParseError(e.to_string())
            })?;

        let pool_contract = Contract::from_json(
            self.web3.eth(),
            pool_address,
            UNISWAP_V3_POOL_ABI
        ).map_err(|e| {
            error!("创建池子合约失败: {}", e);
            UniswapV3Error::ContractError(e.to_string())
        })?;

        // 获取当前 tick
        debug!("获取当前 tick");
        let slot0: (U256, i32, U256, U256, U256, U256, bool) = pool_contract.query("slot0", (), None, Options::default(), None)
            .await
            .map_err(|e| {
                error!("获取 slot0 失败: {}", e);
                UniswapV3Error::ContractError(e.to_string())
            })?;
        let current_tick = slot0.1;

        // 获取当前 tick 附近的流动性分布
        let mut ticks = Vec::new();
        let range = 100;
        debug!("获取 tick 范围: {} -> {}", current_tick - range, current_tick + range);

        for tick in (current_tick - range)..=(current_tick + range) {
            // 获取 tick 信息
            let tick_data: (U256, i128, U256, U256, i64, U256, u32, bool) = pool_contract
                .query("ticks", (tick,), None, Options::default(), None)
                .await
                .map_err(|e| {
                    error!("获取 tick {} 信息失败: {}", tick, e);
                    UniswapV3Error::ContractError(e.to_string())
                })?;

            // 只添加已初始化的 tick
            if tick_data.7 {
                debug!("找到已初始化的 tick: {}", tick);
                let liquidity_gross = tick_data.0.as_u128();
                let liquidity_net = tick_data.1;

                // 计算价格
                let price = 1.0001f64.powf(tick as f64);
                let price_dec = Decimal::from_str(&format!("{:.10}", price))
                    .unwrap_or_default();

                ticks.push(TickData {
                    tick_idx: tick,
                    liquidity_gross,
                    liquidity_net,
                    price0: price_dec,
                    price1: Decimal::from(1) / price_dec,
                });
            }
        }

        info!("成功获取 {} 个已初始化的 tick", ticks.len());
        Ok(ticks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;
    use std::env;
    use tracing_subscriber::{self, EnvFilter};

    const TEST_POOL_ADDRESS: &str = "0x8ad599c3a0ff1de082011efddc58f1908eb6e6d8"; // USDC/ETH 池子

    fn setup_logging() {
        let _ = tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::from_default_env()
                .add_directive("uniswap_v3=debug".parse().unwrap()))
            .try_init();
    }

    fn get_rpc_url() -> String {
        env::var("ETH_PRIMARY_URL").unwrap_or_else(|_| {
            warn!("未设置 ETH_PRIMARY_URL 环境变量，使用默认的演示节点");
            "https://eth-mainnet.g.alchemy.com/v2/demo".to_string()
        })
    }

    #[tokio::test]
    async fn test_get_pool() {
        setup_logging();
        let collector = UniswapV3Collector::new(&get_rpc_url()).unwrap();
        let result = collector.get_pool(TEST_POOL_ADDRESS).await;
        
        if let Err(ref e) = result {
            error!("获取池子信息失败: {:?}", e);
        }
        assert!(result.is_ok(), "获取池子信息失败");
        
        let pool = result.unwrap();
        assert_eq!(pool.pool.address.to_lowercase(), TEST_POOL_ADDRESS.to_lowercase());
        assert_eq!(pool.pool.fee, 3000); // 0.3%费率
        assert_eq!(pool.token0_decimals, 6); // USDC 6位小数
        assert_eq!(pool.token1_decimals, 18); // ETH 18位小数
    }

    #[tokio::test]
    async fn test_get_recent_swaps() {
        setup_logging();
        let collector = UniswapV3Collector::new(&get_rpc_url()).unwrap();
        let result = collector.get_recent_swaps(TEST_POOL_ADDRESS).await;
        
        if let Err(ref e) = result {
            error!("获取最近交易失败: {:?}", e);
        }
        assert!(result.is_ok(), "获取最近交易失败");
        
        let swaps = result.unwrap();
        assert!(!swaps.is_empty(), "应该有最近的交易记录");
        
        if let PoolEvent::Swap { amount0, amount1, .. } = &swaps[0] {
            assert!(*amount0 != Decimal::from(0) || *amount1 != Decimal::from(0), "交易金额不应该都为0");
        }
    }

    #[tokio::test]
    async fn test_get_liquidity_distribution() {
        setup_logging();
        let collector = UniswapV3Collector::new(&get_rpc_url()).unwrap();
        let result = collector.get_liquidity_distribution(TEST_POOL_ADDRESS).await;
        
        if let Err(ref e) = result {
            error!("获取流动性分布失败: {:?}", e);
        }
        assert!(result.is_ok(), "获取流动性分布失败");
        
        let ticks = result.unwrap();
        assert!(!ticks.is_empty(), "应该有流动性分布数据");
        
        // 验证价格计算是否正确
        for tick in ticks {
            assert!(tick.price0 > Decimal::from(0), "price0 应该大于0");
            assert!(tick.price1 > Decimal::from(0), "price1 应该大于0");
            assert_eq!(
                (tick.price0 * tick.price1).round_dp(8),
                Decimal::from(1),
                "price0 * price1 应该约等于1"
            );
        }
    }
} 