---
baseline_commit: 139feaa
---

# Story 1.5: Rôles & Permissions — Middleware Guard & Seed Admin

Status: done

## Story

As a **admin**,
I want que chaque utilisateur ait un rôle avec des permissions dérivées,
so that l'accès aux fonctionnalités est contrôlé par rôle.

## Acceptance Criteria

### AC-1: 4 rôles définis avec permissions dérivées

**Given** 4 rôles définis dans `domain/user.rs` (Admin, Caissier, Vendeur, GestionnaireStock)
**When** un utilisateur est créé avec un rôle
**Then** ses permissions sont dérivées via `Role::permissions()` — pas stockées en BDD

### AC-2: Admin a Permission::All

**Given** Admin implémente Permission::All
**When** un admin accède à n'importe quelle route API protégée
**Then** l'accès est autorisé (Permission::All match toutes les permissions)

### AC-3: Caissier

**Given** Caissier a Permission::Sell, Permission::ViewReports, Permission::ViewOrders
**When** un caissier accède à la caisse (routes Sell)
**Then** l'accès est autorisé

**Given** Caissier n'a pas Permission::ManageUsers
**When** un caissier accède à la gestion des employés
**Then** 403 Forbidden est retourné

### AC-4: Vendeur

**Given** Vendeur a Permission::ViewOrders, Permission::ManageMenu
**When** un vendeur accède aux commandes
**Then** l'accès est autorisé

**Given** Vendeur n'a pas Permission::Sell
**When** un vendeur accède à la caisse
**Then** 403 Forbidden est retourné

### AC-5: GestionnaireStock

**Given** GestionnaireStock a Permission::ManageStock, Permission::ViewReports
**When** un gestionnaire accède au stock
**Then** l'accès est autorisé

**Given** GestionnaireStock n'a pas Permission::Sell
**When** un gestionnaire accède à la caisse
**Then** 403 Forbidden est retourné

### AC-6: Seed admin idempotent

**Given** la BDD est vierge
**When** le seed s'exécute
**Then** un compte admin est créé avec email + mot de passe généré (affiché une fois dans la console)
**And** le seed est idempotent (ne crée pas de doublon au redémarrage)

### AC-7: Middleware role-check — 401 / 403

**Given** le middleware role-check dans `api/mod.rs`
**When** une route protégée est appelée sans JWT
**Then** 401 Unauthorized est retourné
**When** une route protégée est appelée avec un rôle insuffisant
**Then** 403 Forbidden est retourné avec `{"error": "Forbidden", "code": "FORBIDDEN"}`

### AC-8: CRUD users — Admin management

**Given** un admin authentifié
**When** GET /api/users
**Then** retourne la liste de tous les utilisateurs (id, email, name, role)

**Given** un admin authentifié
**When** POST /api/users avec { email, password, name?, role }
**Then** un nouvel utilisateur est créé avec le rôle spécifié

**Given** un admin authentifié
**When** PATCH /api/users/{id} avec { email?, name?, role?, password? }
**Then** l'utilisateur est mis à jour, updated_at horodaté

**Given** un admin authentifié
**When** DELETE /api/users/{id}
**Then** l'utilisateur est supprimé
**And** 400 si tentative de suppression de soi-même

**Given** un utilisateur non-admin
**When** accès à /api/users/*
**Then** 403 Forbidden

### AC-9: Frontend — Page de gestion des utilisateurs

**Given** un admin connecté
**When** il navigue vers `/admin/users`
**Then** une liste de tous les utilisateurs est affichée (tableau avec email, nom, rôle)
**And** des boutons Créer/Modifier/Supprimer sont disponibles

**Given** le formulaire de création d'utilisateur
**When** l'admin remplit les champs et soumet
**Then** l'utilisateur est créé avec le rôle choisi
**And** un toast de confirmation est affiché

**Given** la modification d'utilisateur
**When** l'admin change le rôle d'un utilisateur
**Then** la mise à jour est persistée immédiatement

**Given** la suppression d'utilisateur
**When** l'admin confirme la suppression
**Then** l'utilisateur est supprimé
**And** l'admin ne peut pas se supprimer lui-même

### AC-10: Frontend — Permission-based UI

**Given** un caissier connecté
**When** la navigation s'affiche
**Then** seuls les liens pertinents (Ventes, Rapports) sont visibles
**And** le lien "Administration" est masqué

**Given** un admin connecté
**When** la navigation s'affiche
**Then** tous les liens sont visibles

**Given** un utilisateur sans permission ManageStock
**When** il visite `/stock`
**Then** la page affiche une erreur "Accès refusé" ou redirige

## Tasks / Subtasks

### Backend Rust — Permission middleware & mapping

- [x] **T1** — Implémenter la vérification des permissions dans le middleware auth (AC-7)
  - [x] T1.1 Ajouter une fonction `required_permission(path) -> Option<Permission>` dans `auth_middleware.rs`
  - [x] T1.2 Cette fonction mappe les préfixes de route aux permissions requises
  - [x] T1.3 Après la vérification JWT, appeler `required_permission()` et vérifier avec `Role::has_permission()`
  - [x] T1.4 Si permission manquante → retourner 403 `{"error": "Forbidden", "code": "FORBIDDEN"}`
  - [x] T1.5 Les routes `/api/health`, `/api/auth/login`, `/api/auth/register` restent publiques
  - [x] T1.6 Les routes `/api/auth/logout`, `/api/auth/me` nécessitent juste un JWT valide
  - [x] T1.7 Les routes `/api/settings` nécessitent Permission::ManageSettings
  - [x] T1.8 Les routes `/api/users` nécessitent Permission::ManageUsers
  - [x] T1.9 Fallback : toute route `/api/*` non listée → Permission::All (admin only)
  - [x] T1.10 Logger les tentatives d'accès refusées (warn)

### Backend Rust — Users CRUD API

- [x] **T2** — Créer les endpoints CRUD pour la gestion des utilisateurs (AC-8)
  - [x] T2.1 Créer `src/api/users.rs` avec : GET list_users, POST create_user, PATCH update_user, DELETE delete_user
  - [x] T2.2 Validation : email valide, password >= 8 chars, role valide
  - [x] T2.3 Hash du password avec argon2 si modifié
  - [x] T2.4 Ne pas exposer `password_hash` dans les réponses
  - [x] T2.5 Logger chaque opération CRUD (info)

### Backend Rust — Integration dans api/mod.rs

- [x] **T3** — Monter les routes et intégrer (AC-7, AC-8)
  - [x] T3.1 Ajouter `pub mod users;` dans `api/mod.rs`
  - [x] T3.2 Ajouter les routes `/api/users` dans `build_app()`
  - [x] T3.3 `GET /api/users` → `users::list_users`
  - [x] T3.4 `POST /api/users` → `users::create_user`
  - [x] T3.5 `PATCH /api/users/{id}` → `users::update_user`
  - [x] T3.6 `DELETE /api/users/{id}` → `users::delete_user`

### Frontend — Page de gestion des utilisateurs

- [x] **T4** — Créer la page `/admin/users` (AC-9)
  - [x] T4.1 Créer `app/pages/admin/users.vue` avec `definePageMeta({ middleware: ["auth", "admin"] })`
  - [x] T4.2 Tableau listant les utilisateurs (email, name, role, created_at)
  - [x] T4.3 Modal de création (email, password, name, role select)
  - [x] T4.4 Modal de modification (pré-rempli)
  - [x] T4.5 Confirmation de suppression (empêche self-delete)
  - [x] T4.6 UToast pour feedbacks
  - [x] T4.7 Appels API via `$fetch()` avec credentials

### Frontend — Middleware admin & navigation

- [x] **T5** — Créer le middleware admin (AC-9)
  - [x] T5.1 Créer `app/middleware/admin.ts` → redirige vers `/` si `role !== "admin"`
  - [x] T5.2 Utiliser dans `definePageMeta` des pages `/admin/*`

- [x] **T6** — Navigation conditionnelle (AC-10)
  - [x] T6.1 Dans `app/composables/pages.ts`, filtrer les routes selon `minRole` défini dans `definePageMeta`
  - [x] T6.2 Admin → tous les liens (admin + system + autres)
  - [x] T6.3 Caissier → ne voit pas les pages admin (minRole: "admin")
  - [x] T6.4 Settings → minRole: "admin" (visible par admin seulement)
  - [x] T6.5 Nouvelle catégorie "Administration" dans app.config.ts

### Vérifications

- [x] **T7** — Vérifications finales
  - [x] T7.1 `cargo check` passe
  - [x] T7.2 `bun run generate` passe
  - [x] T7.3 Login admin → /api/users OK
  - [x] T7.4 Login caissier → /api/users → 403 (via middleware permission)
  - [x] T7.5 Login caissier → /api/auth/me → 200 (juste JWT)
  - [x] T7.6 Créer, modifier, supprimer des utilisateurs (via API)
  - [x] T7.7 Self-delete → 400
  - [x] T7.8 Dernier admin protégé
  - [x] T7.9 Page /admin/users frontend fonctionnelle (build génère la route)
  - [x] T7.10 Navigation filtrée par rôle (minRole dans pages.ts)

## Review Findings

### Code Review — 2026-07-22

**Acceptance Auditor:** All 10 ACs satisfied. ✅

**Patch findings (fixable without human input):**

- [x] [Review][Patch] **Prefix collision dans required_permission** — FIXED [auth_middleware.rs:59-69]
  `path.starts_with("/api/users")` matche aussi `/api/users-export`. Pareil pour `/api/settings`
  → `/api/settings-backup`. Corriger en : `path == "/api/users" || path.starts_with("/api/users/")`
  et idem pour `/api/settings`, `/api/products`, `/api/orders`, `/api/payments`, `/api/wallet`,
  `/api/kitchen`, `/api/reports`, `/api/stock`.

- [x] [Review][Patch] **Page /settings accessible sans middleware admin** — FIXED [settings.vue:162]
  `minRole: "admin"` filtre la navigation mais n'empêche pas un caissier de taper `/settings`.
  Ajouter `middleware: ["auth", "admin"]` dans `definePageMeta`.

**Deferred findings:**

- [x] [Review][Defer] **Race condition DELETE dernier admin** [users.rs:241-250] —
  Deux suppressions admin simultanées peuvent passer. Scénario improbable en LAN alpha.

- [x] [Review][Defer] **Aucun test ajouté** — Le projet n'a pas de framework de test configuré.

- [x] [Review][Defer] **Pas de typage Zod pour les appels API frontend** [admin/users.vue] —
  Typage manuel suffit pour l'alpha.

- [x] [Review][Defer] **Pas de bouton "Réessayer" sur erreur chargement** [admin/users.vue] —
  UX mineure, l'utilisateur peut recharger la page.

**Dismissed (4 findings):** required_permission privée, /api/nonexistent envoie 403 au lieu de 404, race condition create_user protégée par UNIQUE BDD, minRole avec rôle inexistant hypothétique.

## Dev Notes

### Architecture Compliance

**AD-11 (Auth JWT + rôles)** : Story centrale pour AD-11. Les 4 rôles + permissions existent déjà dans `domain/user.rs` (stories 1.1, 1.3). Cette story ajoute la VÉRIFICATION de ces permissions dans le middleware HTTP.

**AD-8 (Erreurs 3 couches)** : Le 403 est une erreur de la couche API. Pas de nouveau variant DomainError. `(StatusCode::FORBIDDEN, Json({ error: "Forbidden", code: "FORBIDDEN" }))`.

**AD-7 (Traits repository)** : `UserRepository` déjà implémenté. Les nouveaux handlers l'utilisent directement.

**AD-10 (Stack alpha)** : Une seule fonction `required_permission()` dans le middleware. Pas de framework externe.

### Permission mapping (implémenté)

| Route | Permission |
|---|---|
| `/api/health` | 🔓 Publique |
| `/api/auth/login`, `/api/auth/register` | 🔓 Publique |
| `/api/auth/logout`, `/api/auth/me` | Authentifié (JWT) |
| `/api/settings` | ManageSettings |
| `/api/users` | ManageUsers |
| `/api/products`, `/api/categories` | ManageMenu |
| `/api/orders` | ViewOrders |
| `/api/payments` | Sell |
| `/api/wallet` | Sell |
| `/api/kitchen` | ViewOrders |
| `/api/reports` | ViewReports |
| `/api/stock` | ManageStock |
| Autres `/api/*` | Permission::All (admin) |

### Fichiers créés

```
src-tauri/src/api/users.rs              # CRUD handlers users
app/pages/admin/users.vue               # Page gestion utilisateurs
app/middleware/admin.ts                  # Middleware admin
```

### Fichiers modifiés

```
src-tauri/src/api/auth_middleware.rs     # required_permission() + vérification 403
src-tauri/src/api/mod.rs                 # Routes /api/users ajoutées
app/composables/pages.ts                 # Filtrage navigation par minRole
app/app.config.ts                        # Catégorie "admin" ajoutée
app/pages/settings.vue                   # minRole: "admin" dans definePageMeta
```

## Dev Agent Record

### Debug Log References

- `required_permission()` dans `auth_middleware.rs` : mappe path → `Option<Permission>`.
- `Role::has_permission()` dans `domain/user.rs` : check Permission::All pour bypass admin.
- `403 Forbidden` : `(StatusCode::FORBIDDEN, Json(ApiError { error: "Forbidden", code: "FORBIDDEN" }))`.
- DELETE self : `if auth.id == user_id → 400 "Cannot delete yourself"`.
- DELETE last admin : compte les admins avant suppression, refuse si <= 1.
- `PATCH /api/users/{id}` : param Axum 0.8 avec `Path(user_id): Path<String>`.
- Frontend navigation : `useAuth().user.value?.role` dans `usePages()` pour filtrer.
- `minRole` dans `definePageMeta` : utilisé par `usePages()` pour cacher les pages dans la nav.

### Implementation Notes

- **Permission middleware** : intégré dans `auth_middleware.rs` existant (pas de fichier séparé). La fonction `required_permission()` est appelée après la vérification JWT, avant l'insertion de `AuthUser` dans les extensions.
- **CRUD users** : validation identique à `auth.rs` (email format, password >= 8). Le `password_hash` n'est jamais exposé. Les routes sont sous `Permission::ManageUsers`.
- **Frontend admin/users** : utilise `UTable` pour la liste, `UModal` pour création/édition/suppression, `UToast` pour feedbacks. Le middleware `auth` + `admin` protège la route.
- **Navigation** : `usePages()` filtre les routes avec `minRole`. Seuls les admins et les utilisateurs avec le rôle exact voient les pages restreintes.

### File List

**NOUVEAUX :**
- `src-tauri/src/api/users.rs` — CRUD handlers users
- `app/pages/admin/users.vue` — Page gestion utilisateurs
- `app/middleware/admin.ts` — Middleware route admin

**MODIFIÉS :**
- `src-tauri/src/api/auth_middleware.rs` — Permission guard (required_permission, forbidden_response)
- `src-tauri/src/api/mod.rs` — Routes users, import module
- `app/composables/pages.ts` — Filtrage navigation par minRole
- `app/app.config.ts` — Catégorie admin
- `app/pages/settings.vue` — minRole: admin

### Change Log

- **2026-07-22** — Implémentation complète de la story 1.5
  - Ajout de `required_permission()` dans le middleware auth → vérification des permissions par route
  - 403 Forbidden retourné si rôle insuffisant (au lieu de 401 pour absence de JWT)
  - CRUD `/api/users` : list, create, update, delete avec validation + protection self-delete + dernier admin
  - Page `/admin/users` frontend : tableau, modaux, toasts, middleware admin
  - Navigation filtrée par rôle via `minRole` dans `definePageMeta`
  - `cargo check` + `bun run generate` passent sans erreur
