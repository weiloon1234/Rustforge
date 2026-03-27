---
name: add-migration
description: Create a database migration
---

# Create a Database Migration

Follow these steps to create a new database migration.

## Step 1: Create the migration file

Determine the next migration number by checking existing files in `migrations/`. The format is `{number}_{description}.sql` where the number is zero-padded and incremented from the last migration.

Create `migrations/{timestamp}_{description}.sql`.

**For a new table:**
```sql
CREATE TABLE IF NOT EXISTS my_domain (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    amount NUMERIC(10, 2) NOT NULL DEFAULT 0,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_my_domain_user_id ON my_domain (user_id);
CREATE INDEX idx_my_domain_status ON my_domain (status);
CREATE INDEX idx_my_domain_created_at ON my_domain (created_at);
```

**For adding columns:**
```sql
ALTER TABLE my_domain
    ADD COLUMN new_field VARCHAR(255),
    ADD COLUMN priority INTEGER NOT NULL DEFAULT 0;

CREATE INDEX idx_my_domain_priority ON my_domain (priority);
```

**For removing columns:**
```sql
ALTER TABLE my_domain
    DROP COLUMN IF EXISTS old_field;
```

**For adding a unique constraint:**
```sql
ALTER TABLE my_domain
    ADD CONSTRAINT uq_my_domain_name UNIQUE (name);
```

**For adding a composite index:**
```sql
CREATE INDEX idx_my_domain_user_status ON my_domain (user_id, status);
```

Conventions:
- Use `IF NOT EXISTS` / `IF EXISTS` for safety.
- Table names are plural snake_case (e.g., `my_domains`).
- Use `BIGSERIAL` for primary keys.
- Use `TIMESTAMPTZ` for all timestamp columns.
- Use `NUMERIC(precision, scale)` for monetary values.
- Use `JSONB` for semi-structured data.
- Always add indexes for foreign keys and frequently queried columns.
- Foreign keys should specify `ON DELETE` behavior (`CASCADE`, `SET NULL`, or `RESTRICT`).
- Index naming: `idx_{table}_{column}` or `idx_{table}_{col1}_{col2}`.
- Constraint naming: `uq_{table}_{column}` for unique, `fk_{table}_{ref_table}` for foreign keys.

## Step 2: Update the model (if needed)

If the migration changes a table that has a corresponding model, update `app/models/{model}.rs`:
- Add or remove fields to match the new schema.
- Update field types to match column types.
- Update any enums if new enum values were added.

## Step 3: Run code generation (if model changed)

```bash
make gen
```

Only needed if you modified a model file. This regenerates the Rust types from the updated schema.

## Step 4: Run the migration

```bash
./console migrate run
```

This applies all pending migrations to the database.

## Step 5: Verify

```bash
cargo check
```

Common issues:
- SQL syntax errors -- test the migration SQL against a local database.
- Model field type mismatch after schema change.
- Missing `make gen` after model changes -- generated code will be stale.
- Foreign key references to tables that don't exist yet -- order migrations carefully.
