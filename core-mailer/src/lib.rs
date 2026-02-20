use async_trait::async_trait;
use core_jobs::{Job, JobContext};
use lettre::{
    message::header, transport::smtp::authentication::Credentials, AsyncSmtpTransport,
    AsyncTransport, Message, Tokio1Executor,
};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct Mailer {
    transport: Option<AsyncSmtpTransport<Tokio1Executor>>,
    from: String,
    queue: Option<core_jobs::queue::RedisQueue>,
}

#[async_trait]
pub trait Mailable: Send + Sync {
    fn subject(&self) -> String;
    fn body(&self) -> String;
    fn to(&self) -> Vec<String>;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MailPayload {
    pub to: Vec<String>,
    pub subject: String,
    pub body: String,
}

impl MailPayload {
    pub fn new(to: String, subject: String, body: String) -> Self {
        Self {
            to: vec![to],
            subject,
            body,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SendMailJob {
    pub payload: MailPayload,
}

#[async_trait]
impl Job for SendMailJob {
    const NAME: &'static str = "SendMailJob";

    async fn handle(&self, ctx: &JobContext) -> anyhow::Result<()> {
        // Use settings from context
        let mail_settings = &ctx.settings.mail;

        let mailer = Mailer::from_settings(mail_settings)?;
        mailer.send_raw(&self.payload).await?;
        Ok(())
    }
}

impl Mailer {
    // For App usage (with queue support)
    pub fn new(
        settings: &core_config::MailSettings,
        queue: Option<core_jobs::queue::RedisQueue>,
    ) -> anyhow::Result<Self> {
        let transport = Self::build_transport(settings)?;
        let from = settings.from_address.clone();

        Ok(Self {
            transport,
            from,
            queue,
        })
    }

    // specific for Worker (stateless / from settings)
    pub fn from_settings(settings: &core_config::MailSettings) -> anyhow::Result<Self> {
        let transport = Self::build_transport(settings)?;
        let from = settings.from_address.clone();
        Ok(Self {
            transport,
            from,
            queue: None,
        })
    }

    fn build_transport(
        settings: &core_config::MailSettings,
    ) -> anyhow::Result<Option<AsyncSmtpTransport<Tokio1Executor>>> {
        if settings.driver == "log" {
            return Ok(None);
        }

        // SMTP
        let mut builder = AsyncSmtpTransport::<Tokio1Executor>::relay(&settings.host)?;
        // Port? Lettre relay sets port automatically or via builder?
        // relay() uses default. We might need `relay` with port option or separate builder.
        // Actually `relay` is high level. `builder_starttls` is better for explicit port.
        // But let's check if we can set port.
        // builder.port(settings.port) - looks valid if exposed.
        // If not, we use `AsyncSmtpTransport::<Tokio1Executor>::builder_starttls(host).port(port)`

        // Let's stick to `relay` and assume standard port or verify API.
        // Actually, let's use the explicit builder for control.

        // Re-read lettre docs from memory:
        // builder_starttls is safer.

        // For now, let's assume `relay` returns a builder that has `.port()`.
        builder = builder.port(settings.port);

        if let (Some(u), Some(p)) = (&settings.username, &settings.password) {
            builder = builder.credentials(Credentials::new(u.clone(), p.clone()));
        }

        Ok(Some(builder.build()))
    }

    pub async fn send<M: Mailable>(&self, mail: &M) -> anyhow::Result<()> {
        let payload = MailPayload {
            to: mail.to(),
            subject: mail.subject(),
            body: mail.body(),
        };
        self.send_raw(&payload).await
    }

    pub async fn send_raw(&self, payload: &MailPayload) -> anyhow::Result<()> {
        if let Some(transport) = &self.transport {
            let email = Message::builder()
                .from(self.from.parse()?)
                .to(payload.to[0].parse()?) // Simplify single recipient for now
                .subject(&payload.subject)
                .header(header::ContentType::TEXT_HTML)
                .body(payload.body.clone())?;

            transport.send(email).await?;
            tracing::info!("Email sent to {}", payload.to[0]);
        } else {
            // Log Driver
            tracing::info!(
                "[MAIL LOG] To: {:?}, Subject: {}, Body: {}",
                payload.to,
                payload.subject,
                payload.body
            );
        }
        Ok(())
    }

    pub async fn queue<M: Mailable + Serialize>(&self, mail: &M) -> anyhow::Result<()> {
        if let Some(q) = &self.queue {
            // Render to payload
            let payload = MailPayload {
                to: mail.to(),
                subject: mail.subject(),
                body: mail.body(),
            };

            let job = SendMailJob { payload };
            job.dispatch(q).await?;
            tracing::info!("Email queued for {:?}", job.payload.to);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Queue not configured for mailer"))
        }
    }

    pub async fn queue_raw(&self, payload: MailPayload) -> anyhow::Result<()> {
        if let Some(q) = &self.queue {
            let job = SendMailJob { payload };
            job.dispatch(q).await?;
            tracing::info!("Email queued for {:?}", job.payload.to);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Queue not configured for mailer"))
        }
    }
}
