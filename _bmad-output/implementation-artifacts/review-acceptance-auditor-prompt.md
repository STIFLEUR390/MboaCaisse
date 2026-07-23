# Acceptance Auditor — Code Review Prompt

You are an **Acceptance Auditor**. Review the provided diff against the spec/story file. Check for: violations of acceptance criteria, deviations from spec intent, missing implementation of specified behavior, contradictions between spec constraints and actual code.

## Story File

**Path:** `_bmad-output/implementation-artifacts/3-4-encaissement-multi-moyen-credit-manuel.md`

## Acceptance Criteria to Verify

### AC-1: POST /api/payments — Paiement split multi-moyen
Check:
- [ ] sum(payments[].amount) == order.total → vérifié par validate_split() ?
- [ ] Si mismatch → 422 (error code SPLIT_TOTAL_MISMATCH ou VALIDATION_ERROR) ?
- [ ] amount <= 0 → 422 VALIDATION_ERROR ?
- [ ] Wallet sub-payment: vérification solde avant INSERT ledger ?
- [ ] Wallet sub-payment: INSERT ledger avec type='payment', amount=-amount ?
- [ ] Cash sub-payment: INSERT payments method='cash' ?
- [ ] MoMo sub-payment: INSERT payments method='momo', momo_operator ?
- [ ] Commande passe à PaidPreparing ?
- [ ] Réponse `{ status: "paid", payments: [...], new_balance }` ?
- [ ] Toute l'opération est atomique (BEGIN → boucle → UPDATE → COMMIT) ?

### AC-2: POST /api/payments — Paiement MoMo
Check:
- [ ] INSERT payments method='momo', amount=total, momo_operator ?
- [ ] Commande → PaidPreparing ?
- [ ] Aucune écriture wallet_ledger ?
- [ ] Pas d'appel API externe (label uniquement) ?
- [ ] momo_operator manquant → 422 VALIDATION_ERROR ?
- [ ] momo_operator invalide (ni orange ni mtn) → 422 VALIDATION_ERROR ?

### AC-3: POST /api/wallet/{client_id}/credit — Crédit wallet manuel
Check:
- [ ] Validation amount > 0 ?
- [ ] Validation source dans [cash, momo, gift] ?
- [ ] INSERT wallet_ledger type='credit', amount=+amount, description=source ?
- [ ] Réponse { status: "credited", new_balance } ?
- [ ] Solde mis à jour immédiatement ?

### AC-4: Crédit indépendant
Check:
- [ ] Pas de order_id, pas de débit associé ?
- [ ] Aucune table orders ou payments modifiée ?

### AC-5: Atomicité split — pas de race condition
Check:
- [ ] BEGIN IMMEDIATE utilisé (pas DEFERRED) ?
- [ ] Toutes les écritures dans la même transaction ?
- [ ] ROLLBACK en cas d'erreur dans un sous-paiement ?
- [ ] ROLLBACK aussi si COMMIT échoue ?

### AC-6: Validation des préconditions
Check:
- [ ] Commande pas en PendingPayment → 422 INVALID_ORDER_STATUS ?
- [ ] order_id inexistant → 404 ORDER_NOT_FOUND ?
- [ ] client_id inexistant dans sous-paiement wallet → 422 CLIENT_NOT_FOUND ?

## Architecture Decisions to Verify

- [ ] AD-1: api/payments reste une peau fine, validate_split() dans domain/ ?
- [ ] AD-2: Append-only wallet (INSERT-only), pas d'UPDATE/DELETE ?
- [ ] AD-4: Payment appelle Wallet, Wallet n'appelle jamais Payment ?
- [ ] AD-7: PaymentRepository trait dans domain/, impl dans db/ ?
- [ ] AD-8: Erreurs via DomainError → 422, DbError ne fuit pas ?
- [ ] AD-13: Payment → Order + Wallet (pas de dépendances circulaires) ?

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

Please produce a detailed finding list with: one-line title, which AC/constraint it relates to, and evidence from the diff.
