CREATE TABLE crypto_networks (
    id BIGINT PRIMARY KEY CHECK (id > 0),
    name TEXT NOT NULL,
    symbol TEXT NOT NULL,
    status SMALLINT NOT NULL DEFAULT 1,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_crypto_networks_status ON crypto_networks(status);
