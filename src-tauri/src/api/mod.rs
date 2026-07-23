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
pub mod users;
pub mod wallet;

use std::sync::Arc;
use std::sync::OnceLock;

use axum::{
	middleware,
	routing::{delete, get, patch, post, put},
	Router,
};
use tauri::AppHandle;

use crate::domain::product::ProductRepository;
use crate::domain::order::OrderRepository;
use crate::domain::user::UserRepository;
use crate::domain::wallet::WalletRepository;

/// Global Tauri AppHandle, set once during setup().
/// Required by settings handlers to access tauri_plugin_store.
static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

/// Store the AppHandle for later use by API handlers.
/// Must be called once during Tauri setup().
pub fn init_app_handle(handle: AppHandle) {
	let _ = APP_HANDLE.set(handle);
}

/// Retrieve the stored AppHandle.
pub fn app_handle() -> &'static AppHandle {
	APP_HANDLE
		.get()
		.expect("AppHandle not initialized — call init_app_handle() in setup")
}

/// Shared state for all API handlers.
#[derive(Clone)]
pub struct AppApiState {
	pub user_repo: Arc<dyn UserRepository>,
	pub order_repo: Arc<dyn OrderRepository>,
	pub wallet_repo: Arc<dyn WalletRepository>,
	pub product_repo: Arc<dyn ProductRepository>,
	pub jwt_secret: Arc<Vec<u8>>,
}

/// Build the full application router including API routes, static files, and middleware.
///
/// This function takes the full app state and constructs the complete router,
/// avoiding the need to nest stateful routers.
pub fn build_app(state: AppApiState) -> Router {
	let dist_path = resolve_dist_path();

	let api_routes = Router::new()
		// Auth (stories 1.3, 1.5)
		.route("/api/auth/register", axum::routing::post(auth::register))
		.route("/api/auth/login", axum::routing::post(auth::login))
		.route("/api/auth/logout", axum::routing::post(auth::logout))
		.route("/api/auth/me", axum::routing::get(crate::api::auth::me))
		// Health
		.route("/api/health", axum::routing::get(health::health_check))
		// Settings (story 1.4)
		.route("/api/settings", get(settings::get_settings))
		.route("/api/settings", patch(settings::patch_settings))
		.route("/api/settings", delete(settings::reset_settings))
		// Users CRUD (story 1.5)
		.route("/api/users", get(users::list_users))
		.route("/api/users", post(users::create_user))
		.route("/api/users/{id}", patch(users::update_user))
		.route("/api/users/{id}", delete(users::delete_user))
		// Wallet API (story 1.5.2)
		.route(
			"/api/wallet/register",
			post(crate::api::wallet::register),
		)
		.route(
			"/api/wallet/by-phone/{phone}",
			get(crate::api::wallet::get_by_phone),
		)
		.route(
			"/api/wallet/{id}/ledger",
			get(crate::api::wallet::get_ledger),
		)
		// Products CRUD (story 3.1)
		.route("/api/products", get(products::list_products))
		.route("/api/products", post(products::create_product))
		.route("/api/products/{id}", get(products::get_product))
		.route("/api/products/{id}", put(products::update_product))
		.route(
			"/api/products/{id}",
			delete(products::delete_product),
		)
		// Categories CRUD (story 3.1)
		.route("/api/categories", get(products::list_categories))
		.route("/api/categories", post(products::create_category))
		.route("/api/categories/{id}", get(products::get_category))
		.route(
			"/api/categories/{id}",
			put(products::update_category),
		)
		.route(
			"/api/categories/{id}",
			delete(products::delete_category),
		)
		// Orders CRUD (story 3.2)
		.route("/api/orders", post(orders::create_order))
		.route("/api/orders", get(orders::list_orders))
		.route("/api/orders/{id}/status", patch(orders::update_order_status))
		.route("/api/orders/{id}/items", post(orders::add_order_item))
		.route("/api/orders/{id}/items/{item_id}", delete(orders::remove_order_item))
		.route("/api/orders/{id}", get(orders::get_order))
		;

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
