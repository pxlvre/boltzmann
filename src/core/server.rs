//! Server setup and configuration module.
//!
//! This module handles the Axum server setup, middleware configuration,
//! and the main server lifecycle.

use std::error::Error;
use tokio::net::TcpListener;
use tracing_subscriber;

use crate::core::config::{Config, AppState};
use crate::api::routes;

/// Initialize and start the Boltzmann API server.
///
/// This function handles the complete server lifecycle:
/// 1. Load configuration from environment
/// 2. Initialize tracing for logging
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
    // Initialize tracing for logging
    tracing_subscriber::fmt::init();
    
    // Load configuration from environment
    let config = Config::from_env()?;
    let app_state = AppState::new(config);
    
    // Create router with all routes configured
    let app = routes::create_router(app_state.clone());

    // Determine bind address from configuration
    let bind_addr = format!("{}:{}", app_state.config.host, app_state.config.port);
    println!("🚀 Starting Boltzmann API server on {}", bind_addr);
    
    // Bind to the address and start serving
    let listener = TcpListener::bind(&bind_addr).await?;
    axum::serve(listener, app.into_make_service()).await?;

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