//! Auth API handlers — register, login, logout, me.
//!
//! AD-11: JWT cookie (mboa_session), argon2, 4 roles.
//! AC-1, AC-2, AC-5, AC-6, AC-7.

use axum::{
	extract::{Extension, FromRef, State},
	http::{header, HeaderMap, HeaderValue, StatusCode},
	response::IntoResponse,
	Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::domain::crypto;
use crate::domain::jwt;
use crate::domain::user::{Role, User, UserRepository};

use super::AppApiState;

// ─── Auth state ─────────────────────────────────────────────────────

#[derive(Clone)]
pub struct AuthState {
	pub user_repo: Arc<dyn UserRepository>,
	pub jwt_secret: Arc<Vec<u8>>,
}

impl FromRef<AppApiState> for AuthState {
	fn from_ref(state: &AppApiState) -> Self {
		Self {
			user_repo: state.user_repo.clone(),
			jwt_secret: state.jwt_secret.clone(),
		}
	}
}

// ─── Request / Response types ───────────────────────────────────────

#[derive(Deserialize)]
pub struct RegisterRequest {
	pub email: String,
	pub password: String,
	pub name: Option<String>,
}

#[derive(Deserialize)]
pub struct LoginRequest {
	pub email: String,
	pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
	pub id: String,
	pub email: String,
	pub name: String,
	pub role: String,
}

#[derive(Serialize)]
pub struct MessageResponse {
	pub message: String,
}

#[derive(Serialize)]
pub(crate) struct ApiError {
	error: String,
	code: String,
}

// ─── Validation ─────────────────────────────────────────────────────

fn validate_email(email: &str) -> Result<(), (StatusCode, Json<ApiError>)> {
	let email = email.trim();
	if email.is_empty() {
		return Err(error_response("Email is required", "VALIDATION_ERROR", StatusCode::UNPROCESSABLE_ENTITY));
	}
	let parts: Vec<&str> = email.splitn(2, '@').collect();
	if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() || !parts[1].contains('.') {
		return Err(error_response("Invalid email format", "VALIDATION_ERROR", StatusCode::UNPROCESSABLE_ENTITY));
	}
	Ok(())
}

fn validate_password(password: &str) -> Result<(), (StatusCode, Json<ApiError>)> {
	if password.len() < 8 {
		return Err(error_response(
			"Password must be at least 8 characters",
			"VALIDATION_ERROR",
			StatusCode::UNPROCESSABLE_ENTITY,
		));
	}
	Ok(())
}

/// Build a HeaderMap with a Set-Cookie for the mboa_session token.
fn session_cookie_header(token: &str, max_age: i32) -> HeaderMap {
	let cookie_value = format!("mboa_session={}; Path=/; HttpOnly; SameSite=Lax; Max-Age={}", token, max_age);
	let mut headers = HeaderMap::new();
	headers.insert(header::SET_COOKIE, HeaderValue::from_str(&cookie_value).unwrap());
	headers
}

// ─── Handlers ───────────────────────────────────────────────────────

/// POST /api/auth/register
pub async fn register(
	State(state): State<AuthState>,
	Json(body): Json<RegisterRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiError>)> {
	let email = body.email.trim().to_lowercase();
	validate_email(&email)?;
	validate_password(&body.password)?;

	if let Ok(Some(_)) = state.user_repo.find_by_email(&email) {
		return Err(error_response("Email already registered", "DUPLICATE_EMAIL", StatusCode::CONFLICT));
	}

	let is_first = state.user_repo.list_all()
		.map(|users| users.is_empty())
		.map_err(|e| error_response(&e.to_string(), "INTERNAL_ERROR", StatusCode::INTERNAL_SERVER_ERROR))?;

	let password_hash = crypto::hash_password(&body.password)
		.map_err(|e| error_response(&e.to_string(), "INTERNAL_ERROR", StatusCode::INTERNAL_SERVER_ERROR))?;

	let now = chrono_now();
	let user = User {
		id: uuid_v7(),
		email,
		password_hash,
		name: body.name.clone().unwrap_or_default(),
		role: if is_first { Role::Admin } else { Role::Caissier },
		created_at: now.clone(),
		updated_at: now,
	};

	state.user_repo.create(&user)
		.map_err(|e| {
			let (msg, code) = if e.to_string().contains("UNIQUE") || e.to_string().contains("duplicate") {
				("Email already registered", "DUPLICATE_EMAIL")
			} else {
				("Registration failed", "REGISTRATION_ERROR")
			};
			error_response(msg, code, StatusCode::CONFLICT)
		})?;

	// Sign JWT and emit cookie (AC-1)
	let token = jwt::encode_token(&user, &state.jwt_secret)
		.map_err(|e| error_response(&e, "INTERNAL_ERROR", StatusCode::INTERNAL_SERVER_ERROR))?;

	let headers = session_cookie_header(&token, 86400);

	Ok((StatusCode::CREATED, headers, Json(AuthResponse {
		id: user.id,
		email: user.email,
		name: user.name,
		role: user.role.as_str().to_string(),
	})))
}

/// POST /api/auth/login
pub async fn login(
	State(state): State<AuthState>,
	Json(body): Json<LoginRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiError>)> {
	let email = body.email.trim().to_lowercase();

	let user = state.user_repo.find_by_email(&email)
		.map_err(|e| error_response(&e.to_string(), "INTERNAL_ERROR", StatusCode::INTERNAL_SERVER_ERROR))?
		.ok_or_else(|| error_response("Invalid email or password", "INVALID_CREDENTIALS", StatusCode::UNAUTHORIZED))?;

	let valid = crypto::verify_password(&body.password, &user.password_hash)
		.map_err(|e| error_response(&e.to_string(), "INTERNAL_ERROR", StatusCode::INTERNAL_SERVER_ERROR))?;

	if !valid {
		return Err(error_response("Invalid email or password", "INVALID_CREDENTIALS", StatusCode::UNAUTHORIZED));
	}

	// Sign JWT and emit cookie (AC-2)
	let token = jwt::encode_token(&user, &state.jwt_secret)
		.map_err(|e| error_response(&e, "INTERNAL_ERROR", StatusCode::INTERNAL_SERVER_ERROR))?;

	let headers = session_cookie_header(&token, 86400);

	Ok((StatusCode::OK, headers, Json(AuthResponse {
		id: user.id,
		email: user.email,
		name: user.name,
		role: user.role.as_str().to_string(),
	})))
}

/// POST /api/auth/logout — destroys the mboa_session cookie (AC-5)
pub async fn logout() -> impl IntoResponse {
	let mut headers = HeaderMap::new();
	headers.insert(
		header::SET_COOKIE,
		HeaderValue::from_static("mboa_session=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0"),
	);
	(headers, Json(MessageResponse {
		message: "Logged out".to_string(),
	}))
}

/// GET /api/auth/me — returns the authenticated user profile
pub async fn me(
	State(state): State<AuthState>,
	Extension(auth): Extension<super::auth_middleware::AuthUser>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<ApiError>)> {
	let user = state.user_repo.find_by_id(&auth.id)
		.map_err(|e| error_response(&e.to_string(), "INTERNAL_ERROR", StatusCode::INTERNAL_SERVER_ERROR))?
		.ok_or_else(|| error_response("User not found", "USER_NOT_FOUND", StatusCode::NOT_FOUND))?;

	Ok(Json(AuthResponse {
		id: user.id,
		email: user.email,
		name: user.name,
		role: user.role.as_str().to_string(),
	}))
}

// ─── Helpers ────────────────────────────────────────────────────────

fn error_response(error: &str, code: &str, status: StatusCode) -> (StatusCode, Json<ApiError>) {
	(status, Json(ApiError {
		error: error.to_string(),
		code: code.to_string(),
	}))
}

fn uuid_v7() -> String {
	use uuid::Uuid;
	Uuid::now_v7().to_string()
}

fn chrono_now() -> String {
	use chrono::Utc;
	Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
}
