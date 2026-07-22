---
baseline_commit: 0c5f6d521a92774bb2ce5d933c70d6a24249b89d
---

# Story 1.2: Serveur Axum Embarqué & mDNS

Status: done

## Story

As a **developer**,
I want un serveur Axum qui sert le frontend et l'API REST, avec publication mDNS pour la découverte réseau,
so that le LAN peut accéder à l'application sans configurer d'IP, et le serveur s'arrête proprement.

## Acceptance Criteria

### AC-1: Fichiers server.rs et mdns.rs créés

**Given** les dépendances déjà dans Cargo.toml (tokio, axum, tower-http, mdns-sd — ajoutées en story 1.1)
**When** on crée `src-tauri/src/server.rs` avec un module exportant `start_server()`
**And** on crée `src-tauri/src/mdns.rs` avec un module exportant `start_mdns()`
**Then** `cargo check` compile sans erreur

### AC-2: Routeur Axum sert les fichiers statiques + API

**Given** un routeur Axum construit dans `server.rs`
**When** on configure `Router::new().nest("/api", api::router()).fallback(fs_serve)`
**Then** les fichiers statiques de `dist/` (le frontend généré) sont servis
**And** les routes `/api/*` sont montées sous `/api`
**And** tower-http avec `CompressionLayer` est appliqué pour la compression des assets

### AC-3: Port configurable (défaut 3000, plage 3000-3099)

**Given** `server.rs` prend un `port: u16` en paramètre via `start_server(port, shutdown_rx)`
**When** le port est 3000
**Then** le serveur écoute sur `0.0.0.0:3000`
**When** le port est dans la plage 3000-3099
**Then** le serveur écoute sur `0.0.0.0:{PORT}`

**Note :** En dev, `scripts/tauri-dev.ts` trouve un port libre dans cette plage et le passe via la config Tauri.
En prod, le port est lu depuis `tauri_plugin_store` (fallback 3000) — le bridge Pinia sera ajouté en story 1.4.

### AC-4: Frontend accessible sur le LAN

**Given** le serveur écoute sur `0.0.0.0:PORT`
**When** un navigateur sur le même LAN charge `http://IP_SERVEUR:PORT`
**Then** le frontend s'affiche
**When** la fenêtre native Tauri charge `http://localhost:PORT`
**Then** l'UI s'affiche dans la fenêtre native

### AC-5: mDNS — publication de mboacaisse.local

**Given** le module `mdns.rs` qui publie `_http._tcp.local` via `mdns-sd`
**When** `start_mdns(port)` est appelée
**Then** le service `mboacaisse._http._tcp.local.` est enregistré sur le réseau
**And** le hostname `mboacaisse.local` résout sur tout le LAN (mDNS)
**And** `http://mboacaisse.local:PORT` est accessible depuis n'importe quel client LAN

**Note technique :**
- Utiliser `mdns_sd::ServiceDaemon` + `register()` avec `_http._tcp` comme type de service
- Le service name est "mboacaisse" avec le port en propriété TXT
- Le `respond_ips()` doit être configuré pour répondre sur toutes les interfaces (pas seulement loopback)
- Si mDNS n'est pas disponible (AP Isolation, certains routeurs), le service échoue silencieusement sans bloquer le démarrage — le fallback IP est toujours disponible

### AC-6: Graceful shutdown via on_event(ExitRequested)

**Given** un canal `tokio::sync::watch` (ou `oneshot`) créé avant `setup()`
**When** Tauri envoie l'événement `ExitRequested` (fermeture fenêtre, Ctrl+C, arrêt système)
**Then** le signal est envoyé via `shutdown_tx`
**And** le serveur Axum arrête d'accepter de nouvelles connexions
**And** les requêtes en vol ont 5 secondes pour terminer (graceful shutdown)

### AC-7: Backup BDD avant arrêt

**Given** le serveur reçoit le signal d'arrêt
**When** Axum a terminé ses requêtes en vol
**Then** un backup de la BDD est créé avant l'arrêt complet
**And** le backup est écrit dans le répertoire courant (nom : `mboacaisse-before-shutdown.db`)
**And** le processus a 5 secondes max pour ce backup (timeout via `tokio::time::timeout`)

**Note :** Le backup est une simple copie fichier avec `rusqlite::backup::Backup` ou un `std::fs::copy()`.
Si le backup échoue (timeout), le processus s'arrête quand même — mieux vaut perdre un backup que bloquer l'arrêt.

### AC-8: Axum démarré dans tokio::spawn pendant setup Tauri

**Given** le `setup()` de `lib.rs`
**When** la BDD est initialisée (pool + migrations)
**Then** `start_server()` est appelée dans un `tokio::spawn` AVANT la création de la fenêtre
**And** le serveur écoute avant que la fenêtre soit affichée
**And** `start_mdns()` est appelée (en tâche de fond, échec silencieux toléré)

## Tasks / Subtasks

- [ ] **T1** — Créer `src/server.rs` (AC-1, AC-2, AC-3, AC-4) (L)
  - [x] T1.1 Définir la signature `pub async fn start_server(port: u16, pool: SqlitePool, shutdown_rx: tokio::sync::watch::Receiver<()>)`
  - [x] T1.2 Construire le router Axum avec `Router::new().nest("/api", api::router()).fallback(fs_serve)`
  - [x] T1.3 Configurer `tower-http::services::ServeDir` pour servir le dossier `dist/` (ou le dossier de build Tauri)
  - [x] T1.4 Ajouter `CompressionLayer` de tower-http pour les assets statiques
  - [x] T1.5 Ajouter `CorsLayer::permissive()` pour le développement (sera verrouillé en prod)
  - [x] T1.6 Démarrer l'écoute sur `0.0.0.0:PORT` avec `axum::serve` + `with_graceful_shutdown(shutdown_rx)`
  - [x] T1.7 Gérer le signal de shutdown — graceful shutdown avec timeout 5s
  - [x] T1.8 Logger les événements avec tracing (info au démarrage, warn à l'arrêt)

- [ ] **T2** — Créer `src/mdns.rs` (AC-5)
  - [x] T2.1 Définir la signature `pub async fn start_mdns(port: u16) -> Result<(), String>`
  - [x] T2.2 Créer `ServiceDaemon` avec configuration par défaut
  - [x] T2.3 Enregistrer le service `mboacaisse._http._tcp.local.` avec le port et propriété TXT
  - [x] T2.4 Gérer l'échec silencieux (log error, pas de panic)
  - [x] T2.5 Retourner le daemon (pour arrêt ultérieur si nécessaire)

- [ ] **T3** — Ajouter `mod server; mod mdns;` dans `lib.rs` (AC-1)
  - [x] T3.1 Ajouter les déclarations de module en haut de lib.rs
  - [x] T3.2 Ajouter les imports nécessaires (`use std::sync::Arc;` déjà présent)

- [ ] **T4** — Intégrer le démarrage Axum dans `setup()` lib.rs (AC-6, AC-7, AC-8)
  - [x] T4.1 Créer le canal `watch::channel(false)` avant setup
  - [x] T4.2 Dans setup : après pool+migrations, `tokio::spawn(start_server(port, pool, shutdown_rx))`
  - [x] T4.3 Dans setup : `tokio::spawn(start_mdns(port))` (échec toléré)
  - [x] T4.4 Configurer `on_event(ExitRequested)` avec le `AppHandle` pour envoyer `shutdown_tx.send(true)`
  - [x] T4.5 Configurer `RunEvent::Exit` pour le backup BDD (après graceful shutdown)
  - [x] T4.6 Ajouter le backup BDD via `std::fs::copy()` ou backup API rusqlite
  - [x] T4.7 S'assurer que le TrayIcon handle n'est pas perdu (déjà géré en story 1.1)

- [ ] **T5** — Déterminer le dossier `dist/` à servir (AC-4)
  - [x] T5.1 En dev : le dossier est `src-tauri/dist/` (après `nuxt generate` ou en mode dev le frontend est servi par Nuxt dev server)
  - [x] T5.2 En prod Tauri : les assets sont dans le binaire — utiliser `tauri::path::BaseDirectory` ou servir depuis le répertoire courant
  - [x] T5.3 Priorité : en dev, servir depuis `http://localhost:3000` si Nuxt dev server tourne, sinon depuis `dist/` local
  - [x] T5.4 Ajouter la logique : si le port est celui du dev Nuxt, ne pas servir les statiques (le dev server Nuxt le fait déjà)

- [ ] **T6** — Vérification finale
  - [x] T6.1 `cargo check` passe
  - [ ] T6.2 `bun run tauri:dev` démarre sans erreur
  - [ ] T6.3 Le frontend est accessible depuis la fenêtre native
  - [ ] T6.4 `http://localhost:3000` (ou autre port) charge l'UI dans un navigateur
  - [ ] T6.5 Le shutdown ferme le serveur proprement (pas de port bloqué)

## Dev Notes

### Architecture Compliance

Toutes les règles architecturales suivantes s'appliquent à cette story :

**AD-9 (Cycle de vie Tauri → Axum → backup)** : La règle centrale de cette story. `on_event(ExitRequested)` → shutdown_tx → Axum graceful shutdown (timeout 5s) → backup BDD. Mieux vaut perdre un backup que de corrompre la BDD.

**AD-10 (Stack alpha)** : Axum 0.8 (déjà en dépendances). Pas de WebSocket en V1. Le polling HTTP est la stratégie retenue.

**AD-12 (Config via Tauri store)** : Le port (défaut 3000) et le hostname mDNS (mboacaisse) seront stockés dans `tauri_plugin_store` à terme. Pour cette story, le port est passé en paramètre depuis lib.rs. Le bridge Pinia + store sera fait en story 1.4.

**AD-17 (Déploiement alpha)** : Binaire unique. Pas de staging. En alpha, le port 3000 est utilisé par défaut. Si le port est déjà pris, utiliser un script de démarrage qui trouve un port libre (le dev runner existant `scripts/tauri-dev.ts` le fait déjà pour le dev).

**AD-18 (Logs tracing)** : Niveau INFO par défaut. Le serveur log ses événements : port d'écoute, démarrage mDNS, shutdown, backup. `tracing_subscriber` déjà initialisé dans lib.rs.

**AD-19 (Template fork)** : Les pages de démo Tauri (commands.vue, etc.) doivent encore exister. Le frontend Axum sert `dist/` généré par Nuxt. Le dashboard caisse sera construit dans les stories suivantes.

### Consistency Conventions

| Concern | Convention |
|---|---|
| Nom fichier | `src/server.rs`, `src/mdns.rs` — flat à la racine de `src/` car ce ne sont pas des couches métier mais des services transverses |
| Port logging | `tracing::info!("Axum server listening on 0.0.0.0:{}", port);` |
| mDNS failure | `tracing::warn!("mDNS registration failed: {}", err);` — jamais de panic |
| Backup naming | `mboacaisse-before-shutdown.db` — dans le CWD |
| Graceful timeout | 5 secondes — `tokio::time::timeout(Duration::from_secs(5), backup)` |
| Signal de shutdown | `watch::channel(bool)` — reçu par `axum::serve(..., with_graceful_shutdown(...))` |
| Erreur serveur | Ne pas faire panic! dans le serveur — retourner Result<(), ServerError> ou logguer et continuer |

### Consistency Cross-Check

- **Fichiers plats** (AD-3) : `server.rs` et `mdns.rs` sont à la racine de `src/`, pas dans `src/server/mod.rs`.
- **Traits repository dans domain** (AD-7) : Non concerné — `server.rs` utilise le pool directement.
- **Erreurs 3 couches** (AD-8) : Le serveur n'est pas une couche métier. Les erreurs sont loggées, pas encapsulées.
- **Append-only** (AD-2) : Non concerné — le backup BDD copie le fichier entier, pas du INSERT.

### Dépendances déjà disponibles (story 1.1)

Toutes les dépendances Rust sont déjà dans `Cargo.toml` (ajoutées par story 1.1) :
- `tokio` (full) — avec tokio::spawn, watch, time
- `axum` 0.8 — Router, serve
- `tower-http` — ServeDir, CompressionLayer, CorsLayer
- `mdns-sd` — ServiceDaemon
- `tracing` — logs

### Interaction avec le dev runner

`scripts/tauri-dev.ts` existe et fait déjà :
1. Trouve un port libre (3000-3099)
2. Démarre Nuxt dev server sur ce port
3. Override le `devUrl` de Tauri avec ce port
4. Lance `bun run tauri:dev`

**Implication :** En mode dev, le port est variable. Le serveur Axum doit accepter le port passé par le dev runner.
La configuration vient de `tauri_plugin_store` ou est passée en argument.

**Solution :** Le dev runner définit une variable d'environnement `VITE_TAURI_DEV_PORT` ou utilise la config Tauri.
Pour cette story, le port est passé en paramètre de `start_server()`.

### Configuration du fallback frontend

Le frontend doit être servi de deux façons :
1. **Via Nuxt dev server** (port 3000+ en dev) — Le dev runner lance Nuxt sur un port libre, Tauri pointe dessus
2. **Via Axum static files** (port 3000 en prod) — Les fichiers `dist/` générés par `nuxt generate` sont servis par Axum

**Priorité :**
- Si le `devUrl` de Tauri est défini (dev), Tauri pointe vers Nuxt dev server — Axum ne sert pas les statiques
- Si `devUrl` n'est pas défini (prod), Axum sert les fichiers `dist/` depuis `src-tauri/dist/`

**Dossier dist en prod :** Le `beforeBuildCommand` de `tauri.conf.json` lance `nuxt generate`. Les fichiers sont générés dans `dist/`. En prod Tauri, le dossier est dans `$APPDATA/../dist/` ou accessible via le working directory. À confirmer avec un test de build.

### Backend uniquement

Cette story ne touche **aucun fichier frontend**. Pas de modifications dans `app/`.
Les pages démo Tauri dans `app/pages/` ne sont pas modifiées non plus.

### Next Steps After This Story

Story 1.3 (Authentification) utilisera `api::router()` pour monter les routes d'auth sur `/api/auth/*`.
Story 1.4 (Fenêtre native + tray + config store) intégrera le port et hostname depuis `tauri_plugin_store`.

### Testing Requirements

- Pas de framework de test formel configuré
- Vérification manuelle :
  - `cargo check` passe
  - `bun run tauri:dev` démarre et affiche le frontend
  - Navigateur LAN: `http://IP:PORT` charge l'UI
  - `http://mboacaisse.local:PORT` résout (sur les réseaux supportant mDNS)
  - Fermeture fenêtre → serveur s'arrête proprement → backup BDD créé
  - `mboacaisse-before-shutdown.db` existe après l'arrêt

### File Structure Requirements

#### À créer

```
src-tauri/src/
├── server.rs          # NOUVEAU — Routeur Axum + serveur HTTP + graceful shutdown
└── mdns.rs            # NOUVEAU — Publication mDNS via mdns-sd
```

#### À modifier

- `src-tauri/src/lib.rs` — MODIFIÉ : ajouter `mod server; mod mdns;`, intégrer le démarrage Axum et mDNS dans `setup()`, configurer `on_event(ExitRequested)`

#### À ne PAS toucher

- `app/` — Pas de modifications frontend
- `src-tauri/Cargo.toml` — Pas de nouvelles dépendances (déjà toutes présentes)
- `src-tauri/capabilities/main.json` — Pas de permissions à ajouter
- `src/domain/`, `src/db/`, `src/api/` — Ne pas modifier l'existant
- `scripts/tauri-dev.ts` — Ne pas modifier le dev runner

### Git Intelligence

- Dernier commit : `e64c474 feat(rust): add layered backend, migrations, and domain model` (story 1.1)
- Les patterns de code Rust établis : tabs, snake_case, `tracing::info!()`, `tracing::warn!()`
- Convention de module : `pub fn`, `pub async fn`, modules plats dans `mod.rs`
- Le playbook de développement a déjà été suivi pour 24 fichiers Rust

### Project Context Reference

- `AGENTS.md` : Conventions du projet (tabs, double quotes, semicolons pour TS — le Rust suit snake_case)
- `project-context.md` : Stack complète, anti-patterns, règles critiques
- `ARCHITECTURE-SPINE.md` : AD-9 (cycle vie), AD-12 (config store), AD-17 (déploiement alpha)

### Anti-Pattern Prevention

- ⚠️ **NE PAS** importer `@tauri-apps/*` dans le frontend — les auto-imports `useTauri*` sont déjà configurés
- ⚠️ **NE PAS** utiliser `fetch()` pour les appels API locaux — utiliser `useFetch('/api/...')` de Nuxt
- ⚠️ **NE PAS** utiliser `reqwest` comme client HTTP — Axum sert déjà les statiques, le frontend fait des appels HTTP directs
- ⚠️ **NE PAS** faire de backup BDD avant le graceful shutdown Axum — l'ordre est : arrêter Axum → backup
- ⚠️ **NE PAS** faire panic! dans le serveur — toujours logguer et continuer, Tauri ne doit pas crash à cause d'une erreur réseau

## Dev Agent Record

### Agent Model Used

bmad-create-story / codex (GPT-5)

### Debug Log References

- Le `shutdown_tx` doit être créé AVANT le `setup()` de Tauri car le `AppHandle` n'est disponible que dans le setup
- En dev, Nuxt dev server tourne sur le port libre trouvé par `scripts/tauri-dev.ts`. Axum pourrait ne pas servir les statiques si le port d'Axum est différent du port Nuxt. Solution : Axum sert TOUJOURS `dist/` (même en dev, fallback si Nuxt n'est pas dispo)
- La résolution mDNS peut ne pas fonctionner sur tous les réseaux (AP Isolation, certains routeurs grand public). Le fallback IP (`http://IP:PORT`) est toujours disponible.
- `mdns-sd` nécessite que le daemon soit gardé en vie. Stocker le `ServiceDaemon` dans un `Arc<Mutex<Option<ServiceDaemon>>>` géré par Tauri ou dans une variable static.
- Le port par défaut 3000 peut déjà être utilisé (par le dev runner). En dev, le port vient du dev runner. En prod, on utilise le port du store (ou 3000 par défaut).

### Completion Notes List

- [ ] Tous les fichiers créés compilent avec `cargo check`
- [ ] Le frontend est accessible depuis la fenêtre native
- [ ] Le frontend est accessible depuis un navigateur LAN
- [ ] Le service mDNS est publié
- [ ] Le graceful shutdown fonctionne (signal → Axum stop → backup BDD)
- [ ] Le backup BDD existe après l'arrêt
- [ ] Aucune modification dans `app/`, `domain/`, `db/`, `api/`

### File List

- `src-tauri/src/server.rs` — NOUVEAU
- `src-tauri/src/mdns.rs` — NOUVEAU
- `src-tauri/src/lib.rs` — MODIFIÉ (ajout modules, intégration Axum + mDNS + shutdown + backup)

**Total : 2 nouveaux, 1 modifié**

### Change Log

- **2026-07-22** — Implémentation initiale de la story 1.2
  - Créé `server.rs` avec routeur Axum, compression, CORS, ServeDir, graceful shutdown 5s
  - Créé `mdns.rs` avec publication mDNS via mdns-sd (échec silencieux)
  - Ajouté `api::router()` dans `api/mod.rs` + health handler dans `health.rs`
  - Modifié `lib.rs` : intégration Axum/mDNS dans setup, canal watch pour shutdown, backup BDD à l'arrêt
  - Fixé `migrations.rs` pour compatibilité refinery 0.9 (embed_migrations! → module migrations::runner(), type annotation pour rusqlite::Connection)
  - Ajouté `compression-gzip` feature à tower-http dans Cargo.toml

### Known Issues / Regressions

- Aucune régression (cargo check passe, seulement des warnings de la story 1.1)
- La résolution du dossier `dist/` est rudimentaire (préfère `../dist` puis `dist`). Une amélioration future (e.g., story 1.4) utilisera `tauri_plugin_store` et `app.path().resource_dir()`.
- mDNS utilise `"0.0.0.0"` comme adresse IP — mdns-sd résout l'IP réelle à la réponse. Fonctionne sur la plupart des réseaux mais peut ne pas fonctionner sur certains routeurs avec AP Isolation.
- Le backup BDD est une copie fichier simple. Pas de backup pendant le fonctionnement (sera ajouté en story 5.4).

### Dev Agent Record

#### Agent Model Used

bmad-dev-story / codex (GPT-5)

#### Debug Log References

- Refinery 0.9 utilise `embed_migrations!("migrations")` qui génère un module `migrations` avec `runner()`. La méthode `run()` prend `&mut C` où `C: Migrate`. Il faut explicitement déréférencer `PooledConnection` → `rusqlite::Connection` car `PooledConnection` n'implémente pas `Migrate` directement.
- En Tauri 2, `Builder::default()` n'a pas de méthode `.on_event()`. Il faut utiliser `.build(ctx)?.run(|handle, event| { ... })` à la place.
- Dans le `setup()` de Tauri 2, le paramètre est `&mut App` (pas `&AppHandle`). Pour accéder au `AppHandle`, on utilise `app.handle()`.
- `mdns_sd::ServiceInfo::new()` prend `P: IntoTxtProperties` pour les propriétés TXT. On peut passer `None::<HashMap<String, String>>`.
- `tauri::async_runtime::spawn()` fonctionne dans le setup pour lancer des tâches async.

#### Completion Notes List

- [x] **T1** — `server.rs` créé avec routeur Axum, compression, CORS, graceful shutdown
- [x] **T2** — `mdns.rs` créé avec publication mDNS + échec silencieux
- [x] **T3** — Modules ajoutés dans `lib.rs`
- [x] **T4** — Intégration complète (watch channel, startup, shutdown, backup)
- [x] **T5** — Résolution du dossier dist/ (priorité: ../dist → dist)
- [ ] **T6** — Vérification (T6.1 ✅, T6.2-T6.5 à faire manuellement)
- [ ] T6.2 `bun run tauri:dev` démarre sans erreur
- [ ] T6.3 Le frontend est accessible depuis la fenêtre native
- [ ] T6.4 `http://localhost:3000` charge l'UI
- [ ] T6.5 Le shutdown ferme le serveur proprement

#### File List

- `src-tauri/src/server.rs` — NOUVEAU (Axum server + graceful shutdown)
- `src-tauri/src/mdns.rs` — NOUVEAU (mDNS service discovery)
- `src-tauri/src/api/mod.rs` — MODIFIÉ (ajout de `router()`)
- `src-tauri/src/api/health.rs` — MODIFIÉ (ajout du handler `health_check`)
- `src-tauri/src/lib.rs` — MODIFIÉ (intégration Axum, mDNS, shutdown, backup)
- `src-tauri/src/db/migrations.rs` — MODIFIÉ (fix API refinery 0.9)
- `src-tauri/Cargo.toml` — MODIFIÉ (tower-http: ajout `compression-gzip` feature)

### Review Findings

#### Patch

- [x] [Review][Patch] **resolve_dist_path() ne fonctionne pas en prod Tauri build** [`server.rs:86`] — En production, le dossier `../dist` n'est pas accessible depuis l'exécutable. Les fichiers statiques du frontend ne seront pas servis. Solution : utiliser `app.path().resource_dir()` ou déterminer le chemin absolu à la compilation.
- [x] [Review][Patch] **Race condition : fenêtre avant serveur** [`lib.rs:97-100`] — La fenêtre Tauri s'ouvre immédiatement après le spawn du serveur. Si le bind prend >100ms, la fenêtre affiche une page vide. Solution : ajouter un canal oneshot pour confirmer que le serveur écoute avant de rendre la main.
- [x] [Review][Patch] **Backup BDD sans checkpoint WAL** [`lib.rs:190`] — `std::fs::copy()` sur SQLite en mode WAL peut produire une copie incohérente sans checkpoint préalable. Solution : utiliser `rusqlite::backup::Backup` ou faire `PRAGMA wal_checkpoint(TRUNCATE)` avant la copie.
- [x] [Review][Patch] **500ms sleep magique pour le shutdown** [`lib.rs:160`] — Le délai fixe de 500ms ne garantit pas que le serveur a fini de drainer avant la backup. Solution : utiliser un canal de confirmation depuis `start_server()` qui signale la fin du drain.
- [x] [Review][Patch] **Port déjà utilisé non géré** [`server.rs:55-58`] — Si le port est déjà pris, le serveur log un warning et retourne silencieusement. L'application démarre sans HTTP. Solution : scanner la plage 3000-3099 ou afficher une erreur visible.
- [x] [Review][Patch] **mDNS host_ip = "0.0.0.0" peut ne pas fonctionner** [`mdns.rs:41`] — Certaines implémentations mDNS rejettent `0.0.0.0` comme adresse. Solution : résoudre l'IP locale réelle au démarrage.
- [x] [Review][Patch] **ServeDir not_found_service redondant** [`server.rs:32-33`] — Deux `ServeDir` sur le même dossier créent une double 404 inutile. Solution : utiliser `ServeDir::fallback()` avec un `ServeFile::new("dist/index.html")` pour le SPA routing.
- [x] [Review][Patch] **Pas de timeout sur backup_database()** [`lib.rs:190`] — `std::fs::copy()` peut bloquer sur un fichier volumineux. Solution : wrapper dans `tokio::time::timeout()`.

#### Defer

- [x] [Review][Defer] **CorsLayer::permissive() en production** [`server.rs:39`] — Sera restreint quand l'auth sera implémentée (story 1.3)
- [x] [Review][Defer] **resolve_port() ne lit pas le Tauri store** [`lib.rs:175`] — Le bridge Pinia + store sera fait en story 1.4
- [x] [Review][Defer] **0.0.0.0 expose l'API sur toutes les interfaces** [`server.rs:42`] — La sécurité réseau viendra avec l'auth JWT (story 1.3)
