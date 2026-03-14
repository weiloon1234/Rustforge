use std::collections::BTreeMap;

use async_trait::async_trait;
use core_db::{
    common::sql::{generate_snowflake_i64, DbConn, Op},
    seeder::Seeder,
};
use generated::{
    localized::LocalizedInput,
    models::{ContentPageCol, ContentPageModel, ContentPageSystemFlag},
};

#[derive(Debug, Default)]
pub struct ContentPageBootstrapSeeder;

#[async_trait]
impl Seeder for ContentPageBootstrapSeeder {
    async fn run(&self, db: &sqlx::PgPool) -> anyhow::Result<()> {
        ensure_page(
            db,
            "terms_and_conditions",
            "Terms and Conditions",
            "条款与条件",
            "<p>Please provide your terms and conditions content here.</p>",
            "<p>请在此填写条款与条件内容。</p>",
        )
        .await?;

        ensure_page(
            db,
            "privacy_policy",
            "Privacy Policy",
            "隐私政策",
            "<p>Please provide your privacy policy content here.</p>",
            "<p>请在此填写隐私政策内容。</p>",
        )
        .await?;

        Ok(())
    }

    fn name(&self) -> &str {
        "ContentPageBootstrapSeeder"
    }
}

async fn ensure_page(
    db: &sqlx::PgPool,
    tag: &str,
    title_en: &str,
    title_zh: &str,
    content_en: &str,
    content_zh: &str,
) -> anyhow::Result<()> {
    let existing = ContentPageModel::query(DbConn::pool(db))
        .where_col(ContentPageCol::TAG, Op::Eq, tag.to_string())
        .first()
        .await?;

    if let Some(page) = existing {
        ContentPageModel::query(DbConn::pool(db))
            .where_col(ContentPageCol::ID, Op::Eq, page.id)
            .patch()
            .assign(ContentPageCol::TAG, tag.to_string())?
            .assign(ContentPageCol::IS_SYSTEM, ContentPageSystemFlag::Yes)?
            .save()
            .await?;
        return Ok(());
    }

    let title_input = build_localized_input(title_en, title_zh);
    let content_input = build_localized_input(content_en, content_zh);
    let scope = DbConn::pool(db).begin_scope().await?;
    let conn = scope.conn();

    let page = ContentPageModel::create(conn.clone())
        .set(ContentPageCol::ID, generate_snowflake_i64())?
        .set(ContentPageCol::TAG, tag.to_string())?
        .set(ContentPageCol::IS_SYSTEM, ContentPageSystemFlag::Yes)?
        .save()
        .await?;

    page.upsert_title(conn.clone(), Some(title_input)).await?;
    page.upsert_content(conn, Some(content_input)).await?;
    scope.commit().await?;

    Ok(())
}

fn build_localized_input(en_value: &str, zh_value: &str) -> LocalizedInput {
    let mut payload = BTreeMap::new();
    for &locale in generated::SUPPORTED_LOCALES {
        let value = match locale {
            "zh" => zh_value,
            _ => en_value,
        };
        payload.insert(locale.to_string(), value.to_string());
    }
    LocalizedInput::from_map(&payload)
}
