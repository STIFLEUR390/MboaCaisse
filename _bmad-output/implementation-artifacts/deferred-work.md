# Deferred Work

## Deferred from: code review of 1-1-structure-rust-layered-migrations-initiales (2026-07-22)

- Missing Cancelled status in OrderStatus — pas de statut d'annulation pour les commandes, mais le scope de la story 1.1 ne couvre pas le cycle de vie complet des commandes. [`src-tauri/src/domain/order.rs:10`]
- seed::run() est un placeholder no-op — la spec le documente explicitement comme placeholder pour story 1.3/1.5. [`src-tauri/src/db/seed.rs:15`]

## Deferred from: code review of story 1.2 (2026-07-22)

- **CorsLayer::permissive() en production** [`server.rs:39`] — Sera restreint quand l'auth sera implémentée (story 1.3)
- **resolve_port() ne lit pas le Tauri store** [`lib.rs:175`] — Le bridge Pinia + store sera fait en story 1.4
- **0.0.0.0 expose l'API sur toutes les interfaces** [`server.rs:42`] — La sécurité réseau viendra avec l'auth JWT (story 1.3)

## Deferred from: code review of story 1.5 (2026-07-22)

- Race condition DELETE dernier admin [users.rs:241-250] — deux suppressions admin simultanées peuvent passer. Scénario improbable en LAN alpha.
- Aucun test ajouté — le projet n'a pas de framework de test configuré.
- Pas de typage Zod pour les appels API frontend [admin/users.vue] — typage manuel suffit pour l'alpha.
- Pas de bouton "Réessayer" sur erreur chargement [admin/users.vue] — UX mineure, l'utilisateur peut recharger la page.

## Deferred from: code review of story 1.5.1 (2026-07-23)
- Pas de validation du format téléphone (9 chiffres) — sera dans l'API (story 1.5.2)
- get_balance retourne 0 pour client inexistant — acceptable pour l'alpha
- Aucun test ajouté — pas de framework de test configuré

## Deferred from: code review of 3-1-crud-produits-categories (2026-07-23)

- DELETE /api/categories makes 3-4 DB queries instead of one — optimisation, pas bloquant
- Race condition in DELETE category guards (non-atomic check-then-delete) — pas de concurrence en alpha
- update_category only guards self-parent, not deep cycles — non exploitable avec API actuelle
- domain_to_http doesn't return spec error codes INVALID_NAME/INVALID_PRICE — spec secondaire
- delete_category returns NOT_FOUND instead of CATEGORY_NOT_FOUND on race path — edge case
- search_products uses LIKE without escaping % and _ wildcards — pas exposé via API

## Deferred from: code review of 3-2-cycle-de-vie-commande (2026-07-23)

- N+1 queries in list_orders — batch load items per order_ids
- map_order_row converts DomainError to rusqlite Error — pre-existing pattern across all db mappers
- remove_item no PendingPayment guard in DB layer — defense-in-depth, already guarded at API
- chrono_now/uuid_v7 duplicated in db/orders.rs + api/orders.rs — extract to shared util
- list_all without LIMIT — add pagination
