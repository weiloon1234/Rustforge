use std::collections::BTreeMap;

use async_trait::async_trait;
use core_db::{
    common::sql::{generate_snowflake_i64, DbConn, Op},
    seeder::Seeder,
};
use generated::models::{ContentPage, ContentPageSystemFlag};

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
    let model = ContentPage::new(DbConn::pool(db), None);
    let existing = model
        .query()
        .where_tag(Op::Eq, tag.to_string())
        .first()
        .await?;

    if let Some(page) = existing {
        model
            .update()
            .where_id(Op::Eq, page.id)
            .set_tag(tag.to_string())
            .set_is_system(ContentPageSystemFlag::Yes)
            .save()
            .await?;
        return Ok(());
    }

    let title_langs = build_localized_text(title_en, title_zh);
    let content_langs = build_localized_text(content_en, content_zh);

    model
        .insert()
        .set_id(generate_snowflake_i64())
        .set_tag(tag.to_string())
        .set_is_system(ContentPageSystemFlag::Yes)
        .set_title_langs(title_langs)
        .set_content_langs(content_langs)
        .save()
        .await?;

    Ok(())
}

fn build_localized_text(en_value: &str, zh_value: &str) -> generated::LocalizedText {
    let mut payload = BTreeMap::new();
    for &locale in generated::SUPPORTED_LOCALES {
        let value = match locale {
            "zh" => zh_value,
            _ => en_value,
        };
        payload.insert(locale.to_string(), value.to_string());
    }
    generated::LocalizedText::from_map(&payload)
}
