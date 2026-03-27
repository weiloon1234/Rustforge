---
name: add-seeder
description: Add a database seeder for initial or test data
---

# Add a Database Seeder

Follow these steps to add a seeder for populating the database with initial or test data.

## Step 1: Create the seeder file

Create `app/src/seeds/{name}_seeder.rs`:

```rust
use async_trait::async_trait;
use core_db::seeder::Seeder;
use sqlx::PgPool;
use generated::models::*;

#[derive(Debug, Default)]
pub struct MyDomainSeeder;

#[async_trait]
impl Seeder for MyDomainSeeder {
    fn name(&self) -> &str {
        "MyDomainSeeder"
    }

    async fn run(&self, db: &PgPool) -> anyhow::Result<()> {
        // Prefer model builders over raw SQL for type safety:
        MyDomainModel::create(MyDomainCreate {
            name: "Item One".into(),
            status: MyDomainStatus::Active,
            ..Default::default()
        })
        .save(db)
        .await?;

        MyDomainModel::create(MyDomainCreate {
            name: "Item Two".into(),
            status: MyDomainStatus::Inactive,
            ..Default::default()
        })
        .save(db)
        .await?;

        tracing::info!("Seeded my_domain records");
        Ok(())
    }
}
```

Conventions:
- Derive `Debug, Default` on the seeder struct.
- `name()` returns a unique identifier used for selective seeding.
- **Prefer model builders** (`MyDomainModel::create(MyDomainCreate { ... }).save(db)`) over raw SQL for type safety and consistency with observers.
- Use raw SQL with `ON CONFLICT ... DO NOTHING` only when idempotency is needed or the model builder is impractical.
- Log what was seeded using `tracing::info!`.
- Keep seeder data minimal -- just enough for development and testing.

For seeders that need idempotency or depend on other tables:
```rust
async fn run(&self, db: &PgPool) -> anyhow::Result<()> {
    // Use raw SQL when you need ON CONFLICT for idempotency
    sqlx::query!(
        r#"
        INSERT INTO my_domain (name, status)
        VALUES ($1, $2)
        ON CONFLICT (name) DO NOTHING
        "#,
        "Seeded Item",
        "active",
    )
    .execute(db)
    .await?;

    Ok(())
}
```

## Step 2: Register the seeder

Update `app/src/seeds/mod.rs`:

```rust
pub mod {name}_seeder;

// In the register_seeders function:
pub fn register_seeders(seeders: &mut Vec<Box<dyn core_db::seeder::Seeder>>) {
    // ... existing registrations
    seeders.push(Box::new({name}_seeder::MyDomainSeeder));
}
```

Order matters: seeders that depend on data from other seeders must be registered after their dependencies.

## Step 3: Run the seeder

Run all seeders:
```bash
./console db seed
```

Run a specific seeder by name:
```bash
./console db seed --name MyDomain
```

The `--name` flag matches against the string returned by `name()`.

## Step 4: Verify

```bash
cargo check
```

Common issues:
- Non-idempotent seeders failing on re-run -- use `ON CONFLICT` or check-before-insert logic when using raw SQL.
- Foreign key violations -- ensure dependency seeders run first.
- Missing `pub mod` declaration in `seeds/mod.rs`.
