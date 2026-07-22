//! JWT creation, verification, and silent refresh.
//!
//! AD-11: JWT cookie (mboa_session), HS256, 24h expiry, silent refresh if <1h remaining.
//! AD-12: Secret key stored in tauri_plugin_store, generated at first startup.

use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use super::user::User;

/// JWT claims embedded in the `mboa_session` cookie.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
	pub sub: String,
	pub email: String,
	pub role: String,
	pub iat: usize,
	pub exp: usize,
}

impl Claims {
	pub fn should_refresh(&self) -> bool {
		let now = now_epoch();
		(self.exp - now) < 3600
	}
}

/// Create a signed JWT for the given user.
pub fn encode_token(user: &User, secret: &[u8]) -> Result<String, String> {
	let now = now_epoch();
	let claims = Claims {
		sub: user.id.clone(),
		email: user.email.clone(),
		role: user.role.as_str().to_string(),
		iat: now,
		exp: now + 86400,
	};

	encode(&Header::default(), &claims, &EncodingKey::from_secret(secret))
		.map_err(|e| format!("JWT encode failed: {}", e))
}

/// Verify a JWT and return its claims.
pub fn decode_token(token: &str, secret: &[u8]) -> Result<Claims, String> {
	let token_data = decode::<Claims>(token, &DecodingKey::from_secret(secret), &Validation::default())
		.map_err(|e| match e.kind() {
			jsonwebtoken::errors::ErrorKind::ExpiredSignature => "TOKEN_EXPIRED".to_string(),
			jsonwebtoken::errors::ErrorKind::InvalidSignature => "INVALID_TOKEN".to_string(),
			_ => format!("INVALID_TOKEN: {}", e),
		})?;
	Ok(token_data.claims)
}

/// Generate a new JWT for silent refresh when <1h remains.
pub fn refresh_token(token: &str, secret: &[u8]) -> Result<String, String> {
	let claims = decode_token(token, secret)?;
	if !claims.should_refresh() {
		return Err("Token does not need refresh".to_string());
	}
	let now = now_epoch();
	let new_claims = Claims { exp: now + 86400, iat: now, ..claims };
	encode(&Header::default(), &new_claims, &EncodingKey::from_secret(secret))
		.map_err(|e| format!("JWT refresh failed: {}", e))
}

/// Generate a cryptographically secure random secret (32 bytes).
pub fn generate_secret() -> Vec<u8> {
	let mut secret = vec![0u8; 32];
	rand::fill(&mut secret);
	secret
}

fn now_epoch() -> usize {
	SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as usize
}
