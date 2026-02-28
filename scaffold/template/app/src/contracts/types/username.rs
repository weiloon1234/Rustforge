use core_web::contracts::rustforge_string_rule_type;

rustforge_string_rule_type! {
    /// Lowercase username used for admin authentication and admin CRUD inputs.
    pub struct UsernameString {
        #[validate(custom(function = "crate::validation::username::validate_username"))]
        #[rf(length(min = 3, max = 64))]
        #[rf(alpha_dash)]
        #[rf(openapi(description = "Lowercase username using letters, numbers, underscore (_), and hyphen (-).", example = "admin_user"))]
    }
}
