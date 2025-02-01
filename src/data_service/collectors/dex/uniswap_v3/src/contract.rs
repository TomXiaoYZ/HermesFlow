use crate::error::UniswapV3Error;
use crate::models::{Pool, Token, TickData};
use ethers::{
    providers::{Provider, Http},
    contract::Contract,
    core::types::{Address, U256},
    prelude::*,
};
use std::sync::Arc;
use std::str::FromStr;
use rust_decimal::Decimal;

const POOL_ABI: &str = include_str!("../abi/UniswapV3Pool.json");
const FACTORY_ABI: &str = include_str!("../abi/UniswapV3Factory.json");
const FACTORY_ADDRESS: &str = "0x1F98431c8aD98523631AE4a59f267346ea31F984";

/// Uniswap V3合约调用器
pub struct UniswapV3Contract {
    provider: Arc<Provider<Http>>,
    factory: Contract<Provider<Http>>,
}

impl UniswapV3Contract {
    /// 创建新的合约调用器实例
    pub fn new(rpc_url: &str) -> Result<Self, UniswapV3Error> {
        let provider = Provider::<Http>::try_from(rpc_url)
            .map_err(|e| UniswapV3Error::ContractError(format!("RPC连接失败: {}", e)))?;
        let provider = Arc::new(provider);

        let factory_address = Address::from_str(FACTORY_ADDRESS)
            .map_err(|e| UniswapV3Error::ContractError(format!("工厂合约地址无效: {}", e)))?;

        let factory = Contract::new(factory_address, FACTORY_ABI.parse().unwrap(), provider.clone());

        Ok(Self {
            provider,
            factory,
        })
    }

    /// 获取池子合约实例
    async fn get_pool_contract(&self, pool_address: &str) -> Result<Contract<Provider<Http>>, UniswapV3Error> {
        let address = Address::from_str(pool_address)
            .map_err(|e| UniswapV3Error::ContractError(format!("池子地址无效: {}", e)))?;

        Ok(Contract::new(address, POOL_ABI.parse().unwrap(), self.provider.clone()))
    }

    /// 获取实时池子数据
    pub async fn get_pool_state(&self, pool_address: &str) -> Result<(U256, i32, U256, U256, U256), UniswapV3Error> {
        let pool = self.get_pool_contract(pool_address).await?;

        let slot0: (U256, i32, U256, U256, U256, U256, bool) = pool
            .method("slot0", ())
            .map_err(|e| UniswapV3Error::ContractError(format!("获取slot0失败: {}", e)))?
            .call()
            .await
            .map_err(|e| UniswapV3Error::ContractError(format!("调用slot0失败: {}", e)))?;

        Ok((
            slot0.0, // sqrtPriceX96
            slot0.1, // tick
            slot0.2, // observationIndex
            slot0.3, // observationCardinality
            slot0.4, // observationCardinalityNext
        ))
    }

    /// 获取实时流动性数据
    pub async fn get_liquidity(&self, pool_address: &str) -> Result<U256, UniswapV3Error> {
        let pool = self.get_pool_contract(pool_address).await?;

        let liquidity: U256 = pool
            .method("liquidity", ())
            .map_err(|e| UniswapV3Error::ContractError(format!("获取liquidity失败: {}", e)))?
            .call()
            .await
            .map_err(|e| UniswapV3Error::ContractError(format!("调用liquidity失败: {}", e)))?;

        Ok(liquidity)
    }

    /// 获取Tick数据
    pub async fn get_tick_info(&self, pool_address: &str, tick: i32) -> Result<(U256, U256, U256, U256, bool), UniswapV3Error> {
        let pool = self.get_pool_contract(pool_address).await?;

        let tick_info: (U256, U256, U256, U256, bool) = pool
            .method("ticks", tick)
            .map_err(|e| UniswapV3Error::ContractError(format!("获取tick信息失败: {}", e)))?
            .call()
            .await
            .map_err(|e| UniswapV3Error::ContractError(format!("调用ticks失败: {}", e)))?;

        Ok(tick_info)
    }

    /// 获取费率
    pub async fn get_fee(&self, pool_address: &str) -> Result<u32, UniswapV3Error> {
        let pool = self.get_pool_contract(pool_address).await?;

        let fee: u32 = pool
            .method("fee", ())
            .map_err(|e| UniswapV3Error::ContractError(format!("获取fee失败: {}", e)))?
            .call()
            .await
            .map_err(|e| UniswapV3Error::ContractError(format!("调用fee失败: {}", e)))?;

        Ok(fee)
    }

    /// 获取代币地址
    pub async fn get_tokens(&self, pool_address: &str) -> Result<(Address, Address), UniswapV3Error> {
        let pool = self.get_pool_contract(pool_address).await?;

        let token0: Address = pool
            .method("token0", ())
            .map_err(|e| UniswapV3Error::ContractError(format!("获取token0失败: {}", e)))?
            .call()
            .await
            .map_err(|e| UniswapV3Error::ContractError(format!("调用token0失败: {}", e)))?;

        let token1: Address = pool
            .method("token1", ())
            .map_err(|e| UniswapV3Error::ContractError(format!("获取token1失败: {}", e)))?
            .call()
            .await
            .map_err(|e| UniswapV3Error::ContractError(format!("调用token1失败: {}", e)))?;

        Ok((token0, token1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[tokio::test]
    async fn test_contract_calls() {
        let rpc_url = env::var("ETH_RPC_URL").expect("需要设置 ETH_RPC_URL 环境变量");
        let contract = UniswapV3Contract::new(&rpc_url).unwrap();

        // USDC/ETH池子
        let pool_address = "0x8ad599c3a0ff1de082011efddc58f1908eb6e6d8";

        // 测试获取池子状态
        let state = contract.get_pool_state(pool_address).await;
        assert!(state.is_ok());

        // 测试获取流动性
        let liquidity = contract.get_liquidity(pool_address).await;
        assert!(liquidity.is_ok());

        // 测试获取费率
        let fee = contract.get_fee(pool_address).await;
        assert!(fee.is_ok());
        assert_eq!(fee.unwrap(), 3000); // 0.3%费率

        // 测试获取代币地址
        let tokens = contract.get_tokens(pool_address).await;
        assert!(tokens.is_ok());
        let (token0, token1) = tokens.unwrap();
        assert_eq!(
            token0.to_string().to_lowercase(),
            "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48" // USDC
        );
        assert_eq!(
            token1.to_string().to_lowercase(),
            "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2" // WETH
        );
    }
} 