export function Introduction() {
    return (
        <div className="space-y-10">
            {/* Hero Section */}
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold tracking-tight text-gray-900 sm:text-5xl">
                    Welcome to{' '}
                    <span className="bg-clip-text text-transparent bg-gradient-to-r from-orange-600 to-amber-600">
                        Rustforge
                    </span>
                </h1>
                <p className="text-xl text-gray-500 max-w-3xl">
                    A production-ready Rust framework designed for building scalable,
                    maintainable, and type-safe web applications.
                </p>
            </div>

            {/* Philosophy Section */}
            <div className="prose prose-orange max-w-none">
                <h2>Philosophy</h2>
                <p>
                    Rustforge is built on the belief that{' '}
                    <strong>separation of concerns</strong> is the key to long-term
                    maintainability. It enforces a strict boundary between the reusable
                    framework infrastructure and your domain-specific business logic.
                </p>
            </div>

            {/* Architecture Table */}
            <div className="overflow-hidden shadow ring-1 ring-black ring-opacity-5 md:rounded-lg">
                <table className="min-w-full divide-y divide-gray-300">
                    <thead className="bg-gray-50">
                        <tr>
                            <th
                                scope="col"
                                className="py-3.5 pl-4 pr-3 text-left text-sm font-semibold text-gray-900 sm:pl-6"
                            >
                                Layer
                            </th>
                            <th
                                scope="col"
                                className="px-3 py-3.5 text-left text-sm font-semibold text-gray-900"
                            >
                                Components
                            </th>
                            <th
                                scope="col"
                                className="px-3 py-3.5 text-left text-sm font-semibold text-gray-900"
                            >
                                Role
                            </th>
                        </tr>
                    </thead>
                    <tbody className="divide-y divide-gray-200 bg-white">
                        <tr>
                            <td className="whitespace-nowrap py-4 pl-4 pr-3 text-sm font-medium text-gray-900 sm:pl-6">
                                Apps
                            </td>
                            <td className="px-3 py-4 text-sm text-gray-500">
                                <code className="bg-gray-100 px-1 py-0.5 rounded text-gray-800">
                                    api-server
                                </code>
                                ,{' '}
                                <code className="bg-gray-100 px-1 py-0.5 rounded text-gray-800">
                                    worker
                                </code>
                                ,{' '}
                                <code className="bg-gray-100 px-1 py-0.5 rounded text-gray-800">
                                    cli
                                </code>
                            </td>
                            <td className="px-3 py-4 text-sm text-gray-500">
                                <strong>Consumers</strong>: Implement specific business logic
                                (Users, Orders, etc).
                            </td>
                        </tr>
                        <tr>
                            <td className="whitespace-nowrap py-4 pl-4 pr-3 text-sm font-medium text-gray-900 sm:pl-6">
                                Framework
                            </td>
                            <td className="px-3 py-4 text-sm text-gray-500">
                                <code className="bg-gray-100 px-1 py-0.5 rounded text-gray-800">
                                    core-web
                                </code>
                                ,{' '}
                                <code className="bg-gray-100 px-1 py-0.5 rounded text-gray-800">
                                    core-db
                                </code>
                                ,{' '}
                                <code className="bg-gray-100 px-1 py-0.5 rounded text-gray-800">
                                    core-jobs
                                </code>
                            </td>
                            <td className="px-3 py-4 text-sm text-gray-500">
                                <strong>Providers</strong>: Reusable infrastructure, utilities,
                                and tooling.
                            </td>
                        </tr>
                    </tbody>
                </table>
            </div>

            {/* Key Features Grid */}
            <div>
                <h2 className="text-2xl font-bold text-gray-900 mb-6">Key Features</h2>
                <div className="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3">
                    {/* Feature 1 */}
                    <div className="rounded-lg border border-gray-200 bg-white p-6 shadow-sm hover:shadow-md transition-shadow">
                        <div className="flex items-center space-x-3 mb-3">
                            <div className="p-2 bg-orange-100 rounded-lg text-orange-600">
                                <svg
                                    className="w-6 h-6"
                                    fill="none"
                                    stroke="currentColor"
                                    viewBox="0 0 24 24"
                                >
                                    <path
                                        strokeLinecap="round"
                                        strokeLinejoin="round"
                                        strokeWidth={2}
                                        d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
                                    />
                                </svg>
                            </div>
                            <h3 className="font-semibold text-gray-900">Type-Safe APIs</h3>
                        </div>
                        <p className="text-sm text-gray-500">
                            Automatic OpenAPI generation via <code>aide</code>. Handlers are
                            strictly typed.
                        </p>
                    </div>

                    {/* Feature 2 */}
                    <div className="rounded-lg border border-gray-200 bg-white p-6 shadow-sm hover:shadow-md transition-shadow">
                        <div className="flex items-center space-x-3 mb-3">
                            <div className="p-2 bg-blue-100 rounded-lg text-blue-600">
                                <svg
                                    className="w-6 h-6"
                                    fill="none"
                                    stroke="currentColor"
                                    viewBox="0 0 24 24"
                                >
                                    <path
                                        strokeLinecap="round"
                                        strokeLinejoin="round"
                                        strokeWidth={2}
                                        d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4m0 5c0 2.21-3.582 4-8 4s-8-1.79-8-4"
                                    />
                                </svg>
                            </div>
                            <h3 className="font-semibold text-gray-900">ActiveRecord</h3>
                        </div>
                        <p className="text-sm text-gray-500">
                            Powerful <code>sqlx</code> wrapper. Define schemas in TOML, get
                            structs & queries for free.
                        </p>
                    </div>

                    {/* Feature 3 */}
                    <div className="rounded-lg border border-gray-200 bg-white p-6 shadow-sm hover:shadow-md transition-shadow">
                        <div className="flex items-center space-x-3 mb-3">
                            <div className="p-2 bg-purple-100 rounded-lg text-purple-600">
                                <svg
                                    className="w-6 h-6"
                                    fill="none"
                                    stroke="currentColor"
                                    viewBox="0 0 24 24"
                                >
                                    <path
                                        strokeLinecap="round"
                                        strokeLinejoin="round"
                                        strokeWidth={2}
                                        d="M19.428 15.428a2 2 0 00-1.022-.547l-2.384-.477a6 6 0 00-3.86.517l-.318.158a6 6 0 01-3.86.517L6.05 15.21a2 2 0 00-1.806.547M8 4h8l-1 1v5.172a2 2 0 00.586 1.414l5 5c1.26 1.26.367 3.414-1.415 3.414H4.828c-1.782 0-2.674-2.154-1.414-3.414l5-5A2 2 0 009 10.172V5L8 4z"
                                    />
                                </svg>
                            </div>
                            <h3 className="font-semibold text-gray-900">Job Queue</h3>
                        </div>
                        <p className="text-sm text-gray-500">
                            Redis-backed background processing with <code>core-jobs</code>.
                            Distributed & reliable.
                        </p>
                    </div>
                </div>
            </div>
        </div>
    )
}
