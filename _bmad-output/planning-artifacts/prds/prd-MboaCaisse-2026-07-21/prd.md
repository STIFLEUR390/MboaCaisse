---
title: "PRD: MboaCaisse"
status: draft
created: 2026-07-21
updated: 2026-07-21
---

# PRD: MboaCaisse

## 0. Document Purpose

PRD produit pour guider le développement alpha de MboaCaisse — serveur de caisse et gestion pour bars/restos/épiceries en Afrique francophone. Destiné à l'équipe de développement. Suffisamment structuré pour qu'un alpha client ou un grossiste en évaluation comprenne le périmètre.

Ce PRD s'appuie sur le Product Brief, la PRFAQ, le brainstorming expansion, l'architecture technique et le FEATURES.md existants. Les décisions de conception technique (choix de librairies Rust, format de signature, organisation du monorepo) vivent dans ces documents — ce PRD ne les répète pas.

Glossaire en §3 : les termes définis y sont utilisés sans synonyme dans tout le document.

## 1. Vision

MboaCaisse est un serveur de caisse et de gestion qui transforme n'importe quel PC en serveur POS local, accessible depuis tous les écrans de l'établissement (caissier, cuisine, serveurs, clients) via le navigateur — sans Internet, sans abonnement, sans terminal importé à 500K FCFA.

Le wallet client — identifié par numéro de téléphone, alimenté par Mobile Money ou espèces, avec un ledger append-only qui garantit l'intégrité comptable — est le noyau qui distingue le produit. Pas de cartes de fidélité, pas d'app à installer, pas de solde qui peut être trafiqué. Le cashback automatique et le parrainage transforment chaque client en ambassadeur.

Un seul binaire, 6 bundles métier activés par licence Ed25519 — de l'épicerie de quartier (Mboa Cash) au supermarché (Mboa Market). Le même code, la même API, des flags qui déverrouillent les fonctionnalités.

La promesse : un établissement peut fonctionner 30 jours sans jamais toucher à Internet — encaissement, impression, rapports, fidélité, tout est local. Si le PC tombe, on restaure le backup sur un autre PC en 15 minutes.

## 2. Target User

### 2.1 Jobs To Be Done

**Établissement — Propriétaire**
- Savoir exactement combien j'ai gagné aujourd'hui, sans attendre la clôture manuelle
- Arrêter de perdre de l'argent par des caissiers non tracés
- Accepter Orange Money/MTN MoMo sans noter les transactions sur un bout de papier
- Fidéliser mes clients sans carte, sans app, sans effort
- Savoir quand un produit est en rupture avant que le client le demande

**Établissement — Caissier**
- Encaisser rapidement (espèces, wallet client, Mobile Money)
- Imprimer un ticket en un clic, sans popup navigateur

**Établissement — Cuisine**
- Recevoir les commandes immédiatement sur un écran, sans crier

**Client (acheteur)**
- Payer avec mon téléphone (Mobile Money), pas besoin d'espèces
- Voir mon solde fidélité et mes crédits sans carte
- Commander depuis ma table via QR code

### 2.2 Non-Users (v1)

- Grands comptes / chaînes multi-sites (le sync groupe est P2)
- Commerçants sans PC (le produit nécessite un serveur Windows ou Linux)
- Marchés non-francophones (UI française en V1)

### 2.3 Key User Journeys

**UJ-1. Patrick lance sa caisse le matin.**
Patrick allume son PC, MboaCaisse démarre automatiquement (autostart). La fenêtre native affiche l'écran caisse. Les clients du WiFi local peuvent taper `http://mboacaisse.local` sur leur téléphone. Patrick voit les ventes d'hier sur le dashboard en un coup d'œil. Temps total : 30 secondes du boot au premier encaissement.

**UJ-2. Fatima paie avec son wallet.**
Fatima donne son numéro de téléphone à l'inscription. Le caissier sélectionne ses articles (2 bières, 1 brochette). Fatima dit "je paie avec mon wallet". Le système vérifie le solde (7000 FCFA), déduit le montant (3500 FCFA), alloue 175 FCFA de cashback (5%). Le ticket s'imprime. La commande part à la cuisine. Temps total : 45 secondes.

**UJ-3. Jean scanne le QR code de sa table.**
Jean s'assoit, scanne le QR code plastifié sur la table. Le menu s'affiche sur son téléphone. Il sélectionne 2 bières et un planteur. Il entre son téléphone pour s'identifier. La commande part directement à la cuisine. Pas d'app téléchargée. Pas d'attente de serveur.

**UJ-4. Paul consulte ses ventes depuis chez lui.**
Paul est chez lui, il ouvre `http://mboacaisse.local:PORT` depuis son téléphone sur le WiFi du quartier (le routeur porte chez lui aussi [ASSUMPTION : même LAN]). Il voit le rapport du jour : 145 000 FCFA, 12 commandes, 3 produits en rupture de stock imminente. Il peut dormir tranquille.

## 3. Glossary

- **Wallet** — Aggregate de fonds client. Sources : Cash, MoMo, Gift, Cashback, Transfer. Solde = `SUM(amount)` sur wallet_ledger.
- **Wallet Ledger** — Table append-only (INSERT-only, jamais UPDATE/DELETE). Chaque mouvement est une ligne immuable. Backup toutes les 5 min. P0.
- **MoMo** — Mobile Money (Orange Money, MTN MoMo). Label de moyen de paiement et type de source wallet. Pas d'intégration API — le caissier enregistre manuellement.
- **Payment Gate** — Moment où le paiement est validé AVANT que la commande parte en cuisine. Check solde immédiat.
- **Order Lifecycle** — `pending_payment → paid_preparing → ready → delivered`.
- **Bundle** — Édition commerciale définie par un ensemble de feature flags. 6 bundles : Cash, Resto, Stock, Traiteur, Hôtel, Market.
- **Feature Flag** — Entitlement vérifié localement via licence Ed25519. Contrôle l'affichage UI et l'accès API.
- **Licence** — Document JSON signé Ed25519, perpétuel, avec date d'expiration des mises à jour. 4 IDs : Customer, Organization, License, Installation.
- **Client ID** — Identifiant unique du client. Par défaut : numéro de téléphone. Fallback : `CLI-XXXX`.
- **mDNS** — Découverte réseau automatique via `mboacaisse.local`. Pas de configuration IP.
- **ESC/POS** — Protocole d'impression thermique. Généré côté Rust, écrit sur USB ou TCP.

## 4. Features

Les features sont groupées par domaine et ordonnées par priorité d'implémentation. Chaque FR a un ID global stable. Les features P0 sont le socle alpha — sans elles, le produit ne tient pas. Les P1 et P2 sont planifiées mais peuvent être décalées selon la traction alpha.

### 4.1 Serveur LAN embarqué (P0)

**Description :** L'application Tauri lance un serveur Axum au démarrage qui sert à la fois les fichiers statiques du frontend et l'API REST. Accessible depuis tout le LAN sans Internet. Découverte via mDNS. La fenêtre native charge `http://localhost:PORT`.

#### FR-1: Serveur Axum embarqué
Le système lance Axum dans un `tokio::spawn` au `setup()` de Tauri. Il écoute sur `0.0.0.0:PORT`. Sert les fichiers `dist/` du frontend + les routes `/api/*`. La fenêtre native pointe vers `http://localhost:PORT`.

**Conséquences testables :**
- Frontend accessible depuis n'importe quel navigateur du LAN à `http://IP_SERVEUR:PORT`
- Fenêtre native charge l'UI correctement
- Un redémarrage du serveur ne perd pas la BDD

#### FR-2: Découverte mDNS
Le système publie un service mDNS `mboacaisse.local` via la crate `mdns-sd`. Les clients LAN accèdent sans configurer d'IP.

**Conséquences testables :**
- `http://mboacaisse.local:PORT` résout sur tout le LAN
- Le nom est stable après redémarrage du routeur

#### FR-3: Fenêtre native + tray
La fenêtre native Tauri s'ouvre au démarrage (1366×768, min 375×812). L'icône tray est présente avec menu Quit. Le mode headless (pas de fenêtre) est disponible pour les PC partagés.

**Conséquences testables :**
- Fenêtre se ferme sans arrêter le serveur (fonctionne en background)
- Mode headless : pas de fenêtre, serveur actif, notification si arrêt

### 4.2 Authentification & Rôles (P0)

**Description :** 4 rôles avec permissions. Auth locale (email + mot de passe, argon2). Session JWT stockée en cookie HTTP-only. Vérification côté Axum.

#### FR-4: Authentification
Email + mot de passe (argon2). Session JWT en cookie HTTP-only. Middleware Axum vérifie le token sur chaque route protégée.

**Bootstrap admin :** Au premier démarrage (BDD vierge), le système crée automatiquement un compte admin avec des identifiants générés (affichés une fois dans la console/UI de setup). Pas de page login sans admin existant — sinon l'utilisateur arrive sur une page de création admin.

**Conséquences testables :**
- Premier démarrage → écran de création admin (ou identifiants affichés)
- Admin créé → page login normale
- Login avec credentials valides → JWT + cookie
- Login avec mauvais password → 401
- Token expiré → redirection login
- Déconnexion → cookie détruit, plus d'accès aux routes protégées

#### FR-5: 4 rôles et permissions
Rôles : `admin`, `caissier`, `vendeur`, `gestionnaire_stock`. Middleware guard par rôle sur chaque route `/api/*`.

**Conséquences testables :**
- Admin voit tout, crée/modifie utilisateurs
- Caissier voit caisse + paiements, pas stock ni employés
- Vendeur voit commandes + menu, pas caisse
- Gestionnaire voit stock + fournisseurs, pas caisse ni caissiers
- Tentative d'accès hors rôle → 403

### 4.3 Wallet Client (P0 — Cœur)

**Description :** Chaque client est identifié par téléphone (ou ID interne CLI-XXXX sans téléphone). Le wallet est multi-sources (Cash, MoMo, Gift, Cashback, Transfer). Le solde n'est jamais stocké — toujours `SUM(amount)` sur le wallet_ledger append-only. Le paiement est déduit du wallet avant validation commande.

#### FR-6: Identification client
Le client s'enregistre par numéro de téléphone. Fallback : ID interne généré (`CLI-XXXX`) pour les clients sans téléphone.

**Conséquences testables :**
- Enregistrement client → entrée en BDD avec téléphone unique ou ID interne
- Recherche client par téléphone → résultat immédiat
- Client CLI-XXXX pas de pré-commande mais peut payer en caisse

#### FR-7: Wallet multi-sources
Le wallet accepte les transactions de type Cash, MoMo, Gift, Cashback, Transfer. Chaque transaction est une ligne INSERT dans wallet_ledger. Le solde est calculé par `SELECT SUM(amount) FROM wallet_ledger WHERE client_id = ?`.

**Conséquences testables :**
- Dépôt espèces → ligne INSERT avec `type=cash`, `amount=+5000`
- Crédit MoMo → ligne INSERT avec `type=momo`
- Paiement → ligne INSERT avec `type=payment`, `amount=-3500`
- Solde calculé après chaque transaction (vérifié manuellement)
- Deux INSERT simultanés → solde correct (pas de race condition via SQLite sérialisé)

#### FR-8: Wallet ledger append-only (P0 strict)
La table wallet_ledger est INSERT-only. Aucun UPDATE, aucun DELETE. Backup toutes les 5 minutes. Le système refuse de démarrer tant que le ledger n'est pas initialisé — pas de mode dégradé sur les données financières.

**Migration wallet_ledger :** Si un établissement a déjà des commandes payées avant l'implémentation du ledger, la migration crée wallet_ledger et rejoue l'historique des commandes payées en INSERT (une ligne par commande, montant total, type=migration). Sans cette migration, le ledger est vide, le solde de tous les clients est 0, et le système est bloquant à la première tentative de paiement wallet.

**Conséquences testables :**
- Tentative UPDATE sur wallet_ledger → erreur BDD (permission ou trigger)
- Tentative DELETE → erreur BDD
- Backup wallet_ledger auto toutes les 5 min
- Restauration backup → solde correct
- Migration : commandes payées AVANT ledger → lignes INSERT avec `type=migration`

#### FR-9: Payment gate
Le paiement est déduit du wallet AVANT validation de la commande. Si solde insuffisant, message affiché au client. Le wallet négatif n'est pas autorisé par défaut (choix admin, pas par défaut).

**Conséquences testables :**
- Solde suffisant → commande validée → wallet débité → ticket imprimé
- Solde insuffisant → message "Solde insuffisant" → commande non validée
- Admin peut activer le wallet négatif dans les paramètres

#### FR-10: Crédit wallet manuel
Le caissier peut créditer le wallet d'un client manuellement depuis l'UI : montant, type de source (espèces, Orange Money, MTN MoMo). Pas d'appel API externe — le caissier enregistre le paiement que le client a déjà effectué (espèces comptant, ou preuve MoMo sur le téléphone du client). Le type de source est un label de reporting, pas une intégration.

**Conséquences testables :**
- Caissier crédite 5000 FCFA en "Orange Money" → ligne INSERT avec `type=momo`
- Caissier crédite 3000 FCFA en espèces → ligne INSERT avec `type=cash`
- Solde client mis à jour immédiatement
- Rapport du jour ventile par type de source

### 4.4 Ordres & Produits (P0)

**Description :** Cycle de vie complet de la commande. CRUD produits et catégories. Routage vers cuisine.

#### FR-11: Gestion des produits
CRUD complet : produits (nom, prix, catégorie, stock, seuil alerte). Catégories hiérarchiques.

**Conséquences testables :**
- Ajout produit → visible dans le menu
- Modification prix → appliqué immédiatement
- Suppression → produit retiré du menu mais pas des commandes passées

#### FR-12: Cycle de vie commande
La commande suit : `pending_payment → paid_preparing → ready → delivered`. Chaque transition est horodatée. Seul le paiement déclenche le passage en cuisine.

**Conséquences testables :**
- Commande créée → statut `pending_payment`
- Paiement OK → `paid_preparing` → notification cuisine
- Cuisine marque "prête" → `ready` + notification serveur
- Livrée → `delivered`

#### FR-13: Kitchen display
Écran dédié (tablette navigateur) listant les commandes en cours. Filtres : en préparation, prêtes, toutes. Notification sonore sur nouvelle commande.

**Conséquences testables :**
- Nouvelle commande payée → apparaît sur l'écran cuisine en < 2 secondes
- Cuisine marque "prête" → commande disparaît de la liste "en préparation"

### 4.5 Paiements & Impression (P0)

**Description :** Encaissement multi-moyens + impression thermique native.

#### FR-14: Encaissement
Le caissier peut encaisser par wallet client, espèces, Orange Money/MTN MoMo (label seulement), ou combinaison. Le wallet est prioritaire si le client est identifié. Les moyens de paiement sont des étiquettes de reporting — aucune intégration API externe.

**Conséquences testables :**
- Paiement wallet → débit wallet + validation commande
- Paiement espèces → caisse enregistre + pas de débit wallet
- Paiement "Orange Money" → caisse enregistre le label en BDD
- Paiement mixte → wallet partiel + espèces (ou MoMo label)

#### FR-15: Impression thermique native
Le backend Rust génère du buffer ESC/POS et écrit sur `/dev/usb/lp0` (Linux) ou socket TCP:9100. Pas de popup navigateur. Formats 58mm et 80mm.

**Conséquences testables :**
- Validation caisse → ticket imprimé en < 3 secondes
- Ticket contient : nom établissement, articles, totaux, mode paiement, remerciements
- Imprimante déconnectée → pas de crash, message à l'écran
- [P1] File d'attente async avec retry + fallback ticket numérique (écran client)

### 4.6 Fidélité & Parrainage (P0)

**Description :** Cashback automatique et parrainage. Zéro config gérant. Pas de carte.

#### FR-16: Cashback automatique
Chaque commande payée par wallet crédite un cashback de 5%. Optionnellement progressif : 3% (1-10 commandes), 5% (11-30), 8% (31+).

**Conséquences testables :**
- Commande wallet de 5000 FCFA → 250 FCFA crédités (5%) en `type=cashback`
- Cashback disponible immédiatement pour la prochaine commande
- Seuil progressif configurable en backend (pas d'UI V1)

#### FR-17: Parrainage
Le client saisi un numéro de téléphone à l'enregistrement. Le parrain (si existe) et le filleul reçoivent 1000 FCFA chacun sur leur wallet. [ASSUMPTION : le lien est fait à l'enregistrement, pas avant]

**Conséquences testables :**
- Nouveau client avec téléphone parrain → 1000 FCFA sur les deux wallets
- Pas de parrain → pas de bonus
- Double parrainage non autorisé

### 4.7 Table & Menu Public QR (P0)

**Description :** QR code par table, menu public sans auth, identification par téléphone, commande client.

#### FR-18: QR code par table
Chaque table a un QR code (URL encodant le numéro de table). Généré côté serveur. Imprimé sur papier plastifié. Admin peut entrer manuellement un numéro de table (fallback).

**Conséquences testables :**
- Scan QR → ouvre le menu public avec table pré-sélectionnée
- Table incorrecte → admin peut la changer

#### FR-19: Menu public (5 écrans)
Le client scanne → landing QR → menu (catégories + produits) → panier → identification téléphone → confirmation + statut. 5 écrans, pas d'app. Le flux est identique pour takeaway et table.

**Conséquences testables :**
- Menu public accessible sans auth sur `/menu`
- Navigation complète : scan → commande → confirmation
- Client identifié par téléphone (si déjà enregistré) ou invité (pas de wallet, paiera en caisse)

#### FR-20: Table management
Création, modification, suppression des tables. Association client→table→commande. Vue du plan des tables (libre/occupée).

**Conséquences testables :**
- Création table → QR généré
- Commande liée à une table → table marquée occupée
- Commande terminée → table libérée
- Vue plan des tables dans l'UI caissier/admin

### 4.8 Feature Gating & Licences (P0)

**Description :** Feature flags via licence Ed25519. Un seul binaire, 6 bundles. Vérification offline-first.

#### FR-21: Licence Ed25519
Le système vérifie localement la signature Ed25519 de la licence. La clé publique est embarquée dans le binaire. La clé privée est sur le License Server (jamais dans le client).

**Conséquences testables :**
- Licence valide → produit débloqué (fonctionnalités selon entitlements)
- Licence invalide ou trafiquée → message "Licence invalide"
- Pas d'Internet → fonctionnement normal (vérification locale uniquement)

#### FR-22: Feature flags
Chaque entitlement de la licence contrôle l'affichage UI et l'accès API. Les flags sont vérifiés côté client (UI) ET côté serveur (API).

**Conséquences testables :**
- Bundle Cash : wallet, cash, fidélité, rapports basiques → seules ces features sont visibles
- Bundle Resto : Cash + MoMo, pré-commande, cuisine, tables → déverrouille les features supplémentaires
- Feature non comprise dans la licence → 403 sur l'API, masquée dans l'UI
- Même binaire, flags différents → comportement différent

#### FR-23: Activation initiale
À la première installation, l'utilisateur saisit sa clé d'activation. Le système génère un Installation ID (machine + timestamp). La licence signée est stockée localement. Si connecté, activation envoyée au License Server.

**Conséquences testables :**
- Saisie clé valide → licence téléchargée et stockée
- Pas d'Internet → grace period (licence temporaire de 7 jours) [ASSUMPTION]
- Déjà activée → pas de seconde activation
- Installation ID unique par machine

### 4.9 Rapports & Backup (P0)

#### FR-24: Rapports de base
Rapport journalier (ventes par caissier, par mode de paiement, total). Rapport hebdomadaire/mensuel (même indicateurs). Exportable.

**Conséquences testables :**
- Rapport jour en cours disponible à tout moment
- Ventilation par caissier
- Ventilation par moyen de paiement (espèces, wallet, MoMo)
- Rapport J-7, J-30 disponibles si données existent

#### FR-25: Backup/Restore
Backup automatique quotidien + avant mise à jour. Backup manuel via UI admin. Rotation (N derniers backups). Restauration depuis l'UI admin avec double confirmation + backup auto de l'état courant avant restore.

**Conséquences testables :**
- Backup quotidien créé automatiquement
- Backup manuel via bouton admin → ZIP téléchargeable
- Restauration → BDD remplacée → app redémarre
- Backup auto avant restore → filet de sécurité

### 4.10 mDNS Personnalisable (P1)

#### FR-26: Nom mDNS personnalisable
Au setup, l'admin peut changer le nom mDNS (`chezbob.local` au lieu de `mboacaisse.local`). Fallback admin pour IP manuelle.

**Conséquences testables :**
- Changement nom mDNS → résolution sur le nouveau nom
- Fallback IP → accès par IP si mDNS indisponible (AP Isolation, Guest Network)

### 4.11 Bundle Resto (P2)

#### FR-27: Mode restaurant (P2)
Features supplémentaires : pré-commande, kitchen display amélioré, assignation serveur par table, édition commande avant envoi cuisine, notes par article.

**Conséquences testables :**
- Serveur assigné à une table → voit ses tables
- Pré-commande prise par QR → serveur peut modifier avant validation

### 4.12 Bundle Stock (P2)

#### FR-28: Inventaire fournisseurs (P2)
Fournisseurs, alertes seuil (notification stock bas), devis, réception commande fournisseur, multi-dépôt.

### 4.13 WebSocket Temps Réel (P2)

#### FR-29: WebSocket Axum (P2)
Mise à jour en temps réel via `axum::extract::ws`. Plan des tables, notifications cuisine, afficheur client, statut commandes.

## 5. Cross-Cutting NFRs

### NFR-1: Intégrité wallet
Zéro perte de données wallet. Ledger append-only, backup toutes les 5 min, pas de UPDATE/DELETE sur wallet_ledger. Vérifié : toute transaction wallet est une ligne INSERT immuable.

### NFR-2: Offline d'abord
100% des fonctionnalités de caisse fonctionnent sans Internet. Internet n'est nécessaire que pour : activation licence initiale, mises à jour. Un établissement doit pouvoir fonctionner 30 jours sans connexion.

### NFR-3: Résilience triangle
```
    Wallet
    /     \
Ledger — Impression Queue
```
Trois sous-systèmes indépendants. Chacun survit si un autre est down :
- Wallet fonctionne sans imprimante (commande prise, ticket en file)
- Imprimante fonctionne sans wallet sync (file locale, retry)
- Ledger survit à un crash wallet (append-only, rejouable au redémarrage)

### NFR-4: Performance
- Temps encaissement → ticket : < 3 secondes
- Commande → écran cuisine : < 2 secondes
- Chargement menu public : < 1 seconde (fichiers statiques servis par Axum)
- Backup wallet_ledger : < 1 seconde (INSERT-only, petit volume)

### NFR-5: Sécurité
- Mots de passe hashés argon2
- Tokens JWT signés, HTTP-only
- Feature flags vérifiés côté API (pas seulement UI)
- Licence signée Ed25519, clé privée jamais dans le client
- CSP désactivé (nécessaire pour Tauri WebView), mais REST API protégée par JWT

### NFR-6: Traçabilité
Toute transaction financière est inscrite dans wallet_ledger avec timestamp, type, montant, référence. Impossible de supprimer ou modifier une ligne.

### NFR-7: Diagnostic réseau
L'application expose un diagnostic réseau intégré : WiFi, serveur, mDNS, BDD, WebSocket. Utilisable pour le support distant.

## 6. Constraints and Guardrails

### 6.1 Infrastructure
- **PC unique** : l'app tourne sur le PC de l'établissement (souvent partagé, pas dédié). Mode headless + notification si arrêt.
- **Pas d'Internet** : zéro dépendance cloud pour le fonctionnement quotidien. Licence vérifiée localement.
- **Réseau local** : tout le trafic reste sur le LAN. Portée : WiFi de l'établissement.
- **AP Isolation** : certains routeurs bloquent la communication entre clients WiFi. Le diagnostic réseau détecte ce cas et suggère la configuration correcte.

### 6.2 Budget
- **Performance** : doit tourner sur un PC de 5 ans (4 Go RAM, CPU Intel Celeron ou équivalent).
- **Stockage** : BDD SQLite < 500 Mo pour un an d'activité d'un établissement moyen. Backups : rotation 30 jours.
- **Binaire** : < 50 Mo compressé.

### 6.3 Réglementaire
- **Dépôt de garantie** : feature optionnelle, désactivée par défaut, avec avertissement setup (zone grise régulation financière).
- **Données** : tout reste sur le PC du commerçant. Pas de données clients chez un tiers.

## 7. Why Now

Le marché des POS en Afrique francophone est naissant mais s'accélère. HandLit (500 stores), Velko (5200 commerçants), Djouri POS, TigiPOS grandissent — mais aucun n'a de part dominante. La majorité des établissements utilise encore le cash et le carnet.

La fenêtre est ouverte mais se referme : HandLit et Velko ont un an d'avance terrain, des retours quotidiens, des revendeurs dans les quartiers. MboaCaisse arrive avec une architecture plus solide (serveur Rust natif vs app Android/web, ledger append-only, licensing Ed25519) mais zéro relation terrain. Le temps de construire l'avantage technique avant que le marché se standardise sur des solutions Android.

Le timing est bon : le Mobile Money est omniprésent (Orange Money, MTN MoMo), les commerçants commencent à comprendre qu'ils perdent de l'argent sans digitalisation, et le parc de PC existant dans les établissements est sous-exploité.

## 8. Integration and Dependencies

### 8.1 Mobile Money (MoMo)
MoMo est un label de moyen de paiement — pas d'intégration API. Le caissier sélectionne "Orange Money" ou "MTN MoMo" dans l'UI d'encaissement ou de crédit wallet. Aucun appel externe, aucune dépendance réseau. Le reporting ventile par moyen de paiement.

### 8.2 Licence (vérification locale seulement)
En P0 alpha, pas de License Server cloud. La licence est pré-générée hors-band (outil CLI ou script) et embarquée avec le binaire. Le vérification côté client (verify.rs, entitlements.rs) existe.

Le License Server cloud (API REST, dashboard admin, génération de licences, paiements en ligne) est à construire — projet séparé, P4+.
- En alpha : licence signée fournie avec le binaire. Activation = copie du fichier de licence dans le dossier de config.
- En P1+ : endpoint d'activation avec clé, vérification périodique optionnelle.
- En P4+ : License Server complet.

### 8.3 Imprimante thermique
- USB (Linux : `/dev/usb/lp*`). TCP/IP (port 9100).
- Formats : ESC/POS 58mm et 80mm.
- Pas de dépendance externe : génération Rust native du buffer.

### 8.4 Dépendances build
- Tauri 2, Rust, `mdns-sd`, `sqlx`, `axum`, `tokio`, `tower-http`
- Frontend : Vue 3, Nuxt 4, Tailwind v4
- Pas de cloud, pas de SaaS, pas de base externe pour le fonctionnement quotidien

## 9. MVP Scope

### 9.1 In Scope (alpha)

Ce qui doit être livré et stable pour qu'un premier établissement fonctionne :

**Wallet Client (P0)**
- [ ] Client identification par téléphone + ID interne CLI-XXXX
- [ ] Wallet ledger append-only (INSERT-only, backup 5min)
- [ ] Wallet multi-sources (Cash, MoMo, Gift, Cashback, Transfer)
- [ ] Payment gate avant validation commande
- [ ] Solde calculé par SUM

**Caisse (P0)**
- [ ] Cycle commande (pending_payment → paid_preparing → ready → delivered)
- [ ] CRUD produits + catégories
- [ ] Encaissement wallet + espèces
- [ ] Kitchen display sur navigateur
- [ ] Scan QR + menu public 5 écrans
- [ ] Table management

**Serveur LAN (P0)**
- [ ] Axum embarqué dans Tauri
- [ ] Frontend servi statiquement
- [ ] mDNS (mboacaisse.local)
- [ ] Auth + JWT + 4 rôles
- [ ] Fenêtre native + tray + mode headless

**Fidélité (P0)**
- [ ] Cashback auto 5% (progressif 3/5/8% optionnel)
- [ ] Parrainage 1000 FCFA

**Licences (P0)**
- [ ] Vérification signature Ed25519
- [ ] Feature flags par entitlement
- [ ] Activation initiale

**Rapports & Backup (P0)**
- [ ] Rapport journalier (ventes par caissier, par mode)
- [ ] Backup auto quotidien + manuel
- [ ] Restauration depuis UI admin

**Déploiement alpha :** clé USB pour les 3 premiers alphas. Installation en personne ou assistée par WhatsApp. Le binaire est copié depuis la clé, pas de téléchargement.

### 9.2 Out of Scope for MVP

- **Pas d'intégration API MoMo** (label seulement — aucune API externe prévue)
- **mDNS personnalisable** (P1 — `mboacaisse.local` fixe en V1, admin peut entrer IP)
- **Bundles Resto/Stock/etc.** (P2 — seul Mboa Cash est activé à l'alpha)
- **Barcode scanner** (P2.3)
- **Auto-updater** (P2.4)
- **Second display / afficheur client** (P2.2)
- **WebSocket temps réel** (P2.7 — polling HTTP en attendant)
- **Sync multi-instance** (P2 — wallet par instance = acceptable pour 3 ans)
- **Apps mobiles Android/iOS** (navigateur seulement)
- **License Server cloud** (dashboard admin, paiements en ligne, API REST — à construire en P4+)
- **Lien de téléchargement privé / site commercial** (P1 — alpha distribué par clé USB)

## 10. Non-Goals (Explicit)

- MboaCaisse ne sera PAS une solution cloud. Pas de sync cloud. Pas de sauvegarde automatique vers un serveur central.
- MboaCaisse ne sera PAS un terminal de paiement électronique (ni carte bancaire, ni TPE). Le MoMo est un canal d'approvisionnement du wallet.
- MboaCaisse ne sera PAS multi-device avec état partagé en temps réel (pas de WebSocket V1 — les clients qui changent d'écran voient l'état au prochain rafraîchissement serveur).
- MboaCaisse ne sera PAS un CRM, un outil de marketing, ou une plateforme de réservation. La fidélité est limitée au cashback et parrainage.
- MboaCaisse ne sera PAS une app mobile. Les clients utilisent le navigateur. Point.

## 11. Success Metrics

**SM-1 : Wallet adoption > 60%**
% des clients passant par le wallet dans les 3 mois suivant l'installation alpha. Validé sur les 3 premiers établissements.

**SM-2 : Zéro perte de données wallet**
Aucun cas de solde client incorrect ou perte de transaction depuis le déploiement ledger append-only. Vérifié par audit des wallets après 30 jours d'exploitation.

**SM-3 : Temps encaissement (UX) ≤ 90s**
Métrique humaine : du moment où le caissier commence à sélectionner les articles jusqu'à l'impression du ticket (inclut la recherche produit, le dialogue client, le paiement). Chronométré sur 20 transactions. La métrique technique (commande validée → ticket imprimé) est couverte par NFR-4 (< 3s).

**SM-4 : 30 jours sans Internet**
L'établissement tient 30 jours sans connexion Internet. Toutes les fonctions de caisse sont opérationnelles. Vérifié : désactiver Internet, faire fonctionner l'app une journée complète.

**SM-5 : 3 alpha clients en 6 mois**
3 établissements en production dans les 6 mois suivant la sortie alpha. Au moins 1 qui a connu la version "carnet" avant et peut témoigner.

**Counter-metric SM-C1 : Complexité n'augmente pas la friction de distribution**
Ne pas optimiser le nombre de features au détriment de la simplicité d'installation. Mesure : temps d'installation (téléchargement → première commande) < 15 minutes.

## 12. Open Questions

- OQ-1. Comment gérer le scan de QR code si le client n'a pas de forfait data (le QR demande une page web sur le LAN — pas de data nécessaire, mais le client doit être sur le WiFi de l'établissement. Est-ce un problème ?)
- OQ-2. Quel est le comportement exact du webhook MoMo ? Implémentation réelle à documenter avant P1.
- OQ-3. Réservé

## 13. Assumptions Index

- **§4.3/FR-8** — La migration wallet_ledger peut rejouer l'historique des commandes payées en INSERT sans perte ni doublon.
- **§4.8/FR-23** — Grace period de 7 jours si pas d'Internet à l'activation. Déplacer en décision si confirmé.
- **§4.7/FR-18** — Le QR code est imprimé sur papier plastifié. Pas de QR dynamique (même URL pour la table).
- **§2.3/UJ-4** — Le propriétaire est sur le même LAN que le serveur quand il consulte depuis chez lui. Si le routeur ne porte pas sur le même sous-réseau, ça ne marche pas. Solution : mDNS ne traverse pas les sous-réseaux.
- **§4.6/FR-17** — Le lien de parrainage est fait à l'enregistrement, pas en amont.
- **§5/NFR-4** — Les performances cibles sont atteignables sur un PC de 5 ans avec SQLite et Axum.
- **§8.2** — La vérification locale Ed25519 existe (verify.rs, entitlements.rs). Le License Server cloud est à construire (P4+).
- **§9.2** — Le polling HTTP est suffisant pour les écrans cuisine en V1. WebSocket est un confort, pas un besoin. Confirmé.
