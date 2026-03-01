# Custom Project Commands Guide

Use this only when you need app-specific CLI commands beyond built-ins.

## Where to implement

File: `app/src/bin/console.rs`

Use:

1. `clap` derive for command shape
2. `bootstrap::console::ProjectCommand` trait for execution
3. `BootContext` for app resources

## Example

```rust
use bootstrap::boot::BootContext;
use clap::Subcommand;

#[derive(Subcommand, Debug, Clone)]
pub enum ProjectCommands {
    /// Simple command with no args
    Ping,

    /// Command with args
    Demo {
        #[arg(long)]
        name: String,
    },

    /// Nested subcommand group
    #[command(subcommand)]
    Cache(CacheCommands),
}

#[derive(Subcommand, Debug, Clone)]
pub enum CacheCommands {
    /// Flush application cache
    Flush,
}

#[async_trait::async_trait]
impl bootstrap::console::ProjectCommand for ProjectCommands {
    async fn handle(self, ctx: &BootContext) -> anyhow::Result<()> {
        match self {
            Self::Ping => println!("pong"),
            Self::Demo { name } => {
                println!("Hello {name} from {}", ctx.settings.app.name);
            }
            Self::Cache(CacheCommands::Flush) => {
                ctx.redis.flush().await?;
                println!("Cache flushed");
            }
        }
        Ok(())
    }
}
```

## BootContext resources

`BootContext` gives access to:

- `db`
- `redis`
- `storage`
- `queue`
- `mailer`
- `settings`

Use these directly in command handlers; avoid global singletons.

## Verification

```bash
cargo check -p app
./console --help
./console ping
./console demo --name test
./console cache flush
```

