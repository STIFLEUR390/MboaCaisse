//! API layer — thin HTTP handlers.
//!
//! Each file in this module is a thin handler that:
//! 1. Parses the incoming request (JSON path params, query string)
//! 2. Calls domain logic via `Arc<dyn XxxRepository>`
//! 3. Serializes the response
//!
//! No business logic lives here.
//!
//! AD-1: api/ is a thin skin — parse, call domain, serialize.
//! AD-7: receives Arc<dyn XxxRepository> (injected during router construction).
//! AD-8: returns (StatusCode, Json<ApiError>) — never leaks DbError or DomainError directly.

pub mod auth;
pub mod health;
pub mod kitchen;
pub mod orders;
pub mod payments;
pub mod products;
pub mod reports;
pub mod settings;
pub mod wallet;

use axum::Router;

/// Build the Axum router for all `/api/*` endpoints.
///
/// Each story adds its own routes as handlers are implemented.
/// For now, only the health endpoint is wired.
pub fn router() -> Router {
	let r = Router::new()
		.route("/health", axum::routing::get(crate::api::health::health_check));

	// Future stories will mount additional routes here.
	r
}
