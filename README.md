# MboaCaisse

> Application interne de gestion pour bar / restaurant / épicerie.
> **Devise** : FCFA | **Réseau** : LAN local, sans internet obligatoire.
> **Stack** : Tauri 2 + Axum embarqué + SQLite + Nuxt 4.

## Stack

| Couche | Technologie |
|---|---|
| Desktop | Tauri 2 (fenêtre native + tray icon) |
| Serveur HTTP | Axum (tokio) — intégré dans le processus Tauri |
| Base de données | SQLite (via libsql, embarqué, thread-safe, async) |
| Frontend | Nuxt 4 + NuxtUI v4 (Vue 3, TailwindCSS v4) |
| Langage | Rust |
| Package manager | [bun](https://bun.sh) (enforced) |

## Architecture

Le backend Axum est lancé dans le processus Tauri et fait office de serveur web complet. Il sert à la fois le frontend Nuxt (fichiers statiques `dist/`) et l'API REST. Pas de plugin Localhost — Axum écoute sur `0.0.0.0:PORT` et est accessible depuis tout le LAN.

```
Machine serveur (Tauri)
├── Fenêtre native  →  Caissier / Admin (http://localhost:PORT)
└── Serveur Axum    →  Vendeurs, Stock, Cuisine (http://mboacaisse.local:PORT)
```

Les autres postes (tablettes, PC caisse, téléphones) accèdent via navigateur sur `http://mboacaisse.local:PORT` (découverte mDNS automatique).

### Découverte réseau

L'application s'annonce sur le LAN comme `mboacaisse.local` via mDNS. Aucune configuration DNS requise. URL de secours : l'IP locale est affichée au démarrage.

## Fonctionnalités — Roadmap

Voir [`FEATURES.md`](./FEATURES.md) pour la roadmap complète priorisée (P0–P4).

### Socle (P0)
- Serveur HTTP Axum embarqué
- Base de données SQLite
- Authentification & sessions (4 rôles)
- Fenêtre native & tray icon
- Découverte réseau mDNS

### Modules métier (P1)
- Backup / restore BDD
- Notifications OS
- Shell (ouverture fichiers, commandes système)

### Fonctionnalités avancées (P2)
- Impression thermique ESC/POS native
- Afficheur client (seconde fenêtre)
- Scan code-barres
- Auto-updater
- Raccourcis globaux
- WebSocket temps réel

### Packaging (P3)
- Linux (.deb, .rpm, .AppImage)
- macOS (.dmg)
- Windows (.msi, .exe)

### Licensing (P4)
- Système de licences offline-first
- Signature cryptographique Ed25519
- Feature gating par entitlements

## Rôles

| Rôle | Périmètre |
|---|---|
| Admin | Supervision, employés, paramètres, rapports |
| Caissier | Encaissement, clôture caisse, échanges, solde client |
| Vendeur | Prise de commande, service en salle (pas d'argent) |
| Gestionnaire stock | Produits, catégories, stocks, fournisseurs |

## Architecture du backend Rust

```
src-tauri/src/
├── main.rs           # Entry point Tauri
├── lib.rs            # Builder Tauri, plugins, setup
├── server.rs         # Lancement Axum + shutdown handle
├── mdns.rs           # Publication mDNS (mboacaisse.local)
├── db/               # Migrations + seed
├── api/              # Routes Axum par domaine
├── domain/           # Structures pures (pas de dépendance Tauri/Axum)
└── license/          # Vérification Ed25519 + entitlements
```

## Commands

```sh
bun run tauri:dev       # Dev (port libre 3000-3099, puis Tauri)
bun run tauri:build     # Prod (nuxt generate + tauri build)
bun run tauri:build:debug
bun run lint            # ESLint (--fix)
bun run generate        # nuxt generate
bun run bump            # bumpp — version bump (fichiers seulement)
```

## Plugins Tauri

| Plugin | Statut |
|--------|--------|
| `shell` | ✅ |
| `notification` | ✅ |
| `os` | ✅ |
| `fs` | ✅ |
| `store` | ✅ (bridge Pinia à faire) |
| `barcode-scanner` | ❌ À installer |
| `updater` | ❌ À installer |
| `global-shortcut` | ❌ À installer |
| `autostart` | ❌ À installer |
| `log` | ❌ À installer |

## Prérequis

1. [Prérequis Tauri](https://v2.tauri.app/start/prerequisites/) (Rust, dépendances système)
2. [bun](https://bun.sh) — installé via le projet (enforced via `packageManager`)
3. `bun install`
4. `bun run tauri:dev`

## Build

```sh
bun run tauri:build       # Artefacts dans src-tauri/target/release/bundle/
bun run tauri:build:debug
```

Formats : Linux (.deb, .rpm, .AppImage) | macOS (.dmg) | Windows (.msi)

## Documentation

- [`FEATURES.md`](./FEATURES.md) — Roadmap des fonctionnalités par priorité
- [`docs/architecture-mboacaisse.md`](./docs/architecture-mboacaisse.md) — Architecture détaillée
- [`docs/systeme-de-licences.md`](./docs/systeme-de-licences.md) — Système de licences
- [`AGENTS.md`](./AGENTS.md) — Conventions du projet pour les agents IA

## License

MIT © 2025-PRESENT — fork de [Nuxtor](https://github.com/NicolaSpadari/nuxtor) par Nicola Spadari.
