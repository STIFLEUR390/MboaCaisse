//! OrderRepository implementation using rusqlite.
//!
//! AD-7: Implements trait from domain/order.rs.
//! AD-16: Uses r2d2 connection pool.

use crate::domain::order::{Order, OrderItem, OrderRepository, OrderStatus};
use crate::domain::DomainError;

use super::{SqlitePool};
use super::get_conn;

pub struct DbOrderRepository {
	pool: SqlitePool,
}

impl DbOrderRepository {
	pub fn new(pool: SqlitePool) -> Self {
		Self { pool }
	}
}

impl OrderRepository for DbOrderRepository {
	fn create(&self, order: &Order) -> Result<(), DomainError> {
		let conn = get_conn(&self.pool).map_err(|e| DomainError::Internal(e.to_string()))?;
		conn.execute(
			"INSERT INTO orders (id, table_id, client_id, status, total, created_at, updated_at) \
			 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
			rusqlite::params![
				order.id,
				order.table_id,
				order.client_id,
				order.status.as_str(),
				order.total,
				order.created_at,
				order.updated_at,
			],
		)
		.map_err(|e| DomainError::Internal(format!("Failed to create order: {}", e)))?;
		Ok(())
	}

	fn delete(&self, id: &str) -> Result<(), DomainError> {
		let conn = get_conn(&self.pool).map_err(|e| DomainError::Internal(e.to_string()))?;
		conn.execute(
			"DELETE FROM orders WHERE id = ?1",
			rusqlite::params![id],
		)
		.map_err(|e| DomainError::Internal(format!("Failed to delete order: {}", e)))?;
		Ok(())
	}

	fn update_status(&self, id: &str, status: &OrderStatus) -> Result<(), DomainError> {
		let conn = get_conn(&self.pool).map_err(|e| DomainError::Internal(e.to_string()))?;
		let now = chrono_now();
		let affected = conn
			.execute(
				"UPDATE orders SET status = ?1, updated_at = ?2 WHERE id = ?3",
				rusqlite::params![status.as_str(), now, id],
			)
			.map_err(|e| DomainError::Internal(format!("Failed to update order status: {}", e)))?;
		if affected == 0 {
			return Err(DomainError::NotFound(format!("Order {} not found", id)));
		}
		Ok(())
	}

	fn find_by_id(&self, id: &str) -> Result<Option<Order>, DomainError> {
		let conn = get_conn(&self.pool).map_err(|e| DomainError::Internal(e.to_string()))?;
		let mut stmt = conn
			.prepare(
				"SELECT id, table_id, client_id, status, total, created_at, updated_at \
				 FROM orders WHERE id = ?1",
			)
			.map_err(|e| DomainError::Internal(e.to_string()))?;

		let mut rows = stmt
			.query_map(rusqlite::params![id], |row| {
				let status_str: String = row.get("status")?;
				Ok(Order {
					id: row.get("id")?,
					table_id: row.get("table_id")?,
					client_id: row.get("client_id")?,
					status: OrderStatus::from_str(&status_str).map_err(|e| {
						rusqlite::Error::ToSqlConversionFailure(Box::new(e))
					})?,
					total: row.get("total")?,
					created_at: row.get("created_at")?,
					updated_at: row.get("updated_at")?,
				})
			})
			.map_err(|e| DomainError::Internal(e.to_string()))?;

		match rows.next() {
			Some(Ok(order)) => Ok(Some(order)),
			Some(Err(e)) => Err(DomainError::Internal(e.to_string()).into()),
			None => Ok(None),
		}
	}

	fn list_by_status(&self, status: &OrderStatus) -> Result<Vec<Order>, DomainError> {
		let conn = get_conn(&self.pool).map_err(|e| DomainError::Internal(e.to_string()))?;
		let mut stmt = conn
			.prepare(
				"SELECT id, table_id, client_id, status, total, created_at, updated_at \
				 FROM orders WHERE status = ?1 ORDER BY created_at DESC",
			)
			.map_err(|e| DomainError::Internal(e.to_string()))?;

		let orders = stmt
			.query_map(rusqlite::params![status.as_str()], map_order_row)
			.map_err(|e| DomainError::Internal(e.to_string()))?
			.collect::<Result<Vec<_>, _>>()
			.map_err(|e| DomainError::Internal(e.to_string()))?;

		Ok(orders)
	}

	fn list_all(&self) -> Result<Vec<Order>, DomainError> {
		let conn = get_conn(&self.pool).map_err(|e| DomainError::Internal(e.to_string()))?;
		let mut stmt = conn
			.prepare(
				"SELECT id, table_id, client_id, status, total, created_at, updated_at \
				 FROM orders ORDER BY created_at DESC",
			)
			.map_err(|e| DomainError::Internal(e.to_string()))?;

		let orders = stmt
			.query_map([], map_order_row)
			.map_err(|e| DomainError::Internal(e.to_string()))?
			.collect::<Result<Vec<_>, _>>()
			.map_err(|e| DomainError::Internal(e.to_string()))?;

		Ok(orders)
	}

	fn add_item(&self, item: &OrderItem) -> Result<(), DomainError> {
		let conn = get_conn(&self.pool).map_err(|e| DomainError::Internal(e.to_string()))?;
		conn.execute(
			"INSERT INTO order_items (id, order_id, product_id, quantity, unit_price, notes, created_at) \
			 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
			rusqlite::params![
				item.id,
				item.order_id,
				item.product_id,
				item.quantity,
				item.unit_price,
				item.notes,
				item.created_at,
			],
		)
		.map_err(|e| DomainError::Internal(format!("Failed to add order item: {}", e)))?;
		Ok(())
	}

	fn get_items(&self, order_id: &str) -> Result<Vec<OrderItem>, DomainError> {
		let conn = get_conn(&self.pool).map_err(|e| DomainError::Internal(e.to_string()))?;
		let mut stmt = conn
			.prepare(
				"SELECT id, order_id, product_id, quantity, unit_price, notes, created_at \
				 FROM order_items WHERE order_id = ?1 ORDER BY created_at ASC",
			)
			.map_err(|e| DomainError::Internal(e.to_string()))?;

		let items = stmt
			.query_map(rusqlite::params![order_id], |row| {
				Ok(OrderItem {
					id: row.get("id")?,
					order_id: row.get("order_id")?,
					product_id: row.get("product_id")?,
					quantity: row.get("quantity")?,
					unit_price: row.get("unit_price")?,
					notes: row.get("notes")?,
					created_at: row.get("created_at")?,
				})
			})
			.map_err(|e| DomainError::Internal(e.to_string()))?
			.collect::<Result<Vec<_>, _>>()
			.map_err(|e| DomainError::Internal(e.to_string()))?;

		Ok(items)
	}

	fn remove_item(&self, order_id: &str, item_id: &str) -> Result<(), DomainError> {
		let conn = get_conn(&self.pool).map_err(|e| DomainError::Internal(e.to_string()))?;
		let affected = conn
			.execute(
				"DELETE FROM order_items WHERE id = ?1 AND order_id = ?2",
				rusqlite::params![item_id],
			)
			.map_err(|e| DomainError::Internal(format!("Failed to remove order item: {}", e)))?;
		if affected == 0 {
			return Err(DomainError::NotFound(format!("Order item {} not found", item_id)));
		}
		Ok(())
	}

	fn update_total(&self, order_id: &str) -> Result<(), DomainError> {
		let conn = get_conn(&self.pool).map_err(|e| DomainError::Internal(e.to_string()))?;
		let now = chrono_now();
		let affected = conn
			.execute(
				"UPDATE orders SET total = COALESCE((SELECT SUM(quantity * unit_price) FROM order_items WHERE order_id = ?1), 0), updated_at = ?2 WHERE id = ?1",
				rusqlite::params![order_id, now],
			)
			.map_err(|e| DomainError::Internal(format!("Failed to update order total: {}", e)))?;
		if affected == 0 {
			return Err(DomainError::NotFound(format!("Order {} not found", order_id)));
		}
		Ok(())
	}
}

/// Helper to map a SQL row to an Order.
fn map_order_row(row: &rusqlite::Row) -> rusqlite::Result<Order> {
	let status_str: String = row.get("status")?;
	Ok(Order {
		id: row.get("id")?,
		table_id: row.get("table_id")?,
		client_id: row.get("client_id")?,
		status: OrderStatus::from_str(&status_str).map_err(|e| {
			rusqlite::Error::ToSqlConversionFailure(Box::new(e))
		})?,
		total: row.get("total")?,
		created_at: row.get("created_at")?,
		updated_at: row.get("updated_at")?,
	})
}

fn chrono_now() -> String {
	use chrono::Utc;
	Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
}
