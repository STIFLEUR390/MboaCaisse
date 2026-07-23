# Blind Hunter — Code Review Prompt

You are a **Blind Hunter** reviewer. You have NO context about the project, story, or acceptance criteria. Review the diff below with fresh eyes and identify ALL issues: bugs, logic errors, security vulnerabilities, concurrency problems, anti-patterns, code smells, naming issues, and deviations from Rust best practices.

**Focus areas:**
- Race conditions and thread safety
- Error handling (panics, unwraps, expect)
- Resource leaks (connections, files)
- Logic errors in control flow
- Security issues (SQL injection, data leaks)
- Rust idioms and best practices violations
- Unnecessary allocations or clones
- Missing invariants or assertions

**Be aggressive — you are looking for problems, not praising code.**

## Diff

```
diff --git a/src-tauri/src/api/mod.rs b/src-tauri/src/api/mod.rs
--- a/src-tauri/src/api/mod.rs
+++ b/src-tauri/src/api/mod.rs
@@ -22,6 +22,8 @@
+use crate::db::SqlitePool;
+use crate::domain::payment::PaymentRepository;
 use crate::domain::product::ProductRepository;
 use crate::domain::order::OrderRepository;
 use crate::domain::user::UserRepository;
@@ -45,13 +47,19 @@
 #[derive(Clone)]
 pub struct AppApiState {
 	pub user_repo: Arc<dyn UserRepository>,
 	pub order_repo: Arc<dyn OrderRepository>,
 	pub wallet_repo: Arc<dyn WalletRepository>,
 	pub product_repo: Arc<dyn ProductRepository>,
+	pub payment_repo: Arc<dyn PaymentRepository>,
 	pub jwt_secret: Arc<Vec<u8>>,
+	pub db_pool: SqlitePool,
 }
```

**For the full diff, see the actual git changes in the working tree at `/var/home/herold/Project/tauri/MboaCaisse`.**

Key files changed:
1. `src-tauri/src/api/payments.rs` — New payment handler (428 lines)
2. `src-tauri/src/db/payments.rs` — PaymentRepository implementation (120 lines)
3. `src-tauri/src/api/mod.rs` — AppApiState additions
4. `src-tauri/src/lib.rs` — Dependency injection
5. `src-tauri/src/settings.rs` — wallet_negative config
6. `src-tauri/src/api/settings.rs` — wallet_negative API
7. `src-tauri/migrations/V5__payments.sql` — Payments table

Please review ALL files, not just the ones partially shown above.
