//! Order domain — Order, OrderItem, OrderStatus, and OrderRepository trait.
//!
//! AD-13: Order depends on Catalog (ProductRepository) and Wallet (WalletRepository).
//! AD-7: OrderRepository trait defined here, implemented in db/orders.rs.

use super::DomainError;

/// Status of an order through its lifecycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrderStatus {
	PendingPayment,
	PaidPreparing,
	Ready,
	Delivered,
}

impl OrderStatus {
	/// Check whether a transition from `self` to `next` is valid.
	pub fn can_transition_to(&self, next: &Self) -> bool {
		match (self, next) {
			(Self::PendingPayment, Self::PaidPreparing) => true,
			(Self::PaidPreparing, Self::Ready) => true,
			(Self::Ready, Self::Delivered) => true,
			_ => false,
		}
	}

	pub fn from_str(s: &str) -> Result<Self, DomainError> {
		match s.to_lowercase().as_str() {
			"pending_payment" => Ok(Self::PendingPayment),
			"paid_preparing" => Ok(Self::PaidPreparing),
			"ready" => Ok(Self::Ready),
				"delivered" => Ok(Self::Delivered),
				_ => Err(DomainError::InvalidValue(format!("Unknown order status: {}", s))),
		}
	}

	pub fn as_str(&self) -> &'static str {
		match self {
			Self::PendingPayment => "pending_payment",
			Self::PaidPreparing => "paid_preparing",
			Self::Ready => "ready",
			Self::Delivered => "delivered",
		}
	}
}

/// An item within an order.
#[derive(Debug, Clone)]
pub struct OrderItem {
	pub id: String,
	pub order_id: String,
	pub product_id: String,
	pub quantity: i64,
	pub unit_price: i64,
	pub notes: Option<String>,
	pub created_at: String,
}

/// A customer order with its lifecycle status.
#[derive(Debug, Clone)]
pub struct Order {
	pub id: String,
	pub table_id: Option<String>,
	pub client_id: Option<String>,
	pub status: OrderStatus,
	pub total: i64,
	pub created_at: String,
	pub updated_at: String,
}

impl Order {
	/// Create a new order in PendingPayment status.
	pub fn new(
		id: String,
		table_id: Option<String>,
		client_id: Option<String>,
		created_at: String,
	) -> Self {
		Self {
			id,
			table_id,
			client_id,
			status: OrderStatus::PendingPayment,
			total: 0,
			created_at: created_at.clone(),
			updated_at: created_at,
		}
	}

	/// Attempt to transition this order to a new status.
	/// Returns an error if the transition is invalid.
	pub fn transition_to(&mut self, new_status: OrderStatus) -> Result<(), DomainError> {
		if !self.status.can_transition_to(&new_status) {
			return Err(DomainError::InvalidStatusTransition {
				from: self.status.as_str().to_string(),
				to: new_status.as_str().to_string(),
			});
		}
		self.status = new_status;
		Ok(())
	}
}

/// Repository trait for Order persistence.
///
/// AD-7: Defined in domain/, implemented in db/.
pub trait OrderRepository: Send + Sync {
	fn create(&self, order: &Order) -> Result<(), DomainError>;
	/// Delete an order by ID. Used for cleanup on partial failure.
	fn delete(&self, id: &str) -> Result<(), DomainError>;
	fn update_status(&self, id: &str, status: &OrderStatus) -> Result<(), DomainError>;
	fn find_by_id(&self, id: &str) -> Result<Option<Order>, DomainError>;
	fn list_by_status(&self, status: &OrderStatus) -> Result<Vec<Order>, DomainError>;
	fn list_all(&self) -> Result<Vec<Order>, DomainError>;

	fn add_item(&self, item: &OrderItem) -> Result<(), DomainError>;
	fn get_items(&self, order_id: &str) -> Result<Vec<OrderItem>, DomainError>;
	fn remove_item(&self, order_id: &str, item_id: &str) -> Result<(), DomainError>;
	/// Recalculate and persist the order total from order_items.
	fn update_total(&self, order_id: &str) -> Result<(), DomainError>;
}
