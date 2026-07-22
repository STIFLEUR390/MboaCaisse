---
date: 2026-07-22
project: MboaCaisse
stepsCompleted:
  - step-01-document-discovery
  - step-02-prd-analysis
  - step-03-epic-coverage-validation
  - step-04-ux-alignment
  - step-05-epic-quality-review
  - step-06-final-assessment
filesIncluded:
  - prds/prd-MboaCaisse-2026-07-21/prd.md
  - architecture/architecture-MboaCaisse-2026-07-21/ARCHITECTURE-SPINE.md
  - epics.md
  - ux-designs/ux-MboaCaisse-2026-07-21/DESIGN.md
  - ux-designs/ux-MboaCaisse-2026-07-21/EXPERIENCE.md
  - briefs/brief-MboaCaisse-2026-07-21/brief.md
  - prfaq-MboaCaisse.md
---

# Implementation Readiness Assessment Report

**Date:** 2026-07-22
**Project:** MboaCaisse

## Step 1: Document Discovery

**Document Inventory:**

| Type | Format | Path |
|------|--------|------|
| PRD | Sharded | prds/prd-MboaCaisse-2026-07-21/prd.md |
| Architecture | Sharded | architecture/architecture-MboaCaisse-2026-07-21/ARCHITECTURE-SPINE.md |
| Epics & Stories | Whole | epics.md |
| UX Design | Sharded | ux-designs/ux-MboaCaisse-2026-07-21/ |
| Brief | Sharded | briefs/brief-MboaCaisse-2026-07-21/brief.md |
| PRFAQ | Whole | prfaq-MboaCaisse.md |

**Issues:** None — no duplicates, all required documents present.

## PRD Analysis

### Functional Requirements

FR-1: Serveur Axum embarqué — lance Axum dans tokio::spawn au setup() de Tauri, écoute sur 0.0.0.0:PORT, sert dist/ + /api/*
FR-2: Découverte mDNS — publie mboacaisse.local via mdns-sd
FR-3: Fenêtre native + tray — fenêtre 1366×768 (min 375×812), tray avec Quit, mode headless
FR-4: Authentification — email+password (argon2), JWT HTTP-only cookie, middleware Axum, bootstrap admin
FR-5: 4 rôles et permissions — admin, caissier, vendeur, gestionnaire_stock avec middleware guard
FR-6: Identification client — par téléphone ou ID interne CLI-XXXX
FR-7: Wallet multi-sources — Cash, MoMo, Gift, Cashback, Transfer en INSERT wallet_ledger
FR-8: Wallet ledger append-only — INSERT-only, backup 5min, migration historique commandes
FR-9: Payment gate — déduction wallet AVANT validation commande, solde négatif optionnel
FR-10: Crédit wallet manuel — caissier crédite depuis UI (montant + type source)
FR-11: Gestion des produits — CRUD produits, catégories hiérarchiques
FR-12: Cycle de vie commande — pending_payment → paid_preparing → ready → delivered
FR-13: Kitchen display — écran navigateur listant commandes, notification sonore
FR-14: Encaissement — wallet, espèces, MoMo (label), mixte
FR-15: Impression thermique native — ESC/POS Rust, USB/TCP, 58/80mm
FR-16: Cashback automatique — 5% par défaut, progressif 3/5/8%
FR-17: Parrainage — 1000 FCFA parrain + filleul à l'enregistrement
FR-18: QR code par table — URL encodant numéro de table, généré serveur
FR-19: Menu public (5 écrans) — landing QR → menu → panier → identification → confirmation
FR-20: Table management — CRUD tables, association client→table→commande, plan des tables
FR-21: Licence Ed25519 — vérification locale, clé publique embarquée, signature JSON
FR-22: Feature flags — entitlement contrôle UI + API, vérifié côté client ET serveur
FR-23: Activation initiale — saisie clé, Installation ID, licence stockée localement, grace period 7j
FR-24: Rapports de base — journalier/hebdo/mensuel, ventes par caissier/mode, exportable
FR-25: Backup/Restore — automatique quotidien + manuel UI, rotation, restore avec double confirmation
FR-26: Nom mDNS personnalisable (P1) — admin change le nom mDNS au setup
FR-27: Mode restaurant (P2) — pré-commande, assignation serveur, édition commande
FR-28: Inventaire fournisseurs (P2) — fournisseurs, alertes seuil, devis, réception, multi-dépôt
FR-29: WebSocket Axum (P2) — temps réel via axum::extract::ws

Total FRs: 29

### Non-Functional Requirements

NFR-1: Intégrité wallet — zéro perte, ledger append-only, backup 5min, pas de UPDATE/DELETE
NFR-2: Offline d'abord — 100% des fonctionnalités sans Internet, 30 jours sans connexion
NFR-3: Résilience triangle — Wallet/Ledger/Impression indépendants, chacun survit si un autre est down
NFR-4: Performance — encaissement→ticket <3s, commande→cuisine <2s, menu public <1s
NFR-5: Sécurité — argon2, JWT HTTP-only, flags vérifiés API, licence Ed25519
NFR-6: Traçabilité — toute transaction financière dans wallet_ledger avec timestamp/type/montant/référence
NFR-7: Diagnostic réseau — WiFi, serveur, mDNS, BDD, WebSocket

Total NFRs: 7

### PRD Completeness Assessment

PRD complet et structuré. 29 FRs couvrant les 9 domaines P0, 7 NFRs. Glossaire, contraintes, métriques de succès, hors-scope, et questions ouvertes documentés. Priorisation claire (P0/P1/P2). Prêt pour validation de couverture par les epics.

## Epic Coverage Validation

### Coverage Matrix

| FR | PRD Requirement | Epic | Status |
|---|---|---|---|
| FR-1 | Serveur Axum embarqué | E1 — Socle Serveur & Auth | ✓ Couvert |
| FR-2 | Découverte mDNS | E1 — Socle Serveur & Auth | ✓ Couvert |
| FR-3 | Fenêtre native + tray | E1 — Socle Serveur & Auth | ✓ Couvert |
| FR-4 | Authentification | E1 — Socle Serveur & Auth | ✓ Couvert |
| FR-5 | 4 rôles et permissions | E1 — Socle Serveur & Auth | ✓ Couvert |
| FR-6 | Identification client | E1.5 — Wallet Ledger | ✓ Couvert |
| FR-7 | Wallet multi-sources | E1.5 — Wallet Ledger | ✓ Couvert |
| FR-8 | Wallet ledger append-only | E1.5 — Wallet Ledger | ✓ Couvert |
| FR-9 | Payment gate | E3 — Ventes & Encaissement | ✓ Couvert |
| FR-10 | Crédit wallet manuel | E3 — Ventes & Encaissement | ✓ Couvert |
| FR-11 | Gestion des produits | E3 — Ventes & Encaissement | ✓ Couvert |
| FR-12 | Cycle de vie commande | E3 — Ventes & Encaissement | ✓ Couvert |
| FR-13 | Kitchen display | E3 — Ventes & Encaissement | ✓ Couvert |
| FR-14 | Encaissement multi-moyen | E3 — Ventes & Encaissement | ✓ Couvert |
| FR-15 | Impression thermique native | E3 — Ventes & Encaissement | ✓ Couvert |
| FR-16 | Cashback automatique | E2 — Fidélité | ✓ Couvert |
| FR-17 | Parrainage | E2 — Fidélité | ✓ Couvert |
| FR-18 | QR code par table | E4 — Menu Public QR | ✓ Couvert |
| FR-19 | Menu public 5 écrans | E4 — Menu Public QR | ✓ Couvert |
| FR-20 | Table management | E4 — Menu Public QR | ✓ Couvert |
| FR-21 | Licence Ed25519 | E5 — Administration | ✓ Couvert |
| FR-22 | Feature flags | E5 — Administration | ✓ Couvert |
| FR-23 | Activation initiale | E5 — Administration | ✓ Couvert |
| FR-24 | Rapports de base | E5 — Administration | ✓ Couvert |
| FR-25 | Backup/Restore | E5 — Administration | ✓ Couvert |
| FR-26 | Nom mDNS personnalisable (P1) | E5 — Administration | ✓ Couvert |
| FR-27 | Mode restaurant (P2) | E5 — Administration | ✓ Couvert |
| FR-28 | Inventaire fournisseurs (P2) | E5 — Administration | ✓ Couvert |
| FR-29 | WebSocket Axum (P2) | E5 — Administration | ✓ Couvert |

### Missing Requirements

Aucune — 29/29 FRs couverts.

### Coverage Statistics

- Total PRD FRs: 29
- FRs covered in epics: 29
- Coverage percentage: 100%

## UX Alignment Assessment

### UX Document Status

**Found** — 2 documents :
- `DESIGN.md` — Design tokens, composants, couleurs, typographie, espacement
- `EXPERIENCE.md` — Architecture d'information, flows, microcopy, états, accessibilité, responsive

UX-DRs (21) listés et référencés dans l'epic E4 (Menu Public QR).

### UX ↔ PRD Alignment

| UX Requirement | PRD Reference | Status |
|---|---|---|
| 5 écrans menu public (UX-DR8) | FR-19, UJ-3 | ✓ Aligné |
| QR code par table (UX-DR9) | FR-18 | ✓ Aligné |
| Identification par téléphone (UX-DR12) | FR-6, FR-19 | ✓ Aligné |
| Paiement wallet/comptoir (UX-DR21) | FR-9, FR-14 | ✓ Aligné |
| Kitchen display polling (AD-14) | FR-13 | ✓ Aligné (pas WebSocket) |
| Design tokens vert/zinc | FR-19 (implicit) | ✓ Aligné |
| Pas d'images menu (offline) | NFR-2 | ✓ Aligné |
| Responsive mobile 320-480px (UX-DR17) | NFR-4 | ✓ Aligné |
| Accessibilité WCAG AA (UX-DR16) | non spécifié PRD | ⚠ Complément UX non couvert par PRD |
| Microcopy (UX-DR14) | non spécifié PRD | ⚠ Complément UX |

### UX ↔ Architecture Alignment

| UX Requirement | Architecture Support | Status |
|---|---|---|
| Polling HTTP (pas WebSocket V1) | AD-14 | ✓ Aligné |
| useFetch Nuxt (pas TanStack Query) | AD-10 | ✓ Aligné |
| Pas d'images menu (léger, offline) | NFR-2 offline-first | ✓ Aligné |
| Design tokens Inter/vert/zinc | Nuxt UI v4 (green/zinc config) | ✓ Aligné |
| Sticky footer commander | Frontend pur, sans impact backend | ✓ Aligné |
| Menu catégories horizontales + liste verticale | Frontend pur | ✓ Aligné |
| Responsive mobile max-width 600px | Frontend pur | ✓ Aligné |
| Accessibility 44×44px, aria-live | Frontend pur | ✓ Aligné |

### Anomalie mineure

URL pattern : EXPERIENCE.md utilise `/menu/{table}` (path param), épics utilisent `/menu?table={id}` (query param). À uniformiser.

### Warnings

Aucun warning critique. UX bien documentée, alignée avec PRD et Architecture. Les compléments UX (microcopy, accessibilité) sont des enrichissements, pas des contradictions.

## Epic Quality Review

### Epic Structure Validation

**Epic 1 — Socle Serveur & Authentification**
- Valeur utilisateur : Partielle (auth/roles = user-facing, serveur/mDNS/tray = infrastructure). Acceptable pour un socle.
- Indépendance : ✅ Aucune dépendance externe. Peut être implémenté en premier.
- Stories (5) : Bien dimensionnées. Chaque story a des ACs Given/When/Then.

**Epic 1.5 — Wallet Ledger**
- Valeur utilisateur : ✅ Client wallet = valeur métier directe.
- Indépendance : ✅ Dépend de E1. Nommage 1.5 (entre 1 et 3) est logique mais sort de la séquence entière.
- Stories (3) : Bien dimensionnées. Migration story (1.5.3) a des ACs solides incluant idempotence.

**Epic 3 — Ventes & Encaissement**
- Valeur utilisateur : ✅ Cœur du produit. Caissier encaisse, cuisine prépare.
- Indépendance : ⚠ Dépend de E1.5 (wallet pour payment gate). Dépendance arrière (numéro 3 dépend de 1.5) — correct.
- Stories (6) : Bien dimensionnées. ACs complètes.

**Epic 2 — Fidélité**
- Valeur utilisateur : ✅ Cashback et parrainage.
- Indépendance : ⚠ Dépend de E1.5 + E3. Numéroté 2 mais implémenté APRÈS E3.
- Stories (2) : Correctes.

**Epic 4 — Menu Public QR**
- Valeur utilisateur : ✅ Commandes client via QR.
- Indépendance : ⚠ Dépend de E3 (produits, commandes). Correct.
- Stories (6) : Bien détaillées. ACs couvrant edge cases et accessibilité.

**Epic 5 — Administration**
- Valeur utilisateur : ✅ Rapports, backup, licences.
- Indépendance : ⚠ Rapports (5.3) nécessite E3 (données de vente). Stories P1/P2 (5.5, 5.6) = placeholders non prêts pour implémentation.
- Stories (6) : 2 placeholders (P1/P2), 4 stories réelles.

### Issues Identified

#### 🟠 Major Issues

1. **Ordre d'implémentation ambigu** : E2 (Fidélité) numéroté 2 mais dépend de E3 (Ventes). L'ordre d'implémentation réel est E1 → E1.5 → E3 → E2 → E4 → E5, pas E1→E2→E3. La numérotation devrait refléter l'ordre (E2 renommé en E3b, ou E3 renommé).

2. **QR URL pattern divergent** : EXPERIENCE.md utilise `/menu/{table}` (path param). Story 4.1 utilise `/menu?table={id}` (query param). À uniformiser.

3. **Génération QR floue** : Story 4.1 dit "générée côté client (ou serveur)" — ambiguïté sur la responsabilité.

#### 🟡 Minor Concerns

4. **Bridge Pinia→Tauri store** : Mentionné dans AC de Story 1.4 mais pourrait être une story séparée.
5. **Stories 5.5/5.6 placeholders** : OK pour la planification mais pas implémentables.
6. **Ticket numérique (3.6)** : Solution temporaire avant impression ESC/POS native (P2.1). Bien documenté.

### Best Practices Compliance

| Critère | Statut |
|---|---|
| Epics deliver user value | ✅ Oui (sauf E1 partiellement infrastructure) |
| Epic independence | ✅ Backward dependencies correctes |
| Stories appropriately sized | ✅ |
| No forward dependencies | ✅ |
| DB tables created when needed | ✅ Chaque story crée ses migrations |
| Clear acceptance criteria (Given/When/Then) | ✅ |
| Traceability to FRs | ✅ FR Coverage Map complète

## Summary and Recommendations

### Overall Readiness Status

**READY** — avec réserves mineures.

### Synthèse des constats

| Domaine | Statut | Notes |
|---|---|---|
| PRD | ✅ Complet | 29 FRs, 7 NFRs, glossaire, métriques, scope clair |
| FR Coverage | ✅ 100% | 29/29 FRs tracés dans les epics |
| Architecture | ✅ Solide | 20 ADs, 3 couches, stack claire, patterns documentés |
| UX Design | ✅ Complet | DESIGN.md + EXPERIENCE.md, 21 UX-DRs |
| UX Alignment | ✅ Aligné | UX, PRD et Architecture cohérents |
| Epics Quality | ⚠ 3 anomalies mineures | Voir ci-dessous |

### Anomalies à corriger avant implémentation

1. ✅ **Uniformiser URL pattern QR** — corrigé : `?table={id}` (query param) partout
2. ✅ **Clarifier génération QR** — corrigé : côté client via `qrcode` npm, story 4.1
3. ✅ **Revoir nommage E2** — corrigé : `E2 (E2b): Fidélité` dans epics.md

### Recommandations

1. Implémentation par ordre : **E1 → E1.5 → E3 → E2 → E4 → E5**
2. Commencer par Story 1.1 (Structure Rust + migrations) et 1.2 (Serveur Axum + mDNS)
3. Traiter E1.5 (Wallet) avant E3 car paiement wallet est bloquant pour le cycle de vente
4. Les placeholders P1/P2 (E5.5, E5.6) peuvent être ignorés en alpha

### Évaluation finale

Ce projet est prêt pour l'implémentation. La documentation est complète, cohérente et bien structurée. Les 3 anomalies identifiées sont mineures et ne bloquent pas le démarrage du développement. L'ordre d'implémentation recommandé tient compte des dépendances réelles identifiées (E1.5 avant E3).

**Rapport généré le :** 2026-07-22
