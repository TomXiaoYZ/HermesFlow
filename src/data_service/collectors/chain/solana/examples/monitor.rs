use std::sync::Arc;
use tokio::sync::broadcast;
use solana::{
    config::SolConfig,
    models::ChainEvent,
    websocket::WebsocketClient,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 加载环境变量
    dotenv::dotenv().ok();

    // 创建配置
    let config = SolConfig::from_env()?;
    println!("使用 RPC 端点: {}", config.primary_url);

    // 创建事件通道
    let (tx, mut rx) = broadcast::channel(100);

    // 创建 WebSocket 客户端
    let client = WebsocketClient::new(Arc::new(config), tx)?;

    // 启动订阅
    client.start().await?;
    println!("已启动订阅");

    // 处理接收到的事件
    while let Ok(event) = rx.recv().await {
        match event {
            ChainEvent::SlotUpdate { slot, parent, status } => {
                println!("新区块槽位: {} (父槽位: {:?}, 状态: {})", slot, parent, status);
            }
            ChainEvent::NewTransaction(tx) => {
                println!("新交易: {} 在槽位 {}", tx.signature, tx.slot);
                if let Some(error) = tx.error {
                    println!("交易错误: {}", error);
                }
                match tx.status {
                    Ok(()) => println!("交易状态: 成功"),
                    Err(e) => println!("交易状态: 失败 - {}", e),
                }
            }
            ChainEvent::NewBlock(block) => {
                println!(
                    "新区块在槽位 {} 包含 {} 笔交易",
                    block.slot,
                    block.transactions.len()
                );
            }
            ChainEvent::AccountUpdate { slot, account } => {
                println!(
                    "账户更新在槽位 {}: {} (所有者: {})",
                    slot, account.pubkey, account.owner
                );
            }
            ChainEvent::ProgramUpdate { slot, program } => {
                println!(
                    "程序更新在槽位 {}: {}",
                    slot, program.program_id
                );
            }
        }
    }

    Ok(())
} 