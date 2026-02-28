CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE IF NOT EXISTS admin (
    id BIGINT PRIMARY KEY CHECK (id > 0),
    username TEXT NOT NULL UNIQUE,
    email TEXT,
    password TEXT NOT NULL,
    name TEXT NOT NULL,
    admin_type TEXT NOT NULL CHECK (admin_type IN ('developer', 'superadmin', 'admin')),
    abilities JSONB NOT NULL DEFAULT '[]'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    CHECK (username = lower(username))
);

CREATE INDEX IF NOT EXISTS idx_admin_username ON admin (username);
CREATE INDEX IF NOT EXISTS idx_admin_admin_type ON admin (admin_type);
CREATE INDEX IF NOT EXISTS idx_admin_email ON admin (email);
