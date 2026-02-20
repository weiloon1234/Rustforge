use clap::Subcommand;

#[derive(Subcommand)]
pub enum CoreCommands {
    /// Framework migration tools
    #[command(subcommand)]
    Migrate(MigrateCommands),

    /// Database utilities
    #[command(subcommand)]
    Db(DbCommands),

    /// Scaffolding tools
    #[command(subcommand)]
    Make(MakeCommands),
}

#[derive(Subcommand)]
pub enum MigrateCommands {
    /// Run pending migrations
    Run,
    /// Revert last migration
    Revert,
    /// List migrations
    Info,
    /// Create a new migration file (alias to sqlx migrate add)
    Add { name: String },
    /// Generate framework internal migrations
    Pump,
}

#[derive(Subcommand)]
pub enum DbCommands {
    /// Seed the database
    Seed {
        /// Optional name of the seeder to run (e.g. UserSeeder)
        #[arg(long)]
        name: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum MakeCommands {
    /// Create a new seeder
    Seeder {
        /// Name of the seeder (e.g. UserSeeder)
        name: String,

        /// Target directory for seeder files (default: APP_SEEDERS_DIR or src/seeds)
        #[arg(long)]
        dir: Option<String>,
    },
}

/// Handler for Core Commands
/// Note: This handler can do non-app-specific things.
/// For app-specific things (like running seeders), it might need a callback or be handled in the app.
pub async fn handle(cmd: CoreCommands) -> anyhow::Result<()> {
    match cmd {
        CoreCommands::Migrate(sub) => handle_migrate(sub).await,
        CoreCommands::Make(sub) => handle_make(sub).await,
        _ => Ok(()), // Some commands might be handled by the app (like Db::Seed)
    }
}

async fn handle_migrate(cmd: MigrateCommands) -> anyhow::Result<()> {
    match cmd {
        MigrateCommands::Pump => super::migrations::pump().await,
        MigrateCommands::Run => super::sqlx_tool::handle(super::sqlx_tool::MigrateCommand::Run),
        MigrateCommands::Revert => {
            super::sqlx_tool::handle(super::sqlx_tool::MigrateCommand::Revert)
        }
        MigrateCommands::Info => super::sqlx_tool::handle(super::sqlx_tool::MigrateCommand::Info),
        MigrateCommands::Add { name } => {
            super::sqlx_tool::handle(super::sqlx_tool::MigrateCommand::Add { name })
        }
    }
}

async fn handle_make(cmd: MakeCommands) -> anyhow::Result<()> {
    match cmd {
        MakeCommands::Seeder { name, dir } => {
            let target_dir = resolve_seeders_dir(dir);
            super::seeder::make_seeder(&name, &target_dir).await?;
            Ok(())
        }
    }
}

fn resolve_seeders_dir(explicit: Option<String>) -> std::path::PathBuf {
    if let Some(dir) = explicit.filter(|v| !v.trim().is_empty()) {
        return std::path::PathBuf::from(dir);
    }

    if let Ok(dir) = std::env::var("APP_SEEDERS_DIR") {
        if !dir.trim().is_empty() {
            return std::path::PathBuf::from(dir);
        }
    }

    std::path::PathBuf::from("src/seeds")
}
