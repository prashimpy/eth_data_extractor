use clap::{Parser, Subcommand};
use color_eyre::Result;
use alloy_primitives::{Address, B256, U256};
use std::str::FromStr;

mod explorer;
mod utils;
mod rpc;

use explorer::BlockExplorer;

#[derive(Parser)]
#[command(name = "eth_data_extractor")]
#[command(about = "A CLI Ethereum data extraction tool built with Reth")]
struct Cli {
    /// RPC URL for the Ethereum node
    #[arg(short, long, default_value = "http://localhost:8545")]
    rpc_url: String,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Get information about a block
    Block {
        /// Block number or hash
        block_id: String,
    },
    /// Get transaction details
    Transaction {
        /// Transaction hash
        tx_hash: String,
    },
    /// Get account balance and info
    Account {
        /// Account address
        address: String,
        /// Optional block number (default: latest)
        #[arg(short, long)]
        block: Option<u64>,
    },
    /// Get latest blocks
    Latest {
        /// Number of blocks to show (default: 10)
        #[arg(short, long, default_value = "10")]
        count: usize,
    },
    /// Show gas statistics for recent blocks
    Gas {
        /// Number of blocks to analyze (default: 100)
        #[arg(short, long, default_value = "100")]
        blocks: usize,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    
    let cli = Cli::parse();
    let explorer = BlockExplorer::new(&cli.rpc_url).await?;
    
    match cli.command {
        Commands::Block { block_id } => {
            explorer.show_block(&block_id).await?;
        }
        Commands::Transaction { tx_hash } => {
            explorer.show_transaction(&tx_hash).await?;
        }
        Commands::Account { address, block } => {
            explorer.show_account(&address, block).await?;
        }
        Commands::Latest { count } => {
            explorer.show_latest_blocks(count).await?;
        }
        Commands::Gas { blocks } => {
            explorer.show_gas_statistics(blocks).await?;
        }
    }
    
    Ok(())
}
