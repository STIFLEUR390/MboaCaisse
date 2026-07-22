//! Settings API — manage config via tauri_plugin_store.
//!
//! AD-12: Config via Tauri store (port, hostname, backup_interval, headless).
//! Story 1.4.
//!
//! # Endpoints
//! - `GET    /api/settings`          — read all config values
//! - `PATCH  /api/settings`          — update one or more values
//! - `DELETE /api/settings`          — reset all values to defaults

use axum::{
	http::StatusCode,
	response::{IntoResponse, Response},
	Json,
};
use serde::Deserialize;

use crate::api;
use crate::settings::Config;

/// Single key-value response with restart hint.
#[derive(Debug, serde::Serialize)]
pub struct SettingEntry {
	pub key: String,
	pub value: serde_json::Value,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub requires_restart: Option<bool>,
}

/// Full settings response — list of entries.
#[derive(Debug, serde::Serialize)]
pub struct SettingsResponse {
	pub settings: Vec<SettingEntry>,
}

/// Request body for PATCH — partial update.
#[derive(Debug, Deserialize)]
pub struct PatchSettingsBody {
	#[serde(default)]
	pub port: Option<u16>,
	#[serde(default)]
	pub hostname: Option<String>,
	#[serde(default)]
	pub backup_interval_hours: Option<u64>,
	#[serde(default)]
	pub headless: Option<bool>,
}

/// Validation warning returned when a field value is rejected.
#[derive(Debug, serde::Serialize)]
pub struct ValidationWarning {
	pub field: String,
	pub message: String,
}

/// PATCH response with optional warnings.
#[derive(Debug, serde::Serialize)]
pub struct PatchSettingsResponse {
	pub settings: Vec<SettingEntry>,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub warnings: Vec<ValidationWarning>,
}

/// Check whether a string is a valid DNS hostname label.
/// Accepts alphanumeric, hyphens (not at start/end), max 63 chars per label.
fn is_valid_hostname(s: &str) -> bool {
	if s.is_empty() || s.len() > 253 {
		return false;
	}
	s.split('.')
		.all(|label| {
			!label.is_empty()
				&& label.len() <= 63
				&& label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
				&& !label.starts_with('-')
				&& !label.ends_with('-')
		})
}

/// Helper to build the full settings list from a Config.
fn entries_from_config(cfg: &Config) -> Vec<SettingEntry> {
	let pairs = [
		("port", serde_json::json!(cfg.port)),
		("hostname", serde_json::json!(cfg.hostname)),
		("backup_interval_hours", serde_json::json!(cfg.backup_interval_hours)),
		("headless", serde_json::json!(cfg.headless)),
	];
	pairs.into_iter().map(|(key, value)| {
		let requires_restart = if Config::requires_restart(key) { Some(true) } else { None };
		SettingEntry { key: key.into(), value, requires_restart }
	}).collect()
}

/// Error response helper.
fn error_response(code: StatusCode, msg: impl Into<String>) -> Response {
	(StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": msg.into()}))).into_response()
}

/// GET /api/settings — return all config values.
pub async fn get_settings() -> impl IntoResponse {
	let app_handle = api::app_handle();
	let cfg = Config::load(app_handle);
	(StatusCode::OK, Json(SettingsResponse {
		settings: entries_from_config(&cfg),
	}))
}

/// PATCH /api/settings — update one or more config values.
///
/// Only provided fields are updated. Each entry includes `requires_restart`.
/// Invalid values return a 422 error with a descriptive message.
pub async fn patch_settings(
	Json(body): Json<PatchSettingsBody>,
) -> Response {
	let app_handle = api::app_handle();
	let mut updated = Vec::new();
	let mut warnings = Vec::new();

	if let Some(port) = body.port {
		if (3000..=3099).contains(&port) {
			if let Err(e) = Config::set(app_handle, "port", serde_json::json!(port)) {
				tracing::error!("Failed to persist port: {}", e);
				return error_response(StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to save port: {}", e));
			}
			updated.push(SettingEntry {
				key: "port".into(),
				value: serde_json::json!(port),
				requires_restart: Some(true),
			});
		} else {
			warnings.push(ValidationWarning {
				field: "port".into(),
				message: "Port must be between 3000 and 3099".into(),
			});
		}
	}

	if let Some(hostname) = body.hostname {
		if is_valid_hostname(&hostname) {
			if let Err(e) = Config::set(app_handle, "hostname", serde_json::json!(hostname)) {
				tracing::error!("Failed to persist hostname: {}", e);
				return error_response(StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to save hostname: {}", e));
			}
			updated.push(SettingEntry {
				key: "hostname".into(),
				value: serde_json::json!(hostname),
				requires_restart: Some(true),
			});
		} else {
			warnings.push(ValidationWarning {
				field: "hostname".into(),
				message: "Hostname must be a valid DNS name (alphanumeric, hyphens, dots)".into(),
			});
		}
	}

	if let Some(interval) = body.backup_interval_hours {
		if interval >= 1 && interval <= 168 {
			if let Err(e) = Config::set(app_handle, "backup_interval_hours", serde_json::json!(interval)) {
				tracing::error!("Failed to persist backup_interval_hours: {}", e);
				return error_response(StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to save interval: {}", e));
			}
			updated.push(SettingEntry {
				key: "backup_interval_hours".into(),
				value: serde_json::json!(interval),
				requires_restart: Some(false),
			});
		} else {
			warnings.push(ValidationWarning {
				field: "backup_interval_hours".into(),
				message: "Interval must be between 1 and 168 hours".into(),
			});
		}
	}

	if let Some(headless) = body.headless {
		if let Err(e) = Config::set(app_handle, "headless", serde_json::json!(headless)) {
			tracing::error!("Failed to persist headless: {}", e);
			return error_response(StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to save headless: {}", e));
		}
		updated.push(SettingEntry {
			key: "headless".into(),
			value: serde_json::json!(headless),
			requires_restart: Some(true),
		});
	}

	if !warnings.is_empty() {
		return (
			StatusCode::UNPROCESSABLE_ENTITY,
			Json(PatchSettingsResponse { settings: updated, warnings }),
		).into_response();
	}

	(StatusCode::OK, Json(PatchSettingsResponse { settings: updated, warnings })).into_response()
}

/// DELETE /api/settings — reset all values to defaults.
pub async fn reset_settings() -> impl IntoResponse {
	let app_handle = api::app_handle();
	match Config::reset(app_handle) {
		Ok(()) => (StatusCode::OK, Json(serde_json::json!({"status": "reset"}))),
		Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e}))),
	}
}
