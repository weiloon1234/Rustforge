fn main() {
    let app_dir = std::path::Path::new("..").join("app");
    let configs_path = app_dir.join("configs.toml");
    let permissions_path = app_dir.join("permissions.toml");
    let schemas_dir = app_dir.join("schemas");
    let out_dir = std::path::Path::new("src");

    println!("cargo:rerun-if-changed={}", configs_path.display());
    println!("cargo:rerun-if-changed={}", permissions_path.display());
    println!("cargo:rerun-if-changed={}", schemas_dir.display());
    println!("cargo:rerun-if-changed=build.rs");

    let (cfgs, _) =
        db_gen::config::load(configs_path.to_str().unwrap()).expect("Failed to load configs");

    let schema =
        db_gen::schema::load(schemas_dir.to_str().unwrap()).expect("Failed to load schemas");
    let permissions = db_gen::load_permissions(permissions_path.to_str().unwrap())
        .expect("Failed to load permissions");

    let models_out = out_dir.join("models");
    if !models_out.exists() {
        std::fs::create_dir_all(&models_out).expect("Failed to create models out");
    }
    db_gen::generate_enums(&schema, &models_out).expect("Failed to gen enums");
    db_gen::generate_models(&schema, &cfgs, &models_out).expect("Failed to gen models");
    apply_updated_at_save_hotfix(&models_out).expect("Failed to patch generated model save hotfix");

    let guards_out = out_dir.join("guards");
    if !guards_out.exists() {
        std::fs::create_dir_all(&guards_out).expect("Failed to create guards out");
    }
    db_gen::generate_auth(&cfgs, &schema, &guards_out).expect("Failed to gen auth");
    db_gen::generate_permissions(&permissions, &out_dir.join("permissions.rs"))
        .expect("Failed to gen permissions");

    db_gen::generate_localized(&cfgs.languages, &cfgs, &schema, out_dir)
        .expect("Failed to gen localized");

    let root_lib = out_dir.join("lib.rs");
    let mut f = std::fs::File::create(&root_lib).expect("Failed to create root lib.rs");
    use std::io::Write;
    writeln!(f, "#![allow(dead_code)]").unwrap();
    writeln!(f, "// AUTO-GENERATED FILE — DO NOT EDIT").unwrap();
    writeln!(f, "pub mod models;").unwrap();
    writeln!(f, "pub mod guards;").unwrap();
    writeln!(f, "pub mod permissions;").unwrap();
    writeln!(f, "pub mod localized;").unwrap();
    writeln!(f, "pub use localized::*;").unwrap();
    writeln!(f, "pub mod extensions;").unwrap();
    writeln!(f, "pub mod generated {{ pub use crate::*; }}").unwrap();
}

fn apply_updated_at_save_hotfix(models_out: &std::path::Path) -> std::io::Result<()> {
    for entry in std::fs::read_dir(models_out)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }

        let source = std::fs::read_to_string(&path)?;
        let mut patched = source.clone();
        let mut changed = false;

        if let Some(next) = patch_model_updated_at_save(&patched) {
            patched = next;
            changed = true;
        }

        if let Some(next) = patch_localized_repo_db_clone(&patched) {
            patched = next;
            changed = true;
        }

        if changed {
            std::fs::write(&path, patched)?;
        }
    }

    Ok(())
}

fn patch_localized_repo_db_clone(source: &str) -> Option<String> {
    const FROM: &str = "LocalizedRepo::new(db);";
    const TO: &str = "LocalizedRepo::new(db.clone());";

    if !source.contains(FROM) {
        return None;
    }
    Some(source.replace(FROM, TO))
}

fn patch_model_updated_at_save(source: &str) -> Option<String> {
    const ORIGINAL_SET_BINDS: &str =
        "let (cols, set_binds): (Vec<_>, Vec<_>) = self.sets.into_iter().unzip();";
    if !source.contains(ORIGINAL_SET_BINDS) || !source.contains("if HAS_UPDATED_AT {") {
        return None;
    }

    let mut out = source.to_string();

    // Introduce local timestamp auto-touch in the primary UPDATE query.
    out = out.replacen(
        ORIGINAL_SET_BINDS,
        "let (mut cols, mut set_binds): (Vec<_>, Vec<_>) = self.sets.into_iter().unzip();",
        1,
    );

    // Replace enum-specific placeholder with concrete enum variant by introspecting current file.
    // We look for the first `enum XxxCol` and use `XxxCol::UpdatedAt`.
    let col_variant = detect_updated_at_col_variant(&out)?;
    let auto_touch = format!(
        "        if HAS_UPDATED_AT && !cols.iter().any(|c| matches!(c, {col_variant})) {{
            let now = time::OffsetDateTime::now_utc();
            cols.push({col_variant});
            set_binds.push(now.into());
        }}
"
    );

    let inject_anchor =
        "let (mut cols, mut set_binds): (Vec<_>, Vec<_>) = self.sets.into_iter().unzip();\n";
    if let Some(pos) = out.find(inject_anchor) {
        out.insert_str(pos + inject_anchor.len(), &auto_touch);
    } else {
        return None;
    }

    // Remove broken second UPDATE pass for updated_at.
    let save_anchor = "let res = db.execute(q).await?;";
    let Some(save_idx) = out.find(save_anchor) else {
        return None;
    };
    let search_start = save_idx + save_anchor.len();
    let Some(if_rel_idx) = out[search_start..].find("if HAS_UPDATED_AT {") else {
        return None;
    };
    let if_start = search_start + if_rel_idx;
    let Some(block_end) = find_block_end(&out, if_start) else {
        return None;
    };

    // Remove optional preceding `let mut target_ids = target_ids;` line if present.
    let mut remove_start = if_start;
    if let Some(line_start) = out[..if_start].rfind('\n') {
        let prev_line_start = out[..line_start].rfind('\n').map(|i| i + 1).unwrap_or(0);
        let prev_line = &out[prev_line_start..line_start];
        if prev_line.trim() == "let mut target_ids = target_ids;" {
            remove_start = prev_line_start;
        }
    }

    out.replace_range(remove_start..block_end, "");
    Some(out)
}

fn detect_updated_at_col_variant(source: &str) -> Option<String> {
    let enum_idx = source.find("enum ")?;
    let rest = &source[enum_idx + "enum ".len()..];
    let name_end = rest.find(char::is_whitespace)?;
    let enum_name = &rest[..name_end];
    Some(format!("{enum_name}::UpdatedAt"))
}

fn find_block_end(source: &str, if_start: usize) -> Option<usize> {
    let brace_start = source[if_start..].find('{')? + if_start;
    let mut depth = 0usize;
    for (idx, ch) in source[brace_start..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    let end = brace_start + idx + ch.len_utf8();
                    let mut end_with_newline = end;
                    if source[end_with_newline..].starts_with('\n') {
                        end_with_newline += 1;
                    }
                    return Some(end_with_newline);
                }
            }
            _ => {}
        }
    }
    None
}
