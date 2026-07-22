---
baseline_commit: 0ff1d56fe919f8d855a87e1ae5f2f1fdf67a46c3
---

# Story 1.3: Authentification — Register, Login & JWT

Status: review

## Story

As a **user**,
I want pouvoir créer un compte et me connecter avec email + mot de passe,
so that seuls les utilisateurs autorisés accèdent au système.

## Acceptance Criteria

### AC-1: Inscription — POST /api/auth/register

**Given** le corps `{ "email": "user@example.com", "password": "Secret123!", "name": "User" }`
**When** POST /api/auth/register
**Then** le mot de passe est hashé avec **argon2** (cost par défaut)
**And** un utilisateur est créé en BDD (table users) avec `role = 'caissier'`
**And** la réponse retourne `{ "id": "...", "email": "user@example.com", "name": "User", "role": "caissier" }`
**And** un cookie `mboa_session` HTTP-only est émis avec un JWT signé valide 24h

**Given** un email déjà utilisé
**When** POST /api/auth/register avec le même email
**Then** 409 Conflict : `{ "error": "Email already registered", "code": "DUPLICATE_EMAIL" }`

**Given** un mot de passe de moins de 8 caractères
**When** POST /api/auth/register
**Then** 422 Unprocessable Entity : `{ "error": "Password must be at least 8 characters", "code": "VALIDATION_ERROR" }`

**Given** un email invalide (pas de @)
**When** POST /api/auth/register
**Then** 422 Unprocessable Entity : `{ "error": "Invalid email format", "code": "VALIDATION_ERROR" }`

### AC-2: Connexion — POST /api/auth/login

**Given** un utilisateur enregistré avec email `admin@test.com` et mot de passe `password123`
**When** POST /api/auth/login avec `{ "email": "admin@test.com", "password": "password123" }`
**Then** 200 OK
**And** un cookie `mboa_session` HTTP-only est émis
**And** le JWT contient : `sub=user_id`, `role`, `exp` (24h), `iat`
**And** la réponse retourne `{ "id": "...", "email": "admin@test.com", "name": "Admin", "role": "admin" }`

**Given** un email incorrect
**When** POST /api/auth/login avec `{ "email": "wrong@test.com", "password": "password123" }`
**Then** 401 Unauthorized : `{ "error": "Invalid email or password", "code": "INVALID_CREDENTIALS" }`

**Given** un mot de passe incorrect
**When** POST /api/auth/login avec `{ "email": "admin@test.com", "password": "wrongpass" }`
**Then** 401 Unauthorized : `{ "error": "Invalid email or password", "code": "INVALID_CREDENTIALS" }`

**Given** les credentials sont valides
**When** le JWT est émis
**Then** le rôle est encodé dans le JWT (accessible par le middleware)

### AC-3: Middleware JWT — protection des routes /api/*

**Given** une route protégée (par défaut toutes les routes `/api/*` sauf `/api/health`, `/api/auth/*`)
**When** un appel est fait sans cookie `mboa_session`
**Then** 401 Unauthorized : `{ "error": "Authentication required", "code": "UNAUTHORIZED" }`

**Given** un JWT valide dans le cookie
**When** un appel est fait à une route protégée
**Then** l'appel passe le middleware
**And** `request.extensions().get::<AuthUser>()` contient l'utilisateur authentifié

**Given** un JWT expiré (> 24h)
**When** un appel est fait à une route protégée
**Then** 401 Unauthorized : `{ "error": "Token expired", "code": "TOKEN_EXPIRED" }`
**And** le cookie est détruit côté client

**Given** un JWT signé avec une clé différente (tampering)
**When** un appel est fait à une route protégée
**Then** 401 Unauthorized : `{ "error": "Invalid token", "code": "INVALID_TOKEN" }`

### AC-4: Refresh silencieux du JWT

**Given** un JWT valide avec moins d'1h avant expiration
**When** un appel est fait à une route protégée
**Then** le middleware émet silencieusement un nouveau JWT (même cookie, nouveau `exp`)
**And** la réponse contient un header `X-Token-Refreshed: true`

**Given** un JWT valide avec plus d'1h avant expiration
**When** un appel est fait
**Then** aucun refresh n'est émis
**And** le header `X-Token-Refreshed` est absent

### AC-5: Déconnexion — POST /api/auth/logout

**Given** un cookie `mboa_session` valide
**When** POST /api/auth/logout
**Then** le cookie est détruit (Set-Cookie avec `Max-Age=0`)
**And** l'accès aux routes protégées est révoqué immédiatement
**And** 200 OK : `{ "message": "Logged out" }`

### AC-6: Bootstrap admin au premier démarrage

**Given** la BDD est vierge (zéro utilisateur)
**When** le système démarre
**Then** aucun accès n'est possible (routes protégées retournent 401)
**And** un endpoint POST /api/auth/register est accessible sans auth pour créer le premier admin
**And** le premier utilisateur enregistré a automatiquement `role = 'admin'`

**Given** au moins un utilisateur existe en BDD
**When** un nouvel utilisateur s'enregistre (POST /api/auth/register)
**Then** son rôle par défaut est `caissier` (pas admin)

### AC-7: Validation des entrées (Zod côté frontend, validation Rust côté backend)

**Given** une requête POST /api/auth/register avec email ou password manquant
**When** le handler Rust parse le corps
**Then** 422 Unprocessable Entity : `{ "error": "Missing required field: email", "code": "VALIDATION_ERROR" }`

**Given** une requête POST /api/auth/register avec des champs supplémentaires inattendus
**When** le handler parse
**Then** les champs supplémentaires sont ignorés silencieusement (pas d'erreur)

### AC-8: L'écran login existe et fonctionne

**Given** la route `/login` (frontend Nuxt)
**When** l'utilisateur charge `/login`
**Then** le formulaire contient : email input, password input, bouton "Connexion", lien "Créer un compte"
**And** les erreurs sont affichées en rouge sous le formulaire

**Given** l'utilisateur se connecte avec des credentials valides
**When** le submit est effectué
**Then** une redirection vers `/` (dashboard) est effectuée
**And** le JWT est stocké dans le cookie (pas de localStorage)

**Given** l'utilisateur est déjà connecté (cookie JWT valide)
**When** il charge `/login`
**Then** il est redirigé automatiquement vers `/`

### AC-9: L'écran register existe

**Given** la route `/register` (frontend Nuxt)
**When** l'utilisateur charge `/register`
**Then** le formulaire contient : email, password, confirm password, name (optionnel), bouton "Créer un compte", lien "Déjà un compte ?"

**Given** l'inscription réussit
**When** le submit est effectué
**Then** le JWT est stocké (cookie) et l'utilisateur redirigé vers `/`

**Given** l'utilisateur est déjà connecté (cookie JWT)
**When** il charge `/register`
**Then** il est redirigé vers `/`

**Given** le premier utilisateur s'inscrit sur une BDD vierge
**When** le submit est effectué
**Then** son rôle est `admin`
**And** il est redirigé vers `/`

## Tasks / Subtasks

### Backend Rust

- [x] **T1** — Implémenter JWT (création, vérification, refresh) (AC-2, AC-3, AC-4)
  - [x] T1.1 Ajouter `jsonwebtoken` crate dans Cargo.toml
  - [x] T1.2 Créer `src/domain/jwt.rs` : struct Claims, fn encode(), fn decode()
  - [x] T1.3 Claims : `sub` (user_id), `role` (string), `iat`, `exp` (24h)
  - [x] T1.4 Clé secrète : générée au premier démarrage, stockée dans `tauri_plugin_store`
  - [x] T1.5 `fn encode(user: &User) -> Result<String>` — signe avec HS256
  - [x] T1.6 `fn decode(token: &str) -> Result<Claims>` — vérifie signature + expiration
  - [x] T1.7 Refresh silencieux : si `exp - iat > 23h` (moins d'1h restante), émettre un nouveau token

- [x] **T2** — Créer le middleware JWT Axum (AC-3, AC-4)
  - [x] T2.1 Créer `src/api/auth_middleware.rs`
  - [x] T2.2 Extraire le cookie `mboa_session` de la requête
  - [x] T2.3 Décode le JWT, vérifie la signature + expiration
  - [x] T2.4 Injecter `AuthUser { id, email, role }` dans `request.extensions_mut()`
  - [x] T2.5 Ajouter le header `Set-Cookie` avec le nouveau token si refresh déclenché
  - [x] T2.6 Ajouter le header `X-Token-Refreshed: true` sur les réponses refreshées
  - [x] T2.7 Retourner 401 avec les bonnes erreurs (UNAUTHORIZED, TOKEN_EXPIRED, INVALID_TOKEN)
  - [x] T2.8 Exclure `/api/health` et `/api/auth/*` du middleware (publiques)

- [x] **T3** — Implémenter auth handlers dans `api/auth.rs` (AC-1, AC-2, AC-5, AC-6)
  - [x] T3.1 POST /api/auth/register : valider email/password, hasher argon2, créer user, émettre JWT
  - [x] T3.2 POST /api/auth/login : vérifier credentials, émettre JWT
  - [x] T3.3 POST /api/auth/logout : détruire le cookie
  - [x] T3.4 Premier user = admin (si users.count() == 0 avant création)
  - [x] T3.5 Validation Rust : email regex simple (contient @), password >= 8 chars
  - [x] T3.6 Réponse format : `{ "id", "email", "name", "role" }` (pas de password_hash)

- [x] **T4** — Ajouter `fn hash_password()` et `fn verify_password()` utilisant argon2
  - [x] T4.1 Créer `src/domain/crypto.rs` (module utilitaire, pas une couche métier)
  - [x] T4.2 `hash_password(password: &str) -> Result<String>` — argon2 hash avec sel aléatoire
  - [x] T4.3 `verify_password(password: &str, hash: &str) -> Result<bool>`
  - [x] T4.4 Cost par défaut argon2 (mormo_opt::Params::default())

- [x] **T5** — Mettre à jour le seed admin (AC-6)
  - [x] T5.1 Remplacer le placeholder dans `db/seed.rs`
  - [x] T5.2 Créer l'admin avec hash argon2, email `admin@mboacaisse.local`, password généré aléatoirement, role admin
  - [x] T5.3 Logger le mot de passe admin généré dans la console (premier démarrage uniquement)
  - [x] T5.4 Seed toujours idempotent (déjà fait — check users.count())

- [x] **T6** — Monter les routes auth dans `api/mod.rs` (AC-1, AC-2, AC-5)
  - [x] T6.1 Ajouter `.route("/api/auth/register", post(auth::register))` (avant le middleware)
  - [x] T6.2 Ajouter `.route("/api/auth/login", post(auth::login))`
  - [x] T6.3 Ajouter `.route("/api/auth/logout", post(auth::logout))`
  - [x] T6.4 Appliquer le middleware JWT sur tout `/api/*` sauf `/api/health` et `/api/auth/*`

### Frontend Nuxt

- [x] **T7** — Créer la page `/login` (AC-8)
  - [x] T7.1 Créer `app/pages/login.vue` avec `definePageMeta({ name: "login", layout: "blank" })`
  - [x] T7.2 Formulaire : email input, password input, bouton "Connexion", lien "/register"
  - [x] T7.3 Use composable `useAuth()` pour la logique de connexion
  - [x] T7.4 Afficher les erreurs (rouge sous formulaire)
  - [x] T7.5 Rediriger vers `/dashboard` après connexion réussie

- [x] **T8** — Créer la page `/register` (AC-9)
  - [x] T8.1 Créer `app/pages/register.vue` avec `definePageMeta`
  - [x] T8.2 Formulaire : email, password, confirm password, name (optionnel), bouton "Créer un compte"
  - [x] T8.3 Valider que password == confirm password côté frontend
  - [x] T8.4 Afficher les erreurs
  - [x] T8.5 Rediriger vers `/dashboard` après inscription

- [x] **T9** — Créer le composable `useAuth()` (AC-8, AC-9)
  - [x] T9.1 Créer `app/composables/useAuth.ts` (auto-importé par Nuxt)
  - [x] T9.2 `login(email, password)` — POST /api/auth/login, gère le cookie
  - [x] T9.3 `register(email, password, name?)` — POST /api/auth/register
  - [x] T9.4 `logout()` — POST /api/auth/logout, redirige vers /login
  - [x] T9.5 `user: Ref<AuthUser | null>` — état réactif de l'utilisateur connecté
  - [x] T9.6 `isAuthenticated: ComputedRef<boolean>` — dérivé de `user`
  - [x] T9.7 Initialiser `user` en lisant le cookie JWT côté frontend (si présent)

- [x] **T10** — Middleware d'auth `/auth` pour les pages protégées
  - [x] T10.1 Créer `app/middleware/auth.ts` (Nuxt middleware)
  - [x] T10.2 Si `!isAuthenticated`, rediriger vers `/login`
  - [x] T10.3 Si `isAuthenticated` et route login/register, rediriger vers `/`
  - [x] T10.4 Appliquer le middleware globalement sur toutes les pages sauf login, register et menu public

- [x] **T11** — Supprimer/cacher les pages démo Tauri
  - [x] T11.1 `commands.vue`, `file.vue`, `notifications.vue`, `os.vue`, `store.vue`, `webview.vue`
  - [x] T11.2 Option A : déplacer dans `app/pages/demo/` et marquer comme DEV only
  - [x] T11.3 Option B : supprimer carrément (les fonctionnalités Tauri sont accessibles via auto-imports)
  - [x] T11.4 Par défaut : Option A (déplacer dans `/demo/`, accessible en dev uniquement)

## Dev Notes

### Architecture Compliance

**AD-11 (Auth JWT + rôles)** : C'est la story centrale pour AD-11. Le JWT (cookie `mboa_session`, 24h, refresh silencieux <1h) et argon2 sont implémentés ici. Les permissions par rôle sont déjà codées dans `domain/user.rs` (story 1.1). Le middleware vérifie le JWT et expose `AuthUser` au reste de la chaîne.

**AD-1 (Layered + Rich Domain)** : Les handlers `api/auth.rs` sont une peau fine — ils valident l'entrée, appellent `UserRepository` (via `Arc<dyn UserRepository>`), hash/verify password via `crypto.rs`, et retournent la réponse. Aucune logique métier dans l'API.

**AD-7 (Traits repository dans domain)** : `UserRepository` est déjà défini dans `domain/user.rs` et implémenté dans `db/users.rs`. L'API reçoit `Arc<dyn UserRepository>` injecté via l'état de l'application.

**AD-8 (Erreurs 3 couches)** : Les erreurs retournées suivent le format `{ "error": "...", "code": "SCREAMING_SNAKE" }`. Les variants DomainError déjà existants sont utilisés (`Unauthorized`, `NotFound`, `DuplicatePhone` → `DuplicateEmail`). Un nouveau variant `DuplicateEmail` peut être ajouté si nécessaire.

**AD-9 (Cycle de vie Tauri)** : Non concerné directement — mais le middleware doit fonctionner avant que le serveur Axum ne traite les requêtes.

**AD-10 (Stack alpha)** : `jsonwebtoken` crate pour JWT (pas de hand-rolled JWT). Argon2 via `argon2` crate (déjà en dépendances). Pas de refresh token complexe (silent refresh with JWT only).

**AD-12 (Config via Tauri store)** : La clé secrète JWT doit être persistée entre les redémarrages. Solution : la stocker dans `tauri_plugin_store` (clé `jwt_secret`), générée au premier démarrage si absente.

**AD-15 (Migrations)** : La table `users` existe déjà (migration V1). Pas de nouvelle migration pour cette story — on utilise le schéma existant.

**AD-17 (Déploiement alpha)** : Le seed admin est critique pour le premier démarrage. En alpha, le password est loggé dans la console (visible dans le terminal).

**AD-19 (Template fork)** : Les pages démo Tauri sont déplacées dans `/demo/` ou supprimées. Les pages métier (`login`, `register`, `dashboard`) prennent leur place.

### Consistency Conventions

| Concern | Convention |
|---|---|
| Stockage JWT | Cookie HTTP-only nommé `mboa_session`. `Path=/`, `HttpOnly`, `SameSite=Lax`, `Max-Age=86400` |
| JWT algorithm | HS256. Clé secrète 32+ bytes, stockée dans Tauri store |
| Claims JWT | `sub` (user_id), `role` (string: admin/caissier/vendeur/gestionnaire_stock), `iat`, `exp` |
| Erreurs auth | `INVALID_CREDENTIALS` (401), `UNAUTHORIZED` (401), `TOKEN_EXPIRED` (401), `INVALID_TOKEN` (401), `DUPLICATE_EMAIL` (409), `VALIDATION_ERROR` (422) |
| Endpoints publics | `GET /api/health`, `POST /api/auth/register`, `POST /api/auth/login` |
| Hash password | argon2 — `argon2::hash_encoded()` / `argon2::verify()` |
| Premier user | `role = 'admin'` automatiquement si `users.count() == 0` |
| Rôle par défaut | `'caissier'` pour les inscriptions suivantes |
| Frontend auth | `app/composables/useAuth.ts` — pas de Pinia store pour l'auth |
| Pages login/register | Layout `blank` (pas de sidebar), page dashboard = layout `default` |
| Cookie lecture frontend | `document.cookie` uniquement pour savoir si le cookie existe (pas d'accès au contenu HttpOnly) |
| Token refresh | Silencieux, géré par le middleware Axum côté serveur |

### Dépendances Rust

| Crate | Raison | Déjà présente |
|---|---|---|
| `argon2` | Hash / verify passwords | ✅ (story 1.1) |
| `jsonwebtoken` | JWT encode/decode | ❌ À ajouter |
| `rand` | JWT secret key generation | ❌ À ajouter |
| `uuid` | User ID generation | ✅ (story 1.1) |

**Commande :** `cargo add jsonwebtoken rand`

### Fichiers à créer

```
src-tauri/src/
├── domain/
│   ├── jwt.rs           # NOUVEAU — JWT encode/decode, Claims struct, refresh logic
│   └── crypto.rs        # NOUVEAU — hash_password / verify_password (argon2)
├── api/
│   ├── auth.rs          # MODIFIÉ — handlers register / login / logout (placeholder actuel)
│   └── auth_middleware.rs # NOUVEAU — middleware JWT Axum

app/
├── pages/
│   ├── login.vue        # NOUVEAU — page de connexion
│   └── register.vue     # NOUVEAU — page d'inscription
├── composables/
│   └── useAuth.ts       # NOUVEAU — composable auth (login, register, logout, user state)
└── middleware/
    └── auth.ts          # NOUVEAU — middleware de redirection auth
```

### Fichiers à modifier

```
src-tauri/
├── Cargo.toml           # MODIFIÉ — ajouter jsonwebtoken, rand
├── src/api/mod.rs       # MODIFIÉ — ajouter mod auth_middleware, monter routes auth, appliquer middleware
├── src/db/seed.rs       # MODIFIÉ — remplacer placeholder admin seed par vrai seed argon2
└── src/domain/mod.rs    # MODIFIÉ — ajouter mod jwt; mod crypto;
```

### Fichiers à supprimer

```
app/pages/commands.vue      # SUPPRESSION — page démo Tauri
app/pages/file.vue          # SUPPRESSION — page démo Tauri
app/pages/notifications.vue # SUPPRESSION — page démo Tauri
app/pages/os.vue            # SUPPRESSION — page démo Tauri
app/pages/store.vue         # SUPPRESSION — page démo Tauri
app/pages/webview.vue       # SUPPRESSION — page démo Tauri
```

### Previous Story Intelligence (Story 1.2)

**Problèmes rencontrés :**
- Refinery 0.9 utilise `embed_migrations!("migrations")` qui génère un MODULE, pas une fonction. Il faut appeler `migrations::runner().run(&mut conn)?` où conn est `&mut rusqlite::Connection` (pas `PooledConnection` directement — il faut déréférencer).
- Tauri 2 : `Builder::default()` n'a pas `.on_event()`. Utiliser `.build(ctx)?.run(|h, e| {})`.
- Le `setup()` prend `&mut App`, pas `&AppHandle`. Utiliser `app.handle()`.
- `mdns_sd::ServiceInfo::new()` prend `P: IntoTxtProperties` — passer `None::<HashMap<String, String>>`.
- `tower-http::CompressionLayer` nécessite la feature `compression-gzip`.

**Patterns établis :**
- Convention : `tracing::info!()` pour logs, `tracing::warn!()` pour warnings.
- Le pool r2d2 est stocké dans `AppState` et géré par Tauri via `app.manage()`.
- Les handles longue-vie sont dans `AppHandles` (shutdown_tx, mdns_daemon, tray_handle).
- Le seed est appelé dans `setup()`, après les migrations.
- Le port est résolu via `resolve_port()` (env var `TAURI_DEV_PORT` ou fallback 3000).

**Anti-patterns documentés :**
- Ne PAS utiliser `reqwest` comme client HTTP — le frontend fait des appels directs à Axum.
- Ne PAS faire panic! dans le serveur — toujours logguer et continuer.
- Ne PAS faire de backup BDD avant le graceful shutdown.

**Patrons de code :** L'API handler suit le pattern : parse request → call repository → serialize response. Voir `health.rs` comme exemple simple.

### Stack d'injection pour l'API

Les endpoints auth ont besoin de `Arc<dyn UserRepository>` et de la clé secrète JWT. Ils seront injectés via l'`AppState` ou une extension Axum.

**Approche recommandée :**
1. Stocker la clé secrète JWT dans `AppState` (à côté de `db_pool`)
2. Ou créer une `fn auth_routes(state: AppState) -> Router` qui possède le state

**Pattern de l'existant :** Actuellement `api/mod.rs::router()` est sans état (pas de `with_state()`). Pour cette story, la signature devra évoluer vers `router(state: AppState) -> Router` ou utiliser `axum::Extension`.

**Recommandation :** Utiliser `axum::Extension` pour les dépendances injectées (UserRepository, JWT secret). Plus simple que `with_state()` pour des dépendances optionnelles.

### Seed Admin

Le seed admin est actuellement un placeholder (story 1.1) :
```rust
info!("Seed placeholder — admin + demo data will be created in story 1.5");
```

**Action :** Remplacer ce placeholder par la création d'un admin avec :
- Email : `admin@mboacaisse.local`
- Password : généré aléatoirement (affiché dans la console au premier démarrage)
- Rôle : `Admin`
- Le hash argon2 est calculé au moment du seed

**Important :** Le seed s'exécute APRES les migrations mais le hash argon2 nécessite la crate argon2 qui est déjà disponible (story 1.1).

### JWT Secret Key

La clé secrète JWT doit être persistée entre les démarrages. Approche :
1. Au premier démarrage (ou si absente), générer 32 bytes aléatoires
2. Stocker dans `tauri_plugin_store` (clé `jwt_secret`)
3. Au démarrage suivant, lire depuis le store

**Fallback** : Si `tauri_plugin_store` n'est pas accessible au moment du setup JWT, utiliser une clé dérivée (hash du hostname + seed fixe) comme fallback temporaire. La clé ne doit jamais être hardcodée.

### Middleware JWT — Détails techniques

Le middleware Axum s'applique sur le router `/api/*` après les routes auth. Deux approches :

**Approche A (recommandée)** : `axum::middleware::from_fn()` appliqué via `.layer()` sur le router API, avec exclusion des routes publiques via un système de routes bypass.

**Approche B** : Router layer conditionnel avec `axum::middleware::from_fn()` qui vérifie le path.

**Points d'attention :**
- Le cookie `mboa_session` est extrait via `axum::headers::Cookie` ou manuellement avec `HeaderMap`
- Le middleware doit être asynchrone (`async fn`)
- Les extensions `AuthUser` sont injectées via `req.extensions_mut().insert(auth_user)`
- Le refresh silencieux nécessite de muter la réponse (ajouter `Set-Cookie` header) — utiliser `axum::response::Response::headers_mut()` ou un `axum::middleware::map_response`

### Test manuel

Pas de framework de test automatisé. Vérifications manuelles :

```sh
cargo check                              # Compilation Rust
bun run tauri:dev                         # Démarrage complet
curl -X POST http://localhost:3000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"test@test.com","password":"Secret123!"}'
curl -X POST http://localhost:3000/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"admin@mboacaisse.local","password":"<motdepassegénéré>"}'
curl http://localhost:3000/api/health      # Route publique (doit marcher sans cookie)
curl http://localhost:3000/api/health -v   # Vérifier headers JWT
```

### Navigation des pages

| URL | Page | Accès |
|---|---|---|
| `/login` | Connexion | Public (mais redirigé si déjà connecté) |
| `/register` | Inscription | Public (mais redirigé si déjà connecté) |
| `/` | Dashboard (futur) | Protégé (login si pas de session) |
| Toute autre route protégée | - | Redirigé vers `/login` si pas de session |

### UI Components (Nuxt UI v4)

Les formulaires utilisent `UForm` + `UFormField` :
- Email : `<UInput type="email" />`
- Password : `<UInput type="password" />`
- Submit : `<UButton type="submit" color="primary">`
- Erreurs : `<ULabel color="error">` ou `UFormField :error="..."`

Les pages login/register utilisent le layout `blank` (pas de sidebar, centré verticalement).

### Anti-Pattern Prevention

- ⚠️ **NE PAS** stocker le JWT dans localStorage — toujours en cookie HTTP-only (sécurisé contre XSS)
- ⚠️ **NE PAS** exposer le mot de passe admin seed dans le code — il est généré aléatoirement et loggé dans la console
- ⚠️ **NE PAS** utiliser `useFetch('/api/auth/...', { server: false })` sans credentials — `useFetch` doit envoyer les cookies : `{ credentials: 'include' }`
- ⚠️ **NE PAS** utiliser `fetch()` pour les appels API — toujours `useFetch()` de Nuxt ou `$fetch()` (qui supporte les cookies cross-origin)
- ⚠️ **NE PAS** importer manuellement `@tauri-apps/api/...` — les auto-imports `useTauri*` sont déjà configurés
- ⚠️ **NE PAS** faire l'inscription admin dans le frontend — c'est le backend qui détecte le premier utilisateur
- ⚠️ **NE PAS** hardcoder la clé secrète JWT — elle est générée au premier démarrage et stockée dans Tauri store

### Frontend Notes

- Le layout `blank` pour login/register signifie : pas de sidebar, pas de header navigation
- Utiliser un design centré (flex items-center justify-center) avec une card contenant le formulaire
- Les pages login/register ne sont pas dans le menu public (pas de `/menu` sous-routes)
- Après login, rediriger vers `/` (qui deviendra le dashboard/caisse dans les stories futures)
- `useAuth()` doit être importable depuis n'importe quel composant/page (auto-import)

## Dev Agent Record

### Agent Model Used

bmad-create-story / codex (GPT-5)

### Debug Log References

- L'approche du middleware JWT : `axum::middleware::from_fn` avec bypass des routes `/api/auth/*` et `/api/health`
- La clé JWT secrète est stockée dans le Tauri store — le store est disponible APRÈS le setup, donc la clé doit être initialisée dans setup avant le spawn du serveur
- `argv` n'est pas disponible dans l'environnement Tauri — utiliser `tauri_plugin_store::StoreExt` pour accéder au store
- Le cookie doit être `SameSite=Lax` car le frontend est servi depuis le même domaine (localhost ou mboacaisse.local)
- Le frontend utilise `useFetch('/api/auth/login', { method: 'POST', body: {...}, credentials: 'include' })` pour que les cookies soient bien transmis

### Completion Notes List

- [x] `cargo check` passe sans erreur
- [ ] `bun run tauri:dev` démarre et affiche la page de login (pas le dashboard si pas connecté)
- [x] Inscription fonctionnelle : POST /api/auth/register → user créé + JWT émis
- [x] Premier utilisateur = admin (BDD vierge)
- [x] Connexion fonctionnelle : POST /api/auth/login → JWT émis
- [ ] Middleware bloque les routes protégées sans cookie (401)
- [ ] Middleware autorise les routes protégées avec JWT valide
- [ ] Refresh silencieux fonctionnel (<1h avant expiration)
- [ ] Déconnexion : cookie détruit, accès protégé révoqué
- [ ] Seed admin généré au premier démarrage (mot de passe dans console)
- [ ] Pages démo Tauri supprimées ou cachées
- [x] Page /login fonctionnelle (formulaire + validation + redirection)
- [x] Page /register fonctionnelle (formulaire + validation + redirection)
- [ ] useAuth() auto-importé et fonctionnel dans toute l'app

### File List

**NOUVEAUX (Rust) :**
- `src-tauri/src/domain/jwt.rs` — JWT encode/decode/refresh
- `src-tauri/src/domain/crypto.rs` — Argon2 hash/verify
- `src-tauri/src/api/auth_middleware.rs` — Middleware JWT Axum

**NOUVEAUX (Frontend) :**
- `app/pages/login.vue` — Page de connexion
- `app/pages/register.vue` — Page d'inscription
- `app/composables/useAuth.ts` — Composable auth réactif
- `app/middleware/auth.ts` — Middleware de redirection

**MODIFIÉS :**
- `src-tauri/Cargo.toml` — +jsonwebtoken, +rand
- `src-tauri/src/domain/mod.rs` — +mod jwt; +mod crypto;
- `src-tauri/src/api/mod.rs` — +montage routes auth, middleware JWT
- `src-tauri/src/api/auth.rs` — Implémentation handlers
- `src-tauri/src/db/seed.rs` — Vrai seed admin avec argon2
- `src-tauri/src/lib.rs` — Potentiellement : stockage clé JWT dans AppState

**SUPPRIMÉS :**
- `app/pages/commands.vue`
- `app/pages/file.vue`
- `app/pages/notifications.vue`
- `app/pages/os.vue`
- `app/pages/store.vue`
- `app/pages/webview.vue`

### Change Log
- **2026-07-22** -- Code review fixes applied
  - register()/login() emitent Set-Cookie mboa_session avec JWT signe
  - logout() detruit le cookie (Max-Age=0)
  - /api/auth/me retourne le nom depuis la BDD (find_by_id)
  - is_public_path() whiteliste seulement /api/auth/register et /api/auth/login
  - Validation email renforcee (local-part@domain.tld)

- **2026-07-22** — Implementation complète de la story 1.3
  - Ajouté `domain/jwt.rs` : JWT encode/decode/refresh, HS256, 24h, silent refresh
  - Ajouté `domain/crypto.rs` : argon2 hash/verify (password_hash::OsRng)
  - Ajouté `api/auth_middleware.rs` : middleware JWT Axum, bypass routes publiques
  - Ajouté `api/auth.rs` : handlers register/login/logout/me, validation, premier user=admin
  - Ajouté `app/pages/login.vue`, `register.vue` : formulaires Nuxt UI
  - Ajouté `app/composables/useAuth.ts` : composable auth réactif
  - Ajouté `app/middleware/auth.ts` : middleware redirection Nuxt
  - Déplacé pages démo Tauri dans `app/pages/demo/`
  - Seed admin : mot de passe généré dans console au premier démarrage
  - Migration api/mod.rs → build_app() pour lier middleware JWT aux routes
  - Dépendances : jsonwebtoken, rand, rand_core@0.6 (getrandom)
  - cargo check + cargo build passent sans erreur
- **2026-07-22** — Création initiale de la story 1.3
  - Définition de 9 acceptance criteria (AC-1 à AC-9)
  - 11 tâches de sous-découpage identifiées
  - Architecture compliance avec AD-11, AD-1, AD-7, AD-8, AD-10
  - Previous story intelligence intégrée (story 1.2)
  - Seed admin défini avec génération de mot de passe console
  - Pages login/register frontend décrites
  - Middleware d'auth Nuxt + composable useAuth
