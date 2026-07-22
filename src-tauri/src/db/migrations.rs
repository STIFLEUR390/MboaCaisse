//! Database migrations via refinery.
//!
//! AD-15: refinery::Runner runs embedded SQL migrations at startup.
//!        On failure → log error + exit. No server startup without a valid schema.

use refinery::embed_migrations;
use tracing::{error, info};

use super::{DbError, SqliteConn};

// Embed the migrations/ directory at compile time.
// Each file must follow the naming convention: V{version}__{description}.sql
embed_migrations!("migrations");

/// Run all pending migrations against the given connection.
///
/// Should be called once during application startup, before the server starts
/// listening. If migrations fail, the process should exit immediately.
///
/// AD-15: Embedded SQL, table `_schema_version` auto-managed by refinery.
	pub fn run(conn: &mut SqliteConn) -> Result<(), DbError> {
		info!("Running database migrations...");

		let report = migrations_runner()
			.run(&mut *conn)
			.map_err(|e| DbError::Migration(format!("Migration failed: {}", e)))?;

	for migration in report.applied_migrations() {
		info!("Applied migration: V{} — {}", migration.version(), migration.name());
	}

	info!("All migrations applied successfully ({} total)", report.applied_migrations().len());
	Ok(())
}
