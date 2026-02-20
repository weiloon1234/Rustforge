/// Trait implemented by models (e.g., User) to provide routing info.
pub trait Notifiable: Send + Sync {
    /// Get route for a specific driver
    fn route_notification_for(&self, driver: &str) -> Option<String>;

    // Helpers
    fn email(&self) -> Option<String> {
        self.route_notification_for("mail")
    }
    fn phone(&self) -> Option<String> {
        self.route_notification_for("sms")
    }
    fn id(&self) -> String; // For database notifications
}
