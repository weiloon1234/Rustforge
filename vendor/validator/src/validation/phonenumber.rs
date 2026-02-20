use std::borrow::Cow;

pub trait OptionalStr {
    fn as_optional_str(&self) -> Option<&str>;
}

impl OptionalStr for str {
    fn as_optional_str(&self) -> Option<&str> {
        Some(self)
    }
}

impl OptionalStr for String {
    fn as_optional_str(&self) -> Option<&str> {
        Some(self.as_str())
    }
}

impl OptionalStr for Cow<'_, str> {
    fn as_optional_str(&self) -> Option<&str> {
        Some(self.as_ref())
    }
}

impl<T> OptionalStr for &T
where
    T: OptionalStr + ?Sized,
{
    fn as_optional_str(&self) -> Option<&str> {
        T::as_optional_str(self)
    }
}

impl<T> OptionalStr for Option<T>
where
    T: OptionalStr,
{
    fn as_optional_str(&self) -> Option<&str> {
        self.as_ref().and_then(|value| value.as_optional_str())
    }
}

/// Validates phone number format against a sibling country ISO2 field.
///
/// Missing values are treated as valid to preserve validator's optional semantics.
/// Pair with `#[validate(required)]` or `#[validate(length(min = 1))]` when mandatory.
#[must_use]
pub fn validate_phonenumber_by_country_iso2<V, C>(value: &V, country_iso2: &C) -> bool
where
    V: OptionalStr + ?Sized,
    C: OptionalStr + ?Sized,
{
    let Some(phone_raw) = value.as_optional_str() else {
        return true;
    };
    let Some(country_raw) = country_iso2.as_optional_str() else {
        return true;
    };

    let phone = phone_raw.trim();
    if phone.is_empty() {
        return false;
    }

    let iso2 = country_raw.trim().to_ascii_uppercase();
    if iso2.len() != 2 {
        return false;
    }

    let region = match iso2.parse::<phonenumber::country::Id>() {
        Ok(region) => region,
        Err(_) => return false,
    };

    match phonenumber::parse(Some(region), phone) {
        Ok(parsed) => phonenumber::is_valid(&parsed),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::validate_phonenumber_by_country_iso2;

    #[test]
    fn validates_valid_us_numbers() {
        assert!(validate_phonenumber_by_country_iso2(
            &"2025550123",
            &"US"
        ));
        assert!(validate_phonenumber_by_country_iso2(
            &"+12025550123",
            &"US"
        ));
    }

    #[test]
    fn rejects_invalid_phone_or_country() {
        assert!(!validate_phonenumber_by_country_iso2(&"123", &"US"));
        assert!(!validate_phonenumber_by_country_iso2(
            &"2025550123",
            &"ZZ"
        ));
    }

    #[test]
    fn preserves_optional_semantics() {
        let missing_phone: Option<String> = None;
        let missing_country: Option<String> = None;
        assert!(validate_phonenumber_by_country_iso2(&missing_phone, &"US"));
        assert!(validate_phonenumber_by_country_iso2(
            &"2025550123",
            &missing_country
        ));
    }
}
