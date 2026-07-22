# Blind Hunter — Review Prompt

Invoke the `bmad-review-adversarial-general` skill on this diff:

## Diff à reviewer

### Fichiers modifiés (git diff HEAD)

#### `src-tauri/Cargo.toml`
```toml
# Async runtime
tokio = { version = "1", features = ["full"] }

# HTTP server (Axum embarqué)
axum = "0.8"
tower-http = { version = "0.6", features = ["cors", "fs"] }

# Database
rusqlite = { version = "0.32", features = ["bundled"] }
r2d2 = "0.8"
r2d2_sqlite = { version = "0.25", features = ["bundled"] }
refinery = { version = "0.9", features = ["rusqlite-bundled"] }

# Auth (future story)
argon2 = "0.5"

# Network discovery (future story)
mdns-sd = "0.12"

# Telemetry
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }

# ID generation
uuid = { version = "1", features = ["v7", "serde"] }

# Date/time
chrono = { version = "0.4", features = ["serde"] }

# Error handling
thiserror = "2"
```

#### `src-tauri/src/lib.rs`
```rust
//! MboaCaisse — Tauri application entry point.
//!
//! Initialises subsystems in order:
//!   1. Tracing subscriber (logging)
//!   2. Database pool + migrations
//!   3. Tauri plugins (shell, notification, os, fs, store)
//!   4. Tray icon (desktop only)
//!   5. Axum server (future story — server.rs)
//!
//! AD-9: on_event(ExitRequested) → shutdown_tx → Axum graceful → backup DB.
//!       Timeout 5s. Better to lose a backup than corrupt the DB.

// Module declarations — flat structure per AD-3.
mod api;
mod db;
mod domain;

#[cfg(desktop)]
use tauri::{
	menu::{Menu, MenuItem},
	tray::TrayIconBuilder,
	Manager,
};

use std::sync::Arc;

use db::{init_pool, migrations, SqlitePool};

/// Shared application state accessible via Tauri's managed state.
pub struct AppState {
	pub db_pool: SqlitePool,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
	// 1. Initialise tracing subscriber.
	tracing_subscriber::fmt()
		.with_env_filter(
			tracing_subscriber::EnvFilter::try_from_default_env()
				.unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
		)
		.json()
		.init();

	tracing::info!("Starting MboaCaisse...");

	// 2. Initialise database pool and run migrations.
	let db_path = "mboacaisse.db";
	let pool = init_pool(db_path).expect("Failed to initialise database pool");
	{
		let mut conn = pool.get().expect("Failed to acquire connection for migrations");
		migrations::run(&mut conn).expect("Database migrations failed");
		db::seed::run(&mut conn).expect("Database seed failed");
	}
	let pool = Arc::new(pool);
	let app_state = AppState { db_pool: (*pool).clone() };

	tracing::info!("Database initialised successfully");

	// 3. Build Tauri application.
	tauri::Builder::default()
		.setup(|app| {
			app.manage(app_state);

			#[cfg(desktop)]
			{
				let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
				let menu = Menu::with_items(app, &[&quit_i])?;

				let _tray = TrayIconBuilder::new()
					.menu(&menu)
					.show_menu_on_left_click(true)
					.icon(app.default_window_icon().unwrap().clone())
					.on_menu_event(|app_handle, event| match event.id.as_ref() {
						"quit" => {
							tracing::info!("Quit requested via tray menu");
							app_handle.exit(0);
						}
						other => {
							tracing::warn!("Unhandled tray menu item: {}", other);
						}
					})
					.build(app)?;

				tracing::info!("Tray icon created");
			}

			Ok(())
		})
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}
```

### Nouveaux fichiers

#### `src-tauri/src/api/mod.rs`
```rust
pub mod auth;
pub mod health;
pub mod kitchen;
pub mod orders;
pub mod payments;
pub mod products;
pub mod reports;
pub mod settings;
pub mod wallet;
```

#### `src-tauri/src/domain/mod.rs`
```rust
//! Domain layer — business logic, repository traits, domain errors.
pub mod user;
pub mod product;
pub mod order;
pub mod payment;
pub mod wallet;
pub mod print_job;

use std::fmt;

#[derive(Debug)]
pub enum DomainError {
	Unauthorized,
	NotFound(String),
	ProductNotFound,
	DuplicatePhone,
	InsufficientBalance { balance: i64, required: i64 },
	InvalidStatusTransition { from: String, to: String },
	Internal(String),
}

impl fmt::Display for DomainError { /* ... */ }
impl std::error::Error for DomainError {}
impl From<String> for DomainError { /* ... */ }
```

#### `src-tauri/src/domain/user.rs`
- `User` struct (id, email, password_hash, name, role, created_at, updated_at)
- `Role` enum (Admin, Caissier, Vendeur, GestionnaireStock) with `permissions()` and `from_str()`/`as_str()`
- `Permission` enum (All, Sell, ViewReports, ManageUsers, ManageMenu, ManageStock, ViewOrders, ManageSettings)
- `UserRepository` trait (find_by_email, find_by_id, create, update, delete, list_all)

#### `src-tauri/src/domain/product.rs`
- `Product` struct, `Category` struct
- `ProductRepository` trait (CRUD + search + category management)

#### `src-tauri/src/domain/order.rs`
- `OrderStatus` enum (PendingPayment, PaidPreparing, Ready, Delivered) with `can_transition_to()`
- `Order` struct, `OrderItem` struct
- `Order::new()`, `Order::transition_to()`
- `OrderRepository` trait

#### `src-tauri/src/domain/payment.rs`
- `PaymentMethod` enum (Wallet, Cash, MoMo, Split)
- `Payment` struct
- `PaymentRepository` trait

#### `src-tauri/src/domain/wallet.rs`
- `WalletClient` struct, `LedgerEntryType` enum
- `WalletLedgerEntry` struct
- `WalletRepository` trait (register, find, append_entry, get_balance, get_ledger)

#### `src-tauri/src/domain/print_job.rs`
- `PrintJob` struct

#### `src-tauri/src/db/mod.rs`
```rust
pub mod migrations;
pub mod seed;
pub mod users;
pub mod products;
pub mod orders;
pub mod payments;
pub mod wallet_ledger;

#[derive(Debug)]
pub enum DbError {
	Connection(String),
	Query(String),
	Migration(String),
	NotFound(String),
}

impl From<r2d2::Error> for DbError { /* ... */ }
impl From<rusqlite::Error> for DbError {
	fn from(e: rusqlite::Error) -> Self {
		match e {
			rusqlite::Error::QueryReturnedNoRows => DbError::NotFound("query returned no rows".into()),
			other => DbError::Query(other.to_string()),
		}
	}
}

pub type SqlitePool = r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>;
pub type SqliteConn = r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager>;

const POOL_MAX_SIZE: u32 = 5;
const POOL_MIN_IDLE: u32 = 1;

pub fn init_pool(db_path: &str) -> Result<SqlitePool, DbError> { /* ... */ }
pub fn get_conn(pool: &SqlitePool) -> Result<SqliteConn, DbError> { /* ... */ }
```

#### `src-tauri/src/db/migrations.rs`
```rust
use refinery::embed_migrations;
embed_migrations!("migrations");

pub fn run(conn: &mut SqliteConn) -> Result<(), DbError> {
	let report = migrations_runner().run(conn)
		.map_err(|e| DbError::Migration(format!("Migration failed: {}", e)))?;
	Ok(())
}
```

#### `src-tauri/src/db/seed.rs`
- Placeholder seed — checks user count, skips if >0, otherwise logs placeholder message

#### `src-tauri/src/db/users.rs`
- `DbUserRepository` with full `UserRepository` impl using rusqlite

#### `src-tauri/migrations/V1__users.sql`
```sql
CREATE TABLE IF NOT EXISTS users (
    id          TEXT PRIMARY KEY,
    email       TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    name        TEXT NOT NULL DEFAULT '',
    role        TEXT NOT NULL DEFAULT 'caissier',
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);
```

### Contexte — Story 1.1
- **AC-1**: Dépendances Rust installées (cargo check)
- **AC-2**: Structure api/domain/db créée
- **AC-3**: Migration V1 users
- **AC-4**: Runner refinery au startup
- **AC-5**: Role enum avec permissions dérivées
- **AC-6**: DbError / DomainError — 3 couches
- **AC-7**: Pool r2d2 initialisé
