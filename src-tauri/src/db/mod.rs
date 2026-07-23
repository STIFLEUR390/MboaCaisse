//! Database layer — connection pool, DbError, repository implementations.
//!
//! AD-8: DbError never leaks out of this layer. It is wrapped into DomainError by callers.
//! AD-16: r2d2 connection pool (5 max, 1 min).
//! AD-15: Migrations are run via refinery before the server starts listening.

pub mod migrations;
pub mod seed;
pub mod users;
pub mod products;
pub mod orders;
pub mod payments;
pub mod wallet_ledger;
pub mod migration_wallet;

use std::fmt;

/// Errors originating from the database layer.
///
/// These errors are internal to db/ and must never bubble up to domain/ or api/.
/// AD-8: Each caller is responsible for mapping DbError → DomainError.
#[derive(Debug)]
pub enum DbError {
	/// A connection-level failure (pool init, connection acquisition).
	Connection(String),
	/// An SQL query or execution failure.
	Query(String),
	/// A migration failure.
	Migration(String),
	/// The requested entity was not found.
	NotFound(String),
}

impl fmt::Display for DbError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Connection(msg) => write!(f, "DB connection error: {}", msg),
			Self::Query(msg) => write!(f, "DB query error: {}", msg),
			Self::Migration(msg) => write!(f, "DB migration error: {}", msg),
			Self::NotFound(s) => write!(f, "DB not found: {}", s),
		}
	}
}

impl std::error::Error for DbError {}

impl From<r2d2::Error> for DbError {
	fn from(e: r2d2::Error) -> Self {
		DbError::Connection(e.to_string())
	}
}

impl From<rusqlite::Error> for DbError {
	fn from(e: rusqlite::Error) -> Self {
		match e {
			rusqlite::Error::QueryReturnedNoRows => DbError::NotFound("query returned no rows".into()),
			other => DbError::Query(other.to_string()),
		}
	}
}

/// Type alias for the r2d2 connection manager.
pub type SqlitePool = r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>;
/// Type alias for a pooled connection.
pub type SqliteConn = r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager>;

/// Default pool size.
const POOL_MAX_SIZE: u32 = 5;
const POOL_MIN_IDLE: u32 = 1;

/// Initialise the r2d2 connection pool for the SQLite database at `db_path`.
///
/// AD-16: Uses r2d2 + r2d2_sqlite with bundled SQLite.
/// Panics if the pool cannot be created (fails fast — no DB = no app).
pub fn init_pool(db_path: &str) -> Result<SqlitePool, DbError> {
	let manager = r2d2_sqlite::SqliteConnectionManager::file(db_path);
	let pool = r2d2::Pool::builder()
		.max_size(POOL_MAX_SIZE)
		.min_idle(Some(POOL_MIN_IDLE))
		.build(manager)
		.map_err(|e| DbError::Connection(format!("Failed to create pool: {}", e)))?;

	// Verify the pool works by acquiring a connection immediately.
	let _conn = pool
		.get()
		.map_err(|e| DbError::Connection(format!("Failed to acquire initial connection: {}", e)))?;

	Ok(pool)
}

/// Helper: acquire a connection from the pool.
pub fn get_conn(pool: &SqlitePool) -> Result<SqliteConn, DbError> {
	pool.get().map_err(|e| DbError::Connection(e.to_string()))
}
