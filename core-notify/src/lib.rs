// use async_trait::async_trait;

pub mod channel;
// pub mod manager;
pub mod notifiable;
// pub use manager::ChannelManager;

pub use channel::Channel;
pub use notifiable::Notifiable;

/// Trait for objects that can be sent via Email
pub trait Mailable: Send + Sync {
    fn to_mail(&self, notifiable: &dyn Notifiable) -> Option<core_mailer::MailPayload>;
}
