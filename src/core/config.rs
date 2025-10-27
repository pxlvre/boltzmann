//! Application configuration module.
//!
//! This module provides centralized configuration management for the Boltzmann API server.
//! All environment variables are loaded once at startup and stored in the app state.

use std::sync::Arc;

/// Application configuration loaded from environment variables
#[derive(Debug, Clone)]
pub struct Config {
    /// CoinMarketCap API key
    pub coinmarketcap_api_key: Option<String>,
    /// CoinGecko API key  
    pub coingecko_api_key: Option<String>,
    /// Etherscan API key
    pub etherscan_api_key: Option<String>,
    /// Ethereum RPC URL (for alloy provider)
    pub ethereum_rpc_url: Option<String>,
    /// Server host address
    pub host: String,
    /// Server port
    pub port: u16,
}

impl Config {
    /// Load configuration from environment variables
    ///
    /// This should be called once at application startup after loading .env files.
    /// Missing optional API keys will result in None values, which providers can handle gracefully.
    pub fn from_env() -> Result<Self, ConfigError> {
        // Load dotenv first
        dotenvy::dotenv().ok();

        let coinmarketcap_api_key = std::env::var("COINMARKETCAP_API_KEY").ok();
        let coingecko_api_key = std::env::var("COINGECKO_API_KEY").ok();
        let etherscan_api_key = std::env::var("ETHERSCAN_API_KEY").ok();
        let ethereum_rpc_url = std::env::var("ETHEREUM_RPC_URL").ok();

        let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = std::env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse::<u16>()
            .map_err(|e| ConfigError::InvalidPort(e.to_string()))?;

        // Validate required configuration
        // At least one price provider is required
        if coinmarketcap_api_key.is_none() && coingecko_api_key.is_none() {
            return Err(ConfigError::NoPriceProvider);
        }

        // Ethereum RPC URL is required for gas functionality
        if ethereum_rpc_url.is_none() {
            return Err(ConfigError::MissingEthereumRpc);
        }

        println!("🔧 Configuration loaded:");
        println!("   Host: {}", host);
        println!("   Port: {}", port);
        println!("   CoinMarketCap API: {}", if coinmarketcap_api_key.is_some() { "✅" } else { "❌" });
        println!("   CoinGecko API: {}", if coingecko_api_key.is_some() { "✅" } else { "❌" });
        println!("   Etherscan API: {}", if etherscan_api_key.is_some() { "✅" } else { "❌" });
        println!("   Ethereum RPC: {}", if ethereum_rpc_url.is_some() { "✅" } else { "❌" });

        Ok(Config {
            coinmarketcap_api_key,
            coingecko_api_key,
            etherscan_api_key,
            ethereum_rpc_url,
            host,
            port,
        })
    }
}

/// Configuration loading errors
#[derive(Debug)]
pub enum ConfigError {
    /// Invalid port number
    InvalidPort(String),
    /// No price provider configured (need at least CoinMarketCap OR CoinGecko)
    NoPriceProvider,
    /// Missing required Ethereum RPC URL
    MissingEthereumRpc,
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::InvalidPort(e) => write!(f, "Invalid port number: {}", e),
            ConfigError::NoPriceProvider => write!(f, 
                "❌ No price provider configured!\n\n\
                At least one of the following API keys must be set:\n\
                • COINMARKETCAP_API_KEY - Get free key at: https://coinmarketcap.com/api/\n\
                • COINGECKO_API_KEY - Get free key at: https://www.coingecko.com/en/api\n\n\
                Add one of these to your .env file to continue."
            ),
            ConfigError::MissingEthereumRpc => write!(f,
                "❌ Ethereum RPC URL required!\n\n\
                The ETHEREUM_RPC_URL environment variable must be set for gas price functionality.\n\
                You can use:\n\
                • Infura: https://infura.io/\n\
                • Alchemy: https://www.alchemy.com/\n\
                • Or any Ethereum JSON-RPC endpoint\n\n\
                Add ETHEREUM_RPC_URL to your .env file to continue."
            ),
        }
    }
}

impl std::error::Error for ConfigError {}

/// Shared application state
#[derive(Debug, Clone)]
pub struct AppState {
    /// Application configuration
    pub config: Arc<Config>,
}

impl AppState {
    /// Create new app state with configuration
    pub fn new(config: Config) -> Self {
        Self {
            config: Arc::new(config),
        }
    }
}