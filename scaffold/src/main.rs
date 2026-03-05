use anyhow::{bail, Context};
use base64::{engine::general_purpose::STANDARD, Engine};
use clap::Parser;
use colored::*;
use include_dir::{include_dir, Dir, DirEntry, File};
use rand::RngExt;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

static TEMPLATE_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/template");

#[derive(Parser, Debug)]
#[command(name = "scaffold")]
#[command(about = "Generate a minimal Rustforge starter project", long_about = None)]
struct Cli {
    /// Output directory for starter project
    #[arg(long)]
    output: PathBuf,

    /// Overwrite output directory when it is non-empty
    #[arg(long, short)]
    force: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let output = normalize_output_path(&cli.output)?;

    println!("{}", "Rustforge Starter Scaffold".bold().cyan());
    println!("{} {}", "Output:".bold(), output.display());

    ensure_output_ready(&output, cli.force)?;

    let app_key = generate_app_key();
    let replacements = [("APP_KEY", app_key.as_str())];

    let files = template_files();
    if cli.force {
        cleanup_module_path_conflicts(&output, files.iter().map(|file| file.path()))?;
    }
    for file in files {
        let path = output.join(file.path());
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }

        if let Some(content) = file.contents_utf8() {
            let rendered = render_template(content, &replacements);
            fs::write(&path, rendered.as_bytes())
                .with_context(|| format!("failed to write {}", path.display()))?;

            if is_shebang_script(rendered.as_bytes()) {
                make_executable(&path)?;
            }
        } else {
            fs::write(&path, file.contents())
                .with_context(|| format!("failed to write {}", path.display()))?;
        }

        println!("{} {}", "Created".green(), path.display());
    }

    println!("\n{}", "Starter scaffold generated.".bold().green());
    println!("{}", "Next:".cyan());
    println!("{}", "  cd <output>".cyan());
    println!("{}", "  ./console migrate pump".cyan());
    println!("{}", "  ./console migrate run".cyan());
    println!("{}", "  cargo check -p app".cyan());

    Ok(())
}

fn template_files() -> Vec<&'static File<'static>> {
    let mut files = Vec::new();
    collect_template_files(&TEMPLATE_DIR, &mut files);
    files.sort_by(|a, b| a.path().cmp(b.path()));
    files
}

fn collect_template_files<'a>(dir: &'a Dir<'a>, out: &mut Vec<&'a File<'a>>) {
    for entry in dir.entries() {
        match entry {
            DirEntry::Dir(child) => collect_template_files(child, out),
            DirEntry::File(file) => out.push(file),
        }
    }
}

fn render_template(content: &str, replacements: &[(&str, &str)]) -> String {
    let mut rendered = content.to_owned();

    for (key, value) in replacements {
        let token = format!("{{{{{key}}}}}");
        rendered = rendered.replace(&token, value);
    }

    rendered
}

fn is_shebang_script(bytes: &[u8]) -> bool {
    bytes.starts_with(b"#!")
}

fn generate_app_key() -> String {
    let mut key = [0u8; 32];
    rand::rng().fill(&mut key[..]);
    format!("base64:{}", STANDARD.encode(key))
}

fn normalize_output_path(output: &Path) -> anyhow::Result<PathBuf> {
    if output.as_os_str().is_empty() {
        bail!("--output must not be empty");
    }

    let path = if output.is_absolute() {
        output.to_path_buf()
    } else {
        std::env::current_dir()?.join(output)
    };

    Ok(path)
}

fn ensure_output_ready(output: &Path, force: bool) -> anyhow::Result<()> {
    if output.exists() {
        if !output.is_dir() {
            bail!(
                "Output path exists and is not a directory: {}",
                output.display()
            );
        }

        let mut entries =
            fs::read_dir(output).with_context(|| format!("failed to read {}", output.display()))?;
        let non_empty = entries.next().transpose()?.is_some();

        if non_empty && !force {
            bail!(
                "Refusing to scaffold into non-empty directory: {}\nUse --force to overwrite.",
                output.display()
            );
        }
    } else {
        fs::create_dir_all(output)
            .with_context(|| format!("failed to create {}", output.display()))?;
    }

    Ok(())
}

fn cleanup_module_path_conflicts<'a, I>(output: &Path, template_paths: I) -> anyhow::Result<()>
where
    I: IntoIterator<Item = &'a Path>,
{
    // Generic Rust module conflict cleanup:
    // if template contains `.../<module>/mod.rs`, remove stale `.../<module>.rs`
    // in output to avoid E0761 ambiguity.
    let mut removed = BTreeSet::new();

    for rel in template_paths {
        if rel.file_name().and_then(|name| name.to_str()) != Some("mod.rs") {
            continue;
        }

        let Some(module_dir) = rel.parent() else {
            continue;
        };
        let Some(module_name) = module_dir.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let Some(parent_dir) = module_dir.parent() else {
            continue;
        };

        let flat_rel = parent_dir.join(format!("{module_name}.rs"));
        if !removed.insert(flat_rel.clone()) {
            continue;
        }

        let flat_abs = output.join(&flat_rel);
        if flat_abs.is_file() {
            fs::remove_file(&flat_abs)
                .with_context(|| format!("failed to remove conflicting module {}", flat_abs.display()))?;
            println!("{} {}", "Removed conflict".yellow(), flat_abs.display());
        }
    }

    Ok(())
}

#[cfg(unix)]
fn make_executable(path: &Path) -> anyhow::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let mut perms = fs::metadata(path)
        .with_context(|| format!("failed to stat {}", path.display()))?
        .permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms)
        .with_context(|| format!("failed to chmod {}", path.display()))?;
    Ok(())
}

#[cfg(not(unix))]
fn make_executable(path: &Path) -> anyhow::Result<()> {
    let _ = path;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_template_replaces_known_tokens_only() {
        let rendered = render_template(
            "APP_KEY={{APP_KEY}}\nMISSING={{MISSING}}\n",
            &[("APP_KEY", "abc")],
        );
        assert_eq!(rendered, "APP_KEY=abc\nMISSING={{MISSING}}\n");
    }

    #[test]
    fn shebang_detection_is_prefix_based() {
        assert!(is_shebang_script(b"#!/usr/bin/env bash\necho hi\n"));
        assert!(!is_shebang_script(b"echo hi\n"));
    }
}
