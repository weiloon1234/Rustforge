mod templates;

use anyhow::{bail, Context};
use base64::{engine::general_purpose::STANDARD, Engine};
use clap::Parser;
use colored::*;
use rand::RngExt;
use std::fs;
use std::path::{Path, PathBuf};

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

#[derive(Debug, Clone, Copy)]
struct FileTemplate {
    path: &'static str,
    content: &'static str,
    executable: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let output = normalize_output_path(&cli.output)?;

    println!("{}", "Rustforge Starter Scaffold".bold().cyan());
    println!("{} {}", "Output:".bold(), output.display());

    ensure_output_ready(&output, cli.force)?;

    let app_key = generate_app_key();
    let files = file_templates();

    for file in files {
        let path = output.join(file.path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }

        let content = file.content.replace("{{APP_KEY}}", &app_key);
        fs::write(&path, content)
            .with_context(|| format!("failed to write {}", path.display()))?;

        if file.executable {
            make_executable(&path)?;
        }

        println!("{} {}", "Created".green(), path.display());
    }

    println!("\n{}", "Starter scaffold generated.".bold().green());
    println!("{}", "Next: cd <output> && cargo check".cyan());

    Ok(())
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

fn file_templates() -> Vec<FileTemplate> {
    vec![
        FileTemplate {
            path: "Cargo.toml",
            content: templates::ROOT_CARGO_TOML,
            executable: false,
        },
        FileTemplate {
            path: ".env.example",
            content: templates::ROOT_ENV_EXAMPLE,
            executable: false,
        },
        FileTemplate {
            path: ".gitignore",
            content: templates::ROOT_GITIGNORE,
            executable: false,
        },
        FileTemplate {
            path: ".gitattributes",
            content: templates::ROOT_GITATTRIBUTES,
            executable: false,
        },
        FileTemplate {
            path: "Makefile",
            content: templates::ROOT_MAKEFILE,
            executable: false,
        },
        FileTemplate {
            path: "README.md",
            content: templates::ROOT_README_MD,
            executable: false,
        },
        FileTemplate {
            path: "i18n/en.json",
            content: templates::ROOT_I18N_EN_JSON,
            executable: false,
        },
        FileTemplate {
            path: "i18n/zh.json",
            content: templates::ROOT_I18N_ZH_JSON,
            executable: false,
        },
        FileTemplate {
            path: "console",
            content: templates::ROOT_CONSOLE,
            executable: true,
        },
        FileTemplate {
            path: "bin/api-server",
            content: templates::BIN_API_SERVER,
            executable: true,
        },
        FileTemplate {
            path: "bin/websocket-server",
            content: templates::BIN_WEBSOCKET_SERVER,
            executable: true,
        },
        FileTemplate {
            path: "bin/worker",
            content: templates::BIN_WORKER,
            executable: true,
        },
        FileTemplate {
            path: "bin/console",
            content: templates::BIN_CONSOLE,
            executable: true,
        },
        FileTemplate {
            path: "scripts/install-ubuntu.sh",
            content: templates::SCRIPT_INSTALL_UBUNTU_SH,
            executable: true,
        },
        FileTemplate {
            path: "scripts/update.sh",
            content: templates::SCRIPT_UPDATE_SH,
            executable: true,
        },
        FileTemplate {
            path: "migrations/.gitkeep",
            content: templates::MIGRATIONS_GITKEEP,
            executable: false,
        },
        FileTemplate {
            path: "public/.gitkeep",
            content: templates::PUBLIC_GITKEEP,
            executable: false,
        },
        FileTemplate {
            path: "migrations/0000000001000_admin_auth.sql",
            content: templates::MIGRATION_ADMIN_AUTH_SQL,
            executable: false,
        },
        FileTemplate {
            path: "app/Cargo.toml",
            content: templates::APP_CARGO_TOML,
            executable: false,
        },
        FileTemplate {
            path: "app/configs.toml",
            content: templates::APP_CONFIGS_TOML,
            executable: false,
        },
        FileTemplate {
            path: "app/permissions.toml",
            content: templates::APP_PERMISSIONS_TOML,
            executable: false,
        },
        FileTemplate {
            path: "app/schemas/admin.toml",
            content: templates::APP_SCHEMA_ADMIN_TOML,
            executable: false,
        },
        FileTemplate {
            path: "app/src/lib.rs",
            content: templates::APP_LIB_RS,
            executable: false,
        },
        FileTemplate {
            path: "app/src/contracts/mod.rs",
            content: templates::APP_CONTRACTS_MOD_RS,
            executable: false,
        },
        FileTemplate {
            path: "app/src/contracts/types/mod.rs",
            content: templates::APP_CONTRACTS_TYPES_MOD_RS,
            executable: false,
        },
        FileTemplate {
            path: "app/src/contracts/types/username.rs",
            content: templates::APP_CONTRACTS_TYPES_USERNAME_RS,
            executable: false,
        },
        FileTemplate {
            path: "app/src/contracts/datatable/mod.rs",
            content: templates::APP_CONTRACTS_DATATABLE_MOD_RS,
            executable: false,
        },
        FileTemplate {
            path: "app/src/contracts/datatable/admin/mod.rs",
            content: templates::APP_CONTRACTS_DATATABLE_ADMIN_MOD_RS,
            executable: false,
        },
        FileTemplate {
            path: "app/src/contracts/datatable/admin/admin.rs",
            content: templates::APP_CONTRACTS_DATATABLE_ADMIN_ADMIN_RS,
            executable: false,
        },
        FileTemplate {
            path: "app/src/contracts/api/mod.rs",
            content: templates::APP_CONTRACTS_API_MOD_RS,
            executable: false,
        },
        FileTemplate {
            path: "app/src/contracts/api/v1/mod.rs",
            content: templates::APP_CONTRACTS_API_V1_MOD_RS,
            executable: false,
        },
        FileTemplate {
            path: "app/src/contracts/api/v1/admin.rs",
            content: templates::APP_CONTRACTS_API_V1_ADMIN_RS,
            executable: false,
        },
        FileTemplate {
            path: "app/src/contracts/api/v1/admin_auth.rs",
            content: templates::APP_CONTRACTS_API_V1_ADMIN_AUTH_RS,
            executable: false,
        },
        FileTemplate {
            path: "app/src/validation/mod.rs",
            content: templates::APP_VALIDATION_MOD_RS,
            executable: false,
        },
        FileTemplate {
            path: "app/src/validation/sync.rs",
            content: templates::APP_VALIDATION_SYNC_RS,
            executable: false,
        },
        FileTemplate {
            path: "app/src/validation/username.rs",
            content: templates::APP_VALIDATION_USERNAME_RS,
            executable: false,
        },
        FileTemplate {
            path: "app/src/validation/db.rs",
            content: templates::APP_VALIDATION_DB_RS,
            executable: false,
        },
        FileTemplate {
            path: "app/src/internal/mod.rs",
            content: templates::APP_INTERNAL_MOD_RS,
            executable: false,
        },
        FileTemplate {
            path: "app/src/internal/api/mod.rs",
            content: templates::APP_INTERNAL_API_MOD_RS,
            executable: false,
        },
        FileTemplate {
            path: "app/src/internal/api/state.rs",
            content: templates::APP_INTERNAL_API_STATE_RS,
            executable: false,
        },
        FileTemplate {
            path: "app/src/internal/api/datatable.rs",
            content: templates::APP_INTERNAL_API_DATATABLE_RS,
            executable: false,
        },
        FileTemplate {
            path: "app/src/internal/api/v1/mod.rs",
            content: templates::APP_INTERNAL_API_V1_MOD_RS,
            executable: false,
        },
        FileTemplate {
            path: "app/src/internal/api/v1/admin.rs",
            content: templates::APP_INTERNAL_API_V1_ADMIN_RS,
            executable: false,
        },
        FileTemplate {
            path: "app/src/internal/api/v1/admin_auth.rs",
            content: templates::APP_INTERNAL_API_V1_ADMIN_AUTH_RS,
            executable: false,
        },
        FileTemplate {
            path: "app/src/internal/middleware/mod.rs",
            content: templates::APP_INTERNAL_MIDDLEWARE_MOD_RS,
            executable: false,
        },
        FileTemplate {
            path: "app/src/internal/middleware/auth.rs",
            content: templates::APP_INTERNAL_MIDDLEWARE_AUTH_RS,
            executable: false,
        },
        FileTemplate {
            path: "app/src/internal/workflows/mod.rs",
            content: templates::APP_INTERNAL_WORKFLOWS_MOD_RS,
            executable: false,
        },
        FileTemplate {
            path: "app/src/internal/workflows/admin.rs",
            content: templates::APP_INTERNAL_WORKFLOWS_ADMIN_RS,
            executable: false,
        },
        FileTemplate {
            path: "app/src/internal/workflows/admin_auth.rs",
            content: templates::APP_INTERNAL_WORKFLOWS_ADMIN_AUTH_RS,
            executable: false,
        },
        FileTemplate {
            path: "app/src/internal/realtime/mod.rs",
            content: templates::APP_INTERNAL_REALTIME_MOD_RS,
            executable: false,
        },
        FileTemplate {
            path: "app/src/internal/jobs/mod.rs",
            content: templates::APP_INTERNAL_JOBS_MOD_RS,
            executable: false,
        },
        FileTemplate {
            path: "app/src/internal/datatables/mod.rs",
            content: templates::APP_INTERNAL_DATATABLES_MOD_RS,
            executable: false,
        },
        FileTemplate {
            path: "app/src/internal/datatables/admin.rs",
            content: templates::APP_INTERNAL_DATATABLES_ADMIN_RS,
            executable: false,
        },
        FileTemplate {
            path: "app/src/seeds/mod.rs",
            content: templates::APP_SEEDS_MOD_RS,
            executable: false,
        },
        FileTemplate {
            path: "app/src/seeds/countries_seeder.rs",
            content: templates::APP_SEEDS_COUNTRIES_RS,
            executable: false,
        },
        FileTemplate {
            path: "app/src/seeds/admin_bootstrap_seeder.rs",
            content: templates::APP_SEEDS_ADMIN_BOOTSTRAP_RS,
            executable: false,
        },
        FileTemplate {
            path: "app/src/bin/api-server.rs",
            content: templates::APP_BIN_API_SERVER_RS,
            executable: false,
        },
        FileTemplate {
            path: "app/src/bin/websocket-server.rs",
            content: templates::APP_BIN_WEBSOCKET_SERVER_RS,
            executable: false,
        },
        FileTemplate {
            path: "app/src/bin/worker.rs",
            content: templates::APP_BIN_WORKER_RS,
            executable: false,
        },
        FileTemplate {
            path: "app/src/bin/console.rs",
            content: templates::APP_BIN_CONSOLE_RS,
            executable: false,
        },
        FileTemplate {
            path: "generated/Cargo.toml",
            content: templates::GENERATED_CARGO_TOML,
            executable: false,
        },
        FileTemplate {
            path: "generated/build.rs",
            content: templates::GENERATED_BUILD_RS,
            executable: false,
        },
        FileTemplate {
            path: "generated/src/lib.rs",
            content: templates::GENERATED_LIB_RS,
            executable: false,
        },
        FileTemplate {
            path: "generated/src/extensions.rs",
            content: templates::GENERATED_EXTENSIONS_RS,
            executable: false,
        },
    ]
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
