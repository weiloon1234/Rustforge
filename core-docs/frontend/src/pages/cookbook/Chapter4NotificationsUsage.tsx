import { useEffect } from 'react'
import Prism from 'prismjs'

export function Chapter4NotificationsUsage() {
    useEffect(() => {
        Prism.highlightAll()
    }, [])

    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">Recipe: Add Notifications</h1>
                <p className="text-xl text-gray-500">
                    Add a notification flow that stays aligned with workflow ownership, mail payload
                    rendering, and queued delivery boundaries.
                </p>
            </div>

            <div className="prose prose-orange max-w-none">
                <h2>Objective</h2>
                <p>
                    Start from a domain event, not from a transport. The workflow should persist
                    the domain change first, then trigger a mail notification immediately or via the
                    queue depending on the latency and durability requirements.
                </p>

                <h2>Step 1: Decide immediate vs queued delivery</h2>
                <p>
                    This choice belongs in the workflow. If delivery must survive retries and not
                    slow the request path, use <code>MailChannel::dispatch</code> and run the worker.
                    If the request can wait and delivery failure should surface immediately, use
                    <code>MailChannel::dispatch_now</code>.
                </p>

                <h2>Step 2: Implement the recipient route</h2>
                <p>
                    Implement <code>Notifiable</code> on the app-facing recipient type. In the
                    starter, a common place is a generated view extension module when the recipient
                    model already exists.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use core_notify::Notifiable;
use generated::models::admin::types::AdminView;

impl Notifiable for AdminView {
    fn route_notification_for(&self, driver: &str) -> Option<String> {
        match driver {
            "mail" => self.email.clone(),
            _ => None,
        }
    }

    fn id(&self) -> String {
        self.id.to_string()
    }
}`}</code>
                </pre>

                <h2>Step 3: Define the mail payload</h2>
                <p>
                    Keep the payload as a small domain type. It should know how to become a
                    <code>MailPayload</code> for a <code>Notifiable</code>, nothing more.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use core_mailer::MailPayload;
use core_notify::{Mailable, Notifiable};

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
}`}</code>
                </pre>

                <h2>Step 4: Trigger from the workflow</h2>
                <p>
                    Persist the domain change first. Then trigger the notification from the same
                    workflow so the rule stays in one place.
                </p>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-rust">{`use core_notify::channel::MailChannel;

pub async fn update_status(...) -> Result<CountryView, AppError> {
    let country = /* persist status change */;

    MailChannel::dispatch(
        state.mailer.as_ref(),
        &admin,
        &CountryStatusChangedMail {
            iso2: country.iso2.clone(),
            status: country.status.clone(),
        },
    ).await?;

    Ok(country)
}`}</code>
                </pre>

                <h2>Step 5: Add realtime only if the same event needs live fan-out</h2>
                <p>
                    Do not overload the mail notification type with realtime publish logic. If the
                    same workflow should notify connected clients, publish a separate realtime event
                    from the workflow or from a queued job.
                </p>

                <h2>Verification</h2>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto text-xs">
                    <code className="language-bash">{`cargo check -p app
./bin/worker
# trigger the workflow
# confirm mailer logs or SMTP delivery
# confirm failed_jobs stays empty for the happy path`}</code>
                </pre>

                <h2>Starter-local handoff</h2>
                <p>
                    If the recipient type needs computed fields or shared helper methods, continue
                    in the starter-local docs under <code>docs/README.md</code>, especially the
                    computed-model-values guide.
                </p>

                <h2>Cross-links</h2>
                <ul>
                    <li><a href="#/notifications">Notifications</a></li>
                    <li><a href="#/jobs">Job Queue</a></li>
                    <li><a href="#/cookbook/add-realtime-channel">Add a Realtime Channel</a></li>
                    <li><a href="#/cookbook/add-caching">Add Caching</a> if downstream reads should cache notification context.</li>
                </ul>
            </div>
        </div>
    )
}
