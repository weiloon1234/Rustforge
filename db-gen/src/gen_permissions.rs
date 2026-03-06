use crate::permissions::PermissionEntry;
use crate::template::{render_template, TemplateContext};
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

pub fn generate_permissions(
    entries: &[PermissionEntry],
    out_file: &Path,
) -> Result<(), Box<dyn Error>> {
    if let Some(parent) = out_file.parent() {
        fs::create_dir_all(parent)?;
    }

    let variants = build_permission_variants(entries);
    let mut context = TemplateContext::new();
    context.insert(
        "permission_variants",
        render_permission_variants(&variants)?,
    )?;
    context.insert("ts_union_literal", render_ts_union_literal(&variants))?;
    context.insert(
        "permission_meta_entries",
        render_permission_meta_entries(&variants)?,
    )?;
    context.insert(
        "as_str_body",
        render_match_body(&variants, |variant| variant.entry.key.clone())?,
    )?;
    context.insert("from_str_arms", render_from_str_arms(&variants)?)?;
    context.insert("all_variants", render_all_variants(&variants)?)?;
    context.insert(
        "guard_body",
        render_match_body(&variants, |variant| variant.entry.guard.clone())?,
    )?;
    context.insert("meta_body", render_meta_body(&variants)?)?;

    let rendered = render_template("permissions/file.rs.tpl", &context)?;
    fs::write(out_file, rendered)?;
    Ok(())
}

#[derive(Debug)]
struct PermissionVariant<'a> {
    name: String,
    entry: &'a PermissionEntry,
}

fn build_permission_variants(entries: &[PermissionEntry]) -> Vec<PermissionVariant<'_>> {
    let mut variants = Vec::with_capacity(entries.len());
    let mut used = BTreeSet::new();
    for entry in entries {
        let base = key_to_variant(&entry.key);
        let mut name = base.clone();
        let mut suffix = 2usize;
        while used.contains(&name) {
            name = format!("{base}{suffix}");
            suffix += 1;
        }
        used.insert(name.clone());
        variants.push(PermissionVariant { name, entry });
    }
    variants
}

fn render_permission_variants(
    variants: &[PermissionVariant<'_>],
) -> Result<String, std::fmt::Error> {
    let mut out = String::new();
    for (idx, variant) in variants.iter().enumerate() {
        if idx > 0 {
            out.push('\n');
        }
        write!(out, "{}", render_permission_variant_decl(variant))?;
    }
    Ok(out)
}

fn render_permission_variant_decl(variant: &PermissionVariant<'_>) -> String {
    format!(
        "    #[serde(rename = \"{}\")]\n    {},",
        escape(&variant.entry.key),
        variant.name
    )
}

fn render_ts_union_literal(variants: &[PermissionVariant<'_>]) -> String {
    let ts_union = variants
        .iter()
        .map(|variant| format!("\"{}\"", escape(&variant.entry.key)))
        .collect::<Vec<_>>()
        .join(" | ");
    escape(&ts_union)
}

fn render_permission_meta_entries(
    variants: &[PermissionVariant<'_>],
) -> Result<String, std::fmt::Error> {
    let mut out = String::new();
    for variant in variants {
        writeln!(out, "{}", render_permission_meta_entry(variant))?;
    }
    Ok(out.trim_end().to_string())
}

fn render_permission_meta_entry(variant: &PermissionVariant<'_>) -> String {
    format!(
        "    PermissionMeta {{ key: \"{}\", guard: \"{}\", label: \"{}\", group: \"{}\", description: \"{}\" }},",
        escape(&variant.entry.key),
        escape(&variant.entry.guard),
        escape(&variant.entry.label),
        escape(&variant.entry.group),
        escape(&variant.entry.description)
    )
}

fn render_match_body<F>(
    variants: &[PermissionVariant<'_>],
    value_for: F,
) -> Result<String, std::fmt::Error>
where
    F: Fn(&PermissionVariant<'_>) -> String,
{
    if variants.is_empty() {
        return Ok("        match self {}".to_string());
    }

    let arms = variants
        .iter()
        .map(|variant| {
            format!(
                "            Self::{} => \"{}\",",
                variant.name,
                escape(&value_for(variant))
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let out = format!("        match self {{\n{arms}\n        }}");
    Ok(out)
}

fn render_from_str_arms(variants: &[PermissionVariant<'_>]) -> Result<String, std::fmt::Error> {
    let mut out = String::new();
    for variant in variants {
        writeln!(
            out,
            "            \"{}\" => Some(Self::{}),",
            escape(&variant.entry.key),
            variant.name
        )?;
    }
    Ok(out.trim_end().to_string())
}

fn render_all_variants(variants: &[PermissionVariant<'_>]) -> Result<String, std::fmt::Error> {
    let mut out = String::new();
    for variant in variants {
        writeln!(out, "            Self::{},", variant.name)?;
    }
    Ok(out.trim_end().to_string())
}

fn render_meta_body(variants: &[PermissionVariant<'_>]) -> Result<String, std::fmt::Error> {
    if variants.is_empty() {
        return Ok("        match self {}".to_string());
    }

    let arms = variants
        .iter()
        .enumerate()
        .map(|(idx, variant)| {
            format!(
                "            Self::{} => &PERMISSION_META[{}],",
                variant.name, idx
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let out = format!("        match self {{\n{arms}\n        }}");
    Ok(out)
}

fn key_to_variant(key: &str) -> String {
    if key.trim() == "*" {
        return "All".to_string();
    }

    let mut parts = Vec::new();
    let mut cur = String::new();
    for ch in key.chars() {
        if ch.is_ascii_alphanumeric() {
            cur.push(ch);
        } else if !cur.is_empty() {
            parts.push(cur.clone());
            cur.clear();
        }
    }
    if !cur.is_empty() {
        parts.push(cur);
    }

    let mut out = String::new();
    for part in parts {
        let mut chars = part.chars();
        if let Some(first) = chars.next() {
            out.push(first.to_ascii_uppercase());
            out.push_str(chars.as_str());
        }
    }

    if out.is_empty() {
        return "Permission".to_string();
    }

    if out
        .chars()
        .next()
        .map(|c| c.is_ascii_digit())
        .unwrap_or(false)
    {
        format!("P{out}")
    } else {
        out
    }
}

fn escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
