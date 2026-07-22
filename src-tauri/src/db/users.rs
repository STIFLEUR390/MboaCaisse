//! UserRepository implementation using rusqlite.
//!
//! AD-7: Implements the trait defined in domain/user.rs.
//! AD-16: Uses r2d2 connection pool for all database access.

use crate::domain::user::{Role, User, UserRepository};
use crate::domain::DomainError;

use super::{DbError, SqlitePool};

/// Implementation of UserRepository backed by a SQLite pool.
pub struct DbUserRepository {
	pool: SqlitePool,
}

impl DbUserRepository {
	pub fn new(pool: SqlitePool) -> Self {
		Self { pool }
	}

	fn conn(&self) -> Result<impl std::ops::Deref<Target = rusqlite::Connection> + '_, DbError> {
		self.pool.get().map_err(|e| DbError::Connection(e.to_string()))
	}

	fn row_to_user(row: &rusqlite::Row) -> rusqlite::Result<User> {
		let role_str: String = row.get("role")?;
		let role = Role::from_str(&role_str).map_err(|e| {
			rusqlite::Error::ToSqlConversionFailure(Box::new(
				std::io::Error::new(
					std::io::ErrorKind::InvalidData,
					format!("Invalid role '{}' in database: {}", role_str, e),
				)
			))
		})?;

		Ok(User {
			id: row.get("id")?,
			email: row.get("email")?,
			password_hash: row.get("password_hash")?,
			name: row.get("name")?,
			role,
			created_at: row.get("created_at")?,
			updated_at: row.get("updated_at")?,
		})
	}
}

impl UserRepository for DbUserRepository {
	fn find_by_email(&self, email: &str) -> Result<Option<User>, DomainError> {
		let conn = self.conn().map_err(|e| DomainError::Internal(e.to_string()))?;
		let mut stmt = conn
			.prepare("SELECT id, email, password_hash, name, role, created_at, updated_at FROM users WHERE email = ?1")
			.map_err(|e| DomainError::Internal(e.to_string()))?;

		let mut rows = stmt
			.query_map(rusqlite::params![email], Self::row_to_user)
			.map_err(|e| DomainError::Internal(e.to_string()))?;

		match rows.next() {
			Some(Ok(user)) => Ok(Some(user)),
			Some(Err(e)) => Err(DomainError::Internal(e.to_string())),
			None => Ok(None),
		}
	}

	fn find_by_id(&self, id: &str) -> Result<Option<User>, DomainError> {
		let conn = self.conn().map_err(|e| DomainError::Internal(e.to_string()))?;
		let mut stmt = conn
			.prepare("SELECT id, email, password_hash, name, role, created_at, updated_at FROM users WHERE id = ?1")
			.map_err(|e| DomainError::Internal(e.to_string()))?;

		let mut rows = stmt
			.query_map(rusqlite::params![id], Self::row_to_user)
			.map_err(|e| DomainError::Internal(e.to_string()))?;

		match rows.next() {
			Some(Ok(user)) => Ok(Some(user)),
			Some(Err(e)) => Err(DomainError::Internal(e.to_string())),
			None => Ok(None),
		}
	}

	fn create(&self, user: &User) -> Result<(), DomainError> {
		let conn = self.conn().map_err(|e| DomainError::Internal(e.to_string()))?;
		conn.execute(
			"INSERT INTO users (id, email, password_hash, name, role, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
			rusqlite::params![
				user.id,
				user.email,
				user.password_hash,
				user.name,
				user.role.as_str(),
				user.created_at,
				user.updated_at,
			],
		)
		.map_err(|e| DomainError::Internal(format!("Failed to create user: {}", e)))?;
		Ok(())
	}

	fn update(&self, user: &User) -> Result<(), DomainError> {
		let conn = self.conn().map_err(|e| DomainError::Internal(e.to_string()))?;
		let rows = conn.execute(
			"UPDATE users SET email = ?1, password_hash = ?2, name = ?3, role = ?4, updated_at = ?5 WHERE id = ?6",
			rusqlite::params![
				user.email,
				user.password_hash,
				user.name,
				user.role.as_str(),
				user.updated_at,
				user.id,
			],
		)
		.map_err(|e| DomainError::Internal(format!("Failed to update user: {}", e)))?;
		if rows == 0 {
			return Err(DomainError::NotFound(format!("User not found: {}", user.id)));
		}
		Ok(())
	}

	fn delete(&self, id: &str) -> Result<(), DomainError> {
		let conn = self.conn().map_err(|e| DomainError::Internal(e.to_string()))?;
		let rows = conn.execute("DELETE FROM users WHERE id = ?1", rusqlite::params![id])
			.map_err(|e| DomainError::Internal(format!("Failed to delete user: {}", e)))?;
		if rows == 0 {
			return Err(DomainError::NotFound(format!("User not found: {}", id)));
		}
		Ok(())
	}

	fn list_all(&self) -> Result<Vec<User>, DomainError> {
		let conn = self.conn().map_err(|e| DomainError::Internal(e.to_string()))?;
		let mut stmt = conn
			.prepare("SELECT id, email, password_hash, name, role, created_at, updated_at FROM users ORDER BY created_at ASC")
			.map_err(|e| DomainError::Internal(e.to_string()))?;

		let users = stmt
			.query_map([], Self::row_to_user)
			.map_err(|e| DomainError::Internal(e.to_string()))?
			.collect::<Result<Vec<_>, _>>()
			.map_err(|e| DomainError::Internal(e.to_string()))?;

		Ok(users)
	}
}
