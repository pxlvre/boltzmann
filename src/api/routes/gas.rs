//! Gas price endpoints.
//!
//! This module handles requests for Ethereum gas prices from multiple oracle providers.
//! Supports both Etherscan and Alloy (direct RPC) providers with configurable selection.

use axum::{extract::{Query, State}, response::IntoResponse, http::StatusCode, Json};
use serde::Deserialize;

use crate::core::config::AppState;
use crate::domains::gas::price::{GasOracle, GasQuote, GasOracleSource};
use crate::domains::gas::price::etherscan::EtherscanGasOracle;
use crate::domains::gas::price::alloy::AlloyGasOracle;

/// Query parameters for gas price requests.
#[derive(Deserialize)]
pub struct GasPriceQueryParams {
    /// Gas oracle provider to use (defaults to Etherscan)
    #[serde(default = "default_gas_provider")]
    pub provider: GasOracleSource,
}

fn default_gas_provider() -> GasOracleSource {
    GasOracleSource::Etherscan
}

/// Get current Ethereum gas prices from specified provider.
///
/// This endpoint fetches gas price estimates from the selected oracle provider
/// and returns low/average/high recommendations in Gwei.
///
/// # Query Parameters
///
/// * `provider` - Oracle to use: "etherscan" or "alloy" (default: etherscan)
///
/// # Examples
///
/// * `/api/v1/gas/estimates` - Get gas prices from Etherscan
/// * `/api/v1/gas/estimates?provider=alloy` - Get gas prices from Alloy RPC
///
/// # Returns
///
/// JSON object with gas price quote or error if provider fails.
pub async fn get_gas_estimates(
    State(app_state): State<AppState>,
    Query(params): Query<GasPriceQueryParams>,
) -> impl IntoResponse {
    println!("⛽ Boltzmann Gas Price Fetcher");
    println!("Fetching gas prices from {:?} provider...\n", params.provider);

    let gas_quote = match params.provider {
        GasOracleSource::Etherscan => {
            match &app_state.config.etherscan_api_key {
                Some(api_key) => {
                    match EtherscanGasOracle::new(api_key.clone()) {
                        Ok(oracle) => {
                            match oracle.get_gas_prices().await {
                                Ok(gas_price) => Some(GasQuote {
                                    gas_price,
                                    provider: GasOracleSource::Etherscan,
                                }),
                                Err(e) => {
                                    eprintln!("❌ Etherscan gas oracle failed: {}", e);
                                    None
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("❌ Etherscan gas oracle initialization failed: {}", e);
                            None
                        }
                    }
                }
                None => {
                    eprintln!("❌ Etherscan API key not configured");
                    None
                }
            }
        }
        GasOracleSource::Alloy => {
            match &app_state.config.ethereum_rpc_url {
                Some(rpc_url) => {
                    match AlloyGasOracle::new(rpc_url.clone()) {
                        Ok(oracle) => {
                            match oracle.get_gas_prices().await {
                                Ok(gas_price) => Some(GasQuote {
                                    gas_price,
                                    provider: GasOracleSource::Alloy,
                                }),
                                Err(e) => {
                                    eprintln!("❌ Alloy gas oracle failed: {}", e);
                                    None
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("❌ Alloy gas oracle initialization failed: {}", e);
                            None
                        }
                    }
                }
                None => {
                    eprintln!("❌ Ethereum RPC URL not configured");
                    None
                }
            }
        }
    };

    println!("\n✅ Gas price fetching complete!");

    match gas_quote {
        Some(quote) => (StatusCode::OK, Json(serde_json::to_value(quote).unwrap_or_default())),
        None => {
            let error_json = serde_json::json!({"error": "Failed to fetch gas prices from provider"});
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_json))
        }
    }
}