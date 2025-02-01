use std::str::FromStr;
use ethers::{
    providers::{Provider, Http},
    types::{Address, Log, H256, U256, Bytes},
};
use rust_decimal::Decimal;
use chrono::Utc;
use mockall::predicate::*;
use mockall::mock;
use mockall::automock;
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};
use serde_json::json;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::Message;
use futures::{SinkExt, StreamExt};
use std::time::Duration;

use super::*;

// 创建Mock Provider
mock! {
    Provider {
        fn call<T: Send + Sync>(&self) -> T;
    }
}

#[tokio::test]
async fn test_new_client() {
    let provider = Provider::<Http>::try_from("http://localhost:8545")
        .expect("Failed to create provider");
    
    let client = UniswapClient::new(
        provider,
        Some("ws://localhost:8546"),
        "http://localhost:8000/graphql",
    );

    assert!(client.ws_provider.is_some());
    assert_eq!(client.graph_url, "http://localhost:8000/graphql");
}

#[tokio::test]
async fn test_parse_swap_event() {
    // 创建测试事件数据
    let mut log = Log::default();
    log.topics = vec![
        H256::from_str("0xc42079f94a6350d7e6235f29174924f928cc2ac818eb64fed8004e115fbcca67").unwrap(),
        H256::from_str("0x000000000000000000000000def1cafe000000000000000000000000000000000").unwrap(),
        H256::from_str("0x000000000000000000000000beef1234000000000000000000000000000000000").unwrap(),
    ];
    
    // 设置事件数据
    let amount0 = U256::from(1000000000000000000u128); // 1 ETH
    let amount1 = U256::from(2000000000u128); // 2000 USDC
    let sqrt_price_x96 = U256::from(1500000000u128);
    
    let mut data = vec![0u8; 128];
    amount0.to_big_endian(&mut data[0..32]);
    amount1.to_big_endian(&mut data[32..64]);
    sqrt_price_x96.to_big_endian(&mut data[64..96]);
    
    log.data = Bytes::from(data);
    log.transaction_hash = Some(H256::from_low_u64_be(1234));
    log.address = Address::from_str("0x1234567890123456789012345678901234567890").unwrap();

    // 解析事件
    let swap_data = UniswapClient::parse_swap_event(&log).unwrap();

    // 验证结果
    assert_eq!(swap_data.tx_hash, "0x00000000000000000000000000000000000000000000000000000000000004d2");
    assert_eq!(swap_data.pool, "0x1234567890123456789012345678901234567890");
    assert_eq!(swap_data.sender, "0x000000000000000000000000def1cafe000000000000000000000000000000000");
    assert_eq!(swap_data.recipient, "0x000000000000000000000000beef1234000000000000000000000000000000000");
    assert!(swap_data.amount0 > Decimal::ZERO);
    assert!(swap_data.amount1 > Decimal::ZERO);
    assert!(swap_data.price > Decimal::ZERO);
    assert_eq!(swap_data.fee, Decimal::new(3, 3)); // 0.3%
}

#[tokio::test]
async fn test_parse_swap_event_invalid_topics() {
    // 创建测试事件数据，但只有两个主题（无效）
    let mut log = Log::default();
    log.topics = vec![
        H256::from_str("0xc42079f94a6350d7e6235f29174924f928cc2ac818eb64fed8004e115fbcca67").unwrap(),
        H256::from_str("0x000000000000000000000000def1cafe000000000000000000000000000000000").unwrap(),
    ];
    
    let result = UniswapClient::parse_swap_event(&log);
    assert!(matches!(result, Err(UniswapError::EventError(_))));
}

#[tokio::test]
async fn test_parse_swap_event_invalid_data_length() {
    // 创建测试事件数据，但数据长度不足
    let mut log = Log::default();
    log.topics = vec![
        H256::from_str("0xc42079f94a6350d7e6235f29174924f928cc2ac818eb64fed8004e115fbcca67").unwrap(),
        H256::from_str("0x000000000000000000000000def1cafe000000000000000000000000000000000").unwrap(),
        H256::from_str("0x000000000000000000000000beef1234000000000000000000000000000000000").unwrap(),
    ];
    log.data = Bytes::from(vec![0u8; 64]); // 数据长度不足

    let result = UniswapClient::parse_swap_event(&log);
    assert!(matches!(result, Err(UniswapError::EventError(_))));
}

#[tokio::test]
async fn test_calculate_price_from_sqrt_x96() {
    let sqrt_price_x96 = 1500000000u128;
    let price = UniswapClient::calculate_price_from_sqrt_x96(sqrt_price_x96).unwrap();
    
    assert!(price > Decimal::ZERO);
}

#[tokio::test]
async fn test_get_pool_info() {
    // 创建模拟服务器
    let mock_server = MockServer::start().await;

    // 准备模拟响应数据
    let response_body = json!({
        "data": {
            "pool": {
                "id": "0x1234567890123456789012345678901234567890",
                "token0": {
                    "id": "0xtoken0",
                    "decimals": "18"
                },
                "token1": {
                    "id": "0xtoken1",
                    "decimals": "6"
                },
                "feeTier": "3000",
                "liquidity": "1000000",
                "token0Price": "2000.0",
                "token1Price": "0.0005",
                "totalValueLockedToken0": "100",
                "totalValueLockedToken1": "200000"
            }
        }
    });

    // 设置模拟响应
    Mock::given(method("POST"))
        .and(path("/graphql"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
        .mount(&mock_server)
        .await;

    // 创建客户端
    let provider = Provider::<Http>::try_from("http://localhost:8545").unwrap();
    let client = UniswapClient::new(
        provider,
        None,
        &mock_server.uri(),
    );

    // 执行测试
    let pool_info = client.get_pool_info("0x1234567890123456789012345678901234567890").await.unwrap();

    // 验证结果
    assert_eq!(pool_info.address, "0x1234567890123456789012345678901234567890");
    assert_eq!(pool_info.token0, "0xtoken0");
    assert_eq!(pool_info.token1, "0xtoken1");
    assert_eq!(pool_info.fee_tier, "3000");
    assert_eq!(pool_info.liquidity, "1000000");
    assert_eq!(pool_info.token0_price, "2000.0");
    assert_eq!(pool_info.token1_price, "0.0005");
    assert_eq!(pool_info.reserve0, "100");
    assert_eq!(pool_info.reserve1, "200000");
}

#[tokio::test]
async fn test_get_pool_info_not_found() {
    let mock_server = MockServer::start().await;

    // 准备"未找到"的响应数据
    let response_body = json!({
        "data": {
            "pool": null
        }
    });

    Mock::given(method("POST"))
        .and(path("/graphql"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
        .mount(&mock_server)
        .await;

    let provider = Provider::<Http>::try_from("http://localhost:8545").unwrap();
    let client = UniswapClient::new(
        provider,
        None,
        &mock_server.uri(),
    );

    let result = client.get_pool_info("0x1234567890123456789012345678901234567890").await;
    assert!(matches!(result, Err(UniswapError::GraphError(_))));
}

#[tokio::test]
async fn test_get_pool_info_server_error() {
    let mock_server = MockServer::start().await;

    // 模拟服务器错误
    Mock::given(method("POST"))
        .and(path("/graphql"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&mock_server)
        .await;

    let provider = Provider::<Http>::try_from("http://localhost:8545").unwrap();
    let client = UniswapClient::new(
        provider,
        None,
        &mock_server.uri(),
    );

    let result = client.get_pool_info("0x1234567890123456789012345678901234567890").await;
    assert!(matches!(result, Err(UniswapError::GraphError(_))));
}

#[tokio::test]
async fn test_get_price_data() {
    // 创建模拟服务器
    let mock_server = MockServer::start().await;

    // 准备模拟响应数据
    let response_body = json!({
        "data": {
            "token": {
                "id": "0xtoken0",
                "derivedETH": "0.5",
                "totalValueLocked": "1000000",
                "volume": "500000",
                "volumeUSD": "1000000",
                "priceUSD": "2000.0"
            }
        }
    });

    // 设置模拟响应
    Mock::given(method("POST"))
        .and(path("/graphql"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
        .mount(&mock_server)
        .await;

    // 创建客户端
    let provider = Provider::<Http>::try_from("http://localhost:8545").unwrap();
    let client = UniswapClient::new(
        provider,
        None,
        &mock_server.uri(),
    );

    // 执行测试
    let price_data = client.get_price_data("0xtoken0").await.unwrap();

    // 验证结果
    assert_eq!(price_data.token, "0xtoken0");
    assert_eq!(price_data.price_eth.to_string(), "0.5");
    assert_eq!(price_data.price_usd.to_string(), "2000.0");
    assert_eq!(price_data.volume_24h.to_string(), "1000000");
    assert_eq!(price_data.tvl.to_string(), "1000000");
}

#[tokio::test]
async fn test_get_price_data_invalid_response() {
    let mock_server = MockServer::start().await;

    // 准备无效的响应数据（缺少必要字段）
    let response_body = json!({
        "data": {
            "token": {
                "id": "0xtoken0",
                // 缺少其他必要字段
            }
        }
    });

    Mock::given(method("POST"))
        .and(path("/graphql"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
        .mount(&mock_server)
        .await;

    let provider = Provider::<Http>::try_from("http://localhost:8545").unwrap();
    let client = UniswapClient::new(
        provider,
        None,
        &mock_server.uri(),
    );

    let result = client.get_price_data("0xtoken0").await;
    assert!(matches!(result, Err(UniswapError::ParseError(_))));
}

#[tokio::test]
async fn test_get_liquidity_data() {
    // TODO: 添加Graph API响应的模拟测试
}

#[tokio::test]
async fn test_get_swap_data() {
    // 创建模拟服务器
    let mock_server = MockServer::start().await;

    // 准备模拟响应数据
    let response_body = json!({
        "data": {
            "swaps": [{
                "id": "0x123",
                "timestamp": "1634567890",
                "transaction": { "id": "0xabc" },
                "sender": "0xsender",
                "recipient": "0xrecipient",
                "amount0": "1.0",
                "amount1": "-2000.0",
                "amountUSD": "2000.0",
                "sqrtPriceX96": "1500000000"
            }]
        }
    });

    // 设置模拟响应
    Mock::given(method("POST"))
        .and(path("/graphql"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
        .mount(&mock_server)
        .await;

    // 创建客户端
    let provider = Provider::<Http>::try_from("http://localhost:8545").unwrap();
    let client = UniswapClient::new(
        provider,
        None,
        &mock_server.uri(),
    );

    // 执行测试
    let swap_data = client.get_swap_data("0x1234567890123456789012345678901234567890").await.unwrap();

    // 验证结果
    assert_eq!(swap_data.len(), 1);
    let swap = &swap_data[0];
    assert_eq!(swap.tx_hash, "0xabc");
    assert_eq!(swap.sender, "0xsender");
    assert_eq!(swap.recipient, "0xrecipient");
    assert_eq!(swap.amount0.to_string(), "1");
    assert_eq!(swap.amount1.to_string(), "-2000");
    assert!(swap.price > Decimal::ZERO);
}

/// 创建模拟的WebSocket服务器
async fn create_mock_ws_server() -> (String, SocketAddr) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let ws_url = format!("ws://{}", addr);

    tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            let ws_stream = tokio_tungstenite::accept_async(stream).await.unwrap();
            let (mut write, mut read) = ws_stream.split();

            // 处理订阅请求
            while let Some(Ok(msg)) = read.next().await {
                if msg.is_text() {
                    // 发送模拟的事件数据
                    let event_data = r#"{
                        "jsonrpc": "2.0",
                        "method": "eth_subscription",
                        "params": {
                            "subscription": "0x123",
                            "result": {
                                "address": "0x1234567890123456789012345678901234567890",
                                "topics": [
                                    "0xc42079f94a6350d7e6235f29174924f928cc2ac818eb64fed8004e115fbcca67",
                                    "0x000000000000000000000000def1cafe000000000000000000000000000000000",
                                    "0x000000000000000000000000beef1234000000000000000000000000000000000"
                                ],
                                "data": "0x000000000000000000000000000000000000000000000000000de0b6b3a7640000000000000000000000000000000000000000000000000000000000000000007700000000000000000000000000000000000000000000000000000000000059d8",
                                "blockNumber": "0x123",
                                "transactionHash": "0xabc123"
                            }
                        }
                    }"#;
                    write.send(Message::Text(event_data.to_string())).await.unwrap();
                }
            }
        }
    });

    (ws_url, addr)
}

#[tokio::test]
async fn test_subscribe_events() {
    // 创建模拟的WebSocket服务器
    let (ws_url, _addr) = create_mock_ws_server().await;

    // 等待服务器启动
    tokio::time::sleep(Duration::from_millis(100)).await;

    // 创建客户端
    let provider = Provider::<Http>::try_from("http://localhost:8545").unwrap();
    let client = UniswapClient::new(
        provider,
        Some(&ws_url),
        "http://localhost:8000/graphql",
    );

    // 订阅事件
    let mut event_receiver = client.subscribe_events("0x1234567890123456789012345678901234567890")
        .await
        .unwrap();

    // 接收事件数据
    let timeout = tokio::time::sleep(Duration::from_secs(2));
    tokio::pin!(timeout);

    tokio::select! {
        Some(swap_data) = event_receiver.recv() => {
            // 验证接收到的事件数据
            assert_eq!(swap_data.pool, "0x1234567890123456789012345678901234567890");
            assert!(swap_data.amount0 > Decimal::ZERO);
            assert!(swap_data.amount1 > Decimal::ZERO);
            assert!(swap_data.price > Decimal::ZERO);
        }
        _ = &mut timeout => {
            panic!("Timeout waiting for event");
        }
    }
}

#[tokio::test]
async fn test_subscribe_events_connection_error() {
    // 使用一个不存在的WebSocket地址
    let provider = Provider::<Http>::try_from("http://localhost:8545").unwrap();
    let client = UniswapClient::new(
        provider,
        Some("ws://localhost:12345"),
        "http://localhost:8000/graphql",
    );

    let result = client.subscribe_events("0x1234567890123456789012345678901234567890").await;
    assert!(matches!(result, Err(UniswapError::ConnectionError(_))));
}

#[tokio::test]
async fn test_subscribe_events_invalid_pool_address() {
    // 创建模拟的WebSocket服务器
    let (ws_url, _addr) = create_mock_ws_server().await;

    // 等待服务器启动
    tokio::time::sleep(Duration::from_millis(100)).await;

    // 创建客户端
    let provider = Provider::<Http>::try_from("http://localhost:8545").unwrap();
    let client = UniswapClient::new(
        provider,
        Some(&ws_url),
        "http://localhost:8000/graphql",
    );

    // 使用无效的池子地址
    let result = client.subscribe_events("invalid_address").await;
    assert!(matches!(result, Err(UniswapError::ParseError(_))));
}

#[tokio::test]
async fn test_calculate_price_from_sqrt_x96_overflow() {
    let sqrt_price_x96 = u128::MAX;
    let result = UniswapClient::calculate_price_from_sqrt_x96(sqrt_price_x96);
    assert!(matches!(result, Err(UniswapError::ParseError(_))));
}

#[tokio::test]
async fn test_get_swap_data_invalid_amounts() {
    let mock_server = MockServer::start().await;

    // 准备包含无效金额的响应数据
    let response_body = json!({
        "data": {
            "swaps": [{
                "id": "0x123",
                "timestamp": "1634567890",
                "transaction": { "id": "0xabc" },
                "sender": "0xsender",
                "recipient": "0xrecipient",
                "amount0": "invalid",  // 无效的金额
                "amount1": "-2000.0",
                "amountUSD": "2000.0",
                "sqrtPriceX96": "1500000000"
            }]
        }
    });

    Mock::given(method("POST"))
        .and(path("/graphql"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
        .mount(&mock_server)
        .await;

    let provider = Provider::<Http>::try_from("http://localhost:8545").unwrap();
    let client = UniswapClient::new(
        provider,
        None,
        &mock_server.uri(),
    );

    let result = client.get_swap_data("0x1234567890123456789012345678901234567890").await;
    assert!(matches!(result, Err(UniswapError::ParseError(_))));
} 