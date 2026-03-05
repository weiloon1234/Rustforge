use crate::schema::{to_snake, EnumSpec, EnumVariants};

/// Generate Rust enum code from EnumSpec
pub fn generate_enum(name: &str, spec: &EnumSpec) -> String {
    match spec.storage.as_str() {
        "string" | "text" => generate_string_enum(name, spec),
        // PostgreSQL only has SMALLINT (i16), INTEGER (i32), and BIGINT (i64)
        // No TINYINT (i8) or unsigned types
        "i16" | "i32" | "i64" => generate_integer_enum(name, spec),
        _ => panic!(
            "Unsupported enum storage type: {}. Supported: string, i16, i32, i64",
            spec.storage
        ),
    }
}

/// Generate string-based enum (stored as TEXT in database)
fn generate_string_enum(name: &str, spec: &EnumSpec) -> String {
    let variants = extract_variant_names(&spec.variants);
    let variant_self_list = variants
        .iter()
        .map(|variant| format!("Self::{}", variant))
        .collect::<Vec<_>>()
        .join(", ");

    // Get value mappings
    let value_map: Vec<(String, String)> = match &spec.variants {
        EnumVariants::Simple(names) => {
            // Use lowercase variant names as values
            names
                .iter()
                .map(|n| (n.clone(), n.to_lowercase()))
                .collect()
        }
        EnumVariants::Explicit(vars) => vars
            .iter()
            .map(|v| {
                let val = v.value.as_str().expect(&format!(
                    "String enum '{}' variant '{}' must have string value",
                    name, v.name
                ));
                (v.name.clone(), val.to_string())
            })
            .collect(),
    };

    let variant_list = value_map
        .iter()
        .map(|(variant, value)| {
            format!(
                "    #[serde(rename = \"{}\")]\n    {variant}",
                escape_rust_string(value)
            )
        })
        .collect::<Vec<_>>()
        .join(",\n");

    let encode_arms: Vec<String> = value_map
        .iter()
        .map(|(variant, value)| format!("            Self::{} => \"{}\",", variant, value))
        .collect();

    let decode_arms: Vec<String> = value_map
        .iter()
        .map(|(variant, value)| format!("            \"{}\" => Ok(Self::{}),", value, variant))
        .collect();

    // Qualified arms for From<Enum> to avoid Self:: ambiguity
    let encode_arms_qualified: Vec<String> = value_map
        .iter()
        .map(|(variant, value)| format!("            {}::{} => \"{}\",", name, variant, value))
        .collect();

    let default_variant = &variants[0];
    let encode_arms_str = encode_arms.join("\n");
    let decode_arms_str = decode_arms.join("\n");
    let encode_arms_str_qualified = encode_arms_qualified.join("\n");
    let as_str_arms_str = encode_arms.join("\n");
    let as_label_arms_str = value_map
        .iter()
        .map(|(variant, _)| {
            format!(
                "            Self::{} => \"{}\",",
                variant,
                escape_rust_string(variant)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let from_storage_arms_str = value_map
        .iter()
        .map(|(variant, value)| format!("            \"{}\" => Some(Self::{}),", value, variant))
        .collect::<Vec<_>>()
        .join("\n");
    let enum_key = to_snake(name);
    let i18n_key_arms_str = value_map
        .iter()
        .map(|(variant, _)| {
            format!(
                "            Self::{} => \"enum.{}.{}\",",
                variant,
                enum_key,
                to_snake(variant)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let ts_union = value_map
        .iter()
        .map(|(_, value)| format!("\"{}\"", escape_rust_string(value)))
        .collect::<Vec<_>>()
        .join(" | ");
    let ts_union_literal = escape_rust_string(&ts_union);

    format!(
        r#"#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub enum {name} {{
{variant_list}
}}

impl Default for {name} {{
    fn default() -> Self {{
        Self::{default_variant}
    }}
}}

impl ts_rs::TS for {name} {{
    type WithoutGenerics = Self;

    fn name() -> String {{
        "{name}".to_string()
    }}

    fn inline() -> String {{
        Self::name()
    }}

    fn inline_flattened() -> String {{
        panic!("{name} cannot be flattened")
    }}

    fn decl() -> String {{
        "type {name} = {ts_union_literal};".to_string()
    }}

    fn decl_concrete() -> String {{
        Self::decl()
    }}
}}

impl {name} {{
    pub const fn as_str(self) -> &'static str {{
        match self {{
{as_str_arms_str}
        }}
    }}

    pub const fn as_label(self) -> &'static str {{
        match self {{
{as_label_arms_str}
        }}
    }}

    pub fn from_storage(raw: &str) -> Option<Self> {{
        match raw.trim() {{
{from_storage_arms_str}
            _ => None,
        }}
    }}

    pub const fn i18n_key(self) -> &'static str {{
        match self {{
{i18n_key_arms_str}
        }}
    }}

    pub fn explained_label(self) -> String {{
        let i18n_key = self.i18n_key();
        let translated_key = core_i18n::t(i18n_key);
        if translated_key != i18n_key {{
            return translated_key;
        }}
        let fallback_label = self.as_label();
        let translated_label = core_i18n::t(fallback_label);
        if translated_label != fallback_label {{
            return translated_label;
        }}
        fallback_label.to_string()
    }}

    pub const fn variants() -> &'static [Self] {{
        &[{variant_self_list}]
    }}

    pub fn datatable_filter_options() -> Vec<core_web::datatable::DataTableFilterOptionDto> {{
        Self::variants()
            .iter()
            .map(|v| {{
                let label = (*v).explained_label();
                let value = (*v).as_str();
                core_web::datatable::DataTableFilterOptionDto {{
                    label,
                    value: value.to_string(),
                }}
            }})
            .collect()
    }}
}}

// sqlx support for TEXT storage
impl sqlx::Encode<'_, sqlx::Postgres> for {name} {{
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {{
        let s = match self {{
{encode_arms_str}
        }};
        <&str as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&s, buf)
    }}
}}

impl sqlx::Decode<'_, sqlx::Postgres> for {name} {{
    fn decode(value: sqlx::postgres::PgValueRef<'_>) -> Result<Self, sqlx::error::BoxDynError> {{
        let s = <&str as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match s {{
{decode_arms_str}
            _ => Err(format!("Invalid {name}: {{}}", s).into()),
        }}
    }}
}}

impl sqlx::Type<sqlx::Postgres> for {name} {{
    fn type_info() -> sqlx::postgres::PgTypeInfo {{
        <String as sqlx::Type<sqlx::Postgres>>::type_info()
    }}
}}

// For ActiveRecord BindValue
impl From<{name}> for core_db::common::sql::BindValue {{
    fn from(v: {name}) -> Self {{
        let s = match v {{
{encode_arms_str_qualified}
        }};
        core_db::common::sql::BindValue::String(s.to_string())
    }}
}}
"#
    )
}

/// Generate integer-based enum (stored as SMALLINT/INTEGER in database)
fn generate_integer_enum(name: &str, spec: &EnumSpec) -> String {
    let rust_type = &spec.storage; // e.g., "i16", "i32"

    let (variant_decls, value_map) = match &spec.variants {
        EnumVariants::Explicit(vars) => {
            let decls: Vec<String> = vars
                .iter()
                .map(|v| {
                    let value = v.value.as_i64().expect(&format!(
                        "Integer enum '{}' variant '{}' must have integer value",
                        name, v.name
                    ));
                    format!(
                        "    #[serde(rename = \"{}\")]\n    {} = {},",
                        value, v.name, value
                    )
                })
                .collect();
            let map: Vec<(String, i64)> = vars
                .iter()
                .map(|v| (v.name.clone(), v.value.as_i64().unwrap()))
                .collect();
            (decls.join("\n"), map)
        }
        EnumVariants::Simple(_) => {
            panic!("Integer enums require explicit values, use syntax: variants = [{{name = \"Name\", value = 0}}]");
        }
    };

    let default_variant = &value_map[0].0;
    let decode_match = generate_int_decode_match(&value_map, name);
    let as_str_arms = value_map
        .iter()
        .map(|(variant, value)| format!("            Self::{} => \"{}\",", variant, value))
        .collect::<Vec<_>>()
        .join("\n");
    let as_label_arms = value_map
        .iter()
        .map(|(variant, _)| {
            format!(
                "            Self::{} => \"{}\",",
                variant,
                escape_rust_string(variant)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let from_storage_arms = value_map
        .iter()
        .map(|(variant, value)| format!("            {} => Some(Self::{}),", value, variant))
        .collect::<Vec<_>>()
        .join("\n");
    let enum_key = to_snake(name);
    let i18n_key_arms = value_map
        .iter()
        .map(|(variant, _)| {
            format!(
                "            Self::{} => \"enum.{}.{}\",",
                variant,
                enum_key,
                to_snake(variant)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let variant_self_list = value_map
        .iter()
        .map(|(variant, _)| format!("Self::{}", variant))
        .collect::<Vec<_>>()
        .join(", ");
    let ts_union = value_map
        .iter()
        .map(|(_, value)| format!("\"{}\"", value))
        .collect::<Vec<_>>()
        .join(" | ");
    let ts_union_literal = escape_rust_string(&ts_union);

    format!(
        r#"#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[repr({rust_type})]
pub enum {name} {{
{variant_decls}
}}

impl Default for {name} {{
    fn default() -> Self {{
        Self::{default_variant}
    }}
}}

impl ts_rs::TS for {name} {{
    type WithoutGenerics = Self;

    fn name() -> String {{
        "{name}".to_string()
    }}

    fn inline() -> String {{
        Self::name()
    }}

    fn inline_flattened() -> String {{
        panic!("{name} cannot be flattened")
    }}

    fn decl() -> String {{
        "type {name} = {ts_union_literal};".to_string()
    }}

    fn decl_concrete() -> String {{
        Self::decl()
    }}
}}

impl {name} {{
    pub const fn as_str(self) -> &'static str {{
        match self {{
{as_str_arms}
        }}
    }}

    pub const fn as_label(self) -> &'static str {{
        match self {{
{as_label_arms}
        }}
    }}

    pub fn from_storage(raw: &str) -> Option<Self> {{
        let value = raw.trim().parse::<i64>().ok()?;
        match value {{
{from_storage_arms}
            _ => None,
        }}
    }}

    pub const fn i18n_key(self) -> &'static str {{
        match self {{
{i18n_key_arms}
        }}
    }}

    pub fn explained_label(self) -> String {{
        let i18n_key = self.i18n_key();
        let translated_key = core_i18n::t(i18n_key);
        if translated_key != i18n_key {{
            return translated_key;
        }}
        let fallback_label = self.as_label();
        let translated_label = core_i18n::t(fallback_label);
        if translated_label != fallback_label {{
            return translated_label;
        }}
        fallback_label.to_string()
    }}

    pub const fn variants() -> &'static [Self] {{
        &[{variant_self_list}]
    }}

    pub fn datatable_filter_options() -> Vec<core_web::datatable::DataTableFilterOptionDto> {{
        Self::variants()
            .iter()
            .map(|v| {{
                let label = (*v).explained_label();
                let value = (*v).as_str();
                core_web::datatable::DataTableFilterOptionDto {{
                    label,
                    value: value.to_string(),
                }}
            }})
            .collect()
    }}
}}

// sqlx support for integer storage
impl sqlx::Encode<'_, sqlx::Postgres> for {name} {{
    fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {{
        <{rust_type} as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(&(*self as {rust_type}), buf)
    }}
}}

impl sqlx::Decode<'_, sqlx::Postgres> for {name} {{
    fn decode(value: sqlx::postgres::PgValueRef<'_>) -> Result<Self, sqlx::error::BoxDynError> {{
        let num = <{rust_type} as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        {decode_match}
    }}
}}

impl sqlx::Type<sqlx::Postgres> for {name} {{
    fn type_info() -> sqlx::postgres::PgTypeInfo {{
        <{rust_type} as sqlx::Type<sqlx::Postgres>>::type_info()
    }}
}}

// For ActiveRecord BindValue
impl From<{name}> for core_db::common::sql::BindValue {{
    fn from(v: {name}) -> Self {{
        core_db::common::sql::BindValue::I64(v as i64)
    }}
}}
"#
    )
}

fn generate_int_decode_match(value_map: &[(String, i64)], enum_name: &str) -> String {
    let arms: Vec<String> = value_map
        .iter()
        .map(|(variant, value)| format!("            {} => Ok(Self::{}),", value, variant))
        .collect();

    format!(
        r#"match num {{
{}
            _ => Err(format!("Invalid {}: {{}}", num).into()),
        }}"#,
        arms.join("\n"),
        enum_name
    )
}

fn extract_variant_names(variants: &EnumVariants) -> Vec<String> {
    match variants {
        EnumVariants::Simple(names) => names.clone(),
        EnumVariants::Explicit(vars) => vars.iter().map(|v| v.name.clone()).collect(),
    }
}

fn escape_rust_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('\"', "\\\"")
}

/// Generate enums.rs from schema enum definitions.
pub fn generate_enums(
    schema: &crate::schema::Schema,
    out_dir: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    use crate::schema::EnumOrOther;
    use std::io::Write;

    let mut enum_specs: Vec<(String, EnumSpec)> = vec![];

    // Extract all enum definitions from schema
    for (name, section) in &schema.extra_sections {
        if let EnumOrOther::Enum(spec) = section {
            if spec.type_name == "enum" {
                enum_specs.push((name.clone(), spec.clone()));
            }
        }
    }

    // Generate enums.rs file
    let enums_file = out_dir.join("enums.rs");
    let mut f = std::fs::File::create(&enums_file)?;

    writeln!(f, "// AUTO-GENERATED FILE — DO NOT EDIT")?;
    writeln!(f, "// Generated from TOML schema enum definitions\n")?;

    for (name, spec) in &enum_specs {
        let code = generate_enum(name, spec);
        writeln!(f, "{}\n", code)?;
    }

    println!("Generated {} enums to {:?}", enum_specs.len(), enums_file);

    Ok(())
}

/// Backward-compatible alias.
pub fn generate_all_enums(
    schema: &crate::schema::Schema,
    out_dir: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    generate_enums(schema, out_dir)
}
