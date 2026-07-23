---
epic: 1
name: "Socle Serveur & Authentification"
date: 2026-07-23
status: complete
stories:
  - 1-1-structure-rust-layered-migrations-initiales
  - 1-2-serveur-axum-embarque-mdns
  - 1-3-authentification-register-login-jwt
  - 1-4-fenetre-native-tray-mode-headless
  - 1-5-roles-permissions-middleware-guard-seed-admin
---

# Rétrospective — Epic 1 : Socle Serveur & Authentification

## Vue d'ensemble

L'Epic 1 a posé les fondations de MboaCaisse : serveur Axum embarqué avec Tauri, authentification JWT, 4 rôles avec permissions, fenêtre native avec tray, config store, et seed admin. 5 stories, de la structure initiale du projet Rust jusqu'au middleware de vérification des permissions.

### Chiffres clés

- **Stories :** 5 (toutes done)
- **Commits :** 5+ dans la branche principale
- **Fichiers Rust créés :** ~20 (api/, domain/, db/ + server, mdns, settings)
- **Fichiers frontend créés :** ~8 (pages, composables, middleware, stores)
- **Dépendances Rust ajoutées :** 12 (Axum, Tokio, rusqlite, argon2, jsonwebtoken, etc.)
- **Architecture Decisions (ADs) :** 20 ADs définies dans le spine

## Ce qui a bien fonctionné ✅

### Architecture solide dès le départ
Les 20 ADs du Architecture Spine ont fourni une feuille de route claire. Chaque story savait exactement quelle AD elle implémentait. Pas de dettes architecturales majeures accumulées.

### Stories bien séquencées
1.1 (structure+migrations) → 1.2 (serveur) → 1.3 (auth) → 1.4 (fenêtre+tray+config) → 1.5 (permissions). Chaque story dépendait logiquement de la précédente.

### Patterns cohérents
- **3-layer errors** (db/DomainError/api) maintenu partout
- **Repository pattern** (trait dans domain, impl dans db)
- **Auto-imports Tauri** via module Nuxt (pas d'imports manuels @tauri-apps/*)
- **Format erreur API** `{ error, code }` en snake_case

### Code review efficace
La review de la story 1.5 a trouvé 2 bugs réels (prefix collision dans le middleware, page settings accessible sans middleware admin). Fixés immédiatement.

### Sensibilité aux edge cases sur le LAN
- Graceful shutdown avec backup BDD (AD-9)
- Hide-to-tray au lieu de Exit (AD-9)
- Seed admin idempotent (AD-11)
- Silent JWT refresh (AD-11)

## Ce qui peut être amélioré 🔧

### Suivi de statut
La story 1.1 est restée en "in-progress" alors qu'elle était implémentée. L'epic 1 n'était pas marquée "done". Le sprint-status.yaml n'était pas toujours synché avec la réalité.

**Action :** Marquer les stories done immédiatement après implémentation et review.

### Absence de tests
Aucun framework de test n'est configuré. Les stories 1.1 à 1.5 n'ont aucun test unitaire ou d'intégration. Les reviews manuelles et le code review ont compensé, mais c'est risqué pour les épics futures (Wallet Ledger, Ventes).

**Action :** Configurer un framework de test (cargo test pour Rust, vitest pour frontend) avant l'Epic 3.

### Race conditions documentées
- DELETE dernier admin : deux requêtes simultanées peuvent passer
- create_user : vérification email puis INSERT non atomique (protégé par UNIQUE BDD)
- update_user : pas d'optimistic locking

**Action :** Dépriorisé pour l'alpha. À corriger si des bugs sont remontés.

### Complexité du middleware de permissions
Le middleware `required_permission()` utilise un mapping route→permission avec `starts_with`. Le code review a révélé un bug de prefix collision. La solution est robuste maintenant mais pourrait devenir complexe avec +50 routes.

**Action :** Surveiller la croissance. Si le mapping devient ingérable, envisager des annotations

## Leçons apprises 📚

### Techniques

1. **Tauri 2 window lifecycle** : `on_window_event(CloseRequested)` avec `api.prevent_close()` pour le hide-to-tray. Le Quit passe par le menu tray, pas par la fermeture de fenêtre.

2. **middleware Axum + permission** : Un seul middleware (`auth_middleware`) gère JWT + permissions. La fonction `required_permission()` mappe path→Permission. `Role::has_permission()` avec `Permission::All` pour bypass admin.

3. **Config store Tauri** : `tauri_plugin_store` via `app.store("settings.json")`. Résolution : env var > store > default. Les valeurs startup nécessitent un redémarrage (`requires_restart`).

4. **JWT sans refresh token** : Refresh silencieux dans le middleware quand `exp - now < 3600s`. Pas de refresh token stocké. Acceptable pour LAN alpha.

5. **Résolution de port** : Le dev runner (`scripts/tauri-dev.ts`) trouve un port libre 3000-3099 et override `TAURI_DEV_PORT`. En prod, le port vient du store.

### Processus

6. **Code review multi-layer** : La combinaison Blind Hunter + Edge Case Hunter + Acceptance Auditor a trouvé des bugs que chaque layer seul aurait manqué.

7. **Story context complet** : Les stories 1.4 et 1.5 avec leur Dev Notes détaillées ont considérablement réduit le temps d'implémentation. La règle "NE PAS deviner" a payé.

8. **Status tracking** : Le sprint-status.yaml est une source de vérité, mais il faut le maintenir à jour immédiatement après chaque étape.

## Action Items

- [ ] **Configurer un framework de test** (Rust + frontend) avant l'Epic 3
  *Priorité : Haute. Raison : les épics Wallet et Ventes manipulent de l'argent.*

- [ ] **Ajouter un guard pour les routes setting sans middleware** — fait (patch review 1.5)
  *Statut : ✅ Résolu*

- [ ] **Automatiser la mise à jour du sprint status** à chaque étape (create-story, dev-story terminée)
  *Priorité : Basse. Raison : processus manuel actuellement, mais simple.*

- [ ] **Documenter l'ordre des checks dans `required_permission()`** comme contract
  *Priorité : Basse. Raison : documenté dans le code, mais pas dans le story file.*

## Prochaines étapes

**Epic 1.5 — Wallet Ledger** (prochaine dans l'ordre logique) :
- Dépend de : Auth (Epic 1) ✅ — prêt
- FR-6, FR-7, FR-8 : identification client par téléphone, wallet ledger append-only, migration
- Prérequis : 3 stories (1-5-1, 1-5-2, 1-5-3)

**Ou Epic 3 — Ventes & Encaissement** (si wallet pas prioritaire) :
- Dépend de : Auth (Epic 1) + Wallet (Epic 1.5)
- FR-9 à FR-15 : produits, commandes, paiements, encaissement, cuisine, ticket
- 6 stories, plus complexe

## Note de clôture

L'Epic 1 est un socle solide. Les 5 stories sont implémentées, reviewées, et toutes les ACs sont satisfaites. Le backend Rust est structuré, l'auth fonctionne, les permissions sont vérifiées, la config est persistée. Le projet est prêt pour les épics métier (Wallet, Ventes).

Prochaine décision : **Wallet Ledger (Epic 1.5)** ou **Ventes (Epic 3)** directement ?
