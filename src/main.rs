//! Boltzmann API server
//!
//! A gas and fee analytics API for EVM chains providing:
//! - Real-time gas price estimates from multiple oracles
//! - Cryptocurrency price feeds from major providers
//! - Multi-currency support with configurable amounts
//!
//! The server follows modular Axum patterns with centralized configuration
//! and comprehensive error handling for production use.

mod domains;
mod api;
mod core;
mod infrastructure;

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    core::server::start().await
}

#[cfg(target_arch = "wasm32")]
fn main() {}