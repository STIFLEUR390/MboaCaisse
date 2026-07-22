# Acceptance Auditor — Review Prompt

You are an Acceptance Auditor. Review the provided diff against `_bmad-output/implementation-artifacts/1-1-structure-rust-layered-migrations-initiales.md` and any loaded context docs. Check for: violations of acceptance criteria, deviations from spec intent, missing implementation of specified behavior, contradictions between spec constraints and actual code. Output findings as a Markdown list. Each finding: one-line title, which AC/constraint it violates, and evidence from the diff.

## Spec: Story 1.1 — Structure Rust Layered & Migrations Initiales

### AC-1: Dépendances Rust installées (cargo check passe)
Expected deps: tokio (1, full), axum (0.8), tower-http (cors,fs), rusqlite (bundled), r2d2, r2d2-rusqlite (bundled), refinery (rusqlite), refinery-core, argon2, mdns-sd, tracing, tracing-subscriber (env-filter,json), uuid (v7,serde), chrono (serde), thiserror

### AC-2: Structure api/domain/db créée
Expected: api/mod.rs, domain/mod.rs + user.rs, db/mod.rs + migrations.rs + seed.rs + users.rs, lib.rs with mod declarations

### AC-3: Migration V1 — table users
Expected SQL in `migrations/V1__users.sql`

### AC-4: Runner refinery au startup
Expected: executed in setup() before server listens, _schema_version table managed

### AC-5: Role enum avec permissions dérivées
Expected: Role::Admin/Caissier/Vendeur/GestionnaireStock, Permission::All/Sell/ViewReports/ManageUsers/ManageMenu/ManageStock/ViewOrders/ManageSettings, Role::permissions()

### AC-6: DbError / DomainError — 3 couches
Expected: DbError { Connection, Query, Migration, NotFound }, DomainError { InsufficientBalance, ProductNotFound, InvalidStatusTransition, DuplicatePhone, Unauthorized, NotFound, Internal }, impl std::error::Error + Display

### AC-7: Pool r2d2 initialisé
Expected: init_pool() → r2d2 pool, 5 max 1 min, Arc<Pool> in Tauri state

---

## Diff content

### `src-tauri/Cargo.toml` (modifié)
```toml
tokio = { version = "1", features = ["full"] }
axum = "0.8"
tower-http = { version = "0.6", features = ["cors", "fs"] }
rusqlite = { version = "0.32", features = ["bundled"] }
r2d2 = "0.8"
r2d2_sqlite = { version = "0.25", features = ["bundled"] }
refinery = { version = "0.9", features = ["rusqlite-bundled"] }
argon2 = "0.5"
mdns-sd = "0.12"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
uuid = { version = "1", features = ["v7", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
thiserror = "2"
```

### `src-tauri/src/lib.rs` (modifié)
```rust
mod api;
mod db;
mod domain;

use std::sync::Arc;
use db::{init_pool, migrations, SqlitePool};

pub struct AppState {
	pub db_pool: SqlitePool,
}

pub fn run() {
	tracing_subscriber::fmt()
		.with_env_filter(
			tracing_subscriber::EnvFilter::try_from_default_env()
				.unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
		)
		.json()
		.init();

	let db_path = "mboacaisse.db";
	let pool = init_pool(db_path).expect("...");
	{
		let mut conn = pool.get().expect("...");
		migrations::run(&mut conn).expect("...");
		db::seed::run(&mut conn).expect("...");
	}
	let pool = Arc::new(pool);
	let app_state = AppState { db_pool: (*pool).clone() };

	tauri::Builder::default()
		.setup(|app| {
			app.manage(app_state);
			// tray icon setup...
			Ok(())
		})
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}
```

### `src-tauri/src/domain/mod.rs`
- DomainError enum with: Unauthorized, NotFound(String), ProductNotFound, DuplicatePhone, InsufficientBalance{balance,required}, InvalidStatusTransition{from,to}, Internal(String)
- impl Display, Error, From<String>

### `src-tauri/src/domain/user.rs`
- User struct (id, email, password_hash, name, role, created_at, updated_at)
- Role enum (Admin, Caissier, Vendeur, GestionnaireStock) with permissions(): Admin→all 8, Caissier→Sell/ViewReports/ViewOrders, Vendeur→ViewOrders/ManageMenu, GestionnaireStock→ManageStock/ViewReports
- Permission enum with 8 variants
- UserRepository trait (find_by_email, find_by_id, create, update, delete, list_all)

### `src-tauri/src/db/mod.rs`
- DbError enum (Connection, Query, Migration, NotFound) with Display + Error + From impls
- SqlitePool / SqliteConn type aliases
- init_pool() and get_conn()

### `src-tauri/src/db/migrations.rs`
- embed_migrations!("migrations")
- run() calling migrations_runner().run(conn)

### `src-tauri/src/db/seed.rs`
- Placeholder — checks user_count, skips if >0

### `src-tauri/src/db/users.rs`
- DbUserRepository full impl with conn(), row_to_user(), all UserRepository methods

### `src-tauri/migrations/V1__users.sql`
```sql
CREATE TABLE IF NOT EXISTS users (...);
```
