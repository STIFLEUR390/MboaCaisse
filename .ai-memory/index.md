# MboaCaisse — AI Memory

## 2026-07-21 Session: Creative Partner — Expansion, Fidélisation, Monétisation

### Concepts clés générés

**Wallet = noyau dur de tout le produit**
- Wallet multi-source (MoMo, cash, gift, cashback, transfer)
- Solde toujours calculé (`SUM(amount)`), jamais stocké
- Clé = numéro téléphone (identifiant fidélité passif)
- Client sans téléphone = ID interne `CLI-XXXX`
- Paiement **avant** validation commande (wallet check)
- Dépôt client optionnel, désactivé par défaut (zone grise régulation)

**Menu public 5 écrans (P0.5)**
1. QR → landing (table ID dans URL)
2. Menu catégories/produits (prix FCFA, pas d'images)
3. Panier (qté, total, bouton commander)
4. Identification téléphone (wallet existant ou création)
5. Confirmation + statut temps réel

**Fidélité sans friction**
- Cashback auto 5% — pas de carte, pas d'app, juste le numéro
- Seuil progressif : 3% → 5% → 8% (game design passif)
- Parrainage : 1000 FCFA sur les deux wallets via "recommandé par"
- QR par table (pas de sélection), pas de step supplémentaire

**6 bundles = feature flags, pas de forks**
- Mboa Cash (encaissement + wallet + fidélité basique)
- Mboa Resto (Cash + MoMo + pré-commande + kitchen + tables)
- Mboa Stock (Resto + inventaire + fournisseurs + multi-dépôt)
- Mboa Traiteur (Resto + planning + acomptes + fiches techniques)
- Mboa Hôtel (Resto + chambres + minibar + facture séjour)
- Mboa Market (Stock + code-barres + inventaire tournant + marge auto)
- Licensing Ed25519 existant (P4) = vérification offline des flags

**Bug du Succès (3 ans, 50 établissements)**
- P0 **wallet_ledger** — table append-only, INSERT-only, backup toutes les 5 min
- P0 **impression queue async** — file d'attente + retry + fallback ticket numérique
- P1 mDNS personnalisable (chezbob.local)
- P2 sync groupe (repoussée — wallet par instance acceptable)

### Décisions d'architecture
- Wallet = un seul par téléphone, multi-sources (pas wallets séparés fusionnés)
- Paiement wallet avant que la commande parte en cuisine (pas de crédit par défaut)
- MoMo = source d'approvisionnement, pas mode de paiement direct
- Impression asynchrone = ne bloque jamais la commande
- wallet_ledger append-only créé avec rétro-compatibilité (rejoue historique)

### Liens
- [FEATURES.md](../../FEATURES.md) — backlog fonctionnel
- [Architecture](../../docs/architecture-mboacaisse.md)
- [Licensing](../../docs/systeme-de-licences.md)

### Règles importantes
- Le téléphone est la clé universelle (pas login, pas carte, pas app)
- Wallet + impression + ledger = triangle de résilience
- Les 6 bundles sont du feature gating, pas du code séparé
- Offline-first : wallet ledger en append-only, backup fréquent

---

## 2026-07-22 Session: Story 1.1 (Structure) + Story 1.2 (Axum Server)

### Stories implémentées

**Story 1.1** — Structure Rust Layered & Migrations Initiales (statut: in-progress)
- Structure api/domain/db créée avec 24 fichiers Rust
- Toutes les dépendances ajoutées à Cargo.toml
- Role/Permission enum, DbError/DomainError 3 couches
- Pool r2d2 initialisé avec AppState
- Cf. `_bmad-output/implementation-artifacts/1-1-structure-rust-layered-migrations-initiales.md`

**Story 1.2** — Serveur Axum Embarqué & mDNS (statut: done)
- `server.rs` : routeur Axum, compression gzip, CORS, graceful shutdown 5s, fallback port
- `mdns.rs` : publication mDNS via mdns-sd, résolution IP locale, échec silencieux
- `lib.rs` : intégration Axum/mDNS dans setup, canal watch pour shutdown, backup BDD
- 8 findings de code review patchés (dont WAL checkpoint, résolution IP, fallback port)

### Commandes utiles découvertes

```sh
cargo check                          # Vérification compilation Rust
cargo doc --no-deps -p refinery      # Docs d'une crate spécifique
```

### Gotchas et erreurs fréquentes

#### Refinery 0.9 vs 0.8
- `embed_migrations!("migrations")` génère un MODULE `migrations`, pas une fonction `migrations_runner()`
- Utiliser `migrations::runner().run(&mut conn)?` où conn est `&mut rusqlite::Connection`
- `PooledConnection` n'implémente PAS `Migrate` (trait refinery) — il faut explicitement déréférencer à `&mut rusqlite::Connection`
- Solution : `let raw_conn: &mut rusqlite::Connection = &mut *conn;`

#### Tauri 2 API
- `Builder::default()` n'a PAS de méthode `.on_event()`. Utiliser `.build(ctx)?.run(|h, e| {})`
- Le paramètre de `setup()` est `&mut tauri::App`, pas `&AppHandle`. Utiliser `app.handle()` pour obtenir le handle.
- `tauri::async_runtime::spawn()` pour lancer des tâches async dans setup
- `std::thread::spawn()` pour les tâches bloquantes (mDNS, backup)
- `RunEvent::ExitRequested` a un champ `api.prevent_exit()` pour reporter la fermeture
- Le setup s'exécute AVANT la création des fenêtres (doc Tauri)
- Les fenêtres et plugins manager sont accessibles via `app.state::<T>()`

#### mdns-sd
- `ServiceInfo::new()` prend `P: IntoTxtProperties` — passer `None::<HashMap<String, String>>`
- L'adresse IP doit être réelle (pas "0.0.0.0") pour certains routeurs
- Résolution IP locale : `UdpSocket::bind("0.0.0.0:0")` + `connect("8.8.8.8:80")` + `local_addr()`
- Le `ServiceDaemon` doit être gardé en vie (stocké dans un Arc<Mutex> géré par Tauri)

#### tower-http
- `CompressionLayer` nécessite la feature `compression-gzip` (pas seulement `cors` + `fs`)
- `ServeDir::fallback(ServeFile::new(...))` pour le SPA routing (pas de double ServeDir)
- `CorsLayer::permissive()` pratique en dev, à restreindre en prod (story 1.3)

#### Backup BDD
- Toujours faire `PRAGMA wal_checkpoint(TRUNCATE)` avant `std::fs::copy()` sur SQLite
- Utiliser `std::sync::mpsc::channel` + `recv_timeout(5s)` pour éviter de bloquer l'arrêt

#### Gestion de port
- Le serveur doit scanner un range (3000-3005) si le port par défaut est occupé
- `bind_with_fallback()` : essayer base_port, puis base_port+1..base_port+5

### Structure de code approuvée

```text
src-tauri/src/
├── main.rs           # Entry Tauri (inchangé)
├── lib.rs            # Builder Tauri + intégration Axum/mDNS/tray
├── server.rs         # Routeur Axum + graceful shutdown + port fallback
├── mdns.rs           # Publication mDNS avec résolution IP
├── api/              # Peau fine HTTP
│   ├── mod.rs        # Module déclarations + router()
│   ├── health.rs     # GET /api/health (seul handler actif)
│   └── *.rs          # Placeholders (stories futures)
├── domain/           # Comportement métier + traits repository + enums
├── db/               # Implémentations repositories + pool r2d2 + migrations
└── migrations/       # SQL migrations refinery
```

### Conventions à ne pas casser
- Toujours passer `ready_tx` à `server::start_server()` pour synchroniser setup
- Appeler `resolve_port()` + `tauri::async_runtime::spawn(server)` avant de créer la fenêtre
- Dans l'event handler : `ExitRequested` → signaler serveur, `Exit` → backup avec WAL checkpoint
- Stocker les handles longue-vie (tray, mdns) dans une struct `AppHandles` gérée par Tauri

### Liens utiles
- Tauri 2 docs : `npx ctx7@latest docs /websites/tauri_app "query"`
- Refinery docs : `npx ctx7@latest docs /rust-lang/reference "refinery embed_migrations"`
- [Story 1.2](../../_bmad-output/implementation-artifacts/1-2-serveur-axum-embarque-mdns.md)
- [Sprint status](../../_bmad-output/implementation-artifacts/sprint-status.yaml)

---

## 2026-07-22 Session: Story 1.3 (Authentification JWT) — Créée

**Story 1.3** — Authentification — Register, Login & JWT (statut: ready-for-dev)
- Fichier story: `_bmad-output/implementation-artifacts/1-3-authentification-register-login-jwt.md`
- 9 acceptance criteria couvrant register, login, middleware JWT, refresh silencieux, logout, bootstrap admin
- Backend : `jsonwebtoken` + `rand` à ajouter, `argon2` déjà présent
- Frontend : pages login/register, composable `useAuth()`, middleware `auth.ts`
- Pages démo Tauri à supprimer (commands, file, notifications, os, store, webview)
- Seed admin avec mot de passe console à implémenter (remplace le placeholder de 1.1)

---

## 2026-07-22 Session: Story 1.3 (Authentification JWT) — Implémentée

**Story 1.3** — Authentification — Register, Login & JWT (statut: review)
- Backend Rust : JWT (jsonwebtoken HS256, 24h, silent refresh), argon2, middleware Axum, handlers register/login/logout/me
- Frontend Nuxt : pages login/register (Nuxt UI v4), composable useAuth, middleware auth
- Seed admin : admin@mboacaisse.local avec mot de passe console (premier démarrage)
- Pages démo Tauri déplacées dans app/pages/demo/
- Nouvelle dépendance : jsonwebtoken, rand (0.10), rand_core 0.6 (getrandom pour argon2)
- `api::build_app()` remplace `api::router()` pour gérer l'état du router Axum

### Gotchas
- rand 0.10 a renommé `OsRng` en `SysRng`. Mais argon2 a besoin de `rand_core::OsRng` v0.6 → ajouter `rand_core = { version = "0.6", features = ["getrandom"] }`
- `Router<S>` avec state ne peut pas être nesté sous un `Router<()>` → build_app() unifie tout
- Dans Axum 0.8, `from_fn_with_state` + `route_layer` applique le middleware à toutes les routes d'un Router
- `axum::serve(listener, app.into_make_service())` convertit le Router stateful en MakeService
