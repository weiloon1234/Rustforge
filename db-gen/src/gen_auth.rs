use crate::config::ConfigsFile;
use crate::schema::{parse_fields, to_snake, to_title_case, Schema};
use crate::template::{render_template, TemplateContext};
use std::error::Error;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
struct AuthGuardSpec {
    name: String,
    file_stem: String,
    struct_name: String,
    provider_name: String,
    provider_snake: String,
    provider_title: String,
    provider_pk_ty: String,
}

pub fn generate_auth(
    configs: &ConfigsFile,
    schema: &Schema,
    out_dir: &Path,
) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(out_dir)?;
    let guard_specs = collect_auth_guard_specs(configs, schema);

    for guard in &guard_specs {
        let context = build_guard_context(guard)?;
        let rendered = render_template("auth/guard.rs.tpl", &context)?;
        fs::write(out_dir.join(format!("{}.rs", guard.file_stem)), rendered)?;
    }

    let mut mod_context = TemplateContext::new();
    mod_context.insert("guard_modules", render_guard_modules_section(&guard_specs)?)?;
    mod_context.insert(
        "authenticate_any_guard_section",
        render_authenticate_any_guard_section(&guard_specs)?,
    )?;
    let rendered = render_template("auth/mod.rs.tpl", &mod_context)?;
    fs::write(out_dir.join("mod.rs"), rendered)?;
    Ok(())
}

fn collect_auth_guard_specs(configs: &ConfigsFile, schema: &Schema) -> Vec<AuthGuardSpec> {
    configs
        .auth
        .guards
        .iter()
        .map(|(name, config)| {
            let snake_name = to_snake(name);
            let file_stem = format!("{}_guard", snake_name);
            let struct_name = format!("{}Guard", to_title_case(name));
            let provider_name = config.provider.clone();
            let provider_snake = to_snake(&provider_name);
            let provider_title = to_title_case(&provider_name);
            let provider_cfg = schema.models.get(&provider_name).unwrap_or_else(|| {
                panic!(
                    "Guard '{}' provider '{}' not found in schema model definitions",
                    name, provider_name
                )
            });
            let provider_pk = provider_cfg.pk.clone().unwrap_or_else(|| "id".to_string());
            let provider_fields = parse_fields(provider_cfg, &provider_pk);
            let provider_pk_ty = provider_fields
                .iter()
                .find(|field| field.name == provider_pk)
                .map(|field| field.ty.clone())
                .unwrap_or_else(|| {
                    provider_cfg
                        .pk_type
                        .clone()
                        .unwrap_or_else(|| "i64".to_string())
                });

            AuthGuardSpec {
                name: name.clone(),
                file_stem,
                struct_name,
                provider_name,
                provider_snake,
                provider_title,
                provider_pk_ty,
            }
        })
        .collect()
}

fn build_guard_context(guard: &AuthGuardSpec) -> Result<TemplateContext, Box<dyn Error>> {
    let mut context = TemplateContext::new();
    context.insert("struct_name", guard.struct_name.clone())?;
    context.insert("provider_snake", guard.provider_snake.clone())?;
    context.insert("provider_title", guard.provider_title.clone())?;
    context.insert("guard_name", guard.name.clone())?;
    context.insert("provider_name", guard.provider_name.clone())?;
    context.insert(
        "fetch_user_section",
        render_fetch_user_section(
            &guard.name,
            &guard.provider_name,
            &guard.provider_pk_ty,
            &guard.provider_title,
        ),
    )?;
    Ok(context)
}

fn render_guard_modules_section(guards: &[AuthGuardSpec]) -> Result<String, std::fmt::Error> {
    let mut out = String::new();
    for guard in guards {
        writeln!(out, "{}", render_guard_module_entry(guard))?;
    }
    Ok(out.trim_end().to_string())
}

fn render_guard_module_entry(guard: &AuthGuardSpec) -> String {
    format!(
        "pub mod {file_stem};\npub use {file_stem}::{struct_name};",
        file_stem = guard.file_stem,
        struct_name = guard.struct_name
    )
}

fn render_fetch_user_section(
    guard_name: &str,
    provider_name: &str,
    provider_pk_ty: &str,
    provider_title: &str,
) -> String {
    match provider_pk_ty {
        "uuid::Uuid" => format!(
            "        let parsed = match uuid::Uuid::parse_str(id.trim()) {{ Ok(v) => v, Err(_) => return Ok(None), }};\n        {provider_title}Query::new(db, None).find(parsed).await"
        ),
        "i16" | "i32" | "i64" | "u16" | "u32" | "u64" | "isize" | "usize" => format!(
            "        let parsed = match id.trim().parse::<{provider_pk_ty}>() {{ Ok(v) => v, Err(_) => return Ok(None), }};\n        {provider_title}Query::new(db, None).find(parsed).await"
        ),
        "String" => format!(
            "        {provider_title}Query::new(db, None).find(id.to_string()).await"
        ),
        other => panic!(
            "Guard '{}' provider '{}' uses unsupported pk_type '{}' for auth resolution. Supported: i16/i32/i64/u16/u32/u64/isize/usize, uuid, String",
            guard_name, provider_name, other
        ),
    }
}

fn render_authenticate_any_guard_section(
    guards: &[AuthGuardSpec],
) -> Result<String, std::fmt::Error> {
    if guards.is_empty() {
        return Ok("    let _ = (db, token);\n    None".to_string());
    }

    let mut out = String::new();
    for guard in guards {
        writeln!(
            out,
            "    if let Ok(auth) = core_web::auth::authenticate_token::<{}>(db, token).await {{",
            guard.struct_name
        )?;
        writeln!(out, "        if let Some(identity) = auth.as_identity() {{")?;
        writeln!(out, "            return Some(identity);")?;
        writeln!(out, "        }}")?;
        writeln!(out, "    }}")?;
    }
    out.push_str("    None");
    Ok(out)
}
