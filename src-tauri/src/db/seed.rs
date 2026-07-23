//! Idempotent seed data for development and first-start.
//!
//! AD-11: Seed admin account on first startup.
//! AC-6: Admin account created at first startup with generated password.
//! AC-8: Seed catalogue (3 categories, 10 products) on first startup.

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
		// Still try to seed catalogue (idempotent)
		seed_catalogue(conn)?;
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

	// Seed catalogue after admin
	seed_catalogue(conn)?;

	Ok(())
}

/// Seed the catalogue with 3 categories and 10 products (idempotent).
fn seed_catalogue(conn: &mut SqliteConn) -> Result<(), DbError> {
	let cat_count: i64 = conn
		.query_row("SELECT COUNT(*) FROM categories", [], |row| row.get(0))
		.map_err(|e| DbError::Query(format!("Failed to count categories: {}", e)))?;

	if cat_count > 0 {
		info!("Catalogue seed skipped: {} categories already exist", cat_count);
		return Ok(());
	}

	info!("Seeding catalogue with 3 categories and 10 products...");

	let now = chrono_now();

	// Categories
	let boissons_id = uuid_v7();
	let plats_id = uuid_v7();
	let snacks_id = uuid_v7();

	conn.execute(
		"INSERT INTO categories (id, name, parent_id, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
		rusqlite::params![boissons_id, "Boissons", None::<String>, &now, &now],
	).map_err(|e| DbError::Query(format!("Failed to seed category 'Boissons': {}", e)))?;

	conn.execute(
		"INSERT INTO categories (id, name, parent_id, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
		rusqlite::params![plats_id, "Plats", None::<String>, &now, &now],
	).map_err(|e| DbError::Query(format!("Failed to seed category 'Plats': {}", e)))?;

	conn.execute(
		"INSERT INTO categories (id, name, parent_id, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
		rusqlite::params![snacks_id, "Snacks", None::<String>, &now, &now],
	).map_err(|e| DbError::Query(format!("Failed to seed category 'Snacks': {}", e)))?;

	// Products (10)
	let products: Vec<(&str, i64, &str, i64, i64)> = vec![
		("Bière 33", 500, "Boissons", 50, 10),
		("Bière 65", 700, "Boissons", 50, 10),
		("Jus de fruits", 400, "Boissons", 30, 5),
		("Eau minérale", 200, "Boissons", 100, 20),
		("Planteur frites", 1500, "Plats", 20, 5),
		("Poulet braisé", 2000, "Plats", 15, 3),
		("Poisson braisé", 2500, "Plats", 15, 3),
		("Miondo", 500, "Snacks", 40, 10),
		("Beignets (5 pcs)", 300, "Snacks", 50, 10),
		("Brochettes (3 pcs)", 1000, "Snacks", 30, 8),
	];

	let category_ids: std::collections::HashMap<&str, &str> = [
		("Boissons", boissons_id.as_str()),
		("Plats", plats_id.as_str()),
		("Snacks", snacks_id.as_str()),
	]
	.into_iter()
	.collect();

	let mut stmt = conn
		.prepare(
			"INSERT INTO products (id, name, price, category_id, stock, alert_threshold, created_at, updated_at) \
			 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
		)
		.map_err(|e| DbError::Query(format!("Failed to prepare product insert: {}", e)))?;

	for (name, price, cat_name, stock, alert) in &products {
		let cat_id = category_ids.get(cat_name).ok_or_else(|| {
			DbError::Query(format!("Unknown category '{}' for product '{}'", cat_name, name))
		})?;
		let id = uuid_v7();
		stmt.execute(rusqlite::params![id, name, price, cat_id, stock, alert, &now, &now])
			.map_err(|e| DbError::Query(format!("Failed to seed product '{}': {}", name, e)))?;
	}

	info!("Catalogue seeded: 3 categories, {} products", products.len());
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
