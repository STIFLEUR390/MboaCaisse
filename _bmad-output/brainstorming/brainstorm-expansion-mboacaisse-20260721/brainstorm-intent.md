# Brainstorm Intent — MboaCaisse Expansion

## 1. Core Discovery

Phone number = universal key. No login, card, or app needed. Wallet + loyalty + recognition + order history anchored to one identifier. The client wallet is the single source of truth (local SQLite). Mobile Money (MoMo) and cash are *supply channels into the wallet*, not payment endpoints. Payment always debits the wallet, never an external rail directly.

QR code per table (URL-encoded table ID, server-generated, printed on paper) eliminates table selection UX. Admin fallback for manual table entry.

## 2. Architecture Decisions

- **Wallet**: multi-source (Cash, MoMo, Gift, Cashback, Transfer). Balance never stored — always `SUM(amount) WHERE client_id` over wallet_ledger. Zero drift by construction.
- **wallet_ledger**: append-only table (`INSERT`-only, never `UPDATE`/`DELETE`). Every 5min backup. P0.
- **Payment gate**: payment *before* order validation. Wallet check immediate; insufficient-balance message on failure. Negative wallet = admin choice, not default.
- **No sync group for 3 years**: wallet per instance. Client carries different balance per establishment. Acceptable.
- **Feature flags via Ed25519 licensing** (existing P4 infra). No branches, no forks. 6 bundles = same binary, flag-gated UI. API stays business-agnostic (`POST /api/orders` = "client wants something"; flags control surface).
- **Cashback**: auto 5% on every wallet-paid order (zero config). Optional progressive threshold (3%→5%→8% by order count) — pure game design, no UI.
- **Referral**: phone number linked at registration, 1000 FCFA credited to both wallets.

## 3. Priority Tier

| Tier | Item | Rationale |
|------|------|-----------|
| **P0** | wallet + wallet_ledger (append-only) | Core — rest is cosmetic without it |
| **P0** | Async impression queue + retry + fallback ticket numérique (screen) | Credibility-critical for pros |
| **P1** | mDNS customisable (chezbob.local at setup) | Nice-to-have, admin fallback exists |
| **P2** | Multi-establishment sync group (future) | Not needed for 3 years |

## 4. Product Bundles (Feature Flags)

| Bundle | Flags | Target |
|--------|-------|--------|
| **Mboa Cash** | wallet, cash, loyalty, basic reports | Corner shop / épicerie quartier |
| **Mboa Resto** | Cash + MoMo, pre-order, kitchen display, tables | Bar / restaurant |
| **Mboa Stock** | Resto + supplier inventory, threshold alerts, quotes, multi-warehouse | Epicerie + resto + retail |
| **Mboa Traiteur** | Resto + date/time pickup scheduling, deposits, prod calendar, recipe sheets | Catering / traiteur |
| **Mboa Hôtel** | Resto + check-in/out, minibar, stay invoice | Small hotel / auberge |
| **Mboa Market** | Stock + barcode, rotating inventory, supplier orders, auto margin | Small supermarket / supérette |

Same codebase. Fork zero.

## 5. Key Nuances

- **Client without phone**: internal ID (CLI-XXXX), wallet attached, no pre-order but fully functional for cash/over-the-counter.
- **Security deposit**: optional feature, default OFF. Setup warning (regulatory grey zone for financial services).
- **Ledger migration**: retro-compatible. Create ledger table + replay historical paid orders. Empty ledger = fatal crash without this.
- **API business-agnostic**: `orders`, `payments`, `products` — same endpoints across bundles. UI layer decides what's shown.
- **Public menu**: 5 screens (landing QR, menu, cart, phone identification, confirmation). Same logic across bundles — traiteur QR takeaway vs resto QR table = different view, same flow.
- **Impression queue**: async with retry. Numeric ticket on client screen as fallback when printer offline.

## 6. Resilience Triangle

```
    Wallet
    /     \
Ledger — Impression Queue
```

Three independent subsystems. Each survives if another is down:
- Wallet works without printer (order taken, ticket queued).
- Printer works without wallet sync (local queue, retries).
- Ledger survives wallet crash (append-only, replay on restart).
