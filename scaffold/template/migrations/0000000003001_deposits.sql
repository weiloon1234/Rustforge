CREATE TABLE deposits (
    id BIGINT PRIMARY KEY CHECK (id > 0),
    owner_type SMALLINT NOT NULL,
    owner_id BIGINT NOT NULL,
    admin_id BIGINT REFERENCES admin(id),
    credit_type SMALLINT NOT NULL,
    deposit_method SMALLINT NOT NULL,
    company_bank_account_id BIGINT,
    company_crypto_account_id BIGINT,
    conversion_rate NUMERIC(18,8),
    status SMALLINT NOT NULL DEFAULT 1,
    amount NUMERIC(18,8) NOT NULL,
    fee NUMERIC(18,8) NOT NULL DEFAULT 0,
    net_amount NUMERIC(18,8) NOT NULL,
    related_key TEXT,
    params JSONB,
    remark TEXT,
    admin_remark TEXT,
    reviewed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_deposits_company_bank_account_id FOREIGN KEY (company_bank_account_id) REFERENCES company_bank_accounts(id),
    CONSTRAINT fk_deposits_company_crypto_account_id FOREIGN KEY (company_crypto_account_id) REFERENCES company_crypto_accounts(id)
);
CREATE INDEX idx_deposits_owner ON deposits(owner_type, owner_id);
CREATE INDEX idx_deposits_status ON deposits(status);
CREATE INDEX idx_deposits_credit_type ON deposits(credit_type);
CREATE INDEX idx_deposits_deposit_method ON deposits(deposit_method);
CREATE INDEX idx_deposits_related_key ON deposits(related_key);
CREATE INDEX idx_deposits_created_at ON deposits(created_at);
CREATE INDEX idx_deposits_company_bank_account_id ON deposits(company_bank_account_id);
CREATE INDEX idx_deposits_company_crypto_account_id ON deposits(company_crypto_account_id);
