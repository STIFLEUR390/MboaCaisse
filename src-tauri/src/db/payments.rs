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

	fn conn(&self) -> Result<impl std::ops::Deref<Target = rusqlite::Connection> + '_, crate::db::DbError> {
		self.pool.get().map_err(|e| crate::db::DbError::Connection(e.to_string()))
	}

	fn row_to_payment(row: &rusqlite::Row) -> rusqlite::Result<Payment> {
		let method_str: String = row.get("method")?;
		let method = PaymentMethod::from_str(&method_str).map_err(|e| {
			rusqlite::Error::ToSqlConversionFailure(Box::new(
				std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
			))
		})?;

		Ok(Payment {
			id: row.get("id")?,
			order_id: row.get("order_id")?,
			method,
			amount: row.get("amount")?,
			client_id: row.get("client_id")?,
			reference: row.get("reference")?,
			created_at: row.get("created_at")?,
		})
	}
}

impl PaymentRepository for DbPaymentRepository {
	fn create(&self, payment: &Payment) -> Result<(), DomainError> {
		payment.validate().map_err(|e| DomainError::Internal(e.to_string()))?;

		let conn = self.conn().map_err(|e| DomainError::Internal(e.to_string()))?;
		conn.execute(
			"INSERT INTO payments (id, order_id, method, amount, client_id, reference, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
			rusqlite::params![
				payment.id,
				payment.order_id,
				payment.method.as_str(),
				payment.amount,
				payment.client_id,
				payment.reference,
				payment.created_at,
			],
		)
		.map_err(|e| DomainError::Internal(format!("Failed to create payment: {}", e)))?;

		Ok(())
	}

	fn find_by_id(&self, id: &str) -> Result<Option<Payment>, DomainError> {
		let conn = self.conn().map_err(|e| DomainError::Internal(e.to_string()))?;
		let mut stmt = conn
			.prepare("SELECT id, order_id, method, amount, client_id, reference, created_at FROM payments WHERE id = ?1")
			.map_err(|e| DomainError::Internal(e.to_string()))?;

		let mut rows = stmt
			.query_map(rusqlite::params![id], Self::row_to_payment)
			.map_err(|e| DomainError::Internal(e.to_string()))?;

		match rows.next() {
			Some(Ok(payment)) => Ok(Some(payment)),
			Some(Err(e)) => Err(DomainError::Internal(e.to_string())),
			None => Ok(None),
		}
	}

	fn list_by_order(&self, order_id: &str) -> Result<Vec<Payment>, DomainError> {
		let conn = self.conn().map_err(|e| DomainError::Internal(e.to_string()))?;
		let mut stmt = conn
			.prepare(
				"SELECT id, order_id, method, amount, client_id, reference, created_at \
				 FROM payments WHERE order_id = ?1 \
				 ORDER BY created_at ASC",
			)
			.map_err(|e| DomainError::Internal(e.to_string()))?;

		let payments = stmt
			.query_map(rusqlite::params![order_id], Self::row_to_payment)
			.map_err(|e| DomainError::Internal(e.to_string()))?
			.collect::<Result<Vec<_>, _>>()
			.map_err(|e| DomainError::Internal(e.to_string()))?;

		Ok(payments)
	}

	fn list_by_client(&self, client_id: &str) -> Result<Vec<Payment>, DomainError> {
		let conn = self.conn().map_err(|e| DomainError::Internal(e.to_string()))?;
		let mut stmt = conn
			.prepare(
				"SELECT id, order_id, method, amount, client_id, reference, created_at \
				 FROM payments WHERE client_id = ?1 \
				 ORDER BY created_at ASC",
			)
			.map_err(|e| DomainError::Internal(e.to_string()))?;

		let payments = stmt
			.query_map(rusqlite::params![client_id], Self::row_to_payment)
			.map_err(|e| DomainError::Internal(e.to_string()))?
			.collect::<Result<Vec<_>, _>>()
			.map_err(|e| DomainError::Internal(e.to_string()))?;

		Ok(payments)
	}
}
