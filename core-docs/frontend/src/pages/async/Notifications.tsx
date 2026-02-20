export function Notifications() {
    return (
        <div className="space-y-10">
            <div className="space-y-4">
                <h1 className="text-4xl font-extrabold text-gray-900">Notifications</h1>
                <p className="text-xl text-gray-500">Multi-channel messaging system.</p>
            </div>

            <div className="prose prose-orange max-w-none">
                <p>
                    Notifications are defined by combining <code>Notifiable</code> +
                    <code>Mailable</code>, then dispatched through channels. Mail can be sent now
                    or queued through jobs.
                </p>

                <h3>Define a mailable notification</h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`use core_notify::{Mailable, Notifiable};
use core_mailer::MailPayload;

pub struct WelcomeMail;

impl Mailable for WelcomeMail {
    fn to_mail(&self, user: &dyn Notifiable) -> Option<MailPayload> {
        Some(MailPayload {
            to: vec![user.email()?],
            subject: "Welcome".to_string(),
            body: "<p>Thanks for signing up</p>".to_string(),
        })
    }
}`}</code>
                </pre>

                <h3>Dispatch via channel</h3>
                <pre className="bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto">
                    <code className="language-rust">{`use core_notify::channel::MailChannel;

// Immediate send
MailChannel::dispatch_now(ctx.mailer.as_ref(), &user, &WelcomeMail).await?;

// Queued send (uses core-jobs internally)
MailChannel::dispatch(ctx.mailer.as_ref(), &user, &WelcomeMail).await?;`}</code>
                </pre>

                <h3>Channels</h3>
                <p>
                    `MailChannel` is built in. Additional channels (SMS, push, webhook) can be
                    added in app layer by implementing <code>core_notify::Channel</code> and
                    composing with jobs for async delivery.
                </p>
            </div>
        </div>
    )
}
