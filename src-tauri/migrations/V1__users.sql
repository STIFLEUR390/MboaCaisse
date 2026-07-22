CREATE TABLE IF NOT EXISTS users (
    id          TEXT PRIMARY KEY,
    email       TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    name        TEXT NOT NULL DEFAULT '',
    role        TEXT NOT NULL DEFAULT 'caissier',
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);
