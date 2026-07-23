//! Kitchen display API — GET /api/kitchen/orders.
//!
//! AD-14: Polling HTTP 5s. No WebSocket in V1.
//! AD-1: Thin API layer — delegates to domain via OrderRepository + ProductRepository.
//! AD-8: Returns (StatusCode, Json<ApiError>) with standardized error format.
//! Story 3.5.

use std::sync::Arc;

use axum::{
	extract::{FromRef, State},
	http::StatusCode,
	response::IntoResponse,
	Json,
};
use serde::Serialize;

use crate::domain::order::{OrderRepository, OrderStatus};
use crate::domain::product::ProductRepository;
use crate::domain::DomainError;

use super::AppApiState;

// --- State extraction ---

#[derive(Clone)]
pub struct KitchenState {
	pub order_repo: Arc<dyn OrderRepository>,
	pub product_repo: Arc<dyn ProductRepository>,
}

impl FromRef<AppApiState> for KitchenState {
	fn from_ref(state: &AppApiState) -> Self {
		Self {
			order_repo: state.order_repo.clone(),
			product_repo: state.product_repo.clone(),
		}
	}
}

// --- Response types ---

#[derive(Serialize)]
pub struct KitchenItemResponse {
	pub product_id: String,
	pub name: String,
	pub quantity: i64,
	pub unit_price: i64,
	pub notes: Option<String>,
}

#[derive(Serialize)]
pub struct KitchenOrderResponse {
	pub id: String,
	pub table_id: Option<String>,
	pub client_id: Option<String>,
	pub status: String,
	pub total: i64,
	pub created_at: String,
	pub updated_at: String,
	pub items: Vec<KitchenItemResponse>,
	pub elapsed_min: i64,
}

#[derive(Serialize)]
pub struct KitchenResponse {
	pub in_preparation: Vec<KitchenOrderResponse>,
	pub ready: Vec<KitchenOrderResponse>,
}

#[derive(Serialize)]
pub struct ApiError {
	pub error: String,
	pub code: String,
}

// --- Error helpers ---

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
		_ => (
			StatusCode::INTERNAL_SERVER_ERROR,
			Json(ApiError {
				error: "Internal server error".into(),
				code: "INTERNAL_ERROR".into(),
			}),
		),
	}
}

// --- Helpers ---

fn elapsed_min(created_at: &str) -> i64 {
	use chrono::{DateTime, Utc};
	let created: DateTime<Utc> = created_at.parse().unwrap_or_else(|_| Utc::now());
	let now = Utc::now();
	(now - created).num_minutes()
}

fn get_product_name(
	repo: &Arc<dyn ProductRepository>,
	product_id: &str,
) -> String {
	match repo.find_product_by_id(product_id) {
		Ok(Some(p)) => p.name,
		_ => "(Produit supprime)".to_string(),
	}
}

fn build_order_response(
	order: crate::domain::order::Order,
	items: Vec<crate::domain::order::OrderItem>,
	product_repo: &Arc<dyn ProductRepository>,
) -> KitchenOrderResponse {
	let kitchen_items: Vec<KitchenItemResponse> = items
		.into_iter()
		.map(|item| KitchenItemResponse {
			product_id: item.product_id.clone(),
			name: get_product_name(product_repo, &item.product_id),
			quantity: item.quantity,
			unit_price: item.unit_price,
			notes: item.notes,
		})
		.collect();

	KitchenOrderResponse {
		id: order.id,
		table_id: order.table_id,
		client_id: order.client_id,
		status: order.status.as_str().to_string(),
		total: order.total,
		created_at: order.created_at.clone(),
		updated_at: order.updated_at,
		items: kitchen_items,
		elapsed_min: elapsed_min(&order.created_at),
	}
}

// --- Handlers ---

/// GET /api/kitchen/orders — retourne les commandes PaidPreparing et Ready.
///
/// AC-1: Liste les commandes actives de la cuisine, groupees par statut.
/// Triees par created_at ASC (les plus anciennes d'abord).
pub async fn list_kitchen_orders(
	State(state): State<KitchenState>,
) -> impl IntoResponse {
	let mut preparing = match state.order_repo.list_by_status(&OrderStatus::PaidPreparing) {
		Ok(orders) => orders,
		Err(e) => return domain_to_http(e).into_response(),
	};

	let mut ready = match state.order_repo.list_by_status(&OrderStatus::Ready) {
		Ok(orders) => orders,
		Err(e) => return domain_to_http(e).into_response(),
	};

	// Reverse from DB DESC to ASC (oldest first per AC-1)
	preparing.reverse();
	ready.reverse();

	let mut in_preparation: Vec<KitchenOrderResponse> = Vec::with_capacity(preparing.len());
	for order in preparing {
		let items = match state.order_repo.get_items(&order.id) {
			Ok(items) => items,
			Err(e) => return domain_to_http(e).into_response(),
		};
		in_preparation.push(build_order_response(order, items, &state.product_repo));
	}

	let mut ready_responses: Vec<KitchenOrderResponse> = Vec::with_capacity(ready.len());
	for order in ready {
		let items = match state.order_repo.get_items(&order.id) {
			Ok(items) => items,
			Err(e) => return domain_to_http(e).into_response(),
		};
		ready_responses.push(build_order_response(order, items, &state.product_repo));
	}

	(StatusCode::OK, Json(KitchenResponse {
		in_preparation,
		ready: ready_responses,
	}))
		.into_response()
}
