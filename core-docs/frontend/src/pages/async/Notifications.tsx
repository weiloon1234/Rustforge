import { useEffect } from 'react'
import Prism from 'prismjs'

export function Notifications() {
    useEffect(() => {
        Prism.highlightAll()
    }, [])

    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">Notifications</h1>
                <p className="text-xl text-gray-500">
                    Workflow-owned delivery decisions built on top of <code>Notifiable</code>,
                    <code> Mailable</code>, <code>MailChannel</code>, and jobs.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>What the feature is for</h2>
                <p>
                    Use notifications when domain state changes need delivery outside the immediate
                    response body. The framework keeps the notification layer intentionally small:
                    recipient routing, payload rendering, and delivery dispatch. App code decides
                    when a notification should happen and whether it belongs in-request or on the
                    queue.
                </p>

                <h2>Where the SSOT lives</h2>
                <ul>
                    <li>
                        <code>core-notify</code>: <code>Notifiable</code>, <code>Mailable</code>,
                        and <code>MailChannel</code>
                    </li>
                    <li>
                        <code>core-mailer</code>: <code>Mailer</code>, <code>MailPayload</code>,
                        and queued mail job dispatch
                    </li>
                    <li>
                        <code>core-jobs</code>: queue durability, worker execution, and retry/
                        failed-job behavior
                    </li>
                    <li>
                        app workflows: the actual decision that a notification should be sent
                    </li>
                </ul>

                <h2>Main runtime surface</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`pub trait Notifiable: Send + Sync {
    fn route_notification_for(&self, driver: &str) -> Option<String>;
    fn id(&self) -> String;
}

pub trait Mailable: Send + Sync {
    fn to_mail(&self, notifiable: &dyn Notifiable) -> Option<core_mailer::MailPayload>;
}

MailChannel::dispatch_now(mailer, &recipient, &notification).await?;
MailChannel::dispatch(mailer, &recipient, &notification).await?;`}</code>
                </pre>

                <h2>Responsibility split</h2>
                <table>
                    <thead>
                        <tr>
                            <th>Layer</th>
                            <th>Owns</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr>
                            <td>Workflow</td>
                            <td>Whether a notification should happen at all.</td>
                        </tr>
                        <tr>
                            <td>Notifiable</td>
                            <td>How a recipient exposes mail/SMS/etc. routes.</td>
                        </tr>
                        <tr>
                            <td>Mailable</td>
                            <td>The rendered mail payload for one recipient.</td>
                        </tr>
                        <tr>
                            <td>MailChannel</td>
                            <td>Immediate send vs queued send.</td>
                        </tr>
                        <tr>
                            <td>Jobs</td>
                            <td>Durability, retries, and failed-job handling.</td>
                        </tr>
                    </tbody>
                </table>

                <h2>Immediate vs queued delivery</h2>
                <table>
                    <thead>
                        <tr>
                            <th>Use <code>dispatch_now</code> when</th>
                            <th>Use <code>dispatch</code> when</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr>
                            <td>The request can wait for delivery.</td>
                            <td>The response should stay fast.</td>
                        </tr>
                        <tr>
                            <td>You want immediate failure in the request path.</td>
                            <td>You want worker-owned retries and failed-job visibility.</td>
                        </tr>
                        <tr>
                            <td>The notification is cheap and rare.</td>
                            <td>The notification is slow, bursty, or operationally important.</td>
                        </tr>
                    </tbody>
                </table>

                <h2>Minimal usage example</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use core_mailer::MailPayload;
use core_notify::{channel::MailChannel, Mailable, Notifiable};

pub struct CountryStatusChangedMail {
    pub iso2: String,
    pub status: String,
}

impl Mailable for CountryStatusChangedMail {
    fn to_mail(&self, notifiable: &dyn Notifiable) -> Option<MailPayload> {
        Some(MailPayload {
            to: vec![notifiable.email()?],
            subject: format!("Country {} updated", self.iso2),
            body: format!("<p>Status: {}</p>", self.status),
        })
    }
}

MailChannel::dispatch(state.mailer.as_ref(), &admin, &CountryStatusChangedMail {
    iso2: country.iso2.clone(),
    status: country.status.clone(),
}).await?;`}</code>
                </pre>

                <h2>Extension points</h2>
                <ul>
                    <li>
                        Implement <code>Notifiable</code> on an app-facing view type such as
                        <code> AdminView</code> or <code>UserView</code>.
                    </li>
                    <li>
                        Keep mail payload structs close to the workflow or domain module that owns
                        the event.
                    </li>
                    <li>
                        Add app-specific channels only when the delivery path genuinely differs.
                        Do not duplicate mail semantics just to wrap another function.
                    </li>
                    <li>
                        Pair notifications with realtime publish only when the same domain event
                        must reach connected clients live.
                    </li>
                </ul>

                <h2>Practical rule</h2>
                <p>
                    Keep handlers thin. Workflows decide that delivery should happen; notification
                    types decide what gets sent; jobs decide when a slow or durable notification
                    runs.
                </p>

                <h2>Cross-links</h2>
                <ul>
                    <li><a href="#/cookbook/add-notifications">Add Notifications</a></li>
                    <li><a href="#/jobs">Job Queue</a></li>
                    <li><a href="#/feature-realtime">Realtime / WebSocket</a></li>
                    <li><a href="#/cookbook/add-realtime-channel">Add a Realtime Channel</a></li>
                </ul>
            </div>
        </div>
    )
}
