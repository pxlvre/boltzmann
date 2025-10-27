//! Gas price endpoints.
//!
//! This module handles requests for Ethereum gas prices from multiple oracle providers.
//! Supports both Etherscan and Alloy (direct RPC) providers with configurable selection.

use axum::{extract::{Query, State}, response::IntoResponse, http::StatusCode, Json};
use serde::Deserialize;
use utoipa::IntoParams;
use tracing::{info, warn, error};

use crate::core::config::AppState;
use crate::domains::gas::price::{GasOracle, GasQuote, GasOracleSource};
use crate::domains::gas::price::etherscan::EtherscanGasOracle;
use crate::domains::gas::price::alloy::AlloyGasOracle;

/// Query parameters for gas price requests.
#[derive(Deserialize, IntoParams)]
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
#[utoipa::path(
    get,
    path = "/api/v1/gas/prices",
    tag = "gas",
    params(GasPriceQueryParams),
    responses(
        (status = 200, description = "Successful response with gas price estimates", body = GasQuote),
        (status = 500, description = "Failed to fetch gas prices from provider")
    )
)]
pub async fn get_gas_estimates(
    State(app_state): State<AppState>,
    Query(params): Query<GasPriceQueryParams>,
) -> impl IntoResponse {
    info!("⛽ Fetching gas prices from {:?} provider", params.provider);

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
                                    warn!("Etherscan gas oracle failed: {}", e);
                                    None
                                }
                            }
                        }
                        Err(e) => {
                            error!("Etherscan gas oracle initialization failed: {}", e);
                            None
                        }
                    }
                }
                None => {
                    info!("Etherscan API key not configured, provider unavailable");
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
                                    warn!("Alloy gas oracle failed: {}", e);
                                    None
                                }
                            }
                        }
                        Err(e) => {
                            error!("Alloy gas oracle initialization failed: {}", e);
                            None
                        }
                    }
                }
                None => {
                    error!("Ethereum RPC URL not configured, Alloy provider unavailable");
                    None
                }
            }
        }
    };

    info!("Gas price fetching completed. Success: {}", gas_quote.is_some());

    match gas_quote {
        Some(quote) => (StatusCode::OK, Json(serde_json::to_value(quote).unwrap_or_default())),
        None => {
            let error_json = serde_json::json!({"error": "Failed to fetch gas prices from provider"});
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_json))
        }
    }
}