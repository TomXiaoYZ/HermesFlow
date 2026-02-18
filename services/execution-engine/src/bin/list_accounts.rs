use ibapi::accounts::PositionUpdate;
use ibapi::client::sync::Client;
use std::env;

fn main() {
    let host = env::var("IBKR_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("IBKR_PORT").unwrap_or_else(|_| "4002".to_string());
    let addr = format!("{}:{}", host, port);

    println!("Connecting to IBKR at {}...", addr);
    let client = Client::connect(&addr, 99).expect("Failed to connect to IBKR");

    match client.managed_accounts() {
        Ok(accounts) => println!("Managed accounts: {:?}", accounts),
        Err(e) => eprintln!("Failed to get managed accounts: {}", e),
    }

    match client.positions() {
        Ok(subscription) => {
            while let Some(update) = subscription.next() {
                match update {
                    PositionUpdate::Position(pos) => {
                        println!(
                            "  account={} symbol={} qty={} avg_cost={}",
                            pos.account, pos.contract.symbol, pos.position, pos.average_cost
                        );
                    }
                    PositionUpdate::PositionEnd => {
                        subscription.cancel();
                        break;
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to fetch positions: {}", e);
        }
    }
}
