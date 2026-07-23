-- V4__orders.sql
-- Order lifecycle: orders and order_items tables.
--
-- AD-13: Order depends on Catalog (product_id FK conceptual, no FK constraint
--         to avoid blocking product deletion). Referential integrity is
--         enforced at the application layer via ProductRepository lookups.
-- AD-2:  order_items is mutable (add/remove items allowed in PendingPayment).
--         Once past PendingPayment, mutability is gated by the domain layer.
--         Financial mutation is NOT in this table -- wallet_ledger (V2) is
--         the append-only financial record.

CREATE TABLE IF NOT EXISTS orders (
    id          TEXT PRIMARY KEY,
    table_id    TEXT,
    client_id   TEXT,
    status      TEXT NOT NULL DEFAULT 'pending_payment',
    total       INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS order_items (
    id          TEXT PRIMARY KEY,
    order_id    TEXT NOT NULL REFERENCES orders(id),
    product_id  TEXT NOT NULL,
    quantity    INTEGER NOT NULL CHECK(quantity > 0),
    unit_price  INTEGER NOT NULL,
    notes       TEXT,
    created_at  TEXT NOT NULL
);

-- Index for retrieving items by order
CREATE INDEX IF NOT EXISTS idx_order_items_order_id ON order_items(order_id);

-- Index for filtering orders by status (kitchen display, etc.)
CREATE INDEX IF NOT EXISTS idx_orders_status ON orders(status);

-- Index for lookups by table
CREATE INDEX IF NOT EXISTS idx_orders_table_id ON orders(table_id);
