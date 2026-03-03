use core_datatable::{DataTableContext, DataTableInput, DataTableRegistry};
use generated::models::{WebhookLogDataTable, WebhookLogDataTableConfig, WebhookLogDataTableHooks};

#[derive(Default, Clone)]
pub struct WebhookLogDataTableAppHooks;

impl WebhookLogDataTableHooks for WebhookLogDataTableAppHooks {
    fn authorize(&self, _input: &DataTableInput, ctx: &DataTableContext) -> anyhow::Result<bool> {
        Ok(is_developer_actor(ctx))
    }
}

fn is_developer_actor(ctx: &DataTableContext) -> bool {
    ctx.actor
        .as_ref()
        .and_then(|actor| actor.attributes.get("admin_type"))
        .and_then(|value| value.as_str())
        .is_some_and(|admin_type| admin_type.eq_ignore_ascii_case("developer"))
}

pub type AppWebhookLogDataTable = WebhookLogDataTable<WebhookLogDataTableAppHooks>;

pub fn app_webhook_log_datatable(db: sqlx::PgPool) -> AppWebhookLogDataTable {
    WebhookLogDataTable::new(db).with_hooks(WebhookLogDataTableAppHooks::default())
}

pub fn app_webhook_log_datatable_with_config(
    db: sqlx::PgPool,
    config: WebhookLogDataTableConfig,
) -> AppWebhookLogDataTable {
    WebhookLogDataTable::new(db)
        .with_hooks(WebhookLogDataTableAppHooks::default())
        .with_config(config)
}

pub fn register_webhook_log_datatable(registry: &mut DataTableRegistry, db: sqlx::PgPool) {
    registry.register(app_webhook_log_datatable(db));
}
