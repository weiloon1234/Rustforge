import { useEffect } from 'react'
import Prism from 'prismjs'

export function Chapter6WebsocketAuthMiddleware() {
    useEffect(() => {
        Prism.highlightAll()
    }, [])

    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">Recipe: Add WebSocket Auth</h1>
                <p className="text-xl text-gray-500">
                    Reuse the HTTP token identity model for websocket auth, subscribe policy, and
                    publish policy.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Objective</h2>
                <p>
                    Authenticate websocket connections with the same guard and ability model used by
                    HTTP. Do not create a second permission format just because the transport is a
                    socket.
                </p>

                <h2>Core runtime pieces</h2>
                <ul>
                    <li><code>AuthResolver</code>: turns a token into <code>AuthIdentity</code></li>
                    <li><code>PolicyContext</code>: the only input subscribe/publish policy should read</li>
                    <li><code>ChannelPolicy.guard</code>: channel-level guard requirement</li>
                    <li><code>REALTIME_REQUIRE_AUTH</code>: global runtime auth enforcement</li>
                </ul>

                <h2>Step 1: Resolve the same token identity used by HTTP</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use core_realtime::AuthResolver;
use core_web::auth::AuthIdentity;
use std::sync::Arc;

let auth_resolver: AuthResolver = Arc::new(|token: String| {
    Box::pin(async move {
        // resolve the token through the same guard/provider logic you trust for HTTP
        Some(AuthIdentity {
            subject_id: "admin-1".to_string(),
            guard: "admin".to_string(),
            abilities: vec!["country.manage".to_string()],
            token_id: None,
        })
    })
});`}</code>
                </pre>

                <h2>Step 2: Enforce guard + ability in policy</h2>
                <p>
                    Read <code>guard()</code>, <code>subject_id()</code>, <code>room()</code>, and
                    <code>has_ability()</code> from <code>PolicyContext</code>. That keeps the rule
                    transport-aware but permission-SSOT aligned.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`impl SubscribeAuthorizer for AdminSubscribeAuthorizer {
    fn authorize_subscribe(&self, context: &PolicyContext) -> PolicyDecision {
        if context.guard() != Some("admin") {
            return PolicyDecision::deny(ErrorCode::Unauthorized, "admin token required");
        }

        if !context.has_ability("country.read") {
            return PolicyDecision::deny(ErrorCode::Forbidden, "missing country.read");
        }

        if context.room() != Some("country:admin") {
            return PolicyDecision::deny(ErrorCode::Forbidden, "unexpected room");
        }

        PolicyDecision::allow()
    }
}`}</code>
                </pre>

                <h2>Step 3: Add publish policy only when needed</h2>
                <p>
                    Subscribe and publish are separate actions. If your app exposes an HTTP or
                    internal publish path that should be guarded differently, use
                    <code>PublishAuthorizer</code> via
                    <code>ChannelPolicyRegistry::with_publish_authorizer(...)</code>.
                </p>

                <h2>Step 4: Use the actual client auth flow</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-javascript">{`const ws = new WebSocket('ws://127.0.0.1:3010/ws');

ws.onopen = () => {
  ws.send(JSON.stringify({ op: 'auth', token: accessToken }));
  ws.send(JSON.stringify({
    op: 'subscribe',
    channel: 'admin_notifications',
    room: 'country:admin',
  }));
};`}</code>
                </pre>

                <h2>Verification</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-bash">{`./bin/websocket-server
# valid admin token + right ability => auth_ok + subscribe_ok
# wrong guard => unauthorized
# missing ability => forbidden
# wrong room => forbidden`}</code>
                </pre>

                <h2>Starter-local handoff</h2>
                <p>
                    If the generated starter needs project-specific room naming or additional guard
                    wiring, keep those conventions in the starter docs under <code>docs/README.md</code>
                    and the app-owned realtime module.
                </p>

                <h2>Cross-links</h2>
                <ul>
                    <li><a href="#/auth">Guards &amp; Auth</a></li>
                    <li><a href="#/permissions">Permissions &amp; AuthZ</a></li>
                    <li><a href="#/permission-matrix">Permission Matrix</a></li>
                    <li><a href="#/cookbook/add-realtime-channel">Add a Realtime Channel</a></li>
                </ul>
            </div>
        </div>
    )
}
