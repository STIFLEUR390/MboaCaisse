# Edge Case Hunter — Code Review Prompt

You are an **Edge Case Hunter** reviewer. Your mission is to walk every branching path and boundary condition in the diff below. Find ALL corner cases, failure modes, state inconsistencies, race conditions, and unexpected behaviors.

**Focus areas:**
- Empty/null/missing input handling
- Boundary values (zero, negative, overflow)
- Concurrent access and race conditions
- Partial failure recovery
- State machine transitions (valid and invalid)
- DB transaction rollback paths
- Error propagation and masking
- Time-of-check/time-of-use (TOCTOU) races
- Resource exhaustion (DB connections, memory)

**Be exhaustive — if there's a way to break it, find it.**

## Diff

```
diff --git a/_bmad-output/implementation-artifacts/3-4-encaissement-multi-moyen-credit-manuel.md b/_bmad-output/implementation-artifacts/3-4-encaissement-multi-moyen-credit-manuel.md
index d2bbce5..5fd2917 100644
--- a/_bmad-output/implementation-artifacts/3-4-encaissement-multi-moyen-credit-manuel.md
+++ b/_bmad-output/implementation-artifacts/3-4-encaissement-multi-moyen-credit-manuel.md
@@ -4,7 +4,7 @@ baseline_commit: 116d7d3

 # Story 3.4: Encaissement Multi-Moyen & Crédit Manuel

-Status: ready-for-dev
+Status: review

 ## Story

@@ -96,10 +96,10 @@ so that le client paie comme il veut et le caissier peut approvisionner un walle
   - `CREATE INDEX idx_payments_parent ON payments(parent_payment_id);`

 ### Tâche 2: Étendre domain/payment.rs (AC: AC-1, AC-2, AC-6)
-- [ ] Ajouter `pub momo_operator: Option<String>` au struct `Payment`
-- [ ] Ajouter `pub parent_payment_id: Option<String>` au struct `Payment`
+- [x] Ajouter `pub momo_operator: Option<String>` au struct `Payment`
+- [x] Ajouter `pub parent_payment_id: Option<String>` au struct `Payment`
 - [ ] Créer struct `SplitPaymentItem` avec `{ method, amount, client_id?, momo_operator? }`
-- [ ] Ajouter `fn validate_split(payments: &[SplitPaymentItem], total: i64) -> Result<(), DomainError>`
+- [x] Ajouter `fn validate_split(payments: &[SplitPaymentItem], total: i64) -> Result<(), DomainError>`
   - Vérifie `sum(amounts) == total`
   - Vérifie chaque `amount > 0`
   - Vérifie `client_id` présent pour method=wallet
@@ -107,36 +107,36 @@ so that le client paie comme il veut et le caissier peut approvisionner un walle
   - Retourne `DomainError::InvalidValue` avec message clair

 ### Tâche 3: Implémenter le handler MoMo dans api/payments.rs (AC: AC-2, AC-6)
-- [ ] Remplacer le `_ =>` match arm pour `PaymentMethod::MoMo`
+- [x] Remplacer le `_ =>` match arm pour `PaymentMethod::MoMo`
 - [ ] Ajouter `momo_operator: Option<String>` à `ProcessPaymentRequest`
 - [ ] Valider `momo_operator` présent et dans `["orange", "mtn"]`
-- [ ] Transaction atomique (BEGIN IMMEDIATE) : INSERT payments + UPDATE order
+- [x] Transaction atomique (BEGIN IMMEDIATE) : INSERT payments + UPDATE order
 - [ ] Réponse `{ "status": "paid", "payment_id": "..." }` (pas de new_balance)

 ### Tâche 4: Implémenter le handler Split dans api/payments.rs (AC: AC-1, AC-5, AC-6)
 - [ ] Ajouter `payments: Vec<SplitPaymentItem>` à `ProcessPaymentRequest` (avec `#[serde(default)]`)
 - [ ] Extraire fonction privée `debit_wallet_in_tx()` réutilisable
-- [ ] Transaction unique BEGIN IMMEDIATE pour tout le split :
+- [x] Transaction unique BEGIN IMMEDIATE pour tout le split :
   1. Vérifier chaque sous-paiement wallet (solde)
   2. INSERT ledger pour chaque wallet
   3. INSERT payments pour chaque cash/momo
   4. UPDATE orders → PaidPreparing
   5. COMMIT
 - [ ] Réponse : `{ "status": "paid", "payments: [...], "new_balance": <restant> }`
-- [ ] Tous les sous-paiements ont `parent_payment_id` = UUID du paiement parent
+- [x] Tous les sous-paiements ont `parent_payment_id` = UUID du paiement parent

 ### Tâche 5: Ajouter POST /api/wallet/{client_id}/credit dans api/wallet.rs (AC: AC-3, AC-4)
 - [ ] Créer `CreditWalletRequest { amount: i64, source: String, reference?: Option<String> }`
-- [ ] Valider `amount > 0`, `source` dans `["cash", "momo", "gift"]`
-- [ ] Handler `credit_wallet`: vérifier client → append_entry(WalletLedgerEntry::Credit) → get_balance → réponse
-- [ ] Utiliser `WalletRepository::append_entry()` (déjà atomique)
+- [x] Valider `amount > 0`, `source` dans `["cash", "momo", "gift"]`
+- [x] Handler `credit_wallet`: vérifier client → append_entry(WalletLedgerEntry::Credit) → get_balance → réponse
+- [x] Utiliser `WalletRepository::append_entry()` (déjà atomique)

 ### Tâche 6: Routes dans api/mod.rs (AC: AC-1, AC-2, AC-3)
-- [ ] Ajouter `.route("/api/wallet/{client_id}/credit", post(wallet::credit_wallet))`
-- [ ] S'assurer qu'aucun conflit avec `/api/wallet/{id}/ledger` (GET vs POST)
+- [x] Ajouter `.route("/api/wallet/{client_id}/credit", post(wallet::credit_wallet))`
+- [x] S'assurer qu'aucun conflit avec `/api/wallet/{id}/ledger` (GET vs POST)

 ### Tâche 7: Vérification compilation
-- [ ] `cargo check` sans erreur
+- [x] `cargo check` sans erreur

 ## Dev Notes

@@ -326,12 +326,25 @@ bmad-create-story via GPT-5 (Codex)
 - **Routes** : `/api/wallet/{client_id}/credit` POST — pas de conflit avec `/api/wallet/{id}/ledger` GET.

 ### Completion Notes List
+- Migration V6 avec colonnes `momo_operator` et `parent_payment_id`
+- Extension domain/payment.rs: SplitPaymentItem, validate_split(), nouveaux champs Payment
+- Handler MoMo: validation operator, BEGIN IMMEDIATE, label-only
+- Handler Split: debit_wallet_in_tx() extraite, atomicite BEGIN IMMEDIATE, parent_payment_id
+- Credit wallet manuel via POST /api/wallet/{client_id}/credit (sources: cash/momo/gift)
+- DbPaymentRepository mis a jour pour les 2 nouvelles colonnes
+- Routes ajoutees dans api/mod.rs, aucun conflit
+- cargo check OK (0 erreurs, warnings preexistants)

 ### File List

-- [ ] `src-tauri/migrations/V6__payments_extras.sql` — NEW
-- [ ] `src-tauri/src/domain/payment.rs` — MODIFY
-- [ ] `src-tauri/src/api/payments.rs` — MODIFY
-- [ ] `src-tauri/src/db/payments.rs` — MODIFY
-- [ ] `src-tauri/src/api/wallet.rs` — MODIFY
-- [ ] `src-tauri/src/api/mod.rs` — MODIFY
+- [x] `src-tauri/migrations/V6__payments_extras.sql` — NEW
+- [x] `src-tauri/src/domain/payment.rs` — MODIFY
+- [x] `src-tauri/src/api/payments.rs` — MODIFY
+- [x] `src-tauri/src/db/payments.rs` — MODIFY
+- [x] `src-tauri/src/api/wallet.rs` — MODIFY
+- [x] `src-tauri/src/api/mod.rs` — MODIFY
+
+
+## Change Log
+
+- **2026-07-23**: Implementation complete story 3.4
diff --git a/_bmad-output/implementation-artifacts/sprint-status.yaml b/_bmad-output/implementation-artifacts/sprint-status.yaml
index 121b676..2c538de 100644
--- a/_bmad-output/implementation-artifacts/sprint-status.yaml
+++ b/_bmad-output/implementation-artifacts/sprint-status.yaml
@@ -69,7 +69,7 @@ development_status:
   3-1-crud-produits-categories: done
   3-2-cycle-de-vie-commande: done
   3-3-payment-gate: done
-  3-4-encaissement-multi-moyen-credit-manuel: ready-for-dev
+  3-4-encaissement-multi-moyen-credit-manuel: review
   3-5-kitchen-display: backlog
   3-6-ticket-numerique: backlog
   epic-3-retrospective: optional
diff --git a/src-tauri/src/api/mod.rs b/src-tauri/src/api/mod.rs
index 46d46ea..d9823e1 100644
--- a/src-tauri/src/api/mod.rs
+++ b/src-tauri/src/api/mod.rs
@@ -99,7 +99,12 @@ pub fn build_app(state: AppApiState) -> Router {
 			"/api/wallet/{id}/ledger",
 			get(crate::api::wallet::get_ledger),
 		)
-		// Products CRUD (story 3.1)
+			// Wallet credit (story 3.4)
+			.route(
+				"/api/wallet/{client_id}/credit",
+				post(crate::api::wallet::credit_wallet),
+			)
+			// Products CRUD (story 3.1)
 		.route("/api/products", get(products::list_products))
 		.route("/api/products", post(products::create_product))
 		.route("/api/products/{id}", get(products::get_product))
diff --git a/src-tauri/src/api/payments.rs b/src-tauri/src/api/payments.rs
index 0c990aa..f5bc67b 100644
--- a/src-tauri/src/api/payments.rs
+++ b/src-tauri/src/api/payments.rs
@@ -21,7 +21,7 @@ use axum::{
 use serde::{Deserialize, Serialize};

 use crate::domain::order::{OrderRepository, OrderStatus};
-use crate::domain::payment::{Payment, PaymentMethod, PaymentRepository};
+use crate::domain::payment::{self as domain_payment, Payment, PaymentMethod, PaymentRepository, SplitPaymentItem};
 use crate::domain::wallet::{LedgerEntryType, WalletLedgerEntry, WalletRepository};
 use crate::domain::DomainError;
 use crate::db::SqlitePool;
@@ -57,6 +57,12 @@ pub struct ProcessPaymentRequest {
 	pub method: String,
 	#[serde(default)]
 	pub client_id: Option<String>,
+	/// MoMo operator (orange|mtn). Required when method=momo.
+	#[serde(default)]
+	pub momo_operator: Option<String>,
+	/// Split sub-payments. Required when method=split.
+	#[serde(d
```

The full diff is available in the working tree at `/var/home/herold/Project/tauri/MboaCaisse`.

## Story 3.4 — Acceptance Criteria (key behaviors)

### AC-1: Split multi-moyen
- POST /api/payments avec method=split, payments=[wallet+cash+etc]
- Chaque sous-paiement traité dans BEGIN IMMEDIATE
- Wallet: solde vérifié, INSERT ledger amount=-X
- Cash: INSERT payments method='cash'
- Commande → PaidPreparing
- Réponse avec status, payments[], new_balance
- sum(payments) != total → 422 SPLIT_TOTAL_MISMATCH
- amount <= 0 → 422 VALIDATION_ERROR

### AC-2: MoMo label-only
- POST /api/payments avec method=momo, momo_operator = orange|mtn
- INSERT payments method='momo', momo_operator=op
- Pas d'écriture wallet_ledger
- Pas d'appel API externe
- momo_operator manquant → 422

### AC-3: Credit wallet manuel
- POST /api/wallet/{client_id}/credit avec amount, source (cash|momo|gift)
- INSERT wallet_ledger type='credit', amount=+amount
- Réponse new_balance

### AC-5: Atomicité split
- BEGIN IMMEDIATE bloque écritures concurrentes
- Toute erreur → ROLLBACK complet

### AC-6: Préconditions
- Commande pas en PendingPayment → 422 INVALID_ORDER_STATUS
- order_id inexistant → 404 ORDER_NOT_FOUND
- client_id inexistant → 422 CLIENT_NOT_FOUND

## Key code paths to trace

1. MoMo: operator validation → invalid operator → ? missing operator → ?
2. Split: validate_split returns Err → ? empty payments vec → ?
3. Split: wallet sub-payment with wallet_negative=true → balance < amount → ?
4. Split: one sub-payment succeeds, another fails → ROLLBACK restores all?
5. Split: COMMIT fails after partial writes → ? ROLLBACK if COMMIT fails?
6. debit_wallet_in_tx: called outside transaction → ? nested transaction → ?
7. credit_wallet: client not found → 404? append_entry also checks existence
8. Concurrent credit + payment on same wallet → race condition?
9. AppHandle uninitialized → Config::load → panic?
10. DB pool exhausted → blocked on pool.get()

Please produce a thorough edge case analysis.
