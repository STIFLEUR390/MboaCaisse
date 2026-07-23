---
baseline_commit: 139feaa
---

# Story 1.5.2: API Wallet — Identification & Solde

Status: done

## Story

As a **caissier**,
I want pouvoir enregistrer un client par telephone et voir son solde,
so that le client peut payer avec son wallet.

## Acceptance Criteria

### AC-1: POST /api/wallet/register

**Given** POST /api/wallet/register avec { phone, name? }
**When** le telephone est valide (9 chiffres)
**Then** un WalletClient est cree avec un UUID v7
**And** la reponse retourne l'ID client et le solde (0 FCFA)

### AC-2: Duplicate phone

**Given** POST /api/wallet/register avec un telephone deja existant
**When** le telephone est deja enregistre
**Then** 409 Conflict est retourne

### AC-3: GET /api/wallet/by-phone/{phone}

**Given** GET /api/wallet/by-phone/{phone}
**When** le client existe
**Then** la reponse retourne { id, phone, name, balance, created_at }

### AC-4: GET /api/wallet/by-phone/{phone} not found

**Given** GET /api/wallet/by-phone/{phone}
**When** le client n'existe pas
**Then** 404 Not Found est retourne

### AC-5: GET /api/wallet/{id}/ledger

**Given** GET /api/wallet/{id}/ledger?limit=50
**When** le client existe
**Then** la reponse retourne les N dernieres entrees du ledger avec solde calcule

## Tasks / Subtasks

- [x] **T1** — Creer api/wallet.rs avec les handlers REST
  - [x] T1.1 POST /api/wallet/register : validation telephone 9 chiffres, creation client, retour { id, phone, name, balance }
  - [x] T1.2 GET /api/wallet/by-phone/{phone} : recherche par telephone, retour avec solde
  - [x] T1.3 GET /api/wallet/{id}/ledger?limit=50 : retourne les dernieres entrees
  - [x] T1.4 Validation : telephone 9 chiffres, phone requis, name optionnel

- [x] **T2** — Integrer les routes dans api/mod.rs
  - [x] T2.1 Monter les routes /api/wallet/*

- [x] **T3** — Verification
  - [x] T3.1 cargo check passe
  - [x] T3.2 Les endpoints sont accessibles

## Dev Notes

### Architecture Compliance

**AD-4 (Payment et Wallet separes)** : Wallet est une ile. L'API wallet expose les endpoints de gestion client et de consultation du ledger. Payment appelle Wallet, jamais l'inverse.

**AD-7 (Traits repository)** : api/wallet.rs utilise `Arc<dyn WalletRepository>` injecte via AppApiState.

**AD-8 (Erreurs 3 couches)** : Les erreurs suivent le format `{ error, code }`. `DuplicatePhone` → 409, `NotFound` → 404, `InvalidValue` → 422.

**AD-2 (Append-only financier)** : L'API wallet ne fait qu'appeler append_entry qui gere deja l'atomicite (story 1.5.1).

### Pre-requis (deja implementes)

- `domain/wallet.rs` — WalletClient, WalletLedgerEntry, WalletRepository trait ✅
- `db/wallet_ledger.rs` — DbWalletRepository implementation complete ✅
- `AppApiState` — contient deja `wallet_repo: Arc<dyn WalletRepository>` ✅

### Fichiers a modifier

- `src-tauri/src/api/wallet.rs` — Remplacer le stub par les handlers REST
- `src-tauri/src/api/mod.rs` — Verifier que les routes /api/wallet/* sont montees

## Fichiers modifies

- `src-tauri/src/api/wallet.rs` — Implementation complete
- `src-tauri/src/api/mod.rs` — Routes ajoutees
