//! Etherscan Gas API provider implementation.
//!
//! This module implements gas price fetching using the Etherscan Gas Tracker API.

use crate::gas::price::{GasOracle, GasPrice};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use std::error::Error as StdError;
use std::fmt;

/// Etherscan API error types
#[derive(Debug)]
pub enum EtherscanError {
    /// HTTP request failed
    RequestError(reqwest::Error),
    /// API returned an error response
    ApiError(String),
    /// Failed to parse response JSON
    ParseError(serde_json::Error),
    /// Missing required environment variable
    MissingApiKey,
}

impl fmt::Display for EtherscanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EtherscanError::RequestError(e) => write!(f, "Request error: {}", e),
            EtherscanError::ApiError(msg) => write!(f, "API error: {}", msg),
            EtherscanError::ParseError(e) => write!(f, "Parse error: {}", e),
            EtherscanError::MissingApiKey => write!(f, "Missing ETHERSCAN_API_KEY environment variable"),
        }
    }
}

impl StdError for EtherscanError {}

impl From<reqwest::Error> for EtherscanError {
    fn from(error: reqwest::Error) -> Self {
        EtherscanError::RequestError(error)
    }
}

impl From<serde_json::Error> for EtherscanError {
    fn from(error: serde_json::Error) -> Self {
        EtherscanError::ParseError(error)
    }
}

/// Etherscan Gas API response structure
#[derive(Debug, Deserialize)]
struct EtherscanGasResponse {
    status: String,
    message: String,
    result: EtherscanGasResult,
}

#[derive(Debug, Deserialize)]
struct EtherscanGasResult {
    #[serde(rename = "LastBlock")]
    last_block: String,
    #[serde(rename = "SafeGasPrice")]
    safe_gas_price: String,
    #[serde(rename = "ProposeGasPrice")]
    propose_gas_price: String,
    #[serde(rename = "FastGasPrice")]
    fast_gas_price: String,
    #[serde(rename = "suggestBaseFee")]
    suggest_base_fee: String,
    #[serde(rename = "gasUsedRatio")]
    gas_used_ratio: String,
}

/// Etherscan gas price provider
pub struct EtherscanGasOracle {
    client: Client,
    api_key: String,
    base_url: String,
}

impl EtherscanGasOracle {
    /// Creates a new Etherscan gas oracle instance.
    ///
    /// Requires the ETHERSCAN_API_KEY environment variable to be set.
    ///
    /// # Errors
    ///
    /// Returns `EtherscanError::MissingApiKey` if the API key is not found.
    pub fn new() -> Result<Self, EtherscanError> {
        let api_key = std::env::var("ETHERSCAN_API_KEY")
            .map_err(|_| EtherscanError::MissingApiKey)?;

        Ok(Self {
            client: Client::new(),
            api_key,
            base_url: "https://api.etherscan.io/v2/api".to_string(),
        })
    }

    /// Creates a new Etherscan gas oracle with a custom API key.
    pub fn with_api_key(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: "https://api.etherscan.io/v2/api".to_string(),
        }
    }
}

#[async_trait]
impl GasOracle for EtherscanGasOracle {
    type Error = EtherscanError;

    async fn get_gas_prices(&self) -> Result<GasPrice, Self::Error> {
        let url = format!(
            "{}?chainid=1&module=gastracker&action=gasoracle&apikey={}",
            self.base_url, self.api_key
        );

        println!("🔗 Etherscan API URL: {}", url);
        
        let response = self.client.get(&url).send().await?;
        let body = response.text().await?;
        
        println!("📨 Raw API response: {}", body);
        
        let gas_response: EtherscanGasResponse = serde_json::from_str(&body)?;
        
        println!("🔍 Parsed response: {:?}", gas_response);

        if gas_response.status != "1" {
            return Err(EtherscanError::ApiError(gas_response.message));
        }

        // Parse gas prices from decimal strings to f64 (preserve precision)
        println!("💰 Parsing gas prices:");
        println!("   SafeGasPrice: '{}'", gas_response.result.safe_gas_price);
        println!("   ProposeGasPrice: '{}'", gas_response.result.propose_gas_price);
        println!("   FastGasPrice: '{}'", gas_response.result.fast_gas_price);
        
        let low = gas_response.result.safe_gas_price
            .parse::<f64>()
            .map_err(|e| EtherscanError::ApiError(format!("Invalid safe gas price '{}': {}", gas_response.result.safe_gas_price, e)))?;

        let average = gas_response.result.propose_gas_price
            .parse::<f64>()
            .map_err(|e| EtherscanError::ApiError(format!("Invalid propose gas price '{}': {}", gas_response.result.propose_gas_price, e)))?;

        let high = gas_response.result.fast_gas_price
            .parse::<f64>()
            .map_err(|e| EtherscanError::ApiError(format!("Invalid fast gas price '{}': {}", gas_response.result.fast_gas_price, e)))?;
            
        println!("✅ Parsed gas prices: low={:.6}, average={:.6}, high={:.6}", low, average, high);

        Ok(GasPrice {
            low,
            average,
            high,
            timestamp: chrono::Utc::now(),
        })
    }
}