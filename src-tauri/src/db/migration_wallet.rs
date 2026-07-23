//! Wallet migration — replay paid orders into wallet_ledger.
//!
//! AD-2: Creates INSERT-only entries with type='migration'.
//! Idempotent: checks existing migration entries before replaying.
//! Safe to run before orders table exists (pre-Epic 3) — becomes a no-op.

use std::collections::HashSet;
use tracing::info;

use super::{DbError, SqliteConn};

/// Run the wallet migration to replay paid orders into the ledger.
///
/// This function is safe to call at every startup. It checks:
/// 1. Whether the `orders` table exists (may not exist pre-Epic 3)
/// 2. Whether migration has already been run (idempotent)
/// 3. Whether there are paid orders without corresponding migration entries
///
/// If any of these conditions fail, it becomes a no-op.
pub fn run(conn: &mut SqliteConn) -> Result<(), DbError> {
	// Check if orders table exists
	let table_exists: bool = conn
		.query_row(
			"SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='orders'",
			[],
			|row| row.get::<_, i64>(0),
		)
		.map_err(|e| DbError::Query(format!("Failed to check orders table: {}", e)))?
		> 0;

	if !table_exists {
		info!("Wallet migration skipped: orders table does not exist yet (pre-Epic 3)");
		return Ok(());
	}

	// Check if migration was already run (look for any migration entries)
	let has_migration: bool = conn
		.query_row(
			"SELECT COUNT(*) FROM wallet_ledger WHERE type = 'migration'",
			[],
			|row| row.get::<_, i64>(0),
		)
		.map_err(|e| DbError::Query(format!("Failed to check migration status: {}", e)))?
		> 0;

	if has_migration {
		info!("Wallet migration already completed — skipping");
		return Ok(());
	}

	// Check if wallet_ledger exists
	let ledger_exists: bool = conn
		.query_row(
			"SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='wallet_ledger'",
			[],
			|row| row.get::<_, i64>(0),
		)
		.map_err(|e| DbError::Query(format!("Failed to check wallet_ledger table: {}", e)))?
		> 0;

	if !ledger_exists {
		info!("Wallet migration skipped: wallet_ledger table does not exist");
		return Ok(());
	}

	// Find paid orders that need migration
	// Orders with status = 'paid_preparing' or 'ready' or 'delivered' are considered paid.
	// We look for orders that have a client_id set and are in a paid state.
	let raw_conn: &mut rusqlite::Connection = &mut *conn;

	// Start transaction for atomicity
	let tx = raw_conn.transaction()
		.map_err(|e| DbError::Query(format!("Failed to begin transaction: {}", e)))?;

	// Find paid orders with client references
	let mut stmt = tx.prepare(
		"SELECT id, client_id, total FROM orders \
		 WHERE status IN ('paid_preparing', 'ready', 'delivered') \
		 AND client_id IS NOT NULL \
		 ORDER BY created_at ASC"
	).map_err(|e| DbError::Query(format!("Failed to prepare order query: {}", e)))?;

	let orders: Vec<(String, String, i64)> = stmt
		.query_map([], |row| {
			Ok((
				row.get::<_, String>("id")?,
				row.get::<_, String>("client_id")?,
				row.get::<_, i64>("total")?,
			))
		})
		.map_err(|e| DbError::Query(format!("Failed to query orders: {}", e)))?
		.collect::<Result<Vec<_>, _>>()
		.map_err(|e| DbError::Query(format!("Failed to collect orders: {}", e)))?;

	drop(stmt);

	if orders.is_empty() {
		info!("Wallet migration: no paid orders found to migrate");
		return Ok(());
	}

	// Check existing migration entries
	let existing_orders: std::collections::HashSet<String> = tx
		.prepare(
			"SELECT reference FROM wallet_ledger WHERE type = 'migration' AND reference IS NOT NULL"
		)
		.map_err(|e| DbError::Query(format!("Failed to query existing migrations: {}", e)))?
		.query_map([], |row| row.get::<_, String>(0))
		.map_err(|e| DbError::Query(format!("Failed to collect existing migrations: {}", e)))?
		.collect::<Result<std::collections::HashSet<_>, _>>()
		.map_err(|e| DbError::Query(format!("Failed to collect existing migrations: {}", e)))?;

	let mut migrated = 0u64;
	for (order_id, client_id, total) in &orders {
		if existing_orders.contains(order_id) {
			continue;
		}

		// Ensure wallet client exists
		let client_exists: bool = tx
			.query_row(
				"SELECT COUNT(*) FROM wallet_clients WHERE id = ?1",
				rusqlite::params![client_id],
				|row| row.get::<_, i64>(0),
			)
			.map_err(|e| DbError::Query(format!("Failed to check client: {}", e)))?
			> 0;

		if !client_exists {
			// Create minimal wallet client entry for order migration
			let now = chrono_now();
			tx.execute(
				"INSERT OR IGNORE INTO wallet_clients (id, phone, name, referrer_id, created_at, updated_at) \
				 VALUES (?1, ?2, '', NULL, ?3, ?3)",
				rusqlite::params![client_id, format!("CLI-{}", &client_id[..8]), now],
			).map_err(|e| DbError::Query(format!("Failed to create wallet client for migration: {}", e)))?;
		}

		// Insert migration entry
		let now = chrono_now();
		let entry_id = uuid_v7();
		tx.execute(
			"INSERT INTO wallet_ledger (id, client_id, type, amount, reference, description, created_at) \
			 VALUES (?1, ?2, 'migration', ?3, ?4, 'Migration commande anterieure', ?5)",
			rusqlite::params![entry_id, client_id, total, order_id, now],
		).map_err(|e| DbError::Query(format!("Failed to insert migration entry: {}", e)))?;

		migrated += 1;
	}

	tx.commit()
		.map_err(|e| DbError::Query(format!("Failed to commit migration: {}", e)))?;

	info!("Wallet migration complete: {} orders migrated to wallet_ledger", migrated);

	Ok(())
}

fn uuid_v7() -> String {
	use uuid::Uuid;
	Uuid::now_v7().to_string()
}

fn chrono_now() -> String {
	use chrono::Utc;
	Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
}
