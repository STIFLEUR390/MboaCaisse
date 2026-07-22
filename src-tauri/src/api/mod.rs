//! API layer — thin HTTP handlers.

pub mod auth;
pub mod auth_middleware;
pub mod health;
pub mod kitchen;
pub mod orders;
pub mod payments;
pub mod products;
pub mod reports;
pub mod settings;
pub mod wallet;

use std::sync::Arc;

use axum::{
	middleware,
	routing::{get, post},
	Router,
};

/// Shared state for all API handlers.
#[derive(Clone)]
pub struct AppApiState {
	pub user_repo: Arc<dyn crate::domain::user::UserRepository>,
	pub jwt_secret: Arc<Vec<u8>>,
}

/// Build the full application router including API routes, static files, and middleware.
///
/// This function takes the full app state and constructs the complete router,
/// avoiding the need to nest stateful routers.
pub fn build_app(state: AppApiState) -> Router {
	let dist_path = resolve_dist_path();

	let api_routes = Router::new()
		.route("/api/auth/register", post(auth::register))
		.route("/api/auth/login", post(auth::login))
		.route("/api/auth/logout", post(auth::logout))
		.route("/api/auth/me", get(crate::api::auth::me))
		.route("/api/health", get(health::health_check));

	// Static file serving with SPA fallback.
	if std::path::Path::new(&dist_path).exists() {
		let index_path = format!("{}/index.html", dist_path);
		let fs_serve = tower_http::services::ServeDir::new(&dist_path)
			.append_index_html_on_directories(true)
			.fallback(tower_http::services::ServeFile::new(index_path));

		Router::new()
			.merge(api_routes)
			.fallback_service(fs_serve)
			.layer(middleware::from_fn_with_state(
				state.clone(),
				auth_middleware::auth_middleware,
			))
			.layer(tower_http::compression::CompressionLayer::new())
			.layer(tower_http::cors::CorsLayer::permissive())
			.with_state(state)
	} else {
		Router::new()
			.merge(api_routes)
			.layer(middleware::from_fn_with_state(
				state.clone(),
				auth_middleware::auth_middleware,
			))
			.layer(tower_http::compression::CompressionLayer::new())
			.layer(tower_http::cors::CorsLayer::permissive())
			.with_state(state)
	}
}

/// Resolve the path to the frontend `dist/` directory.
fn resolve_dist_path() -> String {
	if std::path::Path::new("../dist").exists() {
		return "../dist".to_string();
	}
	if std::path::Path::new("dist").exists() {
		return "dist".to_string();
	}
	"dist".to_string()
}
