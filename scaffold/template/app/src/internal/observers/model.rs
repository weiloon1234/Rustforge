use anyhow::Context;
use core_db::common::model_observer::{ModelEvent, ModelObserver};
use generated::models::{
    AdminModel, AdminCreate, AdminRecord, AdminChanges, BankModel, BankCreate, BankRecord,
    BankChanges, CompanyBankAccountModel, CompanyBankAccountCreate, CompanyBankAccountRecord,
    CompanyBankAccountChanges, CompanyCryptoAccountModel, CompanyCryptoAccountCreate,
    CompanyCryptoAccountRecord, CompanyCryptoAccountChanges, ContentPageModel,
    ContentPageCreate, ContentPageRecord, ContentPageChanges, CountryModel,
    CountryCreate, CountryRecord, CountryChanges, CryptoNetworkModel,
    CryptoNetworkCreate, CryptoNetworkRecord, CryptoNetworkChanges, DepositModel,
    DepositCreate, DepositRecord, DepositChanges, IntroducerChangeModel,
    IntroducerChangeCreate, IntroducerChangeRecord, IntroducerChangeChanges, UserModel,
    UserCreate, UserCreditTransactionModel, UserCreditTransactionCreate,
    UserCreditTransactionRecord, UserCreditTransactionChanges, UserRecord, UserChanges,
    WithdrawalModel, WithdrawalCreate, WithdrawalRecord, WithdrawalChanges,
};
use serde::de::DeserializeOwned;

use crate::internal::observers::{audit, models};

macro_rules! dispatch_creating {
    ($event:expr, $payload:expr, $(($model:ty, $input:ty, $handler:path)),+ $(,)?) => {{
        match $event.model {
            $(
                <$model>::MODEL_KEY => {
                    let payload = decode::<$input>($payload, stringify!($input))?;
                    $handler($event, &payload).await
                }
            )+
            _ => Ok(()),
        }
    }};
}

macro_rules! dispatch_row_hook {
    ($event:expr, $payload:expr, $(($model:ty, $row:ty, $handler:path)),+ $(,)?) => {{
        match $event.model {
            $(
                <$model>::MODEL_KEY => {
                    let row = decode::<$row>($payload, stringify!($row))?;
                    $handler($event, &row).await
                }
            )+
            _ => Ok(()),
        }
    }};
}

macro_rules! dispatch_updating {
    ($event:expr, $old:expr, $changes:expr, $(($model:ty, $row:ty, $update:ty, $handler:path)),+ $(,)?) => {{
        match $event.model {
            $(
                <$model>::MODEL_KEY => {
                    let old_row = decode::<$row>($old, stringify!($row))?;
                    let changes = decode::<$update>($changes, stringify!($update))?;
                    $handler($event, &old_row, &changes).await
                }
            )+
            _ => Ok(()),
        }
    }};
}

macro_rules! dispatch_updated {
    ($event:expr, $old:expr, $new:expr, $(($model:ty, $row:ty, $handler:path)),+ $(,)?) => {{
        match $event.model {
            $(
                <$model>::MODEL_KEY => {
                    let old_row = decode::<$row>($old, stringify!($row))?;
                    let new_row = decode::<$row>($new, stringify!($row))?;
                    $handler($event, &old_row, &new_row).await
                }
            )+
            _ => Ok(()),
        }
    }};
}

pub struct AppModelObserver {
    db: sqlx::PgPool,
    admin_id: i64,
}

impl AppModelObserver {
    pub fn new(db: sqlx::PgPool, admin_id: i64) -> Self {
        Self { db, admin_id }
    }
}

fn decode<T: DeserializeOwned>(payload: &serde_json::Value, label: &str) -> anyhow::Result<T> {
    serde_json::from_value(payload.clone())
        .with_context(|| format!("failed to decode observer payload for {label}"))
}

#[async_trait::async_trait]
impl ModelObserver for AppModelObserver {
    async fn on_creating(
        &self,
        event: &ModelEvent,
        new_data: &serde_json::Value,
    ) -> anyhow::Result<()> {
        dispatch_creating!(
            event,
            new_data,
            (AdminModel, AdminCreate, models::admin::creating),
            (BankModel, BankCreate, models::bank::creating),
            (
                CompanyBankAccountModel,
                CompanyBankAccountCreate,
                models::company_bank_account::creating
            ),
            (
                CompanyCryptoAccountModel,
                CompanyCryptoAccountCreate,
                models::company_crypto_account::creating
            ),
            (
                ContentPageModel,
                ContentPageCreate,
                models::content_page::creating
            ),
            (CountryModel, CountryCreate, models::country::creating),
            (
                CryptoNetworkModel,
                CryptoNetworkCreate,
                models::crypto_network::creating
            ),
            (DepositModel, DepositCreate, models::deposit::creating),
            (
                IntroducerChangeModel,
                IntroducerChangeCreate,
                models::introducer_change::creating
            ),
            (UserModel, UserCreate, models::user::creating),
            (
                UserCreditTransactionModel,
                UserCreditTransactionCreate,
                models::user_credit_transaction::creating
            ),
            (
                WithdrawalModel,
                WithdrawalCreate,
                models::withdrawal::creating
            ),
        )
    }

    async fn on_created(
        &self,
        event: &ModelEvent,
        new_data: &serde_json::Value,
    ) -> anyhow::Result<()> {
        let model_result = dispatch_row_hook!(
            event,
            new_data,
            (AdminModel, AdminRecord, models::admin::created),
            (BankModel, BankRecord, models::bank::created),
            (
                CompanyBankAccountModel,
                CompanyBankAccountRecord,
                models::company_bank_account::created
            ),
            (
                CompanyCryptoAccountModel,
                CompanyCryptoAccountRecord,
                models::company_crypto_account::created
            ),
            (ContentPageModel, ContentPageRecord, models::content_page::created),
            (CountryModel, CountryRecord, models::country::created),
            (
                CryptoNetworkModel,
                CryptoNetworkRecord,
                models::crypto_network::created
            ),
            (DepositModel, DepositRecord, models::deposit::created),
            (
                IntroducerChangeModel,
                IntroducerChangeRecord,
                models::introducer_change::created
            ),
            (UserModel, UserRecord, models::user::created),
            (
                UserCreditTransactionModel,
                UserCreditTransactionRecord,
                models::user_credit_transaction::created
            ),
            (WithdrawalModel, WithdrawalRecord, models::withdrawal::created),
        );
        let audit_result = audit::created(&self.db, self.admin_id, event, new_data).await;
        model_result?;
        audit_result
    }

    async fn on_updating(
        &self,
        event: &ModelEvent,
        old_data: &serde_json::Value,
        changes: &serde_json::Value,
    ) -> anyhow::Result<()> {
        dispatch_updating!(
            event,
            old_data,
            changes,
            (AdminModel, AdminRecord, AdminChanges, models::admin::updating),
            (BankModel, BankRecord, BankChanges, models::bank::updating),
            (
                CompanyBankAccountModel,
                CompanyBankAccountRecord,
                CompanyBankAccountChanges,
                models::company_bank_account::updating
            ),
            (
                CompanyCryptoAccountModel,
                CompanyCryptoAccountRecord,
                CompanyCryptoAccountChanges,
                models::company_crypto_account::updating
            ),
            (
                ContentPageModel,
                ContentPageRecord,
                ContentPageChanges,
                models::content_page::updating
            ),
            (
                CountryModel,
                CountryRecord,
                CountryChanges,
                models::country::updating
            ),
            (
                CryptoNetworkModel,
                CryptoNetworkRecord,
                CryptoNetworkChanges,
                models::crypto_network::updating
            ),
            (
                DepositModel,
                DepositRecord,
                DepositChanges,
                models::deposit::updating
            ),
            (
                IntroducerChangeModel,
                IntroducerChangeRecord,
                IntroducerChangeChanges,
                models::introducer_change::updating
            ),
            (UserModel, UserRecord, UserChanges, models::user::updating),
            (
                UserCreditTransactionModel,
                UserCreditTransactionRecord,
                UserCreditTransactionChanges,
                models::user_credit_transaction::updating
            ),
            (
                WithdrawalModel,
                WithdrawalRecord,
                WithdrawalChanges,
                models::withdrawal::updating
            ),
        )
    }

    async fn on_updated(
        &self,
        event: &ModelEvent,
        old_data: &serde_json::Value,
        new_data: &serde_json::Value,
    ) -> anyhow::Result<()> {
        let model_result = dispatch_updated!(
            event,
            old_data,
            new_data,
            (AdminModel, AdminRecord, models::admin::updated),
            (BankModel, BankRecord, models::bank::updated),
            (
                CompanyBankAccountModel,
                CompanyBankAccountRecord,
                models::company_bank_account::updated
            ),
            (
                CompanyCryptoAccountModel,
                CompanyCryptoAccountRecord,
                models::company_crypto_account::updated
            ),
            (ContentPageModel, ContentPageRecord, models::content_page::updated),
            (CountryModel, CountryRecord, models::country::updated),
            (
                CryptoNetworkModel,
                CryptoNetworkRecord,
                models::crypto_network::updated
            ),
            (DepositModel, DepositRecord, models::deposit::updated),
            (
                IntroducerChangeModel,
                IntroducerChangeRecord,
                models::introducer_change::updated
            ),
            (UserModel, UserRecord, models::user::updated),
            (
                UserCreditTransactionModel,
                UserCreditTransactionRecord,
                models::user_credit_transaction::updated
            ),
            (WithdrawalModel, WithdrawalRecord, models::withdrawal::updated),
        );
        let audit_result = audit::updated(&self.db, self.admin_id, event, old_data, new_data).await;
        model_result?;
        audit_result
    }

    async fn on_deleting(
        &self,
        event: &ModelEvent,
        old_data: &serde_json::Value,
    ) -> anyhow::Result<()> {
        dispatch_row_hook!(
            event,
            old_data,
            (AdminModel, AdminRecord, models::admin::deleting),
            (BankModel, BankRecord, models::bank::deleting),
            (
                CompanyBankAccountModel,
                CompanyBankAccountRecord,
                models::company_bank_account::deleting
            ),
            (
                CompanyCryptoAccountModel,
                CompanyCryptoAccountRecord,
                models::company_crypto_account::deleting
            ),
            (ContentPageModel, ContentPageRecord, models::content_page::deleting),
            (CountryModel, CountryRecord, models::country::deleting),
            (
                CryptoNetworkModel,
                CryptoNetworkRecord,
                models::crypto_network::deleting
            ),
            (DepositModel, DepositRecord, models::deposit::deleting),
            (
                IntroducerChangeModel,
                IntroducerChangeRecord,
                models::introducer_change::deleting
            ),
            (UserModel, UserRecord, models::user::deleting),
            (
                UserCreditTransactionModel,
                UserCreditTransactionRecord,
                models::user_credit_transaction::deleting
            ),
            (WithdrawalModel, WithdrawalRecord, models::withdrawal::deleting),
        )
    }

    async fn on_deleted(
        &self,
        event: &ModelEvent,
        old_data: &serde_json::Value,
    ) -> anyhow::Result<()> {
        let model_result = dispatch_row_hook!(
            event,
            old_data,
            (AdminModel, AdminRecord, models::admin::deleted),
            (BankModel, BankRecord, models::bank::deleted),
            (
                CompanyBankAccountModel,
                CompanyBankAccountRecord,
                models::company_bank_account::deleted
            ),
            (
                CompanyCryptoAccountModel,
                CompanyCryptoAccountRecord,
                models::company_crypto_account::deleted
            ),
            (ContentPageModel, ContentPageRecord, models::content_page::deleted),
            (CountryModel, CountryRecord, models::country::deleted),
            (
                CryptoNetworkModel,
                CryptoNetworkRecord,
                models::crypto_network::deleted
            ),
            (DepositModel, DepositRecord, models::deposit::deleted),
            (
                IntroducerChangeModel,
                IntroducerChangeRecord,
                models::introducer_change::deleted
            ),
            (UserModel, UserRecord, models::user::deleted),
            (
                UserCreditTransactionModel,
                UserCreditTransactionRecord,
                models::user_credit_transaction::deleted
            ),
            (WithdrawalModel, WithdrawalRecord, models::withdrawal::deleted),
        );
        let audit_result = audit::deleted(&self.db, self.admin_id, event, old_data).await;
        model_result?;
        audit_result
    }
}
