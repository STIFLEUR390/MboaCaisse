---
baseline_commit: 8d142b6
---

# Story 3.2: Cycle de Vie Commande


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

## Review Findings

### Review Follow-ups (AI)
- [x] [Review][Patch] Cross-order item deletion — remove_item now validates order_id in SQL [api/orders.rs:483, db/orders.rs:182]
- [x] [Review][Patch] Orphan order on partial create failure — delete() added to trait, cleanup on item/total failure [api/orders.rs:248-256, domain/order.rs:96]
- [x] [Review][Patch] get_items error swallowed in update_order_status — changed Err(_) => vec![] to propagate error [api/orders.rs:418-420]
- [x] [Review][Patch] domain_to_http missing InsufficientBalance — added arm returning 422 [api/orders.rs:170-177]
- [x] [Review][Defer] N+1 queries in list_orders — pre-existing, alpha OK, batch load for future
- [x] [Review][Defer] map_order_row converts DomainError to rusqlite Error — pre-existing pattern
- [x] [Review][Defer] remove_item no PendingPayment guard in DB layer — defense-in-depth, already guarded at API layer
- [x] [Review][Defer] chrono_now/uuid_v7 duplicated in db/orders.rs + api/orders.rs — cosmetic, extract to shared util in future
- [x] [Review][Defer] list_all without LIMIT — pre-existing, pagination deferred

### Status
Status: done

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
