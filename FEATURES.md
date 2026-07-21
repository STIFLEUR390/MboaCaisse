# MboaCaisse — Cahier des fonctionnalités

> Application interne (Rust + Turso/libsql) pour la gestion d'un bar / restaurant / épicerie.
> **Devise** : FCFA | **Accès** : personnel uniquement (4 rôles)
> **Réseau** : fonctionne sur LAN/WiFi local, sans internet.

---

Sommaire

- [1. Architecture technique](#1-architecture-technique)
- [2. Base de données](#2-base-de-donnees)
- [3. Authentification & Rôles](#3-authentification--roles)
- [4. Module Admin](#4-module-admin)
- [5. Module Caissier](#5-module-caissier)
- [6. Module Vendeur (service en salle)](#6-module-vendeur-service-en-salle)
- [7. Module Gestionnaire de stock](#7-module-gestionnaire-de-stock)
- [8. Point de vente (POS)](#8-point-de-vente-pos)
- [9. Gestion des commandes](#9-gestion-des-commandes)
- [10. Solde client / Porte-monnaie](#10-solde-client--porte-monnaie)
- [11. Impression thermique](#11-impression-thermique)
- [12. Multi-écran & afficheur client](#12-multi-ecran--afficheur-client)
- [13. Packaging multi-plateforme](#13-packaging-multi-plateforme)
- [14. Configuration système détaillée](#14-configuration-systeme-detaille)
- [15. Pages de l'interface](#15-pages-de-linterface)
- [16. Sauvegarde & Restauration](#16-sauvegarde--restauration)
- [17. Migrations de schéma](#17-migrations-de-schema)
- [18. Serveur de secours](#18-serveur-de-secours)
- [19. Modules transverses](#19-modules-transverses)
- [20. Bonnes pratiques](#20-bonnes-pratiques)
- [21. Roadmap](#21-roadmap)

---

## 1. Architecture technique

### 1.1 Choix technologique : Tauri (avec plugin Localhost)

Le plugin [**Tauri Localhost**](https://v2.tauri.app/fr/plugin/localhost/) expose les ressources de l'application via un serveur HTTP local au lieu du protocole custom `tauri://`. Combiné à un backend Axum embarqué, une seule instance Tauri est **serveur web + application de bureau** simultanément.

| Critère | Tauri + Localhost | Leptos |
|---|---|---|
| **Nature** | App de bureau + serveur HTTP (les deux dans 1 binaire) | Application web full-stack |
| **Machine serveur** | Fenêtre native + API serveur intégré | Navigateur seulement |
| **Autres postes (LAN)** | Navigateur (connexion HTTP au serveur Tauri) | Navigateur |
| **Impression thermique** | **Natif** — USB, drivers OS | Limité (navigateur) |
| **Scan code-barres** | **Plugin dédié** (`tauri-plugin-barcode-scanner`) | Limité (navigateur) |
| **Raccourcis clavier** | Capturés nativement (même si fenêtre en fond) | Limités au focus navigateur |
| **Taille binaire** | ~10-20 Mo (avec backend + frontend) | ~20-40 Mo (serveur seul) |
| **Déploiement** | 1 binaire sur 1 machine + navigateur pour les autres | 1 binaire sur 1 machine + navigateur |

**Recommandation** : **Tauri** est le meilleur choix. Avec le plugin Localhost, Tauri résout le multi-utilisateur LAN : la machine serveur lance une fenêtre native (caissier/administration) ET un serveur HTTP. Les autres postes (tablettes des vendeurs) se connectent via navigateur.

### 1.2 Architecture réseau

```
┌────────────────────────────────────────────────────────────────┐
│                     Réseau local (LAN)                          │
│                                                                │
│  ┌──────────────────────────┐                                  │
│  │ Machine serveur (Tauri)   │                                  │
│  │                          │    ┌────────────────────────┐    │
│  │  ┌────────────────────┐  │    │ Postes clients         │    │
│  │  │ Fenêtre native      │  │    │ (navigateur web)      │    │
│  │  │ Caissier / Admin    │  │    │                        │    │
│  │  └────────────────────┘  │    │  ┌─────┐ ┌─────┐      │    │
│  │                          │◄───┤  │Vend │ │Vend │      │    │
│  │  ┌────────────────────┐  │    │  │eur 1│ │eur 2│      │    │
│  │  │ Serveur HTTP intégré│  │    │  └─────┘ └─────┘      │    │
│  │  │ Localhost:PORT      │──┤    │  ┌─────┐              │    │
│  │  │ + API (Axum)        │  │    │  │Stock│              │    │
│  │  └────────────────────┘  │    │  │Mgr  │              │    │
│  │         +                │    │  └─────┘              │    │
│  │     libsql (Turso)       │    └────────────────────────┘    │
│  └──────────────────────────┘                                  │
└────────────────────────────────────────────────────────────────┘
```

- Le serveur écoute sur `0.0.0.0:PORT` → la fenêtre native utilise `http://localhost:PORT`
- Les autres postes se connectent via navigateur à `http://IP_DU_SERVEUR:PORT`
- Interface **responsive** : vendeurs sur tablette/téléphone, caissier sur PC fixe
- Aucun accès internet requis

### 1.3 Stack technique

| Couche | Technologie |
|---|---|
| **Langage** | Rust |
| **Desktop framework** | Tauri 2.0 (fenêtre native + webview) |
| **Base de données** | Turso / libsql (embarqué, thread-safe, async, concurrent) |
| **Backend HTTP** | Axum (tokio) — intégré dans le processus Tauri |
| **Frontend** | [Nuxt 4](https://nuxt.com) + [NuxtUI v4](https://ui.nuxt.com) (Vue 3) |
| **State management** | Pinia + [`@tauri-store/pinia`](https://tb.dev.br/tauri-store/plugin-pinia/guide/getting-started) (persistance auto des stores) |
| **PDF / Impressions** | Bibliothèque Rust (printpdf) + API OS |
| **Déploiement** | 1 binaire sur la machine serveur, navigateur pour les autres |
| **Starter** | [Nuxtor](https://github.com/NicolaSpadari/nuxtor) — template Nuxt 4 + Tauri 2 + NuxtUI 4 |

### 1.4 Plugins Tauri

| Plugin | Utilité | Obligatoire ? |
|---|---|---|
| [`localhost`](https://v2.tauri.app/fr/plugin/localhost/) | Expose l'UI via HTTP → nécessaire pour l'accès multi-utilisateur LAN | **Oui** |
| [`autostart`](https://v2.tauri.app/fr/plugin/autostart/) | Lance l'app au démarrage du PC serveur | Recommandé |
| [`barcode-scanner`](https://v2.tauri.app/fr/plugin/barcode-scanner/) | Scan code-barres via caméra ou scanner USB | Optionnel |
| [`fs`](https://v2.tauri.app/fr/plugin/file-system/) | Export CSV/PDF, backup BDD, upload images | Recommandé |
| [`log`](https://v2.tauri.app/fr/plugin/logging/) | Journalisation Rust + frontend | Recommandé |
| [`notification`](https://v2.tauri.app/fr/plugin/notification/) | Notifications OS (nouvelle commande, stock bas) | Recommandé |
| [`updater`](https://v2.tauri.app/fr/plugin/updater/) | Mise à jour automatique de l'application | Recommandé (V2+) |
| [`global-shortcut`](https://v2.tauri.app/fr/plugin/global-shortcut/) | Raccourcis clavier même fenêtre en fond | Optionnel |
| [`websocket`](https://v2.tauri.app/fr/plugin/websocket/) | Temps réel entre fenêtre native et clients web | Optionnel |
| [`@tauri-store/pinia`](https://tb.dev.br/tauri-store/plugin-pinia/guide/getting-started) | Pont Pinia ↔ Tauri store. Persistance auto des stores Vue | Recommandé |

#### Stockage persistant

- **`@tauri-store/pinia`** = stores Pinia synchronisés avec le filesystem via Tauri
- **libsql (Turso)** = base de données principale, côté Rust

### 1.5 Projet starter : Nuxtor

Le template [**Nuxtor**](https://github.com/NicolaSpadari/nuxtor) fournit une base préconfigurée :

- Nuxt 4 + Tauri 2 + NuxtUI v4 + TailwindCSS v4
- Auto-import des API Tauri
- Icône de tray, notifications, stockage local
- ESLint + TypeScript
- Scripts `tauri:dev`, `tauri:build`

```bash
npx degit NicolaSpadari/nuxtor mon-app
cd mon-app
bun install
bun run tauri:dev
```

---

## 2. Base de données (Turso / libsql)

**Turso** est une réécriture de SQLite en Rust (projet libsql), offrant :

- **100% compatible SQLite** : requêtes SQL standard
- **Async-first** : basé sur `io_uring` (Linux) et tokio
- **Writes concurrents** : plusieurs writers sans verrouillage global
- **Embedded** : s'utilise comme une lib, pas de processus séparé
- **Chiffrement** : support du chiffrement au repos (`AES-256-GCM`)
- **Bindings Rust natifs** : `turso::sync::Builder`, `libsql::Database`

```rust
let db = libsql::Database::open("mboacaisse.db")?;
let conn = db.connect()?;
conn.execute("CREATE TABLE IF NOT EXISTS ...", ())?;
```

Fichier unique (`mboacaisse.db`), backup par simple copie.

---

## 3. Authentification & Rôles

Application interne → seuls les employés ont un compte.

| Fonctionnalité | Description |
|---|---|
| **Connexion** | Email + mot de passe (hashé argon2/bcrypt) |
| **Déconnexion** | Fin de session |
| **Rôles** | 4 rôles : `admin`, `caissier`, `vendeur`, `gestionnaire_stock` |
| **Permissions** | Contrôle d'accès par rôle sur chaque module |
| **Session persistante** | Reste connecté même si le navigateur ferme (token) |

### 3.1 Matrice des permissions

| Module | Admin | Caissier | Vendeur | Gestionnaire stock |
|---|---|---|---|---|
| Dashboard global | ✔ | ✘ | ✘ | ✘ |
| Gestion employés | ✔ | ✘ | ✘ | ✘ |
| Paramètres système | ✔ | ✘ | ✘ | ✘ |
| Produits & catégories | ✔ | ✘ | ✘ | ✔ |
| Mouvements de stock | ✔ | ✘ | ✘ | ✔ |
| Fournisseurs | ✔ | ✘ | ✘ | ✔ |
| POS / Encaissement | ✔ | ✔ | ✘ | ✘ |
| Clôture de caisse | ✔ | ✔ | ✘ | ✘ |
| Prise de commande | ✘ | ✘ | ✔ | ✘ |
| Service en salle | ✘ | ✘ | ✔ | ✘ |
| Échanges / remboursements | ✔ | ✔ | ✘ | ✘ |
| Solde client | ✔ | ✔ | Consultation | ✘ |
| Rapport ventes | ✔ | Consultation | Consultation | ✘ |
| Fermeture commande | ✘ | ✔ | ✘ | ✘ |
| Gestion des clients (fiches) | ✔ | ✔ | ✔ | ✘ |

### 3.2 Profils

- **Admin** : superviseur. Gère les employés, la configuration, consulte tous les rapports.
- **Caissier** : encaissement, caisse, clôtures de commande, échanges, solde client.
- **Vendeur** : service en salle. Prise de commande, service. Ne manipule pas d'argent.
- **Gestionnaire stock** : produits, catégories, stocks, fournisseurs.

---

## 4. Module Admin

L'admin supervise. Pas d'opérations quotidiennes.

### 4.1 Dashboard

| Fonctionnalité | Description |
|---|---|
| **Vue d'ensemble** | Cartes : nb tables, produits, clients, employés, commandes du jour |
| **Filtres date** | Aujourd'hui / semaine / mois / personnalisé |
| **Alertes** | Stock bas, commandes en attente, échanges en cours |
| **Activité récente** | Actions des employés (qui a fait quoi) |

### 4.2 Gestion des employés

| Fonctionnalité | Description |
|---|---|
| **CRUD employés** | Ajout / modification / désactivation des comptes (4 rôles) |
| **Champs** | Nom, email, téléphone, mot de passe, rôle, statut actif/inactif |

### 4.3 Rapports & supervision

| Fonctionnalité | Description |
|---|---|
| **Totaux ventes** | Aujourd'hui / Mois / Année, comparaison période précédente |
| **Ventes par employé** | Total encaissé par caissier/vendeur |
| **Ventes par produit** | Quantité vendue et CA par produit |
| **Moyens de paiement** | Répartition espèces / solde / mobile money |
| **Écarts de caisse** | Différence attendu/réel par clôture |
| **Journal d'activité** | Historique complet de toutes les actions |
| **Export rapports** | CSV ou PDF pour chaque vue |

### 4.4 Gestion des clients (fiches)

| Fonctionnalité | Description |
|---|---|
| **CRUD clients** | Nom, téléphone, email optionnel |
| **Solde** | Consultation, historique des transactions |
| **Commandes** | Historique des commandes passées |

### 4.5 Paramètres système

| Fonctionnalité | Description |
|---|---|
| **Mot de passe** | Changement du mot de passe admin |
| **Configuration** | Voir section 14 |
| **Sauvegarde BDD** | Déclenchement manuel |

---

## 5. Module Caissier

Le caissier est le seul à pouvoir **valider financièrement** une commande.

### 5.1 Dashboard

| Fonctionnalité | Description |
|---|---|
| **Vue caisse** | Total encaissé du jour, nb transactions, caisse ouverte/fermée |
| **Commandes à clôturer** | Commandes servies en attente de paiement |

### 5.2 Validation de commande

```
Vendeur prend la commande → Vendeur sert → Vendeur demande clôture
    → CAISSIER valide et encaisse → Commande soldée
```

| Fonctionnalité | Description |
|---|---|
| **Reprendre une commande** | Sélection commande "servie" — détail articles, total, vendeur, table |
| **Valider le total** | Vérification et confirmation du montant |
| **Générer le reçu** | Impression ticket de caisse |

### 5.3 Encaissement

| Fonctionnalité | Description |
|---|---|
| **Paiement espèces** | Saisie montant reçu, calcul auto du rendu |
| **Paiement par solde** | Déduction depuis le solde du client |
| **Paiement mobile money** | Enregistrement OM/MTN |
| **Paiement mixte** | Split sur plusieurs moyens |

### 5.4 Cas particuliers — Gestion des écarts

#### Client paye moins que le total

| Option | Description |
|---|---|
| **Crédit client** | Le reste en solde négatif (dette) |
| **Paiement partiel + solde** | Le reste déduit du solde si suffisant |
| **Report** | Commande en attente |

#### Client paye plus — pas de monnaie

| Option | Description |
|---|---|
| **Crédit sur solde** | Excédent crédité sur le solde client |
| **Pourboire** | Excédent lié au vendeur |

### 5.5 Gestion de caisse

| Fonctionnalité | Description |
|---|---|
| **Ouverture de caisse** | Montant de départ en début de service |
| **Clôture de caisse** | Bilan fin de service : total par moyen de paiement, nb transactions, écart |
| **Détail des écarts** | Liste des opérations ayant généré un écart |
| **Historique des clôtures** | Consultation des clôtures précédentes |

### 5.6 Échanges & remboursements

| Fonctionnalité | Description |
|---|---|
| **Échange produit** | Remplacement d'un article par un autre (même prix) |
| **Remboursement espèces** | Annulation → remboursement en espèces |
| **Remboursement sur solde** | Annulation → crédit sur le solde du client |
| **Annulation commande** | Annulation complète avec remboursement |

### 5.7 Solde client

| Fonctionnalité | Description |
|---|---|
| **Consultation** | Solde (positif ou négatif) |
| **Crédit manuel** | Rechargement du solde |
| **Historique** | Mouvements du client |
| **Règlement de dette** | Encaissement pour réduire un solde négatif |

---

## 6. Module Vendeur (service en salle)

Le vendeur ne manipule **pas d'argent**. Tout paiement passe par le caissier.

### 6.1 Prise de commande

| Fonctionnalité | Description |
|---|---|
| **Dashboard** | Stats du jour : nb commandes, tables occupées |
| **Plan des tables** | Vue visuelle avec statut (libre/occupée/commande en cours) |
| **Prise de commande** | Création pour une table, sélection produits + quantités + notes |
| **Envoi en cuisine/bar** | Transmission (impression ticket ou notification écran) |
| **Impression ticket bar/kitchen** | Ticket dédié pour la préparation |

### 6.2 Service

| Fonctionnalité | Description |
|---|---|
| **Marquage "servi"** | Passage de la commande en "servi" |
| **Ajout d'articles** | Dédicace après service |
| **Consultation** | Détail d'une commande (articles, total, notes) |

### 6.3 Demande de clôture

| Fonctionnalité | Description |
|---|---|
| **Signaler au caissier** | Table marquée "prête à payer" |
| **Notification** | Apparaît dans la liste "À clôturer" du caissier |
| **Split de table** | Demande de répartir les articles entre 2+ sous-commandes |

---

## 7. Module Gestionnaire de stock

### 7.1 Dashboard

| Fonctionnalité | Description |
|---|---|
| **Vue stocks** | Produits en rupture, stock bas, entrées récentes |
| **Alertes** | Seuils configurables, produits presque épuisés |

### 7.2 Gestion des produits

| Fonctionnalité | Description |
|---|---|
| **CRUD produits** | Nom, description, prix vente, prix achat, stock, image, catégorie |
| **Code-barres** | Association EAN-13 |
| **Activation** | Visible ou non au POS/commande |
| **Recherche** | Par nom, code-barres, catégorie |

### 7.3 Gestion des catégories

| Fonctionnalité | Description |
|---|---|
| **CRUD catégories** | Création, modification, suppression |
| **Hiérarchie** | Parent/enfant (ex: Boissons → Soda) |
| **Position** | Ordre d'affichage |
| **Activation** | Masquer sans supprimer |

### 7.4 Mouvements de stock

| Fonctionnalité | Description |
|---|---|
| **Entrée de stock** | Livraison fournisseur (produit, quantité, date, prix d'achat) |
| **Sortie de stock** | Ajustement manuel (perte, casse) |
| **Historique mouvements** | Liste chronologique entrées/sorties |
| **Ajustement** | Correction après inventaire physique |

### 7.5 Fournisseurs

| Fonctionnalité | Description |
|---|---|
| **CRUD fournisseurs** | Nom, contact, téléphone, email |
| **Historique achats** | Livraisons par fournisseur |

---

## 8. Point de vente (POS)

Interface dédiée au **caissier**, plein écran, pensée pour écran tactile au comptoir.

### 8.1 Interface caisse

| Fonctionnalité | Description |
|---|---|
| **Catalogue visuel** | Grille de produits par catégorie, ajout en 1 clic |
| **Scan code-barres** | Champ EAN-13, ajout instantané |
| **Recherche rapide** | Barre avec autocomplétion |
| **Panier** | Liste articles, modification quantités, suppression |
| **Total temps réel** | Mise à jour dynamique |
| **Pavé numérique** | Pour quantités et montants manuels |

### 8.2 Paiement

| Fonctionnalité | Description |
|---|---|
| **Moyens de paiement** | Espèces, Solde client, Mobile Money (OM/MTN), Carte bancaire |
| **Rendu monnaie** | Calcul auto si caisse a la monnaie |
| **Pas de monnaie** | Trop-perçu → solde client ou pourboire |
| **Paiement partiel + solde** | Split sur plusieurs moyens |
| **Mise en crédit** | Commande soldée, montant total en dette |

### 8.3 Modes de vente

| Fonctionnalité | Description |
|---|---|
| **Vente rapide** | Sans identification client |
| **Vente avec client** | Recherche/sélection du client avant encaissement |
| **Vente sur table** | Reprise d'une commande table |
| **Mise en attente** | Suspension du panier |
| **Réductions** | % ou montant fixe, avec motif |
| **Ajout note** | Commentaire interne |

### 8.4 Validation & tickets

| Fonctionnalité | Description |
|---|---|
| **Validation** | Passe en *Closed* immédiatement |
| **Ticket de caisse** | Impression automatique |
| **Format** | Ticket thermique 58/80mm |
| **Reçu A4** | Optionnel |

### 8.5 Raccourcis & contraintes

- Tauri : F1 = scan, F2 = recherche, F3 = paiement, F4 = attente, Échap = annuler
- Navigateur : F2 = recherche
- Validation immédiate → *Closed* (pas de *Pending*)

---

## 9. Gestion des commandes

### 9.1 Cycle de vie

```
┌─────────┐    ┌──────────┐    ┌────────┐    ┌──────────┐    ┌──────────┐
│ Pending  │──►│ Served   │──►│ Closed │──►│ Archived │
│ (vendeur)│    │ (servi)  │    │ (soldée)│    │ (fin du  │
└─────────┘    └──────────┘    └────────┘    └──────────┘    │ jour)    │
                                                              └──────────┘
     ↑                 ↑              ↑
  vendeur          vendeur        caissier
  (création)       (service)      (paiement)
```

| Étape | Qui | Description |
|---|---|---|
| **Pending** | Vendeur | Commande créée, envoyée en préparation |
| **Served** | Vendeur | Client servi, prête à être soldée |
| **Closed** | Caissier | Paiement validé, reçu imprimé |
| **Archived** | Système | Fin de service, archivée automatiquement |

### 9.2 Cas particuliers

| Situation | Comportement |
|---|---|
| **Client paye exactement** | Soldée normalement |
| **Client paye moins** | Solde négatif (dette). Commande soldée |
| **Client paye plus** | Trop-perçu : crédit solde, pourboire, ou rendu |
| **Client ne paye pas** | Montant total en dette (solde négatif) |
| **Split** | Répartition des articles en sous-commandes |
| **Dédicace** | Ajout d'articles après service |

Toute commande passe en **Closed** — un écart de paiement ne bloque jamais la clôture.

### 9.3 POS direct

Le POS (caissier) passe directement en *Closed*. Pas de *Pending* ni *Served*.

### 9.4 Échanges

Remplacement d'un article par un autre (même prix) dans une commande déjà soldée. Transactionnel : +1 ancien produit, -1 nouveau. Écart de prix → paiement ou remboursement.

---

## 10. Solde client / Porte-monnaie

### 10.1 Concept

Le solde client gère tous les cas où le **paiement n'est pas exact**. Chaque client possède un solde en FCFA, positif (crédit) ou négatif (dette).

> C'est le **système de comptes courants** du bar.

### 10.2 Scénarios

| Scénario | Comportement |
|---|---|
| Client n'a que 5 000 F, commande à 5 500 F | 500 F en solde négatif |
| Client donne 6 000 F pour 5 500 F, pas de monnaie | 500 F en crédit solde ou pourboire |
| Client : "Je reviens demain" | Commande soldée, total en dette |
| Client dépose 50 000 F | Crédit manuel, consomme au fur et à mesure |

### 10.3 Règles métier

| Règle | Description |
|---|---|
| **Solde négatif autorisé** | C'est une dette client |
| **Plafond de dette** | Seuil configurable (ex: -20 000 F max) |
| **Paiement prioritaire** | Si solde négatif, inviter à régler avant nouveau crédit |
| **Alerte visuelle** | Au POS, avertissement si solde négatif |

### 10.4 Types de transactions

| Type | Déclencheur | Impact | Qui |
|---|---|---|---|
| `credit` | Rechargement manuel | +X | Caissier/Admin |
| `overpayment` | Trop-perçu non rendu | +X | Caissier |
| `debt` | Impayé partiel/total | -X | Caissier |
| `payment` | Paiement via le solde | -X | Caissier |
| `refund` | Remboursement | +X | Caissier |
| `adjustment` | Correction manuelle | +/-X | Admin only |
| `pourboire` | Excédent laissé | 0 | Caissier |

### 10.5 Contraintes techniques

- Toute opération transactionnelle (ROLLBACK si échec)
- Chaque mouvement horodaté, lié à un utilisateur, **immuable**
- Solde courant stocké dans un champ dédié (pas recalculé depuis l'historique)
- Historique en écriture seule — même l'admin ne peut pas effacer

---

## 11. Impression thermique

### 11.1 Formats supportés

| Format | Largeur | Usage |
|---|---|---|
| **58mm** | 58mm (48mm imprimable) | Ticket de caisse standard, petit comptoir |
| **80mm** | 80mm (72mm imprimable) | Ticket détaillé, commande cuisine/bar |

### 11.2 Marques et protocoles

| Marque | Protocole | Testé |
|---|---|---|
| **Epson** (TM-T20, TM-T88) | USB + ESC/POS, réseau (TCP/IP) | — |
| **Star** (SP700, TSP100) | USB + StarPRNT | — |
| **Xprinter** | USB + ESC/POS (compatible Epson) | — |
| **Generic ESC/POS** | USB | — |

### 11.3 Stratégie d'impression

1. **Impression native (Tauri machine serveur)** : via Rust, envoi des commandes ESC/POS sur port USB/série. La lib `printpdf` ou `escpos-rs` génère le buffer binaire, écrit sur le périphérique USB.
2. **Impression réseau** : les postes clients (navigateur) peuvent imprimer sur une imprimante réseau via JavaScript `window.print()` pour le format A4, ou via une API Rust qui redirige vers l'imprimante USB du serveur.
3. **Afficheur client** : second écran ou petit afficheur LED qui répète le montant à payer (voir section 12).

### 11.4 Contenu du ticket

```
         MON BAR
    123 Rue Example, Ville
         5555 1234
  ============================
   Serveur: Jean
   Table: 5
   Date: 14/07/2026 20:32
  ----------------------------
  2x Bière Castel     2 000F
  1x Brochettes       3 500F
  3x Jus Bissap       2 500F
  ----------------------------
   TOTAL              8 000F
  ----------------------------
   Espèces           10 000F
   Rendu              2 000F
  ============================
   Merci de votre visite !
```

### 11.5 Contrainte technique

- L'imprimante est branchée sur la **machine serveur** (caissier)
- Les vendeurs impriment les tickets bar/kitchen sur une imprimante réseau ou une imprimante dédiée

---

## 12. Multi-écran & afficheur client

### 12.1 Concept

Dans un bar/restaurant, le client doit voir le montant à payer. Deux approches :

### 12.2 Écran miroir (afficheur client)

| Approche | Description |
|---|---|
| **Second moniteur** | La machine serveur a un second écran orienté vers le client. L'application affiche une vue dédiée : logo + montant + articles |
| **Route dédiée** | Route Nuxt `/afficheur` qui affiche une vue plein écran, sans interactions, avec les infos de la commande en cours |
| **Mise à jour temps réel** | WebSocket ou polling pour synchroniser l'affichage avec les actions du caissier |

### 12.3 Fonctionnalités de l'afficheur client

| Fonctionnalité | Description |
|---|---|
| **Vue plein écran** | Pas de barre d'adresse, pas de navigation |
| **Logo de l'enseigne** | Configurable dans les paramètres |
| **Montant total** | Grand caractère, visible de loin |
| **Détail des articles** | Liste des produits avec quantités |
| **Animation** | Lors du passage en *Closed*, message de remerciement |
| **Mode veille** | Si aucune commande active, affiche un écran de veille (logo + message) |

### 12.4 Mise en œuvre

```bash
# Sur la machine serveur, ouvrir dans un navigateur dédié
firefox --kiosk http://localhost:PORT/afficheur
# Ou utiliser un second window Tauri
```

---

## 13. Packaging multi-plateforme

### 13.1 Formats cibles

| Plateforme | Formats | Notes |
|---|---|---|
| **Linux** | `.rpm` (Fedora/RHEL), `.deb` (Debian/Ubuntu), `.AppImage` (universel) | Tauri supporte les 3 nativement |
| **macOS** | `.dmg` + `.app` bundle | Code signing requis pour distribution |
| **Windows** | `.msi` + `.exe` (NSIS) | Code signing optionnel |

### 13.2 Configuration Tauri

```json
// src-tauri/tauri.conf.json (extrait)
{
  "bundle": {
    "active": true,
    "icon": ["icons/icon.png", "icons/icon.ico", "icons/icon.icns"],
    "targets": ["deb", "rpm", "appimage", "dmg", "msi"],
    "linux": {
      "deb": { "depends": ["libsqlite3-0"] },
      "rpm": { "depends": ["libsqlite3"] },
      "appimage": { "bundleMediaFramework": true }
    }
  }
}
```

### 13.3 Dépendances système

| Plateforme | Paquets requis |
|---|---|
| **Linux (Debian/Ubuntu)** | `libwebkit2gtk-4.1-dev`, `libsqlite3-0`, `libgtk-3-dev`, `librsvg2-dev`, `libayatana-appindicator3-dev` |
| **Linux (Fedora)** | `webkit2gtk4.1-devel`, `libsqlite3x-devel`, `gtk3-devel`, `librsvg2-devel`, `libappindicator-gtk3-devel` |
| **macOS** | Xcode Command Line Tools |
| **Windows** | WebView2 (inclus dans Windows 10+), Visual Studio Build Tools |

### 13.4 Scripts de build

```bash
# Build pour la plateforme courante
bun run tauri:build

# Build ciblé (cross-compilation)
cargo tauri build --target x86_64-unknown-linux-gnu  # Linux
cargo tauri build --target x86_64-apple-darwin         # macOS Intel
cargo tauri build --target aarch64-apple-darwin        # macOS ARM
cargo tauri build --target x86_64-pc-windows-msvc      # Windows

# AppImage supplémentaire
cargo tauri build --bundles appimage
```

### 13.5 Bonnes pratiques packaging

- Tester le `.AppImage` sur plusieurs distros avant release
- Le `.deb` et `.rpm` gèrent les dépendances automatiquement
- `.AppImage` est idéal pour les tests / déploiement rapide
- Le "code signing" macOS est obligatoire pour éviter les alertes de sécurité
- Windows : signer le `.msi` avec un certificat (ou accepter l'avertissement SmartScreen en interne)

---

## 14. Configuration système détaillée

### 14.1 Paramètres modifiables

| Paramètre | Type | Défaut | Description |
|---|---|---|---|
| **Nom de l'enseigne** | string | `"Mon Bar"` | Affiché sur les tickets, l'afficheur client, le titre de la fenêtre |
| **Logo** | image (PNG) | — | Logo affiché sur les tickets et l'afficheur client |
| **Adresse** | string | — | Adresse du commerce (sur les tickets) |
| **Téléphone** | string | — | Contact (sur les tickets) |
| **Devise** | string | `"FCFA"` | Symbole monétaire |
| **TVA (%)** | decimal | `0` | Taux de TVA appliqué |
| **Plafond de dette client** | integer | `20000` | Solde négatif maximum autorisé (FCFA) |
| **Seuil alerte stock bas** | integer | `5` | Quantité minimale avant alerte |
| **Impression auto ticket** | bool | `true` | Imprimer automatiquement après validation |
| **Format ticket** | enum | `"80mm"` | `58mm` ou `80mm` |
| **Fusion horaires** | bool | `false` | Si vrai, les commandes de la veille non soldées apparaissent encore |
| **Langue** | enum | `"fr"` | `fr` ou `en` |

### 14.2 Stockage

- Les paramètres sont stockés dans la base de données (table `settings`)
- Le logo est stocké dans le filesystem (dossier `data/`)
- `@tauri-store/pinia` peut être utilisé pour le cache frontend

### 14.3 Ouverture de caisse par défaut

| Paramètre | Description |
|---|---|
| **Montant d'ouverture** | Montant en caisse en début de service (ex: 50 000 FCFA) |
| **Caissier par défaut** | Si un seul caissier, pré-sélectionné |

---

## 15. Pages de l'interface

### 15.1 Liste des écrans (routes Nuxt)

| Route | Écran | Rôle | Description |
|---|---|---|---|
| `/login` | Connexion | Tous | Email + mot de passe |
| `/` | Dashboard admin | Admin | Stats, alertes, activité récente |
| `/admin/employes` | Gestion employés | Admin | CRUD employés |
| `/admin/employes/:id` | Détail employé | Admin | Édition |
| `/admin/rapports` | Rapports | Admin | Ventes, employés, produits, export |
| `/admin/clients` | Clients | Admin | CRUD fiches clients |
| `/admin/clients/:id` | Détail client | Admin | Solde, historique |
| `/admin/parametres` | Paramètres | Admin | Configuration système |
| `/caissier` | Dashboard caissier | Caissier | Vue caisse, commandes à clôturer |
| `/caissier/commande/:id` | Validation commande | Caissier | Détail + encaissement |
| `/pos` | Point de vente | Caissier | Interface caisse plein écran |
| `/pos/attente` | Mise en attente | Caissier | Paniers suspendus |
| `/caissier/cloture` | Clôture de caisse | Caissier | Bilan fin de service |
| `/caissier/historique` | Clôtures passées | Caissier | Consultation |
| `/vendeur` | Dashboard vendeur | Vendeur | Stats, tables |
| `/vendeur/commande` | Nouvelle commande | Vendeur | Sélection table + produits |
| `/vendeur/commande/:id` | Détail commande | Vendeur | Articles, service, demande clôture |
| `/stock` | Dashboard stock | Stock | Alertes, vue stocks |
| `/stock/produits` | Produits | Stock | CRUD |
| `/stock/produits/:id` | Détail produit | Stock | Édition |
| `/stock/categories` | Catégories | Stock | CRUD hiérarchique |
| `/stock/mouvements` | Mouvements | Stock | Entrées/sorties |
| `/stock/fournisseurs` | Fournisseurs | Stock | CRUD |
| `/afficheur` | Afficheur client | Public (lecture) | Second écran client (route ouverte) |

### 15.2 Layouts

| Layout | Description |
|---|---|
| `auth` | Page de login (centrée, minimaliste) |
| `admin` | Sidebar + header avec navigation admin |
| `caissier` | Plein écran, barre supérieure avec état caisse |
| `vendeur` | Mobile-first, navigation basse (tablette) |
| `stock` | Sidebar légère |
| `pos` | Plein écran, aucune chrome, seulement le POS |
| `afficheur` | Plein écran, aucune interaction, données en lecture |

---

## 16. Sauvegarde & Restauration

### 16.1 Stratégie

La base de données est un fichier unique (`mboacaisse.db`). La sauvegarde = copie de ce fichier.

| Mécanisme | Description |
|---|---|
| **Automatique (programmée)** | Copie du fichier `.db` toutes les X minutes/heures |
| **Manuelle** | Bouton dans les paramètres admin |
| **Avant mise à jour** | Sauvegarde automatique avant installation d'une mise à jour |
| **Rotation** | Garder les N dernières sauvegardes (ex: 30 jours) |

### 16.2 Implémentation

```rust
// Côté Rust, tâche tokio périodique
async fn backup_task(db_path: &str, backup_dir: &str) {
    let mut interval = tokio::time::interval(Duration::from_secs(3600));
    loop {
        interval.tick().await;
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let dest = format!("{backup_dir}/mboacaisse_{timestamp}.db");
        tokio::fs::copy(db_path, &dest).await?;
        // Nettoyage : garder les 30 plus récents
        cleanup_old_backups(backup_dir, 30).await?;
    }
}
```

### 16.3 Restauration

| Fonctionnalité | Description |
|---|---|
| **Depuis l'interface admin** | Parcourir les sauvegardes, sélectionner une date, restaurer |
| **Arrêt requis** | L'application doit être redémarrée après restauration |
| **Confirmation** | Double confirmation avant de remplacer la base active |
| **Export/Import** | Télécharger une sauvegarde, l'importer sur une autre machine |

### 16.4 Contenu sauvegardé

- Fichier `mboacaisse.db` (données complètes)
- Dossier `uploads/` (images produits) si présent

### 16.5 Emplacement

| Plateforme | Chemin par défaut |
|---|---|
| Linux | `~/.local/share/detroi-chill/backups/` |
| macOS | `~/Library/Application Support/detroi-chill/backups/` |
| Windows | `%APPDATA%/detroi-chill/backups/` |

---

## 17. Migrations de schéma

### 17.1 Problème

libsql/SQLite n'a pas de système de migrations intégré. Il faut le gérer nous-mêmes.

### 17.2 Solution : table `_migrations`

```sql
CREATE TABLE IF NOT EXISTS _migrations (
    version INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    applied_at TEXT NOT NULL DEFAULT (datetime('now')),
    checksum TEXT NOT NULL
);
```

### 17.3 Principe

```rust
// Au démarrage de l'application
let migrations = [
    (1, "001_create_users", "CREATE TABLE users (...)"),
    (2, "002_create_products", "CREATE TABLE products (...)"),
    (3, "003_add_purchase_price", "ALTER TABLE products ADD COLUMN purchase_price DECIMAL..."),
    // ...
];

for (version, name, sql) in migrations {
    let already = db.query(
        "SELECT 1 FROM _migrations WHERE version = ?", params![version]
    )?;
    if already.is_empty() {
        db.execute(sql, ())?;
        db.execute(
            "INSERT INTO _migrations (version, name, checksum) VALUES (?, ?, ?)",
            params![version, name, hash(sql)]
        )?;
    }
}
```

### 17.4 Règles

- Les migrations sont exécutées dans l'ordre, une seule fois
- Le checksum permet de détecter des modifications non authorisées
- Impossible de rollback (on migre toujours forward)
- Si une migration échoue, l'application ne démarre pas (intégrité)
- Les migrations sont versionnées dans le code Rust (pas de fichiers SQL externes)

---

## 18. Serveur de secours

### 18.1 Problème

La machine serveur tombe en panne → plus de caisse, plus de commandes.

### 18.2 Solutions envisagées

| Solution | Avantages | Inconvénients |
|---|---|---|
| **Redondance BDD** | Script de backup programmé, copie sur NAS/cloud | Nécessite un accès réseau |
| **Machine de réserve** | Second PC avec Tauri préinstallé, pointe vers le backup | Configuration manuelle |
| **Aucune** | Risque acceptable pour un petit commerce | Temps mort |

### 18.3 Recommandation pour V1

Pas de haute disponibilité. Un message s'affiche sur les postes clients : *"Serveur indisponible, contactez le gérant."*

### 18.4 Pour V2+

| Solution | Description |
|---|---|
| **Backup automatique réseau** | Sauvegarde du fichier `.db` sur un partage NAS/Samba |
| **Script de réinstallation rapide** | Script bash/batch qui télécharge la dernière version du binaire + restaure le backup |
| **Mode dégradé** | Si la BDD est inaccessible, le Tauri local peut utiliser une base de cache locale (lecture seule du catalogue produits) |

---

## 19. Modules transverses

| Fonctionnalité | Description |
|---|---|
| **Journal d'activité** | Traçage de toutes les actions : qui, quoi, quand, sur quoi |
| **Recherche globale** | Barre de recherche unique (produits, clients, commandes) |
| **Exports** | CSV/PDF pour chaque liste (produits, ventes, inventaire, clients) |
| **Factures** | Génération PDF pour les clients qui en demandent |
| **Données de démo** | Jeu de données réaliste (catégories, produits, commandes) |
| **Mode serveur** | Affichage de l'URL/IP à partager sur le réseau au démarrage |

---

## 20. Bonnes pratiques

### 20.1 Rust

| Pratique | Description |
|---|---|
| **Panneaux** | Utiliser `Result<T, E>` partout, pas de `unwrap()` en production |
| **Logging** | `tracing` crate pour les logs structurés (niveaux : error, warn, info, debug, trace) |
| **Tests** | Tests unitaires pour la logique métier (solde, échanges, commandes) |
| **Clippy** | `cargo clippy` avant chaque commit |
| **Formatage** | `cargo fmt` (rustfmt) |
| **Modules** | Organisation par domaine (`orders/`, `products/`, `payments/`) pas par couche (`models/`, `controllers/`) |
| **Async** | Utiliser `tokio` et privilégier `async fn` partout |
| **Transactions** | Toute opération qui touche plusieurs tables (commande + stock + solde) dans une transaction SQL |

### 20.2 Tauri

| Pratique | Description |
|---|---|
| **Commandes** | Une commande Tauri = une action métier, pas une requête SQL |
| **Permissions** | Déclarer les permissions nécessaires dans `capabilities/main.json` |
| **Sécurité** | Ne pas exposer de commandes sensibles aux clients web (vérifier l'origine) |
| **Build** | `bun run tauri:build` en CI pour générer les artefacts |

### 20.3 Nuxt / Vue

| Pratique | Description |
|---|---|
| **Composables** | Logique réutilisable dans `composables/` |
| **Pinia stores** | Un store par domaine (`useOrderStore`, `useProductStore`, `useAuthStore`) |
| **NuxtUI** | Utiliser les composants NuxtUI (`UButton`, `UInput`, `UTable`, etc.) |
| **Page state** | Utiliser `useAsyncData` pour le chargement initial, Pinia pour l'état partagé |
| **Validation** | Valider les formulaires côté client avant envoi à l'API |
| **Responsive** | Tester sur écran 1920px (caissier) et 1024px/768px (tablettes vendeurs) |

### 20.4 Base de données

| Pratique | Description |
|---|---|
| **Index** | Indexer les colonnes de recherche (`products.name`, `orders.status`, `orders.created_at`) |
| **VACUUM** | Exécuter `VACUUM` périodiquement pour éviter la fragmentation |
| **WAL mode** | Activer `PRAGMA journal_mode=WAL` pour les lectures concurrentes |
| **Foreign keys** | Activer `PRAGMA foreign_keys=ON` |
| **Backup régulier** | Copie du fichier .db (voir section 16) |

### 20.5 CI/CD

| Pratique | Description |
|---|---|
| **Lint** | `cargo clippy` + `eslint` (Nuxt) dans la CI |
| **Tests** | Tests Rust + tests Nuxt (vitest) |
| **Build** | Build automatique des artefacts (deb, rpm, AppImage, dmg, msi) via GitHub Actions |
| **Release** | Publication automatique des artefacts dans les GitHub Releases |
| **Updater** | Le plugin `updater` pointe vers les GitHub Releases |

### 20.6 Sécurité

| Pratique | Description |
|---|---|
| **Réseau local seulement** | Le serveur HTTP Tauri doit écouter sur `0.0.0.0` mais le pare-feu doit limiter l'accès au LAN |
| **Mots de passe** | Hashés avec argon2 (pas bcrypt en Rust) |
| **Session** | Token JWT ou session ID stocké en cookie HTTP-only |
| **Validation** | Toute entrée utilisateur validée côté serveur (Rust) même si déjà validée côté client |
| **Backup** | Les fichiers de backup ne doivent pas être accessibles via le serveur HTTP |

---

## 21. Roadmap

### 21.1 V1 — socle minimal

- Authentification (4 rôles)
- CRUD produits et catégories
- Gestion des stocks (entrée fournisseur)
- Gestion des tables
- Prise de commande (vendeur)
- POS de base — encaissement espèces (caissier)
- Impression ticket de caisse (58mm/80mm)
- Dashboard admin (stats du jour)
- Mode serveur LAN
- Packaging Linux (deb, rpm, AppImage)

### 21.2 V2 — finition métier

- Solde client / Porte-monnaie
- Paiement mobile money (OM/MTN)
- Gestion des clients (fiches)
- Échanges et remboursements
- Clôture de caisse (ouverture, clôture, écart)
- Rapports ventes (par produit, employé, période)
- Plan des tables en temps réel
- Réductions et notes sur les commandes
- Mise en attente de panier
- Afficheur client (second écran)

### 21.3 V3 — administration & robustesse

- Gestion des fournisseurs
- Seuils d'alerte stock
- Export CSV/PDF des rapports
- Journal d'activité complet
- Factures PDF
- Code-barres (saisie manuelle + scan)
- Sauvegarde automatique de la base
- Paramètres système (nom, TVA, seuils)
- Packaging macOS (dmg) + Windows (msi)
- Système de migrations BDD

### 21.4 V4 — améliorations

- Multi-magasins
- Permissions fines par action
- Mode dégradé / serveur de secours
- Interface en anglais + français
- Page d'accueil configurable (logo, thème)
- API REST pour intégration tierce
- Auto-updater (plugin Tauri)
- CI/CD complet avec GitHub Actions
