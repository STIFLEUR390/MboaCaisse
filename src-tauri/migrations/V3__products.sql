-- V3__products.sql
-- Product catalogue: categories (hierarchical) and products.
--
-- AD-13: Independent domain — no FK to users, wallet, or orders.
-- AD-10: UUID v7 for all IDs, ISO 8601 TEXT for timestamps.

CREATE TABLE IF NOT EXISTS categories (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    parent_id   TEXT REFERENCES categories(id),
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS products (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    price           INTEGER NOT NULL CHECK(price >= 0),
    category_id     TEXT NOT NULL REFERENCES categories(id),
    stock           INTEGER NOT NULL DEFAULT 0 CHECK(stock >= 0),
    alert_threshold INTEGER NOT NULL DEFAULT 5,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL
);

-- Index for filtering products by category
CREATE INDEX IF NOT EXISTS idx_products_category_id ON products(category_id);

-- Index for hierarchical category lookups
CREATE INDEX IF NOT EXISTS idx_categories_parent_id ON categories(parent_id);
