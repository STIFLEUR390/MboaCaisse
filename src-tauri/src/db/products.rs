//! ProductRepository implementation using rusqlite.
//!
//! AD-7: Implements trait from domain/product.rs.
//! AD-16: Uses r2d2 connection pool.

use crate::domain::product::{Category, Product, ProductRepository};
use crate::domain::DomainError;

use super::{DbError, SqlitePool};

pub struct DbProductRepository {
	pool: SqlitePool,
}

impl DbProductRepository {
	pub fn new(pool: SqlitePool) -> Self {
		Self { pool }
	}
}

impl ProductRepository for DbProductRepository {
	fn create_product(&self, _product: &Product) -> Result<(), DomainError> {
		todo!("Story 3.1")
	}
	fn update_product(&self, _product: &Product) -> Result<(), DomainError> {
		todo!("Story 3.1")
	}
	fn delete_product(&self, _id: &str) -> Result<(), DomainError> {
		todo!("Story 3.1")
	}
	fn find_product_by_id(&self, _id: &str) -> Result<Option<Product>, DomainError> {
		todo!("Story 3.1")
	}
	fn list_products_by_category(&self, _category_id: &str) -> Result<Vec<Product>, DomainError> {
		todo!("Story 3.1")
	}
	fn search_products(&self, _query: &str) -> Result<Vec<Product>, DomainError> {
		todo!("Story 3.1")
	}
	fn list_all_products(&self) -> Result<Vec<Product>, DomainError> {
		todo!("Story 3.1")
	}
	fn create_category(&self, _category: &Category) -> Result<(), DomainError> {
		todo!("Story 3.1")
	}
	fn update_category(&self, _category: &Category) -> Result<(), DomainError> {
		todo!("Story 3.1")
	}
	fn delete_category(&self, _id: &str) -> Result<(), DomainError> {
		todo!("Story 3.1")
	}
	fn find_category_by_id(&self, _id: &str) -> Result<Option<Category>, DomainError> {
		todo!("Story 3.1")
	}
	fn list_all_categories(&self) -> Result<Vec<Category>, DomainError> {
		todo!("Story 3.1")
	}
}
