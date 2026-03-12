pub mod audit_log;
pub mod account;
pub mod bank;
pub mod company_bank_account;
pub mod company_crypto_account;
pub mod content_page;
pub mod country;
pub mod crypto_network;
pub mod deposit;
pub mod http_client_log;
pub mod introducer_change;
pub mod sql_profiler_query;
pub mod sql_profiler_request;
pub mod user;
pub mod user_credit_transaction;
pub mod webhook_log;
pub mod withdrawal;

use std::collections::HashSet;

use core_datatable::{DataTableContext, DataTableExportMode, DataTableInput, DataTableRegistry};
use core_web::authz::{has_required_permissions, PermissionMode};
use core_web::openapi::ApiRouter;
use generated::permissions::Permission;

use crate::contracts::datatable::admin::{
    audit_log::{ROUTE_PREFIX as AUDIT_LOG_ROUTE_PREFIX, SCOPED_KEY as AUDIT_LOG_SCOPED_KEY},
    account::{ROUTE_PREFIX as ACCOUNT_ROUTE_PREFIX, SCOPED_KEY as ACCOUNT_SCOPED_KEY},
    bank::{ROUTE_PREFIX as BANK_ROUTE_PREFIX, SCOPED_KEY as BANK_SCOPED_KEY},
    company_bank_account::{
        ROUTE_PREFIX as COMPANY_BANK_ACCOUNT_ROUTE_PREFIX,
        SCOPED_KEY as COMPANY_BANK_ACCOUNT_SCOPED_KEY,
    },
    company_crypto_account::{
        ROUTE_PREFIX as COMPANY_CRYPTO_ACCOUNT_ROUTE_PREFIX,
        SCOPED_KEY as COMPANY_CRYPTO_ACCOUNT_SCOPED_KEY,
    },
    content_page::{
        ROUTE_PREFIX as CONTENT_PAGE_ROUTE_PREFIX, SCOPED_KEY as CONTENT_PAGE_SCOPED_KEY,
    },
    country::{ROUTE_PREFIX as COUNTRY_ROUTE_PREFIX, SCOPED_KEY as COUNTRY_SCOPED_KEY},
    crypto_network::{
        ROUTE_PREFIX as CRYPTO_NETWORK_ROUTE_PREFIX, SCOPED_KEY as CRYPTO_NETWORK_SCOPED_KEY,
    },
    deposit::{ROUTE_PREFIX as DEPOSIT_ROUTE_PREFIX, SCOPED_KEY as DEPOSIT_SCOPED_KEY},
    http_client_log::{
        ROUTE_PREFIX as HTTP_CLIENT_LOG_ROUTE_PREFIX, SCOPED_KEY as HTTP_CLIENT_LOG_SCOPED_KEY,
    },
    introducer_change::{
        ROUTE_PREFIX as INTRODUCER_CHANGE_ROUTE_PREFIX,
        SCOPED_KEY as INTRODUCER_CHANGE_SCOPED_KEY,
    },
    sql_profiler_query::{
        ROUTE_PREFIX as SQL_PROFILER_QUERY_ROUTE_PREFIX,
        SCOPED_KEY as SQL_PROFILER_QUERY_SCOPED_KEY,
    },
    sql_profiler_request::{
        ROUTE_PREFIX as SQL_PROFILER_REQUEST_ROUTE_PREFIX,
        SCOPED_KEY as SQL_PROFILER_REQUEST_SCOPED_KEY,
    },
    user::{ROUTE_PREFIX as USER_ROUTE_PREFIX, SCOPED_KEY as USER_SCOPED_KEY},
    user_credit_transaction::{
        ROUTE_PREFIX as USER_CREDIT_TRANSACTION_ROUTE_PREFIX,
        SCOPED_KEY as USER_CREDIT_TRANSACTION_SCOPED_KEY,
    },
    webhook_log::{ROUTE_PREFIX as WEBHOOK_LOG_ROUTE_PREFIX, SCOPED_KEY as WEBHOOK_LOG_SCOPED_KEY},
    withdrawal::{ROUTE_PREFIX as WITHDRAWAL_ROUTE_PREFIX, SCOPED_KEY as WITHDRAWAL_SCOPED_KEY},
};
use crate::internal::api::state::AppApiState;

pub use audit_log::AuditLogDataTableAppHooks;
pub use account::{build_admin_summary_output, AdminDataTableAppHooks};
pub use bank::BankDataTableAppHooks;
pub use company_bank_account::CompanyBankAccountDataTableAppHooks;
pub use company_crypto_account::CompanyCryptoAccountDataTableAppHooks;
pub use content_page::ContentPageDataTableAppHooks;
pub use country::CountryDataTableAppHooks;
pub use crypto_network::CryptoNetworkDataTableAppHooks;
pub use deposit::DepositDataTableAppHooks;
pub use http_client_log::HttpClientLogDataTableAppHooks;
pub use user::{build_user_summary_output, UserDataTableAppHooks};
pub use user_credit_transaction::UserCreditTransactionDataTableAppHooks;
pub use introducer_change::IntroducerChangeDataTableAppHooks;
pub use sql_profiler_query::SqlProfilerQueryDataTableAppHooks;
pub use sql_profiler_request::SqlProfilerRequestDataTableAppHooks;
pub use webhook_log::WebhookLogDataTableAppHooks;
pub use withdrawal::WithdrawalDataTableAppHooks;

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

fn bank_routes(state: AppApiState) -> ApiRouter {
    bank::routes(state)
}

fn crypto_network_routes(state: AppApiState) -> ApiRouter {
    crypto_network::routes(state)
}

fn company_bank_account_routes(state: AppApiState) -> ApiRouter {
    company_bank_account::routes(state)
}

fn company_crypto_account_routes(state: AppApiState) -> ApiRouter {
    company_crypto_account::routes(state)
}

fn audit_log_routes(state: AppApiState) -> ApiRouter {
    audit_log::routes(state)
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

fn user_routes(state: AppApiState) -> ApiRouter {
    user::routes(state)
}

fn user_credit_transaction_routes(state: AppApiState) -> ApiRouter {
    user_credit_transaction::routes(state)
}

fn introducer_change_routes(state: AppApiState) -> ApiRouter {
    introducer_change::routes(state)
}

fn sql_profiler_query_routes(state: AppApiState) -> ApiRouter {
    sql_profiler_query::routes(state)
}

fn sql_profiler_request_routes(state: AppApiState) -> ApiRouter {
    sql_profiler_request::routes(state)
}

fn deposit_routes(state: AppApiState) -> ApiRouter {
    deposit::routes(state)
}

fn webhook_log_routes(state: AppApiState) -> ApiRouter {
    webhook_log::routes(state)
}

fn withdrawal_routes(state: AppApiState) -> ApiRouter {
    withdrawal::routes(state)
}

pub static ADMIN_SCOPED_DATATABLES: &[ScopedDatatableSpec] = &[
    ScopedDatatableSpec {
        scoped_key: AUDIT_LOG_SCOPED_KEY,
        route_prefix: AUDIT_LOG_ROUTE_PREFIX,
        register: audit_log::register_scoped,
        mount_routes: audit_log_routes,
    },
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
    ScopedDatatableSpec {
        scoped_key: USER_SCOPED_KEY,
        route_prefix: USER_ROUTE_PREFIX,
        register: user::register_scoped,
        mount_routes: user_routes,
    },
    ScopedDatatableSpec {
        scoped_key: USER_CREDIT_TRANSACTION_SCOPED_KEY,
        route_prefix: USER_CREDIT_TRANSACTION_ROUTE_PREFIX,
        register: user_credit_transaction::register_scoped,
        mount_routes: user_credit_transaction_routes,
    },
    ScopedDatatableSpec {
        scoped_key: INTRODUCER_CHANGE_SCOPED_KEY,
        route_prefix: INTRODUCER_CHANGE_ROUTE_PREFIX,
        register: introducer_change::register_scoped,
        mount_routes: introducer_change_routes,
    },
    ScopedDatatableSpec {
        scoped_key: SQL_PROFILER_REQUEST_SCOPED_KEY,
        route_prefix: SQL_PROFILER_REQUEST_ROUTE_PREFIX,
        register: sql_profiler_request::register_scoped,
        mount_routes: sql_profiler_request_routes,
    },
    ScopedDatatableSpec {
        scoped_key: SQL_PROFILER_QUERY_SCOPED_KEY,
        route_prefix: SQL_PROFILER_QUERY_ROUTE_PREFIX,
        register: sql_profiler_query::register_scoped,
        mount_routes: sql_profiler_query_routes,
    },
    ScopedDatatableSpec {
        scoped_key: BANK_SCOPED_KEY,
        route_prefix: BANK_ROUTE_PREFIX,
        register: bank::register_scoped,
        mount_routes: bank_routes,
    },
    ScopedDatatableSpec {
        scoped_key: CRYPTO_NETWORK_SCOPED_KEY,
        route_prefix: CRYPTO_NETWORK_ROUTE_PREFIX,
        register: crypto_network::register_scoped,
        mount_routes: crypto_network_routes,
    },
    ScopedDatatableSpec {
        scoped_key: COMPANY_BANK_ACCOUNT_SCOPED_KEY,
        route_prefix: COMPANY_BANK_ACCOUNT_ROUTE_PREFIX,
        register: company_bank_account::register_scoped,
        mount_routes: company_bank_account_routes,
    },
    ScopedDatatableSpec {
        scoped_key: COMPANY_CRYPTO_ACCOUNT_SCOPED_KEY,
        route_prefix: COMPANY_CRYPTO_ACCOUNT_ROUTE_PREFIX,
        register: company_crypto_account::register_scoped,
        mount_routes: company_crypto_account_routes,
    },
    ScopedDatatableSpec {
        scoped_key: DEPOSIT_SCOPED_KEY,
        route_prefix: DEPOSIT_ROUTE_PREFIX,
        register: deposit::register_scoped,
        mount_routes: deposit_routes,
    },
    ScopedDatatableSpec {
        scoped_key: WITHDRAWAL_SCOPED_KEY,
        route_prefix: WITHDRAWAL_ROUTE_PREFIX,
        register: withdrawal::register_scoped,
        mount_routes: withdrawal_routes,
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
