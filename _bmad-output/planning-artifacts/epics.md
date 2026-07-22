---
stepsCompleted:
  - step-01-validate-prerequisites
  - step-02-design-epics
  - step-03-create-stories
  - step-04-final-validation
inputDocuments:
  - prd-MboaCaisse-2026-07-21/prd.md
  - architecture-MboaCaisse-2026-07-21/ARCHITECTURE-SPINE.md
  - ux-designs/ux-MboaCaisse-2026-07-21/DESIGN.md
  - ux-designs/ux-MboaCaisse-2026-07-21/EXPERIENCE.md
---

# MboaCaisse - Epic Breakdown

## Overview

This document provides the complete epic and story breakdown for MboaCaisse, decomposing the requirements from the PRD, UX Design, and Architecture requirements into implementable stories.

## Requirements Inventory

### Functional Requirements

**Serveur LAN embarqué (P0)**
- FR-1: Serveur Axum embarqué — lance Axum dans tokio::spawn au setup() de Tauri, écoute sur 0.0.0.0:PORT, sert dist/ + /api/*, fenêtre native pointe sur http://localhost:PORT
- FR-2: Découverte mDNS — publication service mDNS mboacaisse.local via mdns-sd
- FR-3: Fenêtre native + tray — fenêtre Tauri 1366×768, min 375×812, tray icon avec Quit, mode headless

**Authentification & Rôles (P0)**
- FR-4: Authentification — email + argon2, JWT cookie HTTP-only, middleware Axum, bootstrap admin au premier démarrage
- FR-5: 4 rôles et permissions — admin, caissier, vendeur, gestionnaire_stock, middleware guard par rôle

**Wallet Client (P0 — Cœur)**
- FR-6: Identification client — enregistrement par téléphone ou ID interne CLI-XXXX
- FR-7: Wallet multi-sources — Cash, MoMo, Gift, Cashback, Transfer, solde = SUM(amount) sur wallet_ledger
- FR-8: Wallet ledger append-only (P0 strict) — INSERT-only, backup 5min, migration des commandes antérieures
- FR-9: Payment gate — paiement déduit AVANT validation commande, solde insuffisant = message, wallet négatif optionnel
- FR-10: Crédit wallet manuel — caissier crédite wallet client (montant + type source), pas d'appel API externe

**Ordres & Produits (P0)**
- FR-11: Gestion des produits — CRUD produits (nom, prix, catégorie, stock, seuil alerte), catégories hiérarchiques
- FR-12: Cycle de vie commande — pending_payment → paid_preparing → ready → delivered, transitions horodatées
- FR-13: Kitchen display — écran dédié liste commandes en cours, filtres, notification sonore

**Paiements & Impression (P0)**
- FR-14: Encaissement — wallet, espèces, MoMo (label), combinaison, wallet prioritaire si client identifié
- FR-15: Impression thermique native — buffer ESC/POS, écriture /dev/usb/lp* ou TCP:9100, 58mm/80mm

**Fidélité & Parrainage (P0)**
- FR-16: Cashback automatique — 5% par défaut, progressif configurable (3/5/8%), crédité en type=cashback
- FR-17: Parrainage — 1000 FCFA sur wallet parrain + filleul à l'enregistrement

**Table & Menu Public QR (P0)**
- FR-18: QR code par table — QR généré serveur, URL encodant numéro de table, admin peut entrer numéro manuellement
- FR-19: Menu public 5 écrans — landing QR → menu → panier → identification téléphone → confirmation
- FR-20: Table management — CRUD tables, association client→table→commande, plan des tables

**Feature Gating & Licences (P0)**
- FR-21: Licence Ed25519 — vérification locale signature Ed25519, clé publique embarquée dans le binaire
- FR-22: Feature flags — entitlements contrôlent affichage UI + accès API (vérifié double côté)
- FR-23: Activation initiale — saisie clé d'activation, Installation ID, licence stockée localement, grace period 7 jours

**Rapports & Backup (P0)**
- FR-24: Rapports de base — journalier (ventes par caissier, par mode), hebdo/mensuel, exportable
- FR-25: Backup/Restore — auto quotidien + avant update, manuel via UI, rotation, restore UI avec double confirmation

**P1**
- FR-26: Nom mDNS personnalisable — changement nom mDNS au setup, fallback IP

**P2**
- FR-27: Mode restaurant — pré-commande, kitchen display amélioré, assignation serveur, édition commande
- FR-28: Inventaire fournisseurs — fournisseurs, alertes seuil, devis, réception commande, multi-dépôt
- FR-29: WebSocket Axum — mise à jour temps réel plan des tables, cuisine, afficheur client

### NonFunctional Requirements

- NFR-1: Intégrité wallet — zéro perte, ledger append-only, backup 5min, pas de UPDATE/DELETE
- NFR-2: Offline d'abord — 100% fonctionnalités sans Internet, 30 jours d'autonomie
- NFR-3: Résilience triangle — Wallet/Ledger/Impression indépendants, chaque sous-système survit si un autre est down
- NFR-4: Performance — encaissement→ticket <3s, commande→cuisine <2s, menu <1s, backup <1s
- NFR-5: Sécurité — argon2, JWT HTTP-only, feature flags vérifiés API, licence Ed25519, CSP désactivé
- NFR-6: Traçabilité — toute transaction financière wallet_ledger avec timestamp, type, montant, référence
- NFR-7: Diagnostic réseau — WiFi, serveur, mDNS, BDD, WebSocket exposé pour support distant

### Additional Requirements (Architecture)

- **Starter template**: Le projet est un fork du template Nuxtor (backend Rust vide, pages démo). Toute implémentation repart de zéro dans api/domain/db.
- **Structure**: api/ (peau fine) → domain/ (comportement métier) → db/ (repositories rusqlite). Structure plate par couche (AD-1, AD-3).
- **Architecture decisions**: 20 ADs couvrant paradigme, append-only financier, auth JWT, stack (rusqlite pas SQLx, useFetch pas TanStack Query), polling 5s cuisine, config Tauri store, migrations refinery, pool r2d2, 3-layer errors, tracing, UUID v7, ISO 8601, snake_case, backup gracefull shutdown
- **Dépendances Rust à ajouter**: Axum, Tokio, rusqlite, r2d2, refinery, argon2, mdns-sd, tracing, uuid
- **Pages démo Tauri à supprimer/cacher**: commands.vue, file.vue, notifications.vue, os.vue, store.vue, webview.vue
- **Config store**: tauri_plugin_store pour port, mDNS hostname, backup interval, stock bas seuil, moyens paiement
- **Seed idempotent**: admin + 10 produits / 3 catégories au premier démarrage
- **Backup graceful shutdown**: on_event(ExitRequested) → Axum graceful → backup BDD, timeout 5s
- **Diagnostic réseau**: endpoint GET /api/health

### UX Design Requirements

**Design tokens & visual system (DESIGN.md)**
- UX-DR1: Appliquer le système de tokens MboaCaisse — palette vert/zinc, typographie Inter (display 28px/700, heading 18px/600, body 14px/400, price 16px/700, caption 12px/400)
- UX-DR2: Implémenter les composants design — ButtonPrimary (vert, blanc, md rayon), ButtonSecondary (transparent, bordure), CardProduct (blanc, bordure, md rayon), PhoneInput
- UX-DR3: Pricing toujours en `price` (16px/700, vert primaire) avec suffixe "FCFA"
- UX-DR4: Sticky footer "Commander" sur écrans de saisie (panier, identification)
- UX-DR5: Catégories en barre horizontale scrollable, produits en liste verticale une colonne
- UX-DR6: Feedback visuel tap sur produit → scale(0.97) 150ms
- UX-DR7: Progress statut commande — 3 steps horizontaux (commandée → en préparation → prête)

**Expérience & flows (EXPERIENCE.md)**
- UX-DR8: Implémenter les 5 écrans menu public — Landing QR, Menu (catégories + produits), Panier, Identification + Paiement, Confirmation/Statut
- UX-DR9: Landing QR → affiche numéro de table ou "À emporter", CTA "Voir le menu"
- UX-DR10: Menu → barre catégories horizontale, liste produits verticale, tap ajoute au panier
- UX-DR11: Panier → révision, modification quantités (-/+), suppression ligne, badge compteur
- UX-DR12: Identification → champ téléphone (validation 9 chiffres), deux choix paiement (wallet / comptoir)
- UX-DR13: Confirmation → numéro commande, montant, message personnalisé, bande progression statut
- UX-DR14: Microcopy — messages chaleureux et efficaces selon contexte (bienvenue, panier vide, solde insuffisant, confirmation)
- UX-DR15: États et edge cases — première visite vs retour, panier vide, solde insuffisant, table non trouvée, erreur serveur
- UX-DR16: Accessibilité WCAG 2.2 AA — zones tactiles ≥44×44px, Tab order logique, aria-live sur statut commande
- UX-DR17: Responsive mobile — 320-480px portrait cible, max-width 600px, gutter 16px
- UX-DR18: Composant Progress statut → 3 steps, actif en vert, complété en success, futur en gris
- UX-DR19: Composant PhoneInput → fond blanc, focus bordure verte, clavier numérique natif (inputmode="numeric")
- UX-DR20: Composant CollapsableOptions → chevron, tap déplie avec animation, padding uniforme
- UX-DR21: PaymentChoice → deux boutons wallet/comptoir, wallet désactivé si solde insuffisant + explication

### FR Coverage Map

| FR | Epic | Description |
|---|---|---|
| FR-1 | E1 | Serveur Axum embarqué |
| FR-2 | E1 | Découverte mDNS |
| FR-3 | E1 | Fenêtre native + tray |
| FR-4 | E1 | Authentification JWT |
| FR-5 | E1 | 4 rôles et permissions |
| FR-6 | E1.5 | Identification client par téléphone |
| FR-7 | E1.5 | Wallet ledger append-only |
| FR-8 | E1.5 | Migration wallet_ledger (commandes antérieures) |
| FR-9 | E3 | Payment gate |
| FR-10 | E3 | Crédit wallet manuel |
| FR-11 | E3 | Gestion des produits |
| FR-12 | E3 | Cycle de vie commande |
| FR-13 | E3 | Kitchen display |
| FR-14 | E3 | Encaissement multi-moyen |
| FR-15 | E3 | Impression thermique native |
| FR-16 | E2 | Cashback automatique |
| FR-17 | E2 | Parrainage |
| FR-18 | E4 | QR code par table |
| FR-19 | E4 | Menu public 5 écrans |
| FR-20 | E4 | Table management |
| FR-21 | E5 | Licence Ed25519 |
| FR-22 | E5 | Feature flags |
| FR-23 | E5 | Activation initiale |
| FR-24 | E5 | Rapports de base |
| FR-25 | E5 | Backup/Restore |
| FR-26 | E5 | Nom mDNS personnalisable |
| FR-27 | E5 | Mode restaurant (P2) |
| FR-28 | E5 | Inventaire fournisseurs (P2) |
| FR-29 | E5 | WebSocket Axum (P2) |

## Epic List

### Epic 1: Socle Serveur & Authentification
Le serveur Axum tourne dans le processus Tauri, accessible sur le LAN via mDNS. Authentification par email/argon2 avec session JWT. 4 rôles avec permissions. Fenêtre native configurable. Le socle sur lequel tout le produit repose.

**FRs covered:** FR-1, FR-2, FR-3, FR-4, FR-5

### Epic 1.5: Wallet Ledger
Les clients sont identifiés par téléphone. Le wallet ledger append-only (INSERT-only) est créé et opérationnel. Le paiement wallet peut être utilisé dans le cycle de vente. Migration des commandes antérieures si existantes.

**FRs covered:** FR-6, FR-7, FR-8

### Epic 3: Ventes & Encaissement
Cycle complet de la vente : CRUD produits et catégories, création commande, payment gate (débit wallet avant validation), encaissement multi-moyen (wallet, espèces, MoMo label), crédit wallet manuel, kitchen display, impression thermique.

**Dépend sur:** E1.5 (Wallet ledger pour payment gate)

**FRs covered:** FR-9, FR-10, FR-11, FR-12, FR-13, FR-14, FR-15

### Epic 2: Fidélité (après E3)
Cashback automatique 5% (progressif 3/5/8%) sur chaque paiement wallet. Parrainage 1000 FCFA à l'enregistrement. Règles métier branchées sur le ledger existant.

**Dépend sur:** E1.5 (Wallet ledger), E3 (volume de ventes)

**FRs covered:** FR-16, FR-17

### Epic 4: Menu Public QR
5 écrans menu public (Landing → Menu → Panier → Identification → Confirmation/Statut). QR code par table. Table management (CRUD, plan des tables). Design tokens MboaCaisse (Inter, vert/zinc, progress steps, etc.).

**Dépend sur:** E3 (produits, cycle commande)

**FRs covered:** FR-18, FR-19, FR-20
**UX-DRs:** UX-DR1 à UX-DR21

### Epic 5: Administration
Licence Ed25519 avec vérification offline, feature gating par entitlements, activation initiale. Rapports journaliers/hebdo/mensuels. Backup/Restore automatique. Nom mDNS personnalisable.

**FRs covered:** FR-21, FR-22, FR-23, FR-24, FR-25, FR-26, FR-27, FR-28, FR-29

## Epic 1: Socle Serveur & Authentification

Le serveur Axum tourne dans le processus Tauri, accessible sur le LAN via mDNS. Authentification par email/argon2 avec session JWT. 4 rôles avec permissions. Fenêtre native configurable.

**FRs covered:** FR-1, FR-2, FR-3, FR-4, FR-5

### Story 1.1: Structure Rust Layered & Migrations Initiales

As a developer,
I want le projet Rust structuré en api/domain/db avec les dépendances et une migration initiale,
So that l'équipe peut implémenter chaque couche sans conflit et la BDD est versionnée dès le départ.

**Acceptance Criteria:**

**Given** le workspace Rust actuel (Cargo.toml, lib.rs, main.rs)
**When** on ajoute les dépendances (Axum, Tokio, rusqlite, r2d2, refinery, argon2, mdns-sd, tracing, uuid, tower-http)
**Then** le projet compile avec `cargo check` sans erreur

**Given** les dépendances installées
**When** on crée la structure `src/api/`, `src/domain/`, `src/db/` avec leurs mod.rs
**Then** la hiérarchie est importable depuis lib.rs

**Given** le dossier `migrations/` avec un fichier `migrations/V1__users.sql` au format refinery (SQL brut, timestampé, forward-only)
**When** la migration crée la table `users` (id TEXT PK, email TEXT UNIQUE NOT NULL, password_hash TEXT NOT NULL, name TEXT, role TEXT NOT NULL DEFAULT 'caissier', created_at TEXT NOT NULL, updated_at TEXT NOT NULL)
**Then** la table users existe après exécution de la migration

**Given** Role enum dans domain/user.rs avec variants Admin, Caissier, Vendeur, GestionnaireStock
**When** chaque variant implémente `fn permissions(&self) -> Vec<Permission>`
**Then** les permissions sont dérivées du rôle, pas stockées en BDD

**Given** refinery::Runner configuré dans db/migrations.rs
**When** l'application démarre
**Then** les migrations sont exécutées avant que le serveur ne commence à écouter
**And** si une migration échoue, le processus exit avec un message d'erreur

### Story 1.2: Serveur Axum Embarqué & mDNS

As a developer,
I want un serveur Axum qui sert le frontend et l'API, et un service mDNS qui publie mboacaisse.local,
So que le LAN peut accéder à l'application sans config IP.

**Acceptance Criteria:**

**Given** les dépendances Axum et tower-services dans Cargo.toml
**When** on crée `src/server.rs` avec un router Axum qui sert `dist/` (fichiers statiques) et monte `/api/*`
**Then** le frontend est accessible à `http://localhost:PORT`

**Given** un port configurable (défaut 3000, plage 3000-3099)
**When** le serveur démarre sur `0.0.0.0:PORT`
**Then** n'importe quel navigateur du LAN peut charger l'UI à `http://IP_SERVEUR:PORT`

**Given** la crate mdns-sd dans Cargo.toml
**When** on crée `src/mdns.rs` qui publie `_http._tcp.local` avec le service `mboacaisse.local`
**Then** `http://mboacaisse.local:PORT` résout sur tout le LAN

**Given** on_event(ExitRequested) dans lib.rs
**When** l'utilisateur ferme la fenêtre ou le système envoie un signal d'arrêt
**Then** shutdown_tx envoie un signal graceful shutdown à Axum
**And** le serveur a 5s pour terminer les requêtes en vol avant de forcer l'arrêt

**Given** une BDD wallet_ledger (qui n'existe pas encore en E1)
**When** le shutdown est demandé
**Then** le backup BDD est déclenché avant l'arrêt complet

**Given** le serveur Axum dans un tokio::spawn depuis lib.rs setup()
**When** Tauri initialise le setup()
**Then** Axum écoute sur le port configuré avant que la fenêtre ne soit affichée

**Given** le shutdown_tx
**When** le signal est reçu
**Then** Axum arrête d'accepter de nouvelles connexions et termine les existantes dans le timeout

### Story 1.3: Authentification — Register, Login & JWT

As a user,
I want pouvoir créer un compte et me connecter avec email + mot de passe,
So que seuls les utilisateurs autorisés accèdent au système.

**Acceptance Criteria:**

**Given** la table users migrée
**When** un nouvel utilisateur s'enregistre avec email et mot de passe
**Then** le mot de passe est hashé avec argon2
**And** un JWT signé est retourné dans un cookie HTTP-only

**Given** un utilisateur enregistré
**When** il se connecte avec email + mot de passe correct
**Then** un JWT 24h est émis dans un cookie `mboa_session`

**Given** un utilisateur avec JWT valide
**When** il accède à une route protégée
**Then** le middleware Axum vérifie le token et autorise l'accès

**Given** un JWT expiré
**When** l'utilisateur accède à une route protégée
**Then** il est redirigé vers la page login

**Given** il reste moins d'1h avant expiration du JWT
**When** l'utilisateur fait une requête
**Then** le refresh silencieux émet un nouveau JWT

**Given** l'utilisateur se déconnecte
**When** il appelle l'endpoint de logout
**Then** le cookie est détruit et l'accès aux routes protégées est révoqué

**Given** la BDD est vierge (premier démarrage)
**When** le système démarre
**Then** un écran de création admin est affiché (ou identifiants générés dans la console)
**And** aucun compte admin existant = pas de page login normale

**Given** le Role enum (Admin, Caissier, Vendeur, GestionnaireStock)
**When** l'utilisateur se connecte
**Then** le rôle est encodé dans le JWT et accessible par le middleware

### Story 1.4: Fenêtre Native & Tray & Mode Headless

As a commerçant,
I want la fenêtre Tauri configurable avec tray icon et mode headless,
So que le serveur tourne même si la fenêtre est fermée ou sur un PC partagé.

**Acceptance Criteria:**

**Given** tauri.conf.json avec minWidth 375, minHeight 812, width 1366, height 768
**When** l'application démarre
**Then** la fenêtre native s'ouvre avec ces dimensions

**Given** l'icône tray est configurée (menu Quit)
**When** l'utilisateur ferme la fenêtre
**Then** le serveur continue à tourner en arrière-plan (pas d'arrêt)

**Given** le mode headless
**When** configuré via tauri_plugin_store ou flag CLI
**Then** aucune fenêtre ne s'ouvre, le serveur est actif, notification si arrêt

**Given** tauri_plugin_store initialisé avec port (3000), hostname (mboacaisse), backup_interval (24h)
**When** le serveur démarre
**Then** la config est chargée depuis le store et utilisée par server.rs

**Given** un bridge Pinia vers le Tauri store
**When** le frontend a besoin de lire/écrire la config
**Then** les valeurs sont synchronisées entre Rust (store) et Nuxt (Pinia)

**Given** les auto-imports Tauri (app/modules/tauri.ts)
**When** le plugin store est utilisé
**Then** les fonctions useTauriStore* sont disponibles sans import manuel

### Story 1.5: Rôles & Permissions — Middleware Guard & Seed Admin

As a admin,
I want que chaque utilisateur ait un rôle avec des permissions dérivées,
So that l'accès aux fonctionnalités est contrôlé par rôle.

**Acceptance Criteria:**

**Given** 4 rôles définis dans domain/user.rs (Admin, Caissier, Vendeur, GestionnaireStock)
**When** un utilisateur est créé avec un rôle
**Then** ses permissions sont dérivées via `Role::permissions()`

**Given** Admin implémente Permission::All
**When** un admin accède à n'importe quelle route
**Then** l'accès est autorisé

**Given** Caissier a Permission::Sell, Permission::ViewReports
**When** un caissier accède à la caisse
**Then** l'accès est autorisé

**Given** Caissier n'a pas Permission::ManageUsers
**When** un caissier accède à la gestion des employés
**Then** 403 Forbidden est retourné

**Given** Vendeur a Permission::ViewOrders, Permission::ManageMenu
**When** un vendeur accède aux commandes
**Then** l'accès est autorisé
**And** un vendeur n'a pas accès à la caisse (403)

**Given** GestionnaireStock a Permission::ManageStock, Permission::ViewReports
**When** un gestionnaire accède au stock
**Then** l'accès est autorisé

**Given** la BDD est vierge
**When** le seed s'exécute
**Then** un compte admin est créé avec email + mot de passe généré
**And** le seed est idempotent (ne crée pas de doublon au redémarrage)

**Given** le middleware role-check dans api/mod.rs
**When** une route protégée est appelée sans JWT
**Then** 401 Unauthorized est retourné
**When** une route protégée est appelée avec un rôle insuffisant
**Then** 403 Forbidden est retourné

## Epic 1.5: Wallet Ledger

Les clients sont identifiés par téléphone. Le wallet ledger append-only (INSERT-only) est créé et opérationnel. Le paiement wallet peut être utilisé dans le cycle de vente. Migration des commandes antérieures si existantes.

**FRs covered:** FR-6, FR-7, FR-8

### Story 1.5.1: Wallet Domain & Migration

As a developer,
I want la table wallet_ledger append-only et les entités WalletClient / WalletLedgerEntry,
So that le socle financier est immuable et les transactions sont tracées.

**Acceptance Criteria:**

**Given** une migration V2__wallet_ledger.sql
**When** exécutée
**Then** la table `wallet_ledger` est créée (id TEXT PK, client_id TEXT NOT NULL, type TEXT NOT NULL, amount INTEGER NOT NULL, reference TEXT, description TEXT, created_at TEXT NOT NULL)
**And** la table `wallet_clients` est créée (id TEXT PK, phone TEXT UNIQUE, name TEXT, created_at TEXT NOT NULL, updated_at TEXT NOT NULL)
**And** la table `wallet_ledger` est INSERT-only (pas de UPDATE/DELETE en production — via trigger SQL ou permission BDD)

**Given** les entités dans domain/wallet.rs
**When** WalletClient (id, phone, name, created_at, updated_at) et WalletLedgerEntry (id, client_id, type, amount, reference, description, created_at) sont définis
**Then** WalletRepository trait expose: register_client, find_by_phone, find_by_id, append_entry, get_balance, get_ledger(client_id)

**Given** un INSERT dans wallet_ledger
**When** la transaction SQL est BEGIN → SELECT SUM → INSERT → COMMIT dans une même session
**Then** le solde est calculé atomiquement, pas de race condition

### Story 1.5.2: API Wallet — Identification & Solde

As a caissier,
I want pouvoir enregistrer un client par téléphone et voir son solde,
So que le client peut payer avec son wallet.

**Acceptance Criteria:**

**Given** POST /api/wallet/register avec { phone, name? }
**When** le téléphone est valide (9 chiffres)
**Then** un WalletClient est créé avec un UUID v7
**And** la réponse retourne l'ID client et le solde (0 FCFA)

**Given** POST /api/wallet/register avec un téléphone déjà existant
**When** le téléphone est déjà enregistré
**Then** 409 Conflict est retourné

**Given** GET /api/wallet/by-phone/{phone}
**When** le client existe
**Then** la réponse retourne { id, phone, name, balance (SUM du ledger), created_at }

**Given** GET /api/wallet/by-phone/{phone}
**When** le client n'existe pas
**Then** 404 Not Found est retourné

**Given** GET /api/wallet/{id}/ledger?limit=50
**When** le client existe
**Then** la réponse retourne les N dernières entrées du ledger avec solde calculé

**Tech note:** Le solde est calculé par SELECT SUM à chaque lecture (O(n)). À optimiser avec un cache wallet_clients.balance mis à jour via trigger SQL ou transaction atomique si les performances deviennent un problème.

### Story 1.5.3: Migration Wallet Ledger

As a admin,
I want que les commandes payées avant l'implémentation du ledger soient rejouées dans wallet_ledger,
So que le solde des clients existants est correct dès l'activation.

**Acceptance Criteria:**

**Given** des commandes payées dans la table orders avant l'implémentation du ledger
**When** le script de migration s'exécute
**Then** une ligne INSERT avec type='migration' est créée par commande payée, montant total, dans wallet_ledger
**And** les clients concernés ont un wallet créé si inexistant

**Given** le script est ré-exécuté
**When** des lignes avec type='migration' existent déjà
**Then** aucune ligne en double n'est créée (idempotent)
**And** les lignes migration sont ignorées

**Given** aucune commande payée n'existe avant le ledger
**When** le script s'exécute
**Then** aucune ligne migration n'est créée, pas d'erreur

## Epic 3: Ventes & Encaissement

Cycle complet de la vente : CRUD produits et catégories, création commande, payment gate (débit wallet avant validation), encaissement multi-moyen (wallet, espèces, MoMo label), crédit wallet manuel, kitchen display, ticket numérique.

**Dépend sur:** E1.5 (Wallet ledger pour payment gate)

**FRs covered:** FR-9, FR-10, FR-11, FR-12, FR-13, FR-14, FR-15

### Story 3.1: CRUD Produits & Catégories

As a gérant,
I want pouvoir créer, modifier et supprimer des produits et catégories,
So que le menu de l'établissement est à jour.

**Acceptance Criteria:**

**Given** une migration V4__products.sql
**When** exécutée
**Then** la table `categories` est créée (id TEXT PK, name TEXT NOT NULL, parent_id TEXT nullable, created_at TEXT NOT NULL, updated_at TEXT NOT NULL)
**And** la table `products` est créée (id TEXT PK, name TEXT NOT NULL, price INTEGER NOT NULL, category_id TEXT NOT NULL REFERENCES categories(id), stock INTEGER DEFAULT 0, alert_threshold INTEGER DEFAULT 5, created_at TEXT NOT NULL, updated_at TEXT NOT NULL)

**Given** les entités dans domain/product.rs
**When** Product (id, name, price, category_id, stock, alert_threshold) et Category (id, name, parent_id) sont définis
**Then** ProductRepository trait expose: create, update, delete, find_by_id, list_by_category, search

**Given** POST /api/products avec { name, price, category_id, stock?, alert_threshold? }
**When** les données sont valides
**Then** un produit est créé avec UUID v7 et retourné

**Given** DELETE /api/products/{id}
**When** le produit existe
**Then** il est retiré du menu mais pas supprimé des commandes passées

**Given** une catégorie parente
**When** une sous-catégorie référence parent_id
**Then** la hiérarchie est stockée (pas de récursion infinie)

### Story 3.2: Cycle de Vie Commande

As a caissier,
I want créer une commande, la faire passer par ses statuts (pending_payment → paid_preparing → ready → delivered),
So que la cuisine, le serveur et le client savent où en est la commande.

**Acceptance Criteria:**

**Given** une migration V5__orders.sql
**When** exécutée
**Then** la table `orders` est créée (id TEXT PK, table_id TEXT, client_id TEXT, status TEXT NOT NULL, total INTEGER NOT NULL, created_at TEXT NOT NULL, updated_at TEXT NOT NULL)
**And** la table `order_items` est créée (id TEXT PK, order_id TEXT NOT NULL REFERENCES orders(id), product_id TEXT NOT NULL, quantity INTEGER NOT NULL, unit_price INTEGER NOT NULL, notes TEXT)

**Given** domain/order.rs avec Order, OrderItem, OrderStatus enum (PendingPayment, PaidPreparing, Ready, Delivered)
**When** Order::new crée une commande avec status=PendingPayment
**Then** chaque transition de statut est horodatée et validée

**Given** POST /api/orders avec { table_id?, client_id?, items: [{ product_id, quantity, notes? }] }
**When** les articles existent et les quantités sont > 0
**Then** une commande est créée avec status=PendingPayment, total calculé (SUM unit_price * quantity)

**Given** une commande avec status=PaidPreparing
**When** le cuisine appelle PATCH /api/orders/{id}/status avec status=ready
**Then** la commande passe à ready

**Given** une commande avec status=PendingPayment
**When** on tente de passer à delivered
**Then** 422 Unprocessable Entity est retourné (transitions invalides)

### Story 3.3: Payment Gate

As un caissier,
I want que le paiement soit déduit du wallet AVANT validation de la commande,
So que le solde client est garanti avant que la cuisine prépare.

**Acceptance Criteria:**

**Given** WalletRepository du wallet ledger (E1.5)
**When** POST /api/payments avec { order_id, method: "wallet", client_id }
**Then** le système vérifie le solde (SELECT SUM du wallet_ledger) avant de débiter
**And** si solde >= total → INSERT ligne dans wallet_ledger avec type='payment', amount=-total
**And** la commande passe à PaidPreparing
**And** la réponse retourne { status: "paid", new_balance }

**Given** solde insuffisant
**When** le payment gate est appelé
**Then** 422 est retourné avec { error: "Solde insuffisant", balance, total }
**And** la commande reste en PendingPayment

**Given** POST /api/payments avec { order_id, method: "cash" }
**When** le paiement est en espèces
**Then** la commande passe à PaidPreparing (pas de débit wallet)

**Given** le wallet négatif est désactivé (par défaut)
**When** un débit rendrait le solde négatif
**Then** le paiement est refusé

**Given** le wallet négatif est activé (paramètre admin)
**When** un débit rend le solde négatif
**Then** le paiement est accepté, le solde devient négatif

**Given** deux paiements simultanés sur le même wallet
**When** les deux transactions s'exécutent
**Then** chaque transaction est atomique (BEGIN → SELECT SUM → INSERT → COMMIT), pas de race condition

### Story 3.4: Encaissement Multi-Moyen & Crédit Manuel

As a caissier,
I want pouvoir encaisser une commande en combinant wallet, espèces et MoMo,
So que le client paie comme il veut.

**Acceptance Criteria:**

**Given** POST /api/payments avec { order_id, method: "split", payments: [{ method: "wallet", amount: 2000 }, { method: "cash", amount: 1500 }] }
**When** la somme des paiements = total de la commande
**Then** chaque méthode est traitée séparément (wallet débité, espèces enregistrées)
**And** la commande passe à PaidPreparing

**Given** POST /api/payments avec { order_id, method: "momo" }
**When** le caissier sélectionne Orange Money ou MTN MoMo
**Then** le label est enregistré en BDD (pas d'appel API externe)
**And** la commande passe à PaidPreparing

**Given** POST /api/wallet/{client_id}/credit avec { amount, source: "cash" | "momo" | "gift", reference? }
**When** le caissier crédite le wallet du client
**Then** une ligne INSERT dans wallet_ledger avec type=credit, amount=+amount, source en description
**And** le solde du client est mis à jour immédiatement

**Given** POST /api/wallet/{client_id}/credit
**When** le crédit est fait pendant l'encaissement (pas associé à une commande)
**Then** le credit est indépendant, pas de débit associé

### Story 3.5: Kitchen Display

As un cuisinier,
I want voir les commandes en préparation sur un écran,
So que je prépare sans attendre le ticket papier.

**Acceptance Criteria:**

**Given** GET /api/kitchen/orders
**When** appelé
**Then** retourne les commandes avec status=PaidPreparing et Ready
**And** chaque commande inclut table, items (produit, quantité, notes), timestamp

**Given** le frontend cuisine (app/pages/cuisine.vue)
**When** la page charge
**Then** elle affiche deux colonnes : "En préparation" (PaidPreparing) et "Prêtes" (Ready)
**And** chaque commande a un bouton "Prête" (Ready) et "Servie" (Delivered)

**Given** le polling HTTP (useFetch + setInterval, 5s)
**When** une nouvelle commande arrive en PaidPreparing
**Then** elle apparaît dans la colonne "En préparation" dans les 5s suivantes

**Given** une nouvelle commande PaidPreparing
**When** l'écran cuisine est ouvert
**Then** une notification sonore est jouée (Web Audio API ou simple <audio>)

**Given** le cuisinier clique "Prête"
**When** PATCH /api/orders/{id}/status avec status=ready
**Then** la commande passe dans la colonne "Prêtes"

**Given** le cuisinier clique "Servie"
**When** PATCH /api/orders/{id}/status avec status=delivered
**Then** la commande disparaît de l'écran cuisine

### Story 3.6: Ticket Numérique

As un caissier,
I want qu'un ticket de caisse soit généré et affiché après validation du paiement,
So que le client repart avec une preuve d'achat.

**Acceptance Criteria:**

**Given** une commande passe à PaidPreparing
**When** le paiement est validé
**Then** un ticket numérique est généré avec : nom établissement, date, articles (nom, qty, prix), total, mode paiement, remerciements

**Given** le ticket généré
**When** l'écran de validation s'affiche
**Then** le ticket est visible à l'écran (modal ou page dédiée)
**And** un bouton "Imprimer" est disponible (window.print() ou PDF natif)

**Given** l'imprimante n'est pas disponible ou pas configurée
**When** le ticket est généré
**Then** le ticket numérique s'affiche sans erreur (fallback fonctionnel)

## Epic 4: Menu Public QR

5 écrans menu public (Landing → Menu → Panier → Identification → Confirmation/Statut). QR code par table. Table management. Design tokens MboaCaisse (Inter, vert/zinc, progress steps, responsive mobile).

**Dépend sur:** E3 (produits, orders, payments APIs)

**FRs covered:** FR-18, FR-19, FR-20
**UX-DRs:** UX-DR1 à UX-DR21

### Story 4.1: Table Management & QR Generation

As a gérant,
I want pouvoir créer des tables et générer leur QR code,
So que les clients scannent et commandent depuis leur table.

**Acceptance Criteria:**

**Given** une migration V7__tables.sql
**When** exécutée
**Then** la table `tables` est créée (id TEXT PK, label TEXT NOT NULL, qr_url TEXT NOT NULL, status TEXT NOT NULL DEFAULT 'free', created_at TEXT NOT NULL, updated_at TEXT NOT NULL)

**Given** POST /api/tables avec { label }
**When** la table est créée
**Then** l'URL du QR est générée côté client (ou serveur) : `http://{host}:{port}/menu?table={id}`
**And** un QR code PNG est encodé (bibliothèque qrcode npm côté frontend)

**Given** GET /api/tables
**When** appelé
**Then** retourne toutes les tables avec leur statut (free/occupied)

**Given** une commande est associée à une table (POST /api/orders avec table_id)
**When** la commande passe à PaidPreparing
**Then** le statut de la table passe à occupied

**Given** une commande associée à une table passe à delivered
**When** le statut est mis à jour
**Then** la table repasse à free

**Given** GET /api/tables/plan
**When** appelé
**Then** retourne le plan des tables avec statuts (utilisé par le frontend pour la vue plan)

**Ce story dépend de:** E3 (produits, orders APIs)

### Story 4.2: Landing QR & Menu Public

As un client,
I want scanner un QR, voir le menu et ajouter des articles au panier,
So que je commande sans attendre le serveur.

**Acceptance Criteria:**

**Given** GET /menu?table={id}
**When** un client scanne le QR
**Then** la Landing page s'affiche : "Bienvenue chez [Établissement] — Table {label}" (ou "À emporter" si takeaway)
**And** un bouton "Voir le menu" est présent

**Given** le client clique "Voir le menu"
**When** la page Menu s'affiche
**Then** les catégories sont affichées en barre horizontale scrollable (UX-DR5)
**And** les produits sont listés verticalement (1 colonne) avec nom + prix FCFA en price (UX-DR3, UX-DR10)

**Given** le client tape sur un produit
**When** le produit a des options (cuisson, sauce)
**Then** un panneau collapsable se déplie avec chevron, animation simple (UX-DR20)

**Given** le client tape "Ajouter"
**When** le produit est ajouté
**Then** un feedback visuel scale(0.97) 150ms est joué (UX-DR6)
**And** le badge panier dans le header est incrémenté (UX-DR11)

**Given** une catégorie sans produits
**When** le client la sélectionne
**Then** "Rien ici pour l'instant" est affiché (UX-DR14, UX-DR15)

**Ce story dépend de:** E3 (GET /api/products, GET /api/categories)

### Story 4.3: Panier

As un client,
I want voir et modifier mon panier avant de commander,
So que je suis sûr de ma sélection.

**Acceptance Criteria:**

**Given** le client a des articles dans son panier
**When** il tape l'icône panier dans le header
**Then** l'écran Panier affiche chaque article : nom, quantité (-/+), prix ligne, bouton supprimer (UX-DR11)

**Given** le client change la quantité
**When** il tape + ou -
**Then** le prix ligne est mis à jour immédiatement
**And** le total est recalculé

**Given** le panier est vide
**When** le client arrive sur l'écran Panier
**Then** "Votre panier est vide. Ajoutez des articles depuis le menu." avec bouton "Retour au menu" (UX-DR14)

**Given** le panier a des articles
**When** le client est sur l'écran Panier
**Then** un sticky footer "Commander" est toujours visible en bas (UX-DR4)

### Story 4.4: Identification & Paiement

As un client,
I want entrer mon téléphone et choisir mon mode de paiement,
So que la commande part à la cuisine.

**Acceptance Criteria:**

**Given** le client clique "Commander" depuis le panier
**When** l'écran d'identification s'affiche
**Then** un champ téléphone avec input numérique (inputmode="numeric", 9 chiffres) est présent (UX-DR12, UX-DR19)
**And** deux choix paiement : "Payer avec mon wallet" / "Payer au comptoir" (UX-DR21)

**Given** le client entre un téléphone connu
**When** le système reconnaît le client
**Then** le solde wallet et le cashback accumulé sont affichés
**And** le bouton wallet est actif si solde ≥ total

**Given** le client entre un nouveau téléphone
**When** le téléphone est inconnu
**Then** un compte est créé avec solde 0 FCFA
**And** "Bienvenue ! Paiement au comptoir sélectionné." (UX-DR14)

**Given** le client choisit wallet mais solde insuffisant
**When** le solde < total
**Then** le wallet est grisé avec "Solde : {solde} FCFA. Paiement au comptoir recommandé." (UX-DR14, UX-DR21)

**Given** le client valide la commande
**When** POST /api/orders avec les items + client_id + table
**Then** la commande est créée en PendingPayment
**And** si wallet: POST /api/payments avec method=wallet est appelé
**And** si comptoir: POST /api/payments avec method=cash est appelé

**Ce story dépend de:** E3 (POST /api/orders, POST /api/payments, POST /api/wallet/register)

### Story 4.5: Confirmation & Statut Commande

As un client,
I want voir ma confirmation de commande et suivre son statut,
So que je sais quand ma commande est prête.

**Acceptance Criteria:**

**Given** la commande est payée et validée
**When** l'écran de confirmation s'affiche
**Then** "Commande #{N} confirmée. Merci {prénom} !" avec le montant total (UX-DR13, UX-DR14)

**Given** l'écran de confirmation
**When** la commande est en cuisine
**Then** une bande progression 3 steps est affichée (UX-DR7, UX-DR18) :
      - Step 1 "Commandée" (complété)
      - Step 2 "En préparation" (actif, green)
      - Step 3 "Prête" (futur, gris)

**Given** GET /menu/order/{id}/status
**When** le client recharge la page
**Then** le statut actuel de la commande est affiché avec la progression

**Given** la commande passe à ready
**When** le client recharge ou la page se rafraîchit
**Then** "Prête ! Bon appétit !" est affiché (UX-DR14)

**Given** un changement de statut commande
**When** la progression est mise à jour
**Then** un aria-live region annonce le changement aux lecteurs d'écran (UX-DR16)

**Ce story dépend de:** E3 (GET /api/orders/{id}, PATCH /api/orders/{id}/status)

### Story 4.6: Edge Cases & Accessibilité

As un client,
I want que l'expérience soit robuste et accessible même en cas d'erreur,
So que je ne suis jamais bloqué sans savoir quoi faire.

**Acceptance Criteria:**

**Given** le client scanne un QR avec une table invalide
**When** GET /menu?table=999
**Then** "Table non trouvée. Scannez le QR de votre table ou demandez au serveur." (UX-DR15)

**Given** une erreur serveur (500, timeout)
**When** n'importe quel écran du menu public
**Then** un écran minimaliste : "Une erreur est survenue. Réessayez ou parlez au serveur." (UX-DR15)
**And** pas de stack trace, pas de détails techniques

**Given** le premier chargement du menu (pas de cookie)
**When** le client arrive sur Landing
**Then** le CTA "Voir le menu" est affiché sans présomption

**Given** un client avec cookie de session existant
**When** il revient sur le menu
**Then** "Bonjour {prénom} !" subtil en haut (UX-DR15)
**And** le téléphone est pré-rempli à l'étape identification

**Given** toutes les pages du menu public
**When** vérifiées
**Then** zones tactiles ≥ 44×44px (UX-DR16)
**And** Tab order logique dans les formulaires
**And** contraste des textes sur fond conforme WCAG AA

**Given** l'écran est mobile (320-480px)
**When** le menu public s'affiche
**Then** pleine largeur, marges 16px (UX-DR17)
**And** contenu centré, max-width 600px sur écrans plus larges

## Epic 2: Fidélité

Cashback automatique 5% sur chaque paiement wallet. Parrainage 1000 FCFA à l'enregistrement. Règles métier branchées sur le ledger existant.

**Dépend sur:** E1.5 (Wallet ledger), E3 (volume de ventes)

**FRs covered:** FR-16, FR-17

### Story 2.1: Cashback Automatique

As un client,
I want recevoir 5% de cashback sur chaque paiement wallet,
So que ma fidélité est récompensée sans carte.

**Acceptance Criteria:**

**Given** un paiement wallet validé (POST /api/payments avec method=wallet)
**When** la commande passe à PaidPreparing
**Then** une ligne INSERT dans wallet_ledger avec type='cashback', amount=+total*0.05 (arrondi à l'entier inférieur)

**Given** une commande wallet de 5000 FCFA
**When** le cashback est calculé
**Then** 250 FCFA sont crédités (5000 * 0.05)

**Given** un cashback de 3 FCFA
**When** calculé
**Then** le montant est crédité (pas de minimum)

**Given** aucun paiement wallet
**When** le client paie en espèces ou MoMo
**Then** aucun cashback n'est crédité

**Tech note:** Taux fixe 5%. Défini comme `const CASHBACK_RATE: f64 = 0.05` dans le domaine. Pas d'UI de configuration en alpha. Si le taux devient configurable, créer une story dédiée (migration + endpoint + formulaire admin).

### Story 2.2: Parrainage

As un client,
I want parrainer un ami et recevoir 1000 FCFA,
So que nous sommes tous les deux récompensés.

**Acceptance Criteria:**

**Given** POST /api/wallet/register avec { phone, name?, referrer_phone? }
**When** referrer_phone est fourni et valide (9 chiffres)
**And** le parrain existe dans wallet_clients
**Then** 1000 FCFA sont crédités sur le wallet du parrain (type='referral_bonus', reference=id_filleul)
**And** 1000 FCFA sont crédités sur le wallet du filleul (type='referral_bonus', reference=id_parrain)

**Given** POST /api/wallet/register avec referrer_phone
**When** le parrain n'existe pas dans wallet_clients
**Then** le client est créé sans parrainage (pas de bonus, pas d'erreur)

**Given** un client déjà parrainé tente d'enregistrer un parrain
**When** le client existe déjà avec un referrer_id
**Then** aucun bonus supplémentaire n'est distribué

**Given** un client tente de se parrainer lui-même
**When** referrer_phone = son propre téléphone
**Then** 400 Bad Request : "Vous ne pouvez pas vous parrainer vous-même"

## Epic 5: Administration

Licence Ed25519 avec vérification offline, feature gating par entitlements, activation initiale. Rapports journaliers. Backup/Restore. mDNS personnalisable (P1). Extensions futures (P2).

**FRs covered:** FR-21, FR-22, FR-23, FR-24, FR-25, FR-26, FR-27, FR-28, FR-29

### Story 5.1: Licence Ed25519 & Activation

As un gérant,
I want pouvoir activer MboaCaisse avec une licence signée Ed25519,
So que le logiciel est déverrouillé et prêt à fonctionner.

**Acceptance Criteria:**

**Given** le module src/license/ avec verify.rs et entitlements.rs
**When** le système démarre
**Then** la licence est chargée depuis le fichier de config (ou grace period activée)
**And** la signature Ed25519 est vérifiée avec la clé publique embarquée dans le binaire

**Given** une licence valide
**When** la signature est vérifiée
**Then** les entitlements sont extraits du payload JSON signé
**And** le système est déverrouillé

**Given** une licence trafiquée ou invalide
**When** la signature ne correspond pas
**Then** "Licence invalide" est affiché
**And** le système refuse de démarrer les fonctionnalités

**Given** un premier démarrage sans licence
**When** le système détecte l'absence de licence
**Then** un écran d'activation est affiché avec un champ de saisie de clé
**And** un grace period de 7 jours est accordé avec toutes les features débloquées

**Given** une clé d'activation valide saisie
**When** le système génère un Installation ID (machine + timestamp)
**Then** la licence signée est téléchargée/chargée et stockée dans tauri_plugin_store

**Given** une licence alpha pré-générée
**When** le système démarre en mode alpha
**Then** les entitlements par défaut incluent : features: ["pos", "inventory", "kitchen", "reports", "loyalty"]
**And** aucun feature n'est bloqué en dev

**Tech note:** En alpha, pas de License Server cloud. La licence est fournie avec le binaire (fichier de licence dans le dossier de config). Activation = copie du fichier.

### Story 5.2: Feature Flags

As un développeur,
I want que les entitlements de la licence contrôlent l'affichage UI et l'accès API,
So que le même binaire sert tous les bundles selon la licence.

**Dépend sur:** 5.1 (Licence — les entitlements sont extraits de la licence vérifiée)

**Acceptance Criteria:**

**Given** une licence avec entitlements features: ["pos"]
**When** le frontend charge
**Then** seules les fonctionnalités POS sont visibles dans l'UI

**Given** une licence sans entitlement "kitchen"
**When** un utilisateur tente d'accéder à /cuisine
**Then** l'UI cache le lien cuisine
**And** GET /api/kitchen/* retourne 403 Forbidden

**Given** un entitlement détermine l'accès API
**When** une route protégée par feature flag est appelée
**Then** le middleware Axum vérifie l'entitlement AVANT le rôle

**Given** le même binaire
**When** deux licences différentes sont chargées (Cash vs Resto)
**Then** le comportement UI + API est différent selon les entitlements

**Given** src/license/entitlements.rs expose fn has_feature(feature: &str) -> bool
**When** appelé dans le middleware ou le frontend
**Then** la vérification est instantanée (pas d'appel réseau)

### Story 5.3: Rapports de Base

As un gérant,
I want voir mes ventes du jour ventilées par caissier et par mode de paiement,
So que je sais exactement combien j'ai gagné et où va l'argent.

**Acceptance Criteria:**

**Given** api/reports.rs avec des queries SQL directes (AD-6)
**When** GET /api/reports/daily
**Then** retourne : ventes totales, par caissier (id, name, total), par mode de paiement (wallet, cash, momo, total)

**Given** GET /api/reports/weekly et GET /api/reports/monthly
**When** les données existent sur la période
**Then** retourne les mêmes indicateurs agrégés par jour

**Given** GET /api/reports/stock-alerts
**When** des produits ont stock ≤ alert_threshold
**Then** retourne la liste des produits à réapprovisionner

**Given** GET /api/reports/daily sur une journée sans vente
**When** aucun order n'existe
**Then** retourne { total: 0, by_caissier: [], by_payment: [] }

**Given** les données de vente
**When** le rapport est généré
**Then** les montants sont en FCFA (entiers, pas de décimales)

### Story 5.4: Backup & Restore

As un gérant,
I want que mes données soient sauvegardées automatiquement et pouvoir restaurer,
So que je ne perds rien même si le PC tombe.

**Acceptance Criteria:**

**Given** une tâche tokio planifiée (intervalle configurable, défaut 24h)
**When** l'heure du backup arrive
**Then** un backup de la BDD SQLite est créé dans $APP_DATA_DIR/backups/
**And** le fichier est nommé mboacaisse-{YYYY-MM-DD}.db

**Given** un backup automatique avant mise à jour (détection de version)
**When** le binaire change
**Then** un backup est créé automatiquement avant l'exécution des migrations

**Given** le bouton "Backup manuel" dans l'UI admin
**When** le gérant clique
**Then** un backup est créé immédiatement

**Given** N backups existent (rotation, N=30)
**When** le backup N+1 est créé
**Then** le plus ancien backup est supprimé

**Given** le bouton "Restaurer" dans l'UI admin
**When** le gérant sélectionne un backup et confirme
**Then** un backup auto de l'état courant est créé (filet de sécurité)
**And** la BDD est remplacée par le backup sélectionné
**And** l'application redémarre

**Given** la double confirmation de restauration
**When** le gérant clique "Restaurer"
**Then** une modale de confirmation est affichée avec "Cette action remplacera toutes les données actuelles"
**And** le restore n'est exécuté qu'après seconde confirmation

### Story 5.5: mDNS Personnalisable (P1)

**Placeholder P1.** FR-26 : changement du nom mDNS depuis l'UI settings. Fallback IP si mDNS indisponible (AP Isolation).

### Story 5.6: Bundles & Extensions P2

**Placeholder P2.** FR-27 (Mode restaurant), FR-28 (Inventaire fournisseurs), FR-29 (WebSocket Axum)
