//! Users API — admin CRUD for user management.
//!
//! AD-11: Only admins (Permission::ManageUsers) may access these endpoints.
//! AC-8: GET /api/users, POST /api/users, PATCH /api/users/{id}, DELETE /api/users/{id}

use axum::{
	extract::{Extension, Path, State},
	http::StatusCode,
	response::IntoResponse,
	Json,
};
use serde::{Deserialize, Serialize};

use crate::domain::crypto;
use crate::domain::user::{Role, User, UserRepository};

use super::auth_middleware::AuthUser;
use super::AppApiState;

// ─── Response types ────────────────────────────────────────────────

#[derive(Serialize)]
pub struct UserResponse {
	pub id: String,
	pub email: String,
	pub name: String,
	pub role: String,
	pub created_at: String,
}

#[derive(Serialize)]
pub(crate) struct ApiError {
	error: String,
	code: String,
}

// ─── Request types ─────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct CreateUserRequest {
	pub email: String,
	pub password: String,
	#[serde(default)]
	pub name: Option<String>,
	#[serde(default = "default_role")]
	pub role: String,
}

#[derive(Deserialize)]
pub struct UpdateUserRequest {
	#[serde(default)]
	pub email: Option<String>,
	#[serde(default)]
	pub name: Option<String>,
	#[serde(default)]
	pub role: Option<String>,
	#[serde(default)]
	pub password: Option<String>,
}

fn default_role() -> String {
	"caissier".to_string()
}

// ─── Helpers ────────────────────────────────────────────────────────

fn to_response(user: &User) -> UserResponse {
	UserResponse {
		id: user.id.clone(),
		email: user.email.clone(),
		name: user.name.clone(),
		role: user.role.as_str().to_string(),
		created_at: user.created_at.clone(),
	}
}

fn error_response(error: &str, code: &str, status: StatusCode) -> (StatusCode, Json<ApiError>) {
	(status, Json(ApiError {
		error: error.to_string(),
		code: code.to_string(),
	}))
}

fn validate_email(email: &str) -> Result<String, (StatusCode, Json<ApiError>)> {
	let email = email.trim().to_lowercase();
	if email.is_empty() {
		return Err(error_response("Email is required", "VALIDATION_ERROR", StatusCode::UNPROCESSABLE_ENTITY));
	}
	let parts: Vec<&str> = email.splitn(2, '@').collect();
	if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() || !parts[1].contains('.') {
		return Err(error_response("Invalid email format", "VALIDATION_ERROR", StatusCode::UNPROCESSABLE_ENTITY));
	}
	Ok(email)
}

fn validate_password(password: &str) -> Result<(), (StatusCode, Json<ApiError>)> {
	if password.len() < 8 {
		return Err(error_response("Password must be at least 8 characters", "VALIDATION_ERROR", StatusCode::UNPROCESSABLE_ENTITY));
	}
	Ok(())
}

fn validate_role(role: &str) -> Result<Role, (StatusCode, Json<ApiError>)> {
	Role::from_str(role).map_err(|_| {
		error_response(
			"Invalid role. Must be: admin, caissier, vendeur, or gestionnaire_stock",
			"VALIDATION_ERROR",
			StatusCode::UNPROCESSABLE_ENTITY,
		)
	})
}

fn uuid_v7() -> String {
	use uuid::Uuid;
	Uuid::now_v7().to_string()
}

fn chrono_now() -> String {
	use chrono::Utc;
	Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
}

// ─── Handlers ───────────────────────────────────────────────────────

/// GET /api/users — list all users (admin only)
pub async fn list_users(
	State(state): State<AppApiState>,
) -> Result<Json<Vec<UserResponse>>, (StatusCode, Json<ApiError>)> {
	let users = state.user_repo.list_all()
		.map_err(|e| error_response(&e.to_string(), "INTERNAL_ERROR", StatusCode::INTERNAL_SERVER_ERROR))?;

	Ok(Json(users.iter().map(to_response).collect()))
}

/// POST /api/users — create a new user with a specific role (admin only)
pub async fn create_user(
	State(state): State<AppApiState>,
	Json(body): Json<CreateUserRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiError>)> {
	let email = validate_email(&body.email)?;
	validate_password(&body.password)?;
	let role = validate_role(&body.role)?;

	// Check for duplicate email
	if let Ok(Some(_)) = state.user_repo.find_by_email(&email) {
		return Err(error_response("Email already registered", "DUPLICATE_EMAIL", StatusCode::CONFLICT));
	}

	let password_hash = crypto::hash_password(&body.password)
		.map_err(|e| error_response(&e.to_string(), "INTERNAL_ERROR", StatusCode::INTERNAL_SERVER_ERROR))?;

	let now = chrono_now();
	let user = User {
		id: uuid_v7(),
		email,
		password_hash,
		name: body.name.unwrap_or_default(),
		role,
		created_at: now.clone(),
		updated_at: now,
	};

	state.user_repo.create(&user)
		.map_err(|e| {
			let msg = if e.to_string().contains("UNIQUE") || e.to_string().contains("duplicate") {
				"Email already registered"
			} else {
				"Failed to create user"
			};
			error_response(msg, "CREATION_ERROR", StatusCode::CONFLICT)
		})?;

	tracing::info!(target: "auth", "User created: email={}, role={}", user.email, user.role.as_str());

	Ok((StatusCode::CREATED, Json(to_response(&user))))
}

/// PATCH /api/users/{id} — update user fields (admin only)
pub async fn update_user(
	State(state): State<AppApiState>,
	Path(user_id): Path<String>,
	Json(body): Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>, (StatusCode, Json<ApiError>)> {
	let mut user = state.user_repo.find_by_id(&user_id)
		.map_err(|e| error_response(&e.to_string(), "INTERNAL_ERROR", StatusCode::INTERNAL_SERVER_ERROR))?
		.ok_or_else(|| error_response("User not found", "NOT_FOUND", StatusCode::NOT_FOUND))?;

	// Apply partial updates
	if let Some(email) = body.email {
		let email = validate_email(&email)?;
		// Check email is not taken by another user
		if let Ok(Some(existing)) = state.user_repo.find_by_email(&email) {
			if existing.id != user.id {
				return Err(error_response("Email already in use", "DUPLICATE_EMAIL", StatusCode::CONFLICT));
			}
		}
		user.email = email;
	}

	if let Some(name) = body.name {
		user.name = name;
	}

	if let Some(role_str) = body.role {
		let role = validate_role(&role_str)?;
		user.role = role;
	}

	if let Some(password) = body.password {
		validate_password(&password)?;
		user.password_hash = crypto::hash_password(&password)
			.map_err(|e| error_response(&e.to_string(), "INTERNAL_ERROR", StatusCode::INTERNAL_SERVER_ERROR))?;
	}

	user.updated_at = chrono_now();

	state.user_repo.update(&user)
		.map_err(|e| error_response(&e.to_string(), "UPDATE_ERROR", StatusCode::INTERNAL_SERVER_ERROR))?;

	tracing::info!(target: "auth", "User updated: id={}, email={}, role={}", user.id, user.email, user.role.as_str());

	Ok(Json(to_response(&user)))
}

/// DELETE /api/users/{id} — delete a user (admin only)
pub async fn delete_user(
	State(state): State<AppApiState>,
	Path(user_id): Path<String>,
	Extension(auth): Extension<AuthUser>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiError>)> {
	// Prevent self-deletion
	if auth.id == user_id {
		return Err(error_response("Cannot delete yourself", "SELF_DELETE", StatusCode::BAD_REQUEST));
	}

	// Prevent deleting the last admin
	let user = state.user_repo.find_by_id(&user_id)
		.map_err(|e| error_response(&e.to_string(), "INTERNAL_ERROR", StatusCode::INTERNAL_SERVER_ERROR))?
		.ok_or_else(|| error_response("User not found", "NOT_FOUND", StatusCode::NOT_FOUND))?;

	if user.role == Role::Admin {
		let admin_count = state.user_repo.list_all()
			.map_err(|e| error_response(&e.to_string(), "INTERNAL_ERROR", StatusCode::INTERNAL_SERVER_ERROR))?
			.iter()
			.filter(|u| u.role == Role::Admin)
			.count();

		if admin_count <= 1 {
			return Err(error_response("Cannot delete the last admin", "LAST_ADMIN", StatusCode::BAD_REQUEST));
		}
	}

	state.user_repo.delete(&user_id)
		.map_err(|e| error_response(&e.to_string(), "DELETE_ERROR", StatusCode::INTERNAL_SERVER_ERROR))?;

	tracing::info!(target: "auth", "User deleted: id={}, email={}", user_id, user.email);

	Ok((StatusCode::OK, Json(serde_json::json!({"status": "deleted"}))))
}
