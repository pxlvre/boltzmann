# boltzmann

<div align="center">
  <img src="assets/boltzmann.png" alt="Boltzmann Logo" width="200" height="200"/>
  
  **boltzmann: gas and fee analytics for EVM chains**
  
  ![License](https://img.shields.io/badge/license-AGPL--3.0-blue.svg)
  ![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)
  ![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)
</div>

## Overview

Boltzmann is a REST API built with **Rust**, **axum**, and **alloy-rs** that provides insights into gas and fee costs for EVM blockchain transactions. Whether you're building a DeFi application, optimizing transaction timing, or simply want to understand gas dynamics, Boltzmann gives you the data you need to make informed decisions.

### Key Features

-  **Transaction Simulation**: Simulate any EVM transaction (P2P transfers, ERC-20, NFTs, smart contract interactions, blob transactions)
-  **Multi-Speed Analysis**: Compare costs across different confirmation speeds (slow, average, fast)
-  **EIP Support**: Full compatibility with EIP-1559, EIP-3198, and EIP-4844
-  **Multi-Currency Pricing**: Get costs in native ETH or fiat currencies (USD, EUR, CHF, etc.)
-  **Real-time Subscriptions**: Monitor gas costs continuously with customizable intervals
-  **Historical Analytics**: Track gas price trends and optimize transaction timing
-  **Multi-Chain Support**: Works across all EVM-compatible networks

## Architecture

Boltzmann is a single Rust application with a modular design:

```
boltzmann/
├── src/
│   └── main.rs                  # Main application
├── assets/                      # Project assets
├── Cargo.toml                   # Package configuration
├── .env                         # Environment variables
└── README.md
```

### System Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Client Apps   │    │   Boltzmann     │    │  External APIs  │
│                 │    │      API        │    │                 │
│  • Web Apps     │◄──►│                 │◄──►│  • CoinGecko    │
│  • Mobile Apps  │    │  • Gas Oracle   │    │  • CoinMarketCap│
│  • DeFi Dapps   │    │  • Simulator    │    │  • RPC Nodes    │
│  • Trading Bots │    │  • Subscriptions│    │                 │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## API Endpoints

### Transaction Simulation

#### `POST /v1/simulate`
Simulate a transaction and get gas cost estimates across different priority levels.

```json
{
  "to": "0x742d35Cc6226926Ac52245b98eaA23Ca95E6A925",
  "value": "1000000000000000000",
  "data": "0x",
  "chain_id": 1,
  "from": "0x8ba1f109551bD432803012645Hac136c86c0A1B8",
  "priority_levels": ["slow", "standard", "fast"],
  "currency": "USD"
}
```

**Response:**
```json
{
  "simulation_id": "sim_1234567890",
  "estimates": {
    "slow": {
      "gas_price": "15000000000",
      "gas_limit": "21000",
      "total_fee_eth": "0.000315",
      "total_fee_fiat": "0.52",
      "confirmation_time_seconds": 300
    },
    "standard": {
      "gas_price": "20000000000",
      "gas_limit": "21000", 
      "total_fee_eth": "0.00042",
      "total_fee_fiat": "0.69",
      "confirmation_time_seconds": 60
    },
    "fast": {
      "gas_price": "30000000000",
      "gas_limit": "21000",
      "total_fee_eth": "0.00063", 
      "total_fee_fiat": "1.04",
      "confirmation_time_seconds": 15
    }
  },
  "timestamp": "2024-10-23T15:30:00Z",
  "chain_id": 1
}
```

#### `POST /v1/simulate/batch`
Simulate multiple transactions in a single request.

#### `GET /v1/simulate/{simulation_id}`
Retrieve details of a previous simulation.

### Gas Price Oracle

#### `GET /v1/gas/current`
Get current gas prices across all supported priority levels.

```json
{
  "chain_id": 1,
  "base_fee": "12000000000",
  "priority_fees": {
    "slow": "1000000000",
    "standard": "2000000000", 
    "fast": "5000000000"
  },
  "blob_base_fee": "1000000000",
  "timestamp": "2024-10-23T15:30:00Z"
}
```

### Subscriptions

#### `POST /v1/subscriptions`
Create a subscription to monitor gas costs for specific transactions.

```json
{
  "name": "Daily DEX Trade Monitor",
  "transaction_template": {
    "to": "0x1f98431c8ad98523631ae4a59f267346ea31f984",
    "data": "0x...",
    "value": "0"
  },
  "interval": {
    "type": "time",
    "seconds": 3600
  },
  "currency": "USD",
  "webhooks": ["https://api.myapp.com/gas-updates"]
}
```

#### `GET /v1/subscriptions`
List all active subscriptions.

#### `GET /v1/subscriptions/{subscription_id}`
Get subscription details and recent updates.

#### `DELETE /v1/subscriptions/{subscription_id}`
Cancel a subscription.

### Pricing

#### `GET /v1/pricing/rates`
Get current ETH exchange rates for supported fiat currencies.

#### `POST /v1/pricing/convert`
Convert between ETH and fiat currencies.

## 🛠️ Installation & Setup

### Prerequisites

- Rust 1.70+
- PostgreSQL 13+ (for subscription storage)
- Redis (for caching)

### Quick Start

```bash
# Clone the repository
git clone https://github.com/pxlvre/boltzmann.git
cd boltzmann

# Copy environment template
cp .env.example .env

# Configure your environment variables
# - RPC endpoints for supported chains
# - CoinGecko/CoinMarketCap API keys
# - Database connections

# Build and run
cargo build --release
cargo run
```

### Docker

```bash
docker build -t boltzmann .
docker run -p 8080:8080 --env-file .env boltzmann
```

### Configuration

Key environment variables:

```env
# Server
PORT=8080
HOST=0.0.0.0

# Database
DATABASE_URL=postgresql://user:pass@localhost/boltzmann
REDIS_URL=redis://localhost:6379

# RPC Endpoints
ETHEREUM_RPC_URL=https://eth-mainnet.alchemyapi.io/v2/your-key
POLYGON_RPC_URL=https://polygon-rpc.com
ARBITRUM_RPC_URL=https://arb1.arbitrum.io/rpc

# Pricing APIs
COINGECKO_API_KEY=your-coingecko-key
COINMARKETCAP_API_KEY=your-cmc-key

# Rate Limiting
RATE_LIMIT_PER_MINUTE=100
```

## Usage Examples

### Basic Transaction Cost Check

```bash
curl -X POST http://localhost:8080/v1/simulate \
  -H "Content-Type: application/json" \
  -d '{
    "to": "0x742d35Cc6226926Ac52245b98eaA23Ca95E6A925",
    "value": "1000000000000000000",
    "chain_id": 1,
    "currency": "USD"
  }'
```

### Monitor Uniswap V3 Swap Costs

```bash
curl -X POST http://localhost:8080/v1/subscriptions \
  -H "Content-Type: application/json" \
  -d '{
    "name": "USDC->ETH Swap Monitor",
    "transaction_template": {
      "to": "0x68b3465833fb72A70ecDF485E0e4C7bD8665Fc45",
      "data": "0x414bf389...",
      "value": "0"
    },
    "interval": {
      "type": "blocks",
      "count": 10
    },
    "currency": "USD"
  }'
```

### Check Current Gas Prices

```bash
curl http://localhost:8080/v1/gas/current?chain_id=1
```

## Development

### Application Structure

The application is organized with modular components:

- **Gas Price Oracle**: Fetches current gas prices from multiple sources
- **Transaction Simulation**: Simulates transaction costs across different scenarios
- **Fee Calculation**: Implements EIP-1559, EIP-3198, and EIP-4844 fee structures
- **Price Conversion**: Integrates with CoinGecko/CoinMarketCap for fiat conversion
- **REST API**: Provides HTTP endpoints for all functionality
- **Subscription System**: Real-time monitoring and webhooks

### Running Tests

```bash
cargo test
```

### Building

```bash
cargo build
```

### API Documentation

Start the server and visit `http://localhost:8080/docs` for interactive API documentation.

### Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## Supported Chains

- **Ethereum Mainnet** (Chain ID: 1)
- **Polygon** (Chain ID: 137)
- **Arbitrum One** (Chain ID: 42161)
- **Optimism** (Chain ID: 10)
- **Base** (Chain ID: 8453)
- **Avalanche C-Chain** (Chain ID: 43114)

More chains coming soon!

## License

This project is licensed under the GNU Affero General Public License v3.0 - see the [LICENSE](LICENSE) file for details.

The choice of AGPL-3.0 aligns with the principles outlined by Vitalik Buterin in his post ["Why I used to prefer permissive licenses and now favor copyleft"](https://vitalik.eth.limo/general/2025/07/07/copyleft.html), ensuring that improvements and derivatives remain open and accessible to the community.
