use crate::boot::{init_app, BootContext};
use anyhow::Result;
use clap::Parser;

fn normalize_seeder_match_key(value: &str) -> String {
    let normalized: String = value
        .trim()
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .map(|ch| ch.to_ascii_lowercase())
        .collect();

    normalized
        .strip_suffix("seeder")
        .unwrap_or(&normalized)
        .to_string()
}

fn seeder_matches_requested_name(seeder_name: &str, requested_name: &str) -> bool {
    let requested = normalize_seeder_match_key(requested_name);
    !requested.is_empty() && normalize_seeder_match_key(seeder_name) == requested
}

fn format_available_seeders(seeders: &[Box<dyn core_db::seeder::Seeder>]) -> String {
    let names: Vec<&str> = seeders.iter().map(|seeder| seeder.name()).collect();
    if names.is_empty() {
        "none registered".to_string()
    } else {
        names.join(", ")
    }
}

/// Trait for a project-specific CLI command enum.
#[async_trait::async_trait]
pub trait ProjectCommand: clap::Subcommand + Sized + Send {
    async fn handle(self, ctx: &BootContext) -> Result<()>;
}

/// Helper struct to combine Framework commands with Project commands.
#[derive(Parser)]
pub struct FrameworkCli<C: clap::Subcommand> {
    #[command(subcommand)]
    pub command: FrameworkCommand<C>,
}

#[derive(clap::Subcommand)]
pub enum FrameworkCommand<C: clap::Subcommand> {
    /// Database migration commands
    #[command(subcommand)]
    Migrate(core_db::commands::cli::MigrateCommands),

    /// Database utility commands (seeds)
    #[command(subcommand)]
    Db(core_db::commands::cli::DbCommands),

    /// Generator commands
    #[command(subcommand)]
    Make(core_db::commands::cli::MakeCommands),

    /// Static asset utility commands
    #[command(subcommand)]
    Assets(crate::assets::AssetCommands),

    /// Project specific commands
    #[command(flatten)]
    Project(C),
}

/// Starts the CLI console.
///
/// # Arguments
/// * `register_seeders` - Function to register app-specific seeders.
pub async fn start_console<C, F>(register_seeders: Option<F>) -> Result<()>
where
    C: ProjectCommand + clap::Subcommand,
    F: Fn(&mut Vec<Box<dyn core_db::seeder::Seeder>>) + Send + Sync,
{
    // 1. Parse Args First (to avoid booting if just --help)
    let cli = FrameworkCli::<C>::parse();
    match cli.command {
        FrameworkCommand::Migrate(cmd) => {
            dotenvy::dotenv().ok();
            core_db::commands::cli::handle(core_db::commands::cli::CoreCommands::Migrate(cmd))
                .await?
        }
        FrameworkCommand::Make(cmd) => {
            dotenvy::dotenv().ok();
            core_db::commands::cli::handle(core_db::commands::cli::CoreCommands::Make(cmd)).await?
        }
        FrameworkCommand::Assets(cmd) => crate::assets::handle(cmd)?,
        FrameworkCommand::Db(cmd) => {
            let (ctx, _guard) = init_app().await?;
            // Intercept Seed
            match cmd {
                core_db::commands::cli::DbCommands::Seed { name } => {
                    tracing::info!("Running Database Seeder...");
                    if let Some(registrar) = register_seeders {
                        let mut seeders: Vec<Box<dyn core_db::seeder::Seeder>> = Vec::new();
                        registrar(&mut seeders);
                        let mut matched = false;

                        for seeder in &seeders {
                            if let Some(target_name) = &name {
                                if !seeder_matches_requested_name(seeder.name(), target_name) {
                                    continue;
                                }
                            } else {
                                // Default execution: Only run if enabled by default
                                if !seeder.run_by_default() {
                                    continue;
                                }
                            }

                            matched = true;
                            tracing::info!("Running Seeder: {}", seeder.name());
                            seeder.run(&ctx.db).await?;
                        }

                        if let Some(target_name) = &name {
                            if !matched {
                                anyhow::bail!(
                                    "Seeder '{}' was not found. Available seeders: {}",
                                    target_name,
                                    format_available_seeders(&seeders)
                                );
                            }
                        }
                    }
                    tracing::info!("Database Seeding Completed.");
                    // Return () as expected by match arm
                }
            }
        }
        FrameworkCommand::Project(cmd) => {
            let (ctx, _guard) = init_app().await?;
            cmd.handle(&ctx).await?
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{normalize_seeder_match_key, seeder_matches_requested_name};

    #[test]
    fn seeder_match_normalization_accepts_suffix_and_common_separators() {
        assert_eq!(
            normalize_seeder_match_key("AdminBootstrapSeeder"),
            "adminbootstrap"
        );
        assert_eq!(
            normalize_seeder_match_key("admin_bootstrap"),
            "adminbootstrap"
        );
        assert_eq!(
            normalize_seeder_match_key("admin-bootstrap"),
            "adminbootstrap"
        );
        assert_eq!(
            normalize_seeder_match_key(" admin bootstrap "),
            "adminbootstrap"
        );
    }

    #[test]
    fn seeder_matcher_accepts_short_and_full_names_case_insensitively() {
        assert!(seeder_matches_requested_name(
            "AdminBootstrapSeeder",
            "AdminBootstrap"
        ));
        assert!(seeder_matches_requested_name(
            "AdminBootstrapSeeder",
            "admin_bootstrap_seeder"
        ));
        assert!(seeder_matches_requested_name(
            "CountriesSeeder",
            "countries"
        ));
        assert!(!seeder_matches_requested_name("CountriesSeeder", "admins"));
    }
}
