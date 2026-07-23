---
baseline_commit: 54e8a4f
---

# Story 3.3: Payment Gate

Status: done

## Story

As a caissier,
I want que le paiement soit déduit du wallet AVANT validation de la commande,
so that le solde client est garanti avant que la cuisine prépare.

## Acceptance Criteria

### AC-1: POST /api/payments — Paiement wallet avec débit
**Given** WalletRepository du wallet ledger (E1.5) et OrderRepository (3.2)
**When** POST `/api/payments` avec
```json
{
	"order_id": "ord-xxx",
	"method": "wallet",
	"client_id": "cli-yyy"
}
```
**Then** le système vérifie le solde du client (`wallet_repo.get_balance`) avant de débiter
**And** si solde >= total de la commande :
  - INSERT dans `wallet_ledger` avec `type='payment'`, `amount=-total`, `reference=order_id`
  - INSERT dans `payments` avec `method='wallet'`, `amount=total`
  - Transition de la commande vers `PaidPreparing`
  - Réponse `{ "status": "paid", "new_balance": <balance_after>, "payment_id": "..." }`
**And** toute cette opération est atomique : BEGIN → vérifications → INSERT ledger → INSERT payment → UPDATE order → COMMIT

### AC-2: Solde insuffisant (wallet négatif désactivé)
**Given** le paramètre `wallet_negative` est `false` (défaut)
**When** POST `/api/payments` avec `method: "wallet"` et `balance < total`
**Then** 422 Unprocessable Entity avec
```json
{ "error": "Insufficient balance: 1000 FCFA (need 2500)", "code": "INSUFFICIENT_BALANCE" }
```
**And** la commande reste en `PendingPayment`, aucune écriture ledger

### AC-3: POST /api/payments — Paiement cash
**Given** une commande en `PendingPayment`
**When** POST `/api/payments` avec
```json
{ "order_id": "ord-xxx", "method": "cash" }
```
**Then** INSERT dans `payments` avec `method='cash'`, `amount=total`
**And** la commande passe à `PaidPreparing`
**And** aucune écriture dans `wallet_ledger`

### AC-4: Wallet négatif activé (paramètre admin)
**Given** le paramètre `wallet_negative` est `true`
**When** un débit wallet rendrait le solde négatif
**Then** le paiement est accepté, le solde devient négatif
**And** la commande passe à `PaidPreparing`

### AC-5: Atomicité — pas de race condition
**Given** deux requêtes POST `/api/payments` simultanées sur le même wallet
**When** les deux transactions s'exécutent
**Then** chaque transaction est atomique :
```
BEGIN IMMEDIATE
  SELECT COALESCE(SUM(amount), 0) FROM wallet_ledger WHERE client_id = ?
  (si assez) INSERT INTO wallet_ledger ...
  INSERT INTO payments ...
  UPDATE orders SET status = 'paid_preparing'
COMMIT
```
**And** pas de race condition possible (BEGIN IMMEDIATE bloque les écritures concurrentes)

### AC-6: Validation des préconditions
**Given** une commande qui n'est pas en `PendingPayment`
**When** POST `/api/payments`
**Then** 422 `INVALID_ORDER_STATUS` : "Order not in PendingPayment status"
**Given** un `order_id` inexistant
**When** POST `/api/payments`
**Then** 404 `ORDER_NOT_FOUND`
**Given** un `client_id` inexistant (pour method=wallet)
**When** POST `/api/payments`
**Then** 422 `CLIENT_NOT_FOUND`
**Given** POST `/api/payments` avec `method: "wallet"` sans `client_id`
**When** validation de la requête
**Then** 400 `VALIDATION_ERROR` : "client_id is required for wallet payments"

## Tasks / Subtasks

### Tâche 1: Migration V5__payments.sql (AC: AC-1)
- [x] Créer `src-tauri/migrations/V5__payments.sql`
  - Table `payments` : `id TEXT PK`, `order_id TEXT NOT NULL REFERENCES orders(id)`, `method TEXT NOT NULL`, `amount INTEGER NOT NULL`, `client_id TEXT`, `reference TEXT`, `created_at TEXT NOT NULL`
  - Index sur `payments(order_id)`, `payments(client_id)`
  - L'ordre des migrations existantes : V1=users, V2=wallet_ledger, V3=products, V4=orders

### Tâche 2: Implémenter PaymentRepository dans db/payments.rs (AC: AC-1, AC-5)
- [x] Remplacer les `todo!("Story 3.3")` par l'implémentation complète
- [x] `create(&self, payment: &Payment) -> Result<(), DomainError>` — INSERT dans payments
- [x] `find_by_id(&self, id: &str)` — SELECT par id
- [x] `list_by_order(&self, order_id: &str)` — SELECT par order_id, ORDER BY created_at
- [x] `list_by_client(&self, client_id: &str)` — SELECT par client_id
- [x] Patterns à suivre : rusqlite params, row mapping comme dans `db/wallet_ledger.rs`

### Tâche 3: Payment gate — api/payments.rs (AC: AC-1 à AC-6)
- [x] Créer `PaymentsState` avec `order_repo`, `wallet_repo`, `payment_repo`
- [x] Implémenter `FromRef<AppApiState>` pour `PaymentsState`
- [x] Handler `process_payment` pour POST `/api/payments`
  - Parse la requête : `{ order_id: String, method: String, client_id?: Option<String> }`
  - Valide la méthode via `PaymentMethod::from_str`
  - Récupère la commande, vérifie `PendingPayment`
  - Si `method=wallet` :
    - Vérifie `client_id` présent
    - Récupère le solde
    - Vérifie solde >= total OU `wallet_negative` activé
    - Transaction atomique : append_entry ledger (type=payment, amount=-total) + create payment + update_status
  - Si `method=cash` :
    - Crée le payment + update_status (pas de ledger)
  - Retourne `{ status: "paid", new_balance, payment_id }`
- [x] Gérer les erreurs 400/404/422 via `domain_to_http`

### Tâche 4: Ajouter payment_repo à AppApiState + routes (AC: AC-1)
- [x] Dans `api/mod.rs` :
  - Ajouter `pub payment_repo: Arc<dyn PaymentRepository>` à `AppApiState`
  - Ajouter la route : `.route("/api/payments", post(payments::process_payment))`
  - Ajouter `use crate::domain::payment::PaymentRepository;`
  - Ajouter `pub db_pool: SqlitePool` à `AppApiState` (pour transactions atomiques)
- [x] Dans `src-tauri/src/lib.rs` :
  - Importer `DbPaymentRepository` : `use db::payments::DbPaymentRepository;`
  - Instancier : `let payment_repo: Arc<dyn PaymentRepository> = Arc::new(DbPaymentRepository::new(pool.clone()));`
  - Ajouter à `AppApiState` : `payment_repo,`

### Tâche 5: Ajouter wallet_negative à Config (AC: AC-4)
- [x] Dans `src-tauri/src/settings.rs` :
  - Ajouter `pub wallet_negative: bool` au struct `Config` (default: `false`)
  - Ajouter `"wallet_negative"` à `KNOWN_CONFIG_KEYS`
  - Charger depuis le store : `store.get("wallet_negative").and_then(|v| v.as_bool()).unwrap_or(false)`
  - Ajouter au `set()` et `reset()` (ne nécessite pas de redémarrage)
- [x] Dans `api/settings.rs` :
  - Ajouter `wallet_negative` à `PatchSettingsBody`
  - Ajouter à `entries_from_config()`
  - Validation : valeur booléenne uniquement

### Tâche 6: Vérification compilation
- [x] `cargo check` sans erreur

## Dev Notes

### Architecture patterns à suivre

- **AD-1** : `api/payments.rs` = peau fine (parse, appelle domain, sérialise). Pas de logique métier.
- **AD-2** : `wallet_ledger` INSERT-only. Le débit wallet = INSERT d'une ligne `type='payment', amount=-total`. Jamais de UPDATE/DELETE.
- **AD-4** : Payment appelle Wallet via `wallet_repo.append_entry()` et `wallet_repo.get_balance()`. Wallet ne dépend jamais de Payment.
- **AD-7** : `PaymentRepository` trait défini dans `domain/payment.rs`. `DbPaymentRepository` dans `db/payments.rs`.
- **AD-8** : Erreurs 3 couches : `DomainError::InsufficientBalance` → 422. `DbError` ne sort jamais de `db/`.
- **AD-13** : Graphe dépendances : Payment → Order (order_repo) + Wallet (wallet_repo). Order déjà en PendingPayment.

### Source tree components to touch

| Fichier | Action |
|---------|--------|
| `src-tauri/migrations/V5__payments.sql` | **NEW** — migration payments table |
| `src-tauri/src/db/payments.rs` | **MODIFY** — implémenter PaymentRepository |
| `src-tauri/src/api/payments.rs` | **MODIFY** — handlers payment gate |
| `src-tauri/src/api/mod.rs` | **MODIFY** — ajouter + payment_repo dans AppApiState + route |
| `src-tauri/src/lib.rs` | **MODIFY** — instancier DbPaymentRepository |
| `src-tauri/src/settings.rs` | **MODIFY** — ajouter wallet_negative config |
| `src-tauri/src/api/settings.rs` | **MODIFY** — ajouter wallet_negative à l'API settings |

### Convention pour la transaction atomique (AC-5)

Utiliser `BEGIN IMMEDIATE` dans `api/payments.rs` directement (pas dans `db/payments.rs`), car la transaction span trois repositories différents (wallet_ledger, payments, orders). Pattern à suivre depuis `db/wallet_ledger.rs::append_entry` :

```rust
// Dans api/payments.rs
let conn = pool.get()?;
conn.execute("BEGIN IMMEDIATE", [])?;

// 1. Vérifier solde
let balance: i64 = conn.query_row(
    "SELECT COALESCE(SUM(amount), 0) FROM wallet_ledger WHERE client_id = ?1",
    params![client_id], |row| row.get(0)
)?;

// 2. Valider solde
if balance < total && !wallet_negative { /* rollback + erreur */ }

// 3. INSERT ledger
conn.execute(
    "INSERT INTO wallet_ledger (...) VALUES (...)",
    params![...]
)?;

// 4. INSERT payment
conn.execute(
    "INSERT INTO payments (...) VALUES (...)",
    params![...]
)?;

// 5. UPDATE order
conn.execute(
    "UPDATE orders SET status = 'paid_preparing', updated_at = ?1 WHERE id = ?2",
    params![now, order_id]
)?;

conn.execute("COMMIT", [])?;
```

Pour accéder au pool depuis api/ : `AppApiState` peut étendre `FromRef` pour exposer le pool directement, ou bien utiliser `wallet_repo.append_entry()` + `order_repo.update_status()` + `payment_repo.create()` dans des transactions séparées si on ne peut pas les faire dans une seule transaction SQL — **mais ce n'est PAS atomique** et viole AC-5.

**Solution recommandée** : Ajouter `pub db_pool: SqlitePool` à `AppApiState` (déjà disponible via `crate::db::SqlitePool`), ou déléguer la transaction atomique à un nouveau trait `PaymentService` dans `domain/payment.rs` qui orchestre les trois écritures dans une transaction unique.

**Approche préférée** : Ajouter une méthode `process_wallet_payment` à `WalletRepository` ou créer une fonction dans `db/payments.rs` qui prend le pool, order_repo+wallet_repo+payment_repo et gère la transaction. Ou plus simplement : exposer le pool dans AppApiState.

### Références pool/connection depuis api/

Le pool (`SqlitePool`) n'est pas actuellement dans `AppApiState`. Il faudrait l'ajouter, ou bien restructurer. Voir comment `server.rs` et `lib.rs` gèrent l'état.

**Alternative pragmatique :** Faire trois appels séparés :
1. `wallet_repo.append_entry()` — déjà atomique dans BEGIN/COMMIT (AD-2)
2. `payment_repo.create()` — INSERT simple
3. `order_repo.update_status()` — UPDATE simple

Risque : si l'appel 2 ou 3 échoue après le 1, le ledger est déjà débité mais la commande pas payée. **Ce n'est pas acceptable** (violation AC-5).

**Donc** : soit ajouter le pool à AppApiState, soit créer une méthode `process_payment()` dans `db/payments.rs` qui prend le pool et fait la transaction complète.

### Imports à ne pas oublier

- `use crate::domain::payment::{Payment, PaymentMethod, PaymentRepository};`
- `use crate::domain::wallet::WalletRepository;`
- `use crate::domain::order::{Order, OrderStatus, OrderRepository};`
- `use crate::db::SqlitePool;` (si pool exposé dans AppApiState)
- `use std::sync::Arc;`
- `use axum::extract::{FromRef, Path, State};`

### Testing standards summary

Pas de framework de test configuré. Vérification manuelle via curl :
```bash
# Créer un client wallet (si pas déjà fait)
curl -X POST http://localhost:3000/api/wallet/register \
  -H 'Content-Type: application/json' \
  -H 'Cookie: mboa_session=...' \
  -d '{"phone":"691234567","name":"Test Client"}'

# Créer une commande
curl -X POST http://localhost:3000/api/orders \
  -H 'Content-Type: application/json' \
  -H 'Cookie: mboa_session=...' \
  -d '{"items":[{"product_id":"p1","quantity":1}]}'

# Payer avec wallet
curl -X POST http://localhost:3000/api/payments \
  -H 'Content-Type: application/json' \
  -H 'Cookie: mboa_session=...' \
  -d '{"order_id":"ord-xxx","method":"wallet","client_id":"cli-yyy"}'

# Payer en cash
curl -X POST http://localhost:3000/api/payments \
  -H 'Content-Type: application/json' \
  -H 'Cookie: mboa_session=...' \
  -d '{"order_id":"ord-xxx","method":"cash"}'

# Vérifier le statut de la commande
curl http://localhost:3000/api/orders/ord-xxx \
  -H 'Cookie: mboa_session=...' | jq .
```

### Project Structure Notes

- Le fichier `api/payments.rs` existe déjà (vide, juste commentaire). Ne pas le recréer, remplir le contenu.
- Le fichier `db/payments.rs` existe déjà avec `todo!()` pour chaque méthode du trait. Remplacer les implémentations.
- `db/mod.rs` a déjà `pub mod payments;` — pas besoin d'ajouter.
- `api/mod.rs` a déjà `pub mod payments;` — pas besoin d'ajouter.
- Le trait `PaymentRepository` existe dans `domain/payment.rs` — ne pas le modifier.
- Le struct `Payment` et `PaymentMethod` existent dans `domain/payment.rs` — ne pas les modifier.

### Gotchas spécifiques

1. **Ordre des routes Axum** : Dans `api/mod.rs`, la route `/api/payments` doit être déclarée dans le `api_routes` builder. Vérifier qu'elle n'entre pas en conflit avec d'autres routes commençant par `/api/pay*`.

2. **BEGIN IMMEDIATE vs DEFERRED** : Toujours utiliser `BEGIN IMMEDIATE` pour les transactions financières. En SQLite, `BEGIN DEFERRED` (le défaut) peut échouer avec `SQLITE_BUSY` sous contention. `IMMEDIATE` acquiert un lock réservé immédiatement.

3. **Refinery & V5** : Le runner refinery s'exécute dans `lib.rs` via `migrations::run(&mut conn)`. S'assurer que la migration V5 est placée dans `src-tauri/migrations/` et suit le format `V5__description.sql`. Vérifier que les migrations existantes sont V1, V2, V3, V4 — V5 est bien le numéro suivant.

4. **Pool non disponible dans api/** : `AppApiState` n'a pas actuellement de champ `db_pool`. Voir "Dev Notes" ci-dessus pour la stratégie de transaction atomique. **Option recommandée** : Ajouter `pub db_pool: SqlitePool` à `AppApiState` pour permettre aux handlers API de faire des transactions跨repositories. C'est un pattern déjà utilisé implicitement (les repositories ont chacun leur pool), mais pour une transaction atomique multi-repo, le handler API a besoin d'accès direct au pool.

5. **ApiError dans wallet** : `api/wallet.rs` a son propre type `ApiError` privé et sa fonction `error_response()`. `api/orders.rs` a aussi son propre `ApiError` et `domain_to_http()`. Ne pas réutiliser l'un pour l'autre — chaque module API a ses propres helpers.

6. **Config `wallet_negative`** : Ajouter à `settings.rs` ET `api/settings.rs`. Dans `api/settings.rs`, suivre le pattern existant (`headless` est un bon modèle car c'est aussi un bool). Pas de `requires_restart` pour `wallet_negative`.

### Pourquoi pas de nouveau trait PaymentService ?

On pourrait créer un `PaymentService` dans domain/ pour encapsuler la logique métier du payment gate (vérification solde, débit, transition). C'est l'approche "Rich Domain" d'AD-1. Cependant, pour garder la story simple et cohérente avec le pattern existant (où les handlers API appellent directement les repositories), la logique est dans `api/payments.rs`. La transaction atomique est gérée via le pool exposé dans AppApiState.

Si la logique devient plus complexe (story 3.4 avec split payments), un refactoring vers `domain/payment.rs` pourra être fait.

### References

- [Source: epics.md#Story-3.3] — Définition originale de la story avec ACs
- [Source: ARCHITECTURE-SPINE.md#AD-1] — Paradigme Layered + Rich Domain
- [Source: ARCHITECTURE-SPINE.md#AD-2] — Append-only financier, transaction atomique
- [Source: ARCHITECTURE-SPINE.md#AD-4] — Payment et Wallet séparés
- [Source: ARCHITECTURE-SPINE.md#AD-7] — Traits dans domain/, impl dans db/
- [Source: ARCHITECTURE-SPINE.md#AD-8] — Erreurs 3 couches
- [Source: ARCHITECTURE-SPINE.md#AD-13] — Graphe dépendances (Payment → Order+Wallet)
- [Source: domain/payment.rs] — PaymentMethod, Payment, PaymentRepository trait (ne pas modifier)
- [Source: domain/wallet.rs] — WalletLedgerEntry, LedgerEntryType, WalletRepository trait
- [Source: domain/order.rs] — Order, OrderStatus, OrderRepository trait
- [Source: db/wallet_ledger.rs] — Pattern BEGIN IMMEDIATE/COMMIT pour transactions atomiques
- [Source: api/orders.rs] — Pattern handlers, FromRef<AppApiState>, domain_to_http()
- [Source: api/wallet.rs] — Pattern erreurs, helpers uuid_v7/chrono_now
- [Source: settings.rs] — Config struct (pattern pour ajouter wallet_negative)
- [Source: .ai-memory/index.md] — Gotchas refinery 0.9, Axum 0.8 Router avec state
- [Source: 3-2-cycle-de-vie-commande.md] — Patterns handlers, review findings, convention DbError/DomainError

## Review Findings

### Action Items
- [x] [Review][Decision] Statut code pour client_id manquant — résolu: garder 422 (cohérent avec le codebase) — AC-6 spécifie 400 mais le code retourne 422. Décider si on garde 422 (plus correct sémantiquement) ou on aligne sur 400 (conformité AC).
- [x] [Review][Patch] Branche redondante dans new_balance —  produit la même valeur dans les deux branches [api/payments.rs:329-333]
- [x] [Review][Patch] Trou atomicité paiement cash — si update_status() échoue après create(), le payment est orphelin (payé mais commande en PendingPayment) [api/payments.rs:380]
- [x] [Review][Patch] Validate appelé dans la transaction — payment.validate() est exécuté APRÈS BEGIN IMMEDIATE; le déplacer avant économiserait la transaction en cas d'échec [api/payments.rs:296]
- [x] [Review][Defer] Config::load() à chaque paiement — lire le store Tauri à chaque requête ajoute de la latence. Accepter pour l'alpha.
- [x] [Review][Defer] WalletLedgerEntry.validate() bypassé — le INSERT brut contourne la validation du domaine. Amount = -total ≠ 0, pas de risque réel.
- [x] [Review][Defer] Panique si AppHandle non initialisé — Config::load() utilise expect() sur le OnceLock. Impossible en pratique car le serveur démarre après setup().
- [x] [Review][Defer] TOCTOU sur existence client — vérifié avant BEGIN IMMEDIATE. Client supprimé entre les deux est improbable et rattrapé par FK.
- [x] [Review][Defer] Échec ROLLBACK silencieux —  ignore l'erreur. Si ROLLBACK échoue, la connexion est probablement déjà cassée.

### Review Follow-ups
- [ ]

<!-- Les éléments ci-dessous seront remplis lors de la revue de code -->

### Review Follow-ups
- [ ]

## Dev Agent Record

### Agent Model Used

bmad-dev-story via GPT-5 (Codex)

### Debug Log References

- **Transaction atomique multi-repo** : Le défi principal de cette story est AC-5 (atomicité). Solution choisie : ajouter `db_pool` à `AppApiState` et faire la transaction complète dans `api/payments.rs` avec `BEGIN IMMEDIATE` + 5 opérations SQL dans une seule transaction. Vérifier que `SqlitePool` est exporté depuis `db/mod.rs` (il l'est).
- **Migration V5** : Créée à la suite de V4 (orders). Contient la table `payments` avec index.
- **Settings** : `wallet_negative` ajouté comme 5ème clé de config avec valeur par défaut `false`. Accessible via `Config::load().wallet_negative`.
- **Tests** : Aucun framework de test Rust configuré. Tester manuellement avec curl.
- **Dépendance sur E1.5** : WalletRepository (wallet_ledger) et OrderRepository (3.2) sont tous deux fonctionnels.
- **Approche transaction atomique** : Au lieu de faire trois appels séparés (append_entry, create, update_status) risquant une incohérence, le handler `process_payment` pour wallet utilise une connexion brute au pool et exécute les 5 étapes dans une seule transaction BEGIN IMMEDIATE → COMMIT. Cash reste simple (deux appels séparés).
- **Routes API** : `/api/payments` en POST, monté dans `api_routes` dans `api/mod.rs`.
- **Code non utilisé** : Les méthodes `find_by_id`, `list_by_order`, `list_by_client` de `PaymentRepository` sont implémentées mais pas encore utilisées (prévues pour les stories futures 3.4+).

### Completion Notes List

- Story 3.3 Payment Gate implémentée complètement
- Migration V5__payments.sql créée (table payments + indexes)
- db/payments.rs : PaymentRepository implémenté (create, find_by_id, list_by_order, list_by_client)
- api/payments.rs : handler POST /api/payments avec wallet (atomique) et cash
- api/mod.rs : PaymentRepository + SqlitePool dans AppApiState, route /api/payments
- lib.rs : DbPaymentRepository instancié, injecté dans AppApiState
- settings.rs + api/settings.rs : wallet_negative config key ajoutée
- cargo check passe sans erreur (17 warnings préexistants)

### Change Log

- **2026-07-23** -- Création du fichier story 3.3-payment-gate
- **2026-07-23** -- Implémentation complète de la story 3.3
  - Migration V5__payments.sql (payments avec indexes)
  - db/payments.rs : toutes les méthodes PaymentRepository implémentées
  - api/payments.rs : handler process_payment avec transaction atomique wallet + cash
  - api/mod.rs : PaymentRepository + SqlitePool dans AppApiState, route POST /api/payments
  - lib.rs : injection DbPaymentRepository dans AppApiState
  - settings.rs + api/settings.rs : wallet_negative config
  - cargo check passe sans erreur

### File List

- [x] `src-tauri/migrations/V5__payments.sql` — NEW
- [x] `src-tauri/src/db/payments.rs` — MODIFY
- [x] `src-tauri/src/api/payments.rs` — MODIFY
- [x] `src-tauri/src/api/mod.rs` — MODIFY
- [x] `src-tauri/src/lib.rs` — MODIFY
- [x] `src-tauri/src/settings.rs` — MODIFY
- [x] `src-tauri/src/api/settings.rs` — MODIFY
