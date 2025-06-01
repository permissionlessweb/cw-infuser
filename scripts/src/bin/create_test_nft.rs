use clap::Parser;
use cw_infuser_scripts::{ELGAFAR_1, STARGAZE_1};
use cw_orch::prelude::*;

#[derive(Parser, Debug)]
#[command(version, about = "Create test NFT collections for testing infusions")]
struct Args {
    /// Network to deploy on
    #[arg(short, long, default_value = "testnet")]
    network: String,
    /// Collection name
    #[arg(long, default_value = "Test Collection")]
    name: String,
    /// Collection symbol
    #[arg(long, default_value = "TEST")]
    symbol: String,
    /// Number of tokens to mint
    #[arg(long, default_value = "10")]
    num_tokens: u32,
}

pub fn main() -> anyhow::Result<()> {
    dotenv::dotenv()?;
    env_logger::init();
    let args = Args::parse();

    let network = match args.network.as_str() {
        "testnet" => ELGAFAR_1,
        "mainnet" => STARGAZE_1,
        _ => anyhow::bail!("Invalid network. Use 'testnet' or 'mainnet'"),
    };

    let chain = Daemon::builder(network.clone()).build()?;

    println!("Creating test NFT collection on {}", network.chain_id);
    println!("Name: {}", args.name);
    println!("Symbol: {}", args.symbol);
    println!("Tokens to mint: {}", args.num_tokens);
    println!("Deployer: {}", chain.sender_addr());

    // Note: This would require implementing a CW721 contract interface
    // For now, this serves as a placeholder for actual NFT collection creation
    println!("Test NFT creation functionality placeholder - implement CW721 interface as needed");

    Ok(())
}
