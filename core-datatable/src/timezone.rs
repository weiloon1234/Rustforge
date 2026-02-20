use anyhow::Result;
use serde_json::{Map, Value};
use time::{format_description::well_known::Rfc3339, OffsetDateTime, UtcOffset};

pub fn parse_utc_offset(raw: &str) -> Result<UtcOffset> {
    let raw = raw.trim();
    if raw.len() != 6 {
        anyhow::bail!("timezone must be in ±HH:MM format");
    }
    let sign = match raw.as_bytes()[0] {
        b'+' => 1,
        b'-' => -1,
        _ => anyhow::bail!("timezone must start with + or -"),
    };
    let hour: i8 = raw[1..3].parse()?;
    let minute: i8 = raw[4..6].parse()?;
    if hour > 23 || minute > 59 {
        anyhow::bail!("invalid timezone offset range");
    }
    Ok(UtcOffset::from_hms(sign * hour, sign * minute, 0)?)
}

pub fn apply_timezone_to_fields(
    record: &mut Map<String, Value>,
    timestamp_columns: &[String],
    timezone: &str,
) {
    let Ok(offset) = parse_utc_offset(timezone) else {
        return;
    };

    for column in timestamp_columns {
        if let Some(value) = record.get_mut(column) {
            let Some(raw) = value.as_str() else {
                continue;
            };
            let Ok(dt) = OffsetDateTime::parse(raw, &Rfc3339) else {
                continue;
            };
            let converted = dt.to_offset(offset);
            if let Ok(formatted) = converted.format(&Rfc3339) {
                *value = Value::String(formatted);
            }
        }
    }
}
