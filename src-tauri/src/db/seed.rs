//! Idempotent seed data for development and first-start.
//!
//! AD-11: Seed admin account on first startup.
//! AC-6: Admin account created at first startup with generated password.

use tracing::info;

use super::{DbError, SqliteConn};
use crate::domain::crypto;
use crate::domain::user::Role;

/// Run the seed if the database is empty (no users exist).
pub fn run(conn: &mut SqliteConn) -> Result<(), DbError> {
	let user_count: i64 = conn
		.query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))
		.map_err(|e| DbError::Query(format!("Failed to count users: {}", e)))?;

	if user_count > 0 {
		info!("Seed skipped: {} users already exist", user_count);
		return Ok(());
	}

	info!("Seeding database with initial data...");

	let admin_password = generate_admin_password();
	let password_hash = crypto::hash_password(&admin_password)
		.map_err(|e| DbError::Query(format!("Failed to hash admin password: {}", e)))?;

	let now = chrono_now();
	let admin_id = uuid_v7();

	conn.execute(
		"INSERT INTO users (id, email, password_hash, name, role, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
		rusqlite::params![admin_id, "admin@mboacaisse.local", password_hash, "Admin", Role::Admin.as_str(), now, now],
	)
	.map_err(|e| DbError::Query(format!("Failed to create admin user: {}", e)))?;

	info!("──────────────────────────────────────────────────────");
	info!("  🚀 MboaCaisse — First startup!");
	info!("  📧 Admin email:    admin@mboacaisse.local");
	info!("  🔑 Admin password: {}", admin_password);
	info!("  ⚠️  Save this password — it will NOT be shown again!");
	info!("──────────────────────────────────────────────────────");

	Ok(())
}

fn generate_admin_password() -> String {
	const CHARSET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZabcdefghjkmnpqrstuvwxyz23456789";
	(0..12)
		.map(|_| {
			let idx = rand::random_range(0..CHARSET.len());
			CHARSET[idx] as char
		})
		.collect()
}

fn uuid_v7() -> String {
	use uuid::Uuid;
	Uuid::now_v7().to_string()
}

fn chrono_now() -> String {
	use chrono::Utc;
	Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
}
