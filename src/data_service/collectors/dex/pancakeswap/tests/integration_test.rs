use std::env;
use ethers::providers::{Provider, Http};
use pancakeswap::{PancakeSwapClient, PancakeSwapProcessor};
use tokio::time::Duration;

// PancakeSwap V2 主要池子地址
const CAKE_BNB_POOL: &str = "0x0eD7e52944161450477ee417DE9Cd3a859b14fD0";
const BUSD_BNB_POOL: &str = "0x58F876857a02D6762E0101bb5C46A8c1ED44Dc16";
const USDT_BNB_POOL: &str = "0x16b9a82891338f9bA80E2D6970FddA79D1eb0daE";

// Token 地址
const CAKE_TOKEN: &str = "0x0E09FaBB73Bd3Ade0a17ECC321fD13a19e81cE82";
const BUSD_TOKEN: &str = "0xe9e7CEA3DedcA5984780Bafc599bD69ADd087D56";

// 农场ID
const CAKE_BNB_FARM: &str = "1";

async fn create_test_client() -> PancakeSwapClient {
    let bsc_rpc = env::var("BSC_RPC_URL")
        .unwrap_or_else(|_| "https://bsc-dataseed1.binance.org".to_string());
    let bsc_ws = env::var("BSC_WS_URL")
        .unwrap_or_else(|_| "wss://bsc-ws-node.nariox.org:443".to_string());
    let graph_url = env::var("PANCAKESWAP_GRAPH_URL")
        .unwrap_or_else(|_| "https://api.thegraph.com/subgraphs/name/pancakeswap/exchange-v2".to_string());

    let provider = Provider::<Http>::try_from(bsc_rpc)
        .expect("Failed to create provider");

    PancakeSwapClient::new(
        provider,
        Some(&bsc_ws),
        &graph_url,
    )
}

#[tokio::test]
async fn test_get_pool_info() {
    let client = create_test_client().await;
    let processor = PancakeSwapProcessor::new();

    let pool_info = client.get_pool_info(CAKE_BNB_POOL).await.unwrap();
    
    assert_eq!(pool_info.address.to_lowercase(), CAKE_BNB_POOL.to_lowercase());
    assert!(pool_info.liquidity.parse::<f64>().unwrap() > 0.0);
    assert!(pool_info.token0_price.parse::<f64>().unwrap() > 0.0);
    assert!(pool_info.token1_price.parse::<f64>().unwrap() > 0.0);
}

#[tokio::test]
async fn test_get_price_data() {
    let client = create_test_client().await;
    let processor = PancakeSwapProcessor::new();

    let price_data = client.get_price_data(CAKE_TOKEN).await.unwrap();
    
    assert_eq!(price_data.token.to_lowercase(), CAKE_TOKEN.to_lowercase());
    assert!(price_data.price_bnb > rust_decimal::Decimal::ZERO);
    assert!(price_data.price_usd > rust_decimal::Decimal::ZERO);
    assert!(price_data.volume_24h > rust_decimal::Decimal::ZERO);
}

#[tokio::test]
async fn test_get_farm_data() {
    let client = create_test_client().await;
    let processor = PancakeSwapProcessor::new();

    let farm_data = client.get_farm_data(CAKE_BNB_FARM).await.unwrap();
    
    assert_eq!(farm_data.farm_id, CAKE_BNB_FARM);
    assert!(farm_data.apr > rust_decimal::Decimal::ZERO);
    assert!(farm_data.total_staked > rust_decimal::Decimal::ZERO);
    assert!(farm_data.reward_per_block > rust_decimal::Decimal::ZERO);
}

#[tokio::test]
async fn test_subscribe_events() {
    let client = create_test_client().await;
    let mut processor = PancakeSwapProcessor::new();

    // 首先获取池子信息
    let pool_info = client.get_pool_info(BUSD_BNB_POOL).await.unwrap();
    processor.process_pool_info(&pool_info).unwrap();

    // 订阅交易事件
    let mut event_receiver = client.subscribe_events(BUSD_BNB_POOL).await.unwrap();

    // 等待接收事件（设置超时时间为30秒）
    let timeout = tokio::time::sleep(Duration::from_secs(30));
    tokio::pin!(timeout);

    tokio::select! {
        Some(swap_data) = event_receiver.recv() => {
            // 处理交易数据
            let market_data = processor.process_swap_data(&swap_data).unwrap();
            match market_data {
                common::MarketData::Trade(trade) => {
                    assert!(!trade.id.is_empty());
                    assert!(trade.price > rust_decimal::Decimal::ZERO);
                    assert!(trade.quantity > rust_decimal::Decimal::ZERO);
                }
                _ => panic!("Expected Trade market data"),
            }
        }
        _ = &mut timeout => {
            println!("No swap events received within timeout period");
        }
    }
}

#[tokio::test]
async fn test_multi_pool_monitoring() {
    let client = create_test_client().await;
    let mut processor = PancakeSwapProcessor::new();

    // 监控多个主要池子
    let pools = vec![CAKE_BNB_POOL, BUSD_BNB_POOL, USDT_BNB_POOL];
    
    for pool in &pools {
        // 获取并处理池子信息
        let pool_info = client.get_pool_info(pool).await.unwrap();
        let market_data = processor.process_pool_info(&pool_info).unwrap();
        
        match market_data {
            common::MarketData::Custom { data_type, quality, .. } => {
                assert_eq!(data_type, "pool_info");
                assert_eq!(quality, common::DataQuality::Real);
            }
            _ => panic!("Expected Custom market data"),
        }
    }
}

#[tokio::test]
async fn test_price_update_frequency() {
    let client = create_test_client().await;
    let mut processor = PancakeSwapProcessor::new();

    // 测试价格更新频率
    let token = CAKE_TOKEN;
    let mut last_price = rust_decimal::Decimal::ZERO;

    for _ in 0..3 {
        let price_data = client.get_price_data(token).await.unwrap();
        let current_price = price_data.price_usd;
        
        if last_price > rust_decimal::Decimal::ZERO {
            // 验证价格是否发生变化
            assert!(current_price != last_price, "Price should update frequently");
        }
        
        last_price = current_price;
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

#[tokio::test]
async fn test_error_handling() {
    let client = create_test_client().await;

    // 测试无效池子地址
    let result = client.get_pool_info("0xinvalid").await;
    assert!(result.is_err());

    // 测试无效代币地址
    let result = client.get_price_data("0xinvalid").await;
    assert!(result.is_err());

    // 测试无效农场ID
    let result = client.get_farm_data("999999").await;
    assert!(result.is_err());
} 