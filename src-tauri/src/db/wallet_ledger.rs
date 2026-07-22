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
}

impl WalletRepository for DbWalletRepository {
	fn register_client(&self, _client: &WalletClient) -> Result<(), DomainError> {
		todo!("Story 1.5.1")
	}
	fn find_client_by_phone(&self, _phone: &str) -> Result<Option<WalletClient>, DomainError> {
		todo!("Story 1.5.1")
	}
	fn find_client_by_id(&self, _id: &str) -> Result<Option<WalletClient>, DomainError> {
		todo!("Story 1.5.1")
	}
	fn list_all_clients(&self) -> Result<Vec<WalletClient>, DomainError> {
		todo!("Story 1.5.1")
	}
	fn append_entry(&self, _entry: &WalletLedgerEntry) -> Result<(), DomainError> {
		todo!("Story 1.5.1")
	}
	fn get_balance(&self, _client_id: &str) -> Result<i64, DomainError> {
		todo!("Story 1.5.1")
	}
	fn get_ledger(&self, _client_id: &str, _limit: i64) -> Result<Vec<WalletLedgerEntry>, DomainError> {
		todo!("Story 1.5.1")
	}
}
