use anyhow::Context;
use core_db::common::model_observer::{ModelEvent, ModelObserver};
use generated::models::{
    Admin, AdminCreateInput, AdminRow, AdminUpdateChanges, Bank, BankCreateInput, BankRow,
    BankUpdateChanges, CompanyBankAccount, CompanyBankAccountCreateInput, CompanyBankAccountRow,
    CompanyBankAccountUpdateChanges, CompanyCryptoAccount, CompanyCryptoAccountCreateInput,
    CompanyCryptoAccountRow, CompanyCryptoAccountUpdateChanges, ContentPage,
    ContentPageCreateInput, ContentPageRow, ContentPageUpdateChanges, Country,
    CountryCreateInput, CountryRow, CountryUpdateChanges, CryptoNetwork,
    CryptoNetworkCreateInput, CryptoNetworkRow, CryptoNetworkUpdateChanges, Deposit,
    DepositCreateInput, DepositRow, DepositUpdateChanges, IntroducerChange,
    IntroducerChangeCreateInput, IntroducerChangeRow, IntroducerChangeUpdateChanges, User,
    UserCreateInput, UserCreditTransaction, UserCreditTransactionCreateInput,
    UserCreditTransactionRow, UserCreditTransactionUpdateChanges, UserRow, UserUpdateChanges,
    Withdrawal, WithdrawalCreateInput, WithdrawalRow, WithdrawalUpdateChanges,
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
            (Admin, AdminCreateInput, models::admin::creating),
            (Bank, BankCreateInput, models::bank::creating),
            (
                CompanyBankAccount,
                CompanyBankAccountCreateInput,
                models::company_bank_account::creating
            ),
            (
                CompanyCryptoAccount,
                CompanyCryptoAccountCreateInput,
                models::company_crypto_account::creating
            ),
            (
                ContentPage,
                ContentPageCreateInput,
                models::content_page::creating
            ),
            (Country, CountryCreateInput, models::country::creating),
            (
                CryptoNetwork,
                CryptoNetworkCreateInput,
                models::crypto_network::creating
            ),
            (Deposit, DepositCreateInput, models::deposit::creating),
            (
                IntroducerChange,
                IntroducerChangeCreateInput,
                models::introducer_change::creating
            ),
            (User, UserCreateInput, models::user::creating),
            (
                UserCreditTransaction,
                UserCreditTransactionCreateInput,
                models::user_credit_transaction::creating
            ),
            (
                Withdrawal,
                WithdrawalCreateInput,
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
            (Admin, AdminRow, models::admin::created),
            (Bank, BankRow, models::bank::created),
            (
                CompanyBankAccount,
                CompanyBankAccountRow,
                models::company_bank_account::created
            ),
            (
                CompanyCryptoAccount,
                CompanyCryptoAccountRow,
                models::company_crypto_account::created
            ),
            (ContentPage, ContentPageRow, models::content_page::created),
            (Country, CountryRow, models::country::created),
            (
                CryptoNetwork,
                CryptoNetworkRow,
                models::crypto_network::created
            ),
            (Deposit, DepositRow, models::deposit::created),
            (
                IntroducerChange,
                IntroducerChangeRow,
                models::introducer_change::created
            ),
            (User, UserRow, models::user::created),
            (
                UserCreditTransaction,
                UserCreditTransactionRow,
                models::user_credit_transaction::created
            ),
            (Withdrawal, WithdrawalRow, models::withdrawal::created),
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
            (Admin, AdminRow, AdminUpdateChanges, models::admin::updating),
            (Bank, BankRow, BankUpdateChanges, models::bank::updating),
            (
                CompanyBankAccount,
                CompanyBankAccountRow,
                CompanyBankAccountUpdateChanges,
                models::company_bank_account::updating
            ),
            (
                CompanyCryptoAccount,
                CompanyCryptoAccountRow,
                CompanyCryptoAccountUpdateChanges,
                models::company_crypto_account::updating
            ),
            (
                ContentPage,
                ContentPageRow,
                ContentPageUpdateChanges,
                models::content_page::updating
            ),
            (
                Country,
                CountryRow,
                CountryUpdateChanges,
                models::country::updating
            ),
            (
                CryptoNetwork,
                CryptoNetworkRow,
                CryptoNetworkUpdateChanges,
                models::crypto_network::updating
            ),
            (
                Deposit,
                DepositRow,
                DepositUpdateChanges,
                models::deposit::updating
            ),
            (
                IntroducerChange,
                IntroducerChangeRow,
                IntroducerChangeUpdateChanges,
                models::introducer_change::updating
            ),
            (User, UserRow, UserUpdateChanges, models::user::updating),
            (
                UserCreditTransaction,
                UserCreditTransactionRow,
                UserCreditTransactionUpdateChanges,
                models::user_credit_transaction::updating
            ),
            (
                Withdrawal,
                WithdrawalRow,
                WithdrawalUpdateChanges,
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
            (Admin, AdminRow, models::admin::updated),
            (Bank, BankRow, models::bank::updated),
            (
                CompanyBankAccount,
                CompanyBankAccountRow,
                models::company_bank_account::updated
            ),
            (
                CompanyCryptoAccount,
                CompanyCryptoAccountRow,
                models::company_crypto_account::updated
            ),
            (ContentPage, ContentPageRow, models::content_page::updated),
            (Country, CountryRow, models::country::updated),
            (
                CryptoNetwork,
                CryptoNetworkRow,
                models::crypto_network::updated
            ),
            (Deposit, DepositRow, models::deposit::updated),
            (
                IntroducerChange,
                IntroducerChangeRow,
                models::introducer_change::updated
            ),
            (User, UserRow, models::user::updated),
            (
                UserCreditTransaction,
                UserCreditTransactionRow,
                models::user_credit_transaction::updated
            ),
            (Withdrawal, WithdrawalRow, models::withdrawal::updated),
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
            (Admin, AdminRow, models::admin::deleting),
            (Bank, BankRow, models::bank::deleting),
            (
                CompanyBankAccount,
                CompanyBankAccountRow,
                models::company_bank_account::deleting
            ),
            (
                CompanyCryptoAccount,
                CompanyCryptoAccountRow,
                models::company_crypto_account::deleting
            ),
            (ContentPage, ContentPageRow, models::content_page::deleting),
            (Country, CountryRow, models::country::deleting),
            (
                CryptoNetwork,
                CryptoNetworkRow,
                models::crypto_network::deleting
            ),
            (Deposit, DepositRow, models::deposit::deleting),
            (
                IntroducerChange,
                IntroducerChangeRow,
                models::introducer_change::deleting
            ),
            (User, UserRow, models::user::deleting),
            (
                UserCreditTransaction,
                UserCreditTransactionRow,
                models::user_credit_transaction::deleting
            ),
            (Withdrawal, WithdrawalRow, models::withdrawal::deleting),
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
            (Admin, AdminRow, models::admin::deleted),
            (Bank, BankRow, models::bank::deleted),
            (
                CompanyBankAccount,
                CompanyBankAccountRow,
                models::company_bank_account::deleted
            ),
            (
                CompanyCryptoAccount,
                CompanyCryptoAccountRow,
                models::company_crypto_account::deleted
            ),
            (ContentPage, ContentPageRow, models::content_page::deleted),
            (Country, CountryRow, models::country::deleted),
            (
                CryptoNetwork,
                CryptoNetworkRow,
                models::crypto_network::deleted
            ),
            (Deposit, DepositRow, models::deposit::deleted),
            (
                IntroducerChange,
                IntroducerChangeRow,
                models::introducer_change::deleted
            ),
            (User, UserRow, models::user::deleted),
            (
                UserCreditTransaction,
                UserCreditTransactionRow,
                models::user_credit_transaction::deleted
            ),
            (Withdrawal, WithdrawalRow, models::withdrawal::deleted),
        );
        let audit_result = audit::deleted(&self.db, self.admin_id, event, old_data).await;
        model_result?;
        audit_result
    }
}
