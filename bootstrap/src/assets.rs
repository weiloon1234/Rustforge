use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use clap::Subcommand;

#[derive(Subcommand, Debug, Clone)]
pub enum AssetCommands {
    /// Publish static build artifacts to PUBLIC_PATH (or --to)
    Publish {
        /// Source directory (for example: frontend/dist)
        #[arg(long)]
        from: PathBuf,

        /// Destination directory (default: PUBLIC_PATH env, fallback: public)
        #[arg(long)]
        to: Option<PathBuf>,

        /// Remove current destination contents before copy
        #[arg(long, default_value_t = false)]
        clean: bool,
    },
}

pub fn handle(cmd: AssetCommands) -> Result<()> {
    dotenvy::dotenv().ok();

    match cmd {
        AssetCommands::Publish { from, to, clean } => publish(from, to, clean),
    }
}

fn publish(from: PathBuf, to: Option<PathBuf>, clean: bool) -> Result<()> {
    if !from.is_dir() {
        bail!(
            "Asset source directory does not exist or is not a directory: {}",
            from.display()
        );
    }

    let target = resolve_public_path(to);
    std::fs::create_dir_all(&target)
        .with_context(|| format!("failed to create target directory {}", target.display()))?;

    if clean {
        clean_directory(&target)?;
    }

    let copied = copy_dir_recursive(&from, &target)?;

    println!(
        "Published {} file(s) from {} -> {}",
        copied,
        from.display(),
        target.display()
    );

    Ok(())
}

fn resolve_public_path(explicit: Option<PathBuf>) -> PathBuf {
    if let Some(path) = explicit {
        return path;
    }

    std::env::var("PUBLIC_PATH")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("public"))
}

fn clean_directory(dir: &Path) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }

    for entry in std::fs::read_dir(dir)
        .with_context(|| format!("failed to read directory {}", dir.display()))?
    {
        let path = entry?.path();
        if path.is_dir() {
            std::fs::remove_dir_all(&path)
                .with_context(|| format!("failed to remove directory {}", path.display()))?;
        } else {
            std::fs::remove_file(&path)
                .with_context(|| format!("failed to remove file {}", path.display()))?;
        }
    }

    Ok(())
}

fn copy_dir_recursive(from: &Path, to: &Path) -> Result<usize> {
    let mut copied = 0usize;
    let mut stack = vec![from.to_path_buf()];

    while let Some(current) = stack.pop() {
        for entry in std::fs::read_dir(&current)
            .with_context(|| format!("failed to read directory {}", current.display()))?
        {
            let path = entry?.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }

            if !path.is_file() {
                continue;
            }

            let relative = path
                .strip_prefix(from)
                .with_context(|| format!("failed to strip prefix {}", from.display()))?;
            let target = to.join(relative);
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent).with_context(|| {
                    format!("failed to create parent directory {}", parent.display())
                })?;
            }

            std::fs::copy(&path, &target).with_context(|| {
                format!(
                    "failed to copy asset from {} to {}",
                    path.display(),
                    target.display()
                )
            })?;
            copied += 1;
        }
    }

    Ok(copied)
}
