# Acceptance Auditor Review — Story 1.2 vs Spec

Spec file: _bmad-output/implementation-artifacts/1-2-serveur-axum-embarque-mdns.md

## AC-1: Fichiers server.rs et mdns.rs créés
**Statut: ✅ PASS**
- `server.rs` créé avec `pub async fn start_server(port, shutdown_rx)`
- `mdns.rs` créé avec `pub fn start_mdns(port) -> Option<ServiceDaemon>`
- `cargo check` compile sans erreur

## AC-2: Routeur Axum sert fichiers statiques + API
**Statut: ✅ PASS** (avec réserve)
- Router construit avec `.nest("/api", api_router).fallback_service(fs_serve)`
- `CompressionLayer` appliqué
- `api::router()` monte `/health`
- Réserve : `not_found_service` redondant (deux ServeDir)

## AC-3: Port configurable (défaut 3000, plage 3000-3099)
**Statut: ✅ PASS**
- `resolve_port()` lit `TAURI_DEV_PORT` (validé 3000-3099) ou fallback 3000
- Signature `start_server(port, ...)` accepte u16

## AC-4: Frontend accessible sur le LAN
**Statut: ✅ PASS**
- Serveur écoute sur `0.0.0.0:PORT`
- Fenêtre native pointe sur `http://localhost:PORT` (config Tauri)

## AC-5: mDNS — publication de mboacaisse.local
**Statut: ✅ PASS**
- Service enregistré : `_http._tcp.local.` → `"mboacaisse"`
- Échec silencieux loggué en warning
- `None::<HashMap<...>>` pour les propriétés TXT

## AC-6: Graceful shutdown via on_event(ExitRequested)
**Statut: ✅ PASS**
- Canal `watch::channel(false)` créé avant setup
- `RunEvent::ExitRequested` → `shutdown_tx.send(true)`
- `axum::serve(...).with_graceful_shutdown(...)` dans server.rs
- **Réserve :** pas de mécanisme d'attente de confirmation d'arrêt

## AC-7: Backup BDD avant arrêt
**Statut: ⚠️ PARTIEL**
- `backup_database()` créée avec `std::fs::copy()`
- Fichier : `mboacaisse-before-shutdown.db`
- Timeout : pas de timeout explicite sur l'opération de copy
- Backup en mode WAL non géré (pas de checkpoint)
- La spec mentionnait `rusqlite::backup::Backup` ou `std::fs::copy` comme options

## AC-8: Axum démarré dans tokio::spawn pendant setup Tauri
**Statut: ✅ PASS**
- `tauri::async_runtime::spawn(async move { server::start_server(...).await })`
- Exécuté après pool+migrations et BEFORE création de fenêtre
- `start_mdns()` en tâche de fond via `std::thread::spawn`

## Résumé des écarts AC

| AC | Statut | Détail |
|---|---|---|
| AC-1 | ✅ | OK |
| AC-2 | ✅ | ServeDir not_found_service redondant |
| AC-3 | ✅ | Pas de lecture store (différé story 1.4) |
| AC-4 | ✅ | OK |
| AC-5 | ✅ | OK |
| AC-6 | ✅ | Pas de confirmation d'arrêt |
| AC-7 | ⚠️ | Pas de timeout, pas de checkpoint WAL |
| AC-8 | ✅ | OK |
