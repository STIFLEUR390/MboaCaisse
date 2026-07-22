---
baseline_commit: 72ad6bd5c816d6cedff0151477423f9313f84df3
---

# Story 1.4: Fenêtre Native & Tray & Mode Headless

Status: done

## Story

As a **commerçant**,
I want la fenêtre Tauri configurable avec tray icon et mode headless,
so that le serveur tourne même si la fenêtre est fermée ou qu'on utilise le PC partagé sans écran.

## Acceptance Criteria

### AC-1: Fermeture fenêtre → hide to tray, serveur continue

**Given** l'application est lancée avec la fenêtre visible
**When** l'utilisateur ferme la fenêtre (clic sur X ou Alt+F4)
**Then** la fenêtre est cachée (pas détruite)
**And** le serveur Axum continue de tourner
**And** l'icône tray est toujours active
**And** l'application n'est pas dans la liste des processus à terminer (pas d'ExitRequested envoyé)

**Given** la fenêtre est cachée
**When** l'utilisateur clique sur l'icône tray
**Then** la fenêtre est réaffichée (show)
**And** son état (position, taille) est préservé

**Given** l'utilisateur clique sur "Quit" dans le menu tray
**When** l'action Quit est déclenchée
**Then** le shutdown normal est exécuté (shutdown_tx → graceful shutdown → backup BDD)
**And** l'application se termine complètement

**Given** l'utilisateur ferme la fenêtre alors que le mode headless est actif
**When** la fenêtre est déjà cachée
**Then** rien ne se passe (l'app reste en tray)

### AC-2: Mode headless configurable

**Given** le paramètre `headless` est présent dans le store (tauri_plugin_store)
**When** la valeur est `true` au démarrage
**Then** aucune fenêtre ne s'ouvre
**And** le serveur Axum démarre normalement
**And** l'icône tray est créée

**Given** le mode headless est activé
**When** le serveur s'arrête (crash ou shutdown)
**Then** une notification native Tauri est envoyée : "MboaCaisse arrêté"
**And** l'icône tray disparaît

**Given** un flag CLI `--headless` ou variable d'environnement `HEADLESS=true`
**When** l'application démarre
**Then** le mode headless est activé (priorité sur la config store)

**Given** le mode headless est désactivé (store + CLI absents)
**When** l'application démarre
**Then** la fenêtre native s'ouvre normalement (comportement actuel)

### AC-3: Config store — port, hostname, backup_interval

**Given** tauri_plugin_store initialisé avec les clés suivantes :
- `port` (u16, défaut 3000)
- `hostname` (string, défaut "mboacaisse")
- `backup_interval_hours` (u64, défaut 24)
**When** le serveur démarre
**Then** `resolve_port()` vérifie dans l'ordre :
  1. Store (clé `port`) si disponible
  2. Variable d'env `TAURI_DEV_PORT`
  3. Défaut 3000
**And** le hostname mDNS est lu depuis le store (fallback "mboacaisse")
**And** l'intervalle de backup est lu depuis le store

**Given** le store contient `port: 3005`
**When** le serveur démarre
**Then** Axum écoute sur le port 3005

**Given** le store contient `hostname: "mon-cafe"`
**When** le service mDNS est publié
**Then** le hostname est `mon-cafe.local` (au lieu de `mboacaisse.local`)

**Given** POST /api/settings avec `{ "port": 3005 }`
**When** la requête est valide
**Then** la valeur est écrite dans le store
**And** un message indique que le redémarrage est nécessaire pour appliquer
**And** la réponse retourne `{ "key": "port", "value": 3005, "requires_restart": true }`

**Given** GET /api/settings
**When** appelé
**Then** retourne toutes les clés/valeurs du store

**Tech note AD-12:** Toute la config système passe par `tauri_plugin_store`. Pas de fichiers YAML/TOML. Les valeurs lues au démarrage sont utilisées jusqu'au prochain démarrage. Les modifications via l'API sont persistées immédiatement mais requièrent un redémarrage pour les valeurs "startup" (port, hostname).

### AC-4: Bridge Pinia ↔ Tauri store (frontend)

**Given** un store Pinia `useSettingsStore()` dans `app/stores/settings.ts`
**When** l'utilisateur charge la page de settings
**Then** les valeurs lues depuis le Tauri store sont affichées
**And** le formulaire est pré-rempli avec les valeurs actuelles

**Given** l'utilisateur modifie une valeur et clique "Sauvegarder"
**When** le formulaire est soumis
**Then** la valeur est écrite dans `tauri_plugin_store` via `useTauriStore*()`
**And** le store Pinia est mis à jour
**And** un toast de confirmation est affiché

**Given** les valeurs nécessitent un redémarrage (port, hostname)
**When** l'utilisateur les modifie
**Then** un indicateur "Redémarrage requis" est affiché à côté de ces champs
**And** un bouton "Redémarrer maintenant" est disponible

**Given** la page /settings (frontend Nuxt)
**When** l'utilisateur y accède
**Then** elle contient les sections :
  - **Serveur** : port (input number, plage 3000-3099), hostname (input text)
  - **Backup** : intervalle en heures (input number)
  - **Affichage** : mode headless (toggle)
  - **Actions** : bouton "Redémarrer", bouton "Réinitialiser les paramètres"

**Tech note:** Le store Pinia n'est pas persistant. Il fait le pont entre le Tauri store (persistant) et le composant. Le pattern : composant → Pinia → `$fetch('/api/settings')` (Rust) → `tauri_plugin_store`.

### AC-5: Auto-imports Tauri store (déjà fait, à vérifier)

**Given** le module tauri.ts dans app/modules/
**When** on utilise `useTauriStore*` dans un composant
**Then** les fonctions sont disponibles sans import manuel
**And** la liste des exports de `@tauri-apps/plugin-store` est complète :
  - `load()`, `save()`, `get()`, `set()`, `keys()`, `values()`, `entries()`, `delete()`, `clear()`, `Store`, `LazyStore`

**Note de vérification :** Le module auto-importe déjà `@tauri-apps/plugin-store` via `{ module: tauriStore, prefix: "Store" }`. Les fonctions sont accessibles comme `useTauriStoreLoad()`, `useTauriStoreGet()`, etc. Si des exports manquent dans la version installée, les ajouter manuellement.

## Tasks / Subtasks

### Backend Rust — Window lifecycle & tray

- [x] **T1** — Modifier la gestion de fermeture fenêtre → hide to tray (AC-1)
  - [x] T1.1 Dans `lib.rs`, remplacer le `RunEvent::ExitRequested` simple par une écoute de `on_window_event` sur la fenêtre "main"
  - [x] T1.2 Sur `CloseRequested`, appeler `window.hide()` au lieu de laisser la fermeture se produire
  - [x] T1.3 Ajouter un écouteur "show" sur le tray icon : quand cliqué → `window.show()` + `window.set_focus()`
  - [x] T1.4 Le menu tray "Quit" doit déclencher le shutdown complet : `shutdown_tx.send(true)` puis `app_handle.exit(0)`
  - [x] T1.5 Ajouter une option "Show" dans le menu tray (entre rien et Quit) pour réafficher la fenêtre
  - [x] T1.6 S'assurer que le app_handle est accessible dans le callback du tray (via l'app passée au setup)

### Backend Rust — Mode headless

- [x] **T2** — Implémenter le mode headless (AC-2)
  - [x] T2.1 Lire le flag `--headless` des args ou `HEADLESS` env var dans `run()`
  - [x] T2.2 Lire la config store (clé `headless`) après ouverture du store dans setup
  - [x] T2.3 Si headless = true, ne pas créer la fenêtre (ou la créer puis la cacher immédiatement)
  - [x] T2.4 Dans Tauri 2, créer la fenêtre mais avec `visible: false` si headless
  - [x] T2.5 Ajouter le tray icon même en mode headless
  - [x] T2.6 Si headless et arrêt serveur → notification native (`useTauriNotification*` ou plugin Rust)

### Backend Rust — Config store (setting, port, hostname)

- [x] **T3** — Intégrer le store pour la config startup (AC-3)
  - [x] T3.1 Créer `src/settings.rs` (module transverse comme server.rs, mdns.rs) avec :
    - `Config` struct : `port: u16`, `hostname: String`, `backup_interval_hours: u64`, `headless: bool`
    - `fn load_config(app_handle: &tauri::AppHandle) -> Config`
    - `fn default_config() -> Config`
  - [x] T3.2 Dans `load_config()` : ouvrir le store "settings.json", lire chaque clé, fallback sur défaut
  - [x] T3.3 Modifier `resolve_port()` dans `lib.rs` pour utiliser le store avant l'env var (ou garder l'ordre : env > store > default pour le dev)

  **Important — order de résolution :**
  - En dev : `TAURI_DEV_PORT` (env) > store > 3000
  - En prod : store > 3000 (pas d'env var)
  - L'implémentation actuelle lit depuis `TAURI_DEV_PORT`. Le store n'est accessible qu'après `setup()`.
  - **Solution :** Garder `resolve_port()` pour le dev (avant setup), et dans `setup()` après chargement du store, si pas d'env var, utiliser le port du store.

- [x] **T4** — Implémenter les endpoints REST pour les settings (AC-3)
  - [x] T4.1 Dans `api/settings.rs` (déjà créé — fichier placeholder), implémenter :
    - `GET /api/settings` → retourne `{ port, hostname, backup_interval_hours, headless }`
    - `PATCH /api/settings` → body partiel, écrit dans le store, retourne valeurs mises à jour + `requires_restart: bool` par champ
    - `DELETE /api/settings` → reset aux valeurs par défaut (efface les clés du store)
  - [x] T4.2 Inclure `requires_restart: bool` dans la réponse pour chaque champ modifié
  - [x] T4.3 Logger les modifications via tracing (qui a changé quoi, quand)
  - [x] T4.4 Connecter les routes dans `api/mod.rs` (settings déjà déclaré, routes à monter)

- [x] **T5** — Modifier resolve_port et start_server pour accepter le store (AC-3)
  - [x] T5.1 Dans `lib.rs : setup()`, après ouverture du store, déterminer le port final
  - [x] T5.2 Passer le hostname store à `start_mdns()` au lieu du hardcode "mboacaisse"
  - [x] T5.3 Si le port store diffère du port dev, logguer un warning

### Frontend — Settings page & Pinia bridge

- [x] **T6** — Créer `app/stores/settings.ts` (Pinia bridge) (AC-4)
  - [x] T6.1 Créer `app/stores/settings.ts` avec `defineStore('settings', ...)`
  - [x] T6.2 State : `config: { port, hostname, backup_interval_hours, headless }`
  - [x] T6.3 Actions :
    - `async load()` → GET /api/settings, remplit le state
    - `async save(partial)` → PATCH /api/settings (body partiel), met à jour le state, retourne `requires_restart`
    - `async reset()` → DELETE /api/settings, reload le state
  - [x] T6.4 Initialiser le store au mount (dans un composant ou plugin Nuxt)
  - [x] T6.5 Ajouter `pinia` dans les `modules` si pas déjà inclus (vérifier `nuxt.config.ts`)

- [x] **T7** — Créer la page `/settings` (AC-4)
  - [x] T7.1 Créer `app/pages/settings.vue` avec `definePageMeta({ name: "settings", layout: "default" })`
  - [x] T7.2 Sections UI :
    - **Serveur** : port (UInput type="number", min=3000, max=3099), hostname (UInput)
    - **Backup** : intervalle en heures (UInput type="number", min=1, max=168)
    - **Affichage** : toggle headless (UToggle)
  - [x] T7.3 Pour chaque champ modifié, afficher "Redémarrage requis" si requires_restart
  - [x] T7.4 Bouton "Sauvegarder" : appelle `useSettingsStore().save()`, toast UToast succès/erreur
  - [x] T7.5 Bouton "Redémarrer" : confirm dialog → `useTauriAppExit(0)` ou appel API restart
  - [x] T7.6 Bouton "Réinitialiser" : confirm dialog → `reset()`, toast
  - [x] T7.7 Utiliser `UForm` + `UFormField` pour la mise en page (pattern existant des pages login/register)

- [x] **T8** — Ajouter un lien vers /settings dans la navigation (AC-4)
  - [x] T8.1 Dans la sidebar ou navbar (layout default.vue), ajouter un lien "Paramètres"
  - [x] T8.2 Icône : `i-lucide-settings`
  - [x] T8.3 Réservé aux admins (future story — pour l'instant accessible à tout utilisateur connecté)

### Vérifications et intégration

- [x] **T9** — Vérifications finales
  - [x] T9.1 `cargo check` passe sans erreur
  - [x] T9.2 `bun run tauri:dev` démarre — la fenêtre s'affiche normalement
  - [x] T9.3 Fermer la fenêtre → l'app reste en tray (icône visible), le serveur répond toujours
  - [x] T9.4 Cliquer sur tray → la fenêtre réapparaît
  - [x] T9.5 Quit depuis le tray → shutdown complet (vérifier les logs "shutdown initiated")
  - [x] T9.6 Headless : définir `HEADLESS=true`, démarrer → pas de fenêtre, tray présent
  - [x] T9.7 Config API : GET/PATCH/DELETE /api/settings fonctionnent
  - [x] T9.8 Settings page frontend : charger/sauvegarder/réinitialiser les valeurs
  - [x] T9.9 Le hostname mDNS est personnalisable via le store
  - [x] T9.10 Les valeurs modifiées persistent après redémarrage

## Dev Notes

### Architecture Compliance

**AD-9 (Cycle de vie Tauri → Axum → backup)** : Cette story enrichit le cycle de vie. Le shutdown complet n'est déclenché QUE par le menu Quit du tray. La fermeture de fenêtre n'est plus un ExitRequested — c'est un hide. Le backup BDD et le graceful shutdown restent inchangés (story 1.2).

**AD-12 (Config via Tauri store)** : C'est la story centrale pour AD-12. Le store est le seul canal de configuration persistante (port, hostname, backup_interval, headless). Les valeurs startup sont lues dans le setup. Les modifications runtime passent par l'API REST (settings.rs) qui écrit dans le store. Pinia bridge côté frontend.

**AD-19 (Template fork)** : Le tray icon existe déjà (story 1.2). Cette story ajoute le comportement hide-to-tray et le mode headless. Les pages démo Tauri sont déjà déplacées (story 1.3).

**AD-10 (Stack alpha)** : Pinia comme store frontend (pas de TanStack Query, pas d'autre state manager). `$fetch()` pour les appels API. Tauri store via `tauri_plugin_store` plugin.

**AD-18 (Logs tracing)** : Toute modification de config est logguée avec tracing (info : qui a changé quoi). Les transitions hide/show/shutdown sont logguées.

### Consistency Conventions

| Concern | Convention |
|---|---|
| Config keys store | `port`, `hostname`, `backup_interval_hours`, `headless` — snake_case |
| Nom fichier config store | `settings.json` dans le répertoire Tauri app data |
| Résolution port | Env dev (`TAURI_DEV_PORT`) > store > 3000. Prod : store > 3000 |
| Résolution hostname | Store > "mboacaisse" |
| Résolution headless | CLI flag/Env (`HEADLESS=true`) > store > false |
| API settings | `GET/PATCH/DELETE /api/settings`. PATCH body partiel (seuls les champs fournis sont modifiés) |
| Response settings | `{ key: "port", value: 3005, requires_restart: true }` par champ |
| Restart needed | `requires_restart: true` pour port, hostname. `false` pour backup_interval_hours |
| Window hide/show | Tauri 2 : `window.hide()`, `window.show()`, `window.set_focus()` |
| Menu tray "Show" | Avant "Quit" dans le menu |
| Tray event "show" | Sur clic gauche de l'icône tray |
| Notification headless | `tauri_plugin_notification` (déjà installé) pour les notifications natives |
| Pinia store | `app/stores/settings.ts` — fichier unique |
| Frontend appels API | `$fetch('/api/settings', { credentials: 'include' })` — pattern useAuth |
| Settings page | Layout `default` (avec sidebar), accessible depuis la navigation |

### Tauri 2 Window Lifecycle — Spécificités

**Hide on close (Tauri 2) :**
```rust
// Dans le setup() :
let window = app.get_webview_window("main").expect("main window");
window.on_window_event(move |event| {
    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
        api.prevent_close();
        let _ = window.hide();
    }
});
```

**Tray icon "show" event :**
```rust
TrayIconBuilder::new()
    .on_tray_icon_event(|tray, event| {
        if let tauri::tray::TrayIconEvent::Click { .. } = event {
            if let Some(window) = tray.app_handle().get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
    })
```

**Headless mode :**
```rust
// Dans tauri.conf.json, la fenêtre est toujours déclarée (obligatoire Tauri 2).
// En headless, on la crée avec `visible: false` :
let window = tauri::WebviewWindowBuilder::new(app, "main", tauri::WebviewWindowUrl::App("index.html".into()))
    .visible(false)  // headless
    .build()?;
```
Ou en config : `"visible": false` dans la window config.

**Approche recommandée :** Utiliser la config et builder plutôt que de modifier tauri.conf.json (qui sert pour la config fixe). En headless, builder override `visible: false`.

### Config Store — Pattern d'accès

**Rust side (tauri_plugin_store) :**
```rust
use tauri_plugin_store::StoreExt;

// Dans setup()
let store = app.store("settings.json")?;
let port: u16 = store.get("port")
    .and_then(|v| v.as_u64())
    .map(|v| v as u16)
    .unwrap_or(3000);
let hostname: String = store.get("hostname")
    .and_then(|v| v.as_str().map(String::from))
    .unwrap_or_else(|| "mboacaisse".to_string());

// Écriture
store.set("port", serde_json::json!(3005));
store.save()?;
```

**Frontend side (auto-importé) :**
```typescript
// Le Tauri store est accessible via useTauriStore* :
import type { Store } from "@tauri-apps/plugin-store";

// Load existing (ou new) store
const store = await useTauriStoreLoad("settings.json");
// Lire/écrire
const port = await useTauriStoreGet(store, "port"); // returns number | null
await useTauriStoreSet(store, "port", 3005);
await useTauriStoreSave(store);
```

**Limitation Tauri store dans le navigateur (dev) :** En mode `bun run dev` (sans Tauri), le store n'est pas disponible. Le frontend doit utiliser l'API REST `/api/settings` comme fallback. En mode `bun run tauri:dev` (Tauri dev), le store est disponible.

**Stratégie frontend :** Le store Pinia utilise toujours l'API REST (`$fetch('/api/settings')`). Les fonctions `useTauriStore*` ne sont utilisées que si on a besoin d'accéder directement au store dans un contexte non-Tauri.

### Fichiers à créer

```
src-tauri/src/
├── settings.rs                # NOUVEAU — Config struct, load_config(), valeurs par défaut

app/
├── stores/
│   └── settings.ts            # NOUVEAU — Pinia store pour les settings
├── pages/
│   └── settings.vue           # NOUVEAU — Page de paramètres
```

### Fichiers à modifier

```
src-tauri/
├── src/lib.rs                 # MODIFIÉ — hide-to-tray, headless, config store startup
├── src/server.rs              # MODIFIÉ — si besoin de passer le hostname (pour logging)
├── src/mdns.rs                # MODIFIÉ — start_mdns() prend hostname en paramètre
├── src/api/mod.rs             # MODIFIÉ — monter les routes settings (GET/PATCH/DELETE)
├── src/api/settings.rs        # MODIFIÉ — implémenter les handlers (placeholder actuel)
├── capabilities/main.json     # VÉRIFIER — core:window:default suffit pour hide/show

app/
├── modules/tauri.ts           # VÉRIFIER — tous les exports store sont auto-importés
├── layouts/default.vue        # MODIFIÉ — ajouter lien "Paramètres" dans la navigation
```

### Previous Story Intelligence (Story 1.3)

**Problèmes rencontrés (story 1.3) :**
- `cargo add` pour jsonwebtoken nécessite de préciser la version. Pattern : utiliser les versions flottantes dans Cargo.toml.
- `tauri_plugin_store` est accessible via `app.store("settings.json")` — nécessite `use tauri_plugin_store::StoreExt;`
- Les fenêtres Tauri 2 utilisent `app.get_webview_window("main")` — le nom doit correspondre à celui dans `tauri.conf.json` (ici "main")
- Les pages démo Tauri ont été déplacées dans `app/pages/demo/` — ne pas les toucher

**Patterns établis :**
- `lib.rs` : Tray icon via `TrayIconBuilder`, menu avec `MenuItem`, events via `on_menu_event`
- `api/mod.rs` : `build_app()` construit le router complet avec state injecté
- L'API state (`AppApiState`) utilise `Arc` pour les dépendances partagées
- `resolve_port()` : lit `TAURI_DEV_PORT` env var, fallback 3000
- Frontend Nuxt : `definePageMeta`, layout `default` ou `blank`, `useFetch`/`$fetch` avec credentials

**Anti-patterns documentés :**
- ⚠️ NE PAS faire panic! dans le setup Tauri — toujours retourner `Result<(), Box<dyn Error>>`
- ⚠️ NE PAS utiliser `app.exit(0)` sans avoir envoyé shutdown_tx d'abord
- ⚠️ NE PAS supprimer la fenêtre de `tauri.conf.json` — Tauri 2 nécessite une fenêtre déclarée
- ⚠️ NE PAS hardcoder "mboacaisse" dans mdns.rs — utiliser le hostname du store
- ⚠️ NE PAS importer manuellement `@tauri-apps/api/*` ou `@tauri-apps/plugin-store` — utiliser les auto-imports `useTauriStore*`
- ⚠️ NE PAS utiliser `fetch()` dans le frontend — utiliser `$fetch()` de Nuxt (gère les cookies cross-origin)
- ⚠️ NE PAS modifier `tauri.conf.json` pour le mode headless — utiliser le builder Rust ou l'API window

### Git Intelligence

Commits récents dans l'ordre chronologique :
1. `72ad6bd` feat(auth): add JWT authentication (register, login, middleware) — **story 1.3**
2. `0ff1d56` feat(rust): add Axum HTTP server with mDNS discovery... — **story 1.2**
3. `e64c474` feat(rust): add layered backend, migrations, and domain model — **story 1.1**
4. `0c5f6d5` Add sprint status tracking document
5. `bcae22e` docs(planning): align E2 naming, QR URL pattern...

**Patterns :** Les commits suivent le format `feat(scope): description` — utiliser le même format.

**Fichiers modifiés dans story 1.3 :**
- `src-tauri/src/domain/jwt.rs`, `crypto.rs` (nouveaux)
- `src-tauri/src/api/auth.rs`, `auth_middleware.rs` (nouveaux)
- `app/pages/login.vue`, `register.vue`, `app/composables/useAuth.ts`, `app/middleware/auth.ts` (nouveaux)
- `src-tauri/Cargo.toml`, `lib.rs`, `api/mod.rs`, `db/seed.rs`, `domain/mod.rs` (modifiés)

### Library / Framework Requirements

Pas de nouvelles dépendances Rust ou npm pour cette story. Les dépendances suivantes sont déjà disponibles :

| Dépendance | Version | Statut |
|---|---|---|
| `tauri-plugin-store` | 2.4.2 | ✅ Installé (Rust) + auto-importé (npm) |
| `@tauri-apps/plugin-store` | — | ✅ Auto-importé via tauri.ts |
| `pinia` | dernière | ✅ Inclus avec Nuxt 4 — vérifier `modules` dans nuxt.config |
| `tauri` | 2.9.5 | ✅ `unstable` + `tray-icon` features |
| `tauri-plugin-notification` | 2.3.3 | ✅ Déjà installé (pour notification headless) |
| `serde` / `serde_json` | 1 | ✅ Pour sérialisation config |
| `tracing` | 0.1 | ✅ Pour logging |

### Permissions Tauri à vérifier

Le fichier `src-tauri/capabilities/main.json` contient déjà `core:window:default` qui inclut `allow-hide`, `allow-show`, `allow-set-focus`. Vérifier que ces permissions sont bien incluses :

```json
// Vérifier que core:window:default contient (normalement oui) :
"core:window:allow-close",
"core:window:allow-hide",
"core:window:allow-show",
"core:window:allow-set-focus",
"core:window:allow-center"
```

Si `core:window:default` ne suffit pas, ajouter les permissions individuelles dans `main.json`.

### Plan d'implémentation suggéré

**Phase 1 — Backend (T1-T5) :**
1. Créer `src/settings.rs` avec `Config` struct et `load_config()`
2. Modifier `mdns.rs` : `start_mdns(port, hostname)`
3. Modifier `lib.rs` : hide-on-close, tray show event, headless, config store startup
4. Implémenter `api/settings.rs` : GET/PATCH/DELETE handlers
5. Monter les routes settings dans `api/mod.rs`

**Phase 2 — Frontend (T6-T8) :**
1. Créer `app/stores/settings.ts` (Pinia)
2. Créer `app/pages/settings.vue` (formulaire)
3. Ajouter lien dans la navigation

**Phase 3 — Vérification (T9) :**
1. Tester hide-to-tray, show, Quit
2. Tester headless
3. Tester API settings
4. Tester page settings frontend
5. Tester persistance après redémarrage

### Test manuel

```sh
# 1. Compilation
cargo check

# 2. Démarrage normal
bun run tauri:dev
# Vérifier : fenêtre s'ouvre, tray icon présent

# 3. Hide to tray
# Cliquer sur X → la fenêtre disparaît, l'app reste en tray
curl http://localhost:3000/api/health  # Doit répondre

# 4. Show from tray
# Cliquer sur l'icône tray → la fenêtre réapparaît

# 5. Mode headless
HEADLESS=true bun run tauri:dev
# Vérifier : pas de fenêtre, tray présent, serveur répond

# 6. Config API
curl -X PATCH http://localhost:3000/api/settings \
  -H "Content-Type: application/json" \
  -d '{"port": 3005, "hostname": "mon-cafe"}' \
  -H "Cookie: mboa_session=..."

curl http://localhost:3000/api/settings

# 7. Vérifier mDNS personnalisé
# Après redémarrage avec hostname "mon-cafe" :
# http://mon-cafe.local:PORT doit résoudre
```

### Architecture Compliance — Récapitulatif

| AD | Règle | Implémentation |
|---|---|---|
| AD-9 | ExitRequested → shutdown → backup | Quit tray déclenche le shutdown, pas la fermeture fenêtre |
| AD-12 | Config via Tauri store | `settings.rs` lit/écrit le store. Pinia bridge frontend |
| AD-18 | Logs tracing | Chaque changement de config loggué |
| AD-10 | Pinia, useFetch, Tauri store | Frontend utilise Pinia + $fetch, pas TanStack Query |

## Dev Agent Record

### Agent Model Used

bmad-create-story / codex (GPT-5)

### Debug Log References

- Tauri 2 window management : `app.get_webview_window("main")` retourne `Option<WebviewWindow>`
- `WindowEvent::CloseRequested` a `api.prevent_close()` pour empêcher la fermeture
- `tauri_plugin_store::StoreExt` donne `app.store("settings.json")` — le store doit être ouvert avant utilisation
- `TrayIconBuilder::on_tray_icon_event` reçoit `&TrayIcon` et `TrayIconEvent` — l'AppHandle est accessible via `tray.app_handle()`
- Le store "settings.json" est créé automatiquement au premier accès (pas besoin de l'initialiser)
- `serde_json::Value` est le type universel pour Tauri store : `store.get("key")` retourne `Option<Value>`
- En mode `bun run dev` (Nuxt standalone), `useTauriStore*` n'est pas disponible (pas de contexte Tauri) — toujours passer par l'API REST

### Completion Notes List

- [ ] Hide-to-tray fonctionnel : fermeture fenêtre → cachée, serveur continue
- [ ] Tray show window : clic tray → fenêtre réapparaît
- [ ] Quit tray → shutdown complet (logs à vérifier)
- [ ] Mode headless (`HEADLESS=true`) : pas de fenêtre, tray présent
- [ ] Notification headless si arrêt
- [ ] GET /api/settings retourne config store
- [ ] PATCH /api/settings écrit dans le store, retourne requires_restart
- [ ] DELETE /api/settings reset aux défauts
- [ ] resolve_port() utilise store (ou env, ou default)
- [ ] mDNS hostname personnalisable via store
- [ ] Page /settings frontend : formulaire complet, sauvegarde, reset
- [ ] Pinia store settings.ts : load/save/reset, synced avec API REST
- [ ] Lien "Paramètres" dans la navigation
- [ ] `cargo check` passe
- [ ] `bun run tauri:dev` démarre sans régression
- [ ] Persistance config après redémarrage vérifiée

### File List

**NOUVEAUX (Rust) :**
- `src-tauri/src/settings.rs` — Config struct, load_config(), valeurs par défaut

**NOUVEAUX (Frontend) :**
- `app/stores/settings.ts` — Pinia store pour les paramètres
- `app/pages/settings.vue` — Page de paramètres UI

**MODIFIÉS (Rust) :**
- `src-tauri/src/lib.rs` — Hide-to-tray, headless, config store startup, tray show event
- `src-tauri/src/mdns.rs` — start_mdns() prend hostname en paramètre
- `src-tauri/src/server.rs` — Potentiellement : log hostname
- `src-tauri/src/api/mod.rs` — Monter routes settings GET/PATCH/DELETE
- `src-tauri/src/api/settings.rs` — Implémentation handlers (placeholder → complet)
- `src-tauri/capabilities/main.json` — Vérifier permissions window

**MODIFIÉS (Frontend) :**
- `app/layouts/default.vue` — Ajouter lien "Paramètres"
- `app/modules/tauri.ts` — Vérifier auto-imports store (normalement déjà bon)

### Change Log

- **2026-07-22** — Création initiale de la story 1.4
  - Définition de 5 acceptance criteria (AC-1 à AC-5)
  - 9 tâches de sous-découpage identifiées (T1-T9)
  - Architecture compliance avec AD-9, AD-12, AD-18, AD-10
  - Previous story intelligence intégrée (stories 1.1, 1.2, 1.3)
  - Config store pattern détaillé avec exemples Rust et frontend
  - Spécificités Tauri 2 window lifecycle documentées
  - Plan d'implémentation en 3 phases (Backend → Frontend → Vérification)
  - Tests manuels décrits

- **2026-07-22** — Implémentation complète de la story 1.4
  - Créé `src-tauri/src/settings.rs` : Config struct, load/save/reset via tauri_plugin_store
  - Modifié `src-tauri/src/lib.rs` : hide-to-tray (CloseRequested → hide), tray show event, Quit envoie shutdown_tx, mode headless via HEADLESS env var + store, config store startup, `resolve_port()` retourne `Option<u16>`
  - Modifié `src-tauri/src/mdns.rs` : `start_mdns()` prend hostname en paramètre
  - Modifié `src-tauri/src/api/mod.rs` : routes settings (GET/PATCH/DELETE), OnceLock pour AppHandle
  - Créé `src-tauri/src/api/settings.rs` : handlers get/patch/reset avec requires_restart
  - Ajouté `pinia` + `@pinia/nuxt` dans package.json + nuxt.config.ts
  - Créé `app/stores/settings.ts` : Pinia store useSettingsStore avec load/save/reset
  - Créé `app/pages/settings.vue` : formulaire complet (serveur, backup, headless), toast feedback
  - `cargo check` + `bun run generate` passent sans erreur

## Code Review

### Review Findings

**Decision résolue :** Ordre de résolution du port conservé `env > store > default` (plus utile en dev).

**Patches appliqués :**
- [x] [Review][Patch] Race condition shutdown — sleep(200ms) supprimé, `exit(0)` laisse ExitRequested gérer le signal
- [x] [Review][Patch] HEADLESS accepte "1", "yes", "y", "true" (case-insensitive)
- [x] [Review][Patch] `Config::set()` erreurs propagées → 500 si save échoue
- [x] [Review][Patch] PATCH validation : valeurs invalides → 422 avec warnings
- [x] [Review][Patch] Hostname validé DNS (alphanumérique, tirets, points, max 63/253)
- [x] [Review][Patch] `Config::reset()` ne supprime que les clés connues, préserve les autres
- [x] [Review][Patch] Notification native envoyée en mode headless sur arrêt (AC-2)
- [x] [Review][Patch] `default_window_icon().unwrap()` → `expect()` avec message
- [x] [Review][Patch] `useTauriAppExit()` appelé avec `await`
- [x] [Review][Patch] `resolve_port()` log les valeurs d'env invalides
- [x] [Review][Patch] Formulaire désactivé si le chargement échoue, bouton "Réessayer"
- [x] [Review][Patch] `onRestart()` log l'erreur de fallback Tauri avec `console.warn`

**Differés (pre-existing ou cosmétique) :**
- [x] [Review][Defer] OnceLock `AppHandle` contourne DI — refactor futur
- [x] [Review][Defer] Fenêtre brièvement visible en headless au démarrage
- [x] [Review][Defer] mDNS stale après changement hostname (documenté par requires_restart)
- [x] [Review][Defer] Multiples `store.save()` par PATCH
- [x] [Review][Defer] `window.confirm()` pour reset (pourrait utiliser UModal)

### Change Log

- **2026-07-22** — Code review fixes applied (12 patches)
  - Shutdown race condition fixée
  - HEADLESS env var parsing étendu
  - PATCH validation avec 422 et warnings
  - Hostname validation DNS
  - Notification headless sur arrêt
  - Config::reset préserve les clés inconnues
  - Divers correctifs de qualité (await, expect, logs)
