CREATE TABLE withdrawals (
    id BIGINT PRIMARY KEY CHECK (id > 0),
    owner_type SMALLINT NOT NULL,
    owner_id BIGINT NOT NULL,
    admin_id BIGINT REFERENCES admin(id),
    credit_type SMALLINT NOT NULL,
    withdrawal_method SMALLINT NOT NULL,
    bank_id BIGINT,
    bank_account_name TEXT,
    bank_account_number TEXT,
    crypto_network_id BIGINT,
    crypto_wallet_address TEXT,
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
    CONSTRAINT fk_withdrawals_bank_id FOREIGN KEY (bank_id) REFERENCES banks(id),
    CONSTRAINT fk_withdrawals_crypto_network_id FOREIGN KEY (crypto_network_id) REFERENCES crypto_networks(id)
);
CREATE INDEX idx_withdrawals_owner ON withdrawals(owner_type, owner_id);
CREATE INDEX idx_withdrawals_status ON withdrawals(status);
CREATE INDEX idx_withdrawals_credit_type ON withdrawals(credit_type);
CREATE INDEX idx_withdrawals_withdrawal_method ON withdrawals(withdrawal_method);
CREATE INDEX idx_withdrawals_related_key ON withdrawals(related_key);
CREATE INDEX idx_withdrawals_created_at ON withdrawals(created_at);
CREATE INDEX idx_withdrawals_bank_id ON withdrawals(bank_id);
CREATE INDEX idx_withdrawals_crypto_network_id ON withdrawals(crypto_network_id);
