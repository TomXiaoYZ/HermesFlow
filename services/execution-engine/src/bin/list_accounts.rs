use ibapi::Client;
use std::env;

fn main() {
    let host = env::var("IBKR_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("IBKR_PORT").unwrap_or_else(|_| "4002".to_string());
    let addr = format!("{}:{}", host, port);

    println!("Connecting to IBKR at {}...", addr);
    let client = Client::connect(&addr, 99).expect("Failed to connect to IBKR");
    println!("Managed accounts: {}", client.managed_accounts());

    // Collect positions into a Vec to avoid lifetime issues with the iterator
    let positions: Vec<_> = match client.positions() {
        Ok(iter) => iter.collect(),
        Err(e) => {
            eprintln!("Failed to fetch positions: {}", e);
            vec![]
        }
    };

    for pos in &positions {
        println!(
            "  account={} symbol={} qty={} avg_cost={}",
            pos.account, pos.contract.symbol, pos.position, pos.average_cost
        );
    }
}
