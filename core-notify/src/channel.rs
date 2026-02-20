use crate::{Mailable, Notifiable};
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait Channel: Send + Sync {
    /// The driver name (e.g. "mail", "database")
    fn name(&self) -> &'static str;
}

// --- Mail Channel ---

pub struct MailChannel;

impl MailChannel {
    /// Send immediately
    pub async fn dispatch_now(
        mailer: &core_mailer::Mailer,
        notifiable: &dyn Notifiable,
        mailable: &impl Mailable,
    ) -> Result<()> {
        if let Some(payload) = mailable.to_mail(notifiable) {
            mailer.send_raw(&payload).await?;
        }
        Ok(())
    }

    /// Dispatch to queue (simulated sweep)
    pub async fn dispatch(
        mailer: &core_mailer::Mailer,
        notifiable: &dyn Notifiable,
        mailable: &impl Mailable,
    ) -> Result<()> {
        if let Some(payload) = mailable.to_mail(notifiable) {
            mailer.queue_raw(payload).await?;
        }
        Ok(())
    }
}
