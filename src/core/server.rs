//! Server setup and configuration module.
//!
//! This module handles the Axum server setup, middleware configuration,
//! and the main server lifecycle.

use std::error::Error;
use tokio::net::TcpListener;
use tracing::{info, error};

use crate::core::config::{Config, AppState};
use crate::api::routes;
use crate::infrastructure::logging;

/// Initialize and start the Boltzmann API server.
///
/// This function handles the complete server lifecycle:
/// 1. Initialize structured logging and tracing
/// 2. Load configuration from environment
/// 3. Create application state
/// 4. Set up routes with middleware
/// 5. Start the server
///
/// # Errors
///
/// Returns an error if:
/// - Configuration loading fails (missing env vars, validation errors)
/// - Server binding fails (port already in use, permission issues)
/// - Any other server startup error occurs
///
/// # Examples
///
/// ```rust
/// use boltzmann::server;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     server::start().await
/// }
/// ```
pub async fn start() -> Result<(), Box<dyn Error>> {
    // Initialize structured logging and tracing
    logging::init_default_tracing()
        .map_err(|e| {
            eprintln!("Failed to initialize logging: {}", e);
            e
        })?;
    
    info!("🔧 Initializing Boltzmann API server");
    
    // Load configuration from environment
    let config = Config::from_env().map_err(|e| {
        error!("Failed to load configuration: {}", e);
        e
    })?;
    
    let app_state = AppState::new(config);
    info!("✅ Configuration loaded successfully");
    
    // Log configuration details
    info!("Configuration details:");
    info!("  Host: {}", app_state.config.host);
    info!("  Port: {}", app_state.config.port);
    info!("  CoinMarketCap API: {}", if app_state.config.coinmarketcap_api_key.is_some() { "✅" } else { "❌" });
    info!("  CoinGecko API: {}", if app_state.config.coingecko_api_key.is_some() { "✅" } else { "❌" });
    info!("  Etherscan API: {}", if app_state.config.etherscan_api_key.is_some() { "✅" } else { "❌" });
    info!("  Ethereum RPC: {}", if app_state.config.ethereum_rpc_url.is_some() { "✅" } else { "❌" });
    
    // Create router with all routes configured
    let app = routes::create_router(app_state.clone());
    info!("🔗 Routes configured successfully");

    // Determine bind address from configuration
    let bind_addr = format!("{}:{}", app_state.config.host, app_state.config.port);
    info!("🚀 Starting Boltzmann API server on {}", bind_addr);
    
    // Bind to the address and start serving
    let listener = TcpListener::bind(&bind_addr).await.map_err(|e| {
        error!("Failed to bind to address {}: {}", bind_addr, e);
        e
    })?;
    
    info!("🌐 Server listening on {}", bind_addr);
    info!("📚 Swagger UI available at: http://{}/docs", bind_addr);
    
    axum::serve(listener, app.into_make_service()).await.map_err(|e| {
        error!("Server error: {}", e);
        e
    })?;

    Ok(())
}

/// Create the application state with loaded configuration.
///
/// This is a convenience function for testing or custom server setups
/// where you want to create app state without starting the full server.
///
/// # Errors
///
/// Returns configuration errors if environment variables are missing or invalid.
pub fn create_app_state() -> Result<AppState, Box<dyn Error>> {
    let config = Config::from_env()?;
    Ok(AppState::new(config))
}