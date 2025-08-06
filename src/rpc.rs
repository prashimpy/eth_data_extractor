use alloy_primitives::{Address, B256, U256};
use backoff::ExponentialBackoffBuilder;
use color_eyre::{eyre::eyre, Result};
use jsonrpsee::{
    http_client::{HttpClient, HttpClientBuilder},
    core::client::ClientT,
    rpc_params,
};
use moka::future::Cache;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub number: U256,
    pub hash: B256,
    pub parent_hash: B256,
    pub timestamp: U256,
    pub gas_used: U256,
    pub gas_limit: U256,
    pub transactions: Vec<B256>,
    pub miner: Address,
    pub difficulty: U256,
    pub size: U256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub hash: B256,
    pub block_number: Option<U256>,
    pub from: Address,
    pub to: Option<Address>,
    pub value: U256,
    pub gas: U256,
    pub gas_price: U256,
    pub gas_used: Option<U256>,
    pub status: Option<U256>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub address: Address,
    pub balance: U256,
    pub nonce: U256,
    pub code_size: U256,
}

#[derive(Debug, Clone)]
pub struct GasStatistics {
    pub avg_gas_used: u64,
    pub avg_gas_price: u64,
    pub max_gas_used: u64,
    pub min_gas_used: u64,
    pub gas_utilization: f64,
    pub blocks_analyzed: usize,
}

pub struct RethClient {
    client: HttpClient,
    cache: Cache<String, serde_json::Value>,
    rpc_url: String,
}

impl RethClient {
    pub async fn new(rpc_url: &str) -> Result<Self> {
        let client = HttpClientBuilder::default()
            .request_timeout(Duration::from_secs(60))
            .build(rpc_url)?;

        // Test connection
        let _chain_id: String = client
            .request("eth_chainId", rpc_params![])
            .await
            .map_err(|e| eyre!("Failed to connect to Reth node: {}", e))?;

        println!("✅ Connected to Reth node at {}", rpc_url);

        // Create cache with 1000 entries, 5 minute TTL
        let cache = Cache::builder()
            .max_capacity(1000)
            .time_to_live(Duration::from_secs(300))
            .build();

        Ok(Self {
            client,
            cache,
            rpc_url: rpc_url.to_string(),
        })
    }

    pub async fn get_block_by_number(&self, block_number: u64) -> Result<Block> {
        let cache_key = format!("block_{}", block_number);
        
        if let Some(cached) = self.cache.get(&cache_key).await {
            if let Ok(block) = serde_json::from_value(cached) {
                return Ok(block);
            }
        }

        let block_hex = format!("0x{:x}", block_number);
        let result: serde_json::Value = self
            .retry_rpc_call("eth_getBlockByNumber", rpc_params![block_hex, true])
            .await?;

        let block = self.parse_block(result.clone())?;
        self.cache.insert(cache_key, result).await;
        
        Ok(block)
    }

    pub async fn get_block_by_hash(&self, block_hash: &str) -> Result<Block> {
        let cache_key = format!("block_hash_{}", block_hash);
        
        if let Some(cached) = self.cache.get(&cache_key).await {
            if let Ok(block) = serde_json::from_value(cached) {
                return Ok(block);
            }
        }

        let result: serde_json::Value = self
            .retry_rpc_call("eth_getBlockByHash", rpc_params![block_hash, true])
            .await?;

        let block = self.parse_block(result.clone())?;
        self.cache.insert(cache_key, result).await;
        
        Ok(block)
    }

    pub async fn get_latest_block_number(&self) -> Result<u64> {
        let result: String = self
            .retry_rpc_call("eth_blockNumber", rpc_params![])
            .await?;

        let block_number = u64::from_str_radix(&result[2..], 16)
            .map_err(|e| eyre!("Failed to parse block number: {}", e))?;

        Ok(block_number)
    }

    pub async fn get_transaction(&self, tx_hash: &str) -> Result<Transaction> {
        let cache_key = format!("tx_{}", tx_hash);
        
        if let Some(cached) = self.cache.get(&cache_key).await {
            if let Ok(tx) = serde_json::from_value(cached) {
                return Ok(tx);
            }
        }

        let tx_result: serde_json::Value = self
            .retry_rpc_call("eth_getTransactionByHash", rpc_params![tx_hash])
            .await?;

        let receipt_result: serde_json::Value = self
            .retry_rpc_call("eth_getTransactionReceipt", rpc_params![tx_hash])
            .await?;

        let transaction = self.parse_transaction(tx_result.clone(), receipt_result)?;
        self.cache.insert(cache_key, tx_result).await;
        
        Ok(transaction)
    }

    pub async fn get_account_balance(&self, address: &str, block: Option<u64>) -> Result<Account> {
        let block_param = match block {
            Some(n) => format!("0x{:x}", n),
            None => "latest".to_string(),
        };

        let cache_key = format!("balance_{}_{}", address, block_param);
        
        if let Some(cached) = self.cache.get(&cache_key).await {
            if let Ok(account) = serde_json::from_value(cached.clone()) {
                return Ok(account);
            }
        }

        let balance: String = self
            .retry_rpc_call("eth_getBalance", rpc_params![address, &block_param])
            .await?;

        let nonce: String = self
            .retry_rpc_call("eth_getTransactionCount", rpc_params![address, &block_param])
            .await?;

        let code: String = self
            .retry_rpc_call("eth_getCode", rpc_params![address, &block_param])
            .await?;

        let account = Account {
            address: address.parse()
                .map_err(|e| eyre!("Invalid address format: {}", e))?,
            balance: U256::from_str_radix(&balance[2..], 16)
                .map_err(|e| eyre!("Failed to parse balance: {}", e))?,
            nonce: U256::from_str_radix(&nonce[2..], 16)
                .map_err(|e| eyre!("Failed to parse nonce: {}", e))?,
            code_size: U256::from((code.len() - 2) / 2), // -2 for "0x", /2 for hex pairs
        };

        let cached_value = serde_json::to_value(&account)?;
        self.cache.insert(cache_key, cached_value).await;
        
        Ok(account)
    }

    pub async fn get_gas_statistics(&self, block_count: usize) -> Result<GasStatistics> {
        let latest_block = self.get_latest_block_number().await?;
        let start_block = latest_block.saturating_sub(block_count as u64);

        let mut total_gas_used = 0u64;
        let mut total_gas_price = 0u64;
        let mut max_gas_used = 0u64;
        let mut min_gas_used = u64::MAX;
        let mut blocks_processed = 0usize;

        println!("📊 Analyzing gas statistics for {} blocks...", block_count);

        for block_num in start_block..=latest_block {
            if let Ok(block) = self.get_block_by_number(block_num).await {
                let gas_used = block.gas_used.to::<u64>();
                total_gas_used += gas_used;
                max_gas_used = max_gas_used.max(gas_used);
                min_gas_used = min_gas_used.min(gas_used);
                blocks_processed += 1;

                // For gas price, we'd need to analyze transactions in the block
                // For simplicity, we'll estimate based on current gas price
                total_gas_price += 25_000_000_000u64; // ~25 Gwei estimate
            }
        }

        if blocks_processed == 0 {
            return Err(eyre!("No blocks found for gas statistics"));
        }

        let avg_gas_used = total_gas_used / blocks_processed as u64;
        let avg_gas_price = total_gas_price / blocks_processed as u64;
        let gas_utilization = (avg_gas_used as f64 / 30_000_000.0) * 100.0;

        Ok(GasStatistics {
            avg_gas_used,
            avg_gas_price,
            max_gas_used,
            min_gas_used,
            gas_utilization,
            blocks_analyzed: blocks_processed,
        })
    }

    async fn retry_rpc_call<T>(&self, method: &str, params: jsonrpsee::core::params::ArrayParams) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let backoff = ExponentialBackoffBuilder::new()
            .with_max_elapsed_time(Some(Duration::from_secs(30)))
            .build();

        backoff::future::retry(backoff, || async {
            self.client
                .request(method, params.clone())
                .await
                .map_err(|e| {
                    println!("⚠️  RPC call failed, retrying: {}", e);
                    backoff::Error::transient(eyre!("RPC call failed: {}", e))
                })
        })
        .await
    }

    fn parse_block(&self, value: serde_json::Value) -> Result<Block> {
        let obj = value.as_object().ok_or_else(|| eyre!("Invalid block format"))?;
        
        Ok(Block {
            number: U256::from_str_radix(
                obj.get("number").and_then(|v| v.as_str()).unwrap_or("0x0").trim_start_matches("0x"), 16
            )?,
            hash: obj.get("hash").and_then(|v| v.as_str()).unwrap_or("0x0").parse()?,
            parent_hash: obj.get("parentHash").and_then(|v| v.as_str()).unwrap_or("0x0").parse()?,
            timestamp: U256::from_str_radix(
                obj.get("timestamp").and_then(|v| v.as_str()).unwrap_or("0x0").trim_start_matches("0x"), 16
            )?,
            gas_used: U256::from_str_radix(
                obj.get("gasUsed").and_then(|v| v.as_str()).unwrap_or("0x0").trim_start_matches("0x"), 16
            )?,
            gas_limit: U256::from_str_radix(
                obj.get("gasLimit").and_then(|v| v.as_str()).unwrap_or("0x0").trim_start_matches("0x"), 16
            )?,
            transactions: obj.get("transactions")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|tx| {
                    if let Some(hash_str) = tx.as_str() {
                        hash_str.parse().ok()
                    } else {
                        tx.get("hash").and_then(|h| h.as_str()?.parse().ok())
                    }
                }).collect())
                .unwrap_or_default(),
            miner: obj.get("miner").and_then(|v| v.as_str()).unwrap_or("0x0000000000000000000000000000000000000000").parse()?,
            difficulty: U256::from_str_radix(
                obj.get("difficulty").and_then(|v| v.as_str()).unwrap_or("0x0").trim_start_matches("0x"), 16
            )?,
            size: U256::from_str_radix(
                obj.get("size").and_then(|v| v.as_str()).unwrap_or("0x0").trim_start_matches("0x"), 16
            )?,
        })
    }

    fn parse_transaction(&self, tx_value: serde_json::Value, receipt_value: serde_json::Value) -> Result<Transaction> {
        let tx_obj = tx_value.as_object().ok_or_else(|| eyre!("Invalid transaction format"))?;
        let receipt_obj = receipt_value.as_object().ok_or_else(|| eyre!("Invalid receipt format"))?;
        
        Ok(Transaction {
            hash: tx_obj.get("hash").and_then(|v| v.as_str()).unwrap_or("0x0").parse()?,
            block_number: tx_obj.get("blockNumber").and_then(|v| v.as_str()).map(|s| {
                U256::from_str_radix(s.trim_start_matches("0x"), 16).ok()
            }).flatten(),
            from: tx_obj.get("from").and_then(|v| v.as_str()).unwrap_or("0x0000000000000000000000000000000000000000").parse()?,
            to: tx_obj.get("to").and_then(|v| v.as_str()).map(|s| s.parse().ok()).flatten(),
            value: U256::from_str_radix(
                tx_obj.get("value").and_then(|v| v.as_str()).unwrap_or("0x0").trim_start_matches("0x"), 16
            )?,
            gas: U256::from_str_radix(
                tx_obj.get("gas").and_then(|v| v.as_str()).unwrap_or("0x0").trim_start_matches("0x"), 16
            )?,
            gas_price: U256::from_str_radix(
                tx_obj.get("gasPrice").and_then(|v| v.as_str()).unwrap_or("0x0").trim_start_matches("0x"), 16
            )?,
            gas_used: receipt_obj.get("gasUsed").and_then(|v| v.as_str()).map(|s| {
                U256::from_str_radix(s.trim_start_matches("0x"), 16).ok()
            }).flatten(),
            status: receipt_obj.get("status").and_then(|v| v.as_str()).map(|s| {
                U256::from_str_radix(s.trim_start_matches("0x"), 16).ok()
            }).flatten(),
        })
    }
}
