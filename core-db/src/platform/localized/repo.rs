#![allow(dead_code)]

use anyhow::Result;
use std::collections::HashMap;

use crate::common::sql::DbConn;
use crate::platform::localized::model::LocalizedRow;
use crate::platform::localized::types::LocalizedMap;

pub struct LocalizedRepo<'a> {
    db: DbConn<'a>,
}

impl<'a> LocalizedRepo<'a> {
    pub fn new(db: DbConn<'a>) -> Self {
        Self { db }
    }

    pub async fn load_for_owners(
        &self,
        owner_type: &str,
        owner_ids: &[i64],
        fields: &[&str],
    ) -> Result<LocalizedMap> {
        if owner_ids.is_empty() || fields.is_empty() {
            return Ok(LocalizedMap::default());
        }

        let fields_vec: Vec<String> = fields.iter().map(|s| s.to_string()).collect();

        #[cfg(feature = "sqlx-checked")]
        let _ = sqlx::query!(
            r#"
            SELECT owner_type, owner_id, field, locale, value
            FROM localized
            WHERE owner_type = $1
              AND owner_id = ANY($2)
              AND field = ANY($3)
            "#,
            owner_type,
            owner_ids,
            &fields_vec
        );

        let q = sqlx::query_as::<_, LocalizedRow>(
            r#"
            SELECT owner_type, owner_id, field, locale, value
            FROM localized
            WHERE owner_type = $1
              AND owner_id = ANY($2)
              AND field = ANY($3)
            "#,
        )
        .bind(owner_type)
        .bind(owner_ids)
        .bind(&fields_vec);

        let rows = self.db.fetch_all(q).await?;

        // field -> owner_id -> locale -> value
        let mut map: HashMap<String, HashMap<i64, HashMap<String, String>>> = HashMap::new();

        for r in rows {
            map.entry(r.field)
                .or_default()
                .entry(r.owner_id)
                .or_default()
                .insert(r.locale, r.value);
        }

        Ok(LocalizedMap::new(map))
    }

    pub async fn load_for_owner(
        &self,
        owner_type: &str,
        owner_id: i64,
        fields: &[&str],
    ) -> Result<LocalizedMap> {
        self.load_for_owners(owner_type, &[owner_id], fields).await
    }

    pub async fn upsert_one(
        &self,
        owner_type: &str,
        owner_id: i64,
        field: &str,
        locale: &str,
        value: String,
    ) -> Result<()> {
        #[cfg(feature = "sqlx-checked")]
        let _ = sqlx::query!(
            r#"
            INSERT INTO localized (owner_type, owner_id, field, locale, value)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (owner_type, owner_id, field, locale)
            DO UPDATE SET value = EXCLUDED.value
            "#,
            owner_type,
            owner_id,
            field,
            locale,
            &value
        );

        let q = sqlx::query(
            r#"
            INSERT INTO localized (owner_type, owner_id, field, locale, value)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (owner_type, owner_id, field, locale)
            DO UPDATE SET value = EXCLUDED.value
            "#,
        )
        .bind(owner_type)
        .bind(owner_id)
        .bind(field)
        .bind(locale)
        .bind(value);
        self.db.execute(q).await?;
        Ok(())
    }

    pub async fn upsert_many(
        &self,
        owner_type: &str,
        owner_id: i64,
        field: &str,
        values: &HashMap<String, String>,
    ) -> Result<()> {
        if values.is_empty() {
            return Ok(());
        }

        #[cfg(feature = "sqlx-checked")]
        let _ = sqlx::query!(
            r#"
            INSERT INTO localized (owner_type, owner_id, field, locale, value)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (owner_type, owner_id, field, locale)
            DO UPDATE SET value = EXCLUDED.value
            "#,
            owner_type,
            owner_id,
            field,
            "",
            ""
        );

        // For upsert_many, simply loop execute.
        // If self.db is already a tx, it participates in it.
        // If self.db is a pool, each execution is atomic, which is acceptable or we could start explicit tx if needed.
        // Given DbConn design, iterating executes is the simplest path.
        for (locale, value) in values {
            let q = sqlx::query(
                r#"
                INSERT INTO localized (owner_type, owner_id, field, locale, value)
                VALUES ($1, $2, $3, $4, $5)
                ON CONFLICT (owner_type, owner_id, field, locale)
                DO UPDATE SET value = EXCLUDED.value
                "#,
            )
            .bind(owner_type)
            .bind(owner_id)
            .bind(field)
            .bind(locale)
            .bind(value);
            self.db.execute(q).await?;
        }
        Ok(())
    }
}
