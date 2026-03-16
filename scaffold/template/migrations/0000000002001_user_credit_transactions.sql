CREATE TABLE user_credit_transactions (
    id BIGINT PRIMARY KEY CHECK (id > 0),
    user_id BIGINT NOT NULL REFERENCES users(id),
    admin_id BIGINT REFERENCES admin(id),
    credit_type SMALLINT NOT NULL,
    amount NUMERIC(18,8) NOT NULL,
    transaction_type SMALLINT NOT NULL,
    related_key TEXT,
    params JSONB,
    remark TEXT,
    custom_description BOOLEAN NOT NULL DEFAULT FALSE,
    custom_description_text JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_user_credit_transactions_user_id ON user_credit_transactions(user_id);
CREATE INDEX idx_user_credit_transactions_admin_id ON user_credit_transactions(admin_id);
CREATE INDEX idx_user_credit_transactions_credit_type ON user_credit_transactions(credit_type);
CREATE INDEX idx_user_credit_transactions_transaction_type ON user_credit_transactions(transaction_type);
CREATE INDEX idx_user_credit_transactions_related_key ON user_credit_transactions(related_key);
CREATE INDEX idx_user_credit_transactions_created_at ON user_credit_transactions(created_at);
