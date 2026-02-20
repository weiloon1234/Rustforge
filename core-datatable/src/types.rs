use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortDirection {
    Asc,
    Desc,
}

impl SortDirection {
    pub fn from_input(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "asc" | "ascending" => Some(Self::Asc),
            "desc" | "descending" => Some(Self::Desc),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DataTableExportMode {
    None,
    Csv,
}

impl DataTableExportMode {
    pub fn from_input(raw: &str) -> Self {
        match raw.trim().to_ascii_lowercase().as_str() {
            "1" | "true" | "csv" => Self::Csv,
            _ => Self::None,
        }
    }
}

impl Default for DataTableExportMode {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DataTablePaginationMode {
    Offset,
    Cursor,
}

impl DataTablePaginationMode {
    pub fn from_input(raw: &str) -> Self {
        match raw.trim().to_ascii_lowercase().as_str() {
            "cursor" | "keyset" => Self::Cursor,
            _ => Self::Offset,
        }
    }
}

impl Default for DataTablePaginationMode {
    fn default() -> Self {
        Self::Offset
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DataTableUnknownFilterMode {
    Ignore,
    Warn,
    Error,
}

impl DataTableUnknownFilterMode {
    pub fn from_input(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "ignore" => Some(Self::Ignore),
            "warn" | "warning" => Some(Self::Warn),
            "error" | "strict" | "fail" | "1" | "true" => Some(Self::Error),
            "0" | "false" => Some(Self::Ignore),
            _ => None,
        }
    }
}

impl Default for DataTableUnknownFilterMode {
    fn default() -> Self {
        Self::Ignore
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataTableInput {
    pub model: Option<String>,
    pub page: i64,
    pub ipp: i64,
    pub cursor: Option<String>,
    pub pagination_mode: DataTablePaginationMode,
    pub sorting_column: Option<String>,
    pub sorting: Option<SortDirection>,
    pub export: DataTableExportMode,
    pub export_file_name: Option<String>,
    pub timezone: Option<String>,
    pub params: BTreeMap<String, String>,
}

impl Default for DataTableInput {
    fn default() -> Self {
        Self {
            model: None,
            page: 1,
            ipp: 0,
            cursor: None,
            pagination_mode: DataTablePaginationMode::Offset,
            sorting_column: None,
            sorting: None,
            export: DataTableExportMode::None,
            export_file_name: None,
            timezone: None,
            params: BTreeMap::new(),
        }
    }
}

impl DataTableInput {
    pub fn from_pairs<I, K, V>(pairs: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        let mut input = Self::default();

        for (k, v) in pairs {
            let key = k.into();
            let val = v.into();
            input.params.insert(key.clone(), val.clone());
            match key.as_str() {
                "model" => {
                    if !val.trim().is_empty() {
                        input.model = Some(val.trim().to_string());
                    }
                }
                "p" | "page" => {
                    if let Ok(n) = val.trim().parse::<i64>() {
                        input.page = n;
                    }
                }
                "ipp" | "per_page" => {
                    if let Ok(n) = val.trim().parse::<i64>() {
                        input.ipp = n;
                    }
                }
                "cursor" => {
                    if !val.trim().is_empty() {
                        input.cursor = Some(val.trim().to_string());
                    }
                }
                "pagination_mode" | "paginate_mode" => {
                    input.pagination_mode = DataTablePaginationMode::from_input(&val);
                }
                "sorting" => {
                    input.sorting = SortDirection::from_input(&val);
                }
                "sorting_column" => {
                    if !val.trim().is_empty() {
                        input.sorting_column = Some(val.trim().to_string());
                    }
                }
                "export" => {
                    input.export = DataTableExportMode::from_input(&val);
                }
                "export_file_name" => {
                    if !val.trim().is_empty() {
                        input.export_file_name = Some(val.trim().to_string());
                    }
                }
                "timezone" | "TIMEZONE" | "X-Timezone" | "x-timezone" => {
                    if !val.trim().is_empty() {
                        input.timezone = Some(val.trim().to_string());
                    }
                }
                _ => {}
            }
        }

        input
    }

    pub fn filter_entries(&self) -> impl Iterator<Item = (&str, &str)> {
        self.params.iter().filter_map(|(k, v)| {
            if !k.starts_with("f-") || v.trim().is_empty() {
                return None;
            }
            Some((k.as_str(), v.as_str()))
        })
    }

    pub fn unknown_filter_mode(&self) -> Option<DataTableUnknownFilterMode> {
        self.params
            .get("unknown_filter_mode")
            .or(self.params.get("filter_unknown_mode"))
            .or(self.params.get("strict_filters"))
            .and_then(|v| DataTableUnknownFilterMode::from_input(v))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DataTableActor {
    pub id: Option<String>,
    pub guard: Option<String>,
    #[serde(default)]
    pub roles: Vec<String>,
    #[serde(default)]
    pub permissions: Vec<String>,
    #[serde(default)]
    pub attributes: BTreeMap<String, Value>,
}

impl DataTableActor {
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r.eq_ignore_ascii_case(role))
    }

    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions
            .iter()
            .any(|p| p.eq_ignore_ascii_case(permission))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataTableContext {
    pub default_per_page: i64,
    pub app_timezone: String,
    pub user_timezone: Option<String>,
    pub actor: Option<DataTableActor>,
    pub unknown_filter_mode: DataTableUnknownFilterMode,
}

impl Default for DataTableContext {
    fn default() -> Self {
        Self {
            default_per_page: 30,
            app_timezone: "+00:00".to_string(),
            user_timezone: None,
            actor: None,
            unknown_filter_mode: DataTableUnknownFilterMode::Ignore,
        }
    }
}

impl DataTableContext {
    pub fn effective_timezone(&self, input_timezone: Option<&str>) -> String {
        if let Some(tz) = input_timezone {
            let trimmed = tz.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
        if let Some(tz) = &self.user_timezone {
            let trimmed = tz.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
        self.app_timezone.clone()
    }

    pub fn has_role(&self, role: &str) -> bool {
        self.actor
            .as_ref()
            .map(|a| a.has_role(role))
            .unwrap_or(false)
    }

    pub fn has_permission(&self, permission: &str) -> bool {
        self.actor
            .as_ref()
            .map(|a| a.has_permission(permission))
            .unwrap_or(false)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataTableDiagnostics {
    pub duration_ms: u64,
    pub auto_filters_applied: usize,
    pub unknown_filters: Vec<String>,
    pub unknown_filter_mode: DataTableUnknownFilterMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataTableResult {
    pub records: Vec<Value>,
    pub per_page: i64,
    pub total_records: i64,
    pub total_pages: i64,
    pub page: i64,
    pub pagination_mode: DataTablePaginationMode,
    pub has_more: Option<bool>,
    pub next_cursor: Option<String>,
    pub diagnostics: DataTableDiagnostics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataTableColumnMeta {
    pub name: String,
    pub data_type: String,
    pub sortable: bool,
    pub localized: bool,
    pub filter_ops: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataTableRelationColumnMeta {
    pub relation: String,
    pub column: String,
    pub data_type: String,
    pub filter_ops: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataTableDefaults {
    pub sorting_column: String,
    pub sorted: SortDirection,
    pub per_page: i64,
    pub export_ignore_columns: Vec<String>,
    pub timestamp_columns: Vec<String>,
    pub unsortable: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataTableDescribe {
    pub model: String,
    pub defaults: DataTableDefaults,
    pub columns: Vec<DataTableColumnMeta>,
    pub relation_columns: Vec<DataTableRelationColumnMeta>,
    pub filter_patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataTableCsvExport {
    pub file_path: String,
    pub file_name: String,
    pub content_type: String,
    pub total_records: i64,
    pub diagnostics: DataTableDiagnostics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum DataTableExecution {
    Page(DataTableResult),
    Csv(DataTableCsvExport),
}
