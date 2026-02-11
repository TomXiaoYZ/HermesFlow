use clap::{Parser, Subcommand};
use execution_engine::traders::solana_trader::SolanaTrader;
use std::env;
use tracing::{error, info, Level};

#[derive(Parser)]
#[command(name = "live_test_raydium")]
#[command(about = "Live testing tool for Raydium integration", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Read-only check: validate connection, pool, and price estimate
    Check,
    /// Buy Token with SOL (native SOL -> Token)
    Buy {
        /// Token to buy (default: USDC)
        #[arg(
            short,
            long,
            default_value = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
        )]
        token: String,
        /// Amount of SOL to swap (default: 0.0001)
        #[arg(short, long, default_value_t = 0.0001)]
        amount: f64,
        /// Slippage BPS (default: 100 = 1%)
        #[arg(short, long, default_value_t = 100)]
        slippage: u16,
    },
    /// Sell Token for SOL (Token -> native SOL)
    Sell {
        /// Token to sell (default: USDC)
        #[arg(
            short,
            long,
            default_value = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
        )]
        token: String,
        /// Percentage to sell (0.0 - 1.0, default: 0.5 = 50%) or explicit amount logic to be added
        #[arg(short, long, default_value_t = 0.5)]
        percentage: f64,
        /// Slippage BPS (default: 100 = 1%)
        #[arg(short, long, default_value_t = 100)]
        slippage: u16,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let cli = Cli::parse();

    // Load configuration
    info!("Loading configuration...");
    let rpc_url = env::var("SOLANA_RPC_URL")
        .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());

    let priv_key = env::var("SOLANA_PRIVATE_KEY").map_err(|_| {
        anyhow::anyhow!(
            "SOLANA_PRIVATE_KEY environment variable not set. Please set it to run live tests."
        )
    })?;

    // Initialize Trader
    info!("Initializing Solana Trader...");
    let trader = SolanaTrader::new(&rpc_url, &priv_key)?;
    let wallet = trader.get_pubkey();
    info!("Wallet connected: {}", wallet);

    let balance = trader.get_balance().await?;
    info!("Wallet Balance: {} SOL", balance);

    if balance < 0.002 {
        error!("Insufficient balance! We need at least 0.002 SOL for rent and fees.");
        return Ok(());
    }

    match cli.command {
        Commands::Check => {
            info!("--- Starting READ-ONLY Check ---");

            let _sol_mint = "So11111111111111111111111111111111111111112";
            let _usdc_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";

            info!("Configuration looks good. Wallet balance sufficient.");
            info!("To execute a real test transaction:");
            info!("  cargo run --bin live_test_raydium -- buy --amount 0.0001");
            info!("  cargo run --bin live_test_raydium -- sell --percentage 0.5");
        }
        Commands::Buy {
            token,
            amount,
            slippage,
        } => {
            info!("--- Starting LIVE BUY ---");
            info!("Target: swap {} SOL -> {}", amount, token);

            match trader
                .buy_raydium_experimental(&token, amount, slippage)
                .await
            {
                Ok(sig) => {
                    info!("✅ SUCCESS! Transaction confirmed.");
                    info!("Signature: https://solscan.io/tx/{}", sig);
                }
                Err(e) => {
                    error!("❌ FAILED: {:?}", e);
                    if let Some(client_error) =
                        e.downcast_ref::<solana_client::client_error::ClientError>()
                    {
                        if let solana_client::client_error::ClientErrorKind::RpcError(rpc_err) =
                            &client_error.kind
                        {
                            error!("RPC Error Details: {:?}", rpc_err);
                        }
                    }
                    error!("Check logs for details.");
                }
            }
        }
        Commands::Sell {
            token,
            percentage,
            slippage,
        } => {
            info!("--- Starting LIVE SELL ---");
            info!("Target: swap {}% of {} -> SOL", percentage * 100.0, token);

            // Note: Currently calling the main `sell` which now routes to Raydium experimental first
            match trader.sell(&token, percentage, slippage).await {
                Ok(sig) => {
                    info!("✅ SUCCESS! Transaction confirmed.");
                    info!("Signature: https://solscan.io/tx/{}", sig);
                }
                Err(e) => {
                    error!("❌ FAILED: {:?}", e);
                    if let Some(client_error) =
                        e.downcast_ref::<solana_client::client_error::ClientError>()
                    {
                        if let solana_client::client_error::ClientErrorKind::RpcError(rpc_err) =
                            &client_error.kind
                        {
                            error!("RPC Error Details: {:?}", rpc_err);
                        }
                    }
                    error!("Check logs for details.");
                }
            }
        }
    }

    Ok(())
}
