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
use crate::domain::payment::{self as domain_payment, Payment, PaymentMethod, PaymentRepository, SplitPaymentItem};
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
	pub wallet_negative: bool,
}

impl FromRef<AppApiState> for PaymentsState {
	fn from_ref(state: &AppApiState) -> Self {
		Self {
			order_repo: state.order_repo.clone(),
			wallet_repo: state.wallet_repo.clone(),
			payment_repo: state.payment_repo.clone(),
			db_pool: state.db_pool.clone(),
			wallet_negative: crate::settings::Config::load(
				&crate::api::app_handle(),
			).wallet_negative,
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
	/// MoMo operator (orange|mtn). Required when method=momo.
	#[serde(default)]
	pub momo_operator: Option<String>,
	/// Split sub-payments. Required when method=split.
	#[serde(default)]
	pub payments: Vec<SplitPaymentItem>,
}

#[derive(Serialize)]
pub struct PaymentResponse {
	pub status: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub new_balance: Option<i64>,
	pub payment_id: String,
}

#[derive(Serialize)]
pub struct SplitPaymentSubResponse {
	pub method: String,
	pub amount: i64,
	pub payment_id: String,
}

#[derive(Serialize)]
pub struct SplitPaymentResponse {
	pub status: String,
	pub payments: Vec<SplitPaymentSubResponse>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub new_balance: Option<i64>,
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
				code: "VALIDATION_ERROR".into(),
			}),
		),
		DomainError::SplitTotalMismatch { sum, expected } => (
			StatusCode::UNPROCESSABLE_ENTITY,
			Json(ApiError {
				error: format!("Split total mismatch: sum={}, expected={}", sum, expected),
				code: "SPLIT_TOTAL_MISMATCH".into(),
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

// ─── Shared helpers ─────────────────────────────────────────────────

/// Debit a wallet within an already-open transaction (BEGIN IMMEDIATE must be active).
///
/// Used by both the wallet-only handler (3.3) and the split handler (3.4).
/// Does NOT manage BEGIN/COMMIT/ROLLBACK — the caller owns the transaction.
///
/// Returns `(new_balance, ledger_entry_id)` on success.
fn debit_wallet_in_tx(
	conn: &rusqlite::Connection,
	client_id: &str,
	amount: i64,
	order_id: &str,
	wallet_negative: bool,
	now: &str,
) -> Result<(i64, String), (StatusCode, Json<ApiError>)> {
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

	// Step 2: Check sufficient balance
	if balance < amount && !wallet_negative {
		return Err((
			StatusCode::UNPROCESSABLE_ENTITY,
			Json(ApiError {
				error: format!("Insufficient balance: {} FCFA (need {})", balance, amount),
				code: "INSUFFICIENT_BALANCE".into(),
			}),
		));
	}

	// Step 3: INSERT into wallet_ledger (debit)
	let ledger_id = uuid_v7();
	let entry = WalletLedgerEntry {
		id: ledger_id.clone(),
		client_id: client_id.to_string(),
		entry_type: LedgerEntryType::Payment,
		amount: -amount,
		reference: Some(order_id.to_string()),
		description: Some(format!("Payment for order {}", order_id)),
		created_at: now.to_string(),
	};

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

	Ok((balance - amount, ledger_id))
}

/// Insert a cash payment record within an open transaction.
fn insert_payment_in_tx(
	conn: &rusqlite::Connection,
	order_id: &str,
	amount: i64,
	parent_payment_id: Option<&str>,
	now: &str,
) -> Result<String, (StatusCode, Json<ApiError>)> {
	let payment_id = uuid_v7();
	conn.execute(
		"INSERT INTO payments (id, order_id, method, amount, client_id, reference, momo_operator, parent_payment_id, created_at) \
		 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
		rusqlite::params![
			payment_id,
			order_id,
			"cash",
			amount,
			Option::<String>::None,
			Option::<String>::None,
			Option::<String>::None,
			parent_payment_id,
			now,
		],
	).map_err(|e| {
		error_response(
			&format!("Failed to create cash payment: {}", e),
			"DB_ERROR",
			StatusCode::INTERNAL_SERVER_ERROR,
		)
	})?;

	Ok(payment_id)
}

/// Insert a MoMo payment record within an open transaction.
fn insert_momo_payment_in_tx(
	conn: &rusqlite::Connection,
	order_id: &str,
	amount: i64,
	momo_operator: &str,
	parent_payment_id: Option<&str>,
	now: &str,
) -> Result<String, (StatusCode, Json<ApiError>)> {
	let payment_id = uuid_v7();
	conn.execute(
		"INSERT INTO payments (id, order_id, method, amount, client_id, reference, momo_operator, parent_payment_id, created_at) \
		 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
		rusqlite::params![
			payment_id,
			order_id,
			"momo",
			amount,
			Option::<String>::None,
			Option::<String>::None,
			momo_operator,
			parent_payment_id,
			now,
		],
	).map_err(|e| {
		error_response(
			&format!("Failed to create MoMo payment: {}", e),
			"DB_ERROR",
			StatusCode::INTERNAL_SERVER_ERROR,
		)
	})?;

	Ok(payment_id)
}

// ─── Handler ────────────────────────────────────────────────────────

/// POST /api/payments — Process a payment for an order.
///
/// Dispatches on method:
/// - wallet  → single wallet debit (AC-1, AC-4, AC-5)
/// - cash    → single cash payment (AC-3)
/// - momo    → single MoMo label payment (AC-2)
/// - split   → multi-method split payment (AC-1, AC-5)
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
				Ok(Some(_)) => {}
				Ok(None) => {
					return error_response(
						"Client not found",
						"CLIENT_NOT_FOUND",
						StatusCode::UNPROCESSABLE_ENTITY,
					).into_response();
				}
				Err(e) => return domain_to_http(e).into_response(),
			}


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

			if let Err(e) = conn.execute("BEGIN IMMEDIATE", []) {
				return error_response(
					&format!("Failed to begin transaction: {}", e),
					"DB_ERROR",
					StatusCode::INTERNAL_SERVER_ERROR,
				).into_response();
			}

			let result = (|| -> Result<(i64, String), (StatusCode, Json<ApiError>)> {
				// Use the shared debit function
				let (new_balance, ledger_id) = debit_wallet_in_tx(
					&conn, &client_id, order.total, &req.order_id,
					state.wallet_negative, &now,
				)?;

				// INSERT into payments
				let payment_id = uuid_v7();
				let payment = Payment {
					id: payment_id.clone(),
					order_id: req.order_id.clone(),
					method: PaymentMethod::Wallet,
					amount: order.total,
					client_id: Some(client_id.clone()),
					reference: Some(ledger_id),
					momo_operator: None,
					parent_payment_id: None,
					created_at: now.clone(),
				};

				payment.validate().map_err(|e| {
					error_response(&e.to_string(), "VALIDATION_ERROR", StatusCode::UNPROCESSABLE_ENTITY)
				})?;

				conn.execute(
					"INSERT INTO payments (id, order_id, method, amount, client_id, reference, momo_operator, parent_payment_id, created_at) \
					 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
					rusqlite::params![
						payment.id,
						payment.order_id,
						payment.method.as_str(),
						payment.amount,
						payment.client_id,
						payment.reference,
						payment.momo_operator,
						payment.parent_payment_id,
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

				Ok((new_balance, payment_id))
			})();

			match result {
				Ok((new_balance, payment_id)) => {
					if let Err(e) = conn.execute("COMMIT", []) {
						let _ = conn.execute("ROLLBACK", []);
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
					let _ = conn.execute("ROLLBACK", []);
					e.into_response()
				}
			}
		}
		PaymentMethod::Cash => {
			// ─── Atomic cash payment (AC-3, AC-5) ──────────────────

			let payment_id = uuid_v7();
			let payment = Payment {
				id: payment_id.clone(),
				order_id: req.order_id.clone(),
				method: PaymentMethod::Cash,
				amount: order.total,
				client_id: None,
				reference: None,
				momo_operator: None,
				parent_payment_id: None,
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
				conn.execute(
					"INSERT INTO payments (id, order_id, method, amount, client_id, reference, momo_operator, parent_payment_id, created_at) \
					 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
					rusqlite::params![
						payment.id,
						payment.order_id,
						payment.method.as_str(),
						payment.amount,
						payment.client_id,
						payment.reference,
						payment.momo_operator,
						payment.parent_payment_id,
						payment.created_at,
					],
				).map_err(|e| {
					error_response(
						&format!("Failed to create payment: {}", e),
						"DB_ERROR",
						StatusCode::INTERNAL_SERVER_ERROR,
					)
				})?;

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
						let _ = conn.execute("ROLLBACK", []);
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
		PaymentMethod::MoMo => {
			// ─── MoMo payment — label only, no API call (AC-2) ─────

			// Validate momo_operator (AC-2)
			let operator = match &req.momo_operator {
				Some(op) if op == "orange" || op == "mtn" => op.clone(),
				Some(op) => {
					return error_response(
						&format!("Invalid momo_operator: '{}' (must be 'orange' or 'mtn')", op),
						"VALIDATION_ERROR",
						StatusCode::UNPROCESSABLE_ENTITY,
					).into_response();
				}
				None => {
					return error_response(
						"momo_operator is required for MoMo payments",
						"VALIDATION_ERROR",
						StatusCode::UNPROCESSABLE_ENTITY,
					).into_response();
				}
			};

			// ─── Atomic transaction ────────────────────────────────
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
				let payment_id = insert_momo_payment_in_tx(
					&conn, &req.order_id, order.total, &operator, None, &now,
				)?;

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

				Ok(payment_id)
			})();

			match result {
				Ok(payment_id) => {
					if let Err(e) = conn.execute("COMMIT", []) {
						let _ = conn.execute("ROLLBACK", []);
						return error_response(
							&format!("Failed to commit transaction: {}", e),
							"DB_ERROR",
							StatusCode::INTERNAL_SERVER_ERROR,
						).into_response();
					}

					tracing::info!(
						target: "payments",
						"MoMo payment processed: order={}, operator={}, amount={}",
						req.order_id, operator, order.total
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
		PaymentMethod::Split => {
			// ─── Split multi-moyen payment (AC-1, AC-5) ───────────

			// Validate split structure (AC-1: total match, amounts > 0, required fields)
			if let Err(e) = domain_payment::validate_split(&req.payments, order.total) {
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

			let parent_payment_id = uuid_v7();

			let result = (|| -> Result<SplitPaymentResponse, (StatusCode, Json<ApiError>)> {
				let mut sub_payments: Vec<SplitPaymentSubResponse> = Vec::new();
				let mut wallet_balance: Option<i64> = None;

				for item in &req.payments {
					match item.method.to_lowercase().as_str() {
						"wallet" => {
							let cid = item.client_id.as_deref().unwrap_or("");
							// Verify client exists (AC-6)
							let client_exists: bool = conn
								.query_row(
									"SELECT COUNT(*) FROM wallet_clients WHERE id = ?1",
									rusqlite::params![cid],
									|row| row.get::<_, i64>(0),
								)
								.map_err(|e| {
									error_response(
										&format!("Database error: {}", e),
										"DB_ERROR",
										StatusCode::INTERNAL_SERVER_ERROR,
									)
								})?
								> 0;

							if !client_exists {
								return Err(error_response(
									&format!("Client not found: {}", cid),
									"CLIENT_NOT_FOUND",
									StatusCode::UNPROCESSABLE_ENTITY,
								));
							}

							let (new_bal, ledger_id) = debit_wallet_in_tx(
								&conn, cid, item.amount, &req.order_id,
								state.wallet_negative, &now,
							)?;
							wallet_balance = Some(new_bal);

							// Create a payment record for this wallet sub-payment
							let sp_id = uuid_v7();
							conn.execute(
								"INSERT INTO payments (id, order_id, method, amount, client_id, reference, momo_operator, parent_payment_id, created_at) \
								 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
								rusqlite::params![
									sp_id,
									req.order_id,
									"wallet",
									item.amount,
									cid,
									ledger_id,
									Option::<String>::None,
									&parent_payment_id,
									now,
								],
							).map_err(|e| {
								error_response(
									&format!("Failed to create wallet payment: {}", e),
									"DB_ERROR",
									StatusCode::INTERNAL_SERVER_ERROR,
								)
							})?;

							sub_payments.push(SplitPaymentSubResponse {
								method: "wallet".into(),
								amount: item.amount,
								payment_id: sp_id,
							});
						}
						"cash" => {
							let sp_id = insert_payment_in_tx(
								&conn, &req.order_id, item.amount, Some(&parent_payment_id), &now,
							)?;

							sub_payments.push(SplitPaymentSubResponse {
								method: "cash".into(),
								amount: item.amount,
								payment_id: sp_id,
							});
						}
						"momo" => {
							let op = item.momo_operator.as_deref().unwrap_or("");
							let sp_id = insert_momo_payment_in_tx(
								&conn, &req.order_id, item.amount, op, Some(&parent_payment_id), &now,
							)?;

							sub_payments.push(SplitPaymentSubResponse {
								method: "momo".into(),
								amount: item.amount,
								payment_id: sp_id,
							});
						}
						_ => {
							// Already validated by validate_split, but guard anyway
							return Err(error_response(
								&format!("Unknown payment method: {}", item.method),
								"VALIDATION_ERROR",
								StatusCode::UNPROCESSABLE_ENTITY,
							));
						}
					}
				}

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

				Ok(SplitPaymentResponse {
					status: "paid".into(),
					payments: sub_payments,
					new_balance: wallet_balance,
				})
			})();

			match result {
				Ok(resp) => {
					if let Err(e) = conn.execute("COMMIT", []) {
						let _ = conn.execute("ROLLBACK", []);
						return error_response(
							&format!("Failed to commit transaction: {}", e),
							"DB_ERROR",
							StatusCode::INTERNAL_SERVER_ERROR,
						).into_response();
					}

					tracing::info!(
						target: "payments",
						"Split payment processed: order={}, num_payments={}",
						req.order_id, req.payments.len()
					);

					(StatusCode::OK, Json(resp)).into_response()
				}
				Err(e) => {
					let _ = conn.execute("ROLLBACK", []);
					e.into_response()
				}
			}
		}
	}
}
