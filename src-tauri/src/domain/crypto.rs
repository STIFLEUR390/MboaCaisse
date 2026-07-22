//! Password hashing and verification using argon2.
//!
//! AD-11: Argon2 hashing for user passwords.

use argon2::{
	password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
	Argon2,
};

use super::DomainError;

/// Hash a plaintext password using argon2.
pub fn hash_password(password: &str) -> Result<String, DomainError> {
	let salt = SaltString::generate(&mut OsRng);
	let argon2 = Argon2::default();

	let hash = argon2
		.hash_password(password.as_bytes(), &salt)
		.map_err(|e| DomainError::Internal(format!("Password hashing failed: {}", e)))?;

	Ok(hash.to_string())
}

/// Verify a plaintext password against an argon2 hash.
pub fn verify_password(password: &str, hash: &str) -> Result<bool, DomainError> {
	let parsed_hash = PasswordHash::new(hash)
		.map_err(|e| DomainError::Internal(format!("Invalid password hash format: {}", e)))?;

	Ok(Argon2::default()
		.verify_password(password.as_bytes(), &parsed_hash)
		.is_ok())
}
