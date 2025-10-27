//! Centralized error handling using anyhow.
//!
//! This module provides unified error handling across the application using the anyhow crate
//! for better error context, chaining, and debugging. It includes utilities for logging errors
//! and converting between different error types.

use anyhow::{Context, Result as AnyhowResult};
use axum::{
    response::{IntoResponse, Response},
    http::StatusCode,
    Json,
};
use serde_json::json;
use tracing::error;

/// Type alias for Result with anyhow::Error
pub type Result<T> = AnyhowResult<T>;

/// Application-specific error context for better debugging
pub trait ErrorContext<T> {
    /// Add context about crypto operations
    fn crypto_context(self, operation: &str) -> Result<T>;
    
    /// Add context about gas operations
    fn gas_context(self, operation: &str) -> Result<T>;
    
    /// Add context about API operations
    fn api_context(self, operation: &str) -> Result<T>;
    
    /// Add context about configuration operations
    fn config_context(self, operation: &str) -> Result<T>;
}

impl<T, E> ErrorContext<T> for std::result::Result<T, E>
where
    E: Into<anyhow::Error>,
{
    fn crypto_context(self, operation: &str) -> Result<T> {
        self.map_err(|e| e.into())
            .with_context(|| format!("Crypto operation failed: {}", operation))
    }
    
    fn gas_context(self, operation: &str) -> Result<T> {
        self.map_err(|e| e.into())
            .with_context(|| format!("Gas operation failed: {}", operation))
    }
    
    fn api_context(self, operation: &str) -> Result<T> {
        self.map_err(|e| e.into())
            .with_context(|| format!("API operation failed: {}", operation))
    }
    
    fn config_context(self, operation: &str) -> Result<T> {
        self.map_err(|e| e.into())
            .with_context(|| format!("Configuration error: {}", operation))
    }
}

/// Application error wrapper for HTTP responses
pub struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // Log the full error chain for debugging
        utils::log_error(&self.0, "HTTP request processing");
        
        // Determine status code based on error type
        let status_code = if self.0.downcast_ref::<reqwest::Error>().is_some() {
            StatusCode::BAD_GATEWAY
        } else if self.0.to_string().contains("API key") || self.0.to_string().contains("configuration") {
            StatusCode::SERVICE_UNAVAILABLE
        } else if self.0.to_string().contains("parse") || self.0.to_string().contains("invalid") {
            StatusCode::BAD_REQUEST
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        };
        
        // Create user-friendly error response
        let error_response = json!({
            "error": {
                "message": "An error occurred while processing your request",
                "details": self.0.to_string(),
                "status": status_code.as_u16()
            }
        });
        
        (status_code, Json(error_response)).into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

/// Utility functions for error handling
pub mod utils {
    use super::*;
    
    /// Log an error with full context chain
    pub fn log_error(error: &anyhow::Error, operation: &str) {
        error!("Error in {}: {}", operation, error);
        
        // Log the full error chain for debugging
        let mut source = error.source();
        let mut depth = 1;
        while let Some(err) = source {
            error!("  Caused by ({}): {}", depth, err);
            source = err.source();
            depth += 1;
        }
    }
    
    /// Create a standardized error for missing configuration
    pub fn missing_config_error(config_name: &str) -> anyhow::Error {
        anyhow::anyhow!("Missing required configuration: {}", config_name)
    }
    
    /// Create a standardized error for API failures
    pub fn api_error(provider: &str, message: &str) -> anyhow::Error {
        anyhow::anyhow!("API error from {}: {}", provider, message)
    }
    
    /// Create a standardized error for parsing failures
    pub fn parse_error(data_type: &str, source: impl std::error::Error + Send + Sync + 'static) -> anyhow::Error {
        anyhow::Error::new(source).context(format!("Failed to parse {}", data_type))
    }
}

/// Macro for creating context-aware errors
#[macro_export]
macro_rules! bail_context {
    ($context:expr, $($arg:tt)*) => {
        return Err(anyhow::anyhow!($($arg)*).context($context))
    };
}

/// Macro for ensuring a condition with context
#[macro_export]
macro_rules! ensure_context {
    ($condition:expr, $context:expr, $($arg:tt)*) => {
        if !$condition {
            bail_context!($context, $($arg)*);
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;
    
    #[test]
    fn test_error_context() {
        let result: std::result::Result<(), std::io::Error> = Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "File not found"
        ));
        
        let with_context = result.crypto_context("fetching price data");
        assert!(with_context.is_err());
        assert!(with_context.unwrap_err().to_string().contains("Crypto operation failed"));
    }
    
    #[test]
    fn test_app_error_conversion() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "test error");
        let app_error: AppError = anyhow!(io_error).into();
        
        // Should not panic
        let _response = app_error.into_response();
    }
}