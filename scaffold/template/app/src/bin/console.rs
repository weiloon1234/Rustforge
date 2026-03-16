use bootstrap::boot::BootContext;
use clap::Subcommand;
use core_realtime::RealtimePublisher;

#[derive(Subcommand, Debug, Clone)]
pub enum ProjectCommands {
    /// Health check for project command wiring.
    Ping,

    /// Push a raw event to a realtime channel.
    ///
    /// Examples:
    ///   ./console realtime-push --channel admin --event notification_counts --payload '{"deposit":5,"withdrawal":3}'
    ///   ./console realtime-push --channel user --event test --payload '{"message":"hello"}'
    ///   ./console realtime-push --channel admin --event test --room "country:admin" --payload '{"key":"value"}'
    RealtimePush {
        /// Channel name (e.g. "admin", "user", "public")
        #[arg(long)]
        channel: String,

        /// Event name (e.g. "notification_counts", "test")
        #[arg(long)]
        event: String,

        /// JSON payload string
        #[arg(long)]
        payload: String,

        /// Optional room for targeted delivery (omit for broadcast)
        #[arg(long)]
        room: Option<String>,
    },
}

#[async_trait::async_trait]
impl bootstrap::console::ProjectCommand for ProjectCommands {
    async fn handle(self, ctx: &BootContext) -> anyhow::Result<()> {
        match self {
            ProjectCommands::Ping => {
                println!("pong");
            }
            ProjectCommands::RealtimePush {
                channel,
                event,
                payload,
                room,
            } => {
                let data: serde_json::Value = serde_json::from_str(&payload)
                    .map_err(|e| anyhow::anyhow!("Invalid JSON payload: {e}"))?;

                let publisher = RealtimePublisher::from_realtime_settings(
                    &ctx.settings.redis.url,
                    &ctx.settings.realtime,
                )?;

                publisher
                    .publish_raw(
                        &channel,
                        &event,
                        room.as_deref(),
                        data,
                    )
                    .await?;

                let target = room.as_deref().unwrap_or("(broadcast)");
                println!("Published to channel={channel} event={event} room={target}");
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
    bootstrap::console::start_console::<
        ProjectCommands,
        fn(&mut Vec<Box<dyn core_db::seeder::Seeder>>),
    >(Some(register_seeders))
    .await
}
