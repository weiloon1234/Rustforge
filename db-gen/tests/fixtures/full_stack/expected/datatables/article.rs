// App-level datatable hooks for Article.
// Generated once by db-gen; safe to edit.

use core_datatable::DataTableRegistry;
use generated::models::{ ArticleDataTable, ArticleDataTableConfig, ArticleDataTableHooks };

#[derive(Default, Clone)]
pub struct ArticleDataTableAppHooks;

impl ArticleDataTableHooks for ArticleDataTableAppHooks {
    // Override scope/authorize/filters/mappings/summary when needed.
}

pub type AppArticleDataTable = ArticleDataTable<ArticleDataTableAppHooks>;

pub fn app_article_datatable(db: sqlx::PgPool) -> AppArticleDataTable {
    ArticleDataTable::new(db).with_hooks(ArticleDataTableAppHooks::default())
}

pub fn app_article_datatable_with_config(
    db: sqlx::PgPool,
    config: ArticleDataTableConfig,
) -> AppArticleDataTable {
    ArticleDataTable::new(db)
        .with_hooks(ArticleDataTableAppHooks::default())
        .with_config(config)
}

pub fn register_article_datatable(registry: &mut DataTableRegistry, db: sqlx::PgPool) {
    registry.register(app_article_datatable(db));
}
