use core_datatable::{DataTableContext, DataTableInput, DataTableRegistry};
use core_db::common::sql::{Op, RawClause};
use core_web::authz::{has_required_permissions, PermissionMode};
use core_web::datatable::{
    routes_for_scoped_contract_with_options, DataTableRouteOptions, DataTableRouteState,
};
use core_web::openapi::ApiRouter;
use generated::{
    models::{User, UserBanStatus, UserCol, UserDataTable, UserDataTableHooks, UserQuery},
    permissions::Permission,
};

use crate::contracts::datatable::admin::user::{
    AdminUserDataTableContract, UserDatatableSummaryOutput, ROUTE_PREFIX, SCOPED_KEY,
};
use crate::internal::datatables::v1::admin::authorize_with_optional_export;

#[derive(Default, Clone)]
pub struct UserDataTableAppHooks;

impl UserDataTableHooks for UserDataTableAppHooks {
    fn scope<'db>(
        &'db self,
        query: UserQuery<'db>,
        _input: &DataTableInput,
        _ctx: &DataTableContext,
    ) -> UserQuery<'db> {
        query
    }

    fn authorize(&self, input: &DataTableInput, ctx: &DataTableContext) -> anyhow::Result<bool> {
        let Some(actor) = ctx.actor.as_ref() else {
            return Ok(false);
        };
        let base_authorized = has_required_permissions(
            &actor.permissions,
            &[
                Permission::UserRead.as_str(),
                Permission::UserManage.as_str(),
            ],
            PermissionMode::Any,
        );

        Ok(authorize_with_optional_export(base_authorized, input, ctx))
    }

    fn filter_query<'db>(
        &'db self,
        query: UserQuery<'db>,
        filter_key: &str,
        value: &str,
        _input: &DataTableInput,
        _ctx: &DataTableContext,
    ) -> anyhow::Result<Option<UserQuery<'db>>> {
        match filter_key {
            "q" => Ok(Some(apply_keyword_filter(query, value))),
            _ => Ok(None),
        }
    }

    fn row_to_record(
        &self,
        row: generated::models::UserWithRelations,
        _input: &DataTableInput,
        _ctx: &DataTableContext,
    ) -> anyhow::Result<serde_json::Map<String, serde_json::Value>> {
        let mut record = self.default_row_to_record(row)?;
        record.remove("password");
        Ok(record)
    }
}

fn parse_user_ban_status(value: &str) -> Option<UserBanStatus> {
    UserBanStatus::from_storage(value)
}

fn apply_keyword_filter<'db>(query: UserQuery<'db>, value: &str) -> UserQuery<'db> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return query;
    }
    let pattern = format!("%{trimmed}%");
    query.where_group(|q| {
        q.where_col(UserCol::Username, Op::Like, pattern.clone())
            .or_where_col(UserCol::Name, Op::Like, pattern.clone())
            .or_where_col(UserCol::Email, Op::Like, pattern)
    })
}

fn parse_datetime(raw: &str, end_of_day: bool) -> Option<time::OffsetDateTime> {
    let trimmed = raw.trim();
    if let Ok(dt) =
        time::OffsetDateTime::parse(trimmed, &time::format_description::well_known::Rfc3339)
    {
        return Some(dt);
    }
    if trimmed.len() == 10 {
        let date = time::Date::parse(
            trimmed,
            &time::macros::format_description!("[year]-[month]-[day]"),
        )
        .ok()?;
        let t = if end_of_day {
            time::Time::from_hms(23, 59, 59).ok()?
        } else {
            time::Time::MIDNIGHT
        };
        return Some(date.with_time(t).assume_offset(time::UtcOffset::UTC));
    }
    None
}

fn apply_summary_filters<'db>(
    mut query: UserQuery<'db>,
    input: &DataTableInput,
) -> UserQuery<'db> {
    for (key, value) in input.filter_entries() {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            continue;
        }
        match key {
            "f-like-email" => {
                query = query.where_col(UserCol::Email, Op::Like, format!("%{trimmed}%"));
            }
            "f-like-username" => {
                query = query.where_col(UserCol::Username, Op::Like, format!("%{trimmed}%"));
            }
            "f-like-country_iso2" => {
                query = query.where_col(UserCol::CountryIso2, Op::Like, format!("%{trimmed}%"));
            }
            "f-ban" => {
                if let Some(ban) = parse_user_ban_status(trimmed) {
                    query = query.where_ban(Op::Eq, ban);
                }
            }
            "f-like-introducer" => {
                let pattern = format!("%{trimmed}%");
                if let Ok(clause) = RawClause::new(
                    "introducer_user_id IN (SELECT id FROM users WHERE username LIKE ?)",
                    [pattern],
                ) {
                    query = query.unsafe_sql().where_raw(clause).done();
                }
            }
            "f-date-from-created_at" => {
                if let Some(ts) = parse_datetime(trimmed, false) {
                    query = query.where_col(UserCol::CreatedAt, Op::Ge, ts);
                }
            }
            "f-date-to-created_at" => {
                if let Some(ts) = parse_datetime(trimmed, true) {
                    query = query.where_col(UserCol::CreatedAt, Op::Le, ts);
                }
            }
            _ => {}
        }
    }

    for (key, value) in input.custom_filter_entries() {
        if key == "q" {
            query = apply_keyword_filter(query, value);
        }
    }

    query
}

pub async fn build_user_summary_output(
    db: &sqlx::PgPool,
    input: &DataTableInput,
    _ctx: &DataTableContext,
) -> anyhow::Result<UserDatatableSummaryOutput> {
    let base = User::new(db, None).query();
    let filtered = apply_summary_filters(base, input);

    let total_filtered = filtered.clone().count().await?;
    let banned_count = filtered
        .where_ban(Op::Eq, UserBanStatus::Yes)
        .count()
        .await?;

    let total_credit_1: rust_decimal::Decimal = sqlx::query_scalar(
        "SELECT COALESCE(SUM(credit_1), 0) FROM users",
    )
    .fetch_one(db)
    .await
    .unwrap_or(rust_decimal::Decimal::ZERO);

    let total_credit_2: rust_decimal::Decimal = sqlx::query_scalar(
        "SELECT COALESCE(SUM(credit_2), 0) FROM users",
    )
    .fetch_one(db)
    .await
    .unwrap_or(rust_decimal::Decimal::ZERO);

    Ok(UserDatatableSummaryOutput {
        total_user_count: total_filtered,
        total_filtered,
        banned_count,
        total_credit_1,
        total_credit_2,
    })
}

pub type AppUserDataTable = UserDataTable<UserDataTableAppHooks>;

pub fn app_user_datatable(db: sqlx::PgPool) -> AppUserDataTable {
    UserDataTable::new(db).with_hooks(UserDataTableAppHooks::default())
}

pub fn register_scoped(registry: &mut DataTableRegistry, db: sqlx::PgPool) {
    registry.register_as(SCOPED_KEY, app_user_datatable(db));
}

pub fn routes<S>(state: S) -> ApiRouter
where
    S: DataTableRouteState,
{
    routes_for_scoped_contract_with_options(
        ROUTE_PREFIX,
        state,
        AdminUserDataTableContract,
        DataTableRouteOptions {
            require_bearer_auth: true,
        },
    )
}
