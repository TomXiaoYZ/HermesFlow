use chrono::Utc;
use rust_decimal::Decimal;
use common::{MarketData, DataQuality, Trade};

use super::*;

#[test]
fn test_process_pool_info() {
    let mut processor = PancakeSwapProcessor::new();
    let pool_info = PoolInfo {
        address: "0x1234567890".to_string(),
        token0: "0xtoken0".to_string(),
        token1: "0xtoken1".to_string(),
        fee_tier: "25".to_string(),
        liquidity: "1000000".to_string(),
        token0_price: "2000.0".to_string(),
        token1_price: "0.0005".to_string(),
        reserve0: "100".to_string(),
        reserve1: "200000".to_string(),
    };

    let result = processor.process_pool_info(&pool_info).unwrap();
    match result {
        MarketData::Custom { data_type, symbol, quality, .. } => {
            assert_eq!(data_type, "pool_info");
            assert_eq!(symbol, "0xtoken0-0xtoken1");
            assert_eq!(quality, DataQuality::Real);
        }
        _ => panic!("Expected Custom market data"),
    }

    // 验证缓存更新
    assert!(processor.pool_info_cache.contains_key(&pool_info.address));
}

#[test]
fn test_process_price_data() {
    let mut processor = PancakeSwapProcessor::new();
    let price_data = PriceData {
        token: "0xtoken0".to_string(),
        price_bnb: Decimal::new(5, 1), // 0.5 BNB
        price_usd: Decimal::new(2000, 0), // 2000 USD
        price_change_24h: Decimal::new(0, 0),
        volume_24h: Decimal::new(1000000, 0),
        tvl: Decimal::new(5000000, 0),
        timestamp: Utc::now(),
    };

    let result = processor.process_price_data(&price_data).unwrap();
    match result {
        MarketData::Custom { data_type, symbol, quality, .. } => {
            assert_eq!(data_type, "price_data");
            assert_eq!(symbol, "0xtoken0");
            assert_eq!(quality, DataQuality::Real);
        }
        _ => panic!("Expected Custom market data"),
    }

    // 验证缓存更新
    assert!(processor.price_cache.contains_key(&price_data.token));
}

#[test]
fn test_process_swap_data() {
    let mut processor = PancakeSwapProcessor::new();
    
    // 首先添加池子信息到缓存
    let pool_info = PoolInfo {
        address: "0x1234567890".to_string(),
        token0: "0xtoken0".to_string(),
        token1: "0xtoken1".to_string(),
        fee_tier: "25".to_string(),
        liquidity: "1000000".to_string(),
        token0_price: "2000.0".to_string(),
        token1_price: "0.0005".to_string(),
        reserve0: "100".to_string(),
        reserve1: "200000".to_string(),
    };
    processor.pool_info_cache.insert(pool_info.address.clone(), pool_info);

    let swap_data = SwapData {
        tx_hash: "0xabc123".to_string(),
        pool: "0x1234567890".to_string(),
        sender: "0xsender".to_string(),
        recipient: "0xrecipient".to_string(),
        amount0: Decimal::new(1, 0),
        amount1: Decimal::new(-2000, 0),
        price: Decimal::new(2000, 0),
        fee: Decimal::new(25, 4), // 0.25%
        timestamp: Utc::now(),
    };

    let result = processor.process_swap_data(&swap_data).unwrap();
    match result {
        MarketData::Trade(trade) => {
            assert_eq!(trade.symbol, "0xtoken0-0xtoken1");
            assert_eq!(trade.id, "0xabc123");
            assert_eq!(trade.price, Decimal::new(2000, 0));
            assert_eq!(trade.quantity, Decimal::new(1, 0));
            assert_eq!(trade.quality, DataQuality::Real);
            assert!(trade.is_buyer_maker);
        }
        _ => panic!("Expected Trade market data"),
    }
}

#[test]
fn test_process_farm_data() {
    let processor = PancakeSwapProcessor::new();
    let farm_data = FarmData {
        farm_id: "farm1".to_string(),
        lp_token: "0xlp".to_string(),
        reward_token: "0xcake".to_string(),
        apr: Decimal::new(1000, 1), // 100.0%
        total_staked: Decimal::new(1000000, 0),
        reward_per_block: Decimal::new(100, 0),
        timestamp: Utc::now(),
    };

    let result = processor.process_farm_data(&farm_data).unwrap();
    match result {
        MarketData::Custom { data_type, symbol, quality, .. } => {
            assert_eq!(data_type, "farm_data");
            assert_eq!(symbol, "0xlp-0xcake");
            assert_eq!(quality, DataQuality::Real);
        }
        _ => panic!("Expected Custom market data"),
    }
}

#[test]
fn test_process_prediction_data() {
    let processor = PancakeSwapProcessor::new();
    let prediction_data = PredictionData {
        round_id: 123,
        start_price: Decimal::new(300, 0),
        end_price: Decimal::new(310, 0),
        bull_amount: Decimal::new(5000, 0),
        bear_amount: Decimal::new(4000, 0),
        start_time: Utc::now(),
        end_time: Utc::now(),
    };

    let result = processor.process_prediction_data(&prediction_data).unwrap();
    match result {
        MarketData::Custom { data_type, symbol, quality, .. } => {
            assert_eq!(data_type, "prediction_data");
            assert_eq!(symbol, "PREDICTION-123");
            assert_eq!(quality, DataQuality::Real);
        }
        _ => panic!("Expected Custom market data"),
    }
}

#[test]
fn test_process_swap_data_without_pool_info() {
    let processor = PancakeSwapProcessor::new();
    let swap_data = SwapData {
        tx_hash: "0xabc123".to_string(),
        pool: "0x1234567890".to_string(), // 不存在的池子
        sender: "0xsender".to_string(),
        recipient: "0xrecipient".to_string(),
        amount0: Decimal::new(1, 0),
        amount1: Decimal::new(-2000, 0),
        price: Decimal::new(2000, 0),
        fee: Decimal::new(25, 4),
        timestamp: Utc::now(),
    };

    let result = processor.process_swap_data(&swap_data);
    assert!(matches!(result, Err(PancakeSwapError::ProcessError(_))));
}

#[test]
fn test_process_swap_data_batch() {
    let mut processor = PancakeSwapProcessor::new();
    
    // 添加池子信息到缓存
    let pool_info = PoolInfo {
        address: "0x1234567890".to_string(),
        token0: "0xtoken0".to_string(),
        token1: "0xtoken1".to_string(),
        fee_tier: "25".to_string(),
        liquidity: "1000000".to_string(),
        token0_price: "2000.0".to_string(),
        token1_price: "0.0005".to_string(),
        reserve0: "100".to_string(),
        reserve1: "200000".to_string(),
    };
    processor.pool_info_cache.insert(pool_info.address.clone(), pool_info);

    let swap_data = vec![
        SwapData {
            tx_hash: "0xabc123".to_string(),
            pool: "0x1234567890".to_string(),
            sender: "0xsender".to_string(),
            recipient: "0xrecipient".to_string(),
            amount0: Decimal::new(1, 0),
            amount1: Decimal::new(-2000, 0),
            price: Decimal::new(2000, 0),
            fee: Decimal::new(25, 4),
            timestamp: Utc::now(),
        },
        SwapData {
            tx_hash: "0xdef456".to_string(),
            pool: "0x1234567890".to_string(),
            sender: "0xsender".to_string(),
            recipient: "0xrecipient".to_string(),
            amount0: Decimal::new(-1, 0),
            amount1: Decimal::new(2000, 0),
            price: Decimal::new(2000, 0),
            fee: Decimal::new(25, 4),
            timestamp: Utc::now(),
        },
    ];

    let results = processor.process_swap_data_batch(&swap_data).unwrap();
    assert_eq!(results.len(), 2);
    
    for result in results {
        match result {
            MarketData::Trade(trade) => {
                assert_eq!(trade.symbol, "0xtoken0-0xtoken1");
                assert_eq!(trade.price, Decimal::new(2000, 0));
                assert_eq!(trade.quantity.abs(), Decimal::new(1, 0));
                assert_eq!(trade.quality, DataQuality::Real);
            }
            _ => panic!("Expected Trade market data"),
        }
    }
} 