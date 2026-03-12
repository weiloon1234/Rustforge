CREATE TABLE banks (
    id BIGINT PRIMARY KEY CHECK (id > 0),
    country_iso2 TEXT NOT NULL,
    name TEXT NOT NULL,
    code TEXT,
    status SMALLINT NOT NULL DEFAULT 1,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_banks_country_iso2 FOREIGN KEY (country_iso2) REFERENCES countries(iso2)
);
CREATE INDEX idx_banks_country_iso2 ON banks(country_iso2);
CREATE INDEX idx_banks_status ON banks(status);
