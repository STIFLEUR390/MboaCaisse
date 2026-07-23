//! ProductRepository implementation using rusqlite.
//!
//! AD-7: Implements trait from domain/product.rs.
//! AD-16: Uses r2d2 connection pool.

use crate::domain::product::{Category, Product, ProductRepository};
use crate::domain::DomainError;

use super::{get_conn, DbError, SqlitePool};

pub struct DbProductRepository {
	pool: SqlitePool,
}

impl DbProductRepository {
	pub fn new(pool: SqlitePool) -> Self {
		Self { pool }
	}
}

// ─── Helpers ─────────────────────────────────────────────────────────

fn row_to_product(row: &rusqlite::Row) -> rusqlite::Result<Product> {
	Ok(Product {
		id: row.get("id")?,
		name: row.get("name")?,
		price: row.get("price")?,
		category_id: row.get("category_id")?,
		stock: row.get("stock")?,
		alert_threshold: row.get("alert_threshold")?,
		created_at: row.get("created_at")?,
		updated_at: row.get("updated_at")?,
	})
}

fn row_to_category(row: &rusqlite::Row) -> rusqlite::Result<Category> {
	Ok(Category {
		id: row.get("id")?,
		name: row.get("name")?,
		parent_id: row.get("parent_id")?,
		created_at: row.get("created_at")?,
		updated_at: row.get("updated_at")?,
	})
}

// ─── Map DbError → DomainError ──────────────────────────────────────

// ─── ProductRepository implementation ───────────────────────────────

impl ProductRepository for DbProductRepository {
	// ─── Products ─────────────────────────────────────────────────

	fn create_product(&self, product: &Product) -> Result<(), DomainError> {
		let conn = get_conn(&self.pool).map_err(|e| DomainError::Internal(e.to_string()))?;
		conn.execute(
			"INSERT INTO products (id, name, price, category_id, stock, alert_threshold, created_at, updated_at) \
			 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
			rusqlite::params![
				product.id, product.name, product.price, product.category_id,
				product.stock, product.alert_threshold, product.created_at, product.updated_at
			],
		)
		.map_err(|e| DomainError::Internal(format!("Failed to create product: {}", e)))?;
		Ok(())
	}

	fn update_product(&self, product: &Product) -> Result<(), DomainError> {
		let conn = get_conn(&self.pool).map_err(|e| DomainError::Internal(e.to_string()))?;
		let affected = conn
			.execute(
				"UPDATE products SET name = ?1, price = ?2, category_id = ?3, stock = ?4, \
				 alert_threshold = ?5, updated_at = ?6 WHERE id = ?7",
				rusqlite::params![
					product.name, product.price, product.category_id,
					product.stock, product.alert_threshold, product.updated_at, product.id
				],
			)
			.map_err(|e| DomainError::Internal(format!("Failed to update product: {}", e)))?;
		if affected == 0 {
			return Err(DomainError::ProductNotFound);
		}
		Ok(())
	}

	fn delete_product(&self, id: &str) -> Result<(), DomainError> {
		let conn = get_conn(&self.pool).map_err(|e| DomainError::Internal(e.to_string()))?;
		let affected = conn
			.execute("DELETE FROM products WHERE id = ?1", rusqlite::params![id])
			.map_err(|e| DomainError::Internal(format!("Failed to delete product: {}", e)))?;
		if affected == 0 {
			return Err(DomainError::ProductNotFound);
		}
		Ok(())
	}

	fn find_product_by_id(&self, id: &str) -> Result<Option<Product>, DomainError> {
		let conn = get_conn(&self.pool).map_err(|e| DomainError::Internal(e.to_string()))?;
		let mut stmt = conn
			.prepare("SELECT * FROM products WHERE id = ?1")
			.map_err(|e| DomainError::Internal(format!("Failed to prepare query: {}", e)))?;
		let mut rows = stmt
			.query_map(rusqlite::params![id], row_to_product)
			.map_err(|e| DomainError::Internal(format!("Failed to query product: {}", e)))?;
		match rows.next() {
			Some(Ok(product)) => Ok(Some(product)),
			Some(Err(e)) => Err(DomainError::Internal(format!("Failed to read product row: {}", e))),
			None => Ok(None),
		}
	}

	fn list_products_by_category(&self, category_id: &str) -> Result<Vec<Product>, DomainError> {
		let conn = get_conn(&self.pool).map_err(|e| DomainError::Internal(e.to_string()))?;
		let mut stmt = conn
			.prepare("SELECT * FROM products WHERE category_id = ?1 ORDER BY created_at ASC")
			.map_err(|e| DomainError::Internal(format!("Failed to prepare query: {}", e)))?;
		let rows = stmt
			.query_map(rusqlite::params![category_id], row_to_product)
			.map_err(|e| DomainError::Internal(format!("Failed to query products: {}", e)))?;
		rows.collect::<Result<Vec<_>, _>>()
			.map_err(|e| DomainError::Internal(format!("Failed to collect products: {}", e)))
	}

	fn search_products(&self, query: &str) -> Result<Vec<Product>, DomainError> {
		let conn = get_conn(&self.pool).map_err(|e| DomainError::Internal(e.to_string()))?;
		let pattern = format!("%{}%", query);
		let mut stmt = conn
			.prepare("SELECT * FROM products WHERE name LIKE ?1 ORDER BY created_at ASC")
			.map_err(|e| DomainError::Internal(format!("Failed to prepare query: {}", e)))?;
		let rows = stmt
			.query_map(rusqlite::params![pattern], row_to_product)
			.map_err(|e| DomainError::Internal(format!("Failed to search products: {}", e)))?;
		rows.collect::<Result<Vec<_>, _>>()
			.map_err(|e| DomainError::Internal(format!("Failed to collect products: {}", e)))
	}

	fn list_all_products(&self) -> Result<Vec<Product>, DomainError> {
		let conn = get_conn(&self.pool).map_err(|e| DomainError::Internal(e.to_string()))?;
		let mut stmt = conn
			.prepare("SELECT * FROM products ORDER BY created_at ASC")
			.map_err(|e| DomainError::Internal(format!("Failed to prepare query: {}", e)))?;
		let rows = stmt
			.query_map([], row_to_product)
			.map_err(|e| DomainError::Internal(format!("Failed to query products: {}", e)))?;
		rows.collect::<Result<Vec<_>, _>>()
			.map_err(|e| DomainError::Internal(format!("Failed to collect products: {}", e)))
	}

	// ─── Categories ───────────────────────────────────────────────

	fn create_category(&self, category: &Category) -> Result<(), DomainError> {
		let conn = get_conn(&self.pool).map_err(|e| DomainError::Internal(e.to_string()))?;
		conn.execute(
			"INSERT INTO categories (id, name, parent_id, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
			rusqlite::params![category.id, category.name, category.parent_id, category.created_at, category.updated_at],
		)
		.map_err(|e| DomainError::Internal(format!("Failed to create category: {}", e)))?;
		Ok(())
	}

	fn update_category(&self, category: &Category) -> Result<(), DomainError> {
		let conn = get_conn(&self.pool).map_err(|e| DomainError::Internal(e.to_string()))?;
		let affected = conn
			.execute(
				"UPDATE categories SET name = ?1, parent_id = ?2, updated_at = ?3 WHERE id = ?4",
				rusqlite::params![category.name, category.parent_id, category.updated_at, category.id],
			)
			.map_err(|e| DomainError::Internal(format!("Failed to update category: {}", e)))?;
		if affected == 0 {
			return Err(DomainError::NotFound("Category not found".into()));
		}
		Ok(())
	}

	fn delete_category(&self, id: &str) -> Result<(), DomainError> {
		let conn = get_conn(&self.pool).map_err(|e| DomainError::Internal(e.to_string()))?;
		let affected = conn
			.execute("DELETE FROM categories WHERE id = ?1", rusqlite::params![id])
			.map_err(|e| DomainError::Internal(format!("Failed to delete category: {}", e)))?;
		if affected == 0 {
			return Err(DomainError::NotFound("Category not found".into()));
		}
		Ok(())
	}

	fn find_category_by_id(&self, id: &str) -> Result<Option<Category>, DomainError> {
		let conn = get_conn(&self.pool).map_err(|e| DomainError::Internal(e.to_string()))?;
		let mut stmt = conn
			.prepare("SELECT * FROM categories WHERE id = ?1")
			.map_err(|e| DomainError::Internal(format!("Failed to prepare query: {}", e)))?;
		let mut rows = stmt
			.query_map(rusqlite::params![id], row_to_category)
			.map_err(|e| DomainError::Internal(format!("Failed to query category: {}", e)))?;
		match rows.next() {
			Some(Ok(cat)) => Ok(Some(cat)),
			Some(Err(e)) => Err(DomainError::Internal(format!("Failed to read category row: {}", e))),
			None => Ok(None),
		}
	}

	fn list_all_categories(&self) -> Result<Vec<Category>, DomainError> {
		let conn = get_conn(&self.pool).map_err(|e| DomainError::Internal(e.to_string()))?;
		let mut stmt = conn
			.prepare("SELECT * FROM categories ORDER BY name ASC")
			.map_err(|e| DomainError::Internal(format!("Failed to prepare query: {}", e)))?;
		let rows = stmt
			.query_map([], row_to_category)
			.map_err(|e| DomainError::Internal(format!("Failed to query categories: {}", e)))?;
		rows.collect::<Result<Vec<_>, _>>()
			.map_err(|e| DomainError::Internal(format!("Failed to collect categories: {}", e)))
	}

	fn count_products_by_category(&self, category_id: &str) -> Result<i64, DomainError> {
		let conn = get_conn(&self.pool).map_err(|e| DomainError::Internal(e.to_string()))?;
		conn.query_row(
			"SELECT COUNT(*) FROM products WHERE category_id = ?1",
			rusqlite::params![category_id],
			|row| row.get(0),
		)
		.map_err(|e| DomainError::Internal(format!("Failed to count products: {}", e)))
	}

	fn count_child_categories(&self, parent_id: &str) -> Result<i64, DomainError> {
		let conn = get_conn(&self.pool).map_err(|e| DomainError::Internal(e.to_string()))?;
		conn.query_row(
			"SELECT COUNT(*) FROM categories WHERE parent_id = ?1",
			rusqlite::params![parent_id],
			|row| row.get(0),
		)
		.map_err(|e| DomainError::Internal(format!("Failed to count child categories: {}", e)))
	}

	fn find_child_categories(&self, parent_id: &str) -> Result<Vec<Category>, DomainError> {
		let conn = get_conn(&self.pool).map_err(|e| DomainError::Internal(e.to_string()))?;
		let mut stmt = conn
			.prepare("SELECT * FROM categories WHERE parent_id = ?1 ORDER BY name ASC")
			.map_err(|e| DomainError::Internal(format!("Failed to prepare query: {}", e)))?;
		let rows = stmt
			.query_map(rusqlite::params![parent_id], row_to_category)
			.map_err(|e| DomainError::Internal(format!("Failed to query child categories: {}", e)))?;
		rows.collect::<Result<Vec<_>, _>>()
			.map_err(|e| DomainError::Internal(format!("Failed to collect child categories: {}", e)))
	}
}
