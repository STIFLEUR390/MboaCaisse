//! Product domain — Product, Category, and ProductRepository trait.
//!
//! AD-13: Products module is independent (no dependencies on other domains).
//! AD-7: ProductRepository trait defined here, implemented in db/products.rs.

use super::DomainError;

/// A product sold by the establishment.
#[derive(Debug, Clone)]
pub struct Product {
	pub id: String,
	pub name: String,
	pub price: i64,
	pub category_id: String,
	pub stock: i64,
	pub alert_threshold: i64,
	pub created_at: String,
	pub updated_at: String,
}

/// A category (or sub-category) for organising products.
#[derive(Debug, Clone)]
pub struct Category {
	pub id: String,
	pub name: String,
	pub parent_id: Option<String>,
	pub created_at: String,
	pub updated_at: String,
}

/// Repository trait for Product/Category persistence.
///
/// AD-7: Defined in domain/, implemented in db/.
pub trait ProductRepository: Send + Sync {
	fn create_product(&self, product: &Product) -> Result<(), DomainError>;
	fn update_product(&self, product: &Product) -> Result<(), DomainError>;
	fn delete_product(&self, id: &str) -> Result<(), DomainError>;
	fn find_product_by_id(&self, id: &str) -> Result<Option<Product>, DomainError>;
	fn list_products_by_category(&self, category_id: &str) -> Result<Vec<Product>, DomainError>;
	fn search_products(&self, query: &str) -> Result<Vec<Product>, DomainError>;
	fn list_all_products(&self) -> Result<Vec<Product>, DomainError>;

	fn create_category(&self, category: &Category) -> Result<(), DomainError>;
	fn update_category(&self, category: &Category) -> Result<(), DomainError>;
	fn delete_category(&self, id: &str) -> Result<(), DomainError>;
	fn find_category_by_id(&self, id: &str) -> Result<Option<Category>, DomainError>;
	fn list_all_categories(&self) -> Result<Vec<Category>, DomainError>;
}
