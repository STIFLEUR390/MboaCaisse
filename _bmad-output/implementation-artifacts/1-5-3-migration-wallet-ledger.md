---
baseline_commit: 139feaa
---

# Story 1.5.3: Migration Wallet Ledger

Status: done

## Story

As a **admin**,
I want que les commandes payees avant l'implementation du ledger soient rejouees dans wallet_ledger,
so that le solde des clients existants est correct des l'activation.

## Acceptance Criteria

### AC-1: Migration des commandes payees

**Given** des commandes payees dans la table orders (future Epic 3)
**When** le script de migration s'execute
**Then** une ligne INSERT avec type='migration' est creee par commande payee
**And** les clients concernes ont un wallet cree si inexistant

### AC-2: Idempotent

**Given** le script est re-execute
**When** des lignes avec type='migration' existent deja
**Then** aucune ligne en double n'est creee
**And** les lignes migration sont ignorees

### AC-3: Aucune commande = pas d'erreur

**Given** aucune commande payee n'existe avant le ledger (ou table orders pas encore creee)
**When** le script s'execute
**Then** aucune ligne migration n'est creee, pas d'erreur

## Implementation

- Cree `src-tauri/src/db/migration_wallet.rs` avec fonction `run_migration()`
- Verifie si la table `orders` existe via `sqlite_master`
- Si orders table existe et contient des commandes payees sans migration correspondante :
  - Pour chaque commande payee : cree un wallet client si necessaire + INSERT dans wallet_ledger
- Idempotent : verifie si des entrees `type='migration'` existent deja
- Execute dans `lib.rs` apres le seed existant
