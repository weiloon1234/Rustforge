export function Installation() {
    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">Installation</h1>
                <p className="text-xl text-gray-500">
                    Get your development environment ready in minutes.
                </p>
            </div>

            {/* Prerequisites */}
            <div className="space-y-4">
                <h2 className="text-2xl font-bold text-gray-900">1. Prerequisites</h2>
                <p className="text-gray-600">
                    Ensure you have the following services running locally:
                </p>
                <ul className="list-disc pl-5 space-y-2 text-gray-600">
                    <li>
                        <strong>Rust</strong> (Latest Stable):{' '}
                        <code className="bg-gray-100 px-1 py-0.5 rounded">
                            rustup update stable
                        </code>
                    </li>
                    <li>
                        <strong>PostgreSQL</strong>: Database (v14+)
                    </li>
                    <li>
                        <strong>Redis</strong>: Job Queue & Cache (v6+)
                    </li>
                    <li>
                        <strong>Node.js</strong>: Frontend assets (v18+)
                    </li>
                </ul>
            </div>

            {/* Infrastructure Setup */}
            <div className="space-y-4">
                <h2 className="text-2xl font-bold text-gray-900">
                    2. Setup Infrastructure
                </h2>
                <div className="bg-gray-900 rounded-lg p-4 font-mono text-sm text-gray-100 overflow-x-auto">
                    <span className="text-gray-500"># 1. Clone the repo</span>
                    <br />
                    git clone https://github.com/your-org/rs-core.git
                    <br />
                    cd rs-core
                    <br />
                    <br />
                    <span className="text-gray-500"># 2. Configure Environment</span>
                    <br />
                    cp .env.example .env
                    <br />
                    <span className="text-gray-500">
                        # Edit .env to match your local Postgres/Redis credentials
                    </span>
                    <br />
                    <br />
                    <span className="text-gray-500">
                        # 3. Install Dev Tools (cargo-watch, sqlx-cli)
                    </span>
                    <br />
                    make install-tools
                </div>
            </div>

            {/* Database Setup */}
            <div className="space-y-4">
                <h2 className="text-2xl font-bold text-gray-900">3. Database Setup</h2>
                <p className="text-gray-600">
                    Run migrations to initialize your database schema.
                </p>
                <div className="bg-gray-900 rounded-lg p-4 font-mono text-sm text-gray-100">
                    make migrate
                </div>
            </div>

            {/* Running */}
            <div className="space-y-4">
                <h2 className="text-2xl font-bold text-gray-900">4. Run the App</h2>
                <p className="text-gray-600">
                    Start the API server and Worker in development mode.
                </p>
                <div className="bg-gray-900 rounded-lg p-4 font-mono text-sm text-gray-100">
                    make dev
                </div>
                <div className="bg-blue-50 border-l-4 border-blue-400 p-4">
                    <div className="flex">
                        <div className="flex-shrink-0">
                            <svg
                                className="h-5 w-5 text-blue-400"
                                viewBox="0 0 20 20"
                                fill="currentColor"
                            >
                                <path
                                    fillRule="evenodd"
                                    d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z"
                                    clipRule="evenodd"
                                />
                            </svg>
                        </div>
                        <div className="ml-3">
                            <p className="text-sm text-blue-700">
                                The server will start at{' '}
                                <a
                                    href="http://127.0.0.1:3000"
                                    className="font-medium underline"
                                >
                                    http://127.0.0.1:3000
                                </a>{' '}
                                (or your configured port).
                            </p>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    )
}
