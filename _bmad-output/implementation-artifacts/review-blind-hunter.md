# Blind Hunter Review — Story 1.2 (Serveur Axum Embarqué & mDNS)

## Diff Scope
- New: `src-tauri/src/server.rs`, `src-tauri/src/mdns.rs`
- Modified: `src-tauri/src/lib.rs`, `src-tauri/src/api/mod.rs`, `src-tauri/src/api/health.rs`, `src-tauri/src/db/migrations.rs`, `src-tauri/Cargo.toml`

## Findings

### HIGH: TrayIcon handle perdu dans le nouveau AppHandles
**Fichier:** `lib.rs`
**Problème:** L'ancien code stockait `tray_handle` via `app.manage(tray_handle.clone())` dans le setup, puis y accédait via `app.state::<Arc<...>>()`. Maintenant, `tray_handle` est stocké dans `AppHandles` et le setup fait `app.manage(AppHandles { tray_handle: tray_handle.clone() })`. Mais dans la closure du menu tray event, on n'a plus accès direct à `tray_handle` pour le modifier — ce qui n'est pas un problème car le TrayIcon est déjà construit et stocké.

**Verdict:** Pas de bug. Le TrayIcon est stocké via le lock dans `AppHandles.tray_handle`, et l'icon reste vivant tant que `AppHandles` est managé.

### HIGH: AppState géré après le move de shutdown_rx
**Fichier:** `lib.rs:86`
**Problème:** `app_state` est créée avant le shutdown channel et move dans la closure `move |app|`. C'est correct.

### MEDIUM: resolve_port() ne lit pas le store Tauri
**Fichier:** `lib.rs:165-176`
**Problème:** La story spécifie que le port doit être lu depuis `tauri_plugin_store` (fallback 3000), mais `resolve_port()` ne lit que la variable d'env `TAURI_DEV_PORT`. Le store n'est pas accessible sans `AppHandle`, donc on ne peut pas y accéder depuis `resolve_port()` qui est appelée dans le setup.

**Solution documentée:** Le bridge Pinia + store sera fait en story 1.4.

### MEDIUM: backup_database() ignore le pool et fait un filesystem copy
**Fichier:** `lib.rs:179-199`
**Problème:** `backup_database()` prend un `SqlitePool` mais ne l'utilise pas. Il fait `std::fs::copy()` sur le fichier. Ça fonctionne mais si SQLite est en mode WAL, le fichier principal peut être incohérent sans checkpoint préalable.

**Solution:** Utiliser `rusqlite::backup::Backup` avec un checkpoint WAL, ou au minimum faire un checkpoint SQL (`PRAGMA wal_checkpoint(TRUNCATE)`).

### LOW: Le signal shutdown est envoyé mais la backup File peut être corrompue
**Fichier:** `lib.rs:151-160`
**Problème:** Après `ExitRequested`, le signal est envoyé au serveur avec un sleep de 500ms. Puis dans `Exit`, `backup_database()` est appelée. Si le serveur n'a pas fini de drainer dans les 500ms, la backup est prise pendant que le serveur répond encore à des requêtes. L'ordre devrait être : signaler → attendre que le serveur soit arrêté → backup.

### LOW: Les handles `mdns_daemon` et `tray_handle` sont stockés dans AppHandles mais jamais lus après setup
**Fichier:** `lib.rs`
**Problème:** `mdns_daemon` est stocké dans `AppHandles` et n'est jamais utilisé ailleurs (pas de déréférencement). Ça sert uniquement à éviter que le Drop du ServiceDaemon ne désenregistre le service. C'est correct — le fait qu'il soit dans un `Arc<Mutex<Option<...>>>` géré par Tauri maintient la référence.

### LOW: CompressionLayer et CorsLayer::permissive() en production
**Fichier:** `server.rs`
**Problème:** `CorsLayer::permissive()` autorise toutes les origines. C'est documenté comme "pour le développement". En alpha sur LAN, ce n'est pas un problème immédiat, mais il faudrait le restreindre en production. À tracker pour la phase de build.

### MEDIUM: ServeDir utilise `not_found_service` avec un second ServeDir créant une récursion potentielle
**Fichier:** `server.rs:33-34`
**Problème:** 
```rust
let fs_serve = ServeDir::new(&dist_path)
    .append_index_html_on_directories(true)
    .not_found_service(ServeDir::new(&dist_path).append_index_html_on_directories(true));
```
Le `not_found_service` crée un second ServeDir sur le même dossier. Si un fichier n'existe pas, le second ServeDir va aussi retourner 404. C'est inoffensif (double 404) mais redondant. Le pattern correct pour le SPA routing est d'utiliser `ServeDir::new(&dist_path).fallback(tower_http::services::fs::ServeFile::new(format!("{}/index.html", dist_path)))` ou équivalent.

### LOW: mDNS échoue silencieusement sur AP Isolation
**Fichier:** `mdns.rs`
**Problème:** C'est intentionnel — le fallback IP est documenté. Mais le message de warning pourrait inclure l'URL de fallback à utiliser.

### LOW: Constant 0.0.0.0 écoute sur toutes les interfaces
**Fichier:** `server.rs:53`
**Problème:** `0.0.0.0` expose le serveur sur toutes les interfaces réseau. C'est le comportement désiré (LAN access), mais en headless sur un réseau partagé, n'importe qui sur le LAN peut accéder à l'API. La sécurité viendra avec l'auth (story 1.3).

### INFO: `apt install libgtk-3-dev` requis pour compiler
- Note pré-existante de la story 1.1, pas un bug de cette story.
