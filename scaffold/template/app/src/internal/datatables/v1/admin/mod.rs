pub mod account;
pub mod content_page;
pub mod country;
pub mod http_client_log;
pub mod webhook_log;

use std::collections::HashSet;

use core_datatable::{DataTableContext, DataTableExportMode, DataTableInput, DataTableRegistry};
use core_web::authz::{has_required_permissions, PermissionMode};
use core_web::openapi::ApiRouter;
use generated::permissions::Permission;

use crate::contracts::datatable::admin::{
    account::{ROUTE_PREFIX as ACCOUNT_ROUTE_PREFIX, SCOPED_KEY as ACCOUNT_SCOPED_KEY},
    content_page::{
        ROUTE_PREFIX as CONTENT_PAGE_ROUTE_PREFIX, SCOPED_KEY as CONTENT_PAGE_SCOPED_KEY,
    },
    country::{ROUTE_PREFIX as COUNTRY_ROUTE_PREFIX, SCOPED_KEY as COUNTRY_SCOPED_KEY},
    http_client_log::{
        ROUTE_PREFIX as HTTP_CLIENT_LOG_ROUTE_PREFIX, SCOPED_KEY as HTTP_CLIENT_LOG_SCOPED_KEY,
    },
    webhook_log::{ROUTE_PREFIX as WEBHOOK_LOG_ROUTE_PREFIX, SCOPED_KEY as WEBHOOK_LOG_SCOPED_KEY},
};
use crate::internal::api::state::AppApiState;

pub use account::{build_admin_summary_output, AdminDataTableAppHooks};
pub use content_page::ContentPageDataTableAppHooks;
pub use country::CountryDataTableAppHooks;
pub use http_client_log::HttpClientLogDataTableAppHooks;
pub use webhook_log::WebhookLogDataTableAppHooks;

pub fn authorize_with_optional_export(
    base_authorized: bool,
    input: &DataTableInput,
    ctx: &DataTableContext,
) -> bool {
    if !base_authorized {
        return false;
    }

    if matches!(input.export, DataTableExportMode::None) {
        return true;
    }

    let Some(actor) = ctx.actor.as_ref() else {
        return false;
    };

    has_required_permissions(
        &actor.permissions,
        &[Permission::Export.as_str()],
        PermissionMode::All,
    )
}

pub struct ScopedDatatableSpec {
    pub scoped_key: &'static str,
    pub route_prefix: &'static str,
    pub register: fn(&mut DataTableRegistry, sqlx::PgPool),
    pub mount_routes: fn(AppApiState) -> ApiRouter,
}

fn account_routes(state: AppApiState) -> ApiRouter {
    account::routes(state)
}

fn content_page_routes(state: AppApiState) -> ApiRouter {
    content_page::routes(state)
}

fn country_routes(state: AppApiState) -> ApiRouter {
    country::routes(state)
}

fn http_client_log_routes(state: AppApiState) -> ApiRouter {
    http_client_log::routes(state)
}

fn webhook_log_routes(state: AppApiState) -> ApiRouter {
    webhook_log::routes(state)
}

pub static ADMIN_SCOPED_DATATABLES: &[ScopedDatatableSpec] = &[
    ScopedDatatableSpec {
        scoped_key: ACCOUNT_SCOPED_KEY,
        route_prefix: ACCOUNT_ROUTE_PREFIX,
        register: account::register_scoped,
        mount_routes: account_routes,
    },
    ScopedDatatableSpec {
        scoped_key: HTTP_CLIENT_LOG_SCOPED_KEY,
        route_prefix: HTTP_CLIENT_LOG_ROUTE_PREFIX,
        register: http_client_log::register_scoped,
        mount_routes: http_client_log_routes,
    },
    ScopedDatatableSpec {
        scoped_key: WEBHOOK_LOG_SCOPED_KEY,
        route_prefix: WEBHOOK_LOG_ROUTE_PREFIX,
        register: webhook_log::register_scoped,
        mount_routes: webhook_log_routes,
    },
    ScopedDatatableSpec {
        scoped_key: CONTENT_PAGE_SCOPED_KEY,
        route_prefix: CONTENT_PAGE_ROUTE_PREFIX,
        register: content_page::register_scoped,
        mount_routes: content_page_routes,
    },
    ScopedDatatableSpec {
        scoped_key: COUNTRY_SCOPED_KEY,
        route_prefix: COUNTRY_ROUTE_PREFIX,
        register: country::register_scoped,
        mount_routes: country_routes,
    },
];

fn assert_catalog_valid() {
    let mut scoped_keys = HashSet::new();
    let mut route_prefixes = HashSet::new();

    for spec in ADMIN_SCOPED_DATATABLES {
        assert!(
            scoped_keys.insert(spec.scoped_key),
            "duplicate datatable scoped key in catalog: {}",
            spec.scoped_key
        );
        assert!(
            route_prefixes.insert(spec.route_prefix),
            "duplicate datatable route prefix in catalog: {}",
            spec.route_prefix
        );
    }
}

pub fn register_scoped_datatables(registry: &mut DataTableRegistry, db: &sqlx::PgPool) {
    assert_catalog_valid();
    for spec in ADMIN_SCOPED_DATATABLES {
        (spec.register)(registry, db.clone());
    }
}

pub fn mount_scoped_datatable_routes(state: AppApiState) -> ApiRouter {
    assert_catalog_valid();
    let mut router = ApiRouter::new();
    for spec in ADMIN_SCOPED_DATATABLES {
        router = router.merge((spec.mount_routes)(state.clone()));
    }
    router
}
