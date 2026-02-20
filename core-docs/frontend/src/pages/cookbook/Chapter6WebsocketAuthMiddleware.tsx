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
                    Require websocket auth, bind channel to guard, and enforce ability checks in
                    app policy.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Step 0: Important Behavior</h2>
                <ul>
                    <li>
                        WebSocket uses protocol-level auth message: <code>{`{"op":"auth","token":"..."}`}</code>.
                    </li>
                    <li>
                        Token is the same PAT bearer token from your HTTP login endpoint.
                    </li>
                    <li>
                        Guard checks happen in realtime policy and channel config, not axum HTTP middleware.
                    </li>
                </ul>

                <h2>Step 1: Enable Realtime Auth</h2>
                <h3>
                    File: <code>.env</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-bash">{`REALTIME_ENABLED=true
REALTIME_REQUIRE_AUTH=true
REALTIME_DELIVERY_MODE=durable`}</code>
                </pre>

                <h2>Step 2: Bind Channel to Guard</h2>
                <h3>
                    File: <code>app/configs.toml</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-toml">{`[realtime.channels.admin_notifications]
enabled = true
guard = "admin"
presence_enabled = true
max_message_bytes = 32768
max_frame_bytes = 32768`}</code>
                </pre>

                <h2>Step 3: Ensure Auth Resolver Uses Generated Guards</h2>
                <h3>
                    File: <code>app/src/app.rs</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-sm">
                    <code className="language-rust">{`let db = ctx.db.clone();
let auth_resolver: AuthResolver = Arc::new(move |token: String| {
    let db = db.clone();
    Box::pin(async move {
        models::generated::guards::authenticate_any_guard(&db, &token).await
    })
});`}</code>
                </pre>

                <h2>Step 4: Add Ability Rules (Middleware-like Policy Layer)</h2>
                <h3>
                    File: <code>app/src/internal/realtime_policy.rs</code>
                </h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use core_i18n::t_args;

fn evaluate_policy(context: &PolicyContext) -> PolicyDecision {
    let Some(identity) = context.identity.as_ref() else {
        return PolicyDecision::allow();
    };

    let action = match context.action {
        PolicyAction::Subscribe => "subscribe",
        PolicyAction::Publish => "publish",
    };

    let scoped = format!("realtime:{action}:{}", context.channel());
    let wildcard = format!("realtime:{action}:*");

    let has_any_realtime_scope = identity.abilities.iter().any(|ability| {
        ability.eq_ignore_ascii_case("realtime:*")
            || ability.to_ascii_lowercase().starts_with("realtime:")
    });

    if has_any_realtime_scope
        && !has_ability(&identity.abilities, &scoped)
        && !has_ability(&identity.abilities, &wildcard)
        && !has_ability(&identity.abilities, "realtime:*")
    {
        return PolicyDecision::deny(
            ErrorCode::Forbidden,
            t_args("Missing ability :ability", &[("ability", &scoped)]),
        );
    }

    PolicyDecision::allow()
}`}</code>
                </pre>

                <h2>Step 5: Client Auth + Subscribe Flow</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-javascript">{`const ws = new WebSocket('ws://127.0.0.1:3010/ws');

ws.onopen = () => {
  ws.send(JSON.stringify({ op: 'auth', token: accessToken }));
  ws.send(JSON.stringify({
    op: 'subscribe',
    channel: 'admin_notifications',
    room: 'tenant:1'
  }));
};

ws.onmessage = (evt) => {
  const msg = JSON.parse(evt.data);
  if (msg.op === 'auth_ok') {
    console.log('authenticated', msg.identity);
  }
  if (msg.op === 'error') {
    console.error('realtime error', msg.code, msg.message);
  }
};`}</code>
                </pre>

                <h2>Step 6: Verify Good vs Bad Token</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-bash">{`# 1) obtain admin token from your admin login endpoint
curl -X POST http://127.0.0.1:3000/api/v1/admin/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"email":"admin@example.com","password":"super-secret-123"}'

# 2) use token in ws auth op
# expected: auth_ok then successful subscribe for admin_notifications

# 3) with invalid token
# expected: {"op":"error","code":"UNAUTHORIZED",...}`}</code>
                </pre>

                <h2>Decision Rule</h2>
                <ul>
                    <li>
                        Put global auth requirement in <code>.env</code> via{' '}
                        <code>REALTIME_REQUIRE_AUTH</code>.
                    </li>
                    <li>
                        Put coarse guard boundary in <code>app/configs.toml</code> channel guard.
                    </li>
                    <li>
                        Put fine-grained ability and tenant checks in{' '}
                        <code>app/src/internal/realtime_policy.rs</code>.
                    </li>
                </ul>
            </div>
        </div>
    )
}
