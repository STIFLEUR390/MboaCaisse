//! JWT authentication middleware for Axum.
//!
//! AD-11: JWT cookie, 24h expiry, silent refresh if <1h remaining.
//! Permission check via required_permission() for role-based access control.
//! Only /api/health, /api/auth/register, and /api/auth/login are public.
//! All other /api/* routes require a valid JWT and appropriate permissions.

use axum::{
	extract::{Request, State},
	middleware::Next,
	response::{IntoResponse, Response},
};
use axum::http::{header, HeaderValue, StatusCode};
use serde::Serialize;

use crate::domain::user::{Permission, Role};
use crate::domain::jwt;

use super::AppApiState;

#[derive(Debug, Clone)]
pub struct AuthUser {
	pub id: String,
	pub email: String,
	pub role: String,
}

#[derive(Serialize)]
struct AuthError {
	error: String,
	code: String,
}

impl AuthError {
	fn new(error: impl Into<String>, code: impl Into<String>) -> Self {
		Self { error: error.into(), code: code.into() }
	}
}

/// Determine the permission required for a given API route path.
///
/// Returns `None` for public routes and routes where mere authentication suffices.
/// Returns `Some(Permission)` for routes that require a specific permission.
fn required_permission(path: &str) -> Option<Permission> {
	// Public routes — no auth needed
	if is_public_path(path) {
		return None;
	}
	// Routes requiring only authentication (any valid JWT)
	if path == "/api/auth/logout" || path == "/api/auth/me" {
		return None;
	}
	// Route → permission mapping
	if path == "/api/users" || path.starts_with("/api/users/") { return Some(Permission::ManageUsers); }
	if path == "/api/settings" || path.starts_with("/api/settings/") { return Some(Permission::ManageSettings); }
	if path == "/api/products" || path.starts_with("/api/products/") || path == "/api/categories" || path.starts_with("/api/categories/") { return Some(Permission::ManageMenu); }
	if path == "/api/orders" || path.starts_with("/api/orders/") { return Some(Permission::ViewOrders); }
	if path == "/api/payments" || path.starts_with("/api/payments/") { return Some(Permission::Sell); }
	if path == "/api/wallet" || path.starts_with("/api/wallet/") { return Some(Permission::Sell); }
	if path == "/api/kitchen" || path.starts_with("/api/kitchen/") { return Some(Permission::ViewOrders); }
	if path == "/api/reports" || path.starts_with("/api/reports/") { return Some(Permission::ViewReports); }
	if path == "/api/stock" || path.starts_with("/api/stock/") { return Some(Permission::ManageStock); }
	// Fallback: unknown /api/* routes require admin
	if path.starts_with("/api/") { return Some(Permission::All); }
	None
}

/// Returns true if the path does NOT require JWT authentication.
/// Non-API routes (static files, SPA) are always public.
pub fn is_public_path(path: &str) -> bool {
	if !path.starts_with("/api/") {
		return true;
	}
	// Only register, login, and health are public.
	// logout, me, and all other API routes require authentication.
	path == "/api/health"
		|| path == "/api/auth/register"
		|| path == "/api/auth/login"
}

pub async fn auth_middleware(
	State(state): State<AppApiState>,
	mut req: Request,
	next: Next,
) -> Result<impl IntoResponse, Response> {
	if is_public_path(req.uri().path()) {
		return Ok(next.run(req).await);
	}

	let cookie_header = req.headers()
		.get(header::COOKIE)
		.and_then(|v| v.to_str().ok())
		.unwrap_or("");

	let token = extract_cookie(cookie_header, "mboa_session");
	let token = match token {
		Some(t) => t.to_string(),
		None => return Err(unauthorized_response("Authentication required", "UNAUTHORIZED")),
	};

	let claims = match jwt::decode_token(&token, &state.jwt_secret) {
		Ok(c) => c,
		Err(err) => return match err.as_str() {
			"TOKEN_EXPIRED" => Err(unauthorized_response("Token expired", "TOKEN_EXPIRED")),
			_ => Err(unauthorized_response("Invalid token", "INVALID_TOKEN")),
		},
	};

	req.extensions_mut().insert(AuthUser {
		id: claims.sub.clone(),
		email: claims.email.clone(),
		role: claims.role.clone(),
	});

	// Permission check: verify role grants access to this route (story 1.5)
	if let Some(required_perm) = required_permission(req.uri().path()) {
		let role = Role::from_str(&claims.role)
			.map_err(|_| forbidden_response("Invalid role", "INVALID_ROLE"))?;
		if !role.has_permission(&required_perm) {
			tracing::warn!(target: "auth", "Permission denied: user={}, role={}, path={}, required={:?}", claims.sub, claims.role, req.uri().path(), required_perm);
			return Err(forbidden_response("Forbidden", "FORBIDDEN"));
		}
	}

	let should_refresh = claims.should_refresh();
	let mut response = next.run(req).await;

	if should_refresh {
		if let Ok(new_token) = jwt::refresh_token(&token, &state.jwt_secret) {
			let cookie = format!("mboa_session={}; Path=/; HttpOnly; SameSite=Lax; Max-Age=86400", new_token);
			response.headers_mut().insert(header::SET_COOKIE, HeaderValue::from_str(&cookie).unwrap());
			response.headers_mut().insert("X-Token-Refreshed", HeaderValue::from_static("true"));
		}
	}

	Ok(response)
}

fn extract_cookie<'a>(cookie_header: &'a str, name: &str) -> Option<&'a str> {
	for pair in cookie_header.split(';') {
		let pair = pair.trim();
		if let Some((key, value)) = pair.split_once('=') {
			if key.trim() == name { return Some(value.trim()); }
		}
	}
	None
}

fn unauthorized_response(error: &str, code: &str) -> Response {
	let body = axum::Json(AuthError::new(error, code));
	let mut response = body.into_response();
	*response.status_mut() = StatusCode::UNAUTHORIZED;
	response
}

fn forbidden_response(error: &str, code: &str) -> Response {
	let body = axum::Json(AuthError::new(error, code));
	let mut response = body.into_response();
	*response.status_mut() = StatusCode::FORBIDDEN;
	response
}
