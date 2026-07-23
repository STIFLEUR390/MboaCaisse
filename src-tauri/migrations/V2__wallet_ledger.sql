-- V2__wallet_ledger.sql
-- Wallet clients and append-only ledger tables.
--
-- AD-2: wallet_ledger is INSERT-only. Balance = SELECT SUM(amount).
--       No UPDATE/DELETE allowed — enforced by triggers below.

CREATE TABLE IF NOT EXISTS wallet_clients (
    id          TEXT PRIMARY KEY,
    phone       TEXT NOT NULL UNIQUE,
    name        TEXT NOT NULL DEFAULT '',
    referrer_id TEXT,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS wallet_ledger (
    id          TEXT PRIMARY KEY,
    client_id   TEXT NOT NULL,
    type        TEXT NOT NULL,
    amount      INTEGER NOT NULL,
    reference   TEXT,
    description TEXT,
    created_at  TEXT NOT NULL,
    FOREIGN KEY (client_id) REFERENCES wallet_clients(id)
);

-- Enforce append-only: prevent UPDATE on wallet_ledger
CREATE TRIGGER IF NOT EXISTS prevent_wallet_ledger_update
BEFORE UPDATE ON wallet_ledger
BEGIN
    SELECT RAISE(ABORT, 'wallet_ledger is append-only: UPDATE forbidden');
END;

-- Enforce append-only: prevent DELETE on wallet_ledger
CREATE TRIGGER IF NOT EXISTS prevent_wallet_ledger_delete
BEFORE DELETE ON wallet_ledger
BEGIN
    SELECT RAISE(ABORT, 'wallet_ledger is append-only: DELETE forbidden');
END;

-- Index for balance queries (SUM by client_id)
CREATE INDEX IF NOT EXISTS idx_wallet_ledger_client_id ON wallet_ledger(client_id);

-- Index for phone lookups
CREATE INDEX IF NOT EXISTS idx_wallet_clients_phone ON wallet_clients(phone);
