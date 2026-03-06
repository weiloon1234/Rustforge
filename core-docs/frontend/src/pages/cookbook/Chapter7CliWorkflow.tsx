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
                    Recipe: Add a Console Workflow
                </h1>
                <p className="text-xl text-gray-500">
                    Operate migrations, seeds, route introspection, and project commands from one console entrypoint.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Objective</h2>
                <p>Use one command surface for framework operations and project-specific tasks.</p>

                <h2>Scaffold Now (verified)</h2>
                <ul>
                    <li>
                        Entry binary: <code>app/src/bin/console.rs</code>
                    </li>
                    <li>
                        Project command enum baseline includes <code>Ping</code>
                    </li>
                    <li>
                        Seeder registration is wired through <code>register_seeders</code>
                    </li>
                </ul>

                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-bash">{`./console --help
./console migrate --help
./console db --help
./console route --help`}</code>
                </pre>

                <h2>Concept Extension (optional)</h2>
                <p>Add your own subcommands by extending <code>ProjectCommands</code> in <code>console.rs</code>.</p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`#[derive(clap::Subcommand, Debug, Clone)]
pub enum ProjectCommands {
    Ping,
    Reindex,
}

#[async_trait::async_trait]
impl bootstrap::console::ProjectCommand for ProjectCommands {
    async fn handle(self, _ctx: &bootstrap::boot::BootContext) -> anyhow::Result<()> {
        match self {
            ProjectCommands::Ping => println!("pong"),
            ProjectCommands::Reindex => println!("reindex queued"),
        }
        Ok(())
    }
}`}</code>
                </pre>

                <h2>Verify</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-bash">{`cargo check -p app
./console ping
./console route list --json`}</code>
                </pre>
            </div>
        </div>
    )
}
