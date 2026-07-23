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
	pub momo_operator: Option<String>,
	pub parent_payment_id: Option<String>,
	pub created_at: String,
}

impl Payment {
	/// Validate that payment invariants are satisfied.
	pub fn validate(&self) -> Result<(), DomainError> {
		if self.amount <= 0 {
			return Err(DomainError::InvalidValue("Payment amount must be positive".into()));
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

/// An item in a split payment — a single sub-payment with its method and amount.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct SplitPaymentItem {
	pub method: String,
	pub amount: i64,
	#[serde(default)]
	pub client_id: Option<String>,
	#[serde(default)]
	pub momo_operator: Option<String>,
}

/// Validate that the sum of split payments matches the order total and that
/// each sub-payment has the required fields for its method.
///
/// Returns `DomainError::InvalidValue` on any violation.
pub fn validate_split(payments: &[SplitPaymentItem], total: i64) -> Result<(), DomainError> {
	let sum: i64 = payments.iter().map(|p| p.amount).sum();
	if sum != total {
		return Err(DomainError::SplitTotalMismatch { sum, expected: total });
	}

	for (i, p) in payments.iter().enumerate() {
		if p.amount <= 0 {
			return Err(DomainError::InvalidValue(format!(
				"Split payment item {} has non-positive amount: {}",
				i, p.amount
			)));
		}

		match p.method.to_lowercase().as_str() {
			"wallet" => {
				if p.client_id.is_none() || p.client_id.as_ref().map_or(true, |c| c.is_empty()) {
					return Err(DomainError::InvalidValue(format!(
						"Split payment item {} (wallet) requires client_id",
						i
					)));
				}
			}
			"momo" => {
				let op = p.momo_operator.as_deref().unwrap_or("");
				if op.is_empty() {
					return Err(DomainError::InvalidValue(format!(
						"Split payment item {} (MoMo) requires momo_operator",
						i
					)));
				}
				if op != "orange" && op != "mtn" {
					return Err(DomainError::InvalidValue(format!(
						"Split payment item {} has invalid momo_operator: '{}' (must be 'orange' or 'mtn')",
						i, op
					)));
				}
			}
			"cash" => {
				// No extra fields needed
			}
			_ => {
				return Err(DomainError::InvalidValue(format!(
					"Split payment item {} has unknown method: '{}'",
					i, p.method
				)));
			}
		}
	}

	Ok(())
}
