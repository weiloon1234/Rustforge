use std::borrow::Cow;
use validator::ValidationError;

fn err(code: &'static str, msg: &'static str) -> ValidationError {
    ValidationError::new(code).with_message(Cow::Borrowed(msg))
}

pub fn required_trimmed(value: &str) -> Result<(), ValidationError> {
    core_web::rules::required_trimmed(value).map_err(|_| err("required", "This field is required."))
}

pub fn alpha_dash(value: &str) -> Result<(), ValidationError> {
    core_web::rules::alpha_dash(value)
}
