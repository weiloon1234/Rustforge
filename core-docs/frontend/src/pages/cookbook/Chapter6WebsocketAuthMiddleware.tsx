import { useEffect } from 'react'
import Prism from 'prismjs'

export function Chapter6WebsocketAuthMiddleware() {
    useEffect(() => {
        Prism.highlightAll()
    }, [])

    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">
                    Chapter 6: WebSocket Auth Guard Recipe (Bearer Token Flow)
                </h1>
                <p className="text-xl text-gray-500">
                    Apply guard-level auth and policy checks to channel subscribe/publish operations.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Objective</h2>
                <p>
                    Enforce realtime auth boundaries with the same guard/token model used by HTTP APIs.
                </p>

                <h2>Scaffold Now (verified)</h2>
                <ul>
                    <li>
                        Realtime server startup: <code>app/src/bin/websocket-server.rs</code>
                    </li>
                    <li>
                        Realtime module location: <code>app/src/internal/realtime/mod.rs</code>
                    </li>
                    <li>
                        Global auth switch in <code>.env</code>: <code>REALTIME_REQUIRE_AUTH=true</code>
                    </li>
                    <li>
                        Channel guard binding in <code>app/configs.toml</code>
                    </li>
                </ul>

                <h2>Concept Extension (optional)</h2>
                <ul>
                    <li>
                        Add explicit policy module under <code>app/src/internal/realtime/</code>{' '}
                        (example: <code>policy.rs</code>) and call it from <code>mod.rs</code>.
                    </li>
                    <li>
                        Evaluate abilities like <code>realtime:subscribe:*</code> and tenant scope inside policy.
                    </li>
                    <li>
                        Keep handler extraction/auth resolver in realtime runtime, not HTTP middleware chain.
                    </li>
                </ul>

                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-javascript">{`const ws = new WebSocket('ws://127.0.0.1:3010/ws');

ws.onopen = () => {
  ws.send(JSON.stringify({ op: 'auth', token: accessToken }));
  ws.send(JSON.stringify({ op: 'subscribe', channel: 'admin_notifications' }));
};`}</code>
                </pre>

                <h2>Verify</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-bash">{`./bin/websocket-server
# with valid token => auth_ok + subscribe success
# with invalid token => error frame (unauthorized/forbidden)`}</code>
                </pre>
            </div>
        </div>
    )
}
