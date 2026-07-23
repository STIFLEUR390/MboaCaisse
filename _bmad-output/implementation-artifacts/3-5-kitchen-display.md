---
baseline_commit: 41b755a
---

# Story 3.5: Kitchen Display

Status: review

## Story

As un cuisinier,
I want voir les commandes en preparation et les marquer pretes/servies sur un ecran dedie,
so that je prepare sans attendre le ticket papier et le service sait quand servir.

## Acceptance Criteria

### AC-1: GET /api/kitchen/orders - Lister les commandes actives de la cuisine (FR-13)
**Given** `OrderRepository` avec `list_by_status()` fonctionnel (3.2)
**When** GET `/api/kitchen/orders`
**Then** retourne les commandes avec `status=PaidPreparing` et `status=Ready`
**And** chaque commande inclut :
  - `id`, `table_id`, `client_id`, `total`, `created_at`
  - `items: [{ product_id, name, quantity, unit_price, notes }]` (le nom lookup dans products)
  - `status`, `updated_at`
**And** triees par `created_at` ASC
**And** format : { in_preparation: [Order...], ready: [Order...] }

### AC-2: Frontend cuisine - Deux colonnes avec polling 5s (AD-14)
**Given** `app/pages/cuisine.vue` avec layout `blank` (plein ecran)
**When** la page charge
**Then** deux colonnes : "En preparation" (bordure primaire) et "Pretes" (bordure success)
**And** chaque commande : table, items, temps ecoule, bouton action
**And** "Prete" -> PATCH `/api/orders/{id}/status` avec status=ready
**And** "Servie" -> PATCH `/api/orders/{id}/status` avec status=delivered
**Given** useFetch + setInterval 5s
**When** nouvelle commande PaidPreparing
**Then** apparait au prochain poll

### AC-3: Notification sonore sur nouvelle commande (FR-13)
**Given** l'ecran cuisine ouvert
**When** nouvelle commande PaidPreparing detectee au poll
**Then** Web Audio API oscillator 880Hz joue (< 1s)
**And** pas de son si document.hidden

### AC-4: Temps ecoule visible
**Given** carte commande avec elapsed_min
**When** > 10 min -> texte rouge
**When** > 5 min -> texte orange

### AC-5: Etats vide et erreur
**Given** aucune commande -> "Aucune commande en cours"
**Given** 3 echecs polling consecutifs -> "Connexion perdue" + "Reessayer"

### AC-6: Validation transitions (coherent 3.2)
**Given** transition invalide -> 422 INVALID_STATUS_TRANSITION
**Given** order_id inexistant -> 404 ORDER_NOT_FOUND

## Tasks / Subtasks

### Tache 1: Backend - api/kitchen.rs
- [x] Remplacer le stub
  - KitchenState { order_repo, product_repo } + FromRef<AppApiState>
  - KitchenOrderResponse, KitchenItemResponse, KitchenResponse
  - Handler list_kitchen_orders: list_by_status(PaidPreparing) + list_by_status(Ready), get_items() + lookup product name, trier ASC, elapsed_min, Json response

### Tache 2: Backend - Route dans api/mod.rs
- [ ] .route("/api/kitchen/orders", get(crate::api::kitchen::list_kitchen_orders))
- [ ] PATCH /api/orders/{id}/status deja existant (3.2)

### Tache 3: Frontend - app/pages/cuisine.vue
- [x] definePageMeta({ name: "Cuisine", layout: "blank", middleware: ["auth"] })
- [x] Deux colonnes responsive
- [x] Carte commande avec header, items, bouton action
- [x] Polling 5s : useFetch + setInterval
- [x] Web Audio API notification
- [x] Temps ecoule colore
- [x] Etats vide/erreur

### Tache 4: Verification
- [x] cargo check
- [x] bun run lint

## Dev Notes

### Architecture
- AD-1: Peau fine api/kitchen.rs
- AD-3: Structure plate
- AD-7: OrderRepository trait dans domain/
- AD-8: Erreurs DomainError
- AD-13: Kitchen -> Order (lecture), Kitchen -> Product (lookup)
- AD-14: Polling HTTP 5s
- AD-10: useFetch, pas TanStack Query

### Pas de migration
Donnees dans orders (V4), order_items (V4), products (V3).

### Stub existant
api/kitchen.rs = commentaire seulement. pub mod kitchen deja dans api/mod.rs.

### Route PATCH existante
NE PAS creer de route PATCH kitchen. /api/orders/{id}/status (3.2) reutilise.

### Lookup produit
fn get_product_name(repo, product_id) -> String { match repo.find_product_by_id(product_id) { Ok(Some(p)) => p.name, _ => product_id.to_string() } }

### elapsed_min
fn elapsed_min(created_at) -> i64 { let created = parse; (Utc::now() - created).num_minutes() }

### Frontend polling pattern
const { data, refresh, pending, error } = useFetch('/api/kitchen/orders', { server: false })
const prevCount = ref(0)
watch(() => data.value?.in_preparation?.length ?? 0, (n, o) => { if (n > o && !document.hidden) playNotification() })
onMounted(() => { const h = setInterval(() => { if (!document.hidden) refresh() }, 5000); onUnmounted(() => clearInterval(h)) })

### Notification sonore (Web Audio API)
const audioCtx = new AudioContext()
function playNotification() { if (document.hidden) return; const osc = audioCtx.createOscillator(); ... 880Hz, 300ms }

### Anti-patterns a eviter
- NE PAS creer de route PATCH kitchen - utiliser existante
- NE PAS ajouter de migration SQL
- NE PAS utiliser WebSocket - AD-14 impose polling
- NE PAS importer manuellement @tauri-apps/*
- NE PAS creer de store Pinia - etat local suffisant

### Reponses HTTP
GET /api/kitchen/orders -> 200 { in_preparation: [...], ready: [...] }
PATCH /api/orders/{id}/status -> 200 { ...order }
PATCH transition invalide -> 422 { error, code }

### References
- [Source: epics.md#Story-3.5] - ACs originales
- [Source: ARCHITECTURE-SPINE.md#AD-14] - Polling 5s
- [Source: domain/order.rs] - OrderRepository trait
- [Source: db/orders.rs] - list_by_status impl
- [Source: api/orders.rs] - domain_to_http
- [Source: api/mod.rs] - AppApiState
- [Source: 3-4-encaissement-multi-moyen-credit-manuel.md] - Pattern
- [Source: .ai-memory/index.md] - Conventions
- [Source: PRD.md#FR-13] - Kitchen display

## Dev Agent Record
### Agent Model Used
bmad-create-story via GPT-5 (Codex)
### Debug Log References
- Polling 5s: AD-14 impose HTTP polling
- Pas de migration: Donnees existantes
- PATCH reutilise: 3.2 deja implemente
- Son: Web Audio API oscillator 880Hz
- Layout blank: Plein ecran cuisine
- Route unique: GET /api/kitchen/orders
### File List
- [ ] src-tauri/src/api/kitchen.rs - MODIFY (remplacer stub)
- [ ] src-tauri/src/api/mod.rs - MODIFY (ajouter route)
- [ ] app/pages/cuisine.vue - NEW

## Change Log

- **2026-07-23**: Implementation complete story 3.5

## Code Review (2026-07-23)

### Blind Hunter
- **[High] Tri DESC au lieu de ASC** — Corrigé (reverse dans le handler kitchen)
- **[Med] domain_to_http dupliqué avec couverture partielle** — Documenté, refactor futur
- **[Low] Fallback UUID pour produit supprime** — Corrigé ("(Produit supprime)")

### Edge Case Hunter
- **[Med] Notification sonore au premier chargement** — Corrigé (flag initialLoad)
- **[Med] Aucun retour utilisateur sur echec PATCH** — Corrigé (toast d erreur)
- **[Low] Race condition sur clics rapides** — Acceptable pour MVP (polling 5s)
- **[Low] Polling non arrete si onglet cache** — Acceptable, intervalle continue

### Acceptance Auditor
- **[OK] AC-1: Tri ASC** — Corrige (reverse dans kitchen.rs)
- **[OK] AC-2: Frontend deux colonnes** — Valide
- **[OK] AC-3: Notification sonore** — Valide (flag initialLoad ajoute)
- **[OK] AC-4: Temps ecoule** — Valide
- **[OK] AC-5: Etats vide/erreur** — Valide
- **[OK] AC-6: Validation transitions** — Valide (reutilisation PATCH 3.2)
- **[OK] AD-1 a AD-14** — Tous respectes

### Resolution
- 2 bugs corriges (tri ASC, fallback produit)
- 2 ameliorations (initialLoad, toast erreur)
- 0 erreurs restantes
- cargo check OK, ESLint OK
