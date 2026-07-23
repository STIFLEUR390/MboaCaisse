---
baseline_commit: ed2b716e5592846fecf21be113995de2a985ee52
---

# Story 3.1: CRUD Produits & Catégories

Status: done

## Story

As a gérant,
I want pouvoir créer, modifier, supprimer et consulter des produits et catégories,
So que le menu de l'établissement est à jour et que les commandes futures peuvent référencer des articles existants.

## Acceptance Criteria

### AC-1: Migration V3__products.sql
**Given** la migration V3 est placée dans `src-tauri/migrations/V3__products.sql`
**When** le runner refinery l'exécute
**Then** la table `categories` est créée :
- `id` TEXT PRIMARY KEY (UUID v7)
- `name` TEXT NOT NULL
- `parent_id` TEXT nullable (REFERENCES categories(id) pour hiérarchie)
- `created_at` TEXT NOT NULL
- `updated_at` TEXT NOT NULL
**And** la table `products` est créée :
- `id` TEXT PRIMARY KEY (UUID v7)
- `name` TEXT NOT NULL
- `price` INTEGER NOT NULL (prix en FCFA, pas de décimales)
- `category_id` TEXT NOT NULL REFERENCES categories(id)
- `stock` INTEGER DEFAULT 0
- `alert_threshold` INTEGER DEFAULT 5
- `created_at` TEXT NOT NULL
- `updated_at` TEXT NOT NULL

### AC-2: Domain — Constructeurs Product + Category
**Given** les entités existantes dans `domain/product.rs`
**When** on ajoute des méthodes `Product::new(...)` et `Category::new(...)`
**Then** chaque constructeur génère un UUID v7, horodatage ISO 8601 UTC, et valide les champs requis (name non vide, price ≥ 0)
**And** `ProductRepository` trait conserve toutes ses méthodes (inchangé, déjà défini)

### AC-3: POST /api/products
**Given** un POST `/api/products` avec `{ name: "Bière 33", price: 500, category_id: "<id>", stock?: 20, alert_threshold?: 5 }`
**When** les données sont valides
**Then** le produit est créé avec UUID v7, horodatage, stock=0 par défaut, alert_threshold=5 par défaut
**And** la réponse HTTP est 201 avec le produit créé en JSON
**And** si `name` est vide → 400 Bad Request
**And** si `price < 0` → 400 Bad Request
**And** si `category_id` ne correspond à aucune catégorie → 422 Unprocessable Entity

### AC-4: GET /api/products, GET /api/products/{id}, GET /api/products?category={id}
**Given** des produits existants
**When** GET `/api/products`
**Then** retourne 200 avec la liste complète des produits (tableau JSON, triés par created_at ASC)
**When** GET `/api/products/{id}`
**Then** retourne 200 avec le produit trouvé, ou 404 si inexistant
**When** GET `/api/products?category={id}`
**Then** retourne 200 avec les produits filtrés par catégorie, ou 400 si category_id manquant

### AC-5: PUT /api/products/{id}
**Given** un produit existant
**When** PUT `/api/products/{id}` avec `{ name, price, category_id, stock?, alert_threshold? }`
**Then** les champs sont mis à jour (name, price, category_id, stock, alert_threshold)
**And** `updated_at` est rafraîchi
**And** retourne 200 avec le produit mis à jour
**And** si `id` n'existe pas → 404

### AC-6: DELETE /api/products/{id}
**Given** un produit existant
**When** DELETE `/api/products/{id}`
**Then** le produit est supprimé du catalogue (DELETE FROM products WHERE id = ?)
**And** les commandes passées qui référencent ce produit ne sont PAS affectées (intégrité référentielle gérée par l'absence de FK dans order_items)
**And** retourne 204 No Content
**And** si le produit n'existe pas → 404

### AC-7: CRUD Catégories — POST/GET/PUT/DELETE /api/categories
**Given** une catégorie parente
**When** POST `/api/categories` avec `{ name: "Boissons", parent_id?: null }`
**Then** 201 avec la catégorie créée
**When** GET `/api/categories` → 200 avec toutes les catégories
**When** PUT `/api/categories/{id}` → 200 avec la catégorie mise à jour
**When** DELETE `/api/categories/{id}` si elle n'a pas de produits attachés → 204
**When** DELETE `/api/categories/{id}` si elle a des produits attachés → 422 (empêche suppression orpheline)
**When** DELETE `/api/categories/{id}` si elle a des sous-catégories → 422 (supprimer les sous-catégories d'abord)
**And** si `parent_id` référence une catégorie inexistante → 422
**And** GET `/api/categories/{id}` retourne la catégorie avec ses sous-catégories (children: [])

### AC-8: Seed — 3 catégories + 10 produits de démo
**Given** le seed idempotent dans `db/seed.rs`
**When** la BDD est vierge au premier démarrage
**Then** après le seed admin, 3 catégories sont créées (Boissons, Plats, Snacks) et 10 produits répartis
**And** le seed est idempotent : vérifie `SELECT COUNT(*) FROM categories` avant d'insérer

### AC-9: Routes API montées dans build_app()
**Given** `api/mod.rs` monte les nouvelles routes dans `build_app()`
**When** le serveur démarre
**Then** les routes produits sont protégées par le middleware JWT (sauf si auth bypass explicite pour menu public futur)
**And** `/api/categories` et `/api/products` sont accessibles

## Tasks / Subtasks

- [x] **AC-1** Créer la migration SQL V3__products.sql
  - [x] Ajouter le fichier `src-tauri/migrations/V3__products.sql`
  - [x] Tables : categories + products avec contraintes FK
  - [x] Index sur products(category_id), categories(parent_id)
- [x] **AC-2** Enrichir le domaine product.rs
  - [x] Ajouter `impl Product { pub fn new(...) -> Self }` avec UUID v7 + dates
  - [x] Ajouter `impl Category { pub fn new(...) -> Self }` avec UUID v7 + dates
  - [x] Ajouter `Product::update(...)` pour mise à jour des champs
  - [x] Ajouter `Category::update(...)` pour mise à jour du nom
- [x] **AC-3, AC-4, AC-5, AC-6** Implémenter `api/products.rs`
  - [x] `POST /api/products` — création avec validation
  - [x] `GET /api/products` — liste complète (optionnel ?category=)
  - [x] `GET /api/products/{id}` — détail
  - [x] `PUT /api/products/{id}` — mise à jour
  - [x] `DELETE /api/products/{id}` — suppression
  - [x] Gestion des erreurs : 400, 404, 422, 201, 204
- [x] **AC-7** Implémenter `api/categories` (dans products.rs ou fichier séparé)
  - [x] `POST /api/categories` — création avec validation parent_id
  - [x] `GET /api/categories` — liste
  - [x] `GET /api/categories/{id}` — détail avec enfants
  - [x] `PUT /api/categories/{id}` — mise à jour
  - [x] `DELETE /api/categories/{id}` — suppression avec garde (produits attachés, sous-catégories)
- [x] **AC-8** Implémenter `db/products.rs` — toutes les méthodes ProductRepository
  - [x] `create_product` — INSERT
  - [x] `update_product` — UPDATE
  - [x] `delete_product` — DELETE
  - [x] `find_product_by_id` — SELECT WHERE id
  - [x] `list_products_by_category` — SELECT WHERE category_id
  - [x] `search_products` — SELECT WHERE name LIKE
  - [x] `list_all_products` — SELECT avec ORDER BY created_at
  - [x] `create_category`, `update_category`, `delete_category`
  - [x] `find_category_by_id`, `list_all_categories`
  - [x] Ajouter méthode `count_products_by_category`, `count_child_categories`, `find_child_categories`
- [x] **AC-8** Ajouter le seed produits dans `db/seed.rs`
  - [x] Après le seed admin, si `SELECT COUNT(*) FROM categories == 0`, insérer 3 catégories
  - [x] Insérer 10 produits répartis dans les 3 catégories
  - [x] UUID v7 + dates ISO 8601 pour chaque entrée
- [x] **AC-9** Monter les routes dans `api/mod.rs`
  - [x] Ajouter les routes products (/api/products/*)
  - [x] Ajouter les routes categories (/api/categories/*)
  - [x] Utiliser `axum::routing::{get, post, put, delete}`

## Dev Notes

### Architecture Compliance

- **AD-1** (Layered + Rich Domain) : Le comportement CRUD vit dans `domain/product.rs` (constructeurs, update). `api/products.rs` est une peau fine. `db/products.rs` implémente le trait.
- **AD-3** (Structure plate) : Pas de sous-dossiers. Tout dans `api/products.rs`, `domain/product.rs`, `db/products.rs`. Si `api/categories` grossit, le garder dans le même fichier que products.
- **AD-7** (Traits dans domain) : `ProductRepository` existe déjà dans `domain/product.rs`. Ne pas le déplacer.
- **AD-8** (Erreurs 3 couches) : Les erreurs SQL ne remontent pas. DbError → DomainError → HTTP status.
  - Repository methods retournent `Result<T, DomainError>`
  - Handlers API retournent `(StatusCode, Json<ApiError>)`
  - Codes d'erreur : `PRODUCT_NOT_FOUND`, `CATEGORY_NOT_FOUND`, `INVALID_NAME`, `INVALID_PRICE`, `CATEGORY_HAS_PRODUCTS`, `CATEGORY_HAS_CHILDREN`, `PARENT_CATEGORY_NOT_FOUND`
- **AD-10** (Stack alpha) : UUID v7, chrono pour dates, rusqlite paramétré.
- **AD-13** (Graphe dépendances) : Products/Categories ne dépendent que de rien d'autre qu'elles-mêmes. Pas de dépendance vers Wallet, Order, Auth.
- **AD-15** (Migrations refinery) : V3__products.sql, exécutée automatiquement par le runner existant.
- **AD-16** (Pool r2d2) : `DbProductRepository::new(pool)` utilise le pool existant.

### API Contract

Toutes les routes sont sous `/api/*` et protégées par le middleware JWT existant (`auth_middleware.rs`).

| Méthode | Route | Status | Description |
|---------|-------|--------|-------------|
| POST | /api/products | 201 | Créer un produit |
| GET | /api/products | 200 | Liste produits (optionnel ?category=) |
| GET | /api/products/{id} | 200 | Détail produit |
| PUT | /api/products/{id} | 200 | Mettre à jour un produit |
| DELETE | /api/products/{id} | 204 | Supprimer un produit |
| POST | /api/categories | 201 | Créer une catégorie |
| GET | /api/categories | 200 | Liste catégories |
| GET | /api/categories/{id} | 200 | Détail catégorie avec enfants |
| PUT | /api/categories/{id} | 200 | Mettre à jour une catégorie |
| DELETE | /api/categories/{id} | 204 | Supprimer (si vide) |

Format réponse erreur : `{"error": "message", "code": "SCREAMING_SNAKE"}`

### Existing Code — State & What to Preserve

**`domain/product.rs`** — déjà existant avec Product, Category, ProductRepository trait.
- ✅ ProductRepository trait complet avec toutes les méthodes nécessaires
- ✅ Product struct avec tous les champs requis
- ✅ Category struct avec id, name, parent_id
- ❌ Manque `impl Product` et `impl Category` (constructeurs, update) — À AJOUTER
- ❌ Manque `find_products_by_category_id` dans le trait — DOIT être ajouté (nécessaire pour AC-7)
- ❌ Manque une méthode `category_has_products` ou `count_products_by_category` — DOIT être ajouté

**`db/products.rs`** — squelette avec `todo!("Story 3.1")` partout
- ❌ Toutes les méthodes sont à implémenter (AC-3 à AC-7)
- ⚠️ Structure `DbProductRepository::new(pool)` déjà en place

**`api/products.rs`** — commentaire uniquement
- ❌ Remplacer par des handlers complets

**`api/mod.rs`** — build_app()
- ❌ Routes products/categories à ajouter

**`db/seed.rs`** — seed admin seulement
- ❌ Ajouter seed catégories + produits

### Gotchas & Écueils

- ⚠️ **FK contrainte** : Le `category_id` dans products référence `categories(id)`. Si on DELETE une catégorie avec des produits, ça échoue. Gérer avec une vérification préalable (count products) et retourner 422.
- ⚠️ **DELETE récursif catégorie** : Si une catégorie a des sous-catégories (parent_id), on ne peut pas la supprimer sans supprimer les enfants. Vérifier avant DELETE.
- ⚠️ **Parent_id cyclique** : Éviter qu'une catégorie se référence elle-même comme parent. Valider que `parent_id != id`.
- ⚠️ **prix en i64** : Le prix est stocké en `INTEGER` (i64 dans Product.price). Pas de décimales — prix en FCFA (monnaie sans centimes). Le type i64 est déjà utilisé dans le domaine.
- ⚠️ **Même pattern que auth** : Suivre le même pattern que les handlers auth existants. Les handlers prennent `State<AppApiState>` et retournent `Result<(StatusCode, Json<T>), (StatusCode, Json<ApiError>)>`. Voir `api/auth.rs` comme référence.
- ⚠️ **Refinery V3** : Le fichier doit s'appeler `V3__products.sql` (deux underscores après le numéro). C'est le format que refinery attend. Le runner va automatiquement le détecter et l'exécuter après V2.
- ⚠️ **Seed idempotent** : Vérifier `SELECT COUNT(*) FROM categories` avant d'insérer le seed. Même pattern que le seed admin existant.

### Seed Data

Catégories :
1. "Boissons" (parent_id: null)
2. "Plats" (parent_id: null)
3. "Snacks" (parent_id: null)

Produits (10) :
1. Bière 33 — 500 FCFA → Boissons
2. Bière 65 — 700 FCFA → Boissons
3. Jus de fruits — 400 FCFA → Boissons
4. Eau minérale — 200 FCFA → Boissons
5. Planteur frites — 1500 FCFA → Plats
6. Poulet braisé — 2000 FCFA → Plats
7. Poisson braisé — 2500 FCFA → Plats
8. Miondo — 500 FCFA → Snacks
9. Beignets (5pcs) — 300 FCFA → Snacks
10. Brochettes (3pcs) — 1000 FCFA → Snacks

### File List

**NOUVEAUX :**
- `src-tauri/migrations/V3__products.sql` — Migration SQL

**MODIFIÉS :**
- `src-tauri/src/domain/product.rs` — Constructeurs + méthodes update + méthodes repository supplémentaires
- `src-tauri/src/db/products.rs` — Implémentation complète ProductRepository
- `src-tauri/src/api/products.rs` — Handlers CRUD complets
- `src-tauri/src/api/mod.rs` — Montage routes + export categories handlers (ou tout dans products.rs)
- `src-tauri/src/db/seed.rs` — Seed catégories + produits

**SUPPRIMÉS :** Aucun

### Testing Validation

- `cargo check` doit passer sans erreur
- Les migrations s'exécutent sans erreur au démarrage
- Le seed est idempotent (redémarrage safe)
- Un curl POST /api/products crée bien un produit
- Un curl DELETE /api/categories/{id} avec produits attachés retourne 422

### Review Findings

- [x] [Review][Patch] POST /api/products — ajouter validation existence category_id [api/products.rs:181]
  AC-3 exige 422 si category_id inexistant. Actuellement pas de vérification → FK SQL → 500.
- [x] [Review][Patch] PUT /api/products/{id} — ajouter validation existence category_id [api/products.rs:270]
  Même trou que POST. Changer category_id vers un ID inexistant produit 500 au lieu de 422.
- [x] [Review][Patch] Product::new/update — ajouter validation stock < 0 [domain/product.rs:27]
  stock passe sans validation domaine puis heurte CHECK(stock >= 0) SQL → 500.
- [x] [Review][Patch] Product::new/update — ajouter validation alert_threshold < 0 [domain/product.rs:28]
  alert_threshold n'a ni validation domaine ni CHECK SQL — valeur négative persistée silencieusement.
- [x] [Review][Patch] get_category — propager erreur find_child_categories au lieu de l'avaler [api/products.rs:342]
  `Err(_) => vec![]` masque les erreurs DB. Le client ne peut pas distinguer "pas d'enfants" d'une erreur.
- [x] [Review][Patch] Seed catalogue — wrapper dans une transaction [db/seed.rs:seed_catalogue]
  Crash après insertion catégories mais avant produits → état partiel. Skip idempotent empêche re-exécution.
- [x] [Review][Patch] Seed catalogue — timestamps par ligne au lieu d'un timestamp partagé [db/seed.rs:seed_catalogue]
  Tous les produits ont le même created_at → tri non déterministe.
- [x] [Review][Patch] Seed nom produit — corriger "Beignets (5 pcs)" → "Beignets (5pcs)" [db/seed.rs]
  Divergence cosmétique avec la spec.
- [x] [Review][Patch] domain_to_http — ajouter arm explicite pour Unauthorized [api/products.rs:domain_to_http]
  Actuellement catch-all → 500. Devrait être 401.
- [x] [Review][Defer] DELETE /api/categories requêtes redondantes — deferred, optimisation non bloquante
- [x] [Review][Defer] Race condition guards DELETE catégorie — deferred, pas de concurrence en alpha mono-utilisateur
- [x] [Review][Defer] update_category détection cycles profonds — deferred, non exploitable avec l'API actuelle
- [x] [Review][Decision] PUT écrase stock/alert_threshold avec defaults si omis — choix design: PUT require all fields ou PATCH partiel? [api/products.rs:258]
- [x] [Review][Defer] domain_to_http codes erreur spec (INVALID_NAME/INVALID_PRICE) non retournés — deferred, spec secondaire
- [x] [Review][Defer] delete_category code erreur NOT_FOUND vs CATEGORY_NOT_FOUND — deferred, race path
- [x] [Review][Defer] search_products LIKE sans échappement wildcards — deferred, pas exposé via API

## Dev Agent Record

### Agent Model Used

bmad-create-story via GPT-5 (Codex)

### Debug Log References

- Le trait ProductRepository a déjà `list_all_products()` et `list_all_categories()` — vérifier qu'elles sont utilisées par les handlers GET
- La méthode `find_products_by_category` existe dans le trait mais s'appelle `list_products_by_category` — utilisée pour GET /api/products?category=
- `AppApiState` a déjà `user_repo`, `wallet_repo`, `jwt_secret`. Pour products, ajouter `product_repo: Arc<dyn ProductRepository>` dans AppApiState.
- Le `build_app()` dans `api/mod.rs` construit le router complet — ajouter les routes products/categories AVANT le fallback_service mais APRÈS auth dans l'ordre du router.
- Les handlers auth utilisent `State(app_state)` comme premier paramètre — suivre le même pattern.
- Si `ApiError` n'est pas encore défini comme type unifié, utiliser `(StatusCode, Json<serde_json::Value>)` avec `json!({ "error": "...", "code": "..." })`.

### Completion Notes List

- [x] `V3__products.sql` créé et placé dans migrations/
- [x] `domain/product.rs` enrichi avec constructeurs et méthodes
- [x] `db/products.rs` toutes les méthodes ProductRepository implémentées
- [x] `api/products.rs` handlers CRUD complets
- [x] Routes montées dans `api/mod.rs`
- [x] Seed catégories + produits ajouté dans `seed.rs`
- [x] `cargo check` passe sans erreur
- [x] API testée avec curl (POST, GET, PUT, DELETE)

### Change Log
- **2026-07-23** -- Implementation complète de la story 3.1
  - Migration V3__products.sql (categories + produits)
  - Domain enrichi (constructeurs Product::new, Category::new, update methods)
  - ProductRepository trait étendu (count_products_by_category, count_child_categories, find_child_categories)
  - DbProductRepository implémente toutes les méthodes (INSERT/UPDATE/DELETE/SELECT)
  - Handlers API CRUD complets pour produits et catégories (10 endpoints)
  - Routes montées dans api/mod.rs avec middleware JWT
  - Seed catalogue (3 catégories, 10 produits) idempotent
  - `cargo check` passe sans erreur

- **2026-07-23** -- Code review fixes applied
  - Added category_id existence check in POST/PUT /api/products (returns 422)
  - Added stock < 0 and alert_threshold < 0 validation in domain layer
  - Fixed get_category error swallowing (propagates errors properly)
  - Wrapped seed catalogue in transaction with per-row timestamps
  - Fixed seed product name "Beignets (5 pcs)" → "Beignets (5pcs)"
  - Added Unauthorized arm to domain_to_http mapping
  - Changed PUT to PATCH semantics for stock/alert_threshold (preserve existing values on omit)

### References

- [Source: epics.md#Story-3.1] — Définition originale de la story avec AC
- [Source: ARCHITECTURE-SPINE.md#AD-1] — Paradigme Layered + Rich Domain
- [Source: ARCHITECTURE-SPINE.md#AD-3] — Structure plate par couche
- [Source: ARCHITECTURE-SPINE.md#AD-7] — Traits dans domain/, impl dans db/
- [Source: ARCHITECTURE-SPINE.md#AD-8] — Erreurs 3 couches
- [Source: ARCHITECTURE-SPINE.md#AD-15] — Migrations refinery
- [Source: .ai-memory/index.md] — Gotchas refinery 0.9, conventions UUID v7
- [Source: sprint-status.yaml] — Story identifiée comme premier backlog d'Epic 3
