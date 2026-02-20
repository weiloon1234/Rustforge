use anyhow::{anyhow, Context, Result};
use time::UtcOffset;

#[derive(Debug, Clone)]
pub struct I18nSettings {
    pub default_locale: &'static str,
    pub supported_locales: &'static [&'static str],
    pub default_timezone: UtcOffset,
    pub default_timezone_str: String,
}

impl I18nSettings {
    pub fn parse_utc_offset(raw: &str) -> Result<UtcOffset> {
        if raw.len() != 6 {
            return Err(anyhow!("Timezone must be in ±HH:MM format"));
        }
        let sign = match raw.as_bytes()[0] {
            b'+' => 1,
            b'-' => -1,
            _ => return Err(anyhow!("Timezone must start with + or -")),
        };
        let hour: i8 = raw[1..3]
            .parse()
            .with_context(|| "Invalid hour in timezone")?;
        let minute: i8 = raw[4..6]
            .parse()
            .with_context(|| "Invalid minute in timezone")?;
        if hour > 23 || minute > 59 {
            return Err(anyhow!("Timezone hours must be <=23 and minutes <=59"));
        }
        UtcOffset::from_hms(sign * hour, sign * minute, 0)
            .map_err(|e| anyhow!("Invalid timezone offset: {e}"))
    }
}
