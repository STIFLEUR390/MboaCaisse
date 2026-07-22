# Deferred Work

## Deferred from: code review of 1-1-structure-rust-layered-migrations-initiales (2026-07-22)

- Missing Cancelled status in OrderStatus — pas de statut d'annulation pour les commandes, mais le scope de la story 1.1 ne couvre pas le cycle de vie complet des commandes. [`src-tauri/src/domain/order.rs:10`]
- seed::run() est un placeholder no-op — la spec le documente explicitement comme placeholder pour story 1.3/1.5. [`src-tauri/src/db/seed.rs:15`]
