//! Etherscan Gas API provider implementation.
//!
//! This module implements gas price fetching using the Etherscan Gas Tracker API.

use super::{GasOracle, GasPrice};
use crate::core::errors::{Result, ErrorContext};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use anyhow::Context;


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
    /// Creates a new Etherscan gas oracle instance with the provided API key.
    ///
    /// # Arguments
    ///
    /// * `api_key` - The Etherscan API key to use for requests
    ///
    /// # Errors
    ///
    /// Returns `EtherscanError::MissingApiKey` if the API key is empty.
    pub fn new(api_key: String) -> Result<Self> {
        if api_key.is_empty() {
            anyhow::bail!("Etherscan API key cannot be empty");
        }

        Ok(Self {
            client: Client::new(),
            api_key,
            base_url: "https://api.etherscan.io/v2/api".to_string(),
        })
    }

}

#[async_trait]
impl GasOracle for EtherscanGasOracle {
    type Error = anyhow::Error;

    async fn get_gas_prices(&self) -> std::result::Result<GasPrice, Self::Error> {
        let url = format!(
            "{}?chainid=1&module=gastracker&action=gasoracle&apikey={}",
            self.base_url, self.api_key
        );

        println!("🔗 Etherscan API URL: {}", url);
        
        let response = self.client.get(&url).send().await
            .gas_context("sending request to Etherscan API")?;
        let body = response.text().await
            .gas_context("reading response body from Etherscan API")?;
        
        println!("📨 Raw API response: {}", body);
        
        let gas_response: EtherscanGasResponse = serde_json::from_str(&body)
            .context("parsing JSON response from Etherscan API")?;
        
        println!("🔍 Parsed response: {:?}", gas_response);

        if gas_response.status != "1" {
            anyhow::bail!("Etherscan API error: {}", gas_response.message);
        }

        // Parse gas prices from decimal strings to f64 (preserve precision)
        println!("💰 Parsing gas prices:");
        println!("   SafeGasPrice: '{}'", gas_response.result.safe_gas_price);
        println!("   ProposeGasPrice: '{}'", gas_response.result.propose_gas_price);
        println!("   FastGasPrice: '{}'", gas_response.result.fast_gas_price);
        
        let low = gas_response.result.safe_gas_price
            .parse::<f64>()
            .with_context(|| format!("Invalid safe gas price '{}'", gas_response.result.safe_gas_price))?;

        let average = gas_response.result.propose_gas_price
            .parse::<f64>()
            .with_context(|| format!("Invalid propose gas price '{}'", gas_response.result.propose_gas_price))?;

        let high = gas_response.result.fast_gas_price
            .parse::<f64>()
            .with_context(|| format!("Invalid fast gas price '{}'", gas_response.result.fast_gas_price))?;
            
        println!("✅ Parsed gas prices: low={:.6}, average={:.6}, high={:.6}", low, average, high);

        Ok(GasPrice {
            low,
            average,
            high,
            timestamp: chrono::Utc::now(),
        })
    }
}