use anyhow::{Context, Result};
use time::{format_description::well_known::Rfc3339, OffsetDateTime, UtcOffset};

/// Canonical datatable datetime wire format: RFC3339 normalized to UTC.
pub fn serialize_offset_datetime_rfc3339(value: OffsetDateTime) -> Result<String> {
    value
        .to_offset(UtcOffset::UTC)
        .format(&Rfc3339)
        .context("failed to format OffsetDateTime as RFC3339")
}

#[cfg(test)]
mod tests {
    use super::serialize_offset_datetime_rfc3339;
    use time::{format_description::well_known::Rfc3339, OffsetDateTime, UtcOffset};

    #[test]
    fn serializes_offset_datetime_as_rfc3339_utc() {
        let source = OffsetDateTime::from_unix_timestamp(1_775_733_312)
            .expect("valid unix timestamp")
            .to_offset(UtcOffset::from_hms(8, 0, 0).expect("valid offset"));

        let encoded =
            serialize_offset_datetime_rfc3339(source).expect("datetime should serialize");
        let parsed = OffsetDateTime::parse(&encoded, &Rfc3339).expect("must parse as RFC3339");

        assert_eq!(parsed.offset(), UtcOffset::UTC);
        assert_eq!(parsed.unix_timestamp(), source.unix_timestamp());
    }
}