use core_db::common::sql::{DbConn, Op};
use core_db::platform::attachments::types::AttachmentInput;
use core_i18n::t;
use core_web::error::AppError;
use generated::models::{Bank, BankQuery, BankWithRelations};
use time::OffsetDateTime;

use crate::{
    contracts::api::v1::admin::bank::AdminBankInput,
    internal::api::state::AppApiState,
};

pub async fn detail(state: &AppApiState, id: i64) -> Result<BankWithRelations, AppError> {
    BankQuery::new(DbConn::pool(&state.db), None)
        .where_id(Op::Eq, id)
        .first()
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound(t("Bank not found")))
}

pub async fn create(
    state: &AppApiState,
    req: AdminBankInput,
    logo: Option<AttachmentInput>,
) -> Result<BankWithRelations, AppError> {
    let country_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM countries WHERE iso2 = $1)",
    )
    .bind(&req.country_iso2)
    .fetch_one(&state.db)
    .await
    .map_err(AppError::from)?;

    if !country_exists {
        return Err(AppError::BadRequest(t("Country not found")));
    }

    let now = OffsetDateTime::now_utc();
    let mut insert = Bank::new(DbConn::pool(&state.db), None)
        .insert()
        .set_country_iso2(req.country_iso2)
        .set_name(req.name)
        .set_code(req.code)
        .set_status(req.status)
        .set_sort_order(req.sort_order.unwrap_or(0))
        .set_created_at(now)
        .set_updated_at(now);

    if let Some(logo) = logo {
        insert = insert.set_attachment_logo(logo);
    }

    let row = insert.save().await.map_err(AppError::from)?;

    detail(state, row.id).await
}

pub async fn update(
    state: &AppApiState,
    id: i64,
    req: AdminBankInput,
    logo: Option<AttachmentInput>,
) -> Result<BankWithRelations, AppError> {
    let country_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM countries WHERE iso2 = $1)",
    )
    .bind(&req.country_iso2)
    .fetch_one(&state.db)
    .await
    .map_err(AppError::from)?;

    if !country_exists {
        return Err(AppError::BadRequest(t("Country not found")));
    }

    let mut update = Bank::new(DbConn::pool(&state.db), None)
        .update()
        .where_id(Op::Eq, id)
        .set_country_iso2(req.country_iso2)
        .set_name(req.name)
        .set_code(req.code)
        .set_status(req.status)
        .set_sort_order(req.sort_order.unwrap_or(0))
        .set_updated_at(OffsetDateTime::now_utc());

    if let Some(logo) = logo {
        update = update.set_attachment_logo(logo);
    }

    let affected = update.save().await.map_err(AppError::from)?;

    if affected == 0 {
        return Err(AppError::NotFound(t("Bank not found")));
    }

    detail(state, id).await
}

pub async fn delete(state: &AppApiState, id: i64) -> Result<(), AppError> {
    let affected = Bank::new(DbConn::pool(&state.db), None)
        .delete(id)
        .await
        .map_err(AppError::from)?;

    if affected == 0 {
        return Err(AppError::NotFound(t("Bank not found")));
    }

    Ok(())
}
