// AUTO-GENERATED FILE — DO NOT EDIT
// Generated from Rust model source enum definitions

#[derive(Debug, Clone, Copy)]
pub struct SchemaEnumTsMeta {
    pub name: &'static str,
    pub variants: &'static [&'static str],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[repr(i16)]
pub enum AdjustableCreditType {
    #[serde(rename = "1")]
    Credit1 = 1,
}

impl Default for AdjustableCreditType {
    fn default() -> Self {
        Self::Credit1
    }
}

impl ts_rs::TS for AdjustableCreditType {
    type WithoutGenerics = Self;

    fn name() -> String {
        "AdjustableCreditType".to_string()
    }

    fn inline() -> String {
        Self::name()
    }

    fn inline_flattened() -> String {
        panic!("AdjustableCreditType cannot be flattened")
    }

    fn decl() -> String {
        "type AdjustableCreditType = \"1\";".to_string()
    }

    fn decl_concrete() -> String {
        Self::decl()
    }
}

impl AdjustableCreditType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Credit1 => "1",
        }
    }

    pub const fn as_label(self) -> &'static str {
        match self {
            Self::Credit1 => "Credit1",
        }
    }

    pub fn from_storage(raw: &str) -> Option<Self> {
        let value = raw.trim().parse::<i64>().ok()?;
        match value {
            1 => Some(Self::Credit1),
            _ => None,
        }
    }

    pub const fn i18n_key(self) -> &'static str {
        match self {
            Self::Credit1 => "enum.adjustable_credit_type.credit1",
        }
    }

    pub fn explained_label(self) -> String {
        let i18n_key = self.i18n_key();
        let translated_key = core_i18n::t(i18n_key);
        if translated_key != i18n_key {
            return translated_key;
        }
        let fallback_label = self.as_label();
        let translated_label = core_i18n::t(fallback_label);
        if translated_label != fallback_label {
            return translated_label;
        }
        fallback_label.to_string()
    }

    pub const fn variants() -> &'static [Self] {
        &[Self::Credit1]
    }

    pub fn datatable_filter_options() -> Vec<core_web::datatable::DataTableFilterOptionDto> {
        Self::variants()
            .iter()
            .map(|v| {
                let label = (*v).explained_label();
                let value = (*v).as_str();
                core_web::datatable::DataTableFilterOptionDto {
                    label,
                    value: value.to_string(),
                }
            })
            .collect()
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for AdjustableCreditType {
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        <i16 as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&(*self as i16), buf)
    }
}

impl sqlx::Decode<'_, sqlx::Postgres> for AdjustableCreditType {
    fn decode(value: sqlx::postgres::PgValueRef<'_>) -> Result<Self, sqlx::error::BoxDynError> {
        let num = <i16 as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match num {
            1 => Ok(Self::Credit1),
            _ => Err(format!("Invalid AdjustableCreditType: {}", num).into()),
        }
    }
}

impl sqlx::Type<sqlx::Postgres> for AdjustableCreditType {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <i16 as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

impl From<AdjustableCreditType> for core_db::common::sql::BindValue {
    fn from(v: AdjustableCreditType) -> Self {
        core_db::common::sql::BindValue::I64(v as i64)
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub enum AdminType {
    #[serde(rename = "developer")]
    Developer,
    #[serde(rename = "superadmin")]
    SuperAdmin,
    #[serde(rename = "admin")]
    Admin
}

impl Default for AdminType {
    fn default() -> Self {
        Self::Developer
    }
}

impl ts_rs::TS for AdminType {
    type WithoutGenerics = Self;

    fn name() -> String {
        "AdminType".to_string()
    }

    fn inline() -> String {
        Self::name()
    }

    fn inline_flattened() -> String {
        panic!("AdminType cannot be flattened")
    }

    fn decl() -> String {
        "type AdminType = \"developer\" | \"superadmin\" | \"admin\";".to_string()
    }

    fn decl_concrete() -> String {
        Self::decl()
    }
}

impl AdminType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Developer => "developer",
            Self::SuperAdmin => "superadmin",
            Self::Admin => "admin",
        }
    }

    pub const fn as_label(self) -> &'static str {
        match self {
            Self::Developer => "Developer",
            Self::SuperAdmin => "SuperAdmin",
            Self::Admin => "Admin",
        }
    }

    pub fn from_storage(raw: &str) -> Option<Self> {
        match raw.trim() {
            "developer" => Some(Self::Developer),
            "superadmin" => Some(Self::SuperAdmin),
            "admin" => Some(Self::Admin),
            _ => None,
        }
    }

    pub const fn i18n_key(self) -> &'static str {
        match self {
            Self::Developer => "enum.admin_type.developer",
            Self::SuperAdmin => "enum.admin_type.super_admin",
            Self::Admin => "enum.admin_type.admin",
        }
    }

    pub fn explained_label(self) -> String {
        let i18n_key = self.i18n_key();
        let translated_key = core_i18n::t(i18n_key);
        if translated_key != i18n_key {
            return translated_key;
        }
        let fallback_label = self.as_label();
        let translated_label = core_i18n::t(fallback_label);
        if translated_label != fallback_label {
            return translated_label;
        }
        fallback_label.to_string()
    }

    pub const fn variants() -> &'static [Self] {
        &[Self::Developer, Self::SuperAdmin, Self::Admin]
    }

    pub fn datatable_filter_options() -> Vec<core_web::datatable::DataTableFilterOptionDto> {
        Self::variants()
            .iter()
            .map(|v| {
                let label = (*v).explained_label();
                let value = (*v).as_str();
                core_web::datatable::DataTableFilterOptionDto {
                    label,
                    value: value.to_string(),
                }
            })
            .collect()
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for AdminType {
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        let s = match self {
            Self::Developer => "developer",
            Self::SuperAdmin => "superadmin",
            Self::Admin => "admin",
        };
        <&str as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&s, buf)
    }
}

impl sqlx::Decode<'_, sqlx::Postgres> for AdminType {
    fn decode(value: sqlx::postgres::PgValueRef<'_>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <&str as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match s {
            "developer" => Ok(Self::Developer),
            "superadmin" => Ok(Self::SuperAdmin),
            "admin" => Ok(Self::Admin),
            _ => Err(format!("Invalid AdminType: {}", s).into()),
        }
    }
}

impl sqlx::Type<sqlx::Postgres> for AdminType {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <String as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

impl From<AdminType> for core_db::common::sql::BindValue {
    fn from(v: AdminType) -> Self {
        let s = match v {
            AdminType::Developer => "developer",
            AdminType::SuperAdmin => "superadmin",
            AdminType::Admin => "admin",
        };
        core_db::common::sql::BindValue::String(s.to_string())
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[repr(i16)]
pub enum AuditAction {
    #[serde(rename = "1")]
    Create = 1,
    #[serde(rename = "2")]
    Update = 2,
    #[serde(rename = "3")]
    Delete = 3,
}

impl Default for AuditAction {
    fn default() -> Self {
        Self::Create
    }
}

impl ts_rs::TS for AuditAction {
    type WithoutGenerics = Self;

    fn name() -> String {
        "AuditAction".to_string()
    }

    fn inline() -> String {
        Self::name()
    }

    fn inline_flattened() -> String {
        panic!("AuditAction cannot be flattened")
    }

    fn decl() -> String {
        "type AuditAction = \"1\" | \"2\" | \"3\";".to_string()
    }

    fn decl_concrete() -> String {
        Self::decl()
    }
}

impl AuditAction {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Create => "1",
            Self::Update => "2",
            Self::Delete => "3",
        }
    }

    pub const fn as_label(self) -> &'static str {
        match self {
            Self::Create => "Create",
            Self::Update => "Update",
            Self::Delete => "Delete",
        }
    }

    pub fn from_storage(raw: &str) -> Option<Self> {
        let value = raw.trim().parse::<i64>().ok()?;
        match value {
            1 => Some(Self::Create),
            2 => Some(Self::Update),
            3 => Some(Self::Delete),
            _ => None,
        }
    }

    pub const fn i18n_key(self) -> &'static str {
        match self {
            Self::Create => "enum.audit_action.create",
            Self::Update => "enum.audit_action.update",
            Self::Delete => "enum.audit_action.delete",
        }
    }

    pub fn explained_label(self) -> String {
        let i18n_key = self.i18n_key();
        let translated_key = core_i18n::t(i18n_key);
        if translated_key != i18n_key {
            return translated_key;
        }
        let fallback_label = self.as_label();
        let translated_label = core_i18n::t(fallback_label);
        if translated_label != fallback_label {
            return translated_label;
        }
        fallback_label.to_string()
    }

    pub const fn variants() -> &'static [Self] {
        &[Self::Create, Self::Update, Self::Delete]
    }

    pub fn datatable_filter_options() -> Vec<core_web::datatable::DataTableFilterOptionDto> {
        Self::variants()
            .iter()
            .map(|v| {
                let label = (*v).explained_label();
                let value = (*v).as_str();
                core_web::datatable::DataTableFilterOptionDto {
                    label,
                    value: value.to_string(),
                }
            })
            .collect()
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for AuditAction {
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        <i16 as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&(*self as i16), buf)
    }
}

impl sqlx::Decode<'_, sqlx::Postgres> for AuditAction {
    fn decode(value: sqlx::postgres::PgValueRef<'_>) -> Result<Self, sqlx::error::BoxDynError> {
        let num = <i16 as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match num {
            1 => Ok(Self::Create),
            2 => Ok(Self::Update),
            3 => Ok(Self::Delete),
            _ => Err(format!("Invalid AuditAction: {}", num).into()),
        }
    }
}

impl sqlx::Type<sqlx::Postgres> for AuditAction {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <i16 as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

impl From<AuditAction> for core_db::common::sql::BindValue {
    fn from(v: AuditAction) -> Self {
        core_db::common::sql::BindValue::I64(v as i64)
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[repr(i16)]
pub enum BankStatus {
    #[serde(rename = "1")]
    Enabled = 1,
    #[serde(rename = "2")]
    Disabled = 2,
}

impl Default for BankStatus {
    fn default() -> Self {
        Self::Enabled
    }
}

impl ts_rs::TS for BankStatus {
    type WithoutGenerics = Self;

    fn name() -> String {
        "BankStatus".to_string()
    }

    fn inline() -> String {
        Self::name()
    }

    fn inline_flattened() -> String {
        panic!("BankStatus cannot be flattened")
    }

    fn decl() -> String {
        "type BankStatus = \"1\" | \"2\";".to_string()
    }

    fn decl_concrete() -> String {
        Self::decl()
    }
}

impl BankStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Enabled => "1",
            Self::Disabled => "2",
        }
    }

    pub const fn as_label(self) -> &'static str {
        match self {
            Self::Enabled => "Enabled",
            Self::Disabled => "Disabled",
        }
    }

    pub fn from_storage(raw: &str) -> Option<Self> {
        let value = raw.trim().parse::<i64>().ok()?;
        match value {
            1 => Some(Self::Enabled),
            2 => Some(Self::Disabled),
            _ => None,
        }
    }

    pub const fn i18n_key(self) -> &'static str {
        match self {
            Self::Enabled => "enum.bank_status.enabled",
            Self::Disabled => "enum.bank_status.disabled",
        }
    }

    pub fn explained_label(self) -> String {
        let i18n_key = self.i18n_key();
        let translated_key = core_i18n::t(i18n_key);
        if translated_key != i18n_key {
            return translated_key;
        }
        let fallback_label = self.as_label();
        let translated_label = core_i18n::t(fallback_label);
        if translated_label != fallback_label {
            return translated_label;
        }
        fallback_label.to_string()
    }

    pub const fn variants() -> &'static [Self] {
        &[Self::Enabled, Self::Disabled]
    }

    pub fn datatable_filter_options() -> Vec<core_web::datatable::DataTableFilterOptionDto> {
        Self::variants()
            .iter()
            .map(|v| {
                let label = (*v).explained_label();
                let value = (*v).as_str();
                core_web::datatable::DataTableFilterOptionDto {
                    label,
                    value: value.to_string(),
                }
            })
            .collect()
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for BankStatus {
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        <i16 as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&(*self as i16), buf)
    }
}

impl sqlx::Decode<'_, sqlx::Postgres> for BankStatus {
    fn decode(value: sqlx::postgres::PgValueRef<'_>) -> Result<Self, sqlx::error::BoxDynError> {
        let num = <i16 as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match num {
            1 => Ok(Self::Enabled),
            2 => Ok(Self::Disabled),
            _ => Err(format!("Invalid BankStatus: {}", num).into()),
        }
    }
}

impl sqlx::Type<sqlx::Postgres> for BankStatus {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <i16 as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

impl From<BankStatus> for core_db::common::sql::BindValue {
    fn from(v: BankStatus) -> Self {
        core_db::common::sql::BindValue::I64(v as i64)
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[repr(i16)]
pub enum CompanyBankAccountStatus {
    #[serde(rename = "1")]
    Enabled = 1,
    #[serde(rename = "2")]
    Disabled = 2,
}

impl Default for CompanyBankAccountStatus {
    fn default() -> Self {
        Self::Enabled
    }
}

impl ts_rs::TS for CompanyBankAccountStatus {
    type WithoutGenerics = Self;

    fn name() -> String {
        "CompanyBankAccountStatus".to_string()
    }

    fn inline() -> String {
        Self::name()
    }

    fn inline_flattened() -> String {
        panic!("CompanyBankAccountStatus cannot be flattened")
    }

    fn decl() -> String {
        "type CompanyBankAccountStatus = \"1\" | \"2\";".to_string()
    }

    fn decl_concrete() -> String {
        Self::decl()
    }
}

impl CompanyBankAccountStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Enabled => "1",
            Self::Disabled => "2",
        }
    }

    pub const fn as_label(self) -> &'static str {
        match self {
            Self::Enabled => "Enabled",
            Self::Disabled => "Disabled",
        }
    }

    pub fn from_storage(raw: &str) -> Option<Self> {
        let value = raw.trim().parse::<i64>().ok()?;
        match value {
            1 => Some(Self::Enabled),
            2 => Some(Self::Disabled),
            _ => None,
        }
    }

    pub const fn i18n_key(self) -> &'static str {
        match self {
            Self::Enabled => "enum.company_bank_account_status.enabled",
            Self::Disabled => "enum.company_bank_account_status.disabled",
        }
    }

    pub fn explained_label(self) -> String {
        let i18n_key = self.i18n_key();
        let translated_key = core_i18n::t(i18n_key);
        if translated_key != i18n_key {
            return translated_key;
        }
        let fallback_label = self.as_label();
        let translated_label = core_i18n::t(fallback_label);
        if translated_label != fallback_label {
            return translated_label;
        }
        fallback_label.to_string()
    }

    pub const fn variants() -> &'static [Self] {
        &[Self::Enabled, Self::Disabled]
    }

    pub fn datatable_filter_options() -> Vec<core_web::datatable::DataTableFilterOptionDto> {
        Self::variants()
            .iter()
            .map(|v| {
                let label = (*v).explained_label();
                let value = (*v).as_str();
                core_web::datatable::DataTableFilterOptionDto {
                    label,
                    value: value.to_string(),
                }
            })
            .collect()
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for CompanyBankAccountStatus {
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        <i16 as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&(*self as i16), buf)
    }
}

impl sqlx::Decode<'_, sqlx::Postgres> for CompanyBankAccountStatus {
    fn decode(value: sqlx::postgres::PgValueRef<'_>) -> Result<Self, sqlx::error::BoxDynError> {
        let num = <i16 as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match num {
            1 => Ok(Self::Enabled),
            2 => Ok(Self::Disabled),
            _ => Err(format!("Invalid CompanyBankAccountStatus: {}", num).into()),
        }
    }
}

impl sqlx::Type<sqlx::Postgres> for CompanyBankAccountStatus {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <i16 as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

impl From<CompanyBankAccountStatus> for core_db::common::sql::BindValue {
    fn from(v: CompanyBankAccountStatus) -> Self {
        core_db::common::sql::BindValue::I64(v as i64)
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[repr(i16)]
pub enum CompanyCryptoAccountStatus {
    #[serde(rename = "1")]
    Enabled = 1,
    #[serde(rename = "2")]
    Disabled = 2,
}

impl Default for CompanyCryptoAccountStatus {
    fn default() -> Self {
        Self::Enabled
    }
}

impl ts_rs::TS for CompanyCryptoAccountStatus {
    type WithoutGenerics = Self;

    fn name() -> String {
        "CompanyCryptoAccountStatus".to_string()
    }

    fn inline() -> String {
        Self::name()
    }

    fn inline_flattened() -> String {
        panic!("CompanyCryptoAccountStatus cannot be flattened")
    }

    fn decl() -> String {
        "type CompanyCryptoAccountStatus = \"1\" | \"2\";".to_string()
    }

    fn decl_concrete() -> String {
        Self::decl()
    }
}

impl CompanyCryptoAccountStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Enabled => "1",
            Self::Disabled => "2",
        }
    }

    pub const fn as_label(self) -> &'static str {
        match self {
            Self::Enabled => "Enabled",
            Self::Disabled => "Disabled",
        }
    }

    pub fn from_storage(raw: &str) -> Option<Self> {
        let value = raw.trim().parse::<i64>().ok()?;
        match value {
            1 => Some(Self::Enabled),
            2 => Some(Self::Disabled),
            _ => None,
        }
    }

    pub const fn i18n_key(self) -> &'static str {
        match self {
            Self::Enabled => "enum.company_crypto_account_status.enabled",
            Self::Disabled => "enum.company_crypto_account_status.disabled",
        }
    }

    pub fn explained_label(self) -> String {
        let i18n_key = self.i18n_key();
        let translated_key = core_i18n::t(i18n_key);
        if translated_key != i18n_key {
            return translated_key;
        }
        let fallback_label = self.as_label();
        let translated_label = core_i18n::t(fallback_label);
        if translated_label != fallback_label {
            return translated_label;
        }
        fallback_label.to_string()
    }

    pub const fn variants() -> &'static [Self] {
        &[Self::Enabled, Self::Disabled]
    }

    pub fn datatable_filter_options() -> Vec<core_web::datatable::DataTableFilterOptionDto> {
        Self::variants()
            .iter()
            .map(|v| {
                let label = (*v).explained_label();
                let value = (*v).as_str();
                core_web::datatable::DataTableFilterOptionDto {
                    label,
                    value: value.to_string(),
                }
            })
            .collect()
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for CompanyCryptoAccountStatus {
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        <i16 as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&(*self as i16), buf)
    }
}

impl sqlx::Decode<'_, sqlx::Postgres> for CompanyCryptoAccountStatus {
    fn decode(value: sqlx::postgres::PgValueRef<'_>) -> Result<Self, sqlx::error::BoxDynError> {
        let num = <i16 as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match num {
            1 => Ok(Self::Enabled),
            2 => Ok(Self::Disabled),
            _ => Err(format!("Invalid CompanyCryptoAccountStatus: {}", num).into()),
        }
    }
}

impl sqlx::Type<sqlx::Postgres> for CompanyCryptoAccountStatus {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <i16 as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

impl From<CompanyCryptoAccountStatus> for core_db::common::sql::BindValue {
    fn from(v: CompanyCryptoAccountStatus) -> Self {
        core_db::common::sql::BindValue::I64(v as i64)
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[repr(i16)]
pub enum ContentPageSystemFlag {
    #[serde(rename = "0")]
    No = 0,
    #[serde(rename = "1")]
    Yes = 1,
}

impl Default for ContentPageSystemFlag {
    fn default() -> Self {
        Self::No
    }
}

impl ts_rs::TS for ContentPageSystemFlag {
    type WithoutGenerics = Self;

    fn name() -> String {
        "ContentPageSystemFlag".to_string()
    }

    fn inline() -> String {
        Self::name()
    }

    fn inline_flattened() -> String {
        panic!("ContentPageSystemFlag cannot be flattened")
    }

    fn decl() -> String {
        "type ContentPageSystemFlag = \"0\" | \"1\";".to_string()
    }

    fn decl_concrete() -> String {
        Self::decl()
    }
}

impl ContentPageSystemFlag {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::No => "0",
            Self::Yes => "1",
        }
    }

    pub const fn as_label(self) -> &'static str {
        match self {
            Self::No => "No",
            Self::Yes => "Yes",
        }
    }

    pub fn from_storage(raw: &str) -> Option<Self> {
        let value = raw.trim().parse::<i64>().ok()?;
        match value {
            0 => Some(Self::No),
            1 => Some(Self::Yes),
            _ => None,
        }
    }

    pub const fn i18n_key(self) -> &'static str {
        match self {
            Self::No => "enum.content_page_system_flag.no",
            Self::Yes => "enum.content_page_system_flag.yes",
        }
    }

    pub fn explained_label(self) -> String {
        let i18n_key = self.i18n_key();
        let translated_key = core_i18n::t(i18n_key);
        if translated_key != i18n_key {
            return translated_key;
        }
        let fallback_label = self.as_label();
        let translated_label = core_i18n::t(fallback_label);
        if translated_label != fallback_label {
            return translated_label;
        }
        fallback_label.to_string()
    }

    pub const fn variants() -> &'static [Self] {
        &[Self::No, Self::Yes]
    }

    pub fn datatable_filter_options() -> Vec<core_web::datatable::DataTableFilterOptionDto> {
        Self::variants()
            .iter()
            .map(|v| {
                let label = (*v).explained_label();
                let value = (*v).as_str();
                core_web::datatable::DataTableFilterOptionDto {
                    label,
                    value: value.to_string(),
                }
            })
            .collect()
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for ContentPageSystemFlag {
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        <i16 as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&(*self as i16), buf)
    }
}

impl sqlx::Decode<'_, sqlx::Postgres> for ContentPageSystemFlag {
    fn decode(value: sqlx::postgres::PgValueRef<'_>) -> Result<Self, sqlx::error::BoxDynError> {
        let num = <i16 as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match num {
            0 => Ok(Self::No),
            1 => Ok(Self::Yes),
            _ => Err(format!("Invalid ContentPageSystemFlag: {}", num).into()),
        }
    }
}

impl sqlx::Type<sqlx::Postgres> for ContentPageSystemFlag {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <i16 as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

impl From<ContentPageSystemFlag> for core_db::common::sql::BindValue {
    fn from(v: ContentPageSystemFlag) -> Self {
        core_db::common::sql::BindValue::I64(v as i64)
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[repr(i16)]
pub enum CountryIsDefault {
    #[serde(rename = "0")]
    No = 0,
    #[serde(rename = "1")]
    Yes = 1,
}

impl Default for CountryIsDefault {
    fn default() -> Self {
        Self::No
    }
}

impl ts_rs::TS for CountryIsDefault {
    type WithoutGenerics = Self;

    fn name() -> String {
        "CountryIsDefault".to_string()
    }

    fn inline() -> String {
        Self::name()
    }

    fn inline_flattened() -> String {
        panic!("CountryIsDefault cannot be flattened")
    }

    fn decl() -> String {
        "type CountryIsDefault = \"0\" | \"1\";".to_string()
    }

    fn decl_concrete() -> String {
        Self::decl()
    }
}

impl CountryIsDefault {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::No => "0",
            Self::Yes => "1",
        }
    }

    pub const fn as_label(self) -> &'static str {
        match self {
            Self::No => "No",
            Self::Yes => "Yes",
        }
    }

    pub fn from_storage(raw: &str) -> Option<Self> {
        let value = raw.trim().parse::<i64>().ok()?;
        match value {
            0 => Some(Self::No),
            1 => Some(Self::Yes),
            _ => None,
        }
    }

    pub const fn i18n_key(self) -> &'static str {
        match self {
            Self::No => "enum.country_is_default.no",
            Self::Yes => "enum.country_is_default.yes",
        }
    }

    pub fn explained_label(self) -> String {
        let i18n_key = self.i18n_key();
        let translated_key = core_i18n::t(i18n_key);
        if translated_key != i18n_key {
            return translated_key;
        }
        let fallback_label = self.as_label();
        let translated_label = core_i18n::t(fallback_label);
        if translated_label != fallback_label {
            return translated_label;
        }
        fallback_label.to_string()
    }

    pub const fn variants() -> &'static [Self] {
        &[Self::No, Self::Yes]
    }

    pub fn datatable_filter_options() -> Vec<core_web::datatable::DataTableFilterOptionDto> {
        Self::variants()
            .iter()
            .map(|v| {
                let label = (*v).explained_label();
                let value = (*v).as_str();
                core_web::datatable::DataTableFilterOptionDto {
                    label,
                    value: value.to_string(),
                }
            })
            .collect()
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for CountryIsDefault {
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        <i16 as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&(*self as i16), buf)
    }
}

impl sqlx::Decode<'_, sqlx::Postgres> for CountryIsDefault {
    fn decode(value: sqlx::postgres::PgValueRef<'_>) -> Result<Self, sqlx::error::BoxDynError> {
        let num = <i16 as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match num {
            0 => Ok(Self::No),
            1 => Ok(Self::Yes),
            _ => Err(format!("Invalid CountryIsDefault: {}", num).into()),
        }
    }
}

impl sqlx::Type<sqlx::Postgres> for CountryIsDefault {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <i16 as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

impl From<CountryIsDefault> for core_db::common::sql::BindValue {
    fn from(v: CountryIsDefault) -> Self {
        core_db::common::sql::BindValue::I64(v as i64)
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub enum CountryStatus {
    #[serde(rename = "enabled")]
    Enabled,
    #[serde(rename = "disabled")]
    Disabled
}

impl Default for CountryStatus {
    fn default() -> Self {
        Self::Enabled
    }
}

impl ts_rs::TS for CountryStatus {
    type WithoutGenerics = Self;

    fn name() -> String {
        "CountryStatus".to_string()
    }

    fn inline() -> String {
        Self::name()
    }

    fn inline_flattened() -> String {
        panic!("CountryStatus cannot be flattened")
    }

    fn decl() -> String {
        "type CountryStatus = \"enabled\" | \"disabled\";".to_string()
    }

    fn decl_concrete() -> String {
        Self::decl()
    }
}

impl CountryStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Enabled => "enabled",
            Self::Disabled => "disabled",
        }
    }

    pub const fn as_label(self) -> &'static str {
        match self {
            Self::Enabled => "Enabled",
            Self::Disabled => "Disabled",
        }
    }

    pub fn from_storage(raw: &str) -> Option<Self> {
        match raw.trim() {
            "enabled" => Some(Self::Enabled),
            "disabled" => Some(Self::Disabled),
            _ => None,
        }
    }

    pub const fn i18n_key(self) -> &'static str {
        match self {
            Self::Enabled => "enum.country_status.enabled",
            Self::Disabled => "enum.country_status.disabled",
        }
    }

    pub fn explained_label(self) -> String {
        let i18n_key = self.i18n_key();
        let translated_key = core_i18n::t(i18n_key);
        if translated_key != i18n_key {
            return translated_key;
        }
        let fallback_label = self.as_label();
        let translated_label = core_i18n::t(fallback_label);
        if translated_label != fallback_label {
            return translated_label;
        }
        fallback_label.to_string()
    }

    pub const fn variants() -> &'static [Self] {
        &[Self::Enabled, Self::Disabled]
    }

    pub fn datatable_filter_options() -> Vec<core_web::datatable::DataTableFilterOptionDto> {
        Self::variants()
            .iter()
            .map(|v| {
                let label = (*v).explained_label();
                let value = (*v).as_str();
                core_web::datatable::DataTableFilterOptionDto {
                    label,
                    value: value.to_string(),
                }
            })
            .collect()
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for CountryStatus {
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        let s = match self {
            Self::Enabled => "enabled",
            Self::Disabled => "disabled",
        };
        <&str as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&s, buf)
    }
}

impl sqlx::Decode<'_, sqlx::Postgres> for CountryStatus {
    fn decode(value: sqlx::postgres::PgValueRef<'_>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <&str as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match s {
            "enabled" => Ok(Self::Enabled),
            "disabled" => Ok(Self::Disabled),
            _ => Err(format!("Invalid CountryStatus: {}", s).into()),
        }
    }
}

impl sqlx::Type<sqlx::Postgres> for CountryStatus {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <String as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

impl From<CountryStatus> for core_db::common::sql::BindValue {
    fn from(v: CountryStatus) -> Self {
        let s = match v {
            CountryStatus::Enabled => "enabled",
            CountryStatus::Disabled => "disabled",
        };
        core_db::common::sql::BindValue::String(s.to_string())
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[repr(i16)]
pub enum CreditTransactionType {
    #[serde(rename = "101")]
    AdminAdd = 101,
    #[serde(rename = "102")]
    AdminDeduct = 102,
    #[serde(rename = "201")]
    TransferIn = 201,
    #[serde(rename = "202")]
    TransferOut = 202,
    #[serde(rename = "301")]
    Withdraw = 301,
    #[serde(rename = "302")]
    WithdrawRefund = 302,
    #[serde(rename = "401")]
    TopUp = 401,
}

impl Default for CreditTransactionType {
    fn default() -> Self {
        Self::AdminAdd
    }
}

impl ts_rs::TS for CreditTransactionType {
    type WithoutGenerics = Self;

    fn name() -> String {
        "CreditTransactionType".to_string()
    }

    fn inline() -> String {
        Self::name()
    }

    fn inline_flattened() -> String {
        panic!("CreditTransactionType cannot be flattened")
    }

    fn decl() -> String {
        "type CreditTransactionType = \"101\" | \"102\" | \"201\" | \"202\" | \"301\" | \"302\" | \"401\";".to_string()
    }

    fn decl_concrete() -> String {
        Self::decl()
    }
}

impl CreditTransactionType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AdminAdd => "101",
            Self::AdminDeduct => "102",
            Self::TransferIn => "201",
            Self::TransferOut => "202",
            Self::Withdraw => "301",
            Self::WithdrawRefund => "302",
            Self::TopUp => "401",
        }
    }

    pub const fn as_label(self) -> &'static str {
        match self {
            Self::AdminAdd => "AdminAdd",
            Self::AdminDeduct => "AdminDeduct",
            Self::TransferIn => "TransferIn",
            Self::TransferOut => "TransferOut",
            Self::Withdraw => "Withdraw",
            Self::WithdrawRefund => "WithdrawRefund",
            Self::TopUp => "TopUp",
        }
    }

    pub fn from_storage(raw: &str) -> Option<Self> {
        let value = raw.trim().parse::<i64>().ok()?;
        match value {
            101 => Some(Self::AdminAdd),
            102 => Some(Self::AdminDeduct),
            201 => Some(Self::TransferIn),
            202 => Some(Self::TransferOut),
            301 => Some(Self::Withdraw),
            302 => Some(Self::WithdrawRefund),
            401 => Some(Self::TopUp),
            _ => None,
        }
    }

    pub const fn i18n_key(self) -> &'static str {
        match self {
            Self::AdminAdd => "enum.credit_transaction_type.admin_add",
            Self::AdminDeduct => "enum.credit_transaction_type.admin_deduct",
            Self::TransferIn => "enum.credit_transaction_type.transfer_in",
            Self::TransferOut => "enum.credit_transaction_type.transfer_out",
            Self::Withdraw => "enum.credit_transaction_type.withdraw",
            Self::WithdrawRefund => "enum.credit_transaction_type.withdraw_refund",
            Self::TopUp => "enum.credit_transaction_type.top_up",
        }
    }

    pub fn explained_label(self) -> String {
        let i18n_key = self.i18n_key();
        let translated_key = core_i18n::t(i18n_key);
        if translated_key != i18n_key {
            return translated_key;
        }
        let fallback_label = self.as_label();
        let translated_label = core_i18n::t(fallback_label);
        if translated_label != fallback_label {
            return translated_label;
        }
        fallback_label.to_string()
    }

    pub const fn variants() -> &'static [Self] {
        &[Self::AdminAdd, Self::AdminDeduct, Self::TransferIn, Self::TransferOut, Self::Withdraw, Self::WithdrawRefund, Self::TopUp]
    }

    pub fn datatable_filter_options() -> Vec<core_web::datatable::DataTableFilterOptionDto> {
        Self::variants()
            .iter()
            .map(|v| {
                let label = (*v).explained_label();
                let value = (*v).as_str();
                core_web::datatable::DataTableFilterOptionDto {
                    label,
                    value: value.to_string(),
                }
            })
            .collect()
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for CreditTransactionType {
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        <i16 as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&(*self as i16), buf)
    }
}

impl sqlx::Decode<'_, sqlx::Postgres> for CreditTransactionType {
    fn decode(value: sqlx::postgres::PgValueRef<'_>) -> Result<Self, sqlx::error::BoxDynError> {
        let num = <i16 as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match num {
            101 => Ok(Self::AdminAdd),
            102 => Ok(Self::AdminDeduct),
            201 => Ok(Self::TransferIn),
            202 => Ok(Self::TransferOut),
            301 => Ok(Self::Withdraw),
            302 => Ok(Self::WithdrawRefund),
            401 => Ok(Self::TopUp),
            _ => Err(format!("Invalid CreditTransactionType: {}", num).into()),
        }
    }
}

impl sqlx::Type<sqlx::Postgres> for CreditTransactionType {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <i16 as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

impl From<CreditTransactionType> for core_db::common::sql::BindValue {
    fn from(v: CreditTransactionType) -> Self {
        core_db::common::sql::BindValue::I64(v as i64)
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[repr(i16)]
pub enum CreditType {
    #[serde(rename = "1")]
    Credit1 = 1,
    #[serde(rename = "2")]
    Credit2 = 2,
}

impl Default for CreditType {
    fn default() -> Self {
        Self::Credit1
    }
}

impl ts_rs::TS for CreditType {
    type WithoutGenerics = Self;

    fn name() -> String {
        "CreditType".to_string()
    }

    fn inline() -> String {
        Self::name()
    }

    fn inline_flattened() -> String {
        panic!("CreditType cannot be flattened")
    }

    fn decl() -> String {
        "type CreditType = \"1\" | \"2\";".to_string()
    }

    fn decl_concrete() -> String {
        Self::decl()
    }
}

impl CreditType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Credit1 => "1",
            Self::Credit2 => "2",
        }
    }

    pub const fn as_label(self) -> &'static str {
        match self {
            Self::Credit1 => "Credit1",
            Self::Credit2 => "Credit2",
        }
    }

    pub fn from_storage(raw: &str) -> Option<Self> {
        let value = raw.trim().parse::<i64>().ok()?;
        match value {
            1 => Some(Self::Credit1),
            2 => Some(Self::Credit2),
            _ => None,
        }
    }

    pub const fn i18n_key(self) -> &'static str {
        match self {
            Self::Credit1 => "enum.credit_type.credit1",
            Self::Credit2 => "enum.credit_type.credit2",
        }
    }

    pub fn explained_label(self) -> String {
        let i18n_key = self.i18n_key();
        let translated_key = core_i18n::t(i18n_key);
        if translated_key != i18n_key {
            return translated_key;
        }
        let fallback_label = self.as_label();
        let translated_label = core_i18n::t(fallback_label);
        if translated_label != fallback_label {
            return translated_label;
        }
        fallback_label.to_string()
    }

    pub const fn variants() -> &'static [Self] {
        &[Self::Credit1, Self::Credit2]
    }

    pub fn datatable_filter_options() -> Vec<core_web::datatable::DataTableFilterOptionDto> {
        Self::variants()
            .iter()
            .map(|v| {
                let label = (*v).explained_label();
                let value = (*v).as_str();
                core_web::datatable::DataTableFilterOptionDto {
                    label,
                    value: value.to_string(),
                }
            })
            .collect()
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for CreditType {
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        <i16 as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&(*self as i16), buf)
    }
}

impl sqlx::Decode<'_, sqlx::Postgres> for CreditType {
    fn decode(value: sqlx::postgres::PgValueRef<'_>) -> Result<Self, sqlx::error::BoxDynError> {
        let num = <i16 as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match num {
            1 => Ok(Self::Credit1),
            2 => Ok(Self::Credit2),
            _ => Err(format!("Invalid CreditType: {}", num).into()),
        }
    }
}

impl sqlx::Type<sqlx::Postgres> for CreditType {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <i16 as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

impl From<CreditType> for core_db::common::sql::BindValue {
    fn from(v: CreditType) -> Self {
        core_db::common::sql::BindValue::I64(v as i64)
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[repr(i16)]
pub enum CryptoNetworkStatus {
    #[serde(rename = "1")]
    Enabled = 1,
    #[serde(rename = "2")]
    Disabled = 2,
}

impl Default for CryptoNetworkStatus {
    fn default() -> Self {
        Self::Enabled
    }
}

impl ts_rs::TS for CryptoNetworkStatus {
    type WithoutGenerics = Self;

    fn name() -> String {
        "CryptoNetworkStatus".to_string()
    }

    fn inline() -> String {
        Self::name()
    }

    fn inline_flattened() -> String {
        panic!("CryptoNetworkStatus cannot be flattened")
    }

    fn decl() -> String {
        "type CryptoNetworkStatus = \"1\" | \"2\";".to_string()
    }

    fn decl_concrete() -> String {
        Self::decl()
    }
}

impl CryptoNetworkStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Enabled => "1",
            Self::Disabled => "2",
        }
    }

    pub const fn as_label(self) -> &'static str {
        match self {
            Self::Enabled => "Enabled",
            Self::Disabled => "Disabled",
        }
    }

    pub fn from_storage(raw: &str) -> Option<Self> {
        let value = raw.trim().parse::<i64>().ok()?;
        match value {
            1 => Some(Self::Enabled),
            2 => Some(Self::Disabled),
            _ => None,
        }
    }

    pub const fn i18n_key(self) -> &'static str {
        match self {
            Self::Enabled => "enum.crypto_network_status.enabled",
            Self::Disabled => "enum.crypto_network_status.disabled",
        }
    }

    pub fn explained_label(self) -> String {
        let i18n_key = self.i18n_key();
        let translated_key = core_i18n::t(i18n_key);
        if translated_key != i18n_key {
            return translated_key;
        }
        let fallback_label = self.as_label();
        let translated_label = core_i18n::t(fallback_label);
        if translated_label != fallback_label {
            return translated_label;
        }
        fallback_label.to_string()
    }

    pub const fn variants() -> &'static [Self] {
        &[Self::Enabled, Self::Disabled]
    }

    pub fn datatable_filter_options() -> Vec<core_web::datatable::DataTableFilterOptionDto> {
        Self::variants()
            .iter()
            .map(|v| {
                let label = (*v).explained_label();
                let value = (*v).as_str();
                core_web::datatable::DataTableFilterOptionDto {
                    label,
                    value: value.to_string(),
                }
            })
            .collect()
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for CryptoNetworkStatus {
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        <i16 as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&(*self as i16), buf)
    }
}

impl sqlx::Decode<'_, sqlx::Postgres> for CryptoNetworkStatus {
    fn decode(value: sqlx::postgres::PgValueRef<'_>) -> Result<Self, sqlx::error::BoxDynError> {
        let num = <i16 as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match num {
            1 => Ok(Self::Enabled),
            2 => Ok(Self::Disabled),
            _ => Err(format!("Invalid CryptoNetworkStatus: {}", num).into()),
        }
    }
}

impl sqlx::Type<sqlx::Postgres> for CryptoNetworkStatus {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <i16 as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

impl From<CryptoNetworkStatus> for core_db::common::sql::BindValue {
    fn from(v: CryptoNetworkStatus) -> Self {
        core_db::common::sql::BindValue::I64(v as i64)
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[repr(i16)]
pub enum DepositMethod {
    #[serde(rename = "1")]
    Manual = 1,
}

impl Default for DepositMethod {
    fn default() -> Self {
        Self::Manual
    }
}

impl ts_rs::TS for DepositMethod {
    type WithoutGenerics = Self;

    fn name() -> String {
        "DepositMethod".to_string()
    }

    fn inline() -> String {
        Self::name()
    }

    fn inline_flattened() -> String {
        panic!("DepositMethod cannot be flattened")
    }

    fn decl() -> String {
        "type DepositMethod = \"1\";".to_string()
    }

    fn decl_concrete() -> String {
        Self::decl()
    }
}

impl DepositMethod {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Manual => "1",
        }
    }

    pub const fn as_label(self) -> &'static str {
        match self {
            Self::Manual => "Manual",
        }
    }

    pub fn from_storage(raw: &str) -> Option<Self> {
        let value = raw.trim().parse::<i64>().ok()?;
        match value {
            1 => Some(Self::Manual),
            _ => None,
        }
    }

    pub const fn i18n_key(self) -> &'static str {
        match self {
            Self::Manual => "enum.deposit_method.manual",
        }
    }

    pub fn explained_label(self) -> String {
        let i18n_key = self.i18n_key();
        let translated_key = core_i18n::t(i18n_key);
        if translated_key != i18n_key {
            return translated_key;
        }
        let fallback_label = self.as_label();
        let translated_label = core_i18n::t(fallback_label);
        if translated_label != fallback_label {
            return translated_label;
        }
        fallback_label.to_string()
    }

    pub const fn variants() -> &'static [Self] {
        &[Self::Manual]
    }

    pub fn datatable_filter_options() -> Vec<core_web::datatable::DataTableFilterOptionDto> {
        Self::variants()
            .iter()
            .map(|v| {
                let label = (*v).explained_label();
                let value = (*v).as_str();
                core_web::datatable::DataTableFilterOptionDto {
                    label,
                    value: value.to_string(),
                }
            })
            .collect()
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for DepositMethod {
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        <i16 as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&(*self as i16), buf)
    }
}

impl sqlx::Decode<'_, sqlx::Postgres> for DepositMethod {
    fn decode(value: sqlx::postgres::PgValueRef<'_>) -> Result<Self, sqlx::error::BoxDynError> {
        let num = <i16 as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match num {
            1 => Ok(Self::Manual),
            _ => Err(format!("Invalid DepositMethod: {}", num).into()),
        }
    }
}

impl sqlx::Type<sqlx::Postgres> for DepositMethod {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <i16 as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

impl From<DepositMethod> for core_db::common::sql::BindValue {
    fn from(v: DepositMethod) -> Self {
        core_db::common::sql::BindValue::I64(v as i64)
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[repr(i16)]
pub enum DepositReviewAction {
    #[serde(rename = "1")]
    Approve = 1,
    #[serde(rename = "2")]
    Reject = 2,
}

impl Default for DepositReviewAction {
    fn default() -> Self {
        Self::Approve
    }
}

impl ts_rs::TS for DepositReviewAction {
    type WithoutGenerics = Self;

    fn name() -> String {
        "DepositReviewAction".to_string()
    }

    fn inline() -> String {
        Self::name()
    }

    fn inline_flattened() -> String {
        panic!("DepositReviewAction cannot be flattened")
    }

    fn decl() -> String {
        "type DepositReviewAction = \"1\" | \"2\";".to_string()
    }

    fn decl_concrete() -> String {
        Self::decl()
    }
}

impl DepositReviewAction {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Approve => "1",
            Self::Reject => "2",
        }
    }

    pub const fn as_label(self) -> &'static str {
        match self {
            Self::Approve => "Approve",
            Self::Reject => "Reject",
        }
    }

    pub fn from_storage(raw: &str) -> Option<Self> {
        let value = raw.trim().parse::<i64>().ok()?;
        match value {
            1 => Some(Self::Approve),
            2 => Some(Self::Reject),
            _ => None,
        }
    }

    pub const fn i18n_key(self) -> &'static str {
        match self {
            Self::Approve => "enum.deposit_review_action.approve",
            Self::Reject => "enum.deposit_review_action.reject",
        }
    }

    pub fn explained_label(self) -> String {
        let i18n_key = self.i18n_key();
        let translated_key = core_i18n::t(i18n_key);
        if translated_key != i18n_key {
            return translated_key;
        }
        let fallback_label = self.as_label();
        let translated_label = core_i18n::t(fallback_label);
        if translated_label != fallback_label {
            return translated_label;
        }
        fallback_label.to_string()
    }

    pub const fn variants() -> &'static [Self] {
        &[Self::Approve, Self::Reject]
    }

    pub fn datatable_filter_options() -> Vec<core_web::datatable::DataTableFilterOptionDto> {
        Self::variants()
            .iter()
            .map(|v| {
                let label = (*v).explained_label();
                let value = (*v).as_str();
                core_web::datatable::DataTableFilterOptionDto {
                    label,
                    value: value.to_string(),
                }
            })
            .collect()
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for DepositReviewAction {
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        <i16 as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&(*self as i16), buf)
    }
}

impl sqlx::Decode<'_, sqlx::Postgres> for DepositReviewAction {
    fn decode(value: sqlx::postgres::PgValueRef<'_>) -> Result<Self, sqlx::error::BoxDynError> {
        let num = <i16 as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match num {
            1 => Ok(Self::Approve),
            2 => Ok(Self::Reject),
            _ => Err(format!("Invalid DepositReviewAction: {}", num).into()),
        }
    }
}

impl sqlx::Type<sqlx::Postgres> for DepositReviewAction {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <i16 as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

impl From<DepositReviewAction> for core_db::common::sql::BindValue {
    fn from(v: DepositReviewAction) -> Self {
        core_db::common::sql::BindValue::I64(v as i64)
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[repr(i16)]
pub enum DepositStatus {
    #[serde(rename = "1")]
    Pending = 1,
    #[serde(rename = "2")]
    Approved = 2,
    #[serde(rename = "3")]
    Rejected = 3,
}

impl Default for DepositStatus {
    fn default() -> Self {
        Self::Pending
    }
}

impl ts_rs::TS for DepositStatus {
    type WithoutGenerics = Self;

    fn name() -> String {
        "DepositStatus".to_string()
    }

    fn inline() -> String {
        Self::name()
    }

    fn inline_flattened() -> String {
        panic!("DepositStatus cannot be flattened")
    }

    fn decl() -> String {
        "type DepositStatus = \"1\" | \"2\" | \"3\";".to_string()
    }

    fn decl_concrete() -> String {
        Self::decl()
    }
}

impl DepositStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "1",
            Self::Approved => "2",
            Self::Rejected => "3",
        }
    }

    pub const fn as_label(self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::Approved => "Approved",
            Self::Rejected => "Rejected",
        }
    }

    pub fn from_storage(raw: &str) -> Option<Self> {
        let value = raw.trim().parse::<i64>().ok()?;
        match value {
            1 => Some(Self::Pending),
            2 => Some(Self::Approved),
            3 => Some(Self::Rejected),
            _ => None,
        }
    }

    pub const fn i18n_key(self) -> &'static str {
        match self {
            Self::Pending => "enum.deposit_status.pending",
            Self::Approved => "enum.deposit_status.approved",
            Self::Rejected => "enum.deposit_status.rejected",
        }
    }

    pub fn explained_label(self) -> String {
        let i18n_key = self.i18n_key();
        let translated_key = core_i18n::t(i18n_key);
        if translated_key != i18n_key {
            return translated_key;
        }
        let fallback_label = self.as_label();
        let translated_label = core_i18n::t(fallback_label);
        if translated_label != fallback_label {
            return translated_label;
        }
        fallback_label.to_string()
    }

    pub const fn variants() -> &'static [Self] {
        &[Self::Pending, Self::Approved, Self::Rejected]
    }

    pub fn datatable_filter_options() -> Vec<core_web::datatable::DataTableFilterOptionDto> {
        Self::variants()
            .iter()
            .map(|v| {
                let label = (*v).explained_label();
                let value = (*v).as_str();
                core_web::datatable::DataTableFilterOptionDto {
                    label,
                    value: value.to_string(),
                }
            })
            .collect()
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for DepositStatus {
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        <i16 as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&(*self as i16), buf)
    }
}

impl sqlx::Decode<'_, sqlx::Postgres> for DepositStatus {
    fn decode(value: sqlx::postgres::PgValueRef<'_>) -> Result<Self, sqlx::error::BoxDynError> {
        let num = <i16 as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match num {
            1 => Ok(Self::Pending),
            2 => Ok(Self::Approved),
            3 => Ok(Self::Rejected),
            _ => Err(format!("Invalid DepositStatus: {}", num).into()),
        }
    }
}

impl sqlx::Type<sqlx::Postgres> for DepositStatus {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <i16 as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

impl From<DepositStatus> for core_db::common::sql::BindValue {
    fn from(v: DepositStatus) -> Self {
        core_db::common::sql::BindValue::I64(v as i64)
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[repr(i16)]
pub enum OwnerType {
    #[serde(rename = "1")]
    User = 1,
}

impl Default for OwnerType {
    fn default() -> Self {
        Self::User
    }
}

impl ts_rs::TS for OwnerType {
    type WithoutGenerics = Self;

    fn name() -> String {
        "OwnerType".to_string()
    }

    fn inline() -> String {
        Self::name()
    }

    fn inline_flattened() -> String {
        panic!("OwnerType cannot be flattened")
    }

    fn decl() -> String {
        "type OwnerType = \"1\";".to_string()
    }

    fn decl_concrete() -> String {
        Self::decl()
    }
}

impl OwnerType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::User => "1",
        }
    }

    pub const fn as_label(self) -> &'static str {
        match self {
            Self::User => "User",
        }
    }

    pub fn from_storage(raw: &str) -> Option<Self> {
        let value = raw.trim().parse::<i64>().ok()?;
        match value {
            1 => Some(Self::User),
            _ => None,
        }
    }

    pub const fn i18n_key(self) -> &'static str {
        match self {
            Self::User => "enum.owner_type.user",
        }
    }

    pub fn explained_label(self) -> String {
        let i18n_key = self.i18n_key();
        let translated_key = core_i18n::t(i18n_key);
        if translated_key != i18n_key {
            return translated_key;
        }
        let fallback_label = self.as_label();
        let translated_label = core_i18n::t(fallback_label);
        if translated_label != fallback_label {
            return translated_label;
        }
        fallback_label.to_string()
    }

    pub const fn variants() -> &'static [Self] {
        &[Self::User]
    }

    pub fn datatable_filter_options() -> Vec<core_web::datatable::DataTableFilterOptionDto> {
        Self::variants()
            .iter()
            .map(|v| {
                let label = (*v).explained_label();
                let value = (*v).as_str();
                core_web::datatable::DataTableFilterOptionDto {
                    label,
                    value: value.to_string(),
                }
            })
            .collect()
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for OwnerType {
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        <i16 as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&(*self as i16), buf)
    }
}

impl sqlx::Decode<'_, sqlx::Postgres> for OwnerType {
    fn decode(value: sqlx::postgres::PgValueRef<'_>) -> Result<Self, sqlx::error::BoxDynError> {
        let num = <i16 as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match num {
            1 => Ok(Self::User),
            _ => Err(format!("Invalid OwnerType: {}", num).into()),
        }
    }
}

impl sqlx::Type<sqlx::Postgres> for OwnerType {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <i16 as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

impl From<OwnerType> for core_db::common::sql::BindValue {
    fn from(v: OwnerType) -> Self {
        core_db::common::sql::BindValue::I64(v as i64)
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub enum PersonalAccessTokenKind {
    #[serde(rename = "access")]
    Access,
    #[serde(rename = "refresh")]
    Refresh
}

impl Default for PersonalAccessTokenKind {
    fn default() -> Self {
        Self::Access
    }
}

impl ts_rs::TS for PersonalAccessTokenKind {
    type WithoutGenerics = Self;

    fn name() -> String {
        "PersonalAccessTokenKind".to_string()
    }

    fn inline() -> String {
        Self::name()
    }

    fn inline_flattened() -> String {
        panic!("PersonalAccessTokenKind cannot be flattened")
    }

    fn decl() -> String {
        "type PersonalAccessTokenKind = \"access\" | \"refresh\";".to_string()
    }

    fn decl_concrete() -> String {
        Self::decl()
    }
}

impl PersonalAccessTokenKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Access => "access",
            Self::Refresh => "refresh",
        }
    }

    pub const fn as_label(self) -> &'static str {
        match self {
            Self::Access => "Access",
            Self::Refresh => "Refresh",
        }
    }

    pub fn from_storage(raw: &str) -> Option<Self> {
        match raw.trim() {
            "access" => Some(Self::Access),
            "refresh" => Some(Self::Refresh),
            _ => None,
        }
    }

    pub const fn i18n_key(self) -> &'static str {
        match self {
            Self::Access => "enum.personal_access_token_kind.access",
            Self::Refresh => "enum.personal_access_token_kind.refresh",
        }
    }

    pub fn explained_label(self) -> String {
        let i18n_key = self.i18n_key();
        let translated_key = core_i18n::t(i18n_key);
        if translated_key != i18n_key {
            return translated_key;
        }
        let fallback_label = self.as_label();
        let translated_label = core_i18n::t(fallback_label);
        if translated_label != fallback_label {
            return translated_label;
        }
        fallback_label.to_string()
    }

    pub const fn variants() -> &'static [Self] {
        &[Self::Access, Self::Refresh]
    }

    pub fn datatable_filter_options() -> Vec<core_web::datatable::DataTableFilterOptionDto> {
        Self::variants()
            .iter()
            .map(|v| {
                let label = (*v).explained_label();
                let value = (*v).as_str();
                core_web::datatable::DataTableFilterOptionDto {
                    label,
                    value: value.to_string(),
                }
            })
            .collect()
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for PersonalAccessTokenKind {
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        let s = match self {
            Self::Access => "access",
            Self::Refresh => "refresh",
        };
        <&str as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&s, buf)
    }
}

impl sqlx::Decode<'_, sqlx::Postgres> for PersonalAccessTokenKind {
    fn decode(value: sqlx::postgres::PgValueRef<'_>) -> Result<Self, sqlx::error::BoxDynError> {
        let s = <&str as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match s {
            "access" => Ok(Self::Access),
            "refresh" => Ok(Self::Refresh),
            _ => Err(format!("Invalid PersonalAccessTokenKind: {}", s).into()),
        }
    }
}

impl sqlx::Type<sqlx::Postgres> for PersonalAccessTokenKind {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <String as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

impl From<PersonalAccessTokenKind> for core_db::common::sql::BindValue {
    fn from(v: PersonalAccessTokenKind) -> Self {
        let s = match v {
            PersonalAccessTokenKind::Access => "access",
            PersonalAccessTokenKind::Refresh => "refresh",
        };
        core_db::common::sql::BindValue::String(s.to_string())
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[repr(i16)]
pub enum UserBanStatus {
    #[serde(rename = "0")]
    No = 0,
    #[serde(rename = "1")]
    Yes = 1,
}

impl Default for UserBanStatus {
    fn default() -> Self {
        Self::No
    }
}

impl ts_rs::TS for UserBanStatus {
    type WithoutGenerics = Self;

    fn name() -> String {
        "UserBanStatus".to_string()
    }

    fn inline() -> String {
        Self::name()
    }

    fn inline_flattened() -> String {
        panic!("UserBanStatus cannot be flattened")
    }

    fn decl() -> String {
        "type UserBanStatus = \"0\" | \"1\";".to_string()
    }

    fn decl_concrete() -> String {
        Self::decl()
    }
}

impl UserBanStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::No => "0",
            Self::Yes => "1",
        }
    }

    pub const fn as_label(self) -> &'static str {
        match self {
            Self::No => "No",
            Self::Yes => "Yes",
        }
    }

    pub fn from_storage(raw: &str) -> Option<Self> {
        let value = raw.trim().parse::<i64>().ok()?;
        match value {
            0 => Some(Self::No),
            1 => Some(Self::Yes),
            _ => None,
        }
    }

    pub const fn i18n_key(self) -> &'static str {
        match self {
            Self::No => "enum.user_ban_status.no",
            Self::Yes => "enum.user_ban_status.yes",
        }
    }

    pub fn explained_label(self) -> String {
        let i18n_key = self.i18n_key();
        let translated_key = core_i18n::t(i18n_key);
        if translated_key != i18n_key {
            return translated_key;
        }
        let fallback_label = self.as_label();
        let translated_label = core_i18n::t(fallback_label);
        if translated_label != fallback_label {
            return translated_label;
        }
        fallback_label.to_string()
    }

    pub const fn variants() -> &'static [Self] {
        &[Self::No, Self::Yes]
    }

    pub fn datatable_filter_options() -> Vec<core_web::datatable::DataTableFilterOptionDto> {
        Self::variants()
            .iter()
            .map(|v| {
                let label = (*v).explained_label();
                let value = (*v).as_str();
                core_web::datatable::DataTableFilterOptionDto {
                    label,
                    value: value.to_string(),
                }
            })
            .collect()
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for UserBanStatus {
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        <i16 as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&(*self as i16), buf)
    }
}

impl sqlx::Decode<'_, sqlx::Postgres> for UserBanStatus {
    fn decode(value: sqlx::postgres::PgValueRef<'_>) -> Result<Self, sqlx::error::BoxDynError> {
        let num = <i16 as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match num {
            0 => Ok(Self::No),
            1 => Ok(Self::Yes),
            _ => Err(format!("Invalid UserBanStatus: {}", num).into()),
        }
    }
}

impl sqlx::Type<sqlx::Postgres> for UserBanStatus {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <i16 as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

impl From<UserBanStatus> for core_db::common::sql::BindValue {
    fn from(v: UserBanStatus) -> Self {
        core_db::common::sql::BindValue::I64(v as i64)
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[repr(i16)]
pub enum WithdrawalMethod {
    #[serde(rename = "1")]
    Manual = 1,
}

impl Default for WithdrawalMethod {
    fn default() -> Self {
        Self::Manual
    }
}

impl ts_rs::TS for WithdrawalMethod {
    type WithoutGenerics = Self;

    fn name() -> String {
        "WithdrawalMethod".to_string()
    }

    fn inline() -> String {
        Self::name()
    }

    fn inline_flattened() -> String {
        panic!("WithdrawalMethod cannot be flattened")
    }

    fn decl() -> String {
        "type WithdrawalMethod = \"1\";".to_string()
    }

    fn decl_concrete() -> String {
        Self::decl()
    }
}

impl WithdrawalMethod {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Manual => "1",
        }
    }

    pub const fn as_label(self) -> &'static str {
        match self {
            Self::Manual => "Manual",
        }
    }

    pub fn from_storage(raw: &str) -> Option<Self> {
        let value = raw.trim().parse::<i64>().ok()?;
        match value {
            1 => Some(Self::Manual),
            _ => None,
        }
    }

    pub const fn i18n_key(self) -> &'static str {
        match self {
            Self::Manual => "enum.withdrawal_method.manual",
        }
    }

    pub fn explained_label(self) -> String {
        let i18n_key = self.i18n_key();
        let translated_key = core_i18n::t(i18n_key);
        if translated_key != i18n_key {
            return translated_key;
        }
        let fallback_label = self.as_label();
        let translated_label = core_i18n::t(fallback_label);
        if translated_label != fallback_label {
            return translated_label;
        }
        fallback_label.to_string()
    }

    pub const fn variants() -> &'static [Self] {
        &[Self::Manual]
    }

    pub fn datatable_filter_options() -> Vec<core_web::datatable::DataTableFilterOptionDto> {
        Self::variants()
            .iter()
            .map(|v| {
                let label = (*v).explained_label();
                let value = (*v).as_str();
                core_web::datatable::DataTableFilterOptionDto {
                    label,
                    value: value.to_string(),
                }
            })
            .collect()
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for WithdrawalMethod {
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        <i16 as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&(*self as i16), buf)
    }
}

impl sqlx::Decode<'_, sqlx::Postgres> for WithdrawalMethod {
    fn decode(value: sqlx::postgres::PgValueRef<'_>) -> Result<Self, sqlx::error::BoxDynError> {
        let num = <i16 as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match num {
            1 => Ok(Self::Manual),
            _ => Err(format!("Invalid WithdrawalMethod: {}", num).into()),
        }
    }
}

impl sqlx::Type<sqlx::Postgres> for WithdrawalMethod {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <i16 as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

impl From<WithdrawalMethod> for core_db::common::sql::BindValue {
    fn from(v: WithdrawalMethod) -> Self {
        core_db::common::sql::BindValue::I64(v as i64)
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[repr(i16)]
pub enum WithdrawalReviewAction {
    #[serde(rename = "1")]
    Process = 1,
    #[serde(rename = "2")]
    Approve = 2,
    #[serde(rename = "3")]
    Reject = 3,
}

impl Default for WithdrawalReviewAction {
    fn default() -> Self {
        Self::Process
    }
}

impl ts_rs::TS for WithdrawalReviewAction {
    type WithoutGenerics = Self;

    fn name() -> String {
        "WithdrawalReviewAction".to_string()
    }

    fn inline() -> String {
        Self::name()
    }

    fn inline_flattened() -> String {
        panic!("WithdrawalReviewAction cannot be flattened")
    }

    fn decl() -> String {
        "type WithdrawalReviewAction = \"1\" | \"2\" | \"3\";".to_string()
    }

    fn decl_concrete() -> String {
        Self::decl()
    }
}

impl WithdrawalReviewAction {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Process => "1",
            Self::Approve => "2",
            Self::Reject => "3",
        }
    }

    pub const fn as_label(self) -> &'static str {
        match self {
            Self::Process => "Process",
            Self::Approve => "Approve",
            Self::Reject => "Reject",
        }
    }

    pub fn from_storage(raw: &str) -> Option<Self> {
        let value = raw.trim().parse::<i64>().ok()?;
        match value {
            1 => Some(Self::Process),
            2 => Some(Self::Approve),
            3 => Some(Self::Reject),
            _ => None,
        }
    }

    pub const fn i18n_key(self) -> &'static str {
        match self {
            Self::Process => "enum.withdrawal_review_action.process",
            Self::Approve => "enum.withdrawal_review_action.approve",
            Self::Reject => "enum.withdrawal_review_action.reject",
        }
    }

    pub fn explained_label(self) -> String {
        let i18n_key = self.i18n_key();
        let translated_key = core_i18n::t(i18n_key);
        if translated_key != i18n_key {
            return translated_key;
        }
        let fallback_label = self.as_label();
        let translated_label = core_i18n::t(fallback_label);
        if translated_label != fallback_label {
            return translated_label;
        }
        fallback_label.to_string()
    }

    pub const fn variants() -> &'static [Self] {
        &[Self::Process, Self::Approve, Self::Reject]
    }

    pub fn datatable_filter_options() -> Vec<core_web::datatable::DataTableFilterOptionDto> {
        Self::variants()
            .iter()
            .map(|v| {
                let label = (*v).explained_label();
                let value = (*v).as_str();
                core_web::datatable::DataTableFilterOptionDto {
                    label,
                    value: value.to_string(),
                }
            })
            .collect()
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for WithdrawalReviewAction {
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        <i16 as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&(*self as i16), buf)
    }
}

impl sqlx::Decode<'_, sqlx::Postgres> for WithdrawalReviewAction {
    fn decode(value: sqlx::postgres::PgValueRef<'_>) -> Result<Self, sqlx::error::BoxDynError> {
        let num = <i16 as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match num {
            1 => Ok(Self::Process),
            2 => Ok(Self::Approve),
            3 => Ok(Self::Reject),
            _ => Err(format!("Invalid WithdrawalReviewAction: {}", num).into()),
        }
    }
}

impl sqlx::Type<sqlx::Postgres> for WithdrawalReviewAction {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <i16 as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

impl From<WithdrawalReviewAction> for core_db::common::sql::BindValue {
    fn from(v: WithdrawalReviewAction) -> Self {
        core_db::common::sql::BindValue::I64(v as i64)
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[repr(i16)]
pub enum WithdrawalStatus {
    #[serde(rename = "1")]
    Pending = 1,
    #[serde(rename = "2")]
    Processing = 2,
    #[serde(rename = "3")]
    Approved = 3,
    #[serde(rename = "4")]
    Rejected = 4,
}

impl Default for WithdrawalStatus {
    fn default() -> Self {
        Self::Pending
    }
}

impl ts_rs::TS for WithdrawalStatus {
    type WithoutGenerics = Self;

    fn name() -> String {
        "WithdrawalStatus".to_string()
    }

    fn inline() -> String {
        Self::name()
    }

    fn inline_flattened() -> String {
        panic!("WithdrawalStatus cannot be flattened")
    }

    fn decl() -> String {
        "type WithdrawalStatus = \"1\" | \"2\" | \"3\" | \"4\";".to_string()
    }

    fn decl_concrete() -> String {
        Self::decl()
    }
}

impl WithdrawalStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "1",
            Self::Processing => "2",
            Self::Approved => "3",
            Self::Rejected => "4",
        }
    }

    pub const fn as_label(self) -> &'static str {
        match self {
            Self::Pending => "Pending",
            Self::Processing => "Processing",
            Self::Approved => "Approved",
            Self::Rejected => "Rejected",
        }
    }

    pub fn from_storage(raw: &str) -> Option<Self> {
        let value = raw.trim().parse::<i64>().ok()?;
        match value {
            1 => Some(Self::Pending),
            2 => Some(Self::Processing),
            3 => Some(Self::Approved),
            4 => Some(Self::Rejected),
            _ => None,
        }
    }

    pub const fn i18n_key(self) -> &'static str {
        match self {
            Self::Pending => "enum.withdrawal_status.pending",
            Self::Processing => "enum.withdrawal_status.processing",
            Self::Approved => "enum.withdrawal_status.approved",
            Self::Rejected => "enum.withdrawal_status.rejected",
        }
    }

    pub fn explained_label(self) -> String {
        let i18n_key = self.i18n_key();
        let translated_key = core_i18n::t(i18n_key);
        if translated_key != i18n_key {
            return translated_key;
        }
        let fallback_label = self.as_label();
        let translated_label = core_i18n::t(fallback_label);
        if translated_label != fallback_label {
            return translated_label;
        }
        fallback_label.to_string()
    }

    pub const fn variants() -> &'static [Self] {
        &[Self::Pending, Self::Processing, Self::Approved, Self::Rejected]
    }

    pub fn datatable_filter_options() -> Vec<core_web::datatable::DataTableFilterOptionDto> {
        Self::variants()
            .iter()
            .map(|v| {
                let label = (*v).explained_label();
                let value = (*v).as_str();
                core_web::datatable::DataTableFilterOptionDto {
                    label,
                    value: value.to_string(),
                }
            })
            .collect()
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for WithdrawalStatus {
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        <i16 as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&(*self as i16), buf)
    }
}

impl sqlx::Decode<'_, sqlx::Postgres> for WithdrawalStatus {
    fn decode(value: sqlx::postgres::PgValueRef<'_>) -> Result<Self, sqlx::error::BoxDynError> {
        let num = <i16 as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match num {
            1 => Ok(Self::Pending),
            2 => Ok(Self::Processing),
            3 => Ok(Self::Approved),
            4 => Ok(Self::Rejected),
            _ => Err(format!("Invalid WithdrawalStatus: {}", num).into()),
        }
    }
}

impl sqlx::Type<sqlx::Postgres> for WithdrawalStatus {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <i16 as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

impl From<WithdrawalStatus> for core_db::common::sql::BindValue {
    fn from(v: WithdrawalStatus) -> Self {
        core_db::common::sql::BindValue::I64(v as i64)
    }
}



pub const SCHEMA_ENUM_TS_META: &[SchemaEnumTsMeta] = &[
    SchemaEnumTsMeta { name: "AdjustableCreditType", variants: &["1"] },
    SchemaEnumTsMeta { name: "AdminType", variants: &["developer", "superadmin", "admin"] },
    SchemaEnumTsMeta { name: "AuditAction", variants: &["1", "2", "3"] },
    SchemaEnumTsMeta { name: "BankStatus", variants: &["1", "2"] },
    SchemaEnumTsMeta { name: "CompanyBankAccountStatus", variants: &["1", "2"] },
    SchemaEnumTsMeta { name: "CompanyCryptoAccountStatus", variants: &["1", "2"] },
    SchemaEnumTsMeta { name: "ContentPageSystemFlag", variants: &["0", "1"] },
    SchemaEnumTsMeta { name: "CountryIsDefault", variants: &["0", "1"] },
    SchemaEnumTsMeta { name: "CountryStatus", variants: &["enabled", "disabled"] },
    SchemaEnumTsMeta { name: "CreditTransactionType", variants: &["101", "102", "201", "202", "301", "302", "401"] },
    SchemaEnumTsMeta { name: "CreditType", variants: &["1", "2"] },
    SchemaEnumTsMeta { name: "CryptoNetworkStatus", variants: &["1", "2"] },
    SchemaEnumTsMeta { name: "DepositMethod", variants: &["1"] },
    SchemaEnumTsMeta { name: "DepositReviewAction", variants: &["1", "2"] },
    SchemaEnumTsMeta { name: "DepositStatus", variants: &["1", "2", "3"] },
    SchemaEnumTsMeta { name: "OwnerType", variants: &["1"] },
    SchemaEnumTsMeta { name: "PersonalAccessTokenKind", variants: &["access", "refresh"] },
    SchemaEnumTsMeta { name: "UserBanStatus", variants: &["0", "1"] },
    SchemaEnumTsMeta { name: "WithdrawalMethod", variants: &["1"] },
    SchemaEnumTsMeta { name: "WithdrawalReviewAction", variants: &["1", "2", "3"] },
    SchemaEnumTsMeta { name: "WithdrawalStatus", variants: &["1", "2", "3", "4"] },
];
