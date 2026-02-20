#![allow(dead_code)]

use std::collections::HashMap;

use anyhow::Result;

use crate::common::sql::DbConn;
use crate::platform::meta::model::MetaRow;
use crate::platform::meta::types::MetaMap;

pub struct MetaRepo<'a> {
    db: DbConn<'a>,
}

impl<'a> MetaRepo<'a> {
    pub fn new(db: DbConn<'a>) -> Self {
        Self { db }
    }

    pub async fn load_for_owners(
        &self,
        owner_type: &str,
        owner_ids: &[i64],
        fields: &[&str],
    ) -> Result<MetaMap> {
        if owner_ids.is_empty() || fields.is_empty() {
            return Ok(MetaMap::default());
        }
        let fields_vec: Vec<String> = fields.iter().map(|s| s.to_string()).collect();
        #[cfg(feature = "sqlx-checked")]
        let _ = sqlx::query!(
            r#"
            SELECT owner_type, owner_id, field, value
            FROM meta
            WHERE owner_type = $1
              AND owner_id = ANY($2)
              AND field = ANY($3)
            "#,
            owner_type,
            owner_ids,
            &fields_vec
        );
        let q = sqlx::query_as::<_, MetaRow>(
            r#"
            SELECT owner_type, owner_id, field, value
            FROM meta
            WHERE owner_type = $1
              AND owner_id = ANY($2)
              AND field = ANY($3)
            "#,
        )
        .bind(owner_type)
        .bind(owner_ids)
        .bind(&fields_vec);
        let rows = self.db.fetch_all(q).await?;

        let mut map: HashMap<String, HashMap<i64, serde_json::Value>> = HashMap::new();
        for row in rows {
            map.entry(row.field)
                .or_default()
                .insert(row.owner_id, row.value);
        }

        Ok(MetaMap::new(map))
    }

    pub async fn load_for_owner(
        &self,
        owner_type: &str,
        owner_id: i64,
        fields: &[&str],
    ) -> Result<MetaMap> {
        self.load_for_owners(owner_type, &[owner_id], fields).await
    }

    pub async fn upsert_many(
        &self,
        owner_type: &str,
        owner_id: i64,
        values: &HashMap<String, serde_json::Value>,
    ) -> Result<()> {
        if values.is_empty() {
            return Ok(());
        }
        #[cfg(feature = "sqlx-checked")]
        let _ = sqlx::query!(
            r#"
            INSERT INTO meta (owner_type, owner_id, field, value)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (owner_type, owner_id, field)
            DO UPDATE SET value = EXCLUDED.value
            "#,
            owner_type,
            owner_id,
            "",
            serde_json::Value::Null
        );

        // Simply iterate execute, supporting both pool and transaction
        for (field, value) in values {
            let q = sqlx::query(
                r#"
                INSERT INTO meta (owner_type, owner_id, field, value)
                VALUES ($1, $2, $3, $4)
                ON CONFLICT (owner_type, owner_id, field)
                DO UPDATE SET value = EXCLUDED.value
                "#,
            )
            .bind(owner_type)
            .bind(owner_id)
            .bind(field)
            .bind(value);
            self.db.execute(q).await?;
        }
        Ok(())
    }
}
