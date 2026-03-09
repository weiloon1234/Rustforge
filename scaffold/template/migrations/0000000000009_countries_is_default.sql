ALTER TABLE countries ADD COLUMN is_default SMALLINT NOT NULL DEFAULT 0;
CREATE INDEX IF NOT EXISTS idx_countries_is_default ON countries(is_default);
