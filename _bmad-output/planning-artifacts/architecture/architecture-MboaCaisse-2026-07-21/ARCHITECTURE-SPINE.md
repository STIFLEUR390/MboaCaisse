---
name: MboaCaisse
type: architecture-spine
purpose: build-substrate
altitude: feature
paradigm: Layered + Rich Domain
scope: MboaCaisse plateforme alpha complète
status: draft
created: 2026-07-21
updated: 2026-07-21
binds: [build]
sources:
  - FEATURES.md
  - docs/architecture-mboacaisse.md
  - docs/systeme-de-licences.md
  - planning-artifacts/prds/prd-MboaCaisse-2026-07-21/prd.md
  - planning-artifacts/ux-designs/ux-MboaCaisse-2026-07-21/EXPERIENCE.md
  - .ai-memory/index.md
companions:
  - ACTUAL-STATE.md (gap analysis, delivery roadmap)
---

# Architecture Spine — MboaCaisse

## Design Paradigm

**Layered + Rich Domain.**

```
Frontend (Nuxt 4 / Vue 3 / Nuxt UI v4)
  │  HTTP (LAN, <10ms)
  ▼
api/  (peau fine : parse → appelle domain → sérialise)
  │
  ▼
domain/  (comportement métier, traits repository, DomainError enum)
  │
  ▼
db/  (repositories impl, rusqlite, r2d2 pool, refinery migrations)
```

- `domain/` contient le comportement, pas des structs anémiques. Un `Order` a une méthode `pay()` qui vérifie le solde, appelle le ledger, change le statut.
- `api/` est une peau fine : parse la requête, appelle le domaine via `Arc<dyn XxxRepository>`, sérialise la réponse.
- `db/` implémente les traits définis dans `domain/`. DbError ne sort jamais de cette couche.
- Pas d'hexagonal : boilerplate injustifié pour équipe 1-2 devs, infra stable (Axum, SQLite, USB), risque = logique métier, pas changement d'infra.

## Actual State (2026-07-21)

Le projet est au stade **template/fork Nuxtor**. Aucune logique métier Rust ou frontend n'est implémentée.

```
src-tauri/
└── src/
    ├── main.rs      # entry Tauri
    └── lib.rs       # Tauri builder + plugins (shell, notification, os, fs, store)
                     # PAS de Axum, PAS de DB, PAS de domain

app/                 # Nuxt 4
└── pages/
    ├── index.vue    # Landing page Nuxtor template
    ├── commands.vue # Tauri shell demo
    ├── file.vue     # Tauri fs demo
    ├── notifications.vue # Tauri notification demo
    ├── os.vue       # Tauri OS info demo
    ├── store.vue    # Tauri store demo
    ├── webview.vue  # Tauri webview demo
    └── [...all].vue # 404
```

**Rust backend réel** (Cargo.toml) : `tauri 2.9`, `tauri-plugin-{shell,notification,os,fs,store}`, `serde`, `serde_json`. **Aucune** des dépendances listées dans la Stack cible (Axum, Tokio, rusqlite, r2d2, refinery, argon2, mdns-sd, tracing) n'est présente.

**Frontend réel** : Nuxt 4 + Nuxt UI v4 + VueUse + Zod + reka-ui. Pages Tauri API demo uniquement. Pas de pages métier (auth, caisse, cuisine, stock, rapports).

Le spine ci-dessous décrit l'**architecture cible** à construire. Voir `companions/ACTUAL-STATE.md` pour le plan de delivery.

---

## Invariants & Rules

```mermaid
graph TD
    subgraph api/
        A[auth.rs]
        P[products.rs]
        O[orders.rs]
        PY[payments.rs]
        W[wallet.rs]
        K[kitchen.rs]
        R[reports.rs]
    end
    subgraph domain/
        DU[user.rs]
        DP[product.rs]
        DO[order.rs]
        DPY[payment.rs]
        DW[wallet.rs]
        DPJ[print_job.rs]
    end
    subgraph db/
        DUM[users.rs]
        DPM[products.rs]
        DOM[orders.rs]
        DPYM[payments.rs]
        DLM[wallet_ledger.rs]
    end
    A --> DU
    P --> DP
    O --> DO
    O --> DW
    PY --> DO
    PY --> DW
    PY --> DPJ
    W --> DW
    K --> DO
    R -.->|SQL directe| DPM
    R -.->|SQL directe| DOM
    R -.->|SQL directe| DLM
    DO --> DPM
    DO --> DLM
    DPY --> DLM
```

### AD-1 — Paradigme Layered + Rich Domain

- **Binds:** `all`
- **Prevents:** Domain anémique (structs sans méthodes), hexagonal boilerplate, dépendances remontant de db/ vers api/
- **Rule:** `domain/` contient le comportement métier. Les méthodes d'aggregat prennent `dyn Repository` en paramètre. `api/` ne contient pas de logique. `db/` implémente les traits.
- **Statut:** 🔧 À construire (backend Rust inexistant)

### AD-2 — Append-only financier (pattern système)

- **Binds:** `wallet`, futures `factures`, `commissions`, `payroll`
- **Prevents:** UPDATE/DELETE sur données financières, perte de traçabilité, race condition sur solde concurrent
- **Rule:** Toute table à valeur financière est INSERT-only. `wallet_ledger` = append-only, backup toutes les 5 min. Pattern s'étend à toute nouvelle feature financière. **Tout calcul de solde (SELECT SUM) + INSERT est fait dans une même transaction SQL (BEGIN → SELECT → INSERT → COMMIT).** Jamais de read-then-write en deux requêtes séparées.
- **Statut:** 🔧 Design validé, pas encore implémenté

### AD-3 — Structure plate par couche

- **Binds:** `all`
- **Prevents:** Arborescence profonde (navigation lente pour petite équipe)
- **Rule:** Fichiers directement dans `api/`, `domain/`, `db/`. Pas de sous-dossiers par module. Profondeur ajoutée quand >15 fichiers par dossier.
- **Note:** Divergence avec `FEATURES.md §P0.5` qui propose organisation par domaine métier. **AD-3 prévaut** — la structure plate est plus simple pour équipe 1-2 devs.
- **Statut:** ✅ Structure cible définie

### AD-4 — Payment et Wallet séparés

- **Binds:** `payment`, `wallet`
- **Prevents:** Fusion du compte client et de l'acte d'encaissement
- **Rule:** Wallet = comptes + ledger + identification téléphone. Payment = encaissement + multi-moyen + validation + écriture ledger. Payment appelle Wallet. Wallet n'appelle jamais Payment.
- **Statut:** 🔧 Design validé, pas encore implémenté

### AD-5 — Print = service transverse

- **Binds:** `payment`
- **Prevents:** Logique d'impression dispersée dans api/domain/db
- **Rule:** `src/print.rs`. File d'attente asynchrone + writer ESC/POS. Appelé par Payment. Ne bloque jamais la commande.
- **Statut:** 🔧 Reporté P2.1 (FEATURES.md). Ticket numérique comme fallback immédiat.

### AD-6 — Reports = lecture seule

- **Binds:** `reports`
- **Prevents:** Duplication logique métier dans les rapports
- **Rule:** `api/reports.rs`. Queries SQL directes sur toutes les tables. Retourne DTOs de présentation. Pas de fichier dans `domain/` pour les rapports.
- **Statut:** 🔧 Design validé, pas encore implémenté

### AD-7 — Traits repository dans domain

- **Binds:** `all`
- **Prevents:** Fuite de dépendance db/ vers domain/ (ex: SQLx importé dans une struct métier)
- **Rule:** `domain/` définit `trait XxxRepository { ... }`. `db/` implémente. `api/` prend `Arc<dyn XxxRepository>`.
- **Statut:** 🔧 Design validé

### AD-8 — Erreurs 3 couches sans fuite

- **Binds:** `all`
- **Prevents:** Erreur SQL qui remonte à l'API, perte de contexte métier
- **Rule:**
  - `db/` → `Result<T, DbError>` (interne, ne sort pas)
  - `domain/` → `Result<T, DomainError>` (enum: InsufficientBalance, ProductNotFound, InvalidStatusTransition, DuplicatePhone, ...)
  - `api/` → `(StatusCode, Json<ApiError>)` avec `{"error": "...", "code": "..."}`. Status code = HTTP standard (200/201/400/401/403/422/500). Pas d'enveloppe `{ok}` générique.
- **Statut:** 🔧 Design validé

### AD-9 — Cycle de vie Tauri → Axum → backup

- **Binds:** `lib.rs`, `main.rs`
- **Prevents:** Corruption BDD à la fermeture, requêtes en vol perdues brutalement
- **Rule:** `on_event(ExitRequested)` → shutdown_tx → Axum graceful shutdown → backup BDD. Timeout 5s sur le backup. Mieux vaut perdre un backup qu'une corruption.
- **Statut:** 🔧 Pas encore implémenté (lib.rs actuel n'a que tray icon + plugins)

### AD-10 — Stack alpha

- **Binds:** `all`
- **Prevents:** Sur-ingénierie, dépendances instables
- **Rule:** Rust edition 2021. Tokio/Axum versions flottantes (lockfile gèle). rusqlite + r2d2. refinery migrations. TanStack Query supprimé (useFetch() Nuxt suffit en LAN <10ms). Pas de WebSocket en V1. Nuxt 4 / Vue 3 / Pinia / TailwindCSS v4.
- **Note:** `FEATURES.md` et `docs/architecture-mboacaisse.md` mentionnent encore TanStack Query et SQLx — **AD-10 prévaut**: rusqlite (pas SQLx), useFetch (pas TanStack Query).
- **Statut:** 🔧 Stack cible définie, dépendances Rust manquantes dans Cargo.toml actuel

### AD-11 — Auth JWT + rôles

- **Binds:** `auth`
- **Prevents:** Sessions non standard, refresh token complexe
- **Rule:** Cookie `mboa_session`. JWT 24h. Refresh silencieux si <1h restante. Argon2. 4 rôles avec permissions granulaires `Vec<Permission>`. Seed idempotent au premier démarrage (admin + 10 produits / 3 catégories).
- **Statut:** 🔧 Design validé, pas encore implémenté

### AD-12 — Config via Tauri store

- **Binds:** `all`
- **Prevents:** Fragmentation config (YAML + env + store), fichiers externes non gérés
- **Rule:** `tauri_plugin_store` dans `$APP_DATA_DIR` pour port (3000), mDNS hostname (mboacaisse), backup interval (24h), stock bas seuil (5), moyens paiement (['cash']). Pas de YAML/TOML.
- **Statut:** ✅ Plugin store installé. 🔧 Bridge Pinia + utilisation métier à implémenter.

### AD-13 — Graphe dépendances

- **Binds:** `all`
- **Prevents:** Dépendances circulaires, couplage non maîtrisé
- **Rule:** Voir diagramme ci-dessus. Wallet est une île (pas de dépendance sortante). Auth indépendant. Payment → Order+Wallet+Print. Order → Catalog+Wallet. Kitchen → Order (lecture statut). Stock → Catalog (lecture produits, conso en P2). Reports → SQL directe.
- **Statut:** 🔧 Design validé

### AD-14 — Kitchen display polling

- **Binds:** `kitchen`
- **Prevents:** Complexité WebSocket en V1
- **Rule:** Polling HTTP 5s via `useFetch()` + `setInterval`. Pas de connexion persistante.
- **Note:** `docs/architecture-mboacaisse.md` mentionne WebSocket natif Axum — **AD-14 prévaut**: polling V1, WebSocket reporté P2.7.
- **Statut:** 🔧 Design validé

### AD-15 — Migrations refinery

- **Binds:** `db/`
- **Prevents:** Schema non versionné, mise-à-jour manuelle
- **Rule:** `refinery::Runner::new().run()` au startup. SQL embarquées, table `_schema_version` auto-gérée. Échec → log + exit. Pas de démarrage serveur sans schéma validé.
- **Statut:** 🔧 Design validé, dépendances manquantes

### AD-16 — Pool r2d2-rusqlite

- **Binds:** `db/`
- **Prevents:** Panne pool non mature en alpha
- **Rule:** r2d2-rusqlite. Switch deadpool-sqlite si obsolescence.
- **Note:** `FEATURES.md §P0.2` mentionne `libsql::Database` — **AD-16 prévaut**: r2d2-rusqlite.
- **Statut:** 🔧 Design validé

### AD-17 — Déploiement alpha

- **Binds:** `build`, `deploy`
- **Prevents:** Sur-ingénierie CI/CD, staging inutile
- **Rule:** Pas de staging. Binaire unique. Dev = poste développeur, Prod = PC commerçant. Licence alpha pré-générée = flag alpha dans entitlements = logs DEBUG + diagnostics activés.
- **Statut:** 🔧 Design validé. Pas de CI/CD pipeline.

### AD-18 — Logs tracing

- **Binds:** `all`
- **Prevents:** Logs bruités en prod, silence gênant en debug
- **Rule:** `tracing` + `tracing-subscriber`. Niveau INFO par défaut, DEBUG si licence alpha. Fichier `mboacaisse.log` dans `$APP_DATA_DIR`. Pas de rolling en alpha.
- **Statut:** 🔧 Design validé, dépendances manquantes

### AD-19 — Template fork (état actuel)

- **Binds:** `current`
- **Prevents:** Confondre le template Nuxtor avec le produit MboaCaisse
- **Rule:** Le projet est actuellement un fork du template Nuxtor. Le backend Rust n'a ni Axum, ni DB, ni domain. Le frontend a des pages de démo Tauri, pas de pages métier. Toute implémentation doit partir de zéro dans les couches api/domain/db. Les pages frontend de démonstration doivent être supprimées ou cachées derrière un flag DEV quand les pages métier arrivent.
- **Statut:** ✅ Documenté

---

## Consistency Conventions

| Concern | Convention |
|---|---|
| Nommage fichiers | `snake_case.rs` — un fichier par capacité dans chaque couche |
| Identifiants | UUID v7 pour toutes les entités (ordonnés temporellement, indexables) |
| Dates | ISO 8601 en UTC. Stocké en TEXT SQLite. Jamais de timestamp UNIX |
| Erreurs API | `{"error": "...", "code": "SCREAMING_SNAKE"}` — code = nom du variant DomainError |
| Erreurs domaine | Enum `DomainError` avec cas nommés, pas de `anyhow` dans domain/ |
| Mutation data | Append-only pour données financières. UPDATE autorisé pour data non-financière |
| Wallet | `wallet_ledger` INSERT-only. `wallet_clients` UPDATE pour email/phone |
| Dépendances | Wallet ne dépend de rien. Payment → Wallet. Order → Catalog + Wallet |
| Frontend API | `useFetch('/api/...', { server: false })`. Pas de TanStack Query. Pas de WebSocket |
| Config store | `tauri_plugin_store`. Chargé au startup. Accessible Rust + frontend (via Pinia bridge) |
| Pages frontend | Pages métier dans `app/pages/`. Pages démo Tauri supprimées après implémentation métier |
| Téléphone | Clé universelle wallet (pas login, pas carte, pas app) |

---

## Stack

### Cible (architecture)

| Name | Version / Résolution |
|---|---|
| Rust edition | 2021 |
| Tokio | 1 (flottant, lockfile gèle) |
| Axum | 0.8 (flottant) |
| rusqlite | dernière stable |
| r2d2-rusqlite | dernière stable |
| refinery | dernière stable |
| tracing | dernière stable |
| argon2 | dernière stable |
| mdns-sd | dernière stable |
| tauri-plugin-store | dernière stable |
| Nuxt | 4 |
| Vue | 3 |
| Pinia | dernière stable |
| TailwindCSS | v4 |

### Actuelle (Cargo.toml)

| Name | Version | Note |
|---|---|---|
| tauri | 2.9.5 | ✅ |
| tauri-plugin-shell | 2.3.4 | ✅ |
| tauri-plugin-notification | 2.3.3 | ✅ |
| tauri-plugin-os | 2.3.2 | ✅ |
| tauri-plugin-fs | 2.4.5 | ✅ |
| tauri-plugin-store | 2.4.2 | ✅ |
| serde | 1 | ✅ |
| serde_json | 1 | ✅ |
| Tokio/Axum/rusqlite/... | — | ❌ À ajouter |

---

## Structural Seed

### Actuel (2026-07-21)

```text
src-tauri/src/
├── main.rs              # Entry Tauri
└── lib.rs               # Tauri builder + plugins (6 plugins)
                         # PAS de server.rs, mdns.rs, print.rs

app/                     # Nuxt 4 — pages template uniquement
├── app.vue
├── pages/               # Démo Tauri API, pas de métier
├── components/          # Design/Layout/Site — template
├── layouts/             # default, blank, home
├── modules/tauri.ts     # Auto-import Tauri APIs
└── composables/pages.ts # Navigation générique
```

### Cible (à construire)

```text
src-tauri/src/
├── main.rs              # Entry Tauri, on_event ExitRequested
├── lib.rs               # Builder Tauri, plugins registration, setup Axum
├── server.rs            # Axum serve + graceful shutdown handle
├── mdns.rs              # Publication mDNS (mboacaisse.local)
├── print.rs             # Service impression asynchrone (ESC/POS)
├── api/
│   ├── mod.rs           # Router Axum + middleware auth
│   ├── auth.rs          # Login, logout, refresh JWT
│   ├── products.rs      # CRUD catalogue
│   ├── orders.rs        # Cycle commande (création → validation → cuisine)
│   ├── payments.rs      # Encaissement multi-moyen
│   ├── wallet.rs        # Solde, ledger, identification client
│   ├── kitchen.rs       # Affichage cuisine (lecture commandes actives)
│   ├── reports.rs       # Rapports agrégés (SQL direct)
│   ├── health.rs        # GET /api/health diagnostic
│   └── settings.rs      # Configuration store
├── domain/
│   ├── mod.rs
│   ├── user.rs          # User, Role, Permission, trait UserRepository
│   ├── product.rs       # Product, Category, trait ProductRepository
│   ├── order.rs         # Order, OrderStatus, trait OrderRepository
│   ├── payment.rs       # Payment, PaymentMethod, trait PaymentRepository
│   ├── wallet.rs        # WalletClient, WalletLedgerEntry, trait WalletRepository
│   └── print_job.rs     # PrintJob struct (pas de repository)
├── db/
│   ├── mod.rs
│   ├── migrations.rs    # Runner refinery + SQL embarquées
│   ├── seed.rs          # Seed idempotent admin + produits démo
│   ├── users.rs         # impl UserRepository pour rusqlite
│   ├── products.rs      # impl ProductRepository
│   ├── orders.rs        # impl OrderRepository
│   ├── payments.rs      # impl PaymentRepository
│   └── wallet_ledger.rs # impl WalletRepository
└── license/
    ├── mod.rs
    ├── verify.rs        # Vérification signature Ed25519
    └── entitlements.rs  # Feature gating
```

---

## Capability → Architecture Map

| Capabilité | Vit dans | Gouverné par | Statut |
|---|---|---|---|
| **Socle Rust** | `lib.rs`, `main.rs` | AD-9 (cycle vie) | 🔧 Tray + plugins faits. Axum/DB à construire |
| **Auth** | api/auth.rs, domain/user.rs, db/users.rs | AD-11 (JWT cookie, argon2, 4 rôles) | 🔧 Design validé |
| **Catalog** | api/products.rs, domain/product.rs, db/products.rs | AD-13 (indépendant) | 🔧 Design validé |
| **Order** | api/orders.rs, domain/order.rs, db/orders.rs | AD-13 (→Catalog, →Wallet) | 🔧 Design validé |
| **Payment** | api/payments.rs, domain/payment.rs, db/payments.rs | AD-4 (→Order, →Wallet, →Print) | 🔧 Design validé |
| **Wallet** | api/wallet.rs, domain/wallet.rs, db/wallet_ledger.rs | AD-2 (append-only), AD-4 (île) | 🔧 Design validé |
| **Kitchen** | api/kitchen.rs, domain/order.rs (lecture) | AD-14 (polling 5s) | 🔧 Design validé |
| **Stock** | api/stock.rs, domain/product.rs, db/products.rs | AD-13 (→Catalog; conso en P2) | 🔧 Design validé |
| **Reports** | api/reports.rs | AD-6 (SQL directe, pas de domain) | 🔧 Design validé |
| **Print** | src/print.rs | AD-5 (service transverse, file async) | 🔧 Reporté P2.1 |
| **Config** | api/settings.rs, tauri_plugin_store | AD-12 (store Tauri, pas YAML) | ✅ Plugin installé. 🔧 Bridge à faire |
| **mDNS** | src/mdns.rs | Découverte réseau | 🔧 Design validé |
| **Licensing** | src/license/ | P4 (Ed25519 offline) | 🔧 Design validé, docs existent |
| **Frontend métier** | app/pages/ | AD-10 (useFetch, pas TanStack Query) | ❌ Pages démo seulement |
| **Backup** | api/settings.rs + tâche tokio | P1.2 (ZIP, rotation 30) | 🔧 Design validé |

---

## Deferred

| Décision | Raison | Revisit |
|---|---|---|
| WebSocket serveur (P2.7) | V1 = polling HTTP, WebSocket non nécessaire | Si besoin temps réel confirmé |
| Tauri updater (P2.4) | Trop lourd pour 3 alpha. Remplacement binaire suffit | Passage 1→10 établissements |
| Impression ESC/POS (P2.1) | V1 = ticket numérique ou impression générique | Quand client demande imprimante |
| Licence platform (P4) | Système cloud séparé, pas d'impact archi alpha | Lancement commercial |
| Bundles commerciaux | Feature gating via licence (Ed25519). Architecture = même binaire | Quand >1 bundle à vendre |
| Migration Rust 2024 | Frictions potentielles avec crates Tauri | P2+ si justifié |
| Conso stock auto | Stock → Order prévu en P2 | Après alpha |
| JavaScript frontend | Organisation par feature si >15 fichiers/dossier | Quand croissance atteint seuil |
| Log rolling | Pas nécessaire en alpha | Pré-prod |
| TanStack Query | useFetch() suffit en LAN <10ms | Si besoin cache avancé confirmé |
| Fenêtre secondaire (P2.2) | Afficheur client Tauri | P2 |
| Scan code-barres (P2.3) | Plugin barcode-scanner | P2 |
| Global shortcuts (P2.5) | Plugin global-shortcut | P2 |
| Autostart (P2.6) | Plugin autostart | P2 |
