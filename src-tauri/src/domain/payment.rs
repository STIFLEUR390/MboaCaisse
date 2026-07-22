//! Payment domain — Payment, PaymentMethod, and PaymentRepository trait.
//!
//! AD-4: Payment and Wallet are separate.
//!       Payment = encaissement + multi-moyen + validation + écriture ledger.
//!       Payment calls Wallet. Wallet never calls Payment.
//! AD-7: PaymentRepository trait defined here, implemented in db/payments.rs.

use super::DomainError;

/// Supported payment methods.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaymentMethod {
	Wallet,
	Cash,
	MoMo,
	Split,
}

impl PaymentMethod {
	pub fn from_str(s: &str) -> Result<Self, DomainError> {
		match s.to_lowercase().as_str() {
			"wallet" => Ok(Self::Wallet),
			"cash" => Ok(Self::Cash),
			"momo" => Ok(Self::MoMo),
				"split" => Ok(Self::Split),
				_ => Err(DomainError::InvalidValue(format!("Unknown payment method: {}", s))),
		}
	}

	pub fn as_str(&self) -> &'static str {
		match self {
			Self::Wallet => "wallet",
			Self::Cash => "cash",
			Self::MoMo => "momo",
			Self::Split => "split",
		}
	}
}

/// A single payment transaction.
#[derive(Debug, Clone)]
pub struct Payment {
	pub id: String,
	pub order_id: String,
	pub method: PaymentMethod,
	pub amount: i64,
	pub client_id: Option<String>,
	pub reference: Option<String>,
	pub created_at: String,
}

impl Payment {
	/// Validate that payment invariants are satisfied.
	pub fn validate(&self) -> Result<(), DomainError> {
		if self.amount <= 0 {
			return Err(DomainError::Internal("Payment amount must be positive".into()));
		}
		Ok(())
	}
}

/// Repository trait for Payment persistence.
///
/// AD-7: Defined in domain/, implemented in db/.
pub trait PaymentRepository: Send + Sync {
	fn create(&self, payment: &Payment) -> Result<(), DomainError>;
	fn find_by_id(&self, id: &str) -> Result<Option<Payment>, DomainError>;
	fn list_by_order(&self, order_id: &str) -> Result<Vec<Payment>, DomainError>;
	fn list_by_client(&self, client_id: &str) -> Result<Vec<Payment>, DomainError>;
}
