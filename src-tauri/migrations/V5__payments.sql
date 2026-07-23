-- V5__payments.sql
-- Payment transactions: logs every payment attempt, method, and status.
--
-- AD-4: Payment calls Wallet (wallet_ledger tracks the actual financial movement).
--       This table is the audit log *after* the ledger entry is committed.
-- AD-2: wallet_ledger is the append-only financial record.
--       payments table is a secondary log for UI display & reporting.
-- AD-13: Payment depends on Order (order_id FK) and Wallet (client_id).

CREATE TABLE IF NOT EXISTS payments (
    id          TEXT PRIMARY KEY,
    order_id    TEXT NOT NULL REFERENCES orders(id),
    method      TEXT NOT NULL CHECK(method IN ('wallet', 'cash', 'momo', 'split')),
    amount      INTEGER NOT NULL CHECK(amount > 0),
    client_id   TEXT,
    reference   TEXT,
    created_at  TEXT NOT NULL
);

-- Index for retrieving payments by order (order detail view)
CREATE INDEX IF NOT EXISTS idx_payments_order_id ON payments(order_id);

-- Index for retrieving payments by client (wallet history)
CREATE INDEX IF NOT EXISTS idx_payments_client_id ON payments(client_id);
