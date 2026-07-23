//! Product domain — Product, Category, and ProductRepository trait.
//!
//! AD-13: Products module is independent (no dependencies on other domains).
//! AD-7: ProductRepository trait defined here, implemented in db/products.rs.

use super::DomainError;

/// Current timestamp in ISO 8601 UTC format.
fn now_iso() -> String {
	use chrono::Utc;
	Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
}

/// Generate a UUID v7 string.
fn uuid_v7() -> String {
	use uuid::Uuid;
	Uuid::now_v7().to_string()
}

impl Product {
	/// Create a new product with generated UUID v7 and timestamps.
	/// Validates that name is non-empty and price >= 0.
	pub fn new(
		name: String,
		price: i64,
		category_id: String,
		stock: i64,
		alert_threshold: i64,
	) -> Result<Self, DomainError> {
		if name.trim().is_empty() {
			return Err(DomainError::InvalidValue(
				"Product name cannot be empty".into(),
			));
		}
		if price < 0 {
			return Err(DomainError::InvalidValue(
				"Product price cannot be negative".into(),
			));
		}
		if stock < 0 {
			return Err(DomainError::InvalidValue(
				"Stock cannot be negative".into(),
			));
		}
		if alert_threshold < 0 {
			return Err(DomainError::InvalidValue(
				"Alert threshold cannot be negative".into(),
			));
		}
		let now = now_iso();
		Ok(Self {
			id: uuid_v7(),
			name,
			price,
			category_id,
			stock,
			alert_threshold,
			created_at: now.clone(),
			updated_at: now,
		})
	}

	/// Update product fields and refresh updated_at.
	pub fn update(
		&mut self,
		name: String,
		price: i64,
		category_id: String,
		stock: i64,
		alert_threshold: i64,
	) -> Result<(), DomainError> {
		if name.trim().is_empty() {
			return Err(DomainError::InvalidValue(
				"Product name cannot be empty".into(),
			));
		}
		if price < 0 {
			return Err(DomainError::InvalidValue(
				"Product price cannot be negative".into(),
			));
		}
		if stock < 0 {
			return Err(DomainError::InvalidValue(
				"Stock cannot be negative".into(),
			));
		}
		if alert_threshold < 0 {
			return Err(DomainError::InvalidValue(
				"Alert threshold cannot be negative".into(),
			));
		}
		self.name = name;
		self.price = price;
		self.category_id = category_id;
		self.stock = stock;
		self.alert_threshold = alert_threshold;
		self.updated_at = now_iso();
		Ok(())
	}
}

impl Category {
	/// Create a new category with generated UUID v7 and timestamps.
	pub fn new(name: String, parent_id: Option<String>) -> Result<Self, DomainError> {
		if name.trim().is_empty() {
			return Err(DomainError::InvalidValue(
				"Category name cannot be empty".into(),
			));
		}
		let now = now_iso();
		Ok(Self {
			id: uuid_v7(),
			name,
			parent_id,
			created_at: now.clone(),
			updated_at: now,
		})
	}

	/// Update category name and refresh updated_at.
	pub fn update(
		&mut self,
		name: String,
		parent_id: Option<String>,
	) -> Result<(), DomainError> {
		if name.trim().is_empty() {
			return Err(DomainError::InvalidValue(
				"Category name cannot be empty".into(),
			));
		}
		self.name = name;
		self.parent_id = parent_id;
		self.updated_at = now_iso();
		Ok(())
	}
}

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
	fn list_products_by_category(
		&self,
		category_id: &str,
	) -> Result<Vec<Product>, DomainError>;
	fn search_products(&self, query: &str) -> Result<Vec<Product>, DomainError>;
	fn list_all_products(&self) -> Result<Vec<Product>, DomainError>;

	fn create_category(&self, category: &Category) -> Result<(), DomainError>;
	fn update_category(&self, category: &Category) -> Result<(), DomainError>;
	fn delete_category(&self, id: &str) -> Result<(), DomainError>;
	fn find_category_by_id(&self, id: &str) -> Result<Option<Category>, DomainError>;
	fn list_all_categories(&self) -> Result<Vec<Category>, DomainError>;

	/// Count products belonging to a category (used for delete guard).
	fn count_products_by_category(&self, category_id: &str) -> Result<i64, DomainError>;

	/// Count child categories referencing this parent_id.
	fn count_child_categories(&self, parent_id: &str) -> Result<i64, DomainError>;

	/// Find child categories by parent_id.
	fn find_child_categories(
		&self,
		parent_id: &str,
	) -> Result<Vec<Category>, DomainError>;
}
