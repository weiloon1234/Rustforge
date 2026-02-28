use bootstrap::boot::BootContext;
use clap::Subcommand;

#[derive(Subcommand, Debug, Clone)]
pub enum ProjectCommands {
    /// Health check for project command wiring.
    Ping,
}

#[async_trait::async_trait]
impl bootstrap::console::ProjectCommand for ProjectCommands {
    async fn handle(self, _ctx: &BootContext) -> anyhow::Result<()> {
        match self {
            ProjectCommands::Ping => {
                println!("pong");
            }
        }
        Ok(())
    }
}

fn register_seeders(seeders: &mut Vec<Box<dyn core_db::seeder::Seeder>>) {
    app::seeds::register_seeders(seeders);
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    bootstrap::console::start_console::<ProjectCommands, fn(&mut Vec<Box<dyn core_db::seeder::Seeder>>)>(Some(register_seeders))
        .await
}
