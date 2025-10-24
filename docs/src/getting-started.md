# Getting Started

## Installation

Add Boltzmann to your `Cargo.toml`:

```toml
[dependencies]
boltzmann = { git = "https://github.com/pxlvre/boltzmann" }
```

## Environment Setup

The library requires API keys for the price providers. Create a `.env` file in your project root:

```bash
# CoinMarketCap API key (required)
COINMARKETCAP_API_KEY=your_cmc_api_key_here

# CoinGecko API key (optional - free tier available)
COINGECKO_API_KEY=your_coingecko_api_key_here
```

## Basic Example

```rust
use boltzmann::coins::{Coin, Currency, PriceProvider};
use boltzmann::coins::coinmarketcap::CoinMarketCap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a CoinMarketCap provider
    let provider = CoinMarketCap::new()?;
    
    // Fetch ETH price in USD
    let quotes = provider.get_quotes(Coin::ETH, &[Currency::USD]).await?;
    
    // Display the result
    for quote in quotes {
        println!("{} = {}{:.2}", 
            quote.coin, 
            quote.currency.symbol(), 
            quote.price
        );
    }
    
    Ok(())
}
```

## Multiple Currencies

Fetch prices in multiple currencies with a single API call:

```rust
use boltzmann::coins::{Coin, Currency, PriceProvider};
use boltzmann::coins::coinmarketcap::CoinMarketCap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = CoinMarketCap::new()?;
    
    // Fetch ETH in all supported currencies
    let quotes = provider.get_quotes(Coin::ETH, Currency::all()).await?;
    
    for quote in quotes {
        println!("{} = {}{:.2}", 
            quote.coin, 
            quote.currency.symbol(), 
            quote.price
        );
    }
    
    Ok(())
}
```