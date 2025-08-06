use std::str::FromStr;

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
    // Mock implementation - in reality you'd use chrono or similar
    format!("{} seconds since epoch", timestamp)
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
