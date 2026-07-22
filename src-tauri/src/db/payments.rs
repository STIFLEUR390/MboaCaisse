//! PaymentRepository implementation using rusqlite.
//!
//! AD-7: Implements trait from domain/payment.rs.
//! AD-16: Uses r2d2 connection pool.

use crate::domain::payment::{Payment, PaymentMethod, PaymentRepository};
use crate::domain::DomainError;

use super::SqlitePool;

pub struct DbPaymentRepository {
	pool: SqlitePool,
}

impl DbPaymentRepository {
	pub fn new(pool: SqlitePool) -> Self {
		Self { pool }
	}
}

impl PaymentRepository for DbPaymentRepository {
	fn create(&self, _payment: &Payment) -> Result<(), DomainError> {
		todo!("Story 3.3")
	}
	fn find_by_id(&self, _id: &str) -> Result<Option<Payment>, DomainError> {
		todo!("Story 3.3")
	}
	fn list_by_order(&self, _order_id: &str) -> Result<Vec<Payment>, DomainError> {
		todo!("Story 3.3")
	}
	fn list_by_client(&self, _client_id: &str) -> Result<Vec<Payment>, DomainError> {
		todo!("Story 3.3")
	}
}
