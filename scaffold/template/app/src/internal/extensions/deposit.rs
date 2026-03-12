use generated::models::DepositWithRelations;

pub trait DepositViewExt {
    /// Get the human-readable status label.
    fn status_label(&self) -> String;
}

impl DepositViewExt for DepositWithRelations {
    fn status_label(&self) -> String {
        self.status.explained_label().to_string()
    }
}
