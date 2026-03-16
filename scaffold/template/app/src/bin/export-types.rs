//! Exports Rust contract types to TypeScript.
//!
//! Uses `ts-rs` to convert Rust types with `#[derive(TS)]` into TypeScript
//! definitions, then writes them to `frontend/src/` alongside framework
//! shared types (ApiResponse, DataTable*, platform primitives, enums).
//!
//! Run: `cargo run -p app --bin export-types`
//! Or:  `make gen-types`

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use ts_rs::TS;

// ── Generated types (ts-rs) ──────────────────────────────────

/// A generated TypeScript file: imports + ts-rs type definitions.
struct TsFile {
    /// Relative path from `frontend/src/`, e.g. `admin/types/admin.ts`
    rel_path: String,
    /// TypeScript definitions produced by ts-rs (collected at runtime).
    definitions: Vec<String>,
    /// Contract-facing enums referenced by the definitions.
    enums: BTreeSet<String>,
    /// Shared framework TS types referenced by the definitions.
    shared_types: BTreeSet<String>,
}

#[derive(Clone, Copy)]
struct AutoTsType {
    rel_path: &'static str,
    rust_path: &'static str,
    export: fn() -> String,
}

#[derive(Clone)]
struct FrameworkTsFile {
    rel_path: String,
    rust_path: String,
    definition: String,
}

include!(concat!(env!("OUT_DIR"), "/export_types_registry.rs"));

fn main() {
    let base = frontend_src_dir();

    // ── 1. Framework-owned shared TS files & registry ──────
    let framework_files = framework_ts_files();
    let shared_type_registry = build_shared_type_registry(&framework_files);

    // ── 2. Contract types via ts-rs ─────────────────────────
    let mut files = load_discovered_ts_files();
    let contract_enum_renderers = collect_contract_enum_renderers();
    let known_contract_enum_names: BTreeSet<String> =
        contract_enum_renderers.keys().cloned().collect();

    // Determine which contract-facing enums are actually referenced by DTOs.
    let known_contract_types = collect_declared_contract_types(&files);
    for ts_file in &mut files {
        let (enums, shared_types) = detect_type_references(
            ts_file.rel_path.as_str(),
            &ts_file.definitions,
            &known_contract_types,
            &known_contract_enum_names,
            &shared_type_registry,
        );
        ts_file.enums = enums;
        ts_file.shared_types = shared_types;
    }

    // Write ts-rs generated files
    for ts_file in &files {
        let path = base.join(&ts_file.rel_path);
        write_file(&path, &assemble(ts_file, &shared_type_registry));
    }
    for (rel_path, definitions) in group_framework_ts_files(&framework_files) {
        let path = base.join(rel_path);
        write_file(&path, &assemble_framework_file(&definitions));
    }

    // ── 3. Enum types (serde-derived) ────────────────────────
    let required_enums_by_portal = collect_required_enums_by_portal(&files);
    let required_enums_shared = collect_required_enums(&required_enums_by_portal);
    let shared_enums_ts = assemble_enums_file(&required_enums_shared, &contract_enum_renderers);
    write_file(&base.join("shared/types/enums.ts"), &shared_enums_ts);

    let all_portals = collect_all_portals(&files, &required_enums_by_portal);
    for portal in &all_portals {
        let enums = required_enums_by_portal
            .get(portal.as_str())
            .cloned()
            .unwrap_or_default();
        let enums_ts = assemble_enums_file(&enums, &contract_enum_renderers);
        write_file(&base.join(format!("{portal}/types/enums.ts")), &enums_ts);
    }

    // ── 4. Portal/shared barrels ─────────────────────────────
    for (portal, content) in assemble_portal_indexes(&all_portals, &files) {
        write_file(&base.join(format!("{portal}/types/index.ts")), &content);
    }
    let shared_index = assemble_shared_index(&files, &framework_files);
    write_file(&base.join("shared/types/index.ts"), &shared_index);

    println!("\nTypeScript types regenerated in {}", base.display());
}

// ── Helpers ──────────────────────────────────────────────────

fn load_discovered_ts_files() -> Vec<TsFile> {
    let mut grouped: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();

    for discovered in discovered_ts_types() {
        grouped
            .entry(discovered.rel_path.to_string())
            .or_default()
            .push((discovered.rust_path.to_string(), (discovered.export)()));
    }

    let mut files = Vec::new();
    for (rel_path, mut exports) in grouped {
        exports.sort_by(|a, b| a.0.cmp(&b.0));
        let definitions = exports.into_iter().map(|(_, def)| def).collect();
        files.push(TsFile {
            rel_path,
            definitions,
            enums: BTreeSet::new(),
            shared_types: BTreeSet::new(),
        });
    }
    files.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));
    files
}

fn frontend_src_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../frontend/src")
}

fn framework_ts_files() -> Vec<FrameworkTsFile> {
    let mut files = Vec::new();

    for file in generated::ts_exports::ts_export_files() {
        files.push(FrameworkTsFile {
            rel_path: file.rel_path.to_string(),
            rust_path: file.rust_path.to_string(),
            definition: file.definition,
        });
    }

    files.sort_by(|a, b| {
        a.rel_path
            .cmp(&b.rel_path)
            .then(a.rust_path.cmp(&b.rust_path))
    });

    let mut seen = BTreeSet::new();
    for file in &files {
        let key = format!("{}::{}", file.rel_path, file.rust_path);
        if !seen.insert(key.clone()) {
            panic!("duplicate framework TS exporter entry: `{key}`");
        }
    }

    files
}

fn group_framework_ts_files(framework_files: &[FrameworkTsFile]) -> BTreeMap<String, Vec<String>> {
    let mut grouped: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
    for file in framework_files {
        grouped
            .entry(file.rel_path.clone())
            .or_default()
            .push((file.rust_path.clone(), file.definition.clone()));
    }

    let mut out = BTreeMap::new();
    for (rel_path, mut defs) in grouped {
        defs.sort_by(|a, b| a.0.cmp(&b.0));
        assert_no_framework_symbol_collisions(&rel_path, &defs);
        out.insert(
            rel_path,
            defs.into_iter().map(|(_, definition)| definition).collect(),
        );
    }
    out
}

fn assert_no_framework_symbol_collisions(rel_path: &str, defs: &[(String, String)]) {
    let mut seen_by_name: BTreeMap<String, String> = BTreeMap::new();
    for (rust_path, definition) in defs {
        let mut declared = BTreeSet::new();
        collect_declared_types(definition, &mut declared);
        for name in declared {
            if let Some(previous) = seen_by_name.insert(name.clone(), rust_path.clone()) {
                panic!(
                    "duplicate framework TS declaration `{name}` in `{rel_path}` from `{previous}` and `{rust_path}`"
                );
            }
        }
    }
}

fn assemble_framework_file(definitions: &[String]) -> String {
    let header = "// Auto-generated by `cargo run -p app --bin export-types`.\n\
                  // Do not edit manually — run `make gen-types` to regenerate.\n";
    let mut out = String::from(header);
    out.push('\n');
    for (index, definition) in definitions.iter().enumerate() {
        if index > 0 {
            out.push('\n');
            out.push('\n');
        }
        out.push_str(definition.trim());
    }
    out.push('\n');
    out
}

fn assemble_shared_index(files: &[TsFile], framework_files: &[FrameworkTsFile]) -> String {
    let mut modules = BTreeSet::new();

    for file in files {
        if let Some(module) = shared_module_from_rel_path(&file.rel_path) {
            modules.insert(module.to_string());
        }
    }
    for file in framework_files {
        if let Some(module) = shared_module_from_rel_path(&file.rel_path) {
            modules.insert(module.to_string());
        }
    }

    modules.insert("enums".to_string());

    let mut out = String::new();
    for module in modules {
        out.push_str(&format!("export * from \"@shared/types/{module}\";\n"));
    }
    out
}

fn shared_module_from_rel_path(rel_path: &str) -> Option<&str> {
    let rel = rel_path.strip_prefix("shared/types/")?;
    let module = rel.strip_suffix(".ts")?;
    if module.is_empty() || module == "index" {
        None
    } else {
        Some(module)
    }
}

/// Build a registry of framework-provided shared types from the generated TS files.
/// Returns: type_name -> module_name (e.g., "LocalizedText" -> "platform")
fn build_shared_type_registry(framework_files: &[FrameworkTsFile]) -> BTreeMap<String, String> {
    let mut registry = BTreeMap::new();
    for file in framework_files {
        let Some(module) = shared_module_from_rel_path(&file.rel_path) else {
            continue;
        };
        let mut declared = BTreeSet::new();
        collect_declared_types(&file.definition, &mut declared);
        for name in declared {
            registry.insert(name, module.to_string());
        }
    }
    registry
}

fn collect_all_portals(
    files: &[TsFile],
    required_enums_by_portal: &BTreeMap<String, BTreeSet<String>>,
) -> BTreeSet<String> {
    let mut portals = BTreeSet::new();

    for portal in discovered_portals() {
        portals.insert((*portal).to_string());
    }

    for file in files {
        if let Some(portal) = portal_from_rel_path(&file.rel_path) {
            portals.insert(portal.to_string());
        }
    }

    portals.extend(required_enums_by_portal.keys().cloned());
    portals
}

fn assemble_portal_indexes(
    portals: &BTreeSet<String>,
    files: &[TsFile],
) -> BTreeMap<String, String> {
    let mut modules_by_portal: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for file in files {
        let Some(portal) = portal_from_rel_path(&file.rel_path) else {
            continue;
        };

        let Some(rel_from_types_dir) = file.rel_path.strip_prefix(&format!("{portal}/types/"))
        else {
            continue;
        };
        if rel_from_types_dir == "index.ts" || rel_from_types_dir == "enums.ts" {
            continue;
        }

        let Some(module_name) = rel_from_types_dir.strip_suffix(".ts") else {
            continue;
        };
        modules_by_portal
            .entry(portal.to_string())
            .or_default()
            .insert(module_name.to_string());
    }

    let mut indexes = BTreeMap::new();
    for portal in portals {
        let mut out = String::new();
        out.push_str(&format!("export * from \"@{portal}/types/enums\";\n"));

        if let Some(modules) = modules_by_portal.get(portal) {
            for module in modules {
                out.push_str(&format!("export * from \"@{portal}/types/{module}\";\n"));
            }
        } else {
            out.push_str("// Add portal-specific contract types here as contracts are created.\n");
        }

        indexes.insert(portal.to_string(), out);
    }
    indexes
}

fn assemble(f: &TsFile, shared_type_registry: &BTreeMap<String, String>) -> String {
    let header = "// Auto-generated by `cargo run -p app --bin export-types`.\n\
                  // Do not edit manually — run `make gen-types` to regenerate.\n";
    let mut out = String::from(header);
    let portal = portal_from_rel_path(&f.rel_path)
        .unwrap_or_else(|| panic!("invalid TS export path (missing portal): {}", f.rel_path));
    for import in shared_import_lines(&f.shared_types, shared_type_registry) {
        out.push_str(&import);
        out.push('\n');
    }
    if let Some(import) = enum_import_line(portal, &f.enums) {
        out.push_str(&import);
        out.push('\n');
        out.push('\n');
    } else if !f.shared_types.is_empty() {
        out.push('\n');
    } else {
        out.push('\n');
    }
    for (i, def) in f.definitions.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        // Strip ts-rs self-referencing imports (e.g. `import type { Foo } from "./Foo"`)
        // since all types are bundled into the same file.
        out.push_str(&strip_local_imports(def));
        out.push('\n');
    }
    out
}

fn assemble_enums_file(
    required_enums: &BTreeSet<String>,
    contract_enum_renderers: &BTreeMap<String, String>,
) -> String {
    let header = "// Auto-generated by `cargo run -p app --bin export-types`.\n\
                  // Do not edit manually — run `make gen-types` to regenerate.\n";
    let mut out = String::from(header);

    if required_enums.is_empty() {
        out.push_str("// No contract-facing enums currently required.\n");
        return out;
    }

    out.push('\n');
    for (idx, name) in required_enums.iter().enumerate() {
        if idx > 0 {
            out.push('\n');
            out.push('\n');
        }
        let rendered = contract_enum_renderers.get(name).unwrap_or_else(|| {
            panic!(
                "missing enum renderer for `{name}`; add it to collect_contract_enum_renderers()"
            )
        });
        out.push_str(&rendered);
    }
    out.push('\n');
    out
}

fn collect_required_enums_by_portal(files: &[TsFile]) -> BTreeMap<String, BTreeSet<String>> {
    let mut out: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for file in files {
        if file.enums.is_empty() {
            continue;
        }
        let portal = portal_from_rel_path(&file.rel_path).unwrap_or_else(|| {
            panic!("invalid TS export path (missing portal): {}", file.rel_path)
        });
        out.entry(portal.to_string())
            .or_default()
            .extend(file.enums.iter().cloned());
    }
    out
}

fn collect_required_enums(
    required_enums_by_portal: &BTreeMap<String, BTreeSet<String>>,
) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for enums in required_enums_by_portal.values() {
        out.extend(enums.iter().cloned());
    }
    out
}

fn collect_declared_contract_types(files: &[TsFile]) -> BTreeSet<String> {
    let mut declared = BTreeSet::new();
    for file in files {
        for def in &file.definitions {
            collect_declared_types(def, &mut declared);
        }
    }
    declared
}

fn detect_type_references(
    rel_path: &str,
    definitions: &[String],
    known_contract_types: &BTreeSet<String>,
    known_contract_enums: &BTreeSet<String>,
    shared_type_registry: &BTreeMap<String, String>,
) -> (BTreeSet<String>, BTreeSet<String>) {
    let mut used_identifiers = BTreeSet::new();
    for def in definitions {
        collect_used_type_identifiers(def, &mut used_identifiers);
    }

    for known_type in known_contract_types {
        used_identifiers.remove(known_type);
    }
    for builtin in ts_builtins() {
        used_identifiers.remove(*builtin);
    }
    let mut shared_types = BTreeSet::new();
    for name in shared_type_registry.keys() {
        if used_identifiers.remove(name) {
            shared_types.insert(name.clone());
        }
    }

    let mut enums = BTreeSet::new();
    let mut unknown = Vec::new();
    for ident in used_identifiers {
        if known_contract_enums.contains(&ident) {
            enums.insert(ident);
        } else {
            unknown.push(ident);
        }
    }

    if !unknown.is_empty() {
        panic!(
            "Type export `{}` references external types without enum exporters: {}",
            rel_path,
            unknown.join(", ")
        );
    }

    (enums, shared_types)
}

fn collect_used_type_identifiers(definition: &str, out: &mut BTreeSet<String>) {
    for line in definition.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty()
            || trimmed.starts_with("//")
            || trimmed.starts_with("/*")
            || trimmed.starts_with('*')
            || trimmed.starts_with("*/")
            || trimmed == "{"
            || trimmed == "}"
        {
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("export type ") {
            if let Some(idx) = rest.find('=') {
                collect_uppercase_identifiers(&rest[idx + 1..], out);
            }
            continue;
        }

        if trimmed.starts_with("export interface ") || trimmed.starts_with("export enum ") {
            continue;
        }

        if let Some(type_expr) = trimmed.strip_prefix('|') {
            collect_uppercase_identifiers(type_expr, out);
            continue;
        }

        if let Some(idx) = trimmed.find(':') {
            collect_uppercase_identifiers(&trimmed[idx + 1..], out);
        }
    }
}

fn collect_declared_types(definition: &str, out: &mut BTreeSet<String>) {
    for line in definition.lines() {
        let trimmed = line.trim_start();
        for prefix in ["export interface ", "export type ", "export enum "] {
            if let Some(rest) = trimmed.strip_prefix(prefix) {
                if let Some(name) = read_identifier(rest) {
                    out.insert(name.to_string());
                }
                break;
            }
        }
    }
}

fn read_identifier(input: &str) -> Option<&str> {
    let mut end = 0usize;
    for (idx, ch) in input.char_indices() {
        if idx == 0 {
            if !(ch.is_ascii_alphabetic() || ch == '_') {
                return None;
            }
            end = ch.len_utf8();
            continue;
        }
        if ch.is_ascii_alphanumeric() || ch == '_' {
            end = idx + ch.len_utf8();
        } else {
            break;
        }
    }
    if end == 0 {
        None
    } else {
        Some(&input[..end])
    }
}

fn collect_uppercase_identifiers(text: &str, out: &mut BTreeSet<String>) {
    let mut token = String::new();
    for ch in text.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            token.push(ch);
        } else {
            maybe_push_uppercase(&token, out);
            token.clear();
        }
    }
    maybe_push_uppercase(&token, out);
}

fn maybe_push_uppercase(token: &str, out: &mut BTreeSet<String>) {
    if token
        .chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_uppercase())
    {
        out.insert(token.to_string());
    }
}

fn ts_builtins() -> &'static [&'static str] {
    &[
        "Array",
        "Date",
        "Map",
        "Promise",
        "ReadonlyArray",
        "Record",
        "Set",
    ]
}

fn collect_contract_enum_renderers() -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();

    for (name, definition) in generated::ts_exports::contract_enum_renderers() {
        let previous = out.insert(name.clone(), definition);
        if previous.is_some() {
            panic!("duplicate contract enum renderer registered for `{name}`");
        }
    }

    out
}

fn portal_from_rel_path(rel_path: &str) -> Option<&str> {
    rel_path.split('/').next()
}

fn enum_import_line(portal: &str, enums: &BTreeSet<String>) -> Option<String> {
    if enums.is_empty() {
        return None;
    }
    let joined = enums
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join(", ");
    Some(format!(
        r#"import type {{ {} }} from "@{}/types/enums";"#,
        joined, portal
    ))
}

fn shared_import_lines(
    shared_types: &BTreeSet<String>,
    shared_type_registry: &BTreeMap<String, String>,
) -> Vec<String> {
    if shared_types.is_empty() {
        return Vec::new();
    }

    let mut by_module: BTreeMap<&str, BTreeSet<String>> = BTreeMap::new();
    for shared_type in shared_types {
        let module = shared_type_registry.get(shared_type).unwrap_or_else(|| {
            panic!("missing shared type module mapping for `{shared_type}`");
        });
        by_module
            .entry(module.as_str())
            .or_default()
            .insert(shared_type.clone());
    }

    let mut lines = Vec::new();
    for (module, types) in by_module {
        let joined = types.into_iter().collect::<Vec<_>>().join(", ");
        lines.push(format!(
            r#"import type {{ {} }} from "@shared/types/{}";"#,
            joined, module
        ));
    }
    lines
}

/// Remove `import type { ... } from "./..."` lines that ts-rs emits for
/// cross-type references. When multiple types are bundled into one file,
/// these self-imports are invalid — the referenced types are already present.
fn strip_local_imports(definition: &str) -> String {
    definition
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            !(trimmed.starts_with("import type ") && trimmed.contains("from \"./"))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn write_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("failed to create directory");
    }
    fs::write(path, content).unwrap_or_else(|e| {
        panic!("failed to write {}: {e}", path.display());
    });
    println!("  wrote {}", path.display());
}
