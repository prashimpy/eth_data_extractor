use alloy_primitives::{Address, B256, U256};
use chrono::{DateTime, Utc};

/// Format U256 Wei values to ETH with proper decimal places
pub fn format_wei_u256(wei: &U256) -> String {
    let wei_as_u128 = wei.to::<u128>();
    let eth = wei_as_u128 as f64 / 1_000_000_000_000_000_000.0;
    if eth >= 1.0 {
        format!("{:.4} ETH", eth)
    } else if eth >= 0.001 {
        format!("{:.6} ETH", eth)
    } else {
        format!("{:.9} ETH", eth)
    }
}

/// Format Wei values to ETH with proper decimal places
pub fn format_wei(wei_str: &str) -> String {
    match wei_str.parse::<u128>() {
        Ok(wei) => {
            let eth = wei as f64 / 1_000_000_000_000_000_000.0;
            if eth >= 1.0 {
                format!("{:.4} ETH", eth)
            } else if eth >= 0.001 {
                format!("{:.6} ETH", eth)
            } else {
                format!("{:.9} ETH", eth)
            }
        }
        Err(_) => format!("{} wei", wei_str),
    }
}

/// Format hash to show first 10 and last 4 characters
pub fn format_hash(hash: &str) -> String {
    if hash.len() >= 14 {
        format!("{}...{}", &hash[..10], &hash[hash.len()-4..])
    } else {
        hash.to_string()
    }
}

/// Format address to show first 6 and last 4 characters
pub fn format_address(address: &str) -> String {
    if address.len() >= 10 {
        format!("{}...{}", &address[..6], &address[address.len()-4..])
    } else {
        address.to_string()
    }
}

/// Convert hex string to decimal
pub fn hex_to_decimal(hex_str: &str) -> Result<u64, std::num::ParseIntError> {
    let clean_hex = hex_str.strip_prefix("0x").unwrap_or(hex_str);
    u64::from_str_radix(clean_hex, 16)
}

/// Format large numbers with commas
pub fn format_number(num: u64) -> String {
    let result = num.to_string();
    let mut chars: Vec<char> = result.chars().collect();
    
    let mut i = chars.len();
    while i > 3 {
        i -= 3;
        chars.insert(i, ',');
    }
    
    chars.into_iter().collect()
}

/// Format timestamp from Unix timestamp to human readable
pub fn format_timestamp(timestamp: u64) -> String {
    let dt = DateTime::from_timestamp(timestamp as i64, 0)
        .unwrap_or_else(|| Utc::now());
    dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

/// Format U256 timestamp to human readable
pub fn format_timestamp_u256(timestamp: &U256) -> String {
    let timestamp_u64 = timestamp.to::<u64>();
    format_timestamp(timestamp_u64)
}

/// Format B256 hash for display
pub fn format_b256_hash(hash: &B256) -> String {
    let hash_str = format!("{:?}", hash);
    format_hash(&hash_str)
}

/// Format Address for display
pub fn format_eth_address(address: &Address) -> String {
    let addr_str = format!("{:?}", address);
    format_address(&addr_str)
}

/// Format gas price from wei to gwei
pub fn format_gas_price(gas_price: &U256) -> String {
    let gwei = gas_price.to::<u128>() as f64 / 1_000_000_000.0;
    format!("{:.2} Gwei", gwei)
}

/// Format transaction status
pub fn format_tx_status(status: &Option<U256>) -> String {
    match status {
        Some(s) if s.is_zero() => "❌ Failed".to_string(),
        Some(_) => "✅ Success".to_string(),
        None => "⏳ Pending".to_string(),
    }
}

/// Calculate time ago from timestamp
pub fn time_ago(timestamp: u64) -> String {
    let now = Utc::now().timestamp() as u64;
    if now <= timestamp {
        return "just now".to_string();
    }
    
    let diff = now - timestamp;
    
    if diff < 60 {
        format!("{} sec ago", diff)
    } else if diff < 3600 {
        format!("{} min ago", diff / 60)
    } else if diff < 86400 {
        format!("{} hr ago", diff / 3600)
    } else {
        format!("{} days ago", diff / 86400)
    }
}

/// Detect contract vs EOA
pub fn account_type(code_size: &U256) -> &'static str {
    if code_size.is_zero() {
        "👤 Externally Owned Account"
    } else {
        "💼 Smart Contract"
    }
}

/// Calculate gas utilization percentage
pub fn calculate_gas_utilization(gas_used: u64, gas_limit: u64) -> f64 {
    if gas_limit == 0 {
        0.0
    } else {
        (gas_used as f64 / gas_limit as f64) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_wei() {
        assert_eq!(format_wei("1000000000000000000"), "1.0000 ETH");
        assert_eq!(format_wei("500000000000000000"), "0.500000 ETH");
        assert!(format_wei("invalid").contains("wei"));
    }

    #[test]
    fn test_format_hash() {
        let hash = "0x1234567890abcdef1234567890abcdef12345678";
        assert_eq!(format_hash(hash), "0x12345678...5678");
    }

    #[test]
    fn test_format_address() {
        let addr = "0x742d35Cc622C1E0532F7fd0e7c0e6f7D8F2B2B6f";
        assert_eq!(format_address(addr), "0x742d...2B6f");
    }

    #[test]
    fn test_hex_to_decimal() {
        assert_eq!(hex_to_decimal("0x10"), Ok(16));
        assert_eq!(hex_to_decimal("FF"), Ok(255));
    }

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(1234567), "1,234,567");
        assert_eq!(format_number(123), "123");
    }

    #[test]
    fn test_calculate_gas_utilization() {
        assert_eq!(calculate_gas_utilization(15_000_000, 30_000_000), 50.0);
        assert_eq!(calculate_gas_utilization(0, 30_000_000), 0.0);
        assert_eq!(calculate_gas_utilization(100, 0), 0.0);
    }
}
