---
title: "Product Brief: MboaCaisse"
status: reviewed
created: 2026-07-21
updated: 2026-07-21
---

# Product Brief: MboaCaisse

## Executive Summary

MboaCaisse is a POS and business management server for bars, restaurants, and grocery stores in Francophone Africa. It runs on a single PC inside the establishment and is accessible from any device on the local network — phones, tablets, other PCs — all via browser, with no internet required.

Unlike cloud-based alternatives that break when connectivity drops, or expensive imported terminals that cost 500K+ FCFA, MboaCaisse is offline-first, affordable, and designed for the African market: wallet with Mobile Money integration, thermal printing, loyalty via phone number, and license-based pricing with feature gating. The product is built on a Rust/Tauri 2 desktop app with an embedded Axum HTTP server, SQLite database, and a Nuxt 4 frontend — all served locally over LAN with mDNS discovery.

This is the only solution in its niche combining offline-first POS, client wallet, Mobile Money top-up, and cryptographic licensing in a single package, targeting a market of hundreds of thousands of small food and retail businesses across Francophone Africa.

## The Problem

Small food and retail businesses in Francophone Africa operate with three inadequate options:

**Cash + notebook.** No tracking, no reporting, no insight. The owner cannot know if the day was profitable, which products sell, or whether the cashier is honest. Reconciliation is manual, theft is easy, and growth decisions are guesses.

**Cloud POS apps.** Kippa, Odyera, and similar cloud-first apps require a stable internet connection. In markets where connectivity is unreliable or expensive, the POS stops working mid-service. A restaurant cannot tell customers "wait for the internet to come back" — it loses revenue and credibility.

**Imported terminals.** PAX, Ingenico, and other hardware terminals cost upwards of 500,000 FCFA, require specialized maintenance, and do not include inventory management, client wallets, or loyalty — they are single-purpose payment devices. They are designed for card-present payments, which are rare in this market compared to Mobile Money and cash.

No existing solution combines offline-first reliability, Mobile Money integration, client wallet, inventory management, local thermal printing, and an affordable license model — in a product that runs on a standard PC already present in the establishment.

## The Solution

MboaCaisse is a single-install desktop application (Tauri 2) that turns a PC into a complete POS server for the establishment:

- **LAN-first.** The app starts an Axum HTTP server on the local network, accessible from any browser (PC, tablet, smartphone) at `http://mboacaisse.local`. The server serves both the Nuxt frontend and the REST API. A Tauri native window provides the primary cashier/admin interface.

- **Offline-first.** No internet is required for daily operation. SQLite database is local, all API calls stay on the LAN, and the app works even if the ISP is down for days.

- **Client wallet system.** Every customer is identified by phone number. The wallet aggregates multiple sources — Mobile Money top-up, cash deposit, gift credit, cashback — with the balance computed as `SUM(amount)` from an append-only ledger, never stored as a single mutable field. Payment is deducted from the wallet before the order goes to the kitchen.

- **Mobile Money integration.** MoMo (Orange Money, MTN MoMo, etc.) serves as an external funding source for the wallet — customers deposit via MoMo, then spend from their wallet in-store. This bridges the gap between digital money and on-premise cash transactions. Direct MoMo payment (synchronous webhook) is deferred to P2 pending market demand.

- **Loyalty built-in.** Cashback accrues automatically on every purchase (3% → 5% → 8% by spending tiers). No cards, no apps, no stamps — just the phone number. Referral credits (1,000 FCFA each) are applied to both parties' wallets. The phone number is the universal loyalty identifier.

- **Product bundles via feature gating.** Instead of separate forks or builds, a single binary serves 6 editions via Ed25519-signed license flags: Cash, Resto, Stock, Traiteur, Hotel, Market. Each bundle unlocks a subset of features (pos, inventory, kitchen, planning, rooms, barcode scanning, etc.).

- **Thermal printing.** Native ESC/POS generation from Rust writes directly to USB or network thermal printers — no browser print dialog, no popup blocking, no fragile `window.print()`.

- **mDNS discovery.** The server announces itself as `mboacaisse.local` on the LAN. Clients discover it with no IP configuration.

- **Public menu via QR.** Walk-in customers scan a QR code on their table (or at the counter) to browse the menu, check their wallet, and place orders — no app install, no device pairing.

## What Makes This Different

| Dimension | MboaCaisse | Cloud POS | Imported Terminal |
|-----------|-----------|-----------|-------------------|
| Requires internet | No | Yes | No (device-dependent) |
| Client wallet + MoMo | Native | Rare | No |
| Loyalty via phone | Built-in (cashback tiers, referral) | Requires separate app | No |
| Runs on existing PC | Yes | Yes | No (dedicated hardware) |
| Cost model | License (perpetual + 12mo updates) | Monthly subscription | 500K+ FCFA upfront |
| Multiple roles (admin, cashier, server, kitchen) | Yes | Varies | No (single role) |
| Offline thermal printing | Native Rust (ESC/POS) | Web print (fragile) | Hardware-dependent |
| Multi-device (tables, phones, PCs) | Any browser on LAN | Needs internet per device | Terminal only |

**Unfair advantage**: The combination of offline-first architecture, Mobile Money wallet integration, and cryptographic feature gating — all in a single Rust binary — is a moat in execution, not in technology. It requires deep domain knowledge of the African small-business context that cloud-first SaaS companies from outside the continent rarely invest in.

## Who This Serves

**Primary — Establishment owners** in Francophone Africa (Cameroun, Côte d'Ivoire, Sénégal, Mali, RDC, etc.) who operate bars, restaurants, small grocery stores. They own a PC (often old, shared), their staff has basic digital literacy, and they lose money to cash leakage, lack of inventory visibility, and the inability to accept digital payments gracefully.

**Secondary — Staff roles** within the establishment: cashiers (process payments, print receipts), servers/take-order staff (use tablets or phones to take orders), kitchen staff (receive orders on a dedicated display), and managers/owners (view reports, adjust pricing, manage stock).

**Tertiary — Potential investors and alpha clients** who need to evaluate the product's market fit, technical depth, and commercialization readiness.

## Success Criteria

**User signals:**
- An establishment runs for 30 days without requiring internet access
- Wallet adoption rate > 60% of walk-in customers within 3 months
- Owner can produce a daily sales report at any time
- Average transaction time (order → payment → receipt) ≤ 90 seconds
- Loyalty cashback redemption rate > 20%
- Zero complaints about wallet data loss (ledger mandatory before any deployment)

**Business signals:**
- 3 alpha clients in production within 6 months of MVP launch
- 50 active establishments within 2 years
- License server operational: activation, verification, and feature gating pipeline
- Distribution-ready bundles (Linux .deb/.AppImage, Windows .msi)
- Support infrastructure: backup/restore workflow, diagnostic tooling

## Scope

**In (MVP — P0 + P1):**
- Axum server embedded in Tauri, serving frontend + API on LAN
- SQLite with migrations, authentication (email/password, JWT, 4 roles)
- Product CRUD, categories, order lifecycle, payment processing
- Client wallet system with append-only ledger and Mobile Money integration
- Basic reporting (daily sales, per-cashier totals)
- Backup/restore (manual + automatic)
- mDNS discovery (`mboacaisse.local`)
- Thermal printing (ESC/POS over USB)
- License verification with Ed25519 (offline-first)
- One bundle (Mboa Cash)

**Explicitly out (MVP):**
- Multi-instance sync (wallet per instance is acceptable for V1)
- Barcode scanning (P2.3, post-MVP)
- Auto-updater (P2.4, post-MVP)
- Second display / customer-facing screen (P2.2, post-MVP)
- WebSocket real-time (P2.7, post-MVP)
- Bundles beyond Cash (Resto, Stock, etc. — feature gating exists, but gated features lock nothing until implemented)
- Mobile apps for Android/iOS (browser-only on mobile)

## Vision

Within 2-3 years, MboaCaisse becomes the default POS and management platform for small food and retail businesses across Francophone Africa — the digital backbone that works whether the internet is up or down.

The wallet ecosystem extends beyond individual establishments: a customer's phone number carries their loyalty balance, referral credits, and purchasing history to any MboaCaisse-equipped business. Feature bundles diversify across segments (hotels, bakeries, butcher shops, mini-markets). The licensing platform matures into a self-serve marketplace where businesses buy, upgrade, and manage their installations online — but the POS itself never depends on it.

The technical trajectory anticipates scaling pains ("bugs of success"): the wallet_ledger hardens to append-only with verified backups; the print queue becomes async with retry and digital fallback; network discovery becomes customizable; and the product earns its place as infrastructure, not just software.
