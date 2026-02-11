use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use std::env;

fn main() {
    let key_str =
        "3MQhJGzdzKuxeVe7VLVp3VGpkhMC7iJHnNk4hUP4ZtfP7gXHhJBezbwb1KuhMkg64J8XDZah4jjxHAfy8iUoe8Sf";
    println!("Testing Key: {}", key_str);

    match bs58::decode(key_str).into_vec() {
        Ok(decoded) => {
            println!("Decoded {} bytes", decoded.len());
            match Keypair::from_bytes(&decoded) {
                Ok(kp) => println!("Valid Keypair! Pubkey: {}", kp.pubkey()),
                Err(e) => println!("Invalid Keypair bytes: {}", e),
            }
        }
        Err(e) => println!("Base58 decode error: {}", e),
    }
}
