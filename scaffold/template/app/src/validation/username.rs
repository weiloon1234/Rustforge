use std::borrow::Cow;
use validator::ValidationError;

fn err(code: &'static str, msg: &'static str) -> ValidationError {
    ValidationError::new(code).with_message(Cow::Borrowed(msg))
}

pub fn validate_username(value: &str) -> Result<(), ValidationError> {
    let trimmed = value.trim();

    core_web::rules::required_trimmed(trimmed)
        .map_err(|_| err("required", "This field is required."))?;
    core_web::rules::alpha_dash(trimmed)
        .map_err(|_| err("alpha_dash", "Only lowercase letters, numbers, '-' and '_' are allowed."))?;

    if trimmed != trimmed.to_ascii_lowercase() {
        return Err(err(
            "lowercase",
            "Username must be lowercase.",
        ));
    }

    Ok(())
}
