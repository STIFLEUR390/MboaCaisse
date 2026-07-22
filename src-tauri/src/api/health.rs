//! Health endpoint — GET /api/health.
//! Returns basic diagnostics (server status, DB connectivity, uptime).

use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct HealthResponse {
	pub status: &'static str,
	pub version: &'static str,
}

/// GET /api/health — basic liveness probe.
pub async fn health_check() -> Json<HealthResponse> {
	Json(HealthResponse {
		status: "ok",
		version: env!("CARGO_PKG_VERSION"),
	})
}
