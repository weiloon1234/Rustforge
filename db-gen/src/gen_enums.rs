use crate::schema::{to_snake, EnumSpec, EnumVariants};
use crate::template::{render_template, TemplateContext};
use std::error::Error;

#[derive(Debug, Clone, Copy)]
pub struct GenerateEnumsOptions {
    pub include_datatable_filter_options: bool,
}

impl Default for GenerateEnumsOptions {
    fn default() -> Self {
        Self {
            include_datatable_filter_options: true,
        }
    }
}

pub fn generate_enum(name: &str, spec: &EnumSpec) -> String {
    generate_enum_with_options(name, spec, GenerateEnumsOptions::default())
}

pub fn generate_enum_with_options(
    name: &str,
    spec: &EnumSpec,
    options: GenerateEnumsOptions,
) -> String {
    match spec.storage.as_str() {
        "string" | "text" => generate_string_enum(name, spec, options),
        "i16" | "i32" | "i64" => generate_integer_enum(name, spec, options),
        _ => panic!(
            "Unsupported enum storage type: {}. Supported: string, i16, i32, i64",
            spec.storage
        ),
    }
}

fn generate_string_enum(name: &str, spec: &EnumSpec, options: GenerateEnumsOptions) -> String {
    let value_map = string_value_map(name, spec);
    let variant_names = value_map
        .iter()
        .map(|(variant, _)| variant.clone())
        .collect::<Vec<_>>();
    let storage_values = value_map
        .iter()
        .map(|(_, value)| value.clone())
        .collect::<Vec<_>>();

    let mut context = TemplateContext::new();
    context.insert("name", name.to_string()).unwrap();
    context
        .insert("variant_list", render_string_variant_list(&value_map))
        .unwrap();
    context
        .insert(
            "as_str_arms",
            render_string_match_arms(&value_map, |(_, value)| value),
        )
        .unwrap();
    context
        .insert(
            "as_label_arms",
            render_string_match_arms(&value_map, |(variant, _)| variant),
        )
        .unwrap();
    context
        .insert(
            "from_storage_arms",
            render_string_from_storage_arms(&value_map),
        )
        .unwrap();
    context
        .insert(
            "encode_arms",
            render_string_match_arms(&value_map, |(_, value)| value),
        )
        .unwrap();
    context
        .insert("decode_arms", render_string_decode_arms(&value_map))
        .unwrap();
    context
        .insert(
            "encode_arms_qualified",
            render_string_encode_arms_qualified(name, &value_map),
        )
        .unwrap();
    insert_shared_enum_context(&mut context, name, &variant_names, &storage_values, options);

    render_template("enums/string_enum.rs.tpl", &context).unwrap()
}

fn generate_integer_enum(name: &str, spec: &EnumSpec, options: GenerateEnumsOptions) -> String {
    let rust_type = &spec.storage;
    let (variant_decls, value_map) = integer_enum_parts(name, spec);
    let variant_names = value_map
        .iter()
        .map(|(variant, _)| variant.clone())
        .collect::<Vec<_>>();
    let storage_values = value_map
        .iter()
        .map(|(_, value)| value.to_string())
        .collect::<Vec<_>>();

    let mut context = TemplateContext::new();
    context.insert("name", name.to_string()).unwrap();
    context.insert("rust_type", rust_type.to_string()).unwrap();
    context.insert("variant_decls", variant_decls).unwrap();
    context
        .insert("as_str_arms", render_integer_match_arms(&value_map, true))
        .unwrap();
    context
        .insert(
            "as_label_arms",
            render_integer_match_arms(&value_map, false),
        )
        .unwrap();
    context
        .insert(
            "from_storage_arms",
            value_map
                .iter()
                .map(|(variant, value)| format!("            {value} => Some(Self::{variant}),"))
                .collect::<Vec<_>>()
                .join("\n"),
        )
        .unwrap();
    context
        .insert(
            "decode_match",
            indent_block(&generate_int_decode_match(&value_map, name), 8),
        )
        .unwrap();
    insert_shared_enum_context(&mut context, name, &variant_names, &storage_values, options);

    render_template("enums/integer_enum.rs.tpl", &context).unwrap()
}

fn string_value_map(name: &str, spec: &EnumSpec) -> Vec<(String, String)> {
    match &spec.variants {
        EnumVariants::Simple(names) => names
            .iter()
            .map(|name| (name.clone(), name.to_lowercase()))
            .collect(),
        EnumVariants::Explicit(vars) => vars
            .iter()
            .map(|variant| {
                let value = variant.value.as_str().unwrap_or_else(|| {
                    panic!(
                        "String enum '{}' variant '{}' must have string value",
                        name, variant.name
                    )
                });
                (variant.name.clone(), value.to_string())
            })
            .collect(),
    }
}

fn integer_enum_parts(name: &str, spec: &EnumSpec) -> (String, Vec<(String, i64)>) {
    match &spec.variants {
        EnumVariants::Explicit(vars) => {
            let decls = vars
                .iter()
                .map(|variant| {
                    let value = variant.value.as_i64().unwrap_or_else(|| {
                        panic!(
                            "Integer enum '{}' variant '{}' must have integer value",
                            name, variant.name
                        )
                    });
                    format!(
                        "    #[serde(rename = \"{}\")]\n    {} = {},",
                        value, variant.name, value
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");
            let values = vars
                .iter()
                .map(|variant| {
                    let value = variant.value.as_i64().unwrap_or_else(|| {
                        panic!(
                            "Integer enum '{}' variant '{}' must have integer value",
                            name, variant.name
                        )
                    });
                    (variant.name.clone(), value)
                })
                .collect::<Vec<_>>();
            (decls, values)
        }
        EnumVariants::Simple(_) => panic!(
            "Integer enums require explicit values, use syntax: variants = [{{name = \"Name\", value = 0}}]"
        ),
    }
}

fn insert_shared_enum_context(
    context: &mut TemplateContext,
    name: &str,
    variant_names: &[String],
    storage_values: &[String],
    options: GenerateEnumsOptions,
) {
    context
        .insert("default_variant", variant_names[0].clone())
        .unwrap();
    context
        .insert("ts_union_literal", render_ts_union_literal(storage_values))
        .unwrap();
    context
        .insert("i18n_key_arms", render_i18n_key_arms(name, variant_names))
        .unwrap();
    context
        .insert("variant_self_list", render_variant_self_list(variant_names))
        .unwrap();
    context
        .insert(
            "datatable_filter_options_section",
            render_datatable_filter_options_section(options),
        )
        .unwrap();
}

fn render_variant_self_list(variant_names: &[String]) -> String {
    variant_names
        .iter()
        .map(|variant| format!("Self::{variant}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn render_string_variant_list(value_map: &[(String, String)]) -> String {
    value_map
        .iter()
        .map(|(variant, value)| {
            format!(
                "    #[serde(rename = \"{}\")]\n    {variant}",
                escape_rust_string(value)
            )
        })
        .collect::<Vec<_>>()
        .join(",\n")
}

fn render_string_match_arms<F>(value_map: &[(String, String)], value_for: F) -> String
where
    F: Fn(&(String, String)) -> &String,
{
    value_map
        .iter()
        .map(|pair| {
            let (variant, _) = pair;
            format!(
                "            Self::{variant} => \"{}\",",
                escape_rust_string(value_for(pair))
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_string_from_storage_arms(value_map: &[(String, String)]) -> String {
    value_map
        .iter()
        .map(|(variant, value)| format!("            \"{}\" => Some(Self::{variant}),", value))
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_string_decode_arms(value_map: &[(String, String)]) -> String {
    value_map
        .iter()
        .map(|(variant, value)| format!("            \"{}\" => Ok(Self::{variant}),", value))
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_string_encode_arms_qualified(name: &str, value_map: &[(String, String)]) -> String {
    value_map
        .iter()
        .map(|(variant, value)| format!("            {name}::{variant} => \"{}\",", value))
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_integer_match_arms(value_map: &[(String, i64)], storage_values: bool) -> String {
    value_map
        .iter()
        .map(|(variant, value)| {
            let rendered = if storage_values {
                value.to_string()
            } else {
                escape_rust_string(variant)
            };
            format!("            Self::{variant} => \"{rendered}\",")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_i18n_key_arms(name: &str, variants: &[String]) -> String {
    let enum_key = to_snake(name);
    variants
        .iter()
        .map(|variant| {
            format!(
                "            Self::{variant} => \"enum.{enum_key}.{}\",",
                to_snake(variant)
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_ts_union_literal(values: &[String]) -> String {
    let union = values
        .iter()
        .map(|value| format!("\"{}\"", escape_rust_string(value)))
        .collect::<Vec<_>>()
        .join(" | ");
    escape_rust_string(&union)
}

fn render_datatable_filter_options_section(options: GenerateEnumsOptions) -> String {
    if !options.include_datatable_filter_options {
        return String::new();
    }

    r#"
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
    }"#
    .to_string()
}

fn generate_int_decode_match(value_map: &[(String, i64)], enum_name: &str) -> String {
    let arms = value_map
        .iter()
        .map(|(variant, value)| format!("{value} => Ok(Self::{variant}),"))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "match num {{\n{}\n    _ => Err(format!(\"Invalid {}: {{}}\", num).into()),\n}}",
        indent_block(&arms, 4),
        enum_name
    )
}

fn indent_block(block: &str, spaces: usize) -> String {
    let indent = " ".repeat(spaces);
    block
        .lines()
        .map(|line| {
            if line.is_empty() {
                String::new()
            } else {
                format!("{indent}{line}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn escape_rust_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn enum_storage_values(spec: &EnumSpec) -> Vec<String> {
    match spec.storage.as_str() {
        "string" | "text" => match &spec.variants {
            EnumVariants::Simple(names) => names.iter().map(|name| name.to_lowercase()).collect(),
            EnumVariants::Explicit(vars) => vars
                .iter()
                .map(|variant| {
                    variant.value.as_str().unwrap_or_else(|| {
                        panic!(
                            "String enum variant '{}' must have string value",
                            variant.name
                        )
                    })
                    .to_string()
                })
                .collect(),
        },
        "i16" | "i32" | "i64" => match &spec.variants {
            EnumVariants::Explicit(vars) => vars
                .iter()
                .map(|variant| {
                    variant.value.as_i64().unwrap_or_else(|| {
                        panic!(
                            "Integer enum variant '{}' must have integer value",
                            variant.name
                        )
                    })
                    .to_string()
                })
                .collect(),
            EnumVariants::Simple(_) => panic!(
                "Integer enums require explicit values, use syntax: variants = [{{name = \"Name\", value = 0}}]"
            ),
        },
        other => panic!(
            "Unsupported enum storage type: {}. Supported: string, i16, i32, i64",
            other
        ),
    }
}

pub fn generate_enums(
    schema: &crate::schema::Schema,
    out_dir: &std::path::Path,
) -> Result<(), Box<dyn Error>> {
    generate_enums_with_options(schema, out_dir, GenerateEnumsOptions::default())
}

pub fn generate_enums_with_options(
    schema: &crate::schema::Schema,
    out_dir: &std::path::Path,
    options: GenerateEnumsOptions,
) -> Result<(), Box<dyn Error>> {
    use crate::schema::EnumOrOther;

    let mut enum_specs: Vec<(String, EnumSpec)> = vec![];
    for (name, section) in &schema.extra_sections {
        if let EnumOrOther::Enum(spec) = section {
            if spec.type_name == "enum" {
                enum_specs.push((name.clone(), spec.clone()));
            }
        }
    }

    let enum_blocks = enum_specs
        .iter()
        .map(|(name, spec)| generate_enum_with_options(name, spec, options))
        .collect::<Vec<_>>()
        .join("\n\n");
    let schema_enum_ts_meta_entries = enum_specs
        .iter()
        .map(|(name, spec)| {
            let variants = enum_storage_values(spec);
            let variants_list = variants
                .iter()
                .map(|value| format!("\"{}\"", escape_rust_string(value)))
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "    SchemaEnumTsMeta {{ name: \"{}\", variants: &[{}] }},",
                escape_rust_string(name),
                variants_list
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let mut context = TemplateContext::new();
    context.insert(
        "enum_blocks",
        if enum_blocks.is_empty() {
            String::new()
        } else {
            format!("{enum_blocks}\n\n")
        },
    )?;
    context.insert("schema_enum_ts_meta_entries", schema_enum_ts_meta_entries)?;
    let rendered = render_template("enums/file.rs.tpl", &context)?;

    let enums_file = out_dir.join("enums.rs");
    std::fs::write(&enums_file, rendered)?;
    println!("Generated {} enums to {:?}", enum_specs.len(), enums_file);
    Ok(())
}

pub fn generate_all_enums(
    schema: &crate::schema::Schema,
    out_dir: &std::path::Path,
) -> Result<(), Box<dyn Error>> {
    generate_enums(schema, out_dir)
}
