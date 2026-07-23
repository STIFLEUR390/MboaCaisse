# Edge Case Hunter — Code Review Prompt

You are an **Edge Case Hunter** reviewer. Your mission is to walk every branching path and boundary condition in the diff below. Find ALL corner cases, failure modes, state inconsistencies, race conditions, and unexpected behaviors.

**Focus areas:**
- Empty/null/missing input handling
- Boundary values (zero, negative, overflow)
- Concurrent access and race conditions
- Partial failure recovery
- State machine transitions (valid and invalid)
- DB transaction rollback paths
- Error propagation and masking
- Time-of-check/time-of-use (TOCTOU) races
- Resource exhaustion (DB connections, memory)

**Be exhaustive — if there's a way to break it, find it.**

## Files to review

### src-tauri/src/api/payments.rs (428 lines)
The `process_payment` handler handles:
- PaymentMethod::Wallet — atomic transaction with BEGIN IMMEDIATE
- PaymentMethod::Cash — simple payment
- Other methods → 501 Not Implemented

Key code paths to trace:
1. Parse method → invalid method string → ?
2. Order lookup → order not found → ? order in wrong status → ?
3. Wallet payment: missing client_id → empty client_id → client not found →
4. Wallet payment: balance check → insufficient balance with wallet_negative=false →
5. Wallet payment: balance check → insufficient balance with wallet_negative=true →
6. Wallet payment: BEGIN IMMEDIATE fails → ? any SQL INSERT fails → ?
7. Wallet payment: COMMIT fails after successful writes → ?
8. Wallet payment: ROLLBACK path → what happens if ROLLBACK also fails?
9. Cash payment: payment_repo.create() succeeds but order_repo.update_status() fails → orphaned payment
10. Concurrent payments on same wallet (AC-5) → does BEGIN IMMEDIATE prevent races?
11. Config::load() inside the handler → what if AppHandle uninitialized?
12. `total` vs `order.total` — is order consumed by the closure?

### src-tauri/src/db/payments.rs (120 lines)
- create() → what if payment.validate() fails? → error type?
- find_by_id() → no rows → Ok(None) or error?
- row_to_payment() — can method parsing fail from DB? → error mapping?
- Connection errors propagation

### src-tauri/src/settings.rs — wallet_negative config
- Load from store → what if store has non-boolean value?
- Default when store inaccessible → already handled?

### src-tauri/migrations/V5__payments.sql
- CHECK constraint on method → insert with invalid method → ?
- REFERENCES orders(id) — order might be deleted → ?
- amount > 0 CHECK — what about zero or negative?

Please produce a thorough edge case analysis of ALL files.
