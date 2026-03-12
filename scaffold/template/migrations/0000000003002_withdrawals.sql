CREATE TABLE withdrawals (
    id BIGINT PRIMARY KEY CHECK (id > 0),
    owner_type SMALLINT NOT NULL,
    owner_id BIGINT NOT NULL,
    admin_id BIGINT REFERENCES admins(id),
    credit_type SMALLINT NOT NULL,
    withdrawal_method SMALLINT NOT NULL,
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
CREATE INDEX idx_withdrawals_owner ON withdrawals(owner_type, owner_id);
CREATE INDEX idx_withdrawals_status ON withdrawals(status);
CREATE INDEX idx_withdrawals_credit_type ON withdrawals(credit_type);
CREATE INDEX idx_withdrawals_withdrawal_method ON withdrawals(withdrawal_method);
CREATE INDEX idx_withdrawals_related_key ON withdrawals(related_key);
CREATE INDEX idx_withdrawals_created_at ON withdrawals(created_at);
