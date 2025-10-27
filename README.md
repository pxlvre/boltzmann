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

## 🏗️ Architecture

Boltzmann follows a **domain-driven design** with a modular Rust architecture:

```
boltzmann/
├── src/
│   ├── domains/                 # Business logic domains
│   │   ├── crypto/             # Cryptocurrency price providers
│   │   │   ├── coingecko.rs    # CoinGecko integration
│   │   │   ├── coinmarketcap.rs # CoinMarketCap integration  
│   │   │   └── mod.rs          # Core crypto types
│   │   └── gas/                # Gas price oracles
│   │       └── price/          # Gas price providers
│   │           ├── etherscan.rs # Etherscan gas oracle
│   │           ├── alloy.rs    # Alloy RPC gas oracle
│   │           └── mod.rs      # Gas oracle types
│   ├── api/                    # HTTP layer
│   │   └── routes/             # API route handlers
│   │       ├── crypto.rs       # Crypto price endpoints
│   │       ├── gas.rs          # Gas price endpoints
│   │       └── mod.rs          # Router setup
│   ├── core/                   # Core application
│   │   ├── config.rs           # Configuration management
│   │   ├── server.rs           # Server initialization
│   │   └── mod.rs              # Core module root
│   ├── main.rs                 # Application entry point
│   └── lib.rs                  # Library root
├── assets/                     # Project assets
├── Dockerfile                  # Docker configuration
├── docker-compose.yml          # Docker Compose setup
├── Cargo.toml                  # Package configuration
├── .env                        # Environment variables
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

## 📡 API Endpoints

### Health Check

#### `GET /api/v1/health`
Check if the API server is running.

**Response:**
```
Boltzmann API is running
```

### Cryptocurrency Prices

#### `GET /api/v1/price/prices`
Get current ETH prices from multiple providers.

**Query Parameters:**
- `amount` (optional): Number of ETH tokens (default: 1)
- `currency` (optional): Target currency - USD, EUR, CHF, CNY, GBP, JPY, CAD, AUD (default: USD)

**Example:**
```bash
curl "http://localhost:3000/api/v1/price/prices?amount=5&currency=EUR"
```

### Gas Price Estimates

#### `GET /api/v1/gas/prices`  
Get current Ethereum gas price estimates.

**Query Parameters:**
- `provider` (optional): Gas oracle provider - "etherscan" or "alloy" (default: etherscan)

**Example:**
```bash
curl "http://localhost:3000/api/v1/gas/prices?provider=alloy"
```

## 🚧 Planned Features (Coming Soon)

The following advanced features are planned for future releases:

- **Transaction Simulation**: `POST /api/v1/simulate` - Simulate EVM transactions
- **Batch Operations**: `POST /api/v1/simulate/batch` - Multiple transaction simulation
- **Subscriptions**: Real-time monitoring with webhooks
- **Historical Data**: Gas price trends and analytics
- **Multi-Chain**: Support for Polygon, Arbitrum, and other EVM chains

## 🛠️ Installation & Setup

### Prerequisites

- **Docker & Docker Compose** (recommended)
- **OR** Rust 1.75+ for local development

### 🐳 Quick Start with Docker (Recommended)

```bash
# Clone the repository
git clone https://github.com/pxlvre/boltzmann.git
cd boltzmann

# Create environment file from template
cp .env.example .env

# Edit .env with your API keys (see Configuration section below)
vim .env

# Start with Docker Compose
docker-compose up -d

# Check if it's running
curl http://localhost:3000/api/v1/health
```

### 🦀 Local Development Setup

```bash
# Clone the repository
git clone https://github.com/pxlvre/boltzmann.git
cd boltzmann

# Create environment file
cp .env.example .env

# Configure your environment variables (see below)
vim .env

# Build and run
cargo build --release
cargo run
```

### 🐳 Docker Commands

```bash
# Build the image
docker build -t boltzmann-api .

# Run with Docker Compose (recommended)
docker-compose up -d

# Run standalone container
docker run -p 3000:3000 \
  -e COINMARKETCAP_API_KEY=your_key \
  -e COINGECKO_API_KEY=your_key \
  -e ETHERSCAN_API_KEY=your_key \
  -e ETHEREUM_RPC_URL=your_rpc_url \
  boltzmann-api

# View logs
docker-compose logs -f boltzmann-api

# Stop services
docker-compose down
```

### ⚙️ Configuration

Create a `.env` file with the following environment variables:

```env
# Server Configuration
HOST=127.0.0.1         # Use 0.0.0.0 for Docker
PORT=3000

# API Keys (at least one price provider required)
COINMARKETCAP_API_KEY=your-coinmarketcap-key    # Get from: https://coinmarketcap.com/api/
COINGECKO_API_KEY=your-coingecko-key           # Get from: https://www.coingecko.com/en/api

# Gas Price Providers
ETHERSCAN_API_KEY=your-etherscan-key           # Optional - Get from: https://etherscan.io/apis
ETHEREUM_RPC_URL=your-rpc-url                  # Required - Get from Infura, Alchemy, etc.
```

**Required Configuration:**
- At least **one** of `COINMARKETCAP_API_KEY` or `COINGECKO_API_KEY`
- `ETHEREUM_RPC_URL` for gas price functionality

**Getting API Keys:**
- **CoinMarketCap**: [https://coinmarketcap.com/api/](https://coinmarketcap.com/api/) (free tier available)
- **CoinGecko**: [https://www.coingecko.com/en/api](https://www.coingecko.com/en/api) (free tier available)
- **Etherscan**: [https://etherscan.io/apis](https://etherscan.io/apis) (optional, free tier available)
- **Ethereum RPC**: [Infura](https://infura.io/), [Alchemy](https://www.alchemy.com/), or any Ethereum JSON-RPC endpoint

## 🚀 API Usage Examples

### Health Check

```bash
curl http://localhost:3000/api/v1/health
```

### Get Cryptocurrency Prices

```bash
# Get 1 ETH price in USD (default)
curl http://localhost:3000/api/v1/price/prices

# Get 5 ETH price in EUR
curl "http://localhost:3000/api/v1/price/prices?amount=5&currency=EUR"
```

**Response:**
```json
[
  {
    "coin": "ETH",
    "currency": "USD", 
    "price": 4164.82,
    "provider": "coinmarketcap",
    "quote_per_amount": {
      "amount": 1.0,
      "total_price": 4164.82
    },
    "timestamp": "2025-10-27T15:30:00Z"
  },
  {
    "coin": "ETH",
    "currency": "USD",
    "price": 4162.15,
    "provider": "coingecko", 
    "quote_per_amount": {
      "amount": 1.0,
      "total_price": 4162.15
    },
    "timestamp": "2025-10-27T15:30:00Z"
  }
]
```

### Get Gas Price Estimates

```bash
# Get gas prices from Etherscan (default)
curl http://localhost:3000/api/v1/gas/prices

# Get gas prices from Alloy RPC 
curl "http://localhost:3000/api/v1/gas/prices?provider=alloy"
```

**Response:**
```json
{
  "gas_price": {
    "low": 2.361777,
    "average": 2.402917,
    "high": 2.798484,
    "timestamp": "2025-10-27T15:30:00Z"
  },
  "provider": "etherscan"
}
```

### Supported Currencies

- **USD** - US Dollar
- **EUR** - Euro  
- **CHF** - Swiss Franc
- **CNY** - Chinese Yuan
- **GBP** - British Pound
- **JPY** - Japanese Yen
- **CAD** - Canadian Dollar
- **AUD** - Australian Dollar

### Supported Gas Providers

- **etherscan** - Etherscan Gas Tracker API (default)
- **alloy** - Direct Ethereum RPC via Alloy

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

More chains coming soon!

## License

This project is licensed under the GNU Affero General Public License v3.0 - see the [LICENSE](LICENSE) file for details.

The choice of AGPL-3.0 aligns with the principles outlined by Vitalik Buterin in his post ["Why I used to prefer permissive licenses and now favor copyleft"](https://vitalik.eth.limo/general/2025/07/07/copyleft.html), ensuring that improvements and derivatives remain open and accessible to the community.
