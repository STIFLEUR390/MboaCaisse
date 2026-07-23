//! WalletRepository implementation using rusqlite.
//!
//! AD-2: wallet_ledger is INSERT-only. Balance calculated as SELECT SUM in transaction.
//! AD-7: Implements trait from domain/wallet.rs.
//! AD-16: Uses r2d2 connection pool.

use crate::domain::wallet::{LedgerEntryType, WalletClient, WalletLedgerEntry, WalletRepository};
use crate::domain::DomainError;

use super::{DbError, SqlitePool};

pub struct DbWalletRepository {
	pool: SqlitePool,
}

impl DbWalletRepository {
	pub fn new(pool: SqlitePool) -> Self {
		Self { pool }
	}

	fn conn(&self) -> Result<impl std::ops::Deref<Target = rusqlite::Connection> + '_, DbError> {
		self.pool.get().map_err(|e| DbError::Connection(e.to_string()))
	}

	fn row_to_client(row: &rusqlite::Row) -> rusqlite::Result<WalletClient> {
		Ok(WalletClient {
			id: row.get("id")?,
			phone: row.get("phone")?,
			name: row.get("name")?,
			referrer_id: row.get("referrer_id")?,
			created_at: row.get("created_at")?,
			updated_at: row.get("updated_at")?,
		})
	}

	fn row_to_ledger_entry(row: &rusqlite::Row) -> rusqlite::Result<WalletLedgerEntry> {
		let type_str: String = row.get("type")?;
		let entry_type = LedgerEntryType::from_str(&type_str).map_err(|e| {
			rusqlite::Error::ToSqlConversionFailure(Box::new(
				std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
			))
		})?;

		Ok(WalletLedgerEntry {
			id: row.get("id")?,
			client_id: row.get("client_id")?,
			entry_type,
			amount: row.get("amount")?,
			reference: row.get("reference")?,
			description: row.get("description")?,
			created_at: row.get("created_at")?,
		})
	}
}

impl WalletRepository for DbWalletRepository {
	fn register_client(&self, client: &WalletClient) -> Result<(), DomainError> {
		let conn = self.conn().map_err(|e| DomainError::Internal(e.to_string()))?;
		conn.execute(
			"INSERT INTO wallet_clients (id, phone, name, referrer_id, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
			rusqlite::params![
				client.id,
				client.phone,
				client.name,
				client.referrer_id,
				client.created_at,
				client.updated_at,
			],
		)
		.map_err(|e| {
			if e.to_string().contains("UNIQUE") {
				DomainError::DuplicatePhone
			} else {
				DomainError::Internal(format!("Failed to register client: {}", e))
			}
		})?;
		Ok(())
	}

	fn find_client_by_phone(&self, phone: &str) -> Result<Option<WalletClient>, DomainError> {
		let conn = self.conn().map_err(|e| DomainError::Internal(e.to_string()))?;
		let mut stmt = conn
			.prepare("SELECT id, phone, name, referrer_id, created_at, updated_at FROM wallet_clients WHERE phone = ?1")
			.map_err(|e| DomainError::Internal(e.to_string()))?;

		let mut rows = stmt
			.query_map(rusqlite::params![phone], Self::row_to_client)
			.map_err(|e| DomainError::Internal(e.to_string()))?;

		match rows.next() {
			Some(Ok(client)) => Ok(Some(client)),
			Some(Err(e)) => Err(DomainError::Internal(e.to_string())),
			None => Ok(None),
		}
	}

	fn find_client_by_id(&self, id: &str) -> Result<Option<WalletClient>, DomainError> {
		let conn = self.conn().map_err(|e| DomainError::Internal(e.to_string()))?;
		let mut stmt = conn
			.prepare("SELECT id, phone, name, referrer_id, created_at, updated_at FROM wallet_clients WHERE id = ?1")
			.map_err(|e| DomainError::Internal(e.to_string()))?;

		let mut rows = stmt
			.query_map(rusqlite::params![id], Self::row_to_client)
			.map_err(|e| DomainError::Internal(e.to_string()))?;

		match rows.next() {
			Some(Ok(client)) => Ok(Some(client)),
			Some(Err(e)) => Err(DomainError::Internal(e.to_string())),
			None => Ok(None),
		}
	}

	fn list_all_clients(&self) -> Result<Vec<WalletClient>, DomainError> {
		let conn = self.conn().map_err(|e| DomainError::Internal(e.to_string()))?;
		let mut stmt = conn
			.prepare("SELECT id, phone, name, referrer_id, created_at, updated_at FROM wallet_clients ORDER BY created_at ASC")
			.map_err(|e| DomainError::Internal(e.to_string()))?;

		let clients = stmt
			.query_map([], Self::row_to_client)
			.map_err(|e| DomainError::Internal(e.to_string()))?
			.collect::<Result<Vec<_>, _>>()
			.map_err(|e| DomainError::Internal(e.to_string()))?;

		Ok(clients)
	}

	fn append_entry(&self, entry: &WalletLedgerEntry) -> Result<(), DomainError> {
		// Validate entry before writing
		entry.validate()?;

		let conn = self.conn().map_err(|e| DomainError::Internal(e.to_string()))?;

		// AD-2: Single transaction for the whole append operation
		conn.execute("BEGIN IMMEDIATE", [])
			.map_err(|e| DomainError::Internal(format!("Failed to begin transaction: {}", e)))?;

		let result = (|| -> Result<(), DomainError> {
			// Verify client exists
			let exists: bool = conn
				.query_row(
					"SELECT COUNT(*) FROM wallet_clients WHERE id = ?1",
					rusqlite::params![entry.client_id],
					|row| row.get::<_, i64>(0),
				)
				.map_err(|e| DomainError::Internal(format!("Failed to check client: {}", e)))?
				> 0;

			if !exists {
				return Err(DomainError::NotFound(format!(
					"Wallet client not found: {}",
					entry.client_id
				)));
			}

			// INSERT the ledger entry
			conn.execute(
				"INSERT INTO wallet_ledger (id, client_id, type, amount, reference, description, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
				rusqlite::params![
					entry.id,
					entry.client_id,
					entry.entry_type.as_str(),
					entry.amount,
					entry.reference,
					entry.description,
					entry.created_at,
				],
			)
			.map_err(|e| DomainError::Internal(format!("Failed to append ledger entry: {}", e)))?;

			Ok(())
		})();

		match result {
			Ok(()) => {
				conn.execute("COMMIT", [])
					.map_err(|e| DomainError::Internal(format!("Failed to commit: {}", e)))?;
				Ok(())
			}
			Err(e) => {
				conn.execute("ROLLBACK", [])
					.map_err(|rollback_err| {
						tracing::error!("Failed to rollback after error: {}", rollback_err);
					})
					.ok();
				Err(e)
			}
		}
	}

	fn get_balance(&self, client_id: &str) -> Result<i64, DomainError> {
		let conn = self.conn().map_err(|e| DomainError::Internal(e.to_string()))?;
		let balance: i64 = conn
			.query_row(
				"SELECT COALESCE(SUM(amount), 0) FROM wallet_ledger WHERE client_id = ?1",
				rusqlite::params![client_id],
				|row| row.get(0),
			)
			.map_err(|e| DomainError::Internal(format!("Failed to calculate balance: {}", e)))?;

		Ok(balance)
	}

	fn get_ledger(&self, client_id: &str, limit: i64) -> Result<Vec<WalletLedgerEntry>, DomainError> {
		let limit = if limit < 0 { 0 } else { limit };
		let conn = self.conn().map_err(|e| DomainError::Internal(e.to_string()))?;
		let mut stmt = conn
			.prepare(
				"SELECT id, client_id, type, amount, reference, description, created_at \
				 FROM wallet_ledger WHERE client_id = ?1 \
				 ORDER BY created_at DESC LIMIT ?2",
			)
			.map_err(|e| DomainError::Internal(e.to_string()))?;

		let entries = stmt
			.query_map(rusqlite::params![client_id, limit], Self::row_to_ledger_entry)
			.map_err(|e| DomainError::Internal(e.to_string()))?
			.collect::<Result<Vec<_>, _>>()
			.map_err(|e| DomainError::Internal(e.to_string()))?;

		Ok(entries)
	}
}
