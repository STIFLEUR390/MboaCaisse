# Acceptance Auditor — Code Review Prompt

You are an **Acceptance Auditor**. You have the full story spec and the diff. Your job is to verify that the implementation matches the spec — no more, no less.

## Your Mission

Compare the diff against every Acceptance Criterion in the story file. Flag:
- **Missing implementation**: ACs that are not addressed in the diff
- **Over-implementation**: Code that goes beyond what the spec asks for (speculative complexity)
- **Contract violations**: API responses, status codes, or error formats that deviate from the spec
- **Behavioral deviations**: Logic that contradicts an AC's Given/When/Then
- **Architecture violations**: Code that breaks documented ADs (AD-1, AD-3, AD-7, AD-8, AD-13, AD-15, AD-16)

## Spec File

```markdown
---
baseline_commit: 8d142b6
---

# Story 3.2: Cycle de Vie Commande

Status: review

## Story

As a caissier,
I want créer une commande, la faire passer par ses statuts (pending_payment → paid_preparing → ready → delivered),
So que la cuisine, le serveur et le client savent où en est la commande.

## Acceptance Criteria

### AC-1: Migration V4__orders.sql
**Given** la migration V4 est placée dans `src-tauri/migrations/V4__orders.sql`
**When** le runner refinery l'exécute
**Then** la table `orders` est créée :
- `id` TEXT PRIMARY KEY (UUID v7)
- `table_id` TEXT nullable
- `client_id` TEXT nullable
- `status` TEXT NOT NULL DEFAULT 'pending_payment'
- `total` INTEGER NOT NULL DEFAULT 0
- `created_at` TEXT NOT NULL
- `updated_at` TEXT NOT NULL
**And** la table `order_items` est créée :
- `id` TEXT PRIMARY KEY (UUID v7)
- `order_id` TEXT NOT NULL REFERENCES orders(id)
- `product_id` TEXT NOT NULL
- `quantity` INTEGER NOT NULL CHECK(quantity > 0)
- `unit_price` INTEGER NOT NULL (prix figé au moment de la création)
- `notes` TEXT nullable
- `created_at` TEXT NOT NULL
**And** un index sur `order_items(order_id)` pour les requêtes par commande
**And** un index sur `orders(status)` pour le filtrage cuisine

### AC-2: POST /api/orders — Création commande avec articles
**Given** des produits existants dans le catalogue
**When** POST `/api/orders` avec :
```json
{
  "table_id": "table-1",
  "client_id": "cli-xxx",
  "items": [
    { "product_id": "p1", "quantity": 2, "notes": "sans glace" },
    { "product_id": "p2", "quantity": 1 }
  ]
}
```
**Then** chaque `product_id` est vérifié (existence dans products)
**And** `unit_price` est lu depuis `products.price` au moment de la création (figé, pas de mise à jour dynamique)
**And** le total est calculé côté serveur : `SUM(unit_price * quantity)` — jamais depuis le client
**And** la commande est créée avec `status = "pending_payment"`
**And** retourne **201 Created** avec l'order + ses items + total calculé
**And** si `items` est vide → **400 Bad Request**
**And** si un `product_id` n'existe pas → **422 Unprocessable Entity** avec `{"error": "Product not found: {id}", "code": "PRODUCT_NOT_FOUND"}`
**And** si une `quantity <= 0` → **422 Unprocessable Entity** avec `{"error": "Invalid quantity", "code": "INVALID_QUANTITY"}`
**And** si `table_id` et `client_id` sont tous les deux absents → la commande est créée sans référence (commande interne valide)

### AC-3: GET /api/orders — Liste des commandes
**Given** des commandes existantes
**When** GET `/api/orders`
**Then** retourne **200** avec la liste complète des commandes triées par `created_at DESC`
**And** chaque commande inclut ses items dans un champ `items: [...]`
**When** GET `/api/orders?status=paid_preparing`
**Then** retourne **200** avec les commandes filtrées par ce statut
**When** GET `/api/orders?status=invalid_status`
**Then** retourne **400 Bad Request** avec `{"error": "Invalid status", "code": "INVALID_VALUE"}`

### AC-4: GET /api/orders/{id} — Détail commande
**Given** une commande existante
**When** GET `/api/orders/{id}`
**Then** retourne **200** avec la commande + ses items
**And** si l'ID n'existe pas → **404 Not Found**

### AC-5: PATCH /api/orders/{id}/status — Transition de statut
**Given** une commande en `pending_payment`
**When** PATCH `/api/orders/{id}/status` avec `{ "status": "paid_preparing" }`
**Then** la transition est validée via `OrderStatus::can_transition_to()`
**And** le statut est mis à jour
**And** `updated_at` est rafraîchi
**And** retourne **200 OK** avec la commande mise à jour

**Given** une commande en `pending_payment`
**When** PATCH avec `{ "status": "delivered" }` (saut de PaidPreparing et Ready)
**Then** retourne **422 Unprocessable Entity** avec `{"error": "Invalid status transition: pending_payment → delivered", "code": "INVALID_STATUS_TRANSITION"}`

**Given** une commande en `paid_preparing`
**When** PATCH avec `{ "status": "ready" }`
**Then** transition acceptée → **200 OK**

**Given** une commande en `ready`
**When** PATCH avec `{ "status": "delivered" }`
**Then** transition acceptée → **200 OK**

**Given** une commande inexistante
**When** PATCH `/api/orders/{id}/status`
**Then** **404 Not Found**

### AC-6: POST /api/orders/{id}/items — Ajouter un article
**Given** une commande en `pending_payment`
**When** POST `/api/orders/{id}/items` avec `{ "product_id": "p1", "quantity": 1, "notes": "extra" }`
**Then** le produit est vérifié (existence)
**And** le `unit_price` est lu depuis `products.price` au moment de l'ajout
**And** l'item est ajouté à la commande
**And** le total est recalculé : `total += unit_price * quantity`
**And** retourne **200 OK** avec l'item créé

**Given** une commande qui n'est pas en `pending_payment`
**When** POST `/api/orders/{id}/items`
**Then** retourne **422 Unprocessable Entity** avec `{"error": "Cannot modify order in status: paid_preparing", "code": "INVALID_ORDER_STATUS"}`

**Given** POST avec un `product_id` inexistant
**When** le produit n'existe pas
**Then** **422 Unprocessable Entity**

### AC-7: DELETE /api/orders/{id}/items/{item_id} — Supprimer un article
**Given** une commande en `pending_payment` avec un item existant
**When** DELETE `/api/orders/{id}/items/{item_id}`
**Then** l'item est supprimé
**And** le total est recalculé : `total -= unit_price * quantity`
**And** retourne **204 No Content**

**Given** une commande qui n'est pas en `pending_payment`
**When** DELETE `/api/orders/{id}/items/{item_id}`
**Then** **422 Unprocessable Entity**

**Given** un item_id inexistant
**When** DELETE avec un mauvais ID
**Then** **404 Not Found**

### AC-8: Idempotence et cohérence
**Given** une commande sans items
**When** GET `/api/orders/{id}`
**Then** le champ `items` est un tableau vide `[]`
**And** `total` est 0

**Given** un DELETE sur le dernier item d'une commande
**When** l'item est supprimé et total devient 0
**Then** la commande existe toujours (total = 0), pas de suppression cascade

## Tasks / Subtasks

- [x] **AC-1** Créer la migration SQL V4__orders.sql
  - [x] Table `orders` avec tous les champs + contraintes
  - [x] Table `order_items` avec FK + CHECK quantité
  - [x] Index sur `order_items(order_id)`, `orders(status)`
- [x] **AC-2** Implémenter `api/orders.rs` — POST /api/orders
  - [x] Handler `create_order` avec validation items
  - [x] Vérification existence produits (boucle + map lookup)
  - [x] Calcul total côté serveur
  - [x] Gestion erreurs : 400, 422, 201
- [x] **AC-3, AC-4** Implémenter GET /api/orders et GET /api/orders/{id}
  - [x] Handler `list_orders` avec filtre status optionnel
  - [x] Handler `get_order` avec items
  - [x] Gestion erreurs : 200, 400 (invalid status), 404
- [x] **AC-5** Implémenter PATCH /api/orders/{id}/status
  - [x] Handler `update_order_status` avec validation domaine
  - [x] Utiliser `Order::transition_to()` existant dans domain
  - [x] Gestion erreurs : 200, 404, 422
- [x] **AC-6** Implémenter POST /api/orders/{id}/items
  - [x] Handler `add_order_item` avec vérification statut
  - [x] Recalcul total après ajout
  - [x] Gestion erreurs : 200, 404, 422
- [x] **AC-7** Implémenter DELETE /api/orders/{id}/items/{item_id}
  - [x] Handler `remove_order_item` avec vérification statut
  - [x] Recalcul total après suppression
  - [ ] Gestion erreurs : 204, 404, 422
- [x] **AC-8** Tester les cas limites
  - [x] Commande sans items (items = [])
  - [x] Dernier item supprimé (commande existe toujours)
- [x] **Routes + State** Intégration dans api/mod.rs et lib.rs
  - [x] Ajouter `order_repo` dans `AppApiState`
  - [x] Ajouter `DbOrderRepository` dans `lib.rs` (instanciation)
  - [x] Monter routes orders dans `build_app()`
  - [x] Ajouter routes sous middleware JWT
- [x] Validation
  - [x] `cargo check` passe sans erreur
  - [x] Test curl POST/GET/PATCH/DELETE

## Dev Notes

### Architecture Compliance

- **AD-1** (Layered + Rich Domain) : `api/orders.rs` = peau fine (parse → appelle domain → sérialise). `domain/order.rs` contient déjà `Order::transition_to()` et `OrderStatus::can_transition_to()`. `db/orders.rs` implémente le trait OrderRepository.
- **AD-3** (Structure plate) : Tout dans `api/orders.rs`, `domain/order.rs`, `db/orders.rs`. Pas de sous-dossiers.
- **AD-7** (Traits dans domain) : `OrderRepository` trait déjà défini dans `domain/order.rs`. Ne pas le déplacer ni le modifier.
- **AD-8** (Erreurs 3 couches) :
  - `db/` → `DbError` (ne sort pas)
  - `domain/` → `DomainError` (déjà implémenté : `InvalidStatusTransition`, `NotFound`, `InvalidValue`)
  - `api/` → `(StatusCode, Json<ApiError>)` avec format standardisé `{"error": "...", "code": "..."}`
  - Ajouter les cas manquants dans `domain_to_http()` : `InsufficientBalance` → 422, `DuplicatePhone` → 409
- **AD-10** (Stack alpha) : UUID v7, chrono pour dates, rusqlite paramétré.
- **AD-13** (Graphe dépendances) : Order dépend de Catalog (vérification produits par `ProductRepository`). Order ne dépend PAS encore de Wallet dans cette story (le paiement est story 3.3). `OrderRepository` est indépendant.
- **AD-15** (Migrations refinery) : V4__orders.sql, exécutée automatiquement par le runner existant.
- **AD-16** (Pool r2d2) : `DbOrderRepository::new(pool)` utilise le pool existant.

### API Contract

Toutes les routes sont sous `/api/*` et protégées par le middleware JWT existant (`auth_middleware.rs`).

| Méthode | Route | Status | Description |
|---------|-------|--------|-------------|
| POST | /api/orders | 201 | Créer une commande avec articles |
| GET | /api/orders | 200 | Liste commandes (optionnel ?status=) |
| GET | /api/orders/{id} | 200 | Détail commande avec items |
| PATCH | /api/orders/{id}/status | 200 | Transition de statut |
| POST | /api/orders/{id}/items | 200 | Ajouter article à une commande |
| DELETE | /api/orders/{id}/items/{item_id} | 204 | Supprimer article |

Format réponse erreur : `{"error": "message", "code": "SCREAMING_SNAKE"}`

Codes d'erreur à utiliser dans cette story :
- `ORDER_NOT_FOUND` — 404
- `PRODUCT_NOT_FOUND` — 422 (quand un product_id est invalide)
- `INVALID_QUANTITY` — 422 (quantity ≤ 0)
- `INVALID_STATUS_TRANSITION` — 422 (transition non autorisée)
- `INVALID_ORDER_STATUS` — 422 (tentative de modifier une commande non modifiable)
- `INVALID_VALUE` — 400 (status invalide dans le filtre)
- `VALIDATION_ERROR` — 400 (items vides)

### Existing Code — State & What to Preserve

**`domain/order.rs`** — déjà complet avec Order, OrderItem, OrderStatus, OrderRepository trait.
- ✅ `Order` struct avec tous les champs requis
- ✅ `OrderItem` struct avec id, order_id, product_id, quantity, unit_price, notes
- ✅ `OrderStatus` enum avec 4 variants + `can_transition_to()` + `from_str()` + `as_str()`
- ✅ `Order::new(id, table_id, client_id, created_at)` crée une commande en PendingPayment
- ✅ `Order::transition_to(new_status)` valide et applique la transition
- ✅ `OrderRepository` trait complet avec : create, update_status, find_by_id, list_by_status, list_all, add_item, get_items, remove_item
- ⚠️ **Méthode manquante** : `update_total(order_id, total)` dans le trait — nécessaire pour le recalcul du total après ajout/suppression d'items. **À AJOUTER** dans le trait + impl.
- ⚠️ **Méthode manquante** : `delete_item` existe déjà (`remove_item`). OK.
- ⚠️ **Méthode manquante** : `list_by_status` existe déjà. OK.

**`db/orders.rs`** — squelette avec `todo!("Story 3.2")` partout.
- ❌ Toutes les méthodes sont à implémenter
- ✅ Structure `DbOrderRepository::new(pool)` déjà en place
- ⚠️ `OrderRepository` trait manque `update_total` — ajouter avant implémentation db

**`api/orders.rs`** — commentaire uniquement.
- ❌ Remplacer par des handlers complets

**`api/mod.rs`** — build_app()
- ❌ Routes orders à ajouter
- ❌ `order_repo` dans `AppApiState`

**`lib.rs`** — setup()
- ❌ Instanciation `DbOrderRepository` + ajout à `AppApiState`

**`db/migration_wallet.rs`** — migration des commandes payées vers wallet_ledger.
- ✅ Déjà existant, s'exécute après les migrations normales dans `lib.rs`
- ✅ Vérifie si la table `orders` existe avant de tenter la migration (no-op pré-Epic 3)
- ✅ Idempotent : ne rejoue pas si des entrées 'migration' existent déjà

### State Extraction Pattern

Pour extraire l'état (State) dans `api/orders.rs`, suivre le même pattern que `api/products.rs` :

```rust
#[derive(Clone)]
pub struct OrdersState {
    pub order_repo: Arc<dyn OrderRepository>,
    pub product_repo: Arc<dyn ProductRepository>,
}

impl FromRef<AppApiState> for OrdersState {
    fn from_ref(state: &AppApiState) -> Self {
        Self {
            order_repo: state.order_repo.clone(),
            product_repo: state.product_repo.clone(),
        }
    }
}
```

⚠️ `OrdersState` nécessite **deux** repositories : `OrderRepository` + `ProductRepository` (pour vérifier l'existence des produits dans POST /api/orders et POST /api/orders/{id}/items). Ne pas oublier d'ajouter `product_repo` dans le `FromRef`.

### Prix figé (unit_price)

Le `unit_price` dans `order_items` est lu depuis `products.price` au moment de la création de l'item. Il n'est **jamais** mis à jour si le prix du produit change ultérieurement. Ceci est un choix d'architecture :
- L'ordre reflète le prix au moment de la commande
- Pas d'ambiguïté sur le montant facturé
- Pattern standard dans les POS

### Modèle de réponse Order

```json
{
  "id": "0190...",
  "table_id": "table-1",
  "client_id": "cli-xxx",
  "status": "pending_payment",
  "total": 3500,
  "created_at": "2026-07-23T10:30:00.000Z",
  "updated_at": "2026-07-23T10:30:00.000Z",
  "items": [
    {
      "id": "0190...",
      "product_id": "p1",
      "quantity": 2,
      "unit_price": 1500,
      "notes": "sans glace",
      "created_at": "2026-07-23T10:30:00.000Z"
    }
  ]
}
```

### Gotchas & Écueils

- ⚠️ **Ordre de routes Axum** : Les routes dynamiques comme `/api/orders/{id}/status` et `/api/orders/{id}/items/{item_id}` doivent être déclarées AVANT `/api/orders/{id}` pour éviter que `{id}` capture `status` ou `items`. Dans Axum, l'ordre de déclaration dans le routeur détermine la priorité. Exemple :
  ```rust
  .route("/api/orders/{id}/status", patch(...))
  .route("/api/orders/{id}/items/{item_id}", delete(...))
  .route("/api/orders/{id}", get(...))
  ```
- ⚠️ **Recalcul du total** : Après chaque `add_item` ou `remove_item`, le total doit être mis à jour dans la table `orders`. Idéalement dans la même transaction que l'INSERT/DELETE sur `order_items`. Utiliser un UPDATE `SET total = (SELECT COALESCE(SUM(quantity * unit_price), 0) FROM order_items WHERE order_id = ?)` pour éviter les erreurs de calcul.
- ⚠️ **ProductRepository nécessaire dans OrdersState** : Le handler POST /api/orders doit vérifier que chaque `product_id` existe dans `products`. Pour cela, il faut que `OrdersState` ait accès à `product_repo` en plus de `order_repo`. Voir le pattern `FromRef<AppApiState>` ci-dessus.
- ⚠️ **Ne pas modifier le trait OrderRepository** sauf pour `update_total` : Le trait a déjà toutes les méthodes nécessaires sauf `update_total`. Ajouter cette méthode au trait ET à l'implémentation. Vérifier que `db/orders.rs` n'a pas de conflit.
- ⚠️ **Même pattern que products** : Suivre le même pattern que `api/products.rs`. Les handlers prennent `State<OrdersState>` et retournent `impl IntoResponse`. Utiliser `domain_to_http()` pour les erreurs domaine.
- ⚠️ **Migration V4** : Le fichier doit s'appeler `V4__orders.sql` (deux underscores après le numéro). C'est le format que refinery attend.
- ⚠️ **V5 dans l'epics** : L'epics.md mentionne `V5__orders.sql` mais la dernière migration existante est `V3__products.sql`. Utiliser **V4** (prochain numéro disponible). Pas de V4 manquante — `db/migration_wallet.rs` est un script inline, pas une migration refinery. Si un V4 conflictuel existait, le runner refinery planterait.

### Previous Story Intelligence (Story 3.1)

**Leçons de 3.1 (CRUD Produits & Catégories) :**

- ❌ **Validation existence category_id** : Dans les handlers POST/PUT products, la vérification d'existence de `category_id` a été oubliée initialement, causant une 500 au lieu de 422. **Applicable ici** : Toujours vérifier l'existence de `product_id` dans POST /api/orders et POST /api/orders/{id}/items.
- ❌ **Validation domaine stock < 0** : Les validateurs de domaine ont été ajoutés après review. **Applicable ici** : Valider `quantity > 0` dans le domaine (`OrderItem`) avant de passer à la DB.
- ❌ **PUT écrase stock/alert_threshold** : Changé en PATCH semantics pour préserver les valeurs non fournies. **Applicable ici** : PATCH /api/orders/{id}/status ne modifie QUE le status — pas besoin de body complet.
- ✅ **Code review fixes** : Tous les correctifs de review ont été appliqués (validation existence FK, wrapping transaction seed, timestamps par ligne). Suivre les mêmes patterns de qualité.
- ✅ **Pattern de handlers** : Les handlers dans `api/products.rs` sont la référence pour `api/orders.rs` : `State<XxxState>`, `impl IntoResponse`, `domain_to_http()`.

### File List

**NOUVEAUX :**
- `src-tauri/migrations/V4__orders.sql` — Migration SQL orders + order_items

**MODIFIÉS :**
- `src-tauri/src/domain/order.rs` — Ajouter `update_total(order_id: &str, total: i64)` au trait `OrderRepository`
- `src-tauri/src/db/orders.rs` — Implémentation complète OrderRepository
- `src-tauri/src/api/orders.rs` — Handlers CRUD complets + OrdersState
- `src-tauri/src/api/mod.rs` — Ajouter `order_repo` dans AppApiState + monter routes orders
- `src-tauri/src/lib.rs` — Instancier `DbOrderRepository` + ajouter à `AppApiState`

**SUPPRIMÉS :** Aucun

### Testing Validation

- `cargo check` passe sans erreur
- Les migrations s'exécutent sans erreur au démarrage
- La table `order_items` a bien la FK vers `orders(id)`
- `db/migration_wallet.rs` reste fonctionnel (vérifie si orders table existe)
- Test curl :
  ```sh
  # Créer une commande
  curl -X POST http://localhost:3000/api/orders \
    -H 'Content-Type: application/json' \
    -H 'Cookie: mboa_session=...' \
    -d '{"items":[{"product_id":"<id>","quantity":2}]}'

  # Lire la commande
  curl http://localhost:3000/api/orders/<id> \
    -H 'Cookie: mboa_session=...'

  # Transition status
  curl -X PATCH http://localhost:3000/api/orders/<id>/status \
    -H 'Content-Type: application/json' \
    -H 'Cookie: mboa_session=...' \
    -d '{"status":"paid_preparing"}'
  ```

### References

- [Source: epics.md#Story-3.2] — Définition originale de la story avec AC
- [Source: ARCHITECTURE-SPINE.md#AD-1] — Paradigme Layered + Rich Domain
- [Source: ARCHITECTURE-SPINE.md#AD-7] — Traits dans domain/, impl dans db/
- [Source: ARCHITECTURE-SPINE.md#AD-8] — Erreurs 3 couches
- [Source: ARCHITECTURE-SPINE.md#AD-13] — Graphe dépendances (Order → Catalog)
- [Source: ARCHITECTURE-SPINE.md#AD-15] — Migrations refinery
- [Source: ARCHITECTURE-SPINE.md#AD-16] — Pool r2d2
- [Source: sprint-status.yaml] — Story identifiée comme backlog dans Epic 3
- [Source: .ai-memory/index.md] — Gotchas refinery 0.9, conventions UUID v7
- [Source: 3-1-crud-produits-categories.md] — Pattern handlers, leçons de review, convention ApiError/domain_to_http
- [Source: api/products.rs] — Pattern FromRef<AppApiState>, domain_to_http(), handlers

## Dev Agent Record

### Agent Model Used

bmad-create-story via GPT-5 (Codex)

### Debug Log References

- **Migration V4** : L'epics original mentionne V5 mais le prochain numéro disponible est V4 (V3 étant products). Vérifier qu'aucune V4 n'existe dans `migrations/` avant de créer.
- **OrderRepository trait** : Vérifier que `update_total(order_id, total)` est ajouté au trait AVANT l'implémentation db. Sans cette méthode, le recalcul du total après add/remove item n'est pas possible.
- **AppApiState dans api/mod.rs** : L'état actuel contient `user_repo`, `wallet_repo`, `product_repo`, `jwt_secret`. Ajouter `order_repo: Arc<dyn OrderRepository>`. Le `payment_repo` (nécessaire pour story 3.3) peut être ajouté maintenant ou dans la story suivante.
- **OrdersState** : A besoin de `order_repo` (OrderRepository) + `product_repo` (ProductRepository). Le `FromRef<AppApiState>` permet d'extraire `product_repo` depuis l'AppApiState principal.
- **Routes order Axum** : Déclarer les routes spécifiques AVANT les routes génériques (`{id}/status` avant `{id}`). Voir section gotchas.

### Completion Notes List


### Completion Notes List

- Migration V4__orders.sql créée avec tables orders + order_items + indexes
- Trait OrderRepository étendu avec update_total(order_id)
- db/orders.rs : implémentation complète des 9 méthodes du trait
- api/orders.rs : 6 handlers CRUD (create, list, get, update_status, add_item, remove_item)
- api/mod.rs : order_repo dans AppApiState + routes orders montées
- lib.rs : DbOrderRepository instancié et injecté dans AppApiState
- Domain enrichi : created_at ajouté à OrderItem
- cargo check passe sans erreur

### Change Log
- **2026-07-23** -- Implementation complète de la story 3.2
  - Migration V4__orders.sql (orders + order_items avec indexes)
  - Trait OrderRepository : ajout de update_total()
  - db/orders.rs : toutes les méthodes OrderRepository implémentées
  - api/orders.rs : handlers CRUD complets (create/list/get/status/items)
  - api/mod.rs : order_repo dans AppApiState + routes ordonnées
  - lib.rs : injection DbOrderRepository dans l'état global
  - Domain : created_at ajouté à OrderItem
  - cargo check passe sans erreur

### Change Log
```

## Diff

```diff
diff --git a/src-tauri/migrations/V4__orders.sql b/src-tauri/migrations/V4__orders.sql
new file mode 100644
index 0000000..3df5c08
--- /dev/null
+++ b/src-tauri/migrations/V4__orders.sql
@@ -0,0 +1,39 @@
+-- V4__orders.sql
+-- Order lifecycle: orders and order_items tables.
+--
+-- AD-13: Order depends on Catalog (product_id FK conceptual, no FK constraint
+--         to avoid blocking product deletion). Referential integrity is
+--         enforced at the application layer via ProductRepository lookups.
+-- AD-2:  order_items is mutable (add/remove items allowed in PendingPayment).
+--         Once past PendingPayment, mutability is gated by the domain layer.
+--         Financial mutation is NOT in this table -- wallet_ledger (V2) is
+--         the append-only financial record.
+
+CREATE TABLE IF NOT EXISTS orders (
+    id          TEXT PRIMARY KEY,
+    table_id    TEXT,
+    client_id   TEXT,
+    status      TEXT NOT NULL DEFAULT 'pending_payment',
+    total       INTEGER NOT NULL DEFAULT 0,
+    created_at  TEXT NOT NULL,
+    updated_at  TEXT NOT NULL
+);
+
+CREATE TABLE IF NOT EXISTS order_items (
+    id          TEXT PRIMARY KEY,
+    order_id    TEXT NOT NULL REFERENCES orders(id),
+    product_id  TEXT NOT NULL,
+    quantity    INTEGER NOT NULL CHECK(quantity > 0),
+    unit_price  INTEGER NOT NULL,
+    notes       TEXT,
+    created_at  TEXT NOT NULL
+);
+
+-- Index for retrieving items by order
+CREATE INDEX IF NOT EXISTS idx_order_items_order_id ON order_items(order_id);
+
+-- Index for filtering orders by status (kitchen display, etc.)
+CREATE INDEX IF NOT EXISTS idx_orders_status ON orders(status);
+
+-- Index for lookups by table
+CREATE INDEX IF NOT EXISTS idx_orders_table_id ON orders(table_id);
diff --git a/src-tauri/src/api/mod.rs b/src-tauri/src/api/mod.rs
index d7e5885..4162923 100644
--- a/src-tauri/src/api/mod.rs
+++ b/src-tauri/src/api/mod.rs
@@ -23,6 +23,7 @@ use axum::{
 use tauri::AppHandle;
 
 use crate::domain::product::ProductRepository;
+use crate::domain::order::OrderRepository;
 use crate::domain::user::UserRepository;
 use crate::domain::wallet::WalletRepository;
 
@@ -47,6 +48,7 @@ pub fn app_handle() -> &'static AppHandle {
 #[derive(Clone)]
 pub struct AppApiState {
 	pub user_repo: Arc<dyn UserRepository>,
+	pub order_repo: Arc<dyn OrderRepository>,
 	pub wallet_repo: Arc<dyn WalletRepository>,
 	pub product_repo: Arc<dyn ProductRepository>,
 	pub jwt_secret: Arc<Vec<u8>>,
@@ -109,7 +111,15 @@ pub fn build_app(state: AppApiState) -> Router {
 		.route(
 			"/api/categories/{id}",
 			delete(products::delete_category),
-		);
+		)
+		// Orders CRUD (story 3.2)
+		.route("/api/orders", post(orders::create_order))
+		.route("/api/orders", get(orders::list_orders))
+		.route("/api/orders/{id}/status", patch(orders::update_order_status))
+		.route("/api/orders/{id}/items", post(orders::add_order_item))
+		.route("/api/orders/{id}/items/{item_id}", delete(orders::remove_order_item))
+		.route("/api/orders/{id}", get(orders::get_order))
+		;
 
 	// Static file serving with SPA fallback.
 	if std::path::Path::new(&dist_path).exists() {
diff --git a/src-tauri/src/api/orders.rs b/src-tauri/src/api/orders.rs
index 8bfbacc..2384ad0 100644
--- a/src-tauri/src/api/orders.rs
+++ b/src-tauri/src/api/orders.rs
@@ -1,3 +1,516 @@
-//! Orders API — CRUD orders, status transitions.
-//! AD-13: Order depends on Catalog + Wallet.
-//! Story 3.2.
\ No newline at end of file
+//! Orders API — CRUD orders, status transitions, item management.
+//!
+//! Story 3.2. Handles /api/orders/*.
+//! AD-1: Thin API layer — delegates to domain via OrderRepository + ProductRepository.
+//! AD-8: Returns (StatusCode, Json<ApiError>) with standardized error format.
+//! AD-13: Order depends on Catalog (product lookups for validation).
+
+use std::sync::Arc;
+
+use axum::{
+	extract::{FromRef, Path, Query, State},
+	http::StatusCode,
+	response::IntoResponse,
+	Json,
+};
+use serde::{Deserialize, Serialize};
+
+use crate::domain::order::{Order, OrderItem, OrderRepository, OrderStatus};
+use crate::domain::product::ProductRepository;
+use crate::domain::DomainError;
+
+use super::AppApiState;
+
+// ─── State extraction ───────────────────────────────────────────────
+
+#[derive(Clone)]
+pub struct OrdersState {
+	pub order_repo: Arc<dyn OrderRepository>,
+	pub product_repo: Arc<dyn ProductRepository>,
+}
+
+impl FromRef<AppApiState> for OrdersState {
+	fn from_ref(state: &AppApiState) -> Self {
+		Self {
+			order_repo: state.order_repo.clone(),
+			product_repo: state.product_repo.clone(),
+		}
+	}
+}
+
+// ─── Request / Response types ───────────────────────────────────────
+
+#[derive(Deserialize)]
+pub struct CreateOrderItem {
+	pub product_id: String,
+	pub quantity: i64,
+	#[serde(default)]
+	pub notes: Option<String>,
+}
+
+#[derive(Deserialize)]
+pub struct CreateOrderRequest {
+	#[serde(default)]
+	pub table_id: Option<String>,
+	#[serde(default)]
+	pub client_id: Option<String>,
+	pub items: Vec<CreateOrderItem>,
+}
+
+#[derive(Deserialize)]
+pub struct UpdateStatusRequest {
+	pub status: String,
+}
+
+#[derive(Deserialize)]
+pub struct AddItemRequest {
+	pub product_id: String,
+	pub quantity: i64,
+	#[serde(default)]
+	pub notes: Option<String>,
+}
+
+#[derive(Deserialize)]
+pub struct OrderListQuery {
+	pub status: Option<String>,
+}
+
+#[derive(Serialize)]
+pub struct OrderItemResponse {
+	pub id: String,
+	pub order_id: String,
+	pub product_id: String,
+	pub quantity: i64,
+	pub unit_price: i64,
+	pub notes: Option<String>,
+	pub created_at: String,
+}
+
+impl From<OrderItem> for OrderItemResponse {
+	fn from(i: OrderItem) -> Self {
+		Self {
+			id: i.id,
+			order_id: i.order_id,
+			product_id: i.product_id,
+			quantity: i.quantity,
+			unit_price: i.unit_price,
+			notes: i.notes,
+			created_at: i.created_at,
+		}
+	}
+}
+
+#[derive(Serialize)]
+pub struct OrderResponse {
+	pub id: String,
+	pub table_id: Option<String>,
+	pub client_id: Option<String>,
+	pub status: String,
+	pub total: i64,
+	pub created_at: String,
+	pub updated_at: String,
+	pub items: Vec<OrderItemResponse>,
+}
+
+#[derive(Serialize)]
+pub struct ApiError {
+	pub error: String,
+	pub code: String,
+}
+
+// ─── Error helpers ──────────────────────────────────────────────────
+
+fn error_response(error: &str, code: &str, status: StatusCode) -> (StatusCode, Json<ApiError>) {
+	(status, Json(ApiError {
+		error: error.to_string(),
+		code: code.to_string(),
+	}))
+}
+
+fn domain_to_http(err: DomainError) -> (StatusCode, Json<ApiError>) {
+	match err {
+		DomainError::Unauthorized => (
+			StatusCode::UNAUTHORIZED,
+			Json(ApiError {
+				error: "Unauthorized".into(),
+				code: "UNAUTHORIZED".into(),
+			}),
+		),
+		DomainError::NotFound(msg) => (
+			StatusCode::NOT_FOUND,
+			Json(ApiError {
+				error: msg,
+				code: "NOT_FOUND".into(),
+			}),
+		),
+		DomainError::InvalidValue(msg) => (
+			StatusCode::BAD_REQUEST,
+			Json(ApiError {
+				error: msg,
+				code: "INVALID_VALUE".into(),
+			}),
+		),
+		DomainError::InvalidStatusTransition { from, to } => (
+			StatusCode::UNPROCESSABLE_ENTITY,
+			Json(ApiError {
+				error: format!("Invalid status transition: {} → {}", from, to),
+				code: "INVALID_STATUS_TRANSITION".into(),
+			}),
+		),
+		_ => (
+			StatusCode::INTERNAL_SERVER_ERROR,
+			Json(ApiError {
+				error: "Internal server error".into(),
+				code: "INTERNAL_ERROR".into(),
+			}),
+		),
+	}
+}
+
+fn uuid_v7() -> String {
+	use uuid::Uuid;
+	Uuid::now_v7().to_string()
+}
+
+fn chrono_now() -> String {
+	use chrono::Utc;
+	Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
+}
+
+// ─── Handlers ───────────────────────────────────────────────────────
+
+/// POST /api/orders — Create an order with items.
+///
+/// AC-2: Validates product existence, calculates total server-side.
+pub async fn create_order(
+	State(state): State<OrdersState>,
+	Json(req): Json<CreateOrderRequest>,
+) -> impl IntoResponse {
+	// Validate items not empty
+	if req.items.is_empty() {
+		return error_response("Items list must not be empty", "VALIDATION_ERROR", StatusCode::BAD_REQUEST)
+			.into_response();
+	}
+
+	// Validate products exist and get prices
+	let mut resolved_items: Vec<(String, i64, Option<String>)> = Vec::new();
+	for item in &req.items {
+		if item.quantity <= 0 {
+			return error_response("Invalid quantity", "INVALID_QUANTITY", StatusCode::UNPROCESSABLE_ENTITY)
+				.into_response();
+		}
+		match state.product_repo.find_product_by_id(&item.product_id) {
+			Ok(Some(product)) => {
+				resolved_items.push((item.product_id.clone(), product.price, item.notes.clone()));
+			}
+			Ok(None) => {
+				return error_response(
+					&format!("Product not found: {}", item.product_id),
+					"PRODUCT_NOT_FOUND",
+					StatusCode::UNPROCESSABLE_ENTITY,
+				)
+					.into_response();
+			}
+			Err(e) => return domain_to_http(e).into_response(),
+		}
+	}
+
+	let now = chrono_now();
+	let order_id = uuid_v7();
+
+	// Calculate total server-side
+	// Recalculate properly
+	let total: i64 = resolved_items.iter()
+		.enumerate()
+		.map(|(i, (_, price, _))| price * req.items[i].quantity)
+		.sum();
+
+	let order = Order::new(order_id.clone(), req.table_id, req.client_id, now.clone());
+
+	// Persist order
+	if let Err(e) = state.order_repo.create(&order) {
+		return domain_to_http(e).into_response();
+	}
+
+	// Persist items
+	let mut order_items: Vec<OrderItem> = Vec::new();
+	for (i, (product_id, unit_price, notes)) in resolved_items.iter().enumerate() {
+		let item_id = uuid_v7();
+		let item = OrderItem {
+			id: item_id,
+			order_id: order_id.clone(),
+			product_id: product_id.clone(),
+			quantity: req.items[i].quantity,
+			unit_price: *unit_price,
+			notes: notes.clone(),
+			created_at: now.clone(),
+		};
+		if let Err(e) = state.order_repo.add_item(&item) {
+			// Ignore cleanup error — log would go here
+			let _ = state.order_repo.remove_item(&item.id);
+			return domain_to_http(e).into_response();
+		}
+		order_items.push(item);
+	}
+
+	// Update total
+	if let Err(e) = state.order_repo.update_total(&order_id) {
+		return domain_to_http(e).into_response();
+	}
+
+	let response = OrderResponse {
+		id: order_id,
+		table_id: order.table_id,
+		client_id: order.client_id,
+		status: order.status.as_str().to_string(),
+		total,
+		created_at: order.created_at,
+		updated_at: order.updated_at,
+		items: order_items.into_iter().map(Into::into).collect(),
+	};
+
+	(StatusCode::CREATED, Json(response)).into_response()
+}
+
+/// GET /api/orders — List orders, optionally filtered by status.
+///
+/// AC-3: Returns orders sorted by created_at DESC, each with items.
+pub async fn list_orders(
+	State(state): State<OrdersState>,
+	Query(query): Query<OrderListQuery>,
+) -> impl IntoResponse {
+	let orders: Vec<Order> = if let Some(ref status_str) = query.status {
+		let status = match OrderStatus::from_str(status_str) {
+			Ok(s) => s,
+			Err(_) => {
+				return error_response(
+					&format!("Invalid status: {}", status_str),
+					"INVALID_VALUE",
+					StatusCode::BAD_REQUEST,
+				)
+					.into_response();
+			}
+		};
+		match state.order_repo.list_by_status(&status) {
+			Ok(orders) => orders,
+			Err(e) => return domain_to_http(e).into_response(),
+		}
+	} else {
+		match state.order_repo.list_all() {
+			Ok(orders) => orders,
+			Err(e) => return domain_to_http(e).into_response(),
+		}
+	};
+
+	// Enrich each order with items
+	let mut responses: Vec<OrderResponse> = Vec::with_capacity(orders.len());
+	for order in orders {
+		let items = match state.order_repo.get_items(&order.id) {
+			Ok(items) => items.into_iter().map(Into::into).collect(),
+			Err(e) => return domain_to_http(e).into_response(),
+		};
+		responses.push(OrderResponse {
+			id: order.id,
+			table_id: order.table_id,
+			client_id: order.client_id,
+			status: order.status.as_str().to_string(),
+			total: order.total,
+			created_at: order.created_at,
+			updated_at: order.updated_at,
+			items,
+		});
+	}
+
+	(StatusCode::OK, Json(responses)).into_response()
+}
+
+/// GET /api/orders/{id} — Get order details with items.
+///
+/// AC-4: Returns 200 with order + items, or 404 if not found.
+pub async fn get_order(
+	State(state): State<OrdersState>,
+	Path(id): Path<String>,
+) -> impl IntoResponse {
+	let order = match state.order_repo.find_by_id(&id) {
+		Ok(Some(order)) => order,
+		Ok(None) => {
+			return error_response("Order not found", "ORDER_NOT_FOUND", StatusCode::NOT_FOUND)
+				.into_response();
+		}
+		Err(e) => return domain_to_http(e).into_response(),
+	};
+
+	let items = match state.order_repo.get_items(&id) {
+		Ok(items) => items.into_iter().map(Into::into).collect(),
+		Err(e) => return domain_to_http(e).into_response(),
+	};
+
+	(StatusCode::OK, Json(OrderResponse {
+		id: order.id,
+		table_id: order.table_id,
+		client_id: order.client_id,
+		status: order.status.as_str().to_string(),
+		total: order.total,
+		created_at: order.created_at,
+		updated_at: order.updated_at,
+		items,
+	})).into_response()
+}
+
+/// PATCH /api/orders/{id}/status — Transition order status.
+///
+/// AC-5: Validates transitions via Order::transition_to().
+pub async fn update_order_status(
+	State(state): State<OrdersState>,
+	Path(id): Path<String>,
+	Json(req): Json<UpdateStatusRequest>,
+) -> impl IntoResponse {
+	let mut order = match state.order_repo.find_by_id(&id) {
+		Ok(Some(order)) => order,
+		Ok(None) => {
+			return error_response("Order not found", "ORDER_NOT_FOUND", StatusCode::NOT_FOUND)
+				.into_response();
+		}
+		Err(e) => return domain_to_http(e).into_response(),
+	};
+
+	let new_status = match OrderStatus::from_str(&req.status) {
+		Ok(s) => s,
+		Err(e) => return domain_to_http(e).into_response(),
+	};
+
+	if let Err(e) = order.transition_to(new_status) {
+		return domain_to_http(e).into_response();
+	}
+
+	if let Err(e) = state.order_repo.update_status(&id, &order.status) {
+		return domain_to_http(e).into_response();
+	}
+
+	let items = match state.order_repo.get_items(&id) {
+		Ok(items) => items.into_iter().map(Into::into).collect(),
+		Err(_) => vec![],
+	};
+
+	(StatusCode::OK, Json(OrderResponse {
+		id: order.id,
+		table_id: order.table_id,
+		client_id: order.client_id,
+		status: order.status.as_str().to_string(),
+		total: order.total,
+		created_at: order.created_at,
+		updated_at: order.updated_at,
+		items,
+	})).into_response()
+}
+
+/// POST /api/orders/{id}/items — Add an item to an existing order.
+///
+/// AC-6: Only allowed in PendingPayment status.
+pub async fn add_order_item(
+	State(state): State<OrdersState>,
+	Path(id): Path<String>,
+	Json(req): Json<AddItemRequest>,
+) -> impl IntoResponse {
+	if req.quantity <= 0 {
+		return error_response("Invalid quantity", "INVALID_QUANTITY", StatusCode::UNPROCESSABLE_ENTITY)
+			.into_response();
+	}
+
+	// Check order exists and is in PendingPayment
+	let order = match state.order_repo.find_by_id(&id) {
+		Ok(Some(order)) => order,
+		Ok(None) => {
+			return error_response("Order not found", "ORDER_NOT_FOUND", StatusCode::NOT_FOUND)
+				.into_response();
+		}
+		Err(e) => return domain_to_http(e).into_response(),
+	};
+
+	if order.status != OrderStatus::PendingPayment {
+		return error_response(
+			&format!("Cannot modify order in status: {}", order.status.as_str()),
+			"INVALID_ORDER_STATUS",
+			StatusCode::UNPROCESSABLE_ENTITY,
+		)
+			.into_response();
+	}
+
+	// Verify product exists
+	let unit_price = match state.product_repo.find_product_by_id(&req.product_id) {
+		Ok(Some(product)) => product.price,
+		Ok(None) => {
+			return error_response(
+				&format!("Product not found: {}", req.product_id),
+				"PRODUCT_NOT_FOUND",
+				StatusCode::UNPROCESSABLE_ENTITY,
+			)
+				.into_response();
+		}
+		Err(e) => return domain_to_http(e).into_response(),
+	};
+
+	let now = chrono_now();
+	let item = OrderItem {
+		id: uuid_v7(),
+		order_id: id.clone(),
+		product_id: req.product_id,
+		quantity: req.quantity,
+		unit_price,
+		notes: req.notes,
+		created_at: now,
+	};
+
+	if let Err(e) = state.order_repo.add_item(&item) {
+		return domain_to_http(e).into_response();
+	}
+
+	// Recalculate total
+	if let Err(e) = state.order_repo.update_total(&id) {
+		return domain_to_http(e).into_response();
+	}
+
+	(StatusCode::OK, Json(OrderItemResponse::from(item))).into_response()
+}
+
+/// DELETE /api/orders/{id}/items/{item_id} — Remove an item from an order.
+///
+/// AC-7: Only allowed in PendingPayment status. Recalculates total after removal.
+pub async fn remove_order_item(
+	State(state): State<OrdersState>,
+	Path((id, item_id)): Path<(String, String)>,
+) -> impl IntoResponse {
+	// Check order exists and is in PendingPayment
+	let order = match state.order_repo.find_by_id(&id) {
+		Ok(Some(order)) => order,
+		Ok(None) => {
+			return error_response("Order not found", "ORDER_NOT_FOUND", StatusCode::NOT_FOUND)
+				.into_response();
+		}
+		Err(e) => return domain_to_http(e).into_response(),
+	};
+
+	if order.status != OrderStatus::PendingPayment {
+		return error_response(
+			&format!("Cannot modify order in status: {}", order.status.as_str()),
+			"INVALID_ORDER_STATUS",
+			StatusCode::UNPROCESSABLE_ENTITY,
+		)
+			.into_response();
+	}
+
+	if let Err(e) = state.order_repo.remove_item(&item_id) {
+		return match e {
+			DomainError::NotFound(_) => error_response("Order item not found", "ITEM_NOT_FOUND", StatusCode::NOT_FOUND),
+			_ => domain_to_http(e),
+		}
+		.into_response();
+	}
+
+	// Recalculate total
+	if let Err(e) = state.order_repo.update_total(&id) {
+		return domain_to_http(e).into_response();
+	}
+
+	StatusCode::NO_CONTENT.into_response()
+}
diff --git a/src-tauri/src/db/orders.rs b/src-tauri/src/db/orders.rs
index aa90738..7b2d186 100644
--- a/src-tauri/src/db/orders.rs
+++ b/src-tauri/src/db/orders.rs
@@ -6,7 +6,8 @@
 use crate::domain::order::{Order, OrderItem, OrderRepository, OrderStatus};
 use crate::domain::DomainError;
 
-use super::SqlitePool;
+use super::{SqlitePool};
+use super::get_conn;
 
 pub struct DbOrderRepository {
 	pool: SqlitePool,
@@ -19,28 +20,203 @@ impl DbOrderRepository {
 }
 
 impl OrderRepository for DbOrderRepository {
-	fn create(&self, _order: &Order) -> Result<(), DomainError> {
-		todo!("Story 3.2")
+	fn create(&self, order: &Order) -> Result<(), DomainError> {
+		let conn = get_conn(&self.pool).map_err(|e| DomainError::Internal(e.to_string()))?;
+		conn.execute(
+			"INSERT INTO orders (id, table_id, client_id, status, total, created_at, updated_at) \
+			 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
+			rusqlite::params![
+				order.id,
+				order.table_id,
+				order.client_id,
+				order.status.as_str(),
+				order.total,
+				order.created_at,
+				order.updated_at,
+			],
+		)
+		.map_err(|e| DomainError::Internal(format!("Failed to create order: {}", e)))?;
+		Ok(())
 	}
-	fn update_status(&self, _id: &str, _status: &OrderStatus) -> Result<(), DomainError> {
-		todo!("Story 3.2")
+
+	fn update_status(&self, id: &str, status: &OrderStatus) -> Result<(), DomainError> {
+		let conn = get_conn(&self.pool).map_err(|e| DomainError::Internal(e.to_string()))?;
+		let now = chrono_now();
+		let affected = conn
+			.execute(
+				"UPDATE orders SET status = ?1, updated_at = ?2 WHERE id = ?3",
+				rusqlite::params![status.as_str(), now, id],
+			)
+			.map_err(|e| DomainError::Internal(format!("Failed to update order status: {}", e)))?;
+		if affected == 0 {
+			return Err(DomainError::NotFound(format!("Order {} not found", id)));
+		}
+		Ok(())
 	}
-	fn find_by_id(&self, _id: &str) -> Result<Option<Order>, DomainError> {
-		todo!("Story 3.2")
+
+	fn find_by_id(&self, id: &str) -> Result<Option<Order>, DomainError> {
+		let conn = get_conn(&self.pool).map_err(|e| DomainError::Internal(e.to_string()))?;
+		let mut stmt = conn
+			.prepare(
+				"SELECT id, table_id, client_id, status, total, created_at, updated_at \
+				 FROM orders WHERE id = ?1",
+			)
+			.map_err(|e| DomainError::Internal(e.to_string()))?;
+
+		let mut rows = stmt
+			.query_map(rusqlite::params![id], |row| {
+				let status_str: String = row.get("status")?;
+				Ok(Order {
+					id: row.get("id")?,
+					table_id: row.get("table_id")?,
+					client_id: row.get("client_id")?,
+					status: OrderStatus::from_str(&status_str).map_err(|e| {
+						rusqlite::Error::ToSqlConversionFailure(Box::new(e))
+					})?,
+					total: row.get("total")?,
+					created_at: row.get("created_at")?,
+					updated_at: row.get("updated_at")?,
+				})
+			})
+			.map_err(|e| DomainError::Internal(e.to_string()))?;
+
+		match rows.next() {
+			Some(Ok(order)) => Ok(Some(order)),
+			Some(Err(e)) => Err(DomainError::Internal(e.to_string()).into()),
+			None => Ok(None),
+		}
 	}
-	fn list_by_status(&self, _status: &OrderStatus) -> Result<Vec<Order>, DomainError> {
-		todo!("Story 3.2")
+
+	fn list_by_status(&self, status: &OrderStatus) -> Result<Vec<Order>, DomainError> {
+		let conn = get_conn(&self.pool).map_err(|e| DomainError::Internal(e.to_string()))?;
+		let mut stmt = conn
+			.prepare(
+				"SELECT id, table_id, client_id, status, total, created_at, updated_at \
+				 FROM orders WHERE status = ?1 ORDER BY created_at DESC",
+			)
+			.map_err(|e| DomainError::Internal(e.to_string()))?;
+
+		let orders = stmt
+			.query_map(rusqlite::params![status.as_str()], map_order_row)
+			.map_err(|e| DomainError::Internal(e.to_string()))?
+			.collect::<Result<Vec<_>, _>>()
+			.map_err(|e| DomainError::Internal(e.to_string()))?;
+
+		Ok(orders)
 	}
+
 	fn list_all(&self) -> Result<Vec<Order>, DomainError> {
-		todo!("Story 3.2")
+		let conn = get_conn(&self.pool).map_err(|e| DomainError::Internal(e.to_string()))?;
+		let mut stmt = conn
+			.prepare(
+				"SELECT id, table_id, client_id, status, total, created_at, updated_at \
+				 FROM orders ORDER BY created_at DESC",
+			)
+			.map_err(|e| DomainError::Internal(e.to_string()))?;
+
+		let orders = stmt
+			.query_map([], map_order_row)
+			.map_err(|e| DomainError::Internal(e.to_string()))?
+			.collect::<Result<Vec<_>, _>>()
+			.map_err(|e| DomainError::Internal(e.to_string()))?;
+
+		Ok(orders)
 	}
-	fn add_item(&self, _item: &OrderItem) -> Result<(), DomainError> {
-		todo!("Story 3.2")
+
+	fn add_item(&self, item: &OrderItem) -> Result<(), DomainError> {
+		let conn = get_conn(&self.pool).map_err(|e| DomainError::Internal(e.to_string()))?;
+		conn.execute(
+			"INSERT INTO order_items (id, order_id, product_id, quantity, unit_price, notes, created_at) \
+			 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
+			rusqlite::params![
+				item.id,
+				item.order_id,
+				item.product_id,
+				item.quantity,
+				item.unit_price,
+				item.notes,
+				item.created_at,
+			],
+		)
+		.map_err(|e| DomainError::Internal(format!("Failed to add order item: {}", e)))?;
+		Ok(())
 	}
-	fn get_items(&self, _order_id: &str) -> Result<Vec<OrderItem>, DomainError> {
-		todo!("Story 3.2")
+
+	fn get_items(&self, order_id: &str) -> Result<Vec<OrderItem>, DomainError> {
+		let conn = get_conn(&self.pool).map_err(|e| DomainError::Internal(e.to_string()))?;
+		let mut stmt = conn
+			.prepare(
+				"SELECT id, order_id, product_id, quantity, unit_price, notes, created_at \
+				 FROM order_items WHERE order_id = ?1 ORDER BY created_at ASC",
+			)
+			.map_err(|e| DomainError::Internal(e.to_string()))?;
+
+		let items = stmt
+			.query_map(rusqlite::params![order_id], |row| {
+				Ok(OrderItem {
+					id: row.get("id")?,
+					order_id: row.get("order_id")?,
+					product_id: row.get("product_id")?,
+					quantity: row.get("quantity")?,
+					unit_price: row.get("unit_price")?,
+					notes: row.get("notes")?,
+					created_at: row.get("created_at")?,
+				})
+			})
+			.map_err(|e| DomainError::Internal(e.to_string()))?
+			.collect::<Result<Vec<_>, _>>()
+			.map_err(|e| DomainError::Internal(e.to_string()))?;
+
+		Ok(items)
+	}
+
+	fn remove_item(&self, item_id: &str) -> Result<(), DomainError> {
+		let conn = get_conn(&self.pool).map_err(|e| DomainError::Internal(e.to_string()))?;
+		let affected = conn
+			.execute(
+				"DELETE FROM order_items WHERE id = ?1",
+				rusqlite::params![item_id],
+			)
+			.map_err(|e| DomainError::Internal(format!("Failed to remove order item: {}", e)))?;
+		if affected == 0 {
+			return Err(DomainError::NotFound(format!("Order item {} not found", item_id)));
+		}
+		Ok(())
 	}
-	fn remove_item(&self, _item_id: &str) -> Result<(), DomainError> {
-		todo!("Story 3.2")
+
+	fn update_total(&self, order_id: &str) -> Result<(), DomainError> {
+		let conn = get_conn(&self.pool).map_err(|e| DomainError::Internal(e.to_string()))?;
+		let now = chrono_now();
+		let affected = conn
+			.execute(
+				"UPDATE orders SET total = COALESCE((SELECT SUM(quantity * unit_price) FROM order_items WHERE order_id = ?1), 0), updated_at = ?2 WHERE id = ?1",
+				rusqlite::params![order_id, now],
+			)
+			.map_err(|e| DomainError::Internal(format!("Failed to update order total: {}", e)))?;
+		if affected == 0 {
+			return Err(DomainError::NotFound(format!("Order {} not found", order_id)));
+		}
+		Ok(())
 	}
 }
+
+/// Helper to map a SQL row to an Order.
+fn map_order_row(row: &rusqlite::Row) -> rusqlite::Result<Order> {
+	let status_str: String = row.get("status")?;
+	Ok(Order {
+		id: row.get("id")?,
+		table_id: row.get("table_id")?,
+		client_id: row.get("client_id")?,
+		status: OrderStatus::from_str(&status_str).map_err(|e| {
+			rusqlite::Error::ToSqlConversionFailure(Box::new(e))
+		})?,
+		total: row.get("total")?,
+		created_at: row.get("created_at")?,
+		updated_at: row.get("updated_at")?,
+	})
+}
+
+fn chrono_now() -> String {
+	use chrono::Utc;
+	Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
+}
diff --git a/src-tauri/src/domain/order.rs b/src-tauri/src/domain/order.rs
index a7d2103..c7bda2d 100644
--- a/src-tauri/src/domain/order.rs
+++ b/src-tauri/src/domain/order.rs
@@ -54,6 +54,7 @@ pub struct OrderItem {
 	pub quantity: i64,
 	pub unit_price: i64,
 	pub notes: Option<String>,
+	pub created_at: String,
 }
 
 /// A customer order with its lifecycle status.
@@ -114,4 +115,6 @@ pub trait OrderRepository: Send + Sync {
 	fn add_item(&self, item: &OrderItem) -> Result<(), DomainError>;
 	fn get_items(&self, order_id: &str) -> Result<Vec<OrderItem>, DomainError>;
 	fn remove_item(&self, item_id: &str) -> Result<(), DomainError>;
+	/// Recalculate and persist the order total from order_items.
+	fn update_total(&self, order_id: &str) -> Result<(), DomainError>;
 }
diff --git a/src-tauri/src/lib.rs b/src-tauri/src/lib.rs
index 957a2b9..009b236 100644
--- a/src-tauri/src/lib.rs
+++ b/src-tauri/src/lib.rs
@@ -31,8 +31,10 @@ use tauri::Manager;
 
 use api::AppApiState;
 use db::users::DbUserRepository;
+use db::orders::DbOrderRepository;
 use db::products::DbProductRepository;
 use crate::domain::product::ProductRepository;
+use crate::domain::order::OrderRepository;
 use db::wallet_ledger::DbWalletRepository;
 use crate::domain::wallet::WalletRepository;
 use domain::user::UserRepository;
@@ -104,10 +106,12 @@ pub fn run() {
 	// Build the full application router
 	let user_repo: Arc<dyn UserRepository> = Arc::new(DbUserRepository::new(pool.clone()));
 	let wallet_repo: Arc<dyn WalletRepository> = Arc::new(DbWalletRepository::new(pool.clone()));
+	let order_repo: Arc<dyn OrderRepository> = Arc::new(DbOrderRepository::new(pool.clone()));
 	let product_repo: Arc<dyn ProductRepository> = Arc::new(DbProductRepository::new(pool.clone()));
 	let api_state = AppApiState {
 		user_repo,
 		wallet_repo,
+		order_repo,
 		product_repo,
 		jwt_secret,
 	};
diff --git a/src-tauri/migrations/V4__orders.sql b/src-tauri/migrations/V4__orders.sql
new file mode 100644
index 0000000..3df5c08
--- /dev/null
+++ b/src-tauri/migrations/V4__orders.sql
@@ -0,0 +1,39 @@
+-- V4__orders.sql
+-- Order lifecycle: orders and order_items tables.
+--
+-- AD-13: Order depends on Catalog (product_id FK conceptual, no FK constraint
+--         to avoid blocking product deletion). Referential integrity is
+--         enforced at the application layer via ProductRepository lookups.
+-- AD-2:  order_items is mutable (add/remove items allowed in PendingPayment).
+--         Once past PendingPayment, mutability is gated by the domain layer.
+--         Financial mutation is NOT in this table -- wallet_ledger (V2) is
+--         the append-only financial record.
+
+CREATE TABLE IF NOT EXISTS orders (
+    id          TEXT PRIMARY KEY,
+    table_id    TEXT,
+    client_id   TEXT,
+    status      TEXT NOT NULL DEFAULT 'pending_payment',
+    total       INTEGER NOT NULL DEFAULT 0,
+    created_at  TEXT NOT NULL,
+    updated_at  TEXT NOT NULL
+);
+
+CREATE TABLE IF NOT EXISTS order_items (
+    id          TEXT PRIMARY KEY,
+    order_id    TEXT NOT NULL REFERENCES orders(id),
+    product_id  TEXT NOT NULL,
+    quantity    INTEGER NOT NULL CHECK(quantity > 0),
+    unit_price  INTEGER NOT NULL,
+    notes       TEXT,
+    created_at  TEXT NOT NULL
+);
+
+-- Index for retrieving items by order
+CREATE INDEX IF NOT EXISTS idx_order_items_order_id ON order_items(order_id);
+
+-- Index for filtering orders by status (kitchen display, etc.)
+CREATE INDEX IF NOT EXISTS idx_orders_status ON orders(status);
+
+-- Index for lookups by table
+CREATE INDEX IF NOT EXISTS idx_orders_table_id ON orders(table_id);
```

## Output Format

```markdown
### Finding N: [CRITICAL|MAJOR|MINOR] — Title

**AC:** Which AC is violated (or "Architecture" for AD violations)
**Evidence:** What the diff shows vs what the spec requires
**Suggestion:** How to align implementation with spec
```
