use generated::models::WithdrawalWithRelations;

pub trait WithdrawalViewExt {
    /// Get the human-readable status label.
    fn status_label(&self) -> String;
}

impl WithdrawalViewExt for WithdrawalWithRelations {
    fn status_label(&self) -> String {
        self.status.explained_label().to_string()
    }
}
