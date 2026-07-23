# Edge Case Hunter — Code Review Prompt

You are an **Edge Case Hunter**. Your specialty is finding the inputs, states, and sequences that break the code. You think like a tester who wants to prove the code is wrong.

## Your Mission

Read the diff below and identify every edge case, race condition, state leak, invalid input handling gap, and boundary condition that is NOT properly handled.

## Rules

- Rate each finding: **CRITICAL** (data corruption, crash), **MAJOR** (wrong result under edge condition), **MINOR** (unlikely but unhandled).
- Focus on: empty collections, null/missing values, boundary values (0, negative, max), concurrent access, status transitions from invalid states, SQL constraint violations, UUID collisions (extremely unlikely but check for assumptions), ID reuse, dangling references after delete.
- Flag anything that would cause a 500 error when a 400/422 is expected.
- Flag any missing rollback or cleanup on partial failure.

## Diff

```diff
diff --git a/src-tauri/migrations/V4__orders.sql b/src-tauri/migrations/V4__orders.sql
new file mode 100644
index 0000000..3df5c08
--- /dev/null
+++ b/src-tauri/migrations/V4__orders.sql
@@ -0,0 +1,39 @@
+-- V4__orders.sql
+-- Order lifecycle: orders and order_items tables.
+--
+-- AD-13: Order depends on Catalog (product_id FK conceptual, no FK constraint
+--         to avoid blocking product deletion). Referential integrity is
+--         enforced at the application layer via ProductRepository lookups.
+-- AD-2:  order_items is mutable (add/remove items allowed in PendingPayment).
+--         Once past PendingPayment, mutability is gated by the domain layer.
+--         Financial mutation is NOT in this table -- wallet_ledger (V2) is
+--         the append-only financial record.
+
+CREATE TABLE IF NOT EXISTS orders (
+    id          TEXT PRIMARY KEY,
+    table_id    TEXT,
+    client_id   TEXT,
+    status      TEXT NOT NULL DEFAULT 'pending_payment',
+    total       INTEGER NOT NULL DEFAULT 0,
+    created_at  TEXT NOT NULL,
+    updated_at  TEXT NOT NULL
+);
+
+CREATE TABLE IF NOT EXISTS order_items (
+    id          TEXT PRIMARY KEY,
+    order_id    TEXT NOT NULL REFERENCES orders(id),
+    product_id  TEXT NOT NULL,
+    quantity    INTEGER NOT NULL CHECK(quantity > 0),
+    unit_price  INTEGER NOT NULL,
+    notes       TEXT,
+    created_at  TEXT NOT NULL
+);
+
+-- Index for retrieving items by order
+CREATE INDEX IF NOT EXISTS idx_order_items_order_id ON order_items(order_id);
+
+-- Index for filtering orders by status (kitchen display, etc.)
+CREATE INDEX IF NOT EXISTS idx_orders_status ON orders(status);
+
+-- Index for lookups by table
+CREATE INDEX IF NOT EXISTS idx_orders_table_id ON orders(table_id);
diff --git a/src-tauri/src/api/mod.rs b/src-tauri/src/api/mod.rs
index d7e5885..4162923 100644
--- a/src-tauri/src/api/mod.rs
+++ b/src-tauri/src/api/mod.rs
@@ -23,6 +23,7 @@ use axum::{
 use tauri::AppHandle;
 
 use crate::domain::product::ProductRepository;
+use crate::domain::order::OrderRepository;
 use crate::domain::user::UserRepository;
 use crate::domain::wallet::WalletRepository;
 
@@ -47,6 +48,7 @@ pub fn app_handle() -> &'static AppHandle {
 #[derive(Clone)]
 pub struct AppApiState {
 	pub user_repo: Arc<dyn UserRepository>,
+	pub order_repo: Arc<dyn OrderRepository>,
 	pub wallet_repo: Arc<dyn WalletRepository>,
 	pub product_repo: Arc<dyn ProductRepository>,
 	pub jwt_secret: Arc<Vec<u8>>,
@@ -109,7 +111,15 @@ pub fn build_app(state: AppApiState) -> Router {
 		.route(
 			"/api/categories/{id}",
 			delete(products::delete_category),
-		);
+		)
+		// Orders CRUD (story 3.2)
+		.route("/api/orders", post(orders::create_order))
+		.route("/api/orders", get(orders::list_orders))
+		.route("/api/orders/{id}/status", patch(orders::update_order_status))
+		.route("/api/orders/{id}/items", post(orders::add_order_item))
+		.route("/api/orders/{id}/items/{item_id}", delete(orders::remove_order_item))
+		.route("/api/orders/{id}", get(orders::get_order))
+		;
 
 	// Static file serving with SPA fallback.
 	if std::path::Path::new(&dist_path).exists() {
diff --git a/src-tauri/src/api/orders.rs b/src-tauri/src/api/orders.rs
index 8bfbacc..2384ad0 100644
--- a/src-tauri/src/api/orders.rs
+++ b/src-tauri/src/api/orders.rs
@@ -1,3 +1,516 @@
-//! Orders API — CRUD orders, status transitions.
-//! AD-13: Order depends on Catalog + Wallet.
-//! Story 3.2.
\ No newline at end of file
+//! Orders API — CRUD orders, status transitions, item management.
+//!
+//! Story 3.2. Handles /api/orders/*.
+//! AD-1: Thin API layer — delegates to domain via OrderRepository + ProductRepository.
+//! AD-8: Returns (StatusCode, Json<ApiError>) with standardized error format.
+//! AD-13: Order depends on Catalog (product lookups for validation).
+
+use std::sync::Arc;
+
+use axum::{
+	extract::{FromRef, Path, Query, State},
+	http::StatusCode,
+	response::IntoResponse,
+	Json,
+};
+use serde::{Deserialize, Serialize};
+
+use crate::domain::order::{Order, OrderItem, OrderRepository, OrderStatus};
+use crate::domain::product::ProductRepository;
+use crate::domain::DomainError;
+
+use super::AppApiState;
+
+// ─── State extraction ───────────────────────────────────────────────
+
+#[derive(Clone)]
+pub struct OrdersState {
+	pub order_repo: Arc<dyn OrderRepository>,
+	pub product_repo: Arc<dyn ProductRepository>,
+}
+
+impl FromRef<AppApiState> for OrdersState {
+	fn from_ref(state: &AppApiState) -> Self {
+		Self {
+			order_repo: state.order_repo.clone(),
+			product_repo: state.product_repo.clone(),
+		}
+	}
+}
+
+// ─── Request / Response types ───────────────────────────────────────
+
+#[derive(Deserialize)]
+pub struct CreateOrderItem {
+	pub product_id: String,
+	pub quantity: i64,
+	#[serde(default)]
+	pub notes: Option<String>,
+}
+
+#[derive(Deserialize)]
+pub struct CreateOrderRequest {
+	#[serde(default)]
+	pub table_id: Option<String>,
+	#[serde(default)]
+	pub client_id: Option<String>,
+	pub items: Vec<CreateOrderItem>,
+}
+
+#[derive(Deserialize)]
+pub struct UpdateStatusRequest {
+	pub status: String,
+}
+
+#[derive(Deserialize)]
+pub struct AddItemRequest {
+	pub product_id: String,
+	pub quantity: i64,
+	#[serde(default)]
+	pub notes: Option<String>,
+}
+
+#[derive(Deserialize)]
+pub struct OrderListQuery {
+	pub status: Option<String>,
+}
+
+#[derive(Serialize)]
+pub struct OrderItemResponse {
+	pub id: String,
+	pub order_id: String,
+	pub product_id: String,
+	pub quantity: i64,
+	pub unit_price: i64,
+	pub notes: Option<String>,
+	pub created_at: String,
+}
+
+impl From<OrderItem> for OrderItemResponse {
+	fn from(i: OrderItem) -> Self {
+		Self {
+			id: i.id,
+			order_id: i.order_id,
+			product_id: i.product_id,
+			quantity: i.quantity,
+			unit_price: i.unit_price,
+			notes: i.notes,
+			created_at: i.created_at,
+		}
+	}
+}
+
+#[derive(Serialize)]
+pub struct OrderResponse {
+	pub id: String,
+	pub table_id: Option<String>,
+	pub client_id: Option<String>,
+	pub status: String,
+	pub total: i64,
+	pub created_at: String,
+	pub updated_at: String,
+	pub items: Vec<OrderItemResponse>,
+}
+
+#[derive(Serialize)]
+pub struct ApiError {
+	pub error: String,
+	pub code: String,
+}
+
+// ─── Error helpers ──────────────────────────────────────────────────
+
+fn error_response(error: &str, code: &str, status: StatusCode) -> (StatusCode, Json<ApiError>) {
+	(status, Json(ApiError {
+		error: error.to_string(),
+		code: code.to_string(),
+	}))
+}
+
+fn domain_to_http(err: DomainError) -> (StatusCode, Json<ApiError>) {
+	match err {
+		DomainError::Unauthorized => (
+			StatusCode::UNAUTHORIZED,
+			Json(ApiError {
+				error: "Unauthorized".into(),
+				code: "UNAUTHORIZED".into(),
+			}),
+		),
+		DomainError::NotFound(msg) => (
+			StatusCode::NOT_FOUND,
+			Json(ApiError {
+				error: msg,
+				code: "NOT_FOUND".into(),
+			}),
+		),
+		DomainError::InvalidValue(msg) => (
+			StatusCode::BAD_REQUEST,
+			Json(ApiError {
+				error: msg,
+				code: "INVALID_VALUE".into(),
+			}),
+		),
+		DomainError::InvalidStatusTransition { from, to } => (
+			StatusCode::UNPROCESSABLE_ENTITY,
+			Json(ApiError {
+				error: format!("Invalid status transition: {} → {}", from, to),
+				code: "INVALID_STATUS_TRANSITION".into(),
+			}),
+		),
+		_ => (
+			StatusCode::INTERNAL_SERVER_ERROR,
+			Json(ApiError {
+				error: "Internal server error".into(),
+				code: "INTERNAL_ERROR".into(),
+			}),
+		),
+	}
+}
+
+fn uuid_v7() -> String {
+	use uuid::Uuid;
+	Uuid::now_v7().to_string()
+}
+
+fn chrono_now() -> String {
+	use chrono::Utc;
+	Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
+}
+
+// ─── Handlers ───────────────────────────────────────────────────────
+
+/// POST /api/orders — Create an order with items.
+///
+/// AC-2: Validates product existence, calculates total server-side.
+pub async fn create_order(
+	State(state): State<OrdersState>,
+	Json(req): Json<CreateOrderRequest>,
+) -> impl IntoResponse {
+	// Validate items not empty
+	if req.items.is_empty() {
+		return error_response("Items list must not be empty", "VALIDATION_ERROR", StatusCode::BAD_REQUEST)
+			.into_response();
+	}
+
+	// Validate products exist and get prices
+	let mut resolved_items: Vec<(String, i64, Option<String>)> = Vec::new();
+	for item in &req.items {
+		if item.quantity <= 0 {
+			return error_response("Invalid quantity", "INVALID_QUANTITY", StatusCode::UNPROCESSABLE_ENTITY)
+				.into_response();
+		}
+		match state.product_repo.find_product_by_id(&item.product_id) {
+			Ok(Some(product)) => {
+				resolved_items.push((item.product_id.clone(), product.price, item.notes.clone()));
+			}
+			Ok(None) => {
+				return error_response(
+					&format!("Product not found: {}", item.product_id),
+					"PRODUCT_NOT_FOUND",
+					StatusCode::UNPROCESSABLE_ENTITY,
+				)
+					.into_response();
+			}
+			Err(e) => return domain_to_http(e).into_response(),
+		}
+	}
+
+	let now = chrono_now();
+	let order_id = uuid_v7();
+
+	// Calculate total server-side
+	// Recalculate properly
+	let total: i64 = resolved_items.iter()
+		.enumerate()
+		.map(|(i, (_, price, _))| price * req.items[i].quantity)
+		.sum();
+
+	let order = Order::new(order_id.clone(), req.table_id, req.client_id, now.clone());
+
+	// Persist order
+	if let Err(e) = state.order_repo.create(&order) {
+		return domain_to_http(e).into_response();
+	}
+
+	// Persist items
+	let mut order_items: Vec<OrderItem> = Vec::new();
+	for (i, (product_id, unit_price, notes)) in resolved_items.iter().enumerate() {
+		let item_id = uuid_v7();
+		let item = OrderItem {
+			id: item_id,
+			order_id: order_id.clone(),
+			product_id: product_id.clone(),
+			quantity: req.items[i].quantity,
+			unit_price: *unit_price,
+			notes: notes.clone(),
+			created_at: now.clone(),
+		};
+		if let Err(e) = state.order_repo.add_item(&item) {
+			// Ignore cleanup error — log would go here
+			let _ = state.order_repo.remove_item(&item.id);
+			return domain_to_http(e).into_response();
+		}
+		order_items.push(item);
+	}
+
+	// Update total
+	if let Err(e) = state.order_repo.update_total(&order_id) {
+		return domain_to_http(e).into_response();
+	}
+
+	let response = OrderResponse {
+		id: order_id,
+		table_id: order.table_id,
+		client_id: order.client_id,
+		status: order.status.as_str().to_string(),
+		total,
+		created_at: order.created_at,
+		updated_at: order.updated_at,
+		items: order_items.into_iter().map(Into::into).collect(),
+	};
+
+	(StatusCode::CREATED, Json(response)).into_response()
+}
+
+/// GET /api/orders — List orders, optionally filtered by status.
+///
+/// AC-3: Returns orders sorted by created_at DESC, each with items.
+pub async fn list_orders(
+	State(state): State<OrdersState>,
+	Query(query): Query<OrderListQuery>,
+) -> impl IntoResponse {
+	let orders: Vec<Order> = if let Some(ref status_str) = query.status {
+		let status = match OrderStatus::from_str(status_str) {
+			Ok(s) => s,
+			Err(_) => {
+				return error_response(
+					&format!("Invalid status: {}", status_str),
+					"INVALID_VALUE",
+					StatusCode::BAD_REQUEST,
+				)
+					.into_response();
+			}
+		};
+		match state.order_repo.list_by_status(&status) {
+			Ok(orders) => orders,
+			Err(e) => return domain_to_http(e).into_response(),
+		}
+	} else {
+		match state.order_repo.list_all() {
+			Ok(orders) => orders,
+			Err(e) => return domain_to_http(e).into_response(),
+		}
+	};
+
+	// Enrich each order with items
+	let mut responses: Vec<OrderResponse> = Vec::with_capacity(orders.len());
+	for order in orders {
+		let items = match state.order_repo.get_items(&order.id) {
+			Ok(items) => items.into_iter().map(Into::into).collect(),
+			Err(e) => return domain_to_http(e).into_response(),
+		};
+		responses.push(OrderResponse {
+			id: order.id,
+			table_id: order.table_id,
+			client_id: order.client_id,
+			status: order.status.as_str().to_string(),
+			total: order.total,
+			created_at: order.created_at,
+			updated_at: order.updated_at,
+			items,
+		});
+	}
+
+	(StatusCode::OK, Json(responses)).into_response()
+}
+
+/// GET /api/orders/{id} — Get order details with items.
+///
+/// AC-4: Returns 200 with order + items, or 404 if not found.
+pub async fn get_order(
+	State(state): State<OrdersState>,
+	Path(id): Path<String>,
+) -> impl IntoResponse {
+	let order = match state.order_repo.find_by_id(&id) {
+		Ok(Some(order)) => order,
+		Ok(None) => {
+			return error_response("Order not found", "ORDER_NOT_FOUND", StatusCode::NOT_FOUND)
+				.into_response();
+		}
+		Err(e) => return domain_to_http(e).into_response(),
+	};
+
+	let items = match state.order_repo.get_items(&id) {
+		Ok(items) => items.into_iter().map(Into::into).collect(),
+		Err(e) => return domain_to_http(e).into_response(),
+	};
+
+	(StatusCode::OK, Json(OrderResponse {
+		id: order.id,
+		table_id: order.table_id,
+		client_id: order.client_id,
+		status: order.status.as_str().to_string(),
+		total: order.total,
+		created_at: order.created_at,
+		updated_at: order.updated_at,
+		items,
+	})).into_response()
+}
+
+/// PATCH /api/orders/{id}/status — Transition order status.
+///
+/// AC-5: Validates transitions via Order::transition_to().
+pub async fn update_order_status(
+	State(state): State<OrdersState>,
+	Path(id): Path<String>,
+	Json(req): Json<UpdateStatusRequest>,
+) -> impl IntoResponse {
+	let mut order = match state.order_repo.find_by_id(&id) {
+		Ok(Some(order)) => order,
+		Ok(None) => {
+			return error_response("Order not found", "ORDER_NOT_FOUND", StatusCode::NOT_FOUND)
+				.into_response();
+		}
+		Err(e) => return domain_to_http(e).into_response(),
+	};
+
+	let new_status = match OrderStatus::from_str(&req.status) {
+		Ok(s) => s,
+		Err(e) => return domain_to_http(e).into_response(),
+	};
+
+	if let Err(e) = order.transition_to(new_status) {
+		return domain_to_http(e).into_response();
+	}
+
+	if let Err(e) = state.order_repo.update_status(&id, &order.status) {
+		return domain_to_http(e).into_response();
+	}
+
+	let items = match state.order_repo.get_items(&id) {
+		Ok(items) => items.into_iter().map(Into::into).collect(),
+		Err(_) => vec![],
+	};
+
+	(StatusCode::OK, Json(OrderResponse {
+		id: order.id,
+		table_id: order.table_id,
+		client_id: order.client_id,
+		status: order.status.as_str().to_string(),
+		total: order.total,
+		created_at: order.created_at,
+		updated_at: order.updated_at,
+		items,
+	})).into_response()
+}
+
+/// POST /api/orders/{id}/items — Add an item to an existing order.
+///
+/// AC-6: Only allowed in PendingPayment status.
+pub async fn add_order_item(
+	State(state): State<OrdersState>,
+	Path(id): Path<String>,
+	Json(req): Json<AddItemRequest>,
+) -> impl IntoResponse {
+	if req.quantity <= 0 {
+		return error_response("Invalid quantity", "INVALID_QUANTITY", StatusCode::UNPROCESSABLE_ENTITY)
+			.into_response();
+	}
+
+	// Check order exists and is in PendingPayment
+	let order = match state.order_repo.find_by_id(&id) {
+		Ok(Some(order)) => order,
+		Ok(None) => {
+			return error_response("Order not found", "ORDER_NOT_FOUND", StatusCode::NOT_FOUND)
+				.into_response();
+		}
+		Err(e) => return domain_to_http(e).into_response(),
+	};
+
+	if order.status != OrderStatus::PendingPayment {
+		return error_response(
+			&format!("Cannot modify order in status: {}", order.status.as_str()),
+			"INVALID_ORDER_STATUS",
+			StatusCode::UNPROCESSABLE_ENTITY,
+		)
+			.into_response();
+	}
+
+	// Verify product exists
+	let unit_price = match state.product_repo.find_product_by_id(&req.product_id) {
+		Ok(Some(product)) => product.price,
+		Ok(None) => {
+			return error_response(
+				&format!("Product not found: {}", req.product_id),
+				"PRODUCT_NOT_FOUND",
+				StatusCode::UNPROCESSABLE_ENTITY,
+			)
+				.into_response();
+		}
+		Err(e) => return domain_to_http(e).into_response(),
+	};
+
+	let now = chrono_now();
+	let item = OrderItem {
+		id: uuid_v7(),
+		order_id: id.clone(),
+		product_id: req.product_id,
+		quantity: req.quantity,
+		unit_price,
+		notes: req.notes,
+		created_at: now,
+	};
+
+	if let Err(e) = state.order_repo.add_item(&item) {
+		return domain_to_http(e).into_response();
+	}
+
+	// Recalculate total
+	if let Err(e) = state.order_repo.update_total(&id) {
+		return domain_to_http(e).into_response();
+	}
+
+	(StatusCode::OK, Json(OrderItemResponse::from(item))).into_response()
+}
+
+/// DELETE /api/orders/{id}/items/{item_id} — Remove an item from an order.
+///
+/// AC-7: Only allowed in PendingPayment status. Recalculates total after removal.
+pub async fn remove_order_item(
+	State(state): State<OrdersState>,
+	Path((id, item_id)): Path<(String, String)>,
+) -> impl IntoResponse {
+	// Check order exists and is in PendingPayment
+	let order = match state.order_repo.find_by_id(&id) {
+		Ok(Some(order)) => order,
+		Ok(None) => {
+			return error_response("Order not found", "ORDER_NOT_FOUND", StatusCode::NOT_FOUND)
+				.into_response();
+		}
+		Err(e) => return domain_to_http(e).into_response(),
+	};
+
+	if order.status != OrderStatus::PendingPayment {
+		return error_response(
+			&format!("Cannot modify order in status: {}", order.status.as_str()),
+			"INVALID_ORDER_STATUS",
+			StatusCode::UNPROCESSABLE_ENTITY,
+		)
+			.into_response();
+	}
+
+	if let Err(e) = state.order_repo.remove_item(&item_id) {
+		return match e {
+			DomainError::NotFound(_) => error_response("Order item not found", "ITEM_NOT_FOUND", StatusCode::NOT_FOUND),
+			_ => domain_to_http(e),
+		}
+		.into_response();
+	}
+
+	// Recalculate total
+	if let Err(e) = state.order_repo.update_total(&id) {
+		return domain_to_http(e).into_response();
+	}
+
+	StatusCode::NO_CONTENT.into_response()
+}
diff --git a/src-tauri/src/db/orders.rs b/src-tauri/src/db/orders.rs
index aa90738..7b2d186 100644
--- a/src-tauri/src/db/orders.rs
+++ b/src-tauri/src/db/orders.rs
@@ -6,7 +6,8 @@
 use crate::domain::order::{Order, OrderItem, OrderRepository, OrderStatus};
 use crate::domain::DomainError;
 
-use super::SqlitePool;
+use super::{SqlitePool};
+use super::get_conn;
 
 pub struct DbOrderRepository {
 	pool: SqlitePool,
@@ -19,28 +20,203 @@ impl DbOrderRepository {
 }
 
 impl OrderRepository for DbOrderRepository {
-	fn create(&self, _order: &Order) -> Result<(), DomainError> {
-		todo!("Story 3.2")
+	fn create(&self, order: &Order) -> Result<(), DomainError> {
+		let conn = get_conn(&self.pool).map_err(|e| DomainError::Internal(e.to_string()))?;
+		conn.execute(
+			"INSERT INTO orders (id, table_id, client_id, status, total, created_at, updated_at) \
+			 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
+			rusqlite::params![
+				order.id,
+				order.table_id,
+				order.client_id,
+				order.status.as_str(),
+				order.total,
+				order.created_at,
+				order.updated_at,
+			],
+		)
+		.map_err(|e| DomainError::Internal(format!("Failed to create order: {}", e)))?;
+		Ok(())
 	}
-	fn update_status(&self, _id: &str, _status: &OrderStatus) -> Result<(), DomainError> {
-		todo!("Story 3.2")
+
+	fn update_status(&self, id: &str, status: &OrderStatus) -> Result<(), DomainError> {
+		let conn = get_conn(&self.pool).map_err(|e| DomainError::Internal(e.to_string()))?;
+		let now = chrono_now();
+		let affected = conn
+			.execute(
+				"UPDATE orders SET status = ?1, updated_at = ?2 WHERE id = ?3",
+				rusqlite::params![status.as_str(), now, id],
+			)
+			.map_err(|e| DomainError::Internal(format!("Failed to update order status: {}", e)))?;
+		if affected == 0 {
+			return Err(DomainError::NotFound(format!("Order {} not found", id)));
+		}
+		Ok(())
 	}
-	fn find_by_id(&self, _id: &str) -> Result<Option<Order>, DomainError> {
-		todo!("Story 3.2")
+
+	fn find_by_id(&self, id: &str) -> Result<Option<Order>, DomainError> {
+		let conn = get_conn(&self.pool).map_err(|e| DomainError::Internal(e.to_string()))?;
+		let mut stmt = conn
+			.prepare(
+				"SELECT id, table_id, client_id, status, total, created_at, updated_at \
+				 FROM orders WHERE id = ?1",
+			)
+			.map_err(|e| DomainError::Internal(e.to_string()))?;
+
+		let mut rows = stmt
+			.query_map(rusqlite::params![id], |row| {
+				let status_str: String = row.get("status")?;
+				Ok(Order {
+					id: row.get("id")?,
+					table_id: row.get("table_id")?,
+					client_id: row.get("client_id")?,
+					status: OrderStatus::from_str(&status_str).map_err(|e| {
+						rusqlite::Error::ToSqlConversionFailure(Box::new(e))
+					})?,
+					total: row.get("total")?,
+					created_at: row.get("created_at")?,
+					updated_at: row.get("updated_at")?,
+				})
+			})
+			.map_err(|e| DomainError::Internal(e.to_string()))?;
+
+		match rows.next() {
+			Some(Ok(order)) => Ok(Some(order)),
+			Some(Err(e)) => Err(DomainError::Internal(e.to_string()).into()),
+			None => Ok(None),
+		}
 	}
-	fn list_by_status(&self, _status: &OrderStatus) -> Result<Vec<Order>, DomainError> {
-		todo!("Story 3.2")
+
+	fn list_by_status(&self, status: &OrderStatus) -> Result<Vec<Order>, DomainError> {
+		let conn = get_conn(&self.pool).map_err(|e| DomainError::Internal(e.to_string()))?;
+		let mut stmt = conn
+			.prepare(
+				"SELECT id, table_id, client_id, status, total, created_at, updated_at \
+				 FROM orders WHERE status = ?1 ORDER BY created_at DESC",
+			)
+			.map_err(|e| DomainError::Internal(e.to_string()))?;
+
+		let orders = stmt
+			.query_map(rusqlite::params![status.as_str()], map_order_row)
+			.map_err(|e| DomainError::Internal(e.to_string()))?
+			.collect::<Result<Vec<_>, _>>()
+			.map_err(|e| DomainError::Internal(e.to_string()))?;
+
+		Ok(orders)
 	}
+
 	fn list_all(&self) -> Result<Vec<Order>, DomainError> {
-		todo!("Story 3.2")
+		let conn = get_conn(&self.pool).map_err(|e| DomainError::Internal(e.to_string()))?;
+		let mut stmt = conn
+			.prepare(
+				"SELECT id, table_id, client_id, status, total, created_at, updated_at \
+				 FROM orders ORDER BY created_at DESC",
+			)
+			.map_err(|e| DomainError::Internal(e.to_string()))?;
+
+		let orders = stmt
+			.query_map([], map_order_row)
+			.map_err(|e| DomainError::Internal(e.to_string()))?
+			.collect::<Result<Vec<_>, _>>()
+			.map_err(|e| DomainError::Internal(e.to_string()))?;
+
+		Ok(orders)
 	}
-	fn add_item(&self, _item: &OrderItem) -> Result<(), DomainError> {
-		todo!("Story 3.2")
+
+	fn add_item(&self, item: &OrderItem) -> Result<(), DomainError> {
+		let conn = get_conn(&self.pool).map_err(|e| DomainError::Internal(e.to_string()))?;
+		conn.execute(
+			"INSERT INTO order_items (id, order_id, product_id, quantity, unit_price, notes, created_at) \
+			 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
+			rusqlite::params![
+				item.id,
+				item.order_id,
+				item.product_id,
+				item.quantity,
+				item.unit_price,
+				item.notes,
+				item.created_at,
+			],
+		)
+		.map_err(|e| DomainError::Internal(format!("Failed to add order item: {}", e)))?;
+		Ok(())
 	}
-	fn get_items(&self, _order_id: &str) -> Result<Vec<OrderItem>, DomainError> {
-		todo!("Story 3.2")
+
+	fn get_items(&self, order_id: &str) -> Result<Vec<OrderItem>, DomainError> {
+		let conn = get_conn(&self.pool).map_err(|e| DomainError::Internal(e.to_string()))?;
+		let mut stmt = conn
+			.prepare(
+				"SELECT id, order_id, product_id, quantity, unit_price, notes, created_at \
+				 FROM order_items WHERE order_id = ?1 ORDER BY created_at ASC",
+			)
+			.map_err(|e| DomainError::Internal(e.to_string()))?;
+
+		let items = stmt
+			.query_map(rusqlite::params![order_id], |row| {
+				Ok(OrderItem {
+					id: row.get("id")?,
+					order_id: row.get("order_id")?,
+					product_id: row.get("product_id")?,
+					quantity: row.get("quantity")?,
+					unit_price: row.get("unit_price")?,
+					notes: row.get("notes")?,
+					created_at: row.get("created_at")?,
+				})
+			})
+			.map_err(|e| DomainError::Internal(e.to_string()))?
+			.collect::<Result<Vec<_>, _>>()
+			.map_err(|e| DomainError::Internal(e.to_string()))?;
+
+		Ok(items)
+	}
+
+	fn remove_item(&self, item_id: &str) -> Result<(), DomainError> {
+		let conn = get_conn(&self.pool).map_err(|e| DomainError::Internal(e.to_string()))?;
+		let affected = conn
+			.execute(
+				"DELETE FROM order_items WHERE id = ?1",
+				rusqlite::params![item_id],
+			)
+			.map_err(|e| DomainError::Internal(format!("Failed to remove order item: {}", e)))?;
+		if affected == 0 {
+			return Err(DomainError::NotFound(format!("Order item {} not found", item_id)));
+		}
+		Ok(())
 	}
-	fn remove_item(&self, _item_id: &str) -> Result<(), DomainError> {
-		todo!("Story 3.2")
+
+	fn update_total(&self, order_id: &str) -> Result<(), DomainError> {
+		let conn = get_conn(&self.pool).map_err(|e| DomainError::Internal(e.to_string()))?;
+		let now = chrono_now();
+		let affected = conn
+			.execute(
+				"UPDATE orders SET total = COALESCE((SELECT SUM(quantity * unit_price) FROM order_items WHERE order_id = ?1), 0), updated_at = ?2 WHERE id = ?1",
+				rusqlite::params![order_id, now],
+			)
+			.map_err(|e| DomainError::Internal(format!("Failed to update order total: {}", e)))?;
+		if affected == 0 {
+			return Err(DomainError::NotFound(format!("Order {} not found", order_id)));
+		}
+		Ok(())
 	}
 }
+
+/// Helper to map a SQL row to an Order.
+fn map_order_row(row: &rusqlite::Row) -> rusqlite::Result<Order> {
+	let status_str: String = row.get("status")?;
+	Ok(Order {
+		id: row.get("id")?,
+		table_id: row.get("table_id")?,
+		client_id: row.get("client_id")?,
+		status: OrderStatus::from_str(&status_str).map_err(|e| {
+			rusqlite::Error::ToSqlConversionFailure(Box::new(e))
+		})?,
+		total: row.get("total")?,
+		created_at: row.get("created_at")?,
+		updated_at: row.get("updated_at")?,
+	})
+}
+
+fn chrono_now() -> String {
+	use chrono::Utc;
+	Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
+}
diff --git a/src-tauri/src/domain/order.rs b/src-tauri/src/domain/order.rs
index a7d2103..c7bda2d 100644
--- a/src-tauri/src/domain/order.rs
+++ b/src-tauri/src/domain/order.rs
@@ -54,6 +54,7 @@ pub struct OrderItem {
 	pub quantity: i64,
 	pub unit_price: i64,
 	pub notes: Option<String>,
+	pub created_at: String,
 }
 
 /// A customer order with its lifecycle status.
@@ -114,4 +115,6 @@ pub trait OrderRepository: Send + Sync {
 	fn add_item(&self, item: &OrderItem) -> Result<(), DomainError>;
 	fn get_items(&self, order_id: &str) -> Result<Vec<OrderItem>, DomainError>;
 	fn remove_item(&self, item_id: &str) -> Result<(), DomainError>;
+	/// Recalculate and persist the order total from order_items.
+	fn update_total(&self, order_id: &str) -> Result<(), DomainError>;
 }
diff --git a/src-tauri/src/lib.rs b/src-tauri/src/lib.rs
index 957a2b9..009b236 100644
--- a/src-tauri/src/lib.rs
+++ b/src-tauri/src/lib.rs
@@ -31,8 +31,10 @@ use tauri::Manager;
 
 use api::AppApiState;
 use db::users::DbUserRepository;
+use db::orders::DbOrderRepository;
 use db::products::DbProductRepository;
 use crate::domain::product::ProductRepository;
+use crate::domain::order::OrderRepository;
 use db::wallet_ledger::DbWalletRepository;
 use crate::domain::wallet::WalletRepository;
 use domain::user::UserRepository;
@@ -104,10 +106,12 @@ pub fn run() {
 	// Build the full application router
 	let user_repo: Arc<dyn UserRepository> = Arc::new(DbUserRepository::new(pool.clone()));
 	let wallet_repo: Arc<dyn WalletRepository> = Arc::new(DbWalletRepository::new(pool.clone()));
+	let order_repo: Arc<dyn OrderRepository> = Arc::new(DbOrderRepository::new(pool.clone()));
 	let product_repo: Arc<dyn ProductRepository> = Arc::new(DbProductRepository::new(pool.clone()));
 	let api_state = AppApiState {
 		user_repo,
 		wallet_repo,
+		order_repo,
 		product_repo,
 		jwt_secret,
 	};
diff --git a/src-tauri/migrations/V4__orders.sql b/src-tauri/migrations/V4__orders.sql
new file mode 100644
index 0000000..3df5c08
--- /dev/null
+++ b/src-tauri/migrations/V4__orders.sql
@@ -0,0 +1,39 @@
+-- V4__orders.sql
+-- Order lifecycle: orders and order_items tables.
+--
+-- AD-13: Order depends on Catalog (product_id FK conceptual, no FK constraint
+--         to avoid blocking product deletion). Referential integrity is
+--         enforced at the application layer via ProductRepository lookups.
+-- AD-2:  order_items is mutable (add/remove items allowed in PendingPayment).
+--         Once past PendingPayment, mutability is gated by the domain layer.
+--         Financial mutation is NOT in this table -- wallet_ledger (V2) is
+--         the append-only financial record.
+
+CREATE TABLE IF NOT EXISTS orders (
+    id          TEXT PRIMARY KEY,
+    table_id    TEXT,
+    client_id   TEXT,
+    status      TEXT NOT NULL DEFAULT 'pending_payment',
+    total       INTEGER NOT NULL DEFAULT 0,
+    created_at  TEXT NOT NULL,
+    updated_at  TEXT NOT NULL
+);
+
+CREATE TABLE IF NOT EXISTS order_items (
+    id          TEXT PRIMARY KEY,
+    order_id    TEXT NOT NULL REFERENCES orders(id),
+    product_id  TEXT NOT NULL,
+    quantity    INTEGER NOT NULL CHECK(quantity > 0),
+    unit_price  INTEGER NOT NULL,
+    notes       TEXT,
+    created_at  TEXT NOT NULL
+);
+
+-- Index for retrieving items by order
+CREATE INDEX IF NOT EXISTS idx_order_items_order_id ON order_items(order_id);
+
+-- Index for filtering orders by status (kitchen display, etc.)
+CREATE INDEX IF NOT EXISTS idx_orders_status ON orders(status);
+
+-- Index for lookups by table
+CREATE INDEX IF NOT EXISTS idx_orders_table_id ON orders(table_id);
```

## Output Format

```markdown
### Finding N: [CRITICAL|MAJOR|MINOR] — Title

**File:** path.rs:line
**Scenario:** What the edge case is
**Current behavior:** What happens now
**Expected behavior:** What should happen
**Suggestion:** How to fix
```
