CREATE TABLE deposits (
    id BIGINT PRIMARY KEY CHECK (id > 0),
    owner_type SMALLINT NOT NULL,
    owner_id BIGINT NOT NULL,
    admin_id BIGINT REFERENCES admins(id),
    credit_type SMALLINT NOT NULL,
    deposit_method SMALLINT NOT NULL,
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
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_deposits_owner ON deposits(owner_type, owner_id);
CREATE INDEX idx_deposits_status ON deposits(status);
CREATE INDEX idx_deposits_credit_type ON deposits(credit_type);
CREATE INDEX idx_deposits_deposit_method ON deposits(deposit_method);
CREATE INDEX idx_deposits_related_key ON deposits(related_key);
CREATE INDEX idx_deposits_created_at ON deposits(created_at);
