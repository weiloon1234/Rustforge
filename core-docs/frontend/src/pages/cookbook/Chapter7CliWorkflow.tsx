import { useEffect } from 'react'
import Prism from 'prismjs'

export function Chapter7CliWorkflow() {
    useEffect(() => {
        Prism.highlightAll()
    }, [])

    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">
                    Chapter 7: CLI Workflow Recipe (Artisan-style)
                </h1>
                <p className="text-xl text-gray-500">
                    Use one console entrypoint for migration, seeding, generators, and custom
                    project commands, similar to <code>php artisan ...</code>.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Step 0: Concept</h2>
                <ul>
                    <li>
                        CLI entry is <code>./console ...</code>.
                    </li>
                    <li>
                        Framework commands + project commands are merged by
                        <code>bootstrap::console::start_console</code>.
                    </li>
                </ul>

                <h2>Step 1: Discover Available Commands</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-bash">{`./console --help
./console migrate --help
./console db --help
./console make --help
./console realtime --help
./console route --help`}</code>
                </pre>

                <h2>Step 2: Core Daily Commands</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-bash">{`# migrations
./console migrate run
./console migrate info
./console migrate revert
./console migrate add create_article_publish_log

# seeding
./console db seed
./console db seed --name UserSeeder

# generator
./console make seeder DemoSeeder

# discovery
./console route list
./console realtime list`}</code>
                </pre>

                <h2>Step 3: Create a Custom CLI Command</h2>
                <h3>
                    File: <code>app/src/bin/main.rs</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use clap::{Args, Subcommand};

#[derive(Subcommand, Debug, Clone)]
pub enum ProjectCommands {
    #[command(subcommand)]
    Realtime(RealtimeCommands),
    #[command(subcommand)]
    Route(RouteCommands),
    Demo(DemoArgs),
}

#[derive(Args, Debug, Clone)]
pub struct DemoArgs {
    #[arg(long)]
    pub name: String,
}`}</code>
                </pre>

                <h2>Step 4: Handle the Custom Command</h2>
                <h3>
                    File: <code>app/src/bin/main.rs</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`#[async_trait::async_trait]
impl bootstrap::console::ProjectCommand for ProjectCommands {
    async fn handle(self, ctx: &BootContext) -> anyhow::Result<()> {
        match self {
            ProjectCommands::Realtime(RealtimeCommands::Bench(args)) => {
                run_realtime_bench(ctx, args).await
            }
            ProjectCommands::Route(RouteCommands::List(args)) => run_route_list(ctx, args).await,
            ProjectCommands::Demo(args) => run_demo(ctx, args).await,
        }
    }
}

async fn run_demo(ctx: &BootContext, args: DemoArgs) -> anyhow::Result<()> {
    println!("demo command: {} @ {}", args.name, ctx.settings.app.name);
    Ok(())
}`}</code>
                </pre>

                <h2>Step 5: Verify the Custom Command</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-bash">{`./console --help
./console demo --name codex`}</code>
                </pre>

                <h2>Step 6: Artisan-style Mapping</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-text">{`php artisan migrate            -> ./console migrate run
php artisan migrate:status     -> ./console migrate info
php artisan migrate:rollback   -> ./console migrate revert
php artisan db:seed            -> ./console db seed
php artisan make:seeder X      -> ./console make seeder X`}</code>
                </pre>

                <h2>Step 7: Realtime Bench (Project Command Example)</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-bash">{`./console realtime bench \
  --ws-url ws://127.0.0.1:3010/ws \
  --token <PAT_TOKEN> \
  --channel public_feed \
  --clients 100 \
  --messages 1000 \
  --publish-rate 500 \
  --ack \
  --json`}</code>
                </pre>
            </div>
        </div>
    )
}
