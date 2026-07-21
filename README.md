# MboaCaisse

> Application interne de gestion pour bar / restaurant / épicerie.
> Devise : FCFA | Accès : personnel (4 rôles) | Réseau : LAN local, sans internet.

## Stack

| Couche | Technologie |
|---|---|
| Desktop | Tauri 2 (fenêtre native + serveur HTTP intégré via plugin Localhost) |
| Backend | Axum (tokio) — intégré dans le processus Tauri |
| Base de données | Turso / libsql (embarqué, thread-safe, async, SQLite-compatible) |
| Frontend | Nuxt 4 + NuxtUI v4 (Vue 3, TailwindCSS v4) |
| Langage | Rust |
| Package manager | [bun](https://bun.sh) (enforced) |

## Architecture

Le plugin [Tauri Localhost](https://v2.tauri.app/fr/plugin/localhost/) expose l'UI via HTTP. La machine serveur lance une fenêtre native **et** un serveur HTTP. Les autres postes (tablettes vendeurs, gestionnaire stock) se connectent via navigateur sur `http://IP_SERVEUR:PORT`.

```
Machine serveur (Tauri)
├── Fenêtre native  →  Caissier / Admin (localhost)
└── Serveur HTTP    →  Vendeurs, Stock (navigateur LAN)
```

Autres postes : navigateur web — interface responsive (tablette, téléphone, PC).

## Commands

```sh
bun run tauri:dev       # Dev (port libre 3000-3099, puis Tauri)
bun run tauri:build     # Prod (nuxt generate + tauri build)
bun run tauri:build:debug
bun run lint            # ESLint (--fix)
bun run generate        # nuxt generate
bun run bump            # bumpp — version bump (fichiers seulement)
```

## Rôles

| Rôle | Périmètre |
|---|---|
| Admin | Supervision, employés, paramètres, rapports |
| Caissier | Encaissement, clôture caisse, échanges, solde client |
| Vendeur | Prise de commande, service en salle (pas d'argent) |
| Gestionnaire stock | Produits, catégories, stocks, fournisseurs |

## Modules clés

- **POS** — Encaissement espèces/solde/mobile money, scan code-barres, ticket thermique 58/80mm
- **Commandes** — Cycle Pending → Served → Closed → Archived, split de table, dédicace
- **Solde client** — Porte-monnaie / dette client, historique immuable
- **Stock** — Entrées fournisseur, ajustements, alertes seuil bas
- **Afficheur client** — Second écran avec montant et articles
- **Rapports** — Ventes par produit/employé/période, écarts de caisse, export CSV/PDF

## Setup

1. Prérequis Rust : [tauri.app/start/prerequisites](https://tauri.app/start/prerequisites)
2. `bun install`
3. `bun run tauri:dev`

## Build

```sh
bun run tauri:build     # Artefacts dans src-tauri/target/
bun run tauri:build:debug
```

Formats Linux : `.deb`, `.rpm`, `.AppImage` | macOS : `.dmg` | Windows : `.msi`

## Documentation

Voir [`SPEC.md`](./SPEC.md) pour le cahier des fonctionnalités complet.

## License

MIT © 2025-PRESENT — fork de [Nuxtor](https://github.com/NicolaSpadari/nuxtor) par Nicola Spadari.
