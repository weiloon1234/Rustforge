// AUTO-GENERATED FILE — DO NOT EDIT
// Generated from TOML schema enum definitions

#[derive(Debug, Clone, Copy)]
pub struct SchemaEnumTsMeta {
    pub name: &'static str,
    pub variants: &'static [&'static str],
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



pub const SCHEMA_ENUM_TS_META: &[SchemaEnumTsMeta] = &[
    SchemaEnumTsMeta { name: "AdminType", variants: &["developer", "superadmin", "admin"] },
    SchemaEnumTsMeta { name: "AuditAction", variants: &["1", "2", "3"] },
    SchemaEnumTsMeta { name: "ContentPageSystemFlag", variants: &["0", "1"] },
    SchemaEnumTsMeta { name: "CountryIsDefault", variants: &["0", "1"] },
    SchemaEnumTsMeta { name: "CountryStatus", variants: &["enabled", "disabled"] },
    SchemaEnumTsMeta { name: "CreditTransactionType", variants: &["101", "102", "201", "202", "301", "302", "401"] },
    SchemaEnumTsMeta { name: "CreditType", variants: &["1", "2"] },
    SchemaEnumTsMeta { name: "PersonalAccessTokenKind", variants: &["access", "refresh"] },
    SchemaEnumTsMeta { name: "UserBanStatus", variants: &["0", "1"] },
];
