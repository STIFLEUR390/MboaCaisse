//! Payments API — payment gate, atomic wallet debit, cash payments.
//!
//! AD-1:  Thin API layer — validates and delegates to repositories.
//!         The atomic transaction (BEGIN IMMEDIATE → writes → COMMIT) is
//!         managed here using the raw pool from AppApiState.
//! AD-2:  Wallet ledger is append-only. Balance = SELECT SUM(amount).
//! AD-4:  Payment calls Wallet. Wallet never calls Payment.
//! AD-8:  Errors via DomainError → (StatusCode, Json<ApiError>).
//! AD-13: Payment → Order + Wallet (graphe dépendances).
//!
//! Story 3.3 (Payment Gate) + 3.4 (Encaissement multi-moyen).

use std::sync::Arc;

use axum::{
	extract::{FromRef, State},
	http::StatusCode,
	response::IntoResponse,
	Json,
};
use serde::{Deserialize, Serialize};

use crate::domain::order::{OrderRepository, OrderStatus};
use crate::domain::payment::{Payment, PaymentMethod, PaymentRepository};
use crate::domain::wallet::{LedgerEntryType, WalletLedgerEntry, WalletRepository};
use crate::domain::DomainError;
use crate::db::SqlitePool;

use super::AppApiState;

// ─── State extraction ───────────────────────────────────────────────

#[derive(Clone)]
pub struct PaymentsState {
	pub order_repo: Arc<dyn OrderRepository>,
	pub wallet_repo: Arc<dyn WalletRepository>,
	pub payment_repo: Arc<dyn PaymentRepository>,
	pub db_pool: SqlitePool,
}

impl FromRef<AppApiState> for PaymentsState {
	fn from_ref(state: &AppApiState) -> Self {
		Self {
			order_repo: state.order_repo.clone(),
			wallet_repo: state.wallet_repo.clone(),
			payment_repo: state.payment_repo.clone(),
			db_pool: state.db_pool.clone(),
		}
	}
}

// ─── Request / Response types ───────────────────────────────────────

#[derive(Deserialize)]
pub struct ProcessPaymentRequest {
	pub order_id: String,
	pub method: String,
	#[serde(default)]
	pub client_id: Option<String>,
}

#[derive(Serialize)]
pub struct PaymentResponse {
	pub status: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub new_balance: Option<i64>,
	pub payment_id: String,
}

#[derive(Serialize)]
pub struct ApiError {
	pub error: String,
	pub code: String,
}

// ─── Error helpers ──────────────────────────────────────────────────

fn error_response(error: &str, code: &str, status: StatusCode) -> (StatusCode, Json<ApiError>) {
	(status, Json(ApiError {
		error: error.to_string(),
		code: code.to_string(),
	}))
}

fn domain_to_http(err: DomainError) -> (StatusCode, Json<ApiError>) {
	match err {
		DomainError::NotFound(msg) => (
			StatusCode::NOT_FOUND,
			Json(ApiError {
				error: msg,
				code: "NOT_FOUND".into(),
			}),
		),
		DomainError::InvalidValue(msg) => (
			StatusCode::UNPROCESSABLE_ENTITY,
			Json(ApiError {
				error: msg,
				code: "INVALID_VALUE".into(),
			}),
		),
		DomainError::InsufficientBalance { balance, required } => (
			StatusCode::UNPROCESSABLE_ENTITY,
			Json(ApiError {
				error: format!("Insufficient balance: {} FCFA (need {})", balance, required),
				code: "INSUFFICIENT_BALANCE".into(),
			}),
		),
		_ => (
			StatusCode::INTERNAL_SERVER_ERROR,
			Json(ApiError {
				error: "Internal server error".into(),
				code: "INTERNAL_ERROR".into(),
			}),
		),
	}
}

fn uuid_v7() -> String {
	use uuid::Uuid;
	Uuid::now_v7().to_string()
}

fn chrono_now() -> String {
	use chrono::Utc;
	Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
}

// ─── Handler ────────────────────────────────────────────────────────

/// POST /api/payments — Process a payment for an order.
///
/// # Wallet payment (AC-1, AC-2, AC-4, AC-5)
/// Uses a single BEGIN IMMEDIATE transaction to atomically:
/// 1. Check the wallet balance
/// 2. Debit the wallet_ledger (INSERT)
/// 3. Create the payment record (INSERT)
/// 4. Transition the order to PaidPreparing (UPDATE)
///
/// # Cash payment (AC-3)
/// Also uses BEGIN IMMEDIATE to prevent orphaned payment records.
pub async fn process_payment(
	State(state): State<PaymentsState>,
	Json(req): Json<ProcessPaymentRequest>,
) -> impl IntoResponse {
	// ─── Resolve payment method ────────────────────────────────────
	let method = match PaymentMethod::from_str(&req.method) {
		Ok(m) => m,
		Err(e) => return domain_to_http(e).into_response(),
	};

	// ─── Validate order exists and is in PendingPayment (AC-6) ──
	let order = match state.order_repo.find_by_id(&req.order_id) {
		Ok(Some(order)) => order,
		Ok(None) => {
			return error_response("Order not found", "ORDER_NOT_FOUND", StatusCode::NOT_FOUND)
				.into_response();
		}
		Err(e) => return domain_to_http(e).into_response(),
	};

	if order.status != OrderStatus::PendingPayment {
		return error_response(
			"Order not in PendingPayment status",
			"INVALID_ORDER_STATUS",
			StatusCode::UNPROCESSABLE_ENTITY,
		).into_response();
	}

	let now = chrono_now();

	match method {
		PaymentMethod::Wallet => {
			// ─── Validate client_id (AC-6) ────────────────────────
			let client_id = match &req.client_id {
				Some(cid) if !cid.is_empty() => cid.clone(),
				_ => {
					return error_response(
						"client_id is required for wallet payments",
						"VALIDATION_ERROR",
						StatusCode::UNPROCESSABLE_ENTITY,
					).into_response();
				}
			};

			// ─── Verify client exists (AC-6) ──────────────────────
			match state.wallet_repo.find_client_by_id(&client_id) {
				Ok(Some(_)) => {} // OK
				Ok(None) => {
					return error_response(
						"Client not found",
						"CLIENT_NOT_FOUND",
						StatusCode::UNPROCESSABLE_ENTITY,
					).into_response();
				}
				Err(e) => return domain_to_http(e).into_response(),
			}

			// ─── Load wallet_negative setting (AC-4) ──────────────
			let wallet_negative = crate::settings::Config::load(&crate::api::app_handle()).wallet_negative;

			// ─── Atomic transaction (AC-5) ────────────────────────
			let pool = &state.db_pool;
			let conn = match pool.get() {
				Ok(c) => c,
				Err(e) => {
					return error_response(
						&format!("Database error: {}", e),
						"DB_ERROR",
						StatusCode::INTERNAL_SERVER_ERROR,
					).into_response();
				}
			};

			// BEGIN IMMEDIATE — blocks concurrent writes
			if let Err(e) = conn.execute("BEGIN IMMEDIATE", []) {
				return error_response(
					&format!("Failed to begin transaction: {}", e),
					"DB_ERROR",
					StatusCode::INTERNAL_SERVER_ERROR,
				).into_response();
			}

			let result = (|| -> Result<(i64, String), (StatusCode, Json<ApiError>)> {
				// Step 1: Calculate current balance
				let balance: i64 = conn
					.query_row(
						"SELECT COALESCE(SUM(amount), 0) FROM wallet_ledger WHERE client_id = ?1",
						rusqlite::params![client_id],
						|row| row.get(0),
					)
					.map_err(|e| {
						error_response(
							&format!("Failed to check balance: {}", e),
							"DB_ERROR",
							StatusCode::INTERNAL_SERVER_ERROR,
						)
					})?;

				// Step 2: Check sufficient balance (AC-2, AC-4)
				let total = order.total;
				if balance < total && !wallet_negative {
					return Err((
						StatusCode::UNPROCESSABLE_ENTITY,
						Json(ApiError {
							error: format!("Insufficient balance: {} FCFA (need {})", balance, total),
							code: "INSUFFICIENT_BALANCE".into(),
						}),
					));
				}

				// Step 3: INSERT into wallet_ledger (debit)
				let ledger_id = uuid_v7();
				let entry = WalletLedgerEntry {
					id: ledger_id.clone(),
					client_id: client_id.clone(),
					entry_type: LedgerEntryType::Payment,
					amount: -total,
					reference: Some(req.order_id.clone()),
					description: Some(format!("Payment for order {}", req.order_id)),
					created_at: now.clone(),
				};

				// Direct INSERT to stay in the same transaction
				conn.execute(
					"INSERT INTO wallet_ledger (id, client_id, type, amount, reference, description, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
					rusqlite::params![
						entry.id,
						entry.client_id,
						entry.entry_type.as_str(),
						entry.amount,
						entry.reference,
						entry.description,
						entry.created_at,
					],
				).map_err(|e| {
					error_response(
						&format!("Failed to debit wallet: {}", e),
						"DB_ERROR",
						StatusCode::INTERNAL_SERVER_ERROR,
					)
				})?;

				// Step 4: INSERT into payments
				let payment_id = uuid_v7();
				let payment = Payment {
					id: payment_id.clone(),
					order_id: req.order_id.clone(),
					method: PaymentMethod::Wallet,
					amount: total,
					client_id: Some(client_id.clone()),
					reference: Some(ledger_id),
					created_at: now.clone(),
				};

				// Validate domain invariants (amount > 0)
				// Done inside the transaction to catch any unexpected state
				payment.validate().map_err(|e| {
					error_response(&e.to_string(), "VALIDATION_ERROR", StatusCode::UNPROCESSABLE_ENTITY)
				})?;

				conn.execute(
					"INSERT INTO payments (id, order_id, method, amount, client_id, reference, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
					rusqlite::params![
						payment.id,
						payment.order_id,
						payment.method.as_str(),
						payment.amount,
						payment.client_id,
						payment.reference,
						payment.created_at,
					],
				).map_err(|e| {
					error_response(
						&format!("Failed to create payment: {}", e),
						"DB_ERROR",
						StatusCode::INTERNAL_SERVER_ERROR,
					)
				})?;

				// Step 5: UPDATE order status to PaidPreparing
				conn.execute(
					"UPDATE orders SET status = ?1, updated_at = ?2 WHERE id = ?3",
					rusqlite::params!["paid_preparing", now, req.order_id],
				).map_err(|e| {
					error_response(
						&format!("Failed to update order status: {}", e),
						"DB_ERROR",
						StatusCode::INTERNAL_SERVER_ERROR,
					)
				})?;

				Ok((balance - total, payment_id))
			})();

			match result {
				Ok((new_balance, payment_id)) => {
					if let Err(e) = conn.execute("COMMIT", []) {
						return error_response(
							&format!("Failed to commit transaction: {}", e),
							"DB_ERROR",
							StatusCode::INTERNAL_SERVER_ERROR,
						).into_response();
					}

					tracing::info!(
						target: "payments",
						"Wallet payment processed: order={}, client={}, amount={}, new_balance={}",
						req.order_id, client_id, order.total, new_balance
					);

					(
						StatusCode::OK,
						Json(PaymentResponse {
							status: "paid".into(),
							new_balance: Some(new_balance),
							payment_id,
						}),
					).into_response()
				}
				Err(e) => {
					// Rollback on failure
					let _ = conn.execute("ROLLBACK", []);
					e.into_response()
				}
			}
		}
		PaymentMethod::Cash => {
			// ─── Atomic cash payment (AC-3, AC-5) ──────────────────
			// Uses the same pool-based transaction pattern as wallet
			// to prevent orphaned payment records.

			let payment_id = uuid_v7();
			let payment = Payment {
				id: payment_id.clone(),
				order_id: req.order_id.clone(),
				method: PaymentMethod::Cash,
				amount: order.total,
				client_id: None,
				reference: None,
				created_at: now.clone(),
			};

			// Validate BEFORE the transaction to avoid wasted BEGIN
			if let Err(e) = payment.validate() {
				return domain_to_http(e).into_response();
			}

			let pool = &state.db_pool;
			let conn = match pool.get() {
				Ok(c) => c,
				Err(e) => {
					return error_response(
						&format!("Database error: {}", e),
						"DB_ERROR",
						StatusCode::INTERNAL_SERVER_ERROR,
					).into_response();
				}
			};

			if let Err(e) = conn.execute("BEGIN IMMEDIATE", []) {
				return error_response(
					&format!("Failed to begin transaction: {}", e),
					"DB_ERROR",
					StatusCode::INTERNAL_SERVER_ERROR,
				).into_response();
			}

			let result = (|| -> Result<String, (StatusCode, Json<ApiError>)> {
				// INSERT payment record
				conn.execute(
					"INSERT INTO payments (id, order_id, method, amount, client_id, reference, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
					rusqlite::params![
						payment.id,
						payment.order_id,
						payment.method.as_str(),
						payment.amount,
						payment.client_id,
						payment.reference,
						payment.created_at,
					],
				).map_err(|e| {
					error_response(
						&format!("Failed to create payment: {}", e),
						"DB_ERROR",
						StatusCode::INTERNAL_SERVER_ERROR,
					)
				})?;

				// UPDATE order status to PaidPreparing
				conn.execute(
					"UPDATE orders SET status = ?1, updated_at = ?2 WHERE id = ?3",
					rusqlite::params!["paid_preparing", now, req.order_id],
				).map_err(|e| {
					error_response(
						&format!("Failed to update order status: {}", e),
						"DB_ERROR",
						StatusCode::INTERNAL_SERVER_ERROR,
					)
				})?;

				Ok(payment.id.clone())
			})();

			match result {
				Ok(payment_id) => {
					if let Err(e) = conn.execute("COMMIT", []) {
						return error_response(
							&format!("Failed to commit transaction: {}", e),
							"DB_ERROR",
							StatusCode::INTERNAL_SERVER_ERROR,
						).into_response();
					}

					tracing::info!(
						target: "payments",
						"Cash payment processed: order={}, amount={}",
						req.order_id, order.total
					);

					(
						StatusCode::OK,
						Json(PaymentResponse {
							status: "paid".into(),
							new_balance: None,
							payment_id,
						}),
					).into_response()
				}
				Err(e) => {
					let _ = conn.execute("ROLLBACK", []);
					e.into_response()
				}
			}
		}
		_ => {
			// MoMo and Split are handled in story 3.4
			error_response(
				&format!("Payment method '{}' not yet implemented", method.as_str()),
				"NOT_IMPLEMENTED",
				StatusCode::NOT_IMPLEMENTED,
			).into_response()
		}
	}
}
