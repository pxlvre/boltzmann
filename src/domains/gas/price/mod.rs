//! Gas price oracle implementations.
//!
//! This module provides a unified interface for fetching current gas prices
//! from different providers.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub mod etherscan;
pub mod alloy;

/// Gas price categories for different transaction priorities
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GasPrice {
    /// Low priority gas price (slower confirmation)
    pub low: f64,
    /// Average gas price (standard confirmation)
    pub average: f64,
    /// High priority gas price (faster confirmation)
    pub high: f64,
    /// When this gas price data was fetched
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Gas price provider sources
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum GasOracleSource {
    #[serde(rename = "etherscan")]
    Etherscan,
    #[serde(rename = "alloy")]
    Alloy,
}

/// A gas price quote with provider information
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct GasQuote {
    /// The gas prices for different priority levels
    pub gas_price: GasPrice,
    /// The provider that supplied this quote
    pub provider: GasOracleSource,
}

/// Trait for gas price oracle providers.
///
/// This trait defines the interface that all gas price providers must implement.
/// It allows for fetching current gas prices for low, average, and high priority transactions.
#[async_trait]
pub trait GasOracle {
    /// The error type returned by this provider
    type Error;

    /// Fetches current gas prices for different priority levels.
    ///
    /// # Returns
    ///
    /// A `GasPrice` object containing low, average, and high priority gas prices.
    ///
    /// # Errors
    ///
    /// Returns `Self::Error` if the request fails, the response cannot be parsed,
    /// or the gas price data is unavailable.
    async fn get_gas_prices(&self) -> Result<GasPrice, Self::Error>;
}