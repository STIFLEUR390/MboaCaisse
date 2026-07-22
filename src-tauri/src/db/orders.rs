//! OrderRepository implementation using rusqlite.
//!
//! AD-7: Implements trait from domain/order.rs.
//! AD-16: Uses r2d2 connection pool.

use crate::domain::order::{Order, OrderItem, OrderRepository, OrderStatus};
use crate::domain::DomainError;

use super::SqlitePool;

pub struct DbOrderRepository {
	pool: SqlitePool,
}

impl DbOrderRepository {
	pub fn new(pool: SqlitePool) -> Self {
		Self { pool }
	}
}

impl OrderRepository for DbOrderRepository {
	fn create(&self, _order: &Order) -> Result<(), DomainError> {
		todo!("Story 3.2")
	}
	fn update_status(&self, _id: &str, _status: &OrderStatus) -> Result<(), DomainError> {
		todo!("Story 3.2")
	}
	fn find_by_id(&self, _id: &str) -> Result<Option<Order>, DomainError> {
		todo!("Story 3.2")
	}
	fn list_by_status(&self, _status: &OrderStatus) -> Result<Vec<Order>, DomainError> {
		todo!("Story 3.2")
	}
	fn list_all(&self) -> Result<Vec<Order>, DomainError> {
		todo!("Story 3.2")
	}
	fn add_item(&self, _item: &OrderItem) -> Result<(), DomainError> {
		todo!("Story 3.2")
	}
	fn get_items(&self, _order_id: &str) -> Result<Vec<OrderItem>, DomainError> {
		todo!("Story 3.2")
	}
	fn remove_item(&self, _item_id: &str) -> Result<(), DomainError> {
		todo!("Story 3.2")
	}
}
