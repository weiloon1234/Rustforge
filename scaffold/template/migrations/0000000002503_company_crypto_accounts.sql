CREATE TABLE company_crypto_accounts (
    id BIGINT PRIMARY KEY CHECK (id > 0),
    crypto_network_id BIGINT NOT NULL,
    wallet_address TEXT NOT NULL,
    conversion_rate NUMERIC(18,8) NOT NULL DEFAULT 1.0,
    status SMALLINT NOT NULL DEFAULT 1,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_cca_crypto_network_id FOREIGN KEY (crypto_network_id) REFERENCES crypto_networks(id)
);
CREATE INDEX idx_company_crypto_accounts_crypto_network_id ON company_crypto_accounts(crypto_network_id);
CREATE INDEX idx_company_crypto_accounts_status ON company_crypto_accounts(status);
