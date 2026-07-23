//! Wallet API — register client, get balance, view ledger.
//!
//! AD-2: wallet_ledger append-only. Balance = SELECT SUM(amount).
//! AD-4: Wallet is an island — no outgoing dependencies.
//! Story 1.5.2.

use axum::{
	extract::{Path, Query, State},
	http::StatusCode,
	response::IntoResponse,
	Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::domain::wallet::{LedgerEntryType, WalletClient, WalletLedgerEntry, WalletRepository};
use crate::domain::DomainError;

use super::AppApiState;

// ─── Response types ────────────────────────────────────────────────

#[derive(Serialize)]
pub struct WalletClientResponse {
	pub id: String,
	pub phone: String,
	pub name: String,
	pub balance: i64,
	pub created_at: String,
}

#[derive(Serialize)]
pub struct LedgerEntryResponse {
	pub id: String,
	pub client_id: String,
	pub entry_type: String,
	pub amount: i64,
	pub reference: Option<String>,
	pub description: Option<String>,
	pub created_at: String,
}

#[derive(Serialize)]
pub struct LedgerResponse {
	pub client_id: String,
	pub entries: Vec<LedgerEntryResponse>,
	pub balance: i64,
}

#[derive(Serialize)]
pub struct RegisterResponse {
	pub id: String,
	pub phone: String,
	pub name: String,
	pub balance: i64,
}

#[derive(Serialize)]
pub(crate) struct ApiError {
	error: String,
	code: String,
}

// ─── Request types ─────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct RegisterRequest {
	pub phone: String,
	#[serde(default)]
	pub name: Option<String>,
}

#[derive(Deserialize)]
pub struct LedgerQuery {
	#[serde(default = "default_limit")]
	pub limit: i64,
}

fn default_limit() -> i64 {
	50
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

/// Validate phone number: exactly 9 digits.
fn validate_phone(phone: &str) -> Result<String, (StatusCode, Json<ApiError>)> {
	let phone = phone.trim();
	if phone.len() != 9 || !phone.chars().all(|c| c.is_ascii_digit()) {
		return Err(error_response(
			"Phone must be exactly 9 digits",
			"VALIDATION_ERROR",
			StatusCode::UNPROCESSABLE_ENTITY,
		));
	}
	Ok(phone.to_string())
}

fn client_to_response(client: &WalletClient, balance: i64) -> WalletClientResponse {
	WalletClientResponse {
		id: client.id.clone(),
		phone: client.phone.clone(),
		name: client.name.clone().unwrap_or_default(),
		balance,
		created_at: client.created_at.clone(),
	}
}

fn entry_to_response(entry: &WalletLedgerEntry) -> LedgerEntryResponse {
	LedgerEntryResponse {
		id: entry.id.clone(),
		client_id: entry.client_id.clone(),
		entry_type: entry.entry_type.as_str().to_string(),
		amount: entry.amount,
		reference: entry.reference.clone(),
		description: entry.description.clone(),
		created_at: entry.created_at.clone(),
	}
}

// ─── Handlers ───────────────────────────────────────────────────────

/// POST /api/wallet/register — register a new wallet client by phone.
pub async fn register(
	State(state): State<AppApiState>,
	Json(body): Json<RegisterRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiError>)> {
	let phone = validate_phone(&body.phone)?;

	let now = chrono_now();
	let client = WalletClient {
		id: uuid_v7(),
		phone,
		name: body.name,
		referrer_id: None,
		created_at: now.clone(),
		updated_at: now,
	};

	state.wallet_repo.register_client(&client)
		.map_err(|e| match e {
			DomainError::DuplicatePhone => error_response("Phone already registered", "DUPLICATE_PHONE", StatusCode::CONFLICT),
			_ => error_response(&e.to_string(), "REGISTRATION_ERROR", StatusCode::INTERNAL_SERVER_ERROR),
		})?;

	// Balance is 0 for a new client
	let balance = 0;

	tracing::info!(target: "wallet", "Client registered: id={}, phone={}", client.id, client.phone);

	Ok((StatusCode::CREATED, Json(RegisterResponse {
		id: client.id,
		phone: client.phone,
		name: client.name.unwrap_or_default(),
		balance,
	})))
}

/// GET /api/wallet/by-phone/{phone} — get wallet client info and balance by phone.
pub async fn get_by_phone(
	State(state): State<AppApiState>,
	Path(phone): Path<String>,
) -> Result<Json<WalletClientResponse>, (StatusCode, Json<ApiError>)> {
	let client = state.wallet_repo.find_client_by_phone(&phone)
		.map_err(|e| error_response(&e.to_string(), "INTERNAL_ERROR", StatusCode::INTERNAL_SERVER_ERROR))?
		.ok_or_else(|| error_response("Client not found", "NOT_FOUND", StatusCode::NOT_FOUND))?;

	let balance = state.wallet_repo.get_balance(&client.id)
		.map_err(|e| error_response(&e.to_string(), "INTERNAL_ERROR", StatusCode::INTERNAL_SERVER_ERROR))?;

	Ok(Json(client_to_response(&client, balance)))
}

/// GET /api/wallet/{id}/ledger?limit=50 — get ledger entries for a client.
pub async fn get_ledger(
	State(state): State<AppApiState>,
	Path(client_id): Path<String>,
	Query(query): Query<LedgerQuery>,
) -> Result<Json<LedgerResponse>, (StatusCode, Json<ApiError>)> {
	// Verify client exists
	let _client = state.wallet_repo.find_client_by_id(&client_id)
		.map_err(|e| error_response(&e.to_string(), "INTERNAL_ERROR", StatusCode::INTERNAL_SERVER_ERROR))?
		.ok_or_else(|| error_response("Client not found", "NOT_FOUND", StatusCode::NOT_FOUND))?;

	let limit = if query.limit < 0 { 0 } else { query.limit };
	let entries = state.wallet_repo.get_ledger(&client_id, limit)
		.map_err(|e| error_response(&e.to_string(), "INTERNAL_ERROR", StatusCode::INTERNAL_SERVER_ERROR))?;

	let balance = state.wallet_repo.get_balance(&client_id)
		.map_err(|e| error_response(&e.to_string(), "INTERNAL_ERROR", StatusCode::INTERNAL_SERVER_ERROR))?;

	Ok(Json(LedgerResponse {
		client_id,
		entries: entries.iter().map(entry_to_response).collect(),
		balance,
	}))
}

// ─── Credit wallet types ───────────────────────────────────────────

#[derive(Deserialize)]
pub struct CreditWalletRequest {
	pub amount: i64,
	pub source: String,
	#[serde(default)]
	pub reference: Option<String>,
}

#[derive(Serialize)]
pub struct CreditWalletResponse {
	pub status: String,
	pub new_balance: i64,
}

/// POST /api/wallet/{client_id}/credit — manually credit a wallet (AC-3, AC-4).
///
/// Sources: cash, momo, gift. Amount must be > 0.
/// This is an independent operation — no order_id, no payment association.
pub async fn credit_wallet(
	State(state): State<AppApiState>,
	Path(client_id): Path<String>,
	Json(body): Json<CreditWalletRequest>,
) -> Result<Json<CreditWalletResponse>, (StatusCode, Json<ApiError>)> {
	// Validate amount > 0 (AC-3)
	if body.amount <= 0 {
		return Err(error_response(
			"Amount must be positive",
			"VALIDATION_ERROR",
			StatusCode::UNPROCESSABLE_ENTITY,
		));
	}

	// Validate source (AC-3)
	let source = body.source.to_lowercase();
	if !matches!(source.as_str(), "cash" | "momo" | "gift") {
		return Err(error_response(
			&format!("Invalid source: '{}' (must be cash, momo, or gift)", body.source),
			"VALIDATION_ERROR",
			StatusCode::UNPROCESSABLE_ENTITY,
		));
	}

	// Verify client exists
	let _client = state.wallet_repo.find_client_by_id(&client_id)
		.map_err(|e| error_response(&e.to_string(), "INTERNAL_ERROR", StatusCode::INTERNAL_SERVER_ERROR))?
		.ok_or_else(|| error_response("Client not found", "NOT_FOUND", StatusCode::NOT_FOUND))?;

	// Create and append ledger entry
	let now = chrono_now();
	let entry = WalletLedgerEntry {
		id: uuid_v7(),
		client_id: client_id.clone(),
		entry_type: LedgerEntryType::Credit,
		amount: body.amount,
		reference: body.reference,
		description: Some(format!("Manual credit via {}", source)),
		created_at: now,
	};

	state.wallet_repo.append_entry(&entry)
		.map_err(|e| error_response(&e.to_string(), "CREDIT_ERROR", StatusCode::INTERNAL_SERVER_ERROR))?;

	// Read new balance
	let new_balance = state.wallet_repo.get_balance(&client_id)
		.map_err(|e| error_response(&e.to_string(), "INTERNAL_ERROR", StatusCode::INTERNAL_SERVER_ERROR))?;

	tracing::info!(
		target: "wallet",
		"Manual credit: client={}, amount={}, source={}, new_balance={}",
		client_id, body.amount, source, new_balance
	);

	Ok(Json(CreditWalletResponse {
		status: "credited".into(),
		new_balance,
	}))
}
