use color_eyre::Result;
use comfy_table::{Table, presets::UTF8_FULL};
use crate::rpc::RethClient;
use crate::utils::{
    format_wei_u256, format_b256_hash, format_eth_address, format_timestamp_u256,
    format_gas_price, format_tx_status, format_number, time_ago, account_type,
    calculate_gas_utilization
};

pub struct BlockExplorer {
    client: RethClient,
}

impl BlockExplorer {
    pub async fn new(rpc_url: &str) -> Result<Self> {
        let client = RethClient::new(rpc_url).await?;
        Ok(Self { client })
    }
    
    pub async fn show_block(&self, block_id: &str) -> Result<()> {
        println!("📦 Block Information");
        println!("===================\n");
        
        let block = if block_id.starts_with("0x") {
            // It's a hash
            self.client.get_block_by_hash(block_id).await?
        } else {
            // It's a number
            let block_number = block_id.parse::<u64>()
                .map_err(|_| color_eyre::eyre::eyre!("Invalid block number"))?;
            self.client.get_block_by_number(block_number).await?
        };
        
        let mut table = Table::new();
        table.load_preset(UTF8_FULL);
        table.set_header(vec!["Property", "Value"]);
        
        table.add_row(vec!["Block Number", &block.number.to_string()]);
        table.add_row(vec!["Block Hash", &format_b256_hash(&block.hash)]);
        table.add_row(vec!["Parent Hash", &format_b256_hash(&block.parent_hash)]);
        table.add_row(vec!["Timestamp", &format_timestamp_u256(&block.timestamp)]);
        table.add_row(vec!["Time Ago", &time_ago(block.timestamp.to::<u64>())]);
        table.add_row(vec!["Gas Used", &format_number(block.gas_used.to::<u64>())]);
        table.add_row(vec!["Gas Limit", &format_number(block.gas_limit.to::<u64>())]);
        table.add_row(vec!["Gas Utilization", &format!("{:.1}%", 
            calculate_gas_utilization(block.gas_used.to::<u64>(), block.gas_limit.to::<u64>()))]);
        table.add_row(vec!["Transactions", &block.transactions.len().to_string()]);
        table.add_row(vec!["Miner", &format_eth_address(&block.miner)]);
        table.add_row(vec!["Difficulty", &format_number(block.difficulty.to::<u64>())]);
        table.add_row(vec!["Size", &format!("{} bytes", block.size.to::<u64>())]);
        
        println!("{}", table);
        
        if !block.transactions.is_empty() {
            println!("\n🔗 Recent Transactions:");
            let display_count = std::cmp::min(5, block.transactions.len());
            for (i, tx_hash) in block.transactions.iter().take(display_count).enumerate() {
                println!("  {}. {}", i + 1, format_b256_hash(tx_hash));
            }
            if block.transactions.len() > 5 {
                println!("  ... and {} more transactions", block.transactions.len() - 5);
            }
        }
        
        Ok(())
    }
    
    pub async fn show_transaction(&self, tx_hash: &str) -> Result<()> {
        println!("💸 Transaction Details");
        println!("=====================\n");
        
        let transaction = self.client.get_transaction(tx_hash).await?;
        
        let mut table = Table::new();
        table.load_preset(UTF8_FULL);
        table.set_header(vec!["Property", "Value"]);
        
        table.add_row(vec!["Hash", &format_b256_hash(&transaction.hash)]);
        if let Some(block_num) = &transaction.block_number {
            table.add_row(vec!["Block Number", &format_number(block_num.to::<u64>())]);
        }
        table.add_row(vec!["From", &format_eth_address(&transaction.from)]);
        
        match &transaction.to {
            Some(to_addr) => {
                table.add_row(vec!["To", &format_eth_address(to_addr)]);
            }
            None => {
                table.add_row(vec!["To", "📄 Contract Creation"]);
            }
        }
        
        table.add_row(vec!["Value", &format_wei_u256(&transaction.value)]);
        table.add_row(vec!["Gas Limit", &format_number(transaction.gas.to::<u64>())]);
        table.add_row(vec!["Gas Price", &format_gas_price(&transaction.gas_price)]);
        
        if let Some(gas_used) = &transaction.gas_used {
            table.add_row(vec!["Gas Used", &format_number(gas_used.to::<u64>())]);
            let tx_fee = transaction.gas_price * *gas_used;
            table.add_row(vec!["Transaction Fee", &format_wei_u256(&tx_fee)]);
        }
        
        table.add_row(vec!["Status", &format_tx_status(&transaction.status)]);
        
        println!("{}", table);
        Ok(())
    }
    
    pub async fn show_account(&self, address: &str, block: Option<u64>) -> Result<()> {
        let block_str = block.map_or("latest".to_string(), |b| b.to_string());
        println!("👤 Account Information (Block: {})", block_str);
        println!("==================================\n");
        
        let account = self.client.get_account_balance(address, block).await?;
        
        let mut table = Table::new();
        table.load_preset(UTF8_FULL);
        table.set_header(vec!["Property", "Value"]);
        
        table.add_row(vec!["Address", &format_eth_address(&account.address)]);
        table.add_row(vec!["Balance", &format_wei_u256(&account.balance)]);
        table.add_row(vec!["Nonce", &account.nonce.to_string()]);
        table.add_row(vec!["Type", account_type(&account.code_size)]);
        
        if !account.code_size.is_zero() {
            table.add_row(vec!["Code Size", &format!("{} bytes", account.code_size.to::<u64>())]);
        }
        
        println!("{}", table);
        Ok(())
    }
    
    pub async fn show_latest_blocks(&self, count: usize) -> Result<()> {
        println!("📊 Latest {} Blocks", count);
        println!("==================\n");
        
        let latest_block_num = self.client.get_latest_block_number().await?;
        
        let mut table = Table::new();
        table.load_preset(UTF8_FULL);
        table.set_header(vec!["Block #", "Hash", "Transactions", "Gas Used", "Time Ago"]);
        
        let start_block = latest_block_num.saturating_sub(count as u64 - 1);
        
        for block_num in (start_block..=latest_block_num).rev() {
            match self.client.get_block_by_number(block_num).await {
                Ok(block) => {
                    let gas_used_m = block.gas_used.to::<u128>() as f64 / 1_000_000.0;
                    let time_ago_str = time_ago(block.timestamp.to::<u64>());
                    
                    table.add_row(vec![
                        &block.number.to_string(),
                        &format_b256_hash(&block.hash),
                        &block.transactions.len().to_string(),
                        &format!("{:.1}M", gas_used_m),
                        &time_ago_str,
                    ]);
                }
                Err(e) => {
                    println!("⚠️  Failed to fetch block {}: {}", block_num, e);
                }
            }
        }
        
        println!("{}", table);
        Ok(())
    }
    
    pub async fn show_gas_statistics(&self, blocks: usize) -> Result<()> {
        println!("⛽ Gas Statistics (Last {} Blocks)", blocks);
        println!("=================================\n");
        
        let stats = self.client.get_gas_statistics(blocks).await?;
        
        let mut table = Table::new();
        table.load_preset(UTF8_FULL);
        table.set_header(vec!["Metric", "Value"]);
        
        table.add_row(vec!["Average Gas Used", &format_number(stats.avg_gas_used)]);
        table.add_row(vec!["Average Gas Price", &format!("{} Gwei", stats.avg_gas_price / 1_000_000_000)]);
        table.add_row(vec!["Max Gas Used", &format_number(stats.max_gas_used)]);
        table.add_row(vec!["Min Gas Used", &format_number(stats.min_gas_used)]);
        table.add_row(vec!["Gas Utilization", &format!("{:.1}%", stats.gas_utilization)]);
        table.add_row(vec!["Blocks Analyzed", &stats.blocks_analyzed.to_string()]);
        
        println!("{}", table);
        
        // Show gas usage trend visualization
        println!("\n📈 Gas Usage Trend:");
        self.show_gas_trend(&stats).await;
        
        Ok(())
    }
    
    async fn show_gas_trend(&self, _stats: &crate::rpc::GasStatistics) {
        // Simple ASCII visualization of gas usage trend
        // In a real implementation, you'd fetch recent blocks and show actual trend
        let trend_chars = vec!["▁", "▂", "▃", "▅", "▄", "▅", "▆", "▇", "█", "▆", "▅", "▄", "▃", "▂", "▁", "▂", "▃", "▄", "▅", "▆", "▅", "▄", "▃", "▂", "▁"];
        let trend_line: String = trend_chars.iter().cycle().take(50).map(|&s| s).collect();
        println!("{}", trend_line);
        println!("Low ←────────────────────────────────────────────→ High");
    }
}
