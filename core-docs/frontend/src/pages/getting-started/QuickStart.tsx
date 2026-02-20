export function QuickStart() {
    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">Quick Start</h1>
                <p className="text-xl text-gray-500">
                    Generate a starter app from Rustforge scaffold.
                </p>
            </div>

            <div className="space-y-4">
                <h2 className="text-2xl font-bold text-gray-900">1. Generate Starter</h2>
                <p className="text-gray-600">
                    Run scaffold from Rustforge root and choose your output directory.
                </p>
                <div className="bg-gray-900 rounded-lg p-4 font-mono text-sm text-gray-100">
                    cargo run --manifest-path scaffold/Cargo.toml -- --output /tmp/rustforge-starter --force
                </div>
            </div>

            <div className="space-y-4">
                <h2 className="text-2xl font-bold text-gray-900">2. Prepare Starter</h2>
                <div className="bg-gray-900 rounded-lg p-4 font-mono text-sm text-gray-100">
                    {`cd /tmp/rustforge-starter
cp .env.example .env
./bin/console migrate pump
./bin/console migrate run`}
                </div>
            </div>

            <div className="space-y-4">
                <h2 className="text-2xl font-bold text-gray-900">3. Run</h2>
                <div className="bg-gray-900 rounded-lg p-4 font-mono text-sm text-gray-100">
                    {`cargo check --workspace
./bin/api-server
./bin/console route list`}
                </div>
            </div>
        </div>
    )
}
