# Ethereum Data Extractor CLI

[![CI](https://github.com/prashimpy/eth_data_extractor/workflows/CI/badge.svg)](https://github.com/prashimpy/eth_data_extractor/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-brightgreen.svg)](https://www.rust-lang.org)

A command-line Ethereum data extraction tool built with Rust and Reth, Paradigm's high-performance Ethereum execution client.

## Features

🔍 **Block Information**: Get detailed information about any block by number or hash
💸 **Transaction Details**: View comprehensive transaction data including gas usage and fees
👤 **Account Explorer**: Check account balances, nonces, and contract information
📊 **Latest Blocks**: Display recent blocks with key metrics
⛽ **Gas Analytics**: Analyze gas usage patterns and statistics

## Installation

1. Ensure you have Rust installed:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. Clone and build the project:
   ```bash
   cd Downloads/code/rust/eth_data_extractor
   cargo build --release
   ```

## Usage

### View Block Information
```bash
# By block number
cargo run -- block 18234567

# By block hash
cargo run -- block 0x1234567890abcdef...
```

### Get Transaction Details
```bash
cargo run -- transaction 0xabcdef1234567890...
```

### Check Account Information
```bash
# Latest block
cargo run -- account 0x742d35Cc622C1E0532F7fd0e7c0e6f7D8F2B2B6f

# Specific block
cargo run -- account 0x742d35Cc622C1E0532F7fd0e7c0e6f7D8F2B2B6f --block 18234567
```

### View Latest Blocks
```bash
# Default: 10 blocks
cargo run -- latest

# Custom count
cargo run -- latest --count 25
```

### Gas Statistics
```bash
# Default: 100 blocks
cargo run -- gas

# Custom range
cargo run -- gas --blocks 500
```

## Current Implementation Status

⚠️ **Note**: This is currently a **mock implementation** for demonstration purposes. The actual Reth integration requires:

1. **Reth Node Connection**: Connect to a running Reth node via RPC
2. **Real Data Fetching**: Replace mock data with actual blockchain queries
3. **Error Handling**: Implement proper error handling for network issues
4. **Caching**: Add caching mechanisms for better performance

## Extending to Real Reth Integration

To connect this to a real Reth node, you would:

### 1. Set up Reth Node
```bash
# Install Reth
cargo install --git https://github.com/paradigmxyz/reth reth

# Run Reth node
reth node
```

### 2. Update Dependencies
Add RPC client dependencies:
```toml
[dependencies]
reqwest = { version = "0.11", features = ["json"] }
jsonrpsee = { version = "0.20", features = ["client"] }
```

### 3. Implement Real RPC Calls
Replace mock data in `explorer.rs` with actual RPC calls:
```rust
// Example RPC call
let block = client
    .request("eth_getBlockByNumber", rpc_params![block_id, true])
    .await?;
```

## Architecture

- **`main.rs`**: CLI interface and command parsing
- **`explorer.rs`**: Core blockchain exploration logic
- **`utils.rs`**: Formatting and utility functions
- **`Cargo.toml`**: Dependencies and project configuration

## Dependencies

- **Reth**: Ethereum execution client libraries
- **Alloy**: Ethereum primitive types
- **Clap**: Command-line argument parsing
- **Tokio**: Async runtime
- **Color-eyre**: Enhanced error handling
- **Comfy-table**: Terminal table formatting

## Testing

Run the test suite:
```bash
cargo test
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Submit a pull request

## License

MIT License - feel free to use this code for learning and building!

## Next Steps

🚀 **Potential Enhancements**:
- Real-time block monitoring
- WebSocket support for live updates
- Export data to JSON/CSV
- Interactive TUI interface
- MEV detection and analysis
- Smart contract interaction
- Multi-chain support (L2s)
