use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use db_gen::{
    config, generate_auth, generate_datatable_skeletons, generate_enums, generate_models, schema,
};

fn temp_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after unix epoch")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "rs_core_db_gen_{prefix}_{}_{}",
        std::process::id(),
        nanos
    ));
    fs::create_dir_all(&dir).expect("failed to create temp dir");
    dir
}

#[test]
fn generated_models_use_typed_first_api_with_explicit_unsafe_escape_hatch() {
    let root = temp_dir("typed_first");
    let schema_dir = root.join("schemas");
    let out_dir = root.join("out");
    fs::create_dir_all(&schema_dir).expect("failed to create schemas dir");
    fs::create_dir_all(&out_dir).expect("failed to create out dir");

    fs::write(
        root.join("configs.toml"),
        r#"
[languages]
default = "en"
supported = ["en"]
"#,
    )
    .expect("failed to write configs");

    fs::write(
        schema_dir.join("article.toml"),
        r#"
[ArticleStatus]
type = "enum"
storage = "string"
variants = ["Draft", "Published"]

[model.article]
table = "articles"
pk = "id"
fields = [
  "id:i64",
  "title:string",
  "status:ArticleStatus",
  "views:i64",
  "created_at:datetime",
  "updated_at:datetime"
]
meta = ["flags:bool", "extra:json"]
"#,
    )
    .expect("failed to write schema");

    let configs_path = root.join("configs.toml");
    let schemas_path = schema_dir.clone();
    let (cfgs, _) = config::load(
        configs_path
            .to_str()
            .expect("configs path should be valid utf-8"),
    )
    .expect("failed to load config");
    let parsed_schema = schema::load(
        schemas_path
            .to_str()
            .expect("schema path should be valid utf-8"),
    )
    .expect("failed to load schema");

    generate_enums(&parsed_schema, &out_dir).expect("enum generation should succeed");
    generate_models(&parsed_schema, &cfgs, &out_dir).expect("model generation should succeed");

    let mod_rs = fs::read_to_string(out_dir.join("mod.rs")).expect("mod.rs should exist");
    let enums_rs = fs::read_to_string(out_dir.join("enums.rs")).expect("enums.rs should exist");
    let article_rs =
        fs::read_to_string(out_dir.join("article.rs")).expect("article.rs should exist");

    assert!(mod_rs.contains("pub mod enums;"));
    assert!(mod_rs.contains("pub use enums::*;"));
    assert!(enums_rs.contains("pub enum ArticleStatus"));
    assert!(mod_rs.contains(
        "pub use article::{Article, ArticleView, ArticleQuery, ArticleInsert, ArticleUpdate, ArticleCol, ArticleTableAdapter, ArticleDataTable, ArticleDataTableConfig, ArticleDataTableHooks, ArticleDefaultDataTableHooks, ArticleViewsExt};"
    ));
    assert!(!mod_rs.contains("pub use article::*;"));

    assert!(article_rs.contains("pub struct Article<'db>"));
    assert!(!article_rs.contains("pub struct ArticleModel<'db>"));
    assert!(article_rs.contains("#[derive(Clone)]\npub struct ArticleQuery<'db>"));
    assert!(article_rs.contains("pub struct ArticleTableAdapter;"));
    assert!(article_rs.contains("pub struct ArticleDataTableConfig {"));
    assert!(article_rs.contains("pub trait ArticleDataTableHooks: Send + Sync + 'static {"));
    assert!(article_rs.contains("pub struct ArticleDataTable<H = ArticleDefaultDataTableHooks> where H: ArticleDataTableHooks {"));
    assert!(article_rs
        .contains("impl<H: ArticleDataTableHooks> AutoDataTable for ArticleDataTable<H> {"));
    assert!(article_rs.contains("impl GeneratedTableAdapter for ArticleTableAdapter {"));
    assert!(article_rs.contains("type Query<'db> = ArticleQuery<'db>;"));
    assert!(article_rs.contains("fn apply_auto_filter<'db>(&self, query: ArticleQuery<'db>, filter: &ParsedFilter, value: &str)"));
    assert!(article_rs.contains("fn apply_cursor<'db>(&self, query: ArticleQuery<'db>, column: &str, dir: SortDirection, cursor: &str)"));
    assert!(article_rs
        .contains("fn cursor_from_row(&self, row: &ArticleView, column: &str) -> Option<String>"));
    assert!(article_rs.contains("let per_page = resolve_per_page(per_page);"));
    assert!(!article_rs.contains("std::env::var(\"DEFAULT_PER_PAGE\")"));

    // Raw SQL is not on the default fluent surface.
    assert!(!article_rs.contains("pub fn where_raw<T: Into<BindValue>>"));
    assert!(!article_rs.contains("pub fn select_raw(mut self, sql: impl Into<String>) -> Self"));
    assert!(!article_rs.contains("pub fn add_select_raw(mut self, sql: impl Into<String>) -> Self"));
    assert!(!article_rs.contains("pub fn order_by_raw(mut self, sql: impl Into<String>) -> Self"));
    assert!(!article_rs.contains("pub fn group_by_raw(mut self, sql: impl Into<String>) -> Self"));

    // Unsafe escape hatch is explicitly available.
    assert!(article_rs.contains("pub fn unsafe_sql(self) -> ArticleUnsafeQuery<'db>"));
    assert!(article_rs.contains("pub struct ArticleUnsafeQuery<'db>"));
    assert!(article_rs.contains("pub struct ArticleUnsafeUpdate<'db>"));
    assert!(article_rs.contains("#[doc(hidden)]\npub struct ArticleUnsafeQuery<'db>"));
    assert!(article_rs.contains("#[doc(hidden)]\npub struct ArticleUnsafeUpdate<'db>"));
    assert!(article_rs.contains("#[doc(hidden)]\npub struct ArticleRow {"));
    assert!(article_rs.contains("#[doc(hidden)]\npub struct ArticleJson {"));
    assert!(article_rs.contains("pub fn where_raw(mut self, clause: RawClause) -> Self"));
    assert!(article_rs.contains("pub fn done(self) -> ArticleQuery<'db>"));
    assert!(article_rs.contains("pub fn where_key(self, id: i64) -> Self"));
    assert!(article_rs
        .contains("pub fn where_key_in<T: Clone + Into<BindValue>>(self, vals: &[T]) -> Self"));
    assert!(article_rs.contains("pub async fn first_or_fail(self) -> Result<ArticleView>"));
    assert!(article_rs.contains("pub async fn find_or_fail(self, id: i64) -> Result<ArticleView>"));
    assert!(article_rs.contains("pub trait ArticleViewsExt {"));
    assert!(article_rs.contains("impl ArticleViewsExt for Vec<ArticleView> {"));
    assert!(article_rs.contains("use core_db::common::collection::TypedCollectionExt;"));
    assert!(article_rs.contains("DbConn::Pool(pool) => {"));
    assert!(article_rs.contains("let tx_lock = std::sync::Arc::new(tokio::sync::Mutex::new(tx));"));
    assert!(article_rs.contains("let db = DbConn::tx(tx_lock.clone());"));
    assert!(article_rs.contains("self.save_with_db(db).await?"));
    assert!(!article_rs.contains("pub async fn returning_row"));
    assert!(!article_rs.contains("pub async fn returning_view"));

    // Module path correctness for relation imports.
    assert!(!article_rs.contains("crate::generated_models::"));

    // Typed meta accessors are generated on views.
    assert!(article_rs.contains("pub fn meta_flags(&self) -> Option<bool>"));
    assert!(article_rs.contains(
        "pub fn meta_extra_as<T: serde::de::DeserializeOwned>(&self) -> anyhow::Result<Option<T>>"
    ));
    assert!(article_rs.contains("pub fn set_meta_flags(mut self, val: bool) -> Self"));
    assert!(article_rs.contains(
        "pub fn set_meta_extra_as<T: serde::Serialize>(mut self, val: &T) -> anyhow::Result<Self>"
    ));

    // i64 PK default strategy is snowflake.
    assert!(article_rs.contains("binds.push(generate_snowflake_i64().into());"));
    assert!(article_rs.contains("matches!(c, ArticleCol::Id)"));

    fs::remove_dir_all(root).expect("failed to remove temp dir");
}

#[test]
fn generated_models_support_custom_meta_shape_without_cast_helper() {
    let root = temp_dir("meta_custom_shape");
    let schema_dir = root.join("schemas");
    let out_dir = root.join("out");
    fs::create_dir_all(&schema_dir).expect("failed to create schemas dir");
    fs::create_dir_all(&out_dir).expect("failed to create out dir");

    fs::write(
        root.join("configs.toml"),
        r#"
[languages]
default = "en"
supported = ["en"]
"#,
    )
    .expect("failed to write configs");

    fs::write(
        schema_dir.join("article.toml"),
        r#"
[ArticleStatus]
type = "enum"
storage = "string"
variants = ["Draft", "Published"]

[model.article]
table = "articles"
pk = "id"
fields = [
  "id:i64",
  "title:string",
  "status:ArticleStatus"
]
meta = ["extra:ExtraMeta"]
"#,
    )
    .expect("failed to write schema");

    let (cfgs, _) = config::load(
        root.join("configs.toml")
            .to_str()
            .expect("configs path should be valid utf-8"),
    )
    .expect("failed to load config");
    let parsed_schema = schema::load(
        schema_dir
            .to_str()
            .expect("schema path should be valid utf-8"),
    )
    .expect("failed to load schema");

    generate_enums(&parsed_schema, &out_dir).expect("enum generation should succeed");
    generate_models(&parsed_schema, &cfgs, &out_dir).expect("model generation should succeed");

    let article_rs =
        fs::read_to_string(out_dir.join("article.rs")).expect("article.rs should exist");

    assert!(article_rs.contains("use crate::extensions::article::types::*;"));
    assert!(article_rs.contains("pub fn meta_extra(&self) -> anyhow::Result<Option<ExtraMeta>>"));
    assert!(article_rs
        .contains("pub fn set_meta_extra(mut self, val: &ExtraMeta) -> anyhow::Result<Self>"));

    fs::remove_dir_all(root).expect("failed to remove temp dir");
}

#[test]
fn generated_models_support_nested_relation_filter_paths() {
    let root = temp_dir("nested_rel_filters");
    let schema_dir = root.join("schemas");
    let out_dir = root.join("out");
    fs::create_dir_all(&schema_dir).expect("failed to create schemas dir");
    fs::create_dir_all(&out_dir).expect("failed to create out dir");

    fs::write(
        root.join("configs.toml"),
        r#"
[languages]
default = "en"
supported = ["en"]
"#,
    )
    .expect("failed to write configs");

    fs::write(
        schema_dir.join("profile.toml"),
        r#"
[model.profile]
table = "profiles"
fields = ["id:i64", "display_name:string"]
"#,
    )
    .expect("failed to write profile schema");

    fs::write(
        schema_dir.join("user.toml"),
        r#"
[model.user]
table = "users"
fields = ["id:i64", "name:string", "profile_id:i64"]
relations = ["profile:belongs_to:profile:profile_id:id"]
"#,
    )
    .expect("failed to write user schema");

    fs::write(
        schema_dir.join("article.toml"),
        r#"
[model.article]
table = "articles"
fields = ["id:i64", "title:string", "author_id:i64"]
relations = ["author:belongs_to:user:author_id:id"]
"#,
    )
    .expect("failed to write article schema");

    let (cfgs, _) = config::load(
        root.join("configs.toml")
            .to_str()
            .expect("configs path should be valid utf-8"),
    )
    .expect("failed to load config");
    let parsed_schema = schema::load(
        schema_dir
            .to_str()
            .expect("schema path should be valid utf-8"),
    )
    .expect("failed to load schema");

    generate_enums(&parsed_schema, &out_dir).expect("enum generation should succeed");
    generate_models(&parsed_schema, &cfgs, &out_dir).expect("model generation should succeed");

    let article_rs =
        fs::read_to_string(out_dir.join("article.rs")).expect("article.rs should exist");

    assert!(article_rs.contains("(\"author__profile\", \"display_name\")"));
    assert!(article_rs.contains(
        "Ok(Some(query.where_has_author(|rq| rq.where_has_profile(|rq| rq.where_col(ProfileCol::DisplayName, Op::Eq, bind)))))"
    ));
    assert!(article_rs.contains(
        "Ok(Some(query.where_has_author(|rq| rq.where_has_profile(|rq| rq.where_col(ProfileCol::DisplayName, Op::Like, pattern.clone())))))"
    ));

    fs::remove_dir_all(root).expect("failed to remove temp dir");
}

#[test]
fn generate_datatable_skeletons_creates_app_stubs_without_overwriting_existing_files() {
    let root = temp_dir("datatable_stubs");
    let schema_dir = root.join("schemas");
    let out_dir = root.join("models").join("src").join("datatables");
    fs::create_dir_all(&schema_dir).expect("failed to create schemas dir");
    fs::create_dir_all(&out_dir).expect("failed to create out dir");

    fs::write(
        schema_dir.join("article.toml"),
        r#"
[model.article]
table = "articles"
fields = ["id:i64", "title:string"]
"#,
    )
    .expect("failed to write schema");

    let parsed_schema = schema::load(
        schema_dir
            .to_str()
            .expect("schema path should be valid utf-8"),
    )
    .expect("failed to load schema");

    generate_datatable_skeletons(&parsed_schema, &out_dir)
        .expect("datatable skeleton generation should succeed");

    let mod_rs = fs::read_to_string(out_dir.join("mod.rs")).expect("mod.rs should exist");
    let generated_rs = fs::read_to_string(out_dir.join("mod.generated.rs"))
        .expect("mod.generated.rs should exist");
    let article_rs =
        fs::read_to_string(out_dir.join("article.rs")).expect("article.rs should exist");

    assert!(mod_rs.contains("include!(\"mod.generated.rs\");"));
    assert!(generated_rs.contains("register_all_generated_datatables"));
    assert!(generated_rs.contains("register_article_datatable"));
    assert!(article_rs.contains("pub struct ArticleDataTableAppHooks;"));
    assert!(article_rs.contains("impl ArticleDataTableHooks for ArticleDataTableAppHooks {"));

    fs::write(out_dir.join("article.rs"), "// custom").expect("failed to write custom marker");
    generate_datatable_skeletons(&parsed_schema, &out_dir)
        .expect("datatable skeleton re-generation should succeed");
    let article_after =
        fs::read_to_string(out_dir.join("article.rs")).expect("article.rs should still exist");
    assert_eq!(article_after, "// custom");

    fs::remove_dir_all(root).expect("failed to remove temp dir");
}

#[test]
fn generate_auth_always_emits_guard_agnostic_resolvers() {
    let root = temp_dir("auth_resolver");
    let out_dir = root.join("out");
    fs::create_dir_all(&out_dir).expect("failed to create out dir");

    fs::write(
        root.join("configs.toml"),
        r#"
[languages]
default = "en"
supported = ["en"]
"#,
    )
    .expect("failed to write configs");

    let (cfgs, _) = config::load(
        root.join("configs.toml")
            .to_str()
            .expect("configs path should be valid utf-8"),
    )
    .expect("failed to load config");

    generate_auth(&cfgs, &out_dir).expect("auth generation should succeed");
    let mod_rs = fs::read_to_string(out_dir.join("mod.rs")).expect("mod.rs should exist");

    assert!(mod_rs.contains("pub async fn authenticate_any_guard("));
    assert!(mod_rs.contains("pub async fn datatable_actor_from_token("));
    assert!(mod_rs.contains("let _ = (db, token);"));
    assert!(mod_rs.contains("None"));

    fs::remove_dir_all(root).expect("failed to remove temp dir");
}

#[test]
fn generated_models_support_locale_filter_variants() {
    let root = temp_dir("locale_filters");
    let schema_dir = root.join("schemas");
    let out_dir = root.join("out");
    fs::create_dir_all(&schema_dir).expect("failed to create schemas dir");
    fs::create_dir_all(&out_dir).expect("failed to create out dir");

    fs::write(
        root.join("configs.toml"),
        r#"
[languages]
default = "en"
supported = ["en", "zh"]
"#,
    )
    .expect("failed to write configs");

    fs::write(
        schema_dir.join("profile.toml"),
        r#"
[model.profile]
table = "profiles"
fields = ["id:i64", "display_name:string"]
multilang = ["display_name"]
"#,
    )
    .expect("failed to write profile schema");

    fs::write(
        schema_dir.join("user.toml"),
        r#"
[model.user]
table = "users"
fields = ["id:i64", "profile_id:i64", "name:string"]
relations = ["profile:belongs_to:profile:profile_id:id"]
"#,
    )
    .expect("failed to write user schema");

    fs::write(
        schema_dir.join("article.toml"),
        r#"
[model.article]
table = "articles"
fields = ["id:i64", "author_id:i64", "title:string"]
multilang = ["title"]
relations = ["author:belongs_to:user:author_id:id"]
"#,
    )
    .expect("failed to write article schema");

    let (cfgs, _) = config::load(
        root.join("configs.toml")
            .to_str()
            .expect("configs path should be valid utf-8"),
    )
    .expect("failed to load config");
    let parsed_schema = schema::load(
        schema_dir
            .to_str()
            .expect("schema path should be valid utf-8"),
    )
    .expect("failed to load schema");

    generate_enums(&parsed_schema, &out_dir).expect("enum generation should succeed");
    generate_models(&parsed_schema, &cfgs, &out_dir).expect("model generation should succeed");

    let article_rs =
        fs::read_to_string(out_dir.join("article.rs")).expect("article.rs should exist");

    assert!(article_rs.contains("ParsedFilter::LocaleEq { column }"));
    assert!(article_rs.contains("ParsedFilter::LocaleLike { column }"));
    assert!(article_rs.contains("ParsedFilter::LocaleHas { relation, column }"));
    assert!(article_rs.contains("ParsedFilter::LocaleHasLike { relation, column }"));
    assert!(article_rs.contains("localized::ARTICLE_OWNER_TYPE"));
    assert!(article_rs.contains("f-locale-<col>"));
    assert!(article_rs.contains("f-locale-has-<relation>-<col>"));
    assert!(article_rs.contains("(\"author__profile\", \"display_name\") => Ok(Some("));

    fs::remove_dir_all(root).expect("failed to remove temp dir");
}
