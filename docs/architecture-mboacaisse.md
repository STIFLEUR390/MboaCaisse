# Architecture — MboaCaisse

Architecture cross-platform, local-first et LAN-first pour la gestion de bar / restaurant / épicerie.
Fonctionne sans connexion Internet obligatoire.

---

## Principe

Une seule installation Tauri sur la machine principale joue le rôle de serveur local.
Tous les autres appareils (PC, tablettes, téléphones) accèdent à l'application via leur navigateur sur le réseau local.

```
RÉSEAU LOCAL (WiFi)
       │
       ├── PC PRINCIPAL ─── Tauri 2 (serveur Axum + SQLite + fenêtre native)
       ├── PC CAISSE    ─── navigateur
       ├── TABLETTE CUISINE ─── navigateur
       └── TABLETTE SERVEUR ─── navigateur
```

---

## Diagramme d'architecture

```
                    RÉSEAU LOCAL
                         │
                         ▼
               http://mboacaisse.local
                         │
                         ▼
           ┌─────────────────────────┐
           │      Tauri 2            │
           │                         │
           │  ┌─────────────────┐    │
           │  │    Axum         │    │
           │  │    Tokio        │    │
           │  │                 │    │
           │  │  REST API       │    │
           │  │  WebSocket      │    │
           │  │  Fichiers stat. │    │
           │  └───────┬─────────┘    │
           │          │              │
           │          ▼              │
           │  ┌──────────────┐       │
           │  │   SQLite     │       │
           │  │ mboacaisse.db│       │
           │  └──────────────┘       │
           │          │              │
           │     mDNS : mboacaisse.local
           │                         │
           └─────────────────────────┘
```

---

## Stack technique

| Couche | Technologie |
|--------|-------------|
| Desktop | Tauri 2 (fenêtre native, tray icon, plugins) |
| Backend | Rust / Axum (tokio) — intégré dans Tauri |
| Base de données | SQLite — fichier `mboacaisse.db` |
| Frontend | Nuxt 4 / Vue 3 / TailwindCSS v4 (servi par Axum) |
| Temps réel | WebSocket natif Axum |
| Réseau local | mDNS (`mboacaisse.local`) via `mdns-sd` |
| Package manager | bun (enforced) |

---

## Rôles du serveur

L'application Tauri joue deux rôles simultanément :

### 1. Serveur HTTP (Axum)

- Sert les fichiers statiques du frontend Nuxt (`dist/`)
- Expose l'API REST (`/api/*`)
- WebSocket pour le temps réel
- Écoute sur `0.0.0.0:PORT`, accessible depuis tout le LAN
- Publie le service mDNS `mboacaisse.local`

### 2. Fenêtre native

- Interface caissier / administrateur
- Charge `http://localhost:PORT` via la WebView Tauri
- Accès aux plugins système (impression, notifications, etc.)

---

## Frontend — Vue.js

**Stack :** Vue 3 + TypeScript + Vite + Vue Router + Pinia + TanStack Query

```
app/
├── router.ts
├── stores/
├── components/
├── layouts/
│   ├── AuthLayout.vue
│   ├── MainLayout.vue
│   └── KitchenLayout.vue
├── views/
│   ├── Login.vue
│   ├── Dashboard.vue
│   ├── Orders.vue
│   ├── Tables.vue
│   ├── Kitchen.vue
│   ├── Cashier.vue
│   ├── Stock.vue
│   ├── Reports.vue
│   └── Settings.vue
├── services/
│   ├── api.ts
│   └── websocket.ts
└── types/
```

---

## Backend — Rust + Axum

**Stack :** Rust / Tokio / Axum / SQLx / Serde / Tracing / Tower

```
Vue.js
  │
  │ HTTP
  ▼
Axum
  ├── Auth
  ├── Business
  └── API
       │
       ▼
     SQLx
       │
       ▼
     SQLite
```

Toutes les opérations passent par Axum. Le frontend ne parle jamais directement à SQLite.

---

## WebSocket — Temps réel

```
Commande Table 12
  │
  ▼
Backend Axum
  ├── SQLite
  └── WebSocket
       ├── Cuisine   →  réception immédiate
       ├── Caisse    →  notification
       └── Manager   →  notification
```

**Protocole :** HTTP REST + WebSocket

---

## Découverte réseau — mDNS

| Méthode | URL | Priorité |
|---------|-----|----------|
| mDNS | `http://mboacaisse.local:PORT` | Principale |
| IP locale | `http://192.168.x.x:PORT` | Secours |
| QR Code | Généré au démarrage | Utilisateur |

**Bibliothèque :** `mdns-sd` (Rust, pure, multi-plateforme)

Attention : l'AP Isolation / Client Isolation / Guest Network des routeurs peut bloquer la communication entre appareils WiFi. L'application inclut un diagnostic réseau :

```
MboaCaisse Network Check
✓ WiFi connecté
✓ Serveur accessible
✓ mDNS actif
✓ Base de données OK
✓ WebSocket OK
```

---

## Plugins Tauri

| Plugin | Utilité |
|--------|---------|
| `autostart` | Démarrage automatique au boot du PC serveur |
| `log` | Journalisation (fichier `mboacaisse.log`) |
| `updater` | Mise à jour automatique (backup BDD avant MAJ) |
| `store` | Paramètres locaux (port, nom, backup_path, etc.) |
| `dialog` | Interactions utilisateur |
| `notification` | Alertes OS |
| `process` | Redémarrage propre |
| `shell` | Commandes système (Caddy, etc.) |

**Exclu :** `tauri-plugin-localhost` — remplacé par Axum qui sert tout directement.

---

## Sauvegardes

```
SQLite
 ├── Backup automatique quotidien
 ├── Backup avant mise à jour
 ├── Backup manuel
 └── Rotation (daily / monthly)
```

Structure :

```
backups/
├── daily/
│   ├── 2026-07-21.db
│   ├── 2026-07-20.db
│   └── 2026-07-19.db
└── monthly/
    ├── 2026-07.db
    └── 2026-06.db
```

Sauvegardes externes recommandées : disque USB, puis cloud (optionnel).

---

## Gestion des pannes

| Scénario | Comportement |
|----------|-------------|
| Internet coupé | Application continue de fonctionner (LAN uniquement) |
| Routeur redémarre | mDNS résout la nouvelle IP automatiquement |
| PC serveur redémarre | Autostart Tauri → SQLite → Axum → mDNS → prêt |
| SQLite corrompue | Restauration depuis le dernier backup |

---

## Architecture du monorepo

```
mboacaisse/
├── apps/
│   ├── desktop/          # Tauri (fenêtre native + serveur)
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── server.rs
│   │   │   ├── mdns.rs
│   │   │   └── commands.rs
│   │   └── tauri.conf.json
│   └── web/              # Frontend Nuxt
│       ├── src/
│       └── ...
├── crates/
│   ├── server/           # Routes, middleware, WebSocket
│   ├── database/         # Models, repositories, migrations
│   ├── auth/             # Authentification
│   ├── domain/           # Métier (orders, payments, inventory, users)
│   └── network/          # mDNS
├── infrastructure/
│   └── caddy/
│       └── Caddyfile
└── Cargo.toml
```

---

## Cross-platform

| Poste | Technologie |
|-------|-------------|
| Serveur (PC principal) | Tauri 2 — Windows, macOS, Linux |
| Caissier | Fenêtre native Tauri (optionnel) ou navigateur |
| Vendeur / Cuisine / Manager | Navigateur uniquement (Chrome, Edge, Firefox, Safari) |
| Tablettes / Téléphones | Navigateur (Android Chrome, iOS Safari) |

L'application n'a pas besoin d'être compilée pour Android ou iOS — les clients utilisent le navigateur.

---

## Évolution

**Version simple (V1) :** Axum direct sur `0.0.0.0:PORT` sans reverse proxy.
**Version avancée :** Caddy en reverse proxy devant Axum pour servir sur le port 80.
