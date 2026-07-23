# Acceptance Auditor — Code Review Prompt

You are an **Acceptance Auditor**. Review the provided diff against the spec/story file. Check for: violations of acceptance criteria, deviations from spec intent, missing implementation of specified behavior, contradictions between spec constraints and actual code.

## Story File

**Path:** `_bmad-output/implementation-artifacts/3-3-payment-gate.md`

## Acceptance Criteria to Verify

### AC-1: POST /api/payments — Paiement wallet avec débit
Check:
- [ ] Vérification du solde client AVANT débit ?
- [ ] wallet_ledger INSERT avec type='payment', amount=-total, reference=order_id ?
- [ ] payments INSERT avec method='wallet', amount=total ?
- [ ] Transition de la commande vers PaidPreparing ?
- [ ] Réponse `{ status: "paid", new_balance, payment_id }` ?
- [ ] Toute l'opération est atomique (BEGIN → vérifications → INSERT ledger → INSERT payment → UPDATE order → COMMIT) ?

### AC-2: Solde insuffisant (wallet négatif désactivé)
Check:
- [ ] wallet_negative=false par défaut ?
- [ ] balance < total → 422 avec INSUFFICIENT_BALANCE ?
- [ ] Commande reste en PendingPayment, aucune écriture ledger ?

### AC-3: Paiement cash
Check:
- [ ] INSERT dans payments avec method='cash' ?
- [ ] Commande passe à PaidPreparing ?
- [ ] Aucune écriture dans wallet_ledger ?

### AC-4: Wallet négatif activé
Check:
- [ ] wallet_negative=true permet le débit même si solde négatif ?
- [ ] Le setting est chargé depuis tauri_plugin_store ?

### AC-5: Atomicité — pas de race condition
Check:
- [ ] BEGIN IMMEDIATE utilisé (pas DEFERRED) ?
- [ ] Toutes les écritures dans la même transaction ?
- [ ] ROLLBACK en cas d'erreur ?

### AC-6: Validation des préconditions
Check:
- [ ] Commande pas en PendingPayment → 422 INVALID_ORDER_STATUS ?
- [ ] order_id inexistant → 404 ORDER_NOT_FOUND ?
- [ ] client_id inexistant → 422 CLIENT_NOT_FOUND ?
- [ ] wallet sans client_id → 422 VALIDATION_ERROR ?

## Architecture Decisions to Verify

- [ ] AD-2: wallet_ledger INSERT-only (pas d'UPDATE/DELETE) — respecté ?
- [ ] AD-4: Payment appelle Wallet, Wallet n'appelle jamais Payment — respecté ?
- [ ] AD-7: PaymentRepository trait dans domain/, impl dans db/ — respecté ?
- [ ] AD-8: Erreurs 3 couches — DbError ne fuit pas dans api/ ?
- [ ] AD-13: Graphe dépendances — Payment → Order+Wallet (pas de dépendances circulaires) ?

## Other Files

Also review:
- **src-tauri/src/db/payments.rs** — implémentation du trait PaymentRepository
- **src-tauri/src/settings.rs** — wallet_negative config
- **src-tauri/src/api/settings.rs** — wallet_negative API endpoint
- **src-tauri/migrations/V5__payments.sql** — migration

Please produce a detailed finding list with: one-line title, which AC/constraint it relates to, and evidence from the diff.
