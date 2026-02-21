import { useEffect } from 'react'
import Prism from 'prismjs'

export function Cli() {
    useEffect(() => {
        Prism.highlightAll()
    }, [])

    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">CLI</h1>
                <p className="text-xl text-gray-500">
                    Framework + app commands through one console binary.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Usage</h2>
                <p>
                    Entry point is <code>app/src/bin/console.rs</code>. Use wrapper
                    <code> ./console</code> in starter root.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-bash">{`# help
./console --help

# framework db commands
./console migrate run
./console db seed
./console make seeder UserSeeder --dir app/src/seeds

# discover routes/realtime
./console route list
./console realtime list`}</code>
                </pre>

                <h2>Adding Project Commands</h2>
                <p>
                    Extend <code>ProjectCommands</code> in <code>app/src/bin/console.rs</code>.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`#[derive(Subcommand, Debug, Clone)]
pub enum ProjectCommands {
    Ping,
}

#[async_trait::async_trait]
impl bootstrap::console::ProjectCommand for ProjectCommands {
    async fn handle(self, _ctx: &BootContext) -> anyhow::Result<()> {
        match self {
            ProjectCommands::Ping => println!("pong"),
        }
        Ok(())
    }
}`}</code>
                </pre>
            </div>
        </div>
    )
}
