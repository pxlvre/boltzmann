//! Health check endpoints.
//!
//! This module handles health monitoring and service status endpoints
//! for load balancers, monitoring systems, and operational readiness checks.

/// Health check endpoint for monitoring and load balancer probes.
///
/// Returns a simple string response indicating the API service is running.
/// This endpoint can be used by:
/// - Load balancers for health checks
/// - Monitoring systems for uptime verification  
/// - Container orchestration platforms for readiness probes
#[utoipa::path(
    get,
    path = "/api/v1/health",
    tag = "health",
    responses(
        (status = 200, description = "API is running", body = String)
    )
)]
pub async fn health_check() -> &'static str {
    "Boltzmann API is running"
}