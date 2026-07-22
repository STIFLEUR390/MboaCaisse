//! Domain layer — business logic, repository traits, domain errors.
//!
//! AD-1: domain/ contains the business behavior.
//!        Aggregate methods take `dyn Repository` as parameter.
//! AD-7: Repository traits are defined here, implemented in db/.
//! AD-8: DomainError enum — never leaks DbError to upper layers.

pub mod user;
pub mod product;
pub mod order;
pub mod payment;
pub mod wallet;
pub mod print_job;
pub mod jwt;
pub mod crypto;

use std::fmt;

/// Errors that can occur in the domain layer.
///
/// These are the only errors that api/ handlers should deal with.
/// AD-8: DomainError is an enum with named cases. No anyhow in domain/.
#[derive(Debug)]
pub enum DomainError {
	/// The authenticated user lacks the required permission.
	Unauthorized,
	/// The requested entity was not found.
	NotFound(String),
	/// A provided string value is not a valid variant of an enum.
	InvalidValue(String),
	/// A product referenced by an operation does not exist.
	ProductNotFound,
	/// A wallet client with this phone already exists.
	DuplicatePhone,
	/// Wallet balance is insufficient for the requested operation.
	InsufficientBalance {
		balance: i64,
		required: i64,
	},
	/// An order status transition is not allowed.
	InvalidStatusTransition {
		from: String,
		to: String,
	},
	/// A catch-all for internal / unexpected domain errors.
	Internal(String),
}

impl fmt::Display for DomainError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::Unauthorized => write!(f, "Unauthorized"),
			Self::NotFound(s) => write!(f, "Not found: {}", s),
			Self::InvalidValue(s) => write!(f, "Invalid value: {}", s),
			Self::ProductNotFound => write!(f, "Product not found"),
			Self::DuplicatePhone => write!(f, "Duplicate phone number"),
			Self::InsufficientBalance { balance, required } => {
				write!(f, "Insufficient balance: {} < {}", balance, required)
			}
			Self::InvalidStatusTransition { from, to } => {
				write!(f, "Invalid status transition: {} → {}", from, to)
			}
			Self::Internal(msg) => write!(f, "Internal error: {}", msg),
		}
	}
}

impl std::error::Error for DomainError {}

/// Helper to create DomainError::Internal from a stringable value.
impl From<String> for DomainError {
	fn from(msg: String) -> Self {
		Self::Internal(msg)
	}
}
