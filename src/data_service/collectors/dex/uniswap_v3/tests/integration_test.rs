use uniswap_v3::{
    collector::UniswapV3Collector,
    error::UniswapV3Error,
    models::{Pool, PoolState, SwapEvent},
};

use std::env;
use tracing::{debug, error, info};
use tracing_subscriber::{self, EnvFilter};

fn setup() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("uniswap_v3=debug"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .try_init()
        .ok();
}

fn get_test_endpoint() -> String {
    env::var("ETH_PRIMARY_URL")
        .unwrap_or_else(|_| "https://mainnet.infura.io/v3/9aa3d95b3bc440fa88ea12eaa4456161".to_string())
}

#[tokio::test]
async fn test_pool_data_collection() {
    setup();
    info!("开始测试池子数据采集");

    let collector = UniswapV3Collector::new(&get_test_endpoint()).expect("创建采集器失败");
    
    // USDC/ETH 池子
    let pool_address = "0x8ad599c3a0ff1de082011efddc58f1908eb6e6d8";
    
    match collector.get_pool(pool_address).await {
        Ok(pool) => {
            debug!("获取到池子数据: {:?}", pool);
            assert_eq!(pool.address.to_lowercase(), pool_address);
            assert!(pool.token0_price > 0.0);
            assert!(pool.token1_price > 0.0);
            assert!(pool.tvl_usd > 0.0);
        }
        Err(e) => {
            error!("获取池子数据失败: {}", e);
            panic!("测试失败");
        }
    }
}

#[tokio::test]
async fn test_liquidity_distribution() {
    setup();
    info!("开始测试流动性分布数据采集");

    let collector = UniswapV3Collector::new(&get_test_endpoint()).expect("创建采集器失败");
    
    // USDC/ETH 池子
    let pool_address = "0x8ad599c3a0ff1de082011efddc58f1908eb6e6d8";
    
    match collector.get_liquidity_distribution(pool_address).await {
        Ok(distribution) => {
            debug!("获取到流动性分布: {:?}", distribution);
            assert!(!distribution.is_empty(), "流动性分布不应为空");
            
            // 验证数据合理性
            for tick_data in distribution {
                assert!(tick_data.liquidity_net != 0, "净流动性不应为0");
                assert!(tick_data.price > 0.0, "价格应该大于0");
            }
        }
        Err(e) => {
            error!("获取流动性分布失败: {}", e);
            panic!("测试失败");
        }
    }
}

#[tokio::test]
async fn test_recent_swaps() {
    setup();
    info!("开始测试最近交易数据采集");

    let collector = UniswapV3Collector::new(&get_test_endpoint()).expect("创建采集器失败");
    
    // USDC/ETH 池子
    let pool_address = "0x8ad599c3a0ff1de082011efddc58f1908eb6e6d8";
    
    match collector.get_recent_swaps(pool_address).await {
        Ok(swaps) => {
            debug!("获取到最近交易: {:?}", swaps);
            assert!(!swaps.is_empty(), "交易列表不应为空");
            
            // 验证数据合理性
            for swap in swaps {
                assert!(swap.amount0 != 0.0 || swap.amount1 != 0.0, "至少有一个代币的交易量不为0");
                assert!(swap.timestamp > 0, "时间戳应该大于0");
                assert!(!swap.transaction_hash.is_empty(), "交易哈希不应为空");
            }
        }
        Err(e) => {
            error!("获取最近交易失败: {}", e);
            panic!("测试失败");
        }
    }
}

#[tokio::test]
async fn test_error_handling() {
    setup();
    info!("开始测试错误处理");

    let collector = UniswapV3Collector::new(&get_test_endpoint()).expect("创建采集器失败");
    
    // 使用一个无效的池子地址
    let invalid_pool = "0x0000000000000000000000000000000000000000";
    
    match collector.get_pool(invalid_pool).await {
        Ok(_) => {
            panic!("应该返回错误，但获取到了数据");
        }
        Err(e) => {
            debug!("预期的错误: {}", e);
            assert!(matches!(e, UniswapV3Error::ContractError(_)));
        }
    }
}

#[tokio::test]
async fn test_data_validation() {
    setup();
    info!("开始测试数据验证");

    let collector = UniswapV3Collector::new(&get_test_endpoint()).expect("创建采集器失败");
    
    // USDC/ETH 池子
    let pool_address = "0x8ad599c3a0ff1de082011efddc58f1908eb6e6d8";
    
    // 测试池子状态验证
    match collector.get_pool(pool_address).await {
        Ok(pool) => {
            assert!(pool.validate_state(), "池子状态验证失败");
        }
        Err(e) => {
            error!("获取池子数据失败: {}", e);
            panic!("测试失败");
        }
    }
    
    // 测试流动性分布验证
    match collector.get_liquidity_distribution(pool_address).await {
        Ok(distribution) => {
            for tick_data in distribution {
                assert!(tick_data.validate(), "Tick数据验证失败");
            }
        }
        Err(e) => {
            error!("获取流动性分布失败: {}", e);
            panic!("测试失败");
        }
    }
} 