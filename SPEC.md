# SPEC — MboaCaisse v0.1

Machine-actionable spec. One pass, no rollbacks.

---

## §G — Goal

MboaCaisse : caisse et gestion pour bars/restos africains. LAN sans internet. 4 rôles (vendeur, caissier, stock, gérant). Un binaire Tauri sur 1 PC, navigateur sur les autres. Remplacer carnet papier + calcul mental + disputes de fin de service.

Déployé chez Jean à Douala. FCFA. Open-source.

---

## §C — Constraints

| # | Constraint | Detail |
|---|-----------|-------|
| C1 | **Offline-first** | Zéro internet. LAN/WiFi local uniquement. Pas d'appel API externe. |
| C2 | **Multi-postes** | Serveur HTTP Axum embarqué dans Tauri. Fenêtre native = caissier/admin. Clients navigateur = vendeurs/stock. |
| C3 | **Dev locale** | Dev solo Herold, sessions IA. Fedora/Linux. `bun` obligatoire. |
| C4 | **Stack** | Tauri 2 + Nuxt 4 + NuxtUI 4 + Axum + SQLite + Vue 3. SSR désactivé. |
| C5 | **DB** | SQLite fichier unique (`mboacaisse.db`). WAL mode. foreign_keys ON. Backup par copie fichier. |
| C6 | **FCFA** | Prix en entier. Pas de décimales en base. |
| C7 | **Rust natif** | Pas de lib Python/C interop. ESC/POS via pur Rust ou window.print(). |
| C8 | **Expédition** | Aucune deadline. Heures perdues = features en moins. Phase gated. |
| C9 | **Réduit** | Moins de features mieux faites > toutes les features à moitié. |

---

## §I — Interfaces

### I.1 Schéma BDD — 11 tables MVP

```sql
-- users
CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    role TEXT NOT NULL CHECK(role IN ('admin','cashier','waiter','stock_manager')),
    active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- categories
CREATE TABLE categories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    color TEXT NOT NULL DEFAULT '#94a3b8',
    parent_id INTEGER REFERENCES categories(id),
    position INTEGER NOT NULL DEFAULT 0,
    active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- products
CREATE TABLE products (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    description TEXT DEFAULT '',
    price INTEGER NOT NULL,  -- FCFA, entier
    purchase_price INTEGER,  -- FCFA optionnel
    stock INTEGER NOT NULL DEFAULT 0,
    alert_threshold INTEGER NOT NULL DEFAULT 5,
    category_id INTEGER NOT NULL REFERENCES categories(id),
    barcode TEXT,
    active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- tables
CREATE TABLE tables (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    number INTEGER NOT NULL UNIQUE,
    capacity INTEGER NOT NULL DEFAULT 4,
    active INTEGER NOT NULL DEFAULT 1
);

-- orders
CREATE TABLE orders (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    table_id INTEGER REFERENCES tables(id),   -- nullable (takeaway)
    user_id INTEGER NOT NULL REFERENCES users(id),
    customer_id INTEGER REFERENCES customers(id),  -- nullable
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK(status IN ('pending','served','closed','archived')),
    notes TEXT DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- order_items
CREATE TABLE order_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    order_id INTEGER NOT NULL REFERENCES orders(id),
    product_id INTEGER NOT NULL REFERENCES products(id),
    quantity INTEGER NOT NULL CHECK(quantity > 0),
    unit_price INTEGER NOT NULL,  -- prix au moment de la commande
    notes TEXT DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- payments
CREATE TABLE payments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    order_id INTEGER NOT NULL REFERENCES orders(id),
    method TEXT NOT NULL CHECK(method IN ('cash','mobile_money','wallet','card','mixed')),
    amount INTEGER NOT NULL CHECK(amount > 0),
    cash_received INTEGER,  -- montant espèces donné par le client
    reference TEXT,          -- numéro transaction mobile money
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- customers
CREATE TABLE customers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    phone TEXT,
    email TEXT,
    balance INTEGER NOT NULL DEFAULT 0,  -- solde courant (positif ou négatif)
    debt_limit INTEGER NOT NULL DEFAULT -20000,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- balance_transactions
CREATE TABLE balance_transactions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    customer_id INTEGER NOT NULL REFERENCES customers(id),
    type TEXT NOT NULL CHECK(type IN ('credit','payment','debt','overpayment','refund','adjustment','pourboire')),
    amount INTEGER NOT NULL,
    balance_after INTEGER NOT NULL,
    order_id INTEGER REFERENCES orders(id),
    user_id INTEGER NOT NULL REFERENCES users(id),
    notes TEXT DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
) WITHOUT ROWID;  -- historique immutable, pas de mise à jour

-- cash_register_sessions
CREATE TABLE cash_register_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL REFERENCES users(id),
    opening_amount INTEGER NOT NULL DEFAULT 0,
    closing_amount INTEGER,
    status TEXT NOT NULL DEFAULT 'open' CHECK(status IN ('open','closed','interrupted')),
    closed_at TEXT,
    notes TEXT DEFAULT '',
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- settings
CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- _migrations (interne)
CREATE TABLE IF NOT EXISTS _migrations (
    version INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    applied_at TEXT NOT NULL DEFAULT (datetime('now')),
    checksum TEXT NOT NULL
);
```

### I.2 Routes API (Axum)

| Method | Path | Rôle | Description |
|--------|------|------|-------------|
| POST   | `/api/auth/login` | * | Login email+password → token |
| POST   | `/api/auth/logout` | * | Invalider token |
| GET    | `/api/auth/me` | * | Profil courant |
| GET    | `/api/products` | * | Liste produits actifs |
| POST   | `/api/products` | admin,stock | Créer produit |
| PUT    | `/api/products/:id` | admin,stock | Modifier produit |
| GET    | `/api/categories` | * | Liste catégories |
| POST   | `/api/categories` | admin,stock | Créer catégorie |
| GET    | `/api/tables` | * | Liste tables + statut dérivé |
| POST   | `/api/orders` | waiter | Créer commande |
| GET    | `/api/orders` | * | Liste commandes (filtrée par rôle) |
| GET    | `/api/orders/:id` | * | Détail commande |
| PUT    | `/api/orders/:id/status` | waiter,cashier | Changer statut |
| POST   | `/api/orders/:id/payments` | cashier | Ajouter paiement |
| GET    | `/api/customers` | admin,cashier | Liste clients |
| POST   | `/api/customers` | admin,cashier,waiter | Créer client |
| GET    | `/api/customers/:id/balance` | admin,cashier | Solde + historique |
| POST   | `/api/customers/:id/balance` | admin,cashier | Créditer/débiter manuel |
| POST   | `/api/sessions/open` | cashier | Ouvrir caisse |
| POST   | `/api/sessions/close` | cashier | Fermer caisse |
| GET    | `/api/sessions/current` | cashier | Session active |
| GET    | `/api/dashboard` | admin | Stats jour |
| GET    | `/api/settings` | admin | Lire paramètres |
| PUT    | `/api/settings` | admin | Écrire paramètres |
| GET    | `/api/reports/sales` | admin | Rapport ventes |
| GET    | `/api/reports/employees` | admin | Ventes par employé |
| POST   | `/api/stock/entries` | admin,stock | Entrée stock |
| POST   | `/api/stock/adjustments` | admin,stock | Ajustement/perte |

### I.3 Pages Nuxt (frontend)

| Route | Layout | Rôle | Page |
|-------|--------|------|------|
| `/login` | `auth` | * | Login |
| `/` | `admin` | admin | Dashboard |
| `/admin/employes` | `admin` | admin | CRUD employés |
| `/admin/clients` | `admin` | admin | CRUD clients |
| `/admin/clients/:id` | `admin` | admin | Détail client |
| `/admin/parametres` | `admin` | admin | Paramètres système |
| `/caissier` | `caissier` | cashier | Dashboard caisse |
| `/caissier/commande/:id` | `caissier` | cashier | Valider + encaisser |
| `/caissier/cloture` | `caissier` | cashier | Clôture caisse |
| `/pos` | `pos` | cashier | POS direct (Phase 2) |
| `/vendeur` | `vendeur` | waiter | Plan tables + stats |
| `/vendeur/commande` | `vendeur` | waiter | Nouvelle commande |
| `/vendeur/commande/:id` | `vendeur` | waiter | Détail commande |
| `/stock` | `stock` | stock_manager | Dashboard stock |
| `/stock/produits` | `stock` | stock_manager | CRUD produits |
| `/stock/produits/:id` | `stock` | stock_manager | Détail produit |
| `/stock/mouvements` | `stock` | stock_manager | Entrées/sorties |
| `/afficheur` | `afficheur` | public | Écran client (Phase 3) |

### I.4 Layouts Nuxt

| Layout | Design |
|--------|--------|
| `auth` | Centré, minimaliste |
| `admin` | Sidebar + header |
| `caissier` | Plein écran, barre état caisse en haut |
| `waiter` | Mobile-first, navigation basse |
| `stock` | Sidebar légère |
| `pos` | Plein écran, zéro chrome |
| `afficheur` | Plein écran, lecture seule |

---

## §V — Invariants

| # | Invariant | Type |
|---|-----------|------|
| V1 | **Séparation stricte vendeur/caissier** : vendeur ne manipule jamais d'argent. Caissier seul à valider les paiements. | Workflow |
| V2 | **Cycle commande** : `pending → served → closed → archived`. POS direct passe `pending → closed`. | Workflow |
| V3 | **Commande toujours soldée** : un paiement insuffisant crée une dette (solde négatif). Jamais de commande bloquée en attente. | Métier |
| V4 | **Historique immutable** : `balance_transactions` écriture seule. Pas de UPDATE, pas de DELETE. Même l'admin ne peut pas effacer. | Base |
| V5 | **Solde courant stocké** : `customers.balance` = valeur live. Pas recalculé depuis l'historique à chaque lecture. | Base |
| V6 | **Statut table dérivé** : pas de colonne `table.status`. Calculé depuis les commandes actives (pending → orange, served → bleu). | Base |
| V7 | **Validation serveur** : toute entrée validée côté Rust même si déjà validée côté client. Pas de confiance frontend. | Architecture |
| V8 | **Transactions SQL** : toute opération multi-table (commande + stock + solde) dans une transaction. ROLLBACK si échec. | Architecture |
| V9 | **Nullable design** : `orders.table_id` nullable (takeaway). `orders.customer_id` nullable (vente sans client). Dès V1, pas de migration V2. | Base |
| V10 | **Token session** : authentification via token (pas de JWT complexe en V1). Stocké en cookie HTTP-only. | Sécurité |
| V11 | **Table virtuelle "0 — Emporter"** : seed dans `tables` pour takeaway. | Base |
| V12 | **Prix FCFA entier** : pas de décimales dans products.price, payments.amount, balance. | Base |
| V13 | **Migrations forward only** : pas de rollback. Checksum vérifié. Si échec, application ne démarre pas. | Architecture |
| V14 | **Zéro lib externe non Rust** : pas de dépendance Python/C. window.print() est le seul fallback accepté. | Architecture |
| V15 | **Session caisse interrompue** : si PC éteint sans clôture, session marquée "interrupted". Données intactes. Écart non calculable mais historique préservé. | Base |

---

## §T — Tasks

### Phase 0 — Socle Technique (bloc 1)

| # | Tâche | Fichiers | Dépend |
|---|-------|----------|--------|
| T0.1 | Init Rust workspace modules : `db`, `auth`, `api`, `migrations` dans `src-tauri/src/` | `src-tauri/src/db/mod.rs`, `src-tauri/src/auth/mod.rs`, `src-tauri/src/api/mod.rs`, `src-tauri/src/migrations/mod.rs` | — |
| T0.2 | Connexion SQLite : ouvrir `mboacaisse.db`, PRAGMA WAL + foreign_keys ON | `src-tauri/src/db/mod.rs` | T0.1 |
| T0.3 | Serveur HTTP Axum embarqué : plugin Tauri, écoute `0.0.0.0:PORT`, afficher URL au démarrage | `src-tauri/src/api/mod.rs`, `src-tauri/src/lib.rs` | T0.2 |
| T0.4 | Système de migrations : table `_migrations`, SQL dans Rust avec checksum, exécution ordonnée au démarrage | `src-tauri/src/migrations/mod.rs` | T0.2 |
| T0.5 | Auth : table `users`, argon2 hash, endpoint login/logout, session token, middleware Axum | `src-tauri/src/auth/mod.rs` | T0.3, T0.4 |
| T0.6 | Seed data : script charge catégories, produits, tables (dont "0 — Emporter"), utilisateur demo au premier lancement | `src-tauri/src/db/seed.rs` | T0.4 |
| T0.7 | Tracing : crate `tracing`, logs structurés fichier + stdout, niveaux error/warn/info/debug | `src-tauri/src/lib.rs` | T0.1 |
| T0.8 | LAN mode : détecter IP locale, afficher URL `http://IP:PORT` dans la fenêtre native au démarrage | `src-tauri/src/api/mod.rs` | T0.3 |

### Phase 1 — Socle Métier (bloc 2)

| # | Tâche | Fichiers | Dépend |
|---|-------|----------|--------|
| T1.1 | CRUD produits : API list/create/update + page admin avec formulaire | `src-tauri/src/api/products.rs`, `app/pages/stock/produits.vue`, `app/pages/stock/produits/[id].vue` | T0.5 |
| T1.2 | CRUD catégories : API + page admin hiérarchique | `src-tauri/src/api/categories.rs`, `app/pages/stock/categories.vue` | T0.5 |
| T1.3 | CRUD tables : API + page admin, table virtuelle "0 — Emporter" visible | `src-tauri/src/api/tables.rs`, `app/pages/stock/tables.vue` | T0.5 |
| T1.4 | Layouts Nuxt : auth, admin (sidebar), waiter (mobile-bottom), cashier (fullscreen), stock (sidebar-light) | `app/layouts/` (5 fichiers) | T0.5 |
| T1.5 | Page login : email + password, redirect selon rôle | `app/pages/login.vue` | T0.5, T1.4 |
| T1.6 | Middleware auth Nuxt : protéger routes selon rôle, redirect si non connecté | `app/middleware/auth.ts` | T0.5 |

### Phase 2 — MVP Bar (bloc 3)

| # | Tâche | Fichiers | Dépend |
|---|-------|----------|--------|
| T2.1 | Plan des tables vendeur : grille carte avec code couleur vert/orange/bleu, statut dérivé | `app/pages/vendeur/index.vue` | T1.3, T1.4 |
| T2.2 | Menu contextuel table : nouvelle commande, voir détail | Composant `TableCard` | T2.1 |
| T2.3 | Écran prise commande vendeur : grille produits 3 colonnes 55% haut, panier 45% bas fixe, onglets catégories scroll horizontal | `app/pages/vendeur/commande.vue` | T1.1, T1.2 |
| T2.4 | Panier swipe : gauche = supprimer + toast annuler 3s, droite = doubler quantité | `app/components/pos/CartItem.vue` | T2.3 |
| T2.5 | Envoi commande : loading 300ms + vibrate 50ms, pas de modal | `app/composables/useOrder.ts` | T2.3 |
| T2.6 | API cycle commande : create, update status (pending→served→closed), get list | `src-tauri/src/api/orders.rs` | T0.3, T0.5 |
| T2.7 | Dashboard vendeur : stats jour, nb commandes, tables occupées | `app/pages/vendeur/index.vue` | T2.1 |
| T2.8 | Écran caissier : liste commandes "À clôturer" (status=served), détail + encaissement | `app/pages/caissier/index.vue`, `app/pages/caissier/commande/[id].vue` | T2.6 |
| T2.9 | Paiement espèces : saisie montant reçu, calcul rendu auto | `app/components/pos/PaymentForm.vue` | T2.8 |
| T2.10 | Paiement mixte : sélecteur additif (Espèces + Solde + MM), total restant live | `app/components/pos/MixedPayment.vue` | T2.9 |
| T2.11 | Pas de monnaie : radio "Créditer solde client | Pourboire" | `app/components/pos/NoChangeModal.vue` | T2.9 |
| T2.12 | API paiements : create payment, lier à order, mettre à jour customer.balance | `src-tauri/src/api/payments.rs` | T0.3, T2.6 |
| T2.13 | Impression ticket : window.print() PDF avec template ticket (enseigne, articles, total, moyens, rendu, message) | `app/composables/useReceipt.ts` | T2.8 |
| T2.14 | Ouverture de caisse : modal montant départ, POST /api/sessions/open | `app/pages/caissier/index.vue` | T1.5 |
| T2.15 | Clôture de caisse : total par moyen, nb transactions, écart auto, seuil + motif obligatoire si dépassé | `app/pages/caissier/cloture.vue`, `src-tauri/src/api/sessions.rs` | T2.14 |
| T2.16 | Dashboard admin stats jour : nb commandes, CA, écarts, top produits | `app/pages/index.vue` | T2.6, T2.12 |
| T2.17 | Responsive tablette : testé 1024×768 (portrait tablette) et 1920×1080 (desktop caissier) | Tous les composants | T2.3 |

### Phase 3 — Résilience (bloc 4)

| # | Tâche | Fichiers | Dépend |
|---|-------|----------|--------|
| T3.1 | Backup auto : tâche tokio périodique (1h), copie .db, rotation 30 jours | `src-tauri/src/db/backup.rs` | T0.2 |
| T3.2 | Solde client V1 : balance_transactions immutables, debt_limit, alert visuelle si négatif | `src-tauri/src/api/customers.rs` | T2.12 |
| T3.3 | CRUD clients page admin : nom, téléphone, email, historique solde | `app/pages/admin/clients.vue`, `app/pages/admin/clients/[id].vue` | T3.2 |
| T3.4 | Notifications nouvelles commandes : badge 🆕 30s + notification OS pour caissier | `src-tauri/src/api/notifications.rs`, `app/composables/useNotification.ts` | T2.6 |

### Phase 4 — Métier (bloc 5)

| # | Tâche | Fichiers | Dépend |
|---|-------|----------|--------|
| T4.1 | Mobile money : paiement OM/MTN, champ référence transaction | `app/components/pos/MobileMoneyForm.vue` | T2.12 |
| T4.2 | Échange / retour : swap produit, remboursement espèces ou wallet | `src-tauri/src/api/exchanges.rs` | T2.12, T3.2 |
| T4.3 | Rapports ventes : par produit, employé, période | `src-tauri/src/api/reports.rs`, `app/pages/admin/rapports.vue` | T2.16 |
| T4.4 | Split commande : répartir articles en sous-commandes | `src-tauri/src/api/orders.rs` | T2.6 |
| T4.5 | Écran client : route `/afficheur`, vue kiosque, logo + montant + articles | `app/pages/afficheur.vue` | T2.8 |
| T4.6 | Auth rôles UI : page gestion employés, admin peut créer/modifier/désactiver | `app/pages/admin/employes.vue`, `src-tauri/src/api/users.rs` | T0.5 |

### Phase 5 — Administration (bloc 6)

| # | Tâche | Fichiers | Dépend |
|---|-------|----------|--------|
| T5.1 | CRUD fournisseurs : liste, historique achats | `src-tauri/src/api/suppliers.rs`, `app/pages/stock/fournisseurs.vue` | T3.1 |
| T5.2 | Alertes stock : seuil configurable par produit, warning dashboard | `src-tauri/src/api/stock.rs` | T3.1 |
| T5.3 | Export CSV/PDF toutes listes : produits, commandes, clients, rapports | `src-tauri/src/api/exports.rs` | T4.3 |
| T5.4 | Journal d'activité : qui a fait quoi, horodaté, immutable | `src-tauri/src/api/activity.rs` | T0.5 |
| T5.5 | Facture PDF : génération printpdf pour clients qui demandent | `src-tauri/src/api/invoices.rs` | T0.3 |
| T5.6 | Paramètres système : nom boutique, logo, adresse, téléphone, devise, TVA, seuils | `app/pages/admin/parametres.vue` | T0.3 |
| T5.7 | ESC/POS natif Rust : buffer binaire pour imprimantes 58/80mm | `src-tauri/src/printing/escpos.rs` | T2.13 |
| T5.8 | Ticket cuisine ESC/POS : nom serveur + table double hauteur, notes indentées, pas de prix | `src-tauri/src/printing/kitchen.rs` | T5.7 |
| T5.9 | Plan de salle graphique : drag & drop tables, layout visuel custom | `app/components/pos/TablePlan.vue` | T2.1 |
| T5.10 | POS tactile plein écran : catalogue 4-5 colonnes, catégories verticales, scan code-barres focus auto | `app/pages/pos/index.vue` | T2.8 |
| T5.11 | Packaging : deb, rpm, AppImage, dmg, msi via tauri.conf.json | `src-tauri/tauri.conf.json` | — |
| T5.12 | CI/CD : GitHub Actions lint + test + build + release | `.github/workflows/` | T5.11 |

---

## §B — Bugs / Backprop

_Vide. Rempli automatiquement par backprop lors des échecs de build ou de test._

---

## Notes de phase

- **Phase 0-2** = V1 livrable ("le bar tourne"). Blocs 1-3.
- **Phase 3-4** = V2 métier. Blocs 4-5.
- **Phase 5** = administration & premium. Bloc 6.
- Chaque phase est gated : on ne commence pas la phase N+1 avant d'avoir livré la phase N.
- Gate qualitatif Phase 2 : un vrai vendeur prend une commande sans explication. Si l'UI échoue, on itère avant de continuer.
- Pas de deadline. Priorité : workflow tient > UI jolie.
