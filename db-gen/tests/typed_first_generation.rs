use std::fs;
use std::path::{Path, PathBuf};
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

fn write_file(path: impl AsRef<Path>, contents: &str) {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("failed to create parent dir");
    }
    fs::write(path, contents).expect("failed to write file");
}

fn write_basic_configs(root: &Path, supported_locales: &[&str]) {
    let supported = supported_locales
        .iter()
        .map(|locale| format!("\"{locale}\""))
        .collect::<Vec<_>>()
        .join(", ");
    write_file(
        root.join("configs.toml"),
        &format!(
            "[languages]\ndefault = \"{}\"\nsupported = [{}]\n",
            supported_locales
                .first()
                .copied()
                .expect("supported locales must not be empty"),
            supported
        ),
    );
}

#[test]
fn generated_enum_filter_options_use_variant_labels_and_storage_values() {
    let root = temp_dir("enum_filter_options");
    let models_dir = root.join("models");
    let out_dir = root.join("out");
    fs::create_dir_all(&models_dir).expect("failed to create models dir");
    fs::create_dir_all(&out_dir).expect("failed to create out dir");

    write_file(
        models_dir.join("example.rs"),
        r#"
#[rf_db_enum(storage = "string")]
pub enum AdminType {
    Developer,
    SuperAdmin,
    Admin,
}

#[rf_db_enum(storage = "i16")]
pub enum ContentPageSystemFlag {
    No = 0,
    Yes = 1,
}

#[rf_model(table = "examples")]
pub struct Example {
    pub id: i64,
    pub admin_type: AdminType,
    pub is_system: ContentPageSystemFlag,
}
"#,
    );

    let parsed_schema = schema::load(
        models_dir
            .to_str()
            .expect("schema path should be valid utf-8"),
    )
    .expect("failed to load schema");

    generate_enums(&parsed_schema, &out_dir).expect("enum generation should succeed");
    let enums_rs = fs::read_to_string(out_dir.join("enums.rs")).expect("enums.rs should exist");

    assert!(enums_rs.contains("pub enum AdminType"));
    assert!(enums_rs.contains("pub enum ContentPageSystemFlag"));
    assert!(enums_rs.contains("pub struct SchemaEnumVariantMeta"));
    assert!(enums_rs.contains("pub struct SchemaEnumTsMeta"));
    assert!(enums_rs.contains("pub const SCHEMA_ENUM_TS_META: &[SchemaEnumTsMeta] = &["));
    assert!(enums_rs.contains("name: \"AdminType\""));
    assert!(enums_rs.contains("SchemaEnumVariantMeta { value: \"developer\", label: \"Developer\", i18n_key: \"enum.admin_type.developer\" }"));
    assert!(enums_rs.contains("name: \"ContentPageSystemFlag\""));
    assert!(enums_rs.contains("SchemaEnumVariantMeta { value: \"0\", label: \"No\", i18n_key: \"enum.content_page_system_flag.no\" }"));
    assert!(enums_rs.contains("let label = (*v).explained_label();"));
    assert!(enums_rs.contains("let value = (*v).as_str();"));
    assert!(enums_rs.contains("pub fn from_storage(raw: &str) -> Option<Self> {"));
    assert!(enums_rs.contains("pub const fn i18n_key(self) -> &'static str {"));
    assert!(enums_rs.contains("pub fn explained_label(self) -> String {"));
    assert!(enums_rs.contains("Self::Developer => \"Developer\","));
    assert!(enums_rs.contains("Self::Developer => \"developer\","));
    assert!(enums_rs.contains("Self::No => \"No\","));
    assert!(enums_rs.contains("Self::No => \"0\","));

    fs::remove_dir_all(root).expect("failed to remove temp dir");
}

#[test]
fn generated_models_copy_helper_items_plain_methods_and_computed_fields() {
    let root = temp_dir("custom_methods");
    let models_dir = root.join("models");
    let out_dir = root.join("out");
    fs::create_dir_all(&models_dir).expect("failed to create models dir");
    fs::create_dir_all(&out_dir).expect("failed to create out dir");
    write_basic_configs(&root, &["en"]);

    write_file(
        models_dir.join("article.rs"),
        r#"
pub struct SpecialDto {
    pub label: String,
}

pub const SPECIAL_PREFIX: &str = "special";

pub fn build_special(label: &str) -> SpecialDto {
    SpecialDto {
        label: format!("{SPECIAL_PREFIX}:{label}"),
    }
}

#[rf_db_enum(storage = "string")]
pub enum ArticleStatus {
    Draft,
    Published,
}

#[rf_model(table = "articles")]
pub struct Article {
    pub id: i64,
    pub title: Localized<String>,
    pub status: ArticleStatus,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
}

#[rf_record_impl]
impl ArticleRecord {
    pub fn to_special_dto(&self) -> SpecialDto {
        build_special(&self.identity())
    }

    pub fn is_published(&self) -> bool {
        matches!(self.status, ArticleStatus::Published)
    }

    #[rf_computed]
    pub fn identity(&self) -> String {
        format!("article:{}", self.id)
    }
}
"#,
    );

    let (cfgs, _) = config::load(
        root.join("configs.toml")
            .to_str()
            .expect("configs path should be valid utf-8"),
    )
    .expect("failed to load config");
    let parsed_schema = schema::load(
        models_dir
            .to_str()
            .expect("schema path should be valid utf-8"),
    )
    .expect("failed to load schema");

    generate_enums(&parsed_schema, &out_dir).expect("enum generation should succeed");
    generate_models(&parsed_schema, &cfgs, &out_dir).expect("model generation should succeed");

    let article_rs =
        fs::read_to_string(out_dir.join("article.rs")).expect("article.rs should exist");

    assert!(article_rs.contains("pub struct SpecialDto"));
    assert!(article_rs.contains("pub const SPECIAL_PREFIX"));
    assert!(article_rs.contains("\"special\""));
    assert!(article_rs.contains("pub fn build_special"));
    assert!(article_rs.contains("pub fn to_special_dto"));
    assert!(article_rs.contains("-> SpecialDto"));
    assert!(article_rs.contains("pub fn is_published"));
    assert!(article_rs.contains("pub fn identity"));
    assert!(article_rs.contains("use core_datatable::serialize_offset_datetime_rfc3339;"));
    assert!(article_rs.contains("record.insert(\"created_at\".to_string(), serde_json::Value::String(serialize_offset_datetime_rfc3339(row.created_at)"));
    assert!(article_rs.contains("record.insert(\"updated_at\".to_string(), serde_json::Value::String(serialize_offset_datetime_rfc3339(row.updated_at)"));
    assert!(article_rs.contains(
        "record.insert(\"identity\".to_string(), serde_json::to_value(row.identity())?);"
    ));
    assert!(!article_rs.contains("pub struct ArticleJson"));
    assert!(!article_rs.contains("crate::extensions::"));

    fs::remove_dir_all(root).expect("failed to remove temp dir");
}

#[test]
fn generated_models_trim_trailing_underscores_for_enum_explained_fields() {
    let root = temp_dir("enum_explained_suffix");
    let models_dir = root.join("models");
    let out_dir = root.join("out");
    fs::create_dir_all(&models_dir).expect("failed to create models dir");
    fs::create_dir_all(&out_dir).expect("failed to create out dir");
    write_basic_configs(&root, &["en"]);

    write_file(
        models_dir.join("job.rs"),
        r#"
#[rf_db_enum(storage = "string")]
pub enum JobType {
    Image,
}

#[rf_model(table = "jobs")]
pub struct Job {
    pub id: i64,
    pub type_: JobType,
}
"#,
    );

    let (cfgs, _) = config::load(
        root.join("configs.toml")
            .to_str()
            .expect("configs path should be valid utf-8"),
    )
    .expect("failed to load config");
    let parsed_schema = schema::load(
        models_dir
            .to_str()
            .expect("schema path should be valid utf-8"),
    )
    .expect("failed to load schema");

    generate_enums(&parsed_schema, &out_dir).expect("enum generation should succeed");
    generate_models(&parsed_schema, &cfgs, &out_dir).expect("model generation should succeed");

    let job_rs = fs::read_to_string(out_dir.join("job.rs")).expect("job.rs should exist");

    assert!(job_rs.contains("pub type_explained: String,"));
    assert!(job_rs.contains("type_explained: row.type_.explained_label(),"));
    assert!(!job_rs.contains("type__explained"));

    fs::remove_dir_all(root).expect("failed to remove temp dir");
}

#[test]
fn framework_models_load_from_core_db_single_source_of_truth() {
    let framework_paths = schema::framework_model_source_paths_from_core_db();
    let parsed_schema = schema::load_framework_from_paths(&framework_paths)
        .expect("failed to load framework model sources");

    for model in [
        "attachment",
        "localized",
        "meta",
        "country",
        "sql_profiler_request",
        "sql_profiler_query",
        "failed_job",
        "outbox_job",
        "personal_access_token",
        "webhook_log",
        "http_client_log",
    ] {
        assert!(
            parsed_schema.models.contains_key(model),
            "missing framework model: {model}"
        );
    }
}

#[test]
fn framework_models_only_generate_explicit_timestamp_fields() {
    let root = temp_dir("framework_explicit_timestamps");
    let out_dir = root.join("out");
    fs::create_dir_all(&out_dir).expect("failed to create out dir");
    write_basic_configs(&root, &["en"]);

    let framework_paths = schema::framework_model_source_paths_from_core_db();
    let parsed_schema = schema::load_framework_from_paths(&framework_paths)
        .expect("failed to load framework model sources");
    let (cfgs, _) = config::load(
        root.join("configs.toml")
            .to_str()
            .expect("configs path should be valid utf-8"),
    )
    .expect("failed to load config");

    generate_enums(&parsed_schema, &out_dir).expect("enum generation should succeed");
    generate_models(&parsed_schema, &cfgs, &out_dir).expect("model generation should succeed");

    let outbox_rs =
        fs::read_to_string(out_dir.join("outbox_job.rs")).expect("outbox_job.rs should exist");
    let failed_rs =
        fs::read_to_string(out_dir.join("failed_job.rs")).expect("failed_job.rs should exist");
    let request_rs = fs::read_to_string(out_dir.join("sql_profiler_request.rs"))
        .expect("sql_profiler_request.rs should exist");

    assert!(outbox_rs.contains("pub created_at: time::OffsetDateTime,"));
    assert!(!outbox_rs.contains("updated_at"));
    assert!(failed_rs.contains("pub failed_at: time::OffsetDateTime,"));
    assert!(!failed_rs.contains("created_at"));
    assert!(!failed_rs.contains("updated_at"));
    assert!(request_rs.contains("pub created_at: time::OffsetDateTime,"));
    assert!(!request_rs.contains("updated_at"));

    fs::remove_dir_all(root).expect("failed to remove temp dir");
}

#[test]
fn generated_models_support_nested_relation_filter_paths() {
    let root = temp_dir("nested_rel_filters");
    let models_dir = root.join("models");
    let out_dir = root.join("out");
    fs::create_dir_all(&models_dir).expect("failed to create models dir");
    fs::create_dir_all(&out_dir).expect("failed to create out dir");
    write_basic_configs(&root, &["en"]);

    write_file(
        models_dir.join("profile.rs"),
        r#"
#[rf_model(table = "profiles")]
pub struct Profile {
    pub id: i64,
    pub display_name: String,
}
"#,
    );
    write_file(
        models_dir.join("user.rs"),
        r#"
#[rf_model(table = "users")]
pub struct User {
    pub id: i64,
    pub name: String,
    pub profile_id: i64,
    #[rf(foreign_key = "profile_id")]
    pub profile: BelongsTo<Profile>,
}
"#,
    );
    write_file(
        models_dir.join("article.rs"),
        r#"
#[rf_model(table = "articles")]
pub struct Article {
    pub id: i64,
    pub title: String,
    pub author_id: i64,
    #[rf(foreign_key = "author_id")]
    pub author: BelongsTo<User>,
}
"#,
    );

    let (cfgs, _) = config::load(
        root.join("configs.toml")
            .to_str()
            .expect("configs path should be valid utf-8"),
    )
    .expect("failed to load config");
    let parsed_schema = schema::load(
        models_dir
            .to_str()
            .expect("schema path should be valid utf-8"),
    )
    .expect("failed to load schema");

    generate_enums(&parsed_schema, &out_dir).expect("enum generation should succeed");
    generate_models(&parsed_schema, &cfgs, &out_dir).expect("model generation should succeed");

    let article_rs =
        fs::read_to_string(out_dir.join("article.rs")).expect("article.rs should exist");

    assert!(article_rs
        .contains("\"author__profile\" => Self::parse_bind_for_profile_cols(column, raw)"));
    assert!(article_rs.contains(
        "Ok(Some(query.where_has(ArticleRel::AUTHOR, |rq| rq.where_has(UserRel::PROFILE, |rq| Self::filter_has_for_profile_cols(column.as_str(), rq, bind)))))"
    ));
    assert!(article_rs.contains(
        "Ok(Some(query.where_has(ArticleRel::AUTHOR, |rq| rq.where_has(UserRel::PROFILE, |rq| Self::filter_has_like_for_profile_cols(column.as_str(), rq, pattern.clone())))))"
    ));

    fs::remove_dir_all(root).expect("failed to remove temp dir");
}

#[test]
fn generated_models_handle_optional_self_has_many_without_self_imports() {
    let root = temp_dir("self_has_many");
    let models_dir = root.join("models");
    let out_dir = root.join("out");
    fs::create_dir_all(&models_dir).expect("failed to create models dir");
    fs::create_dir_all(&out_dir).expect("failed to create out dir");
    write_basic_configs(&root, &["en"]);

    write_file(
        models_dir.join("user.rs"),
        r#"
#[rf_db_enum(storage = "i16")]
pub enum UserBanStatus {
    No = 0,
    Yes = 1,
}

#[rf_model(table = "users")]
pub struct User {
    pub id: i64,
    pub username: String,
    pub introducer_user_id: Option<i64>,
    pub ban: UserBanStatus,
    pub credit_1: rust_decimal::Decimal,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
    #[rf(foreign_key = "introducer_user_id")]
    pub introducer: BelongsTo<User>,
    #[rf(foreign_key = "introducer_user_id")]
    pub downlines: HasMany<User>,
}
"#,
    );

    let (cfgs, _) = config::load(
        root.join("configs.toml")
            .to_str()
            .expect("configs path should be valid utf-8"),
    )
    .expect("failed to load config");
    let parsed_schema = schema::load(
        models_dir
            .to_str()
            .expect("schema path should be valid utf-8"),
    )
    .expect("failed to load schema");

    generate_enums(&parsed_schema, &out_dir).expect("enum generation should succeed");
    generate_models(&parsed_schema, &cfgs, &out_dir).expect("model generation should succeed");

    let user_rs = fs::read_to_string(out_dir.join("user.rs")).expect("user.rs should exist");

    assert!(!user_rs.contains("use crate::generated::models::user::{UserCol, UserQuery, UserRow};"));
    assert!(user_rs.contains(
        "fn rel_downlines_child_key(record: &UserRecord) -> Option<i64> { record.introducer_user_id.clone() }"
    ));
    assert!(user_rs.contains(
        "static REL_RUNTIME_DOWNLINES: core_db::common::model_api::HasManyRuntime<UserModel, UserModel, i64>"
    ));

    fs::remove_dir_all(root).expect("failed to remove temp dir");
}

#[test]
fn generated_models_emit_scoped_relation_load_and_metric_metadata() {
    let root = temp_dir("scoped_relations");
    let models_dir = root.join("models");
    let out_dir = root.join("out");
    fs::create_dir_all(&models_dir).expect("failed to create models dir");
    fs::create_dir_all(&out_dir).expect("failed to create out dir");
    write_basic_configs(&root, &["en"]);

    write_file(
        models_dir.join("user.rs"),
        r#"
#[rf_db_enum(storage = "i16")]
pub enum UserBanStatus {
    No = 0,
    Yes = 1,
}

#[rf_model(table = "users")]
pub struct User {
    pub id: i64,
    pub username: String,
    pub introducer_user_id: Option<i64>,
    pub ban: UserBanStatus,
    pub credit_1: rust_decimal::Decimal,
    #[rf(foreign_key = "introducer_user_id")]
    pub downlines: HasMany<User>,
    #[rf(foreign_key = "introducer_user_id", scope = active_downlines_scope)]
    pub active_downlines: HasMany<User>,
}

#[rf_model_impl]
impl UserModel {
    pub fn active_downlines_scope(query: Query<UserModel>) -> Query<UserModel> {
        query.where_col(UserCol::BAN, Op::Eq, UserBanStatus::No)
    }
}
"#,
    );

    let (cfgs, _) = config::load(
        root.join("configs.toml")
            .to_str()
            .expect("configs path should be valid utf-8"),
    )
    .expect("failed to load config");
    let parsed_schema = schema::load(
        models_dir
            .to_str()
            .expect("schema path should be valid utf-8"),
    )
    .expect("failed to load schema");

    generate_enums(&parsed_schema, &out_dir).expect("enum generation should succeed");
    generate_models(&parsed_schema, &cfgs, &out_dir).expect("model generation should succeed");

    let user_rs = fs::read_to_string(out_dir.join("user.rs")).expect("user.rs should exist");

    assert!(user_rs.contains("pub __relation_aggregates: std::collections::HashMap<String, f64>,"));
    assert!(user_rs.contains("pub fn aggregate(&self, key: &str) -> Option<f64> {"));
    assert!(user_rs.contains("pub fn sum<R, A>(&self, relation: R, target: A) -> Option<f64>"));
    assert!(user_rs.contains("pub fn avg<R, A>(&self, relation: R, target: A) -> Option<f64>"));
    assert!(user_rs.contains("pub fn min<R, A>(&self, relation: R, target: A) -> Option<f64>"));
    assert!(user_rs.contains("pub fn max<R, A>(&self, relation: R, target: A) -> Option<f64>"));
    assert!(user_rs.contains("active_downlines_scope(UserModel::query_with_base_url(base_url))"));
    assert!(user_rs.contains("WithRelationSpec { name: \"active_downlines\""));
    assert!(user_rs.contains("CountRelationSpec {"));
    assert!(user_rs.contains("name: \"active_downlines\""));
    assert!(user_rs.contains("type TargetModel = UserModel;"));
    assert!(user_rs.contains("impl core_db::common::model_api::RuntimeModel for UserModel {"));
    assert!(user_rs.contains(
        "static REL_RUNTIME_ACTIVE_DOWNLINES: core_db::common::model_api::HasManyRuntime<UserModel, UserModel, i64>"
    ));
    assert!(user_rs.contains("&REL_RUNTIME_ACTIVE_DOWNLINES,"));

    fs::remove_dir_all(root).expect("failed to remove temp dir");
}

#[test]
fn generated_models_support_has_one_relations() {
    let root = temp_dir("has_one_relations");
    let models_dir = root.join("models");
    let out_dir = root.join("out");
    fs::create_dir_all(&models_dir).expect("failed to create models dir");
    fs::create_dir_all(&out_dir).expect("failed to create out dir");
    write_basic_configs(&root, &["en"]);

    write_file(
        models_dir.join("user.rs"),
        r#"
#[rf_model(table = "users")]
pub struct User {
    pub id: i64,
    pub username: String,
    #[rf(foreign_key = "user_id")]
    pub profile: HasOne<Profile>,
}
"#,
    );

    write_file(
        models_dir.join("profile.rs"),
        r#"
#[rf_model(table = "profiles")]
pub struct Profile {
    pub id: i64,
    pub user_id: i64,
    pub bio: String,
}
"#,
    );

    let (cfgs, _) = config::load(
        root.join("configs.toml")
            .to_str()
            .expect("configs path should be valid utf-8"),
    )
    .expect("failed to load config");
    let parsed_schema = schema::load(
        models_dir
            .to_str()
            .expect("schema path should be valid utf-8"),
    )
    .expect("failed to load schema");

    generate_enums(&parsed_schema, &out_dir).expect("enum generation should succeed");
    generate_models(&parsed_schema, &cfgs, &out_dir).expect("model generation should succeed");

    let user_rs = fs::read_to_string(out_dir.join("user.rs")).expect("user.rs should exist");

    assert!(user_rs.contains("pub profile: Option<Box<ProfileRecord>>,"));
    assert!(user_rs.contains("kind: \"has_one\""));
    assert!(user_rs.contains("static REL_RUNTIME_PROFILE: core_db::common::model_api::HasOneRuntime<UserModel, ProfileModel, i64>"));
    assert!(user_rs.contains("type Target = ProfileRecord;"));

    fs::remove_dir_all(root).expect("failed to remove temp dir");
}

#[test]
fn schema_load_rejects_invalid_scoped_relation_signature() {
    let root = temp_dir("invalid_scope_signature");
    let models_dir = root.join("models");
    fs::create_dir_all(&models_dir).expect("failed to create models dir");

    write_file(
        models_dir.join("user.rs"),
        r#"
#[rf_model(table = "users")]
pub struct User {
    pub id: i64,
    pub introducer_user_id: Option<i64>,
    #[rf(foreign_key = "introducer_user_id", scope = bad_scope)]
    pub downlines: HasMany<User>,
}

#[rf_model_impl]
impl UserModel {
    pub fn bad_scope(user_id: i64) -> i64 {
        user_id
    }
}
"#,
    );

    let err = schema::load(
        models_dir
            .to_str()
            .expect("schema path should be valid utf-8"),
    )
    .expect_err("schema load should reject invalid scoped relation signature");
    let err_text = err.to_string();

    assert!(err_text
        .contains("must have signature fn bad_scope(query: Query<UserModel>) -> Query<UserModel>"));

    fs::remove_dir_all(root).expect("failed to remove temp dir");
}

#[test]
fn generate_datatable_skeletons_creates_app_stubs_without_overwriting_existing_files() {
    let root = temp_dir("datatable_stubs");
    let models_dir = root.join("models");
    let out_dir = root.join("app").join("src").join("datatables");
    fs::create_dir_all(&models_dir).expect("failed to create models dir");
    fs::create_dir_all(&out_dir).expect("failed to create out dir");

    write_file(
        models_dir.join("article.rs"),
        r#"
#[rf_model(table = "articles")]
pub struct Article {
    pub id: i64,
    pub title: String,
}
"#,
    );

    let parsed_schema = schema::load(
        models_dir
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
    write_basic_configs(&root, &["en"]);

    let (cfgs, _) = config::load(
        root.join("configs.toml")
            .to_str()
            .expect("configs path should be valid utf-8"),
    )
    .expect("failed to load config");

    let empty_schema = schema::Schema {
        models: std::collections::BTreeMap::new(),
        extra_sections: std::collections::BTreeMap::new(),
    };
    generate_auth(&cfgs, &empty_schema, &out_dir).expect("auth generation should succeed");
    let mod_rs = fs::read_to_string(out_dir.join("mod.rs")).expect("mod.rs should exist");

    assert!(mod_rs.contains("pub async fn authenticate_any_guard("));
    assert!(mod_rs.contains("pub async fn datatable_actor_from_token("));
    assert!(mod_rs.contains("let _ = (db, token);"));
    assert!(mod_rs.contains("None"));

    fs::remove_dir_all(root).expect("failed to remove temp dir");
}

#[test]
fn schema_load_rejects_invalid_computed_signature() {
    let root = temp_dir("invalid_computed");
    let models_dir = root.join("models");
    fs::create_dir_all(&models_dir).expect("failed to create models dir");

    write_file(
        models_dir.join("article.rs"),
        r#"
#[rf_model(table = "articles")]
pub struct Article {
    pub id: i64,
    pub title: String,
}

#[rf_record_impl]
impl ArticleRecord {
    #[rf_computed]
    pub fn identity(&self, prefix: &str) -> String {
        format!("{prefix}:{}", self.id)
    }
}
"#,
    );

    let err = schema::load(
        models_dir
            .to_str()
            .expect("schema path should be valid utf-8"),
    )
    .expect_err("schema load should reject invalid computed method");
    let err_text = err.to_string();

    assert!(err_text.contains("#[rf_computed] method 'identity'"));
    assert!(err_text.contains("must not take extra arguments"));

    fs::remove_dir_all(root).expect("failed to remove temp dir");
}

#[test]
fn schema_load_rejects_legacy_view_impl_attrs() {
    let root = temp_dir("legacy_view_impl");
    let models_dir = root.join("models");
    fs::create_dir_all(&models_dir).expect("failed to create models dir");

    write_file(
        models_dir.join("article.rs"),
        r#"
#[rf_model(table = "articles")]
pub struct Article {
    pub id: i64,
    pub title: String,
}

#[rf_view_impl]
impl ArticleView {
    pub fn identity(&self) -> String {
        format!("article:{}", self.id)
    }
}
"#,
    );

    let err = schema::load(
        models_dir
            .to_str()
            .expect("schema path should be valid utf-8"),
    )
    .expect_err("schema load should reject legacy custom impl attrs");
    let err_text = err.to_string();

    assert!(err_text.contains("#[rf_view_impl]"));
    assert!(err_text.contains("use #[rf_record_impl] on XxxRecord instead"));

    fs::remove_dir_all(root).expect("failed to remove temp dir");
}

#[test]
fn schema_load_rejects_unsupported_enum_storage() {
    let root = temp_dir("invalid_enum_storage");
    let models_dir = root.join("models");
    fs::create_dir_all(&models_dir).expect("failed to create models dir");

    write_file(
        models_dir.join("article.rs"),
        r#"
#[rf_db_enum(storage = "uuid")]
pub enum ArticleStatus {
    Draft,
    Published,
}

#[rf_model(table = "articles")]
pub struct Article {
    pub id: i64,
    pub status: ArticleStatus,
}
"#,
    );

    let err = schema::load(
        models_dir
            .to_str()
            .expect("schema path should be valid utf-8"),
    )
    .expect_err("schema load should reject unsupported enum storage");
    let err_text = err.to_string();

    assert!(err_text.contains("uses unsupported storage 'uuid'"));

    fs::remove_dir_all(root).expect("failed to remove temp dir");
}

#[test]
fn parse_relations_normalizes_relation_names_to_snake_case() {
    let mut parsed_schema = schema::Schema::default();
    parsed_schema
        .models
        .insert("user".to_string(), schema::ModelSpec::default());

    let cfg = schema::ModelSpec {
        relations: Some(vec![
            "AuthorProfile:belongs_to:user:author_id:id".to_string()
        ]),
        ..schema::ModelSpec::default()
    };

    let relations = schema::parse_relations(&parsed_schema, &cfg, "article", &[]);
    assert_eq!(relations.len(), 1);
    assert_eq!(relations[0].name, "author_profile");
}

#[test]
fn generated_models_emit_lifecycle_payloads_and_model_keys() {
    let root = temp_dir("observer_payloads");
    let models_dir = root.join("models");
    let out_dir = root.join("out");
    fs::create_dir_all(&models_dir).expect("failed to create models dir");
    fs::create_dir_all(&out_dir).expect("failed to create out dir");
    write_basic_configs(&root, &["en"]);

    write_file(
        models_dir.join("article.rs"),
        r#"
#[rf_db_enum(storage = "string")]
pub enum ArticleStatus {
    Draft,
    Published,
}

#[rf_model(table = "articles")]
pub struct Article {
    pub id: i64,
    pub status: ArticleStatus,
    pub title: Option<String>,
    pub created_at: time::OffsetDateTime,
    pub updated_at: time::OffsetDateTime,
}
"#,
    );

    let (cfgs, _) = config::load(
        root.join("configs.toml")
            .to_str()
            .expect("configs path should be valid utf-8"),
    )
    .expect("failed to load config");
    let parsed_schema = schema::load(
        models_dir
            .to_str()
            .expect("schema path should be valid utf-8"),
    )
    .expect("failed to load schema");

    generate_enums(&parsed_schema, &out_dir).expect("enum generation should succeed");
    generate_models(&parsed_schema, &cfgs, &out_dir).expect("model generation should succeed");

    let article_rs =
        fs::read_to_string(out_dir.join("article.rs")).expect("article.rs should exist");
    let mod_rs = fs::read_to_string(out_dir.join("mod.rs")).expect("mod.rs should exist");

    assert!(article_rs.contains("pub struct ArticleCreate"));
    assert!(article_rs.contains("pub struct ArticleChanges"));
    assert!(article_rs.contains("pub const MODEL_KEY: &'static str = \"article\";"));
    assert!(article_rs.contains("const OBSERVE_HOOKS: bool = true;"));
    assert!(article_rs.contains("const USE_SNOWFLAKE_ID: bool = true;"));
    assert!(article_rs.contains("fn build_create_input(state: &CreateState<'_>)"));
    assert!(article_rs.contains("fn apply_create_overrides(mut state: CreateState<'_>, overrides: serde_json::Value)"));
    assert!(article_rs.contains("impl core_db::common::model_api::FeaturePersistenceModel for ArticleModel"));
    assert!(article_rs.contains("fn build_patch_changes(state: &PatchState<'_>)"));
    assert!(article_rs.contains("fn apply_patch_overrides<'db>(mut state: PatchState<'db>, overrides: serde_json::Value)"));
    assert!(article_rs.contains("fn row_pk("));
    assert!(article_rs.contains("fn row_pk_text(row: &<Self as core_db::common::model_api::RuntimeModel>::Row) -> String"));
    assert!(article_rs.contains("impl core_db::common::model_api::DeleteModel for ArticleModel"));
    assert!(article_rs.contains("fn delete_override_update<'db>("));
    assert!(article_rs.contains("let value = match bind {"));
    assert!(article_rs.contains("FieldInput::Set(value)"));
    assert!(article_rs.contains("FieldChange::Assign(value)"));
    assert!(mod_rs.contains("ArticleModel"));
    assert!(mod_rs.contains("ArticleRecord"));
    assert!(mod_rs.contains("ArticleCreate"));
    assert!(mod_rs.contains("ArticleChanges"));
    assert!(mod_rs.contains("ArticleCol"));
    assert!(!mod_rs.contains("ArticleCreateInput"));
    assert!(!mod_rs.contains("ArticleUpdateChanges"));
    assert!(!mod_rs.contains("ArticleRow"));

    fs::remove_dir_all(root).expect("failed to remove temp dir");
}

#[test]
fn observe_false_suppresses_generated_observer_hooks() {
    let root = temp_dir("observe_false");
    let models_dir = root.join("models");
    let out_dir = root.join("out");
    fs::create_dir_all(&models_dir).expect("failed to create models dir");
    fs::create_dir_all(&out_dir).expect("failed to create out dir");
    write_basic_configs(&root, &["en"]);

    write_file(
        models_dir.join("session.rs"),
        r#"
#[rf_model(table = "sessions", observe = false)]
pub struct Session {
    pub id: String,
    pub user_id: i64,
    pub created_at: time::OffsetDateTime,
}
"#,
    );

    let (cfgs, _) = config::load(
        root.join("configs.toml")
            .to_str()
            .expect("configs path should be valid utf-8"),
    )
    .expect("failed to load config");
    let parsed_schema = schema::load(
        models_dir
            .to_str()
            .expect("schema path should be valid utf-8"),
    )
    .expect("failed to load schema");

    generate_models(&parsed_schema, &cfgs, &out_dir).expect("model generation should succeed");

    let session_rs =
        fs::read_to_string(out_dir.join("session.rs")).expect("session.rs should exist");

    assert!(session_rs.contains("pub const MODEL_KEY: &'static str = \"session\";"));
    assert!(!session_rs.contains("try_get_observer"));
    assert!(!session_rs.contains("ModelEvent"));
    assert!(!session_rs.contains("observer.on_creating"));
    assert!(!session_rs.contains("observer.on_created"));
    assert!(!session_rs.contains("observer.on_updating"));
    assert!(!session_rs.contains("observer.on_updated"));
    assert!(!session_rs.contains("observer.on_deleting"));
    assert!(!session_rs.contains("observer.on_deleted"));

    fs::remove_dir_all(root).expect("failed to remove temp dir");
}
