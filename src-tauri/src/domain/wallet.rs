//! Wallet domain — WalletClient, WalletLedgerEntry, and WalletRepository trait.
//!
//! AD-2: wallet_ledger is append-only (INSERT-only). Every financial transaction is a row.
//! AD-4: Wallet is an island — no outgoing dependencies. Payment calls Wallet.
//! AD-7: WalletRepository trait defined here, implemented in db/wallet_ledger.rs.

use super::DomainError;

/// A wallet client, identified by phone number.
#[derive(Debug, Clone)]
pub struct WalletClient {
	pub id: String,
	pub phone: String,
	pub name: Option<String>,
	pub referrer_id: Option<String>,
	pub created_at: String,
	pub updated_at: String,
}

/// Source types for wallet ledger entries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LedgerEntryType {
	Payment,
	Credit,
	Cashback,
	ReferralBonus,
	Migration,
}

impl LedgerEntryType {
	pub fn from_str(s: &str) -> Result<Self, DomainError> {
		match s.to_lowercase().as_str() {
			"payment" => Ok(Self::Payment),
			"credit" => Ok(Self::Credit),
			"cashback" => Ok(Self::Cashback),
			"referral_bonus" | "referralbonus" => Ok(Self::ReferralBonus),
			"migration" => Ok(Self::Migration),
			_ => Err(DomainError::InvalidValue(format!("Unknown ledger entry type: {}", s))),
		}
	}

	pub fn as_str(&self) -> &'static str {
		match self {
			Self::Payment => "payment",
			Self::Credit => "credit",
			Self::Cashback => "cashback",
			Self::ReferralBonus => "referral_bonus",
			Self::Migration => "migration",
		}
	}
}

/// A single entry in the append-only wallet ledger.
#[derive(Debug, Clone)]
pub struct WalletLedgerEntry {
	pub id: String,
	pub client_id: String,
	pub entry_type: LedgerEntryType,
	pub amount: i64,
	pub reference: Option<String>,
	pub description: Option<String>,
	pub created_at: String,
}

impl WalletLedgerEntry {
	/// Validate that wallet ledger entry invariants are satisfied.
	pub fn validate(&self) -> Result<(), DomainError> {
		if self.amount == 0 {
			return Err(DomainError::Internal("Wallet entry amount must not be zero".into()));
		}
		Ok(())
	}
}

/// Repository trait for Wallet persistence.
///
/// AD-2: ledger is INSERT-only (no UPDATE/DELETE).
///       Balance is calculated as SELECT SUM in a single transaction.
/// AD-7: Defined in domain/, implemented in db/.
pub trait WalletRepository: Send + Sync {
	fn register_client(&self, client: &WalletClient) -> Result<(), DomainError>;
	fn find_client_by_phone(&self, phone: &str) -> Result<Option<WalletClient>, DomainError>;
	fn find_client_by_id(&self, id: &str) -> Result<Option<WalletClient>, DomainError>;
	fn list_all_clients(&self) -> Result<Vec<WalletClient>, DomainError>;

		/// Append a new entry to the ledger. This is a BEGIN → INSERT → COMMIT
		/// operation inside a single SQL transaction (AD-2).
		/// Balance is calculated separately by `get_balance` using SELECT SUM.
		fn append_entry(&self, entry: &WalletLedgerEntry) -> Result<(), DomainError>;

	/// Calculate the balance for a client as SUM(amount) over all ledger entries.
	fn get_balance(&self, client_id: &str) -> Result<i64, DomainError>;

	/// Return the last N ledger entries for a client, newest first.
	fn get_ledger(&self, client_id: &str, limit: i64) -> Result<Vec<WalletLedgerEntry>, DomainError>;
}
