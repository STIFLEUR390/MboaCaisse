//! Orders API — CRUD orders, status transitions, item management.
//!
//! Story 3.2. Handles /api/orders/*.
//! AD-1: Thin API layer — delegates to domain via OrderRepository + ProductRepository.
//! AD-8: Returns (StatusCode, Json<ApiError>) with standardized error format.
//! AD-13: Order depends on Catalog (product lookups for validation).

use std::sync::Arc;

use axum::{
	extract::{FromRef, Path, Query, State},
	http::StatusCode,
	response::IntoResponse,
	Json,
};
use serde::{Deserialize, Serialize};

use crate::domain::order::{Order, OrderItem, OrderRepository, OrderStatus};
use crate::domain::product::ProductRepository;
use crate::domain::DomainError;

use super::AppApiState;

// ─── State extraction ───────────────────────────────────────────────

#[derive(Clone)]
pub struct OrdersState {
	pub order_repo: Arc<dyn OrderRepository>,
	pub product_repo: Arc<dyn ProductRepository>,
}

impl FromRef<AppApiState> for OrdersState {
	fn from_ref(state: &AppApiState) -> Self {
		Self {
			order_repo: state.order_repo.clone(),
			product_repo: state.product_repo.clone(),
		}
	}
}

// ─── Request / Response types ───────────────────────────────────────

#[derive(Deserialize)]
pub struct CreateOrderItem {
	pub product_id: String,
	pub quantity: i64,
	#[serde(default)]
	pub notes: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateOrderRequest {
	#[serde(default)]
	pub table_id: Option<String>,
	#[serde(default)]
	pub client_id: Option<String>,
	pub items: Vec<CreateOrderItem>,
}

#[derive(Deserialize)]
pub struct UpdateStatusRequest {
	pub status: String,
}

#[derive(Deserialize)]
pub struct AddItemRequest {
	pub product_id: String,
	pub quantity: i64,
	#[serde(default)]
	pub notes: Option<String>,
}

#[derive(Deserialize)]
pub struct OrderListQuery {
	pub status: Option<String>,
}

#[derive(Serialize)]
pub struct OrderItemResponse {
	pub id: String,
	pub order_id: String,
	pub product_id: String,
	pub quantity: i64,
	pub unit_price: i64,
	pub notes: Option<String>,
	pub created_at: String,
}

impl From<OrderItem> for OrderItemResponse {
	fn from(i: OrderItem) -> Self {
		Self {
			id: i.id,
			order_id: i.order_id,
			product_id: i.product_id,
			quantity: i.quantity,
			unit_price: i.unit_price,
			notes: i.notes,
			created_at: i.created_at,
		}
	}
}

#[derive(Serialize)]
pub struct OrderResponse {
	pub id: String,
	pub table_id: Option<String>,
	pub client_id: Option<String>,
	pub status: String,
	pub total: i64,
	pub created_at: String,
	pub updated_at: String,
	pub items: Vec<OrderItemResponse>,
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
		DomainError::Unauthorized => (
			StatusCode::UNAUTHORIZED,
			Json(ApiError {
				error: "Unauthorized".into(),
				code: "UNAUTHORIZED".into(),
			}),
		),
		DomainError::NotFound(msg) => (
			StatusCode::NOT_FOUND,
			Json(ApiError {
				error: msg,
				code: "NOT_FOUND".into(),
			}),
		),
		DomainError::InvalidValue(msg) => (
			StatusCode::BAD_REQUEST,
			Json(ApiError {
				error: msg,
				code: "INVALID_VALUE".into(),
			}),
		),
		DomainError::InvalidStatusTransition { from, to } => (
			StatusCode::UNPROCESSABLE_ENTITY,
			Json(ApiError {
				error: format!("Invalid status transition: {} → {}", from, to),
				code: "INVALID_STATUS_TRANSITION".into(),
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

// ─── Handlers ───────────────────────────────────────────────────────

/// POST /api/orders — Create an order with items.
///
/// AC-2: Validates product existence, calculates total server-side.
pub async fn create_order(
	State(state): State<OrdersState>,
	Json(req): Json<CreateOrderRequest>,
) -> impl IntoResponse {
	// Validate items not empty
	if req.items.is_empty() {
		return error_response("Items list must not be empty", "VALIDATION_ERROR", StatusCode::BAD_REQUEST)
			.into_response();
	}

	// Validate products exist and get prices
	let mut resolved_items: Vec<(String, i64, Option<String>)> = Vec::new();
	for item in &req.items {
		if item.quantity <= 0 {
			return error_response("Invalid quantity", "INVALID_QUANTITY", StatusCode::UNPROCESSABLE_ENTITY)
				.into_response();
		}
		match state.product_repo.find_product_by_id(&item.product_id) {
			Ok(Some(product)) => {
				resolved_items.push((item.product_id.clone(), product.price, item.notes.clone()));
			}
			Ok(None) => {
				return error_response(
					&format!("Product not found: {}", item.product_id),
					"PRODUCT_NOT_FOUND",
					StatusCode::UNPROCESSABLE_ENTITY,
				)
					.into_response();
			}
			Err(e) => return domain_to_http(e).into_response(),
		}
	}

	let now = chrono_now();
	let order_id = uuid_v7();

	// Calculate total server-side
	// Recalculate properly
	let total: i64 = resolved_items.iter()
		.enumerate()
		.map(|(i, (_, price, _))| price * req.items[i].quantity)
		.sum();

	let order = Order::new(order_id.clone(), req.table_id, req.client_id, now.clone());

	// Persist order
	if let Err(e) = state.order_repo.create(&order) {
		return domain_to_http(e).into_response();
	}

	// Persist items
	let mut order_items: Vec<OrderItem> = Vec::new();
	for (i, (product_id, unit_price, notes)) in resolved_items.iter().enumerate() {
		let item_id = uuid_v7();
		let item = OrderItem {
			id: item_id,
			order_id: order_id.clone(),
			product_id: product_id.clone(),
			quantity: req.items[i].quantity,
			unit_price: *unit_price,
			notes: notes.clone(),
			created_at: now.clone(),
		};
		if let Err(e) = state.order_repo.add_item(&item) {
			// Cleanup: delete the order entirely (items cascade via FK)
			let _ = state.order_repo.delete(&order_id);
			return domain_to_http(e).into_response();
		}
		order_items.push(item);
	}

	// Update total
	if let Err(e) = state.order_repo.update_total(&order_id) {
		let _ = state.order_repo.delete(&order_id);
		return domain_to_http(e).into_response();
	}

	let response = OrderResponse {
		id: order_id,
		table_id: order.table_id,
		client_id: order.client_id,
		status: order.status.as_str().to_string(),
		total,
		created_at: order.created_at,
		updated_at: order.updated_at,
		items: order_items.into_iter().map(Into::into).collect(),
	};

	(StatusCode::CREATED, Json(response)).into_response()
}

/// GET /api/orders — List orders, optionally filtered by status.
///
/// AC-3: Returns orders sorted by created_at DESC, each with items.
pub async fn list_orders(
	State(state): State<OrdersState>,
	Query(query): Query<OrderListQuery>,
) -> impl IntoResponse {
	let orders: Vec<Order> = if let Some(ref status_str) = query.status {
		let status = match OrderStatus::from_str(status_str) {
			Ok(s) => s,
			Err(_) => {
				return error_response(
					&format!("Invalid status: {}", status_str),
					"INVALID_VALUE",
					StatusCode::BAD_REQUEST,
				)
					.into_response();
			}
		};
		match state.order_repo.list_by_status(&status) {
			Ok(orders) => orders,
			Err(e) => return domain_to_http(e).into_response(),
		}
	} else {
		match state.order_repo.list_all() {
			Ok(orders) => orders,
			Err(e) => return domain_to_http(e).into_response(),
		}
	};

	// Enrich each order with items
	let mut responses: Vec<OrderResponse> = Vec::with_capacity(orders.len());
	for order in orders {
		let items = match state.order_repo.get_items(&order.id) {
			Ok(items) => items.into_iter().map(Into::into).collect(),
			Err(e) => return domain_to_http(e).into_response(),
		};
		responses.push(OrderResponse {
			id: order.id,
			table_id: order.table_id,
			client_id: order.client_id,
			status: order.status.as_str().to_string(),
			total: order.total,
			created_at: order.created_at,
			updated_at: order.updated_at,
			items,
		});
	}

	(StatusCode::OK, Json(responses)).into_response()
}

/// GET /api/orders/{id} — Get order details with items.
///
/// AC-4: Returns 200 with order + items, or 404 if not found.
pub async fn get_order(
	State(state): State<OrdersState>,
	Path(id): Path<String>,
) -> impl IntoResponse {
	let order = match state.order_repo.find_by_id(&id) {
		Ok(Some(order)) => order,
		Ok(None) => {
			return error_response("Order not found", "ORDER_NOT_FOUND", StatusCode::NOT_FOUND)
				.into_response();
		}
		Err(e) => return domain_to_http(e).into_response(),
	};

	let items = match state.order_repo.get_items(&id) {
		Ok(items) => items.into_iter().map(Into::into).collect(),
		Err(e) => return domain_to_http(e).into_response(),
	};

	(StatusCode::OK, Json(OrderResponse {
		id: order.id,
		table_id: order.table_id,
		client_id: order.client_id,
		status: order.status.as_str().to_string(),
		total: order.total,
		created_at: order.created_at,
		updated_at: order.updated_at,
		items,
	})).into_response()
}

/// PATCH /api/orders/{id}/status — Transition order status.
///
/// AC-5: Validates transitions via Order::transition_to().
pub async fn update_order_status(
	State(state): State<OrdersState>,
	Path(id): Path<String>,
	Json(req): Json<UpdateStatusRequest>,
) -> impl IntoResponse {
	let mut order = match state.order_repo.find_by_id(&id) {
		Ok(Some(order)) => order,
		Ok(None) => {
			return error_response("Order not found", "ORDER_NOT_FOUND", StatusCode::NOT_FOUND)
				.into_response();
		}
		Err(e) => return domain_to_http(e).into_response(),
	};

	let new_status = match OrderStatus::from_str(&req.status) {
		Ok(s) => s,
		Err(e) => return domain_to_http(e).into_response(),
	};

	if let Err(e) = order.transition_to(new_status) {
		return domain_to_http(e).into_response();
	}

	if let Err(e) = state.order_repo.update_status(&id, &order.status) {
		return domain_to_http(e).into_response();
	}

	let items = match state.order_repo.get_items(&id) {
		Ok(items) => items.into_iter().map(Into::into).collect(),
		Err(e) => return domain_to_http(e).into_response(),
	};

	(StatusCode::OK, Json(OrderResponse {
		id: order.id,
		table_id: order.table_id,
		client_id: order.client_id,
		status: order.status.as_str().to_string(),
		total: order.total,
		created_at: order.created_at,
		updated_at: order.updated_at,
		items,
	})).into_response()
}

/// POST /api/orders/{id}/items — Add an item to an existing order.
///
/// AC-6: Only allowed in PendingPayment status.
pub async fn add_order_item(
	State(state): State<OrdersState>,
	Path(id): Path<String>,
	Json(req): Json<AddItemRequest>,
) -> impl IntoResponse {
	if req.quantity <= 0 {
		return error_response("Invalid quantity", "INVALID_QUANTITY", StatusCode::UNPROCESSABLE_ENTITY)
			.into_response();
	}

	// Check order exists and is in PendingPayment
	let order = match state.order_repo.find_by_id(&id) {
		Ok(Some(order)) => order,
		Ok(None) => {
			return error_response("Order not found", "ORDER_NOT_FOUND", StatusCode::NOT_FOUND)
				.into_response();
		}
		Err(e) => return domain_to_http(e).into_response(),
	};

	if order.status != OrderStatus::PendingPayment {
		return error_response(
			&format!("Cannot modify order in status: {}", order.status.as_str()),
			"INVALID_ORDER_STATUS",
			StatusCode::UNPROCESSABLE_ENTITY,
		)
			.into_response();
	}

	// Verify product exists
	let unit_price = match state.product_repo.find_product_by_id(&req.product_id) {
		Ok(Some(product)) => product.price,
		Ok(None) => {
			return error_response(
				&format!("Product not found: {}", req.product_id),
				"PRODUCT_NOT_FOUND",
				StatusCode::UNPROCESSABLE_ENTITY,
			)
				.into_response();
		}
		Err(e) => return domain_to_http(e).into_response(),
	};

	let now = chrono_now();
	let item = OrderItem {
		id: uuid_v7(),
		order_id: id.clone(),
		product_id: req.product_id,
		quantity: req.quantity,
		unit_price,
		notes: req.notes,
		created_at: now,
	};

	if let Err(e) = state.order_repo.add_item(&item) {
		return domain_to_http(e).into_response();
	}

	// Recalculate total
	if let Err(e) = state.order_repo.update_total(&id) {
		return domain_to_http(e).into_response();
	}

	(StatusCode::OK, Json(OrderItemResponse::from(item))).into_response()
}

/// DELETE /api/orders/{id}/items/{item_id} — Remove an item from an order.
///
/// AC-7: Only allowed in PendingPayment status. Recalculates total after removal.
pub async fn remove_order_item(
	State(state): State<OrdersState>,
	Path((id, item_id)): Path<(String, String)>,
) -> impl IntoResponse {
	// Check order exists and is in PendingPayment
	let order = match state.order_repo.find_by_id(&id) {
		Ok(Some(order)) => order,
		Ok(None) => {
			return error_response("Order not found", "ORDER_NOT_FOUND", StatusCode::NOT_FOUND)
				.into_response();
		}
		Err(e) => return domain_to_http(e).into_response(),
	};

	if order.status != OrderStatus::PendingPayment {
		return error_response(
			&format!("Cannot modify order in status: {}", order.status.as_str()),
			"INVALID_ORDER_STATUS",
			StatusCode::UNPROCESSABLE_ENTITY,
		)
			.into_response();
	}

	if let Err(e) = state.order_repo.remove_item(&id, &item_id) {
		return match e {
			DomainError::NotFound(_) => error_response("Order item not found", "ITEM_NOT_FOUND", StatusCode::NOT_FOUND),
			_ => domain_to_http(e),
		}
		.into_response();
	}

	// Recalculate total
	if let Err(e) = state.order_repo.update_total(&id) {
		return domain_to_http(e).into_response();
	}

	StatusCode::NO_CONTENT.into_response()
}
