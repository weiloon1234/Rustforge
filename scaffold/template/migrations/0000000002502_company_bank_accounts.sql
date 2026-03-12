CREATE TABLE company_bank_accounts (
    id BIGINT PRIMARY KEY CHECK (id > 0),
    bank_id BIGINT NOT NULL,
    account_name TEXT NOT NULL,
    account_number TEXT NOT NULL,
    status SMALLINT NOT NULL DEFAULT 1,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_cba_bank_id FOREIGN KEY (bank_id) REFERENCES banks(id)
);
CREATE INDEX idx_company_bank_accounts_bank_id ON company_bank_accounts(bank_id);
CREATE INDEX idx_company_bank_accounts_status ON company_bank_accounts(status);
