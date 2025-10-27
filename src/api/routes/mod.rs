//! API route handlers module.
//!
//! This module contains all the HTTP endpoint handlers for the Boltzmann API server.
//! Routes are organized by functionality (price prices, gas estimates) and use shared app state.

pub mod crypto;
pub mod gas;
pub mod health;
mod fees;
mod subscriptions;

use axum::{Router, routing::get};
use crate::core::config::AppState;
use crate::api::docs::swagger;

/// Creates the main application router with all routes configured.
///
/// This function sets up all the API endpoints using clear, RESTful patterns:
/// - `/api/v1/price/prices` - Cryptocurrency price quotes
/// - `/api/v1/gas/prices` - Gas price estimates
/// - `/api/v1/health` - Health check endpoint
/// - `/docs` - Swagger UI documentation
///
/// # Arguments
///
/// * `app_state` - Shared application state containing configuration
///
/// # Returns
///
/// Configured Axum router ready to serve requests
pub fn create_router(app_state: AppState) -> Router {
    Router::new()
        // API v1 routes
        .route("/api/v1/health", get(health::health_check))
        .route("/api/v1/crypto/prices", get(crypto::get_crypto_prices))
        .route("/api/v1/gas/prices", get(gas::get_gas_estimates))
        // Future endpoints (planned)
        // .route("/api/v1/gas/cost/estimates/native-transfer", get(gas::*))
        // .route("/api/v1/gas/cost/estimates/erc20-transfer", get(gas::*))
        // .route("/api/v1/gas/cost/estimates/nft-transfer", get(gas::*))
        // .route("/api/v1/gas/cost/estimates/call-to-contract", get(gas::*))
        // .route("/api/v1/fee/estimates/native-transfer", get(gas::*))
        // .route("/api/v1/fee/estimates/erc20-transfer", get(gas::*))
        // .route("/api/v1/fee/estimates/nft-transfer", get(gas::*))
        // .route("/api/v1/fee/estimates/call-to-contract", get(gas::*))
        // .route("/api/v1/subscriptions/price/prices", post(create_crypto_price_subscription)
        // .route("/api/v1/subscriptions/price/prices", get(get_crypto_price_subscription)
        // .route("/api/v1/subscriptions/gas/estimates", post(create_gas_estimates_subscription)
        // .route("/api/v1/subscriptions/gas/estimates", get(get_gas_estimates_subscription)
        // Documentation
        .merge(swagger::swagger_ui())
        .with_state(app_state)
}