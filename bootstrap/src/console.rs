use crate::boot::{init_app, BootContext};
use anyhow::Result;
use clap::Parser;

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

                        for seeder in seeders {
                            if let Some(target_name) = &name {
                                // Specific execution: Only run if matches name (ignore default flag)
                                if !seeder.name().eq_ignore_ascii_case(target_name) {
                                    continue;
                                }
                            } else {
                                // Default execution: Only run if enabled by default
                                if !seeder.run_by_default() {
                                    continue;
                                }
                            }

                            tracing::info!("Running Seeder: {}", seeder.name());
                            seeder.run(&ctx.db).await?;
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
