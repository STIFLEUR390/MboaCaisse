//! Database migrations via refinery.
//!
//! AD-15: refinery::Runner runs embedded SQL migrations at startup.
//!        On failure → log error + exit. No server startup without a valid schema.

use tracing::info;

use super::{DbError, SqliteConn};

// Embed the migrations/ directory at compile time.
// In refinery 0.9, this generates a module `migrations` with a `runner()` function.
// The generated Runner::run<C>() takes `&mut C` where C: Migrate.
// Migrate is implemented for rusqlite::Connection (aliased as RqlConnection in refinery-core).
refinery::embed_migrations!("migrations");

/// Run all pending migrations against the given connection.
///
/// Should be called once during application startup, before the server starts
/// listening. If migrations fail, the process should exit immediately.
///
/// AD-15: Embedded SQL, table `_schema_version` auto-managed by refinery.
pub fn run(conn: &mut SqliteConn) -> Result<(), DbError> {
	info!("Running database migrations...");

	// Dereference the PooledConnection to get a &mut rusqlite::Connection,
	// which implements refinery::Migrate.
	let raw_conn: &mut rusqlite::Connection = &mut *conn;

	let report = migrations::runner()
		.run(raw_conn)
		.map_err(|e| DbError::Migration(format!("Migration failed: {}", e)))?;

	for migration in report.applied_migrations() {
		info!("Applied migration: V{} — {}", migration.version(), migration.name());
	}

	info!("All migrations applied successfully ({} total)", report.applied_migrations().len());
	Ok(())
}
