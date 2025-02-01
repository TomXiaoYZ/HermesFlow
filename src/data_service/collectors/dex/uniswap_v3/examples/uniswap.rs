use uniswap_v3::collector::UniswapV3Collector;
use std::time::Duration;
use tokio::time::sleep;
use chrono::Utc;

#[tokio::main]
async fn main() {
    // 创建采集器实例
    let collector = UniswapV3Collector::new("https://eth.meowrpc.com")
        .expect("Failed to create collector");

    // USDC/ETH 池子地址
    let pool_address = "0x8ad599c3a0ff1de082011efddc58f1908eb6e6d8";

    println!("开始监控 Uniswap V3 USDC/ETH 池子...");
    println!("池子地址: {}", pool_address);

    loop {
        let now = Utc::now();
        println!("\n时间: {}", now.format("%Y-%m-%d %H:%M:%S UTC"));

        // 获取池子信息
        match collector.get_pool(pool_address).await {
            Ok(pool_data) => {
                println!("\n池子信息:");
                println!("ETH/USDC 价格: {:.2} USDC", pool_data.price);
                println!("当前 Tick: {}", pool_data.tick);
                println!("流动性: {}", pool_data.liquidity);
                println!("Token0 (USDC): {}", pool_data.pool.token0);
                println!("Token1 (ETH): {}", pool_data.pool.token1);
                println!("手续费率: {}%", pool_data.pool.fee as f64 / 10000.0);
            }
            Err(e) => println!("获取池子信息失败: {}", e),
        }

        // 获取最近的交易
        match collector.get_recent_swaps(pool_address).await {
            Ok(swaps) => {
                println!("\n最近的交易:");
                for swap in swaps {
                    if let uniswap_v3::models::PoolEvent::Swap {
                        sender,
                        recipient,
                        amount0,
                        amount1,
                        tick,
                        ..
                    } = swap {
                        println!("---");
                        println!("发送者: {}", sender);
                        println!("接收者: {}", recipient);
                        if amount0 > rust_decimal::Decimal::ZERO {
                            println!("卖出 {:.2} USDC", amount0);
                            println!("买入 {:.6} ETH", -amount1);
                        } else {
                            println!("卖出 {:.6} ETH", amount1);
                            println!("买入 {:.2} USDC", -amount0);
                        }
                        println!("Tick: {}", tick);
                    }
                }
            }
            Err(e) => println!("获取交易记录失败: {}", e),
        }

        // 获取流动性分布
        match collector.get_liquidity_distribution(pool_address).await {
            Ok(ticks) => {
                println!("\n流动性分布:");
                for tick in ticks {
                    println!("---");
                    println!("Tick: {}", tick.tick_idx);
                    println!("价格: {:.2} USDC/ETH", tick.price0);
                    println!("总流动性: {}", tick.liquidity_gross);
                    println!("净流动性: {}", tick.liquidity_net);
                }
            }
            Err(e) => println!("获取流动性分布失败: {}", e),
        }

        // 等待10秒后继续
        println!("\n等待10秒...\n");
        sleep(Duration::from_secs(10)).await;
    }
} 