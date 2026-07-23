//! Products & Categories API — CRUD catalogue.
//!
//! Story 3.1. Handles /api/products/* and /api/categories/*.
//! AD-1: Thin API layer — delegates to domain via ProductRepository.
//! AD-8: Returns (StatusCode, Json<ApiError>) with standardized error format.

use std::sync::Arc;

use axum::{
	extract::{FromRef, Path, Query, State},
	http::StatusCode,
	response::IntoResponse,
	Json,
};
use serde::{Deserialize, Serialize};

use crate::domain::product::{Category, Product, ProductRepository};
use crate::domain::DomainError;

use super::AppApiState;

// ─── State extraction ───────────────────────────────────────────────

#[derive(Clone)]
pub struct ProductsState {
	pub product_repo: Arc<dyn ProductRepository>,
}

impl FromRef<AppApiState> for ProductsState {
	fn from_ref(state: &AppApiState) -> Self {
		Self {
			product_repo: state.product_repo.clone(),
		}
	}
}

// ─── Request / Response types ───────────────────────────────────────

#[derive(Deserialize)]
pub struct CreateProductRequest {
	pub name: String,
	pub price: i64,
	pub category_id: String,
	pub stock: Option<i64>,
	pub alert_threshold: Option<i64>,
}

#[derive(Deserialize)]
pub struct UpdateProductRequest {
	pub name: String,
	pub price: i64,
	pub category_id: String,
	pub stock: Option<i64>,
	pub alert_threshold: Option<i64>,
}

#[derive(Serialize)]
pub struct ProductResponse {
	pub id: String,
	pub name: String,
	pub price: i64,
	pub category_id: String,
	pub stock: i64,
	pub alert_threshold: i64,
	pub created_at: String,
	pub updated_at: String,
}

impl From<Product> for ProductResponse {
	fn from(p: Product) -> Self {
		Self {
			id: p.id,
			name: p.name,
			price: p.price,
			category_id: p.category_id,
			stock: p.stock,
			alert_threshold: p.alert_threshold,
			created_at: p.created_at,
			updated_at: p.updated_at,
		}
	}
}

#[derive(Deserialize)]
pub struct CreateCategoryRequest {
	pub name: String,
	pub parent_id: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateCategoryRequest {
	pub name: String,
	pub parent_id: Option<String>,
}

#[derive(Serialize)]
pub struct CategoryResponse {
	pub id: String,
	pub name: String,
	pub parent_id: Option<String>,
	pub created_at: String,
	pub updated_at: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub children: Option<Vec<CategoryResponse>>,
}

impl From<Category> for CategoryResponse {
	fn from(c: Category) -> Self {
		Self {
			id: c.id,
			name: c.name,
			parent_id: c.parent_id,
			created_at: c.created_at,
			updated_at: c.updated_at,
			children: None,
		}
	}
}

impl CategoryResponse {
	fn with_children(cat: Category, children: Vec<Category>) -> Self {
		Self {
			id: cat.id,
			name: cat.name,
			parent_id: cat.parent_id,
			created_at: cat.created_at,
			updated_at: cat.updated_at,
			children: Some(children.into_iter().map(Into::into).collect()),
		}
	}
}

#[derive(Deserialize)]
pub struct ProductQueryParams {
	pub category: Option<String>,
}

/// Standard API error response.
#[derive(Serialize)]
pub struct ApiError {
	pub error: String,
	pub code: String,
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
		DomainError::ProductNotFound => (
			StatusCode::NOT_FOUND,
			Json(ApiError {
				error: "Product not found".into(),
				code: "PRODUCT_NOT_FOUND".into(),
			}),
		),
		DomainError::InvalidValue(msg) => (
			StatusCode::BAD_REQUEST,
			Json(ApiError {
				error: msg,
				code: "INVALID_VALUE".into(),
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

// ─── Products handlers ──────────────────────────────────────────────

/// POST /api/products
pub async fn create_product(
	State(state): State<ProductsState>,
	Json(req): Json<CreateProductRequest>,
) -> impl IntoResponse {
	// Verify category exists (AC-3)
	match state.product_repo.find_category_by_id(&req.category_id) {
		Ok(Some(_)) => {}
		Ok(None) => {
			return (
				StatusCode::UNPROCESSABLE_ENTITY,
				Json(ApiError {
					error: "Category not found".into(),
					code: "CATEGORY_NOT_FOUND".into(),
				}),
			)
				.into_response();
		}
		Err(e) => return domain_to_http(e).into_response(),
	}

	let stock = req.stock.unwrap_or(0);
	let alert = req.alert_threshold.unwrap_or(5);

	let product = match Product::new(req.name, req.price, req.category_id, stock, alert) {
		Ok(p) => p,
		Err(e) => return domain_to_http(e).into_response(),
	};

	match state.product_repo.create_product(&product) {
		Ok(()) => (StatusCode::CREATED, Json(ProductResponse::from(product))).into_response(),
		Err(e) => domain_to_http(e).into_response(),
	}
}

/// GET /api/products?category={id}
/// GET /api/products
pub async fn list_products(
	State(state): State<ProductsState>,
	Query(params): Query<ProductQueryParams>,
) -> impl IntoResponse {
	let products = if let Some(category_id) = params.category {
		if category_id.is_empty() {
			return (
				StatusCode::BAD_REQUEST,
				Json(ApiError {
					error: "category parameter is empty".into(),
					code: "INVALID_CATEGORY".into(),
				}),
			)
				.into_response();
		}
		match state.product_repo.list_products_by_category(&category_id) {
			Ok(list) => list,
			Err(e) => return domain_to_http(e).into_response(),
		}
	} else {
		match state.product_repo.list_all_products() {
			Ok(list) => list,
			Err(e) => return domain_to_http(e).into_response(),
		}
	};

	let response: Vec<ProductResponse> = products.into_iter().map(Into::into).collect();
	(StatusCode::OK, Json(response)).into_response()
}

/// GET /api/products/{id}
pub async fn get_product(
	State(state): State<ProductsState>,
	Path(id): Path<String>,
) -> impl IntoResponse {
	match state.product_repo.find_product_by_id(&id) {
		Ok(Some(product)) => {
			(StatusCode::OK, Json(ProductResponse::from(product))).into_response()
		}
		Ok(None) => (
			StatusCode::NOT_FOUND,
			Json(ApiError {
				error: "Product not found".into(),
				code: "PRODUCT_NOT_FOUND".into(),
			}),
		)
			.into_response(),
		Err(e) => domain_to_http(e).into_response(),
	}
}

/// PUT /api/products/{id}
pub async fn update_product(
	State(state): State<ProductsState>,
	Path(id): Path<String>,
	Json(req): Json<UpdateProductRequest>,
) -> impl IntoResponse {
	// Find existing product
	let mut product = match state.product_repo.find_product_by_id(&id) {
		Ok(Some(p)) => p,
		Ok(None) => {
			return (
				StatusCode::NOT_FOUND,
				Json(ApiError {
					error: "Product not found".into(),
					code: "PRODUCT_NOT_FOUND".into(),
				}),
			)
				.into_response()
		}
		Err(e) => return domain_to_http(e).into_response(),
	};

	// PATCH semantics: use existing values when fields are not provided
	let name = req.name;
	let price = req.price;
	let category_id = req.category_id;
	let stock = req.stock.unwrap_or(product.stock);
	let alert = req.alert_threshold.unwrap_or(product.alert_threshold);

	// Verify category exists (AC-5 consistency)
	match state.product_repo.find_category_by_id(&category_id) {
		Ok(Some(_)) => {}
		Ok(None) => {
			return (
				StatusCode::UNPROCESSABLE_ENTITY,
				Json(ApiError {
					error: "Category not found".into(),
					code: "CATEGORY_NOT_FOUND".into(),
				}),
			)
				.into_response();
		}
		Err(e) => return domain_to_http(e).into_response(),
	}

	// Update fields via domain logic
	if let Err(e) = product.update(name, price, category_id, stock, alert) {
		return domain_to_http(e).into_response();
	}

	match state.product_repo.update_product(&product) {
		Ok(()) => (StatusCode::OK, Json(ProductResponse::from(product))).into_response(),
		Err(e) => domain_to_http(e).into_response(),
	}
}

/// DELETE /api/products/{id}
pub async fn delete_product(
	State(state): State<ProductsState>,
	Path(id): Path<String>,
) -> impl IntoResponse {
	match state.product_repo.delete_product(&id) {
		Ok(()) => StatusCode::NO_CONTENT.into_response(),
		Err(DomainError::ProductNotFound) => (
			StatusCode::NOT_FOUND,
			Json(ApiError {
				error: "Product not found".into(),
				code: "PRODUCT_NOT_FOUND".into(),
			}),
		)
			.into_response(),
		Err(e) => domain_to_http(e).into_response(),
	}
}

// ─── Categories handlers ────────────────────────────────────────────

/// POST /api/categories
pub async fn create_category(
	State(state): State<ProductsState>,
	Json(req): Json<CreateCategoryRequest>,
) -> impl IntoResponse {
	// Validate parent_id if provided: check it exists
	if let Some(ref parent_id) = req.parent_id {
		match state.product_repo.find_category_by_id(parent_id) {
			Ok(Some(_)) => {}
			Ok(None) => {
				return (
					StatusCode::UNPROCESSABLE_ENTITY,
					Json(ApiError {
						error: "Parent category not found".into(),
						code: "PARENT_CATEGORY_NOT_FOUND".into(),
					}),
				)
					.into_response()
			}
			Err(e) => return domain_to_http(e).into_response(),
		}
	}

	let category = match Category::new(req.name, req.parent_id) {
		Ok(c) => c,
		Err(e) => return domain_to_http(e).into_response(),
	};

	match state.product_repo.create_category(&category) {
		Ok(()) => (StatusCode::CREATED, Json(CategoryResponse::from(category))).into_response(),
		Err(e) => domain_to_http(e).into_response(),
	}
}

/// GET /api/categories
pub async fn list_categories(
	State(state): State<ProductsState>,
) -> impl IntoResponse {
	match state.product_repo.list_all_categories() {
		Ok(categories) => {
			let response: Vec<CategoryResponse> = categories.into_iter().map(Into::into).collect();
			(StatusCode::OK, Json(response)).into_response()
		}
		Err(e) => domain_to_http(e).into_response(),
	}
}

/// GET /api/categories/{id}
pub async fn get_category(
	State(state): State<ProductsState>,
	Path(id): Path<String>,
) -> impl IntoResponse {
	let category = match state.product_repo.find_category_by_id(&id) {
		Ok(Some(c)) => c,
		Ok(None) => {
			return (
				StatusCode::NOT_FOUND,
				Json(ApiError {
					error: "Category not found".into(),
					code: "CATEGORY_NOT_FOUND".into(),
				}),
			)
				.into_response()
		}
		Err(e) => return domain_to_http(e).into_response(),
	};

	// Fetch children
	let children = match state.product_repo.find_child_categories(&id) {
		Ok(list) => list,
		Err(e) => return domain_to_http(e).into_response(),
	};

	(
		StatusCode::OK,
		Json(CategoryResponse::with_children(category, children)),
	)
		.into_response()
}

/// PUT /api/categories/{id}
pub async fn update_category(
	State(state): State<ProductsState>,
	Path(id): Path<String>,
	Json(req): Json<UpdateCategoryRequest>,
) -> impl IntoResponse {
	// Validate parent_id if provided
	if let Some(ref parent_id) = req.parent_id {
		if parent_id == &id {
			return (
				StatusCode::UNPROCESSABLE_ENTITY,
				Json(ApiError {
					error: "Category cannot be its own parent".into(),
					code: "SELF_PARENT".into(),
				}),
			)
				.into_response();
		}
		match state.product_repo.find_category_by_id(parent_id) {
			Ok(Some(_)) => {}
			Ok(None) => {
				return (
					StatusCode::UNPROCESSABLE_ENTITY,
					Json(ApiError {
						error: "Parent category not found".into(),
						code: "PARENT_CATEGORY_NOT_FOUND".into(),
					}),
				)
					.into_response()
			}
			Err(e) => return domain_to_http(e).into_response(),
		}
	}

	let mut category = match state.product_repo.find_category_by_id(&id) {
		Ok(Some(c)) => c,
		Ok(None) => {
			return (
				StatusCode::NOT_FOUND,
				Json(ApiError {
					error: "Category not found".into(),
					code: "CATEGORY_NOT_FOUND".into(),
				}),
			)
				.into_response()
		}
		Err(e) => return domain_to_http(e).into_response(),
	};

	if let Err(e) = category.update(req.name, req.parent_id) {
		return domain_to_http(e).into_response();
	}

	match state.product_repo.update_category(&category) {
		Ok(()) => (StatusCode::OK, Json(CategoryResponse::from(category))).into_response(),
		Err(e) => domain_to_http(e).into_response(),
	}
}

/// DELETE /api/categories/{id}
pub async fn delete_category(
	State(state): State<ProductsState>,
	Path(id): Path<String>,
) -> impl IntoResponse {
	// Check if category exists
	match state.product_repo.find_category_by_id(&id) {
		Ok(Some(_)) => {}
		Ok(None) => {
			return (
				StatusCode::NOT_FOUND,
				Json(ApiError {
					error: "Category not found".into(),
					code: "CATEGORY_NOT_FOUND".into(),
				}),
			)
				.into_response()
		}
		Err(e) => return domain_to_http(e).into_response(),
	}

	// Guard: category has products?
	match state.product_repo.count_products_by_category(&id) {
		Ok(count) if count > 0 => {
			return (
				StatusCode::UNPROCESSABLE_ENTITY,
				Json(ApiError {
					error: format!("Cannot delete category with {} product(s) attached", count),
					code: "CATEGORY_HAS_PRODUCTS".into(),
				}),
			)
				.into_response()
		}
		Ok(_) => {}
		Err(e) => return domain_to_http(e).into_response(),
	}

	// Guard: category has children?
	match state.product_repo.count_child_categories(&id) {
		Ok(count) if count > 0 => {
			return (
				StatusCode::UNPROCESSABLE_ENTITY,
				Json(ApiError {
					error: format!("Cannot delete category with {} sub-categor(ies)", count),
					code: "CATEGORY_HAS_CHILDREN".into(),
				}),
			)
				.into_response()
		}
		Ok(_) => {}
		Err(e) => return domain_to_http(e).into_response(),
	}

	match state.product_repo.delete_category(&id) {
		Ok(()) => StatusCode::NO_CONTENT.into_response(),
		Err(e) => domain_to_http(e).into_response(),
	}
}
