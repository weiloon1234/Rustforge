use core_datatable::{DataTableContext, DataTableInput, DataTableRegistry};
use core_web::authz::{has_required_permissions, PermissionMode};
use core_web::datatable::{
    routes_for_scoped_contract_with_options, DataTableRouteOptions, DataTableRouteState,
};
use core_web::openapi::ApiRouter;
use generated::{
    models::{ContentPageDataTable, ContentPageDataTableHooks},
    permissions::Permission,
    DEFAULT_LOCALE, SUPPORTED_LOCALES,
};

use crate::contracts::datatable::admin::content_page::{
    AdminContentPageDataTableContract, ROUTE_PREFIX, SCOPED_KEY,
};
use crate::internal::datatables::v1::admin::authorize_with_optional_export;

#[derive(Default, Clone)]
pub struct ContentPageDataTableAppHooks;

impl ContentPageDataTableHooks for ContentPageDataTableAppHooks {
    fn authorize(&self, input: &DataTableInput, ctx: &DataTableContext) -> anyhow::Result<bool> {
        let Some(actor) = ctx.actor.as_ref() else {
            return Ok(false);
        };
        let base_authorized = has_required_permissions(
            &actor.permissions,
            &[
                Permission::ContentPageRead.as_str(),
                Permission::ContentPageManage.as_str(),
            ],
            PermissionMode::Any,
        );

        Ok(authorize_with_optional_export(base_authorized, input, ctx))
    }

    fn mappings(
        &self,
        record: &mut serde_json::Map<String, serde_json::Value>,
        _input: &DataTableInput,
        _ctx: &DataTableContext,
    ) -> anyhow::Result<()> {
        if let Some(id_value) = record.get("id").cloned() {
            let id_text = match id_value {
                serde_json::Value::Number(number) => number.to_string(),
                serde_json::Value::String(text) => text,
                other => other.to_string(),
            };
            record.insert("id".to_string(), serde_json::Value::String(id_text));
        }

        if let Some(is_system_value) = record.get("is_system").cloned() {
            let is_system_yes = match is_system_value {
                serde_json::Value::Number(number) => number.as_i64() == Some(1),
                serde_json::Value::String(text) => {
                    let normalized = text.trim().to_ascii_lowercase();
                    matches!(normalized.as_str(), "1" | "yes" | "true")
                }
                serde_json::Value::Bool(flag) => flag,
                _ => false,
            };
            record.insert(
                "is_system".to_string(),
                serde_json::Value::String(if is_system_yes { "1" } else { "0" }.to_string()),
            );
        }

        let title = record
            .get("title_translations")
            .and_then(|value| value.as_object())
            .and_then(resolve_translated_title);

        record.insert(
            "title".to_string(),
            title
                .map(serde_json::Value::String)
                .unwrap_or(serde_json::Value::Null),
        );

        record.remove("title_translations");
        record.remove("content_translations");
        record.remove("cover_translations");
        record.remove("created_at");
        record.remove("deleted_at");

        Ok(())
    }
}

fn resolve_translated_title(
    localized_values: &serde_json::Map<String, serde_json::Value>,
) -> Option<String> {
    let preferred = localized_values
        .get(DEFAULT_LOCALE)
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string);
    if preferred.is_some() {
        return preferred;
    }

    for &locale in SUPPORTED_LOCALES {
        if let Some(value) = localized_values
            .get(locale)
            .and_then(|item| item.as_str())
            .map(str::trim)
            .filter(|item| !item.is_empty())
            .map(ToString::to_string)
        {
            return Some(value);
        }
    }

    None
}

pub type AppContentPageDataTable = ContentPageDataTable<ContentPageDataTableAppHooks>;

pub fn app_content_page_datatable(db: sqlx::PgPool) -> AppContentPageDataTable {
    ContentPageDataTable::new(db).with_hooks(ContentPageDataTableAppHooks::default())
}

pub fn register_scoped(registry: &mut DataTableRegistry, db: sqlx::PgPool) {
    registry.register_as(SCOPED_KEY, app_content_page_datatable(db));
}

pub fn routes<S>(state: S) -> ApiRouter
where
    S: DataTableRouteState,
{
    routes_for_scoped_contract_with_options(
        ROUTE_PREFIX,
        state,
        AdminContentPageDataTableContract,
        DataTableRouteOptions {
            require_bearer_auth: true,
        },
    )
}
