//! Idempotent seed data for development and first-start.
//!
//! AD-11: Seed admin account + 10 produits / 3 catégories on first startup.
//!        Seed is idempotent — does not create duplicates on restart.

use tracing::info;

use super::{DbError, SqliteConn};
use crate::domain::user::{Role, User, UserRepository};

/// Run the seed if the database is empty (no users exist).
///
/// Must be called AFTER migrations::run().
/// Idempotent: checks if at least one user exists before seeding.
pub fn run(conn: &mut SqliteConn) -> Result<(), DbError> {
	// Check if users exist — if so, skip seeding.
	let user_count: i64 = conn
		.query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))
		.map_err(|e| DbError::Query(format!("Failed to count users: {}", e)))?;

	if user_count > 0 {
		info!("Seed skipped: {} users already exist", user_count);
		return Ok(());
	}

	info!("Seeding database with initial data...");

	// Placeholder: will be populated by story 1.5 (admin creation).
	// The actual admin seed needs argon2 hashing which comes in story 1.3.
	//
	// For now we just mark the seed location. The full seed is implemented
	// together with the auth system.
	info!("Seed placeholder — admin + demo data will be created in story 1.5 (roles & permissions)");

	Ok(())
}
