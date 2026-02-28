use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use syn::punctuated::Punctuated;
use syn::{Attribute, Expr, ExprLit, Item, Lit, Meta, Token, Visibility};

#[derive(Debug)]
struct DiscoveredTsType {
    rel_path: String,
    rust_path: String,
}

fn main() {
    println!("cargo:rerun-if-changed=src/contracts/api/v1");
    println!("cargo:rerun-if-changed=src/contracts/datatable");

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let src_dir = manifest_dir.join("src");

    let mut discovered = Vec::new();
    let mut portals = BTreeSet::new();

    let mut source_files = Vec::new();
    collect_rs_files(&src_dir.join("contracts/api/v1"), &mut source_files);
    collect_rs_files(&src_dir.join("contracts/datatable"), &mut source_files);
    source_files.sort();

    for source in source_files {
        let rel_to_src = source
            .strip_prefix(&src_dir)
            .unwrap_or_else(|_| panic!("source file not under src/: {}", source.display()));
        let rel_to_src = normalize_separators(&rel_to_src.to_string_lossy());

        if let Some(portal) = portal_from_contract_path(&rel_to_src) {
            portals.insert(portal.to_string());
        }

        let module_path = module_path_from_src_rel(&rel_to_src);
        let file_content = fs::read_to_string(&source)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", source.display()));
        let file = syn::parse_file(&file_content)
            .unwrap_or_else(|e| panic!("failed to parse {}: {e}", source.display()));

        for item in file.items {
            match item {
                Item::Struct(item_struct) => {
                    if !matches!(item_struct.vis, Visibility::Public(_)) {
                        continue;
                    }
                    if !has_ts_derive(&item_struct.attrs) {
                        continue;
                    }
                    let Some(export_to) = parse_ts_export_to(&item_struct.attrs) else {
                        continue;
                    };
                    let rel_path = resolve_rel_path(&rel_to_src, &export_to);
                    let rust_path = format!("app::{module_path}::{}", item_struct.ident);
                    discovered.push(DiscoveredTsType {
                        rel_path,
                        rust_path,
                    });
                }
                Item::Enum(item_enum) => {
                    if !matches!(item_enum.vis, Visibility::Public(_)) {
                        continue;
                    }
                    if !has_ts_derive(&item_enum.attrs) {
                        continue;
                    }
                    let Some(export_to) = parse_ts_export_to(&item_enum.attrs) else {
                        continue;
                    };
                    let rel_path = resolve_rel_path(&rel_to_src, &export_to);
                    let rust_path = format!("app::{module_path}::{}", item_enum.ident);
                    discovered.push(DiscoveredTsType {
                        rel_path,
                        rust_path,
                    });
                }
                _ => {}
            }
        }
    }

    discovered.sort_by(|a, b| {
        a.rel_path
            .cmp(&b.rel_path)
            .then(a.rust_path.cmp(&b.rust_path))
    });

    let mut portals_vec: Vec<String> = portals.into_iter().collect();
    portals_vec.sort();

    let out_file =
        PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR")).join("export_types_registry.rs");
    fs::write(&out_file, render_registry_source(&discovered, &portals_vec))
        .unwrap_or_else(|e| panic!("failed to write {}: {e}", out_file.display()));
}

fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, out);
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

fn normalize_separators(input: &str) -> String {
    input.replace('\\', "/")
}

fn module_path_from_src_rel(src_rel: &str) -> String {
    let trimmed = src_rel.strip_suffix(".rs").unwrap_or(src_rel);
    let without_mod = trimmed.strip_suffix("/mod").unwrap_or(trimmed);
    without_mod.replace('/', "::")
}

fn has_ts_derive(attrs: &[Attribute]) -> bool {
    for attr in attrs {
        if !attr.path().is_ident("derive") {
            continue;
        }
        let Ok(paths) = attr.parse_args_with(Punctuated::<syn::Path, Token![,]>::parse_terminated)
        else {
            continue;
        };
        if paths.iter().any(|path| path.is_ident("TS")) {
            return true;
        }
    }
    false
}

fn parse_ts_export_to(attrs: &[Attribute]) -> Option<String> {
    let mut saw_export = false;
    let mut export_to = None;

    for attr in attrs {
        if !attr.path().is_ident("ts") {
            continue;
        }
        let Ok(metas) = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
        else {
            continue;
        };
        for meta in metas {
            match meta {
                Meta::Path(path) if path.is_ident("export") => {
                    saw_export = true;
                }
                Meta::NameValue(name_value) if name_value.path.is_ident("export_to") => {
                    if let Expr::Lit(ExprLit {
                        lit: Lit::Str(value),
                        ..
                    }) = name_value.value
                    {
                        export_to = Some(value.value());
                    }
                }
                _ => {}
            }
        }
    }

    if !saw_export {
        return None;
    }

    if export_to.is_none() {
        panic!("`#[ts(export)]` is missing `export_to = \"...\"`");
    }

    export_to
}

fn portal_from_contract_path(src_rel: &str) -> Option<&str> {
    portal_after_prefix(src_rel, "contracts/api/v1/")
        .or_else(|| portal_after_prefix(src_rel, "contracts/datatable/"))
}

fn portal_after_prefix<'a>(src_rel: &'a str, prefix: &str) -> Option<&'a str> {
    let rest = src_rel.strip_prefix(prefix)?;
    let (portal, _) = rest.split_once('/')?;
    if portal.is_empty() {
        None
    } else {
        Some(portal)
    }
}

fn resolve_rel_path(src_rel: &str, export_to: &str) -> String {
    let export_to = normalize_separators(export_to);
    if export_to.ends_with(".ts") {
        return export_to;
    }

    let base = export_to.trim_end_matches('/');
    if base.is_empty() {
        panic!("invalid empty `export_to` path in contract type");
    }
    let stem = Path::new(src_rel)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_else(|| panic!("invalid source file stem in `{src_rel}`"));
    let file_name = infer_file_name(src_rel, stem);
    format!("{base}/{file_name}.ts")
}

fn infer_file_name(src_rel: &str, stem: &str) -> String {
    if let Some(portal) = portal_after_prefix(src_rel, "contracts/api/v1/") {
        return if stem == "account" {
            portal.to_string()
        } else {
            format!("{portal}-{stem}")
        };
    }

    if let Some(portal) = portal_after_prefix(src_rel, "contracts/datatable/") {
        return if stem == "account" {
            format!("datatable-{portal}")
        } else {
            format!("datatable-{portal}-{stem}")
        };
    }

    stem.to_string()
}

fn render_registry_source(discovered: &[DiscoveredTsType], portals: &[String]) -> String {
    let mut out = String::new();
    out.push_str("// Auto-generated by app/build.rs. Do not edit manually.\n\n");

    out.push_str("pub(crate) fn discovered_ts_types() -> Vec<AutoTsType> {\n");
    out.push_str("    vec![\n");
    for item in discovered {
        out.push_str(&format!(
            "        AutoTsType {{ rel_path: {rel_path:?}, rust_path: {rust_path:?}, export: || {rust_path}::export_to_string().expect({rust_path:?}) }},\n",
            rel_path = item.rel_path,
            rust_path = item.rust_path
        ));
    }
    out.push_str("    ]\n");
    out.push_str("}\n\n");

    out.push_str("pub(crate) fn discovered_portals() -> &'static [&'static str] {\n");
    out.push_str("    &[\n");
    for portal in portals {
        out.push_str(&format!("        {portal:?},\n"));
    }
    out.push_str("    ]\n");
    out.push_str("}\n");

    out
}
