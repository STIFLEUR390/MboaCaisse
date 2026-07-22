//! User domain — User entity, Role, Permission, and UserRepository trait.
//!
//! AD-11: 4 rôles with granular Vec<Permission>. Permissions derived from role, not stored in DB.
//! AD-7: UserRepository trait defined here, implemented in db/users.rs.

use super::DomainError;

/// A user of the MboaCaisse system.
#[derive(Debug, Clone)]
pub struct User {
	pub id: String,
	pub email: String,
	pub password_hash: String,
	pub name: String,
	pub role: Role,
	pub created_at: String,
	pub updated_at: String,
}

/// The four roles in the system.
///
/// Each variant maps to a set of permissions via `permissions()`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Role {
	Admin,
	Caissier,
	Vendeur,
	GestionnaireStock,
}

impl Role {
	/// Return all permissions granted to this role.
	///
	/// Permissions are derived from the role variant, not stored in the database.
	/// AD-11: Every user's permissions are computed at runtime from their role.
	pub fn permissions(&self) -> &'static [Permission] {
		match self {
			Self::Admin => &[
				Permission::All,
				Permission::Sell,
				Permission::ViewReports,
				Permission::ManageUsers,
				Permission::ManageMenu,
				Permission::ManageStock,
				Permission::ViewOrders,
				Permission::ManageSettings,
			],
			Self::Caissier => &[
				Permission::Sell,
				Permission::ViewReports,
				Permission::ViewOrders,
			],
			Self::Vendeur => &[
				Permission::ViewOrders,
				Permission::ManageMenu,
			],
			Self::GestionnaireStock => &[
				Permission::ManageStock,
				Permission::ViewReports,
			],
		}
	}

	/// Parse a role from its database string representation.
		pub fn from_str(s: &str) -> Result<Self, DomainError> {
			match s.to_lowercase().as_str() {
				"admin" => Ok(Self::Admin),
				"caissier" => Ok(Self::Caissier),
				"vendeur" => Ok(Self::Vendeur),
				"gestionnaire_stock" | "gestionnairestock" => Ok(Self::GestionnaireStock),
				_ => Err(DomainError::InvalidValue(format!("Unknown role: {}", s))),
			}
		}

	/// Check whether this role has the given permission.
	/// `Permission::All` matches every role that has any permissions.
	pub fn has_permission(&self, perm: &Permission) -> bool {
		self.permissions().iter().any(|p| match (p, perm) {
			(Permission::All, _) => true,
			(_, Permission::All) => false,
			_ => p == perm,
		})
	}

	/// Serialize role to its database string representation.
	pub fn as_str(&self) -> &'static str {
		match self {
			Self::Admin => "admin",
			Self::Caissier => "caissier",
			Self::Vendeur => "vendeur",
			Self::GestionnaireStock => "gestionnaire_stock",
		}
	}
}

/// Granular permissions in the system.
///
/// AD-11: Permissions are checked by middleware guards on API routes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Permission {
	/// Super-admin access — grants everything.
	All,
	/// Can process sales (encaissement).
	Sell,
	/// Can view reports (daily, weekly, monthly).
	ViewReports,
	/// Can manage users (create, update, delete).
	ManageUsers,
	/// Can manage the product catalogue (menu).
	ManageMenu,
	/// Can manage stock levels and alerts.
	ManageStock,
	/// Can view orders (kitchen display, order history).
	ViewOrders,
	/// Can manage system settings (licence, config, backup).
	ManageSettings,
}

/// Repository trait for User persistence.
///
/// AD-7: Defined in domain/, implemented in db/.
pub trait UserRepository: Send + Sync {
	/// Find a user by their email address.
	fn find_by_email(&self, email: &str) -> Result<Option<User>, DomainError>;
	/// Find a user by their UUID.
	fn find_by_id(&self, id: &str) -> Result<Option<User>, DomainError>;
	/// Create a new user.
	fn create(&self, user: &User) -> Result<(), DomainError>;
	/// Update an existing user.
	fn update(&self, user: &User) -> Result<(), DomainError>;
	/// Delete a user by ID.
	fn delete(&self, id: &str) -> Result<(), DomainError>;
	/// List all users.
	fn list_all(&self) -> Result<Vec<User>, DomainError>;
}
