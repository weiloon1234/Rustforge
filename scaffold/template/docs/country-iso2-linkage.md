# Country ISO2 Linkage Migration Playbook

Use this playbook when a table needs to link to framework countries.

## Canonical Rule

- Country reference column name: `country_iso2`
- Column type: `TEXT`
- Reference key: `countries.iso2`
- Default policy: use both DB foreign key and app-level validation

`countries.iso2` is already the primary key, so it is already unique and indexed.

## New Table (NOT NULL country)

```sql
CREATE TABLE IF NOT EXISTS users (
    id BIGINT PRIMARY KEY CHECK (id > 0),
    country_iso2 TEXT NOT NULL,
    username TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_users_country_iso2
        FOREIGN KEY (country_iso2) REFERENCES countries(iso2)
);

CREATE INDEX IF NOT EXISTS idx_users_country_iso2 ON users(country_iso2);
```

## New Table (nullable country)

```sql
CREATE TABLE IF NOT EXISTS profile (
    id BIGINT PRIMARY KEY CHECK (id > 0),
    country_iso2 TEXT,
    display_name TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT fk_profile_country_iso2
        FOREIGN KEY (country_iso2) REFERENCES countries(iso2)
);

CREATE INDEX IF NOT EXISTS idx_profile_country_iso2 ON profile(country_iso2);
```

## Existing Table Migration (add country linkage)

```sql
ALTER TABLE users
    ADD COLUMN IF NOT EXISTS country_iso2 TEXT;

ALTER TABLE users
    ADD CONSTRAINT fk_users_country_iso2
    FOREIGN KEY (country_iso2) REFERENCES countries(iso2);

CREATE INDEX IF NOT EXISTS idx_users_country_iso2 ON users(country_iso2);
```

## Legacy Conversion (numeric `country_id` -> `country_iso2`)

1. Add new `country_iso2` column (nullable first).
2. Backfill from your old mapping source.
3. Add FK + index.
4. Make column `NOT NULL` only after data is clean.
5. Drop old `country_id` after rollout verification.

Example:

```sql
ALTER TABLE users ADD COLUMN IF NOT EXISTS country_iso2 TEXT;

-- Example backfill. Replace `legacy_country_map` with your own mapping source.
UPDATE users u
SET country_iso2 = m.iso2
FROM legacy_country_map m
WHERE u.country_id = m.legacy_country_id
  AND u.country_iso2 IS NULL;

ALTER TABLE users
    ADD CONSTRAINT fk_users_country_iso2
    FOREIGN KEY (country_iso2) REFERENCES countries(iso2);

CREATE INDEX IF NOT EXISTS idx_users_country_iso2 ON users(country_iso2);

-- Optional final hardening step after cleanup:
-- ALTER TABLE users ALTER COLUMN country_iso2 SET NOT NULL;

-- Final cleanup (after application rollout verification):
-- ALTER TABLE users DROP COLUMN country_id;
```

## Validation Contract Pattern

For request DTOs:

1. Keep field name as `country_iso2`.
2. Normalize to uppercase ISO2 before persistence.
3. Validate existence via async rule against `countries.iso2`.
4. If business flow requires only enabled countries, add `where_eq(column = "status", value = "enabled")`.

## Frontend Contract

Shared contact input shape remains:

```ts
{ country_iso2: string, phone_number: string }
```

Reference implementation:
- `frontend/src/shared/components/ContactInput.tsx`
