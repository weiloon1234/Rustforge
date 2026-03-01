import { useEffect } from 'react'
import Prism from 'prismjs'

export function Chapter5WebsocketChannel() {
    useEffect(() => {
        Prism.highlightAll()
    }, [])

    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">
                    Chapter 5: WebSocket Server + Channel Recipe
                </h1>
                <p className="text-xl text-gray-500">
                    Configure channels and publish events through the realtime runtime.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Objective</h2>
                <p>
                    Run realtime server from scaffold baseline and define channel policy using settings + internal module.
                </p>

                <h2>Scaffold Now (verified)</h2>
                <ul>
                    <li>
                        Binary: <code>app/src/bin/websocket-server.rs</code>
                    </li>
                    <li>
                        Realtime module placeholder: <code>app/src/internal/realtime/mod.rs</code>
                    </li>
                    <li>
                        Channel config source: <code>app/configs.toml</code>
                    </li>
                    <li>
                        Runtime toggles in <code>.env</code> (<code>REALTIME_*</code>)
                    </li>
                </ul>

                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-bash">{`./bin/websocket-server
# endpoint: ws://127.0.0.1:3010/ws`}</code>
                </pre>

                <h2>Concept Extension (optional)</h2>
                <ul>
                    <li>
                        Add application-specific channel router/policy in{' '}
                        <code>app/src/internal/realtime/mod.rs</code>.
                    </li>
                    <li>
                        Add an HTTP publish endpoint only if you need dev smoke or internal tooling.
                    </li>
                    <li>
                        Otherwise publish from workflow/job using <code>core_realtime::RealtimePublisher</code>.
                    </li>
                </ul>

                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use core_realtime::{RealtimeEvent, RealtimePublisher, RealtimeTarget};

#[derive(serde::Serialize)]
struct AdminChanged { id: i64 }

impl RealtimeEvent for AdminChanged {
    const CHANNEL: &'static str = "admin_notifications";
    const EVENT: &'static str = "updated";
}`}</code>
                </pre>

                <h2>Verify</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-bash">{`./bin/websocket-server
# then connect a native WS client and subscribe to your configured channel
# publish event from workflow/job and confirm event frame is received`}</code>
                </pre>
            </div>
        </div>
    )
}
