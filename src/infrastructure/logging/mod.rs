//! Logging and tracing infrastructure.
//!
//! This module provides structured logging capabilities using the `tracing` ecosystem.
//! It includes configuration for different log levels, output formats, and filtering.

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, fmt};
use std::env;

/// Logging configuration levels
#[derive(Debug, Clone)]
pub enum LogLevel {
    /// Only error messages
    Error,
    /// Error and warning messages  
    Warn,
    /// Error, warning, and info messages (default)
    Info,
    /// All messages including debug
    Debug,
    /// All messages including trace (very verbose)
    Trace,
}

impl LogLevel {
    /// Convert LogLevel to tracing filter string
    pub fn as_filter(&self) -> &'static str {
        match self {
            LogLevel::Error => "error",
            LogLevel::Warn => "warn", 
            LogLevel::Info => "info",
            LogLevel::Debug => "debug",
            LogLevel::Trace => "trace",
        }
    }
}

impl Default for LogLevel {
    fn default() -> Self {
        LogLevel::Info
    }
}

/// Logging configuration options
#[derive(Debug, Clone)]
pub struct LogConfig {
    /// The logging level to use
    pub level: LogLevel,
    /// Whether to enable JSON formatted logs (useful for production)
    pub json_format: bool,
    /// Whether to include timestamps in logs
    pub with_timestamps: bool,
    /// Whether to include target information (module names)
    pub with_target: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::default(),
            json_format: false,
            with_timestamps: true,
            with_target: true,
        }
    }
}

impl LogConfig {
    /// Create a new LogConfig from environment variables
    pub fn from_env() -> Self {
        let level = match env::var("RUST_LOG").unwrap_or_default().to_lowercase().as_str() {
            "error" => LogLevel::Error,
            "warn" => LogLevel::Warn,
            "info" => LogLevel::Info,
            "debug" => LogLevel::Debug,
            "trace" => LogLevel::Trace,
            _ => LogLevel::default(),
        };

        let json_format = env::var("LOG_FORMAT")
            .unwrap_or_default()
            .to_lowercase() == "json";

        Self {
            level,
            json_format,
            with_timestamps: true,
            with_target: true,
        }
    }
}

/// Initialize the global tracing subscriber with the given configuration.
/// 
/// This should be called once at application startup, before any logging occurs.
/// 
/// # Arguments
/// 
/// * `config` - The logging configuration to use
/// 
/// # Examples
/// 
/// ```rust
/// use boltzmann::infrastructure::logging::{init_tracing, LogConfig};
/// 
/// // Initialize with default configuration
/// init_tracing(LogConfig::default()).expect("Failed to initialize logging");
/// 
/// // Initialize from environment
/// init_tracing(LogConfig::from_env()).expect("Failed to initialize logging");
/// ```
pub fn init_tracing(config: LogConfig) -> Result<(), Box<dyn std::error::Error>> {
    // Create the base EnvFilter
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(config.level.as_filter()));

    if config.json_format {
        // JSON format for production/structured logging
        tracing_subscriber::registry()
            .with(env_filter)
            .with(
                fmt::layer()
                    .json()
                    .with_target(config.with_target)
            )
            .try_init()?;
    } else {
        // Human-readable format for development
        tracing_subscriber::registry()
            .with(env_filter)
            .with(
                fmt::layer()
                    .with_target(config.with_target)
            )
            .try_init()?;
    }

    Ok(())
}

/// Initialize tracing with sensible defaults.
/// 
/// This is a convenience function that initializes tracing with:
/// - Log level from RUST_LOG environment variable (default: info)
/// - Human-readable format for development
/// - Timestamps and target information enabled
pub fn init_default_tracing() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing(LogConfig::from_env())
}