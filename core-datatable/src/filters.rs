#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedFilter {
    Eq { column: String },
    Like { column: String },
    Gte { column: String },
    Lte { column: String },
    DateFrom { column: String },
    DateTo { column: String },
    LocaleEq { column: String },
    LocaleLike { column: String },
    LikeAny { columns: Vec<String> },
    Any { columns: Vec<String> },
    Has { relation: String, column: String },
    HasLike { relation: String, column: String },
    LocaleHas { relation: String, column: String },
    LocaleHasLike { relation: String, column: String },
}

pub fn parse_filter_key(key: &str) -> Option<ParsedFilter> {
    let key = key.trim();
    if !key.starts_with("f-") {
        return None;
    }

    if let Some(rest) = key.strip_prefix("f-like-any-") {
        return Some(ParsedFilter::LikeAny {
            columns: split_pipe(rest),
        });
    }

    if let Some(rest) = key.strip_prefix("f-any-") {
        return Some(ParsedFilter::Any {
            columns: split_pipe(rest),
        });
    }

    if let Some(rest) = key.strip_prefix("f-has-like-") {
        let (relation, column) = split_relation(rest)?;
        return Some(ParsedFilter::HasLike { relation, column });
    }

    if let Some(rest) = key.strip_prefix("f-has-") {
        let (relation, column) = split_relation(rest)?;
        return Some(ParsedFilter::Has { relation, column });
    }

    if let Some(rest) = key.strip_prefix("f-locale-has-like-") {
        let (relation, column) = split_relation(rest)?;
        return Some(ParsedFilter::LocaleHasLike { relation, column });
    }

    if let Some(rest) = key.strip_prefix("f-locale-has-") {
        let (relation, column) = split_relation(rest)?;
        return Some(ParsedFilter::LocaleHas { relation, column });
    }

    if let Some(column) = key.strip_prefix("f-like-") {
        return Some(ParsedFilter::Like {
            column: column.trim().to_string(),
        });
    }

    if let Some(column) = key.strip_prefix("f-gte-") {
        return Some(ParsedFilter::Gte {
            column: column.trim().to_string(),
        });
    }

    if let Some(column) = key.strip_prefix("f-lte-") {
        return Some(ParsedFilter::Lte {
            column: column.trim().to_string(),
        });
    }

    if let Some(column) = key.strip_prefix("f-date-from-") {
        return Some(ParsedFilter::DateFrom {
            column: column.trim().to_string(),
        });
    }

    if let Some(column) = key.strip_prefix("f-date-to-") {
        return Some(ParsedFilter::DateTo {
            column: column.trim().to_string(),
        });
    }

    if let Some(column) = key.strip_prefix("f-locale-like-") {
        return Some(ParsedFilter::LocaleLike {
            column: column.trim().to_string(),
        });
    }

    if let Some(column) = key.strip_prefix("f-locale-") {
        return Some(ParsedFilter::LocaleEq {
            column: column.trim().to_string(),
        });
    }

    key.strip_prefix("f-").map(|column| ParsedFilter::Eq {
        column: column.trim().to_string(),
    })
}

fn split_pipe(raw: &str) -> Vec<String> {
    raw.split('|')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn split_relation(raw: &str) -> Option<(String, String)> {
    let mut parts = raw.splitn(2, '-');
    let relation = parts.next()?.trim();
    let column = parts.next()?.trim();
    if relation.is_empty() || column.is_empty() {
        return None;
    }
    Some((relation.to_string(), column.to_string()))
}

#[cfg(test)]
mod tests {
    use super::{parse_filter_key, ParsedFilter};

    #[test]
    fn parse_filter_variants() {
        assert_eq!(
            parse_filter_key("f-title"),
            Some(ParsedFilter::Eq {
                column: "title".to_string()
            })
        );
        assert_eq!(
            parse_filter_key("f-like-title"),
            Some(ParsedFilter::Like {
                column: "title".to_string()
            })
        );
        assert_eq!(
            parse_filter_key("f-like-any-title|slug"),
            Some(ParsedFilter::LikeAny {
                columns: vec!["title".to_string(), "slug".to_string()]
            })
        );
        assert_eq!(
            parse_filter_key("f-has-comments-status"),
            Some(ParsedFilter::Has {
                relation: "comments".to_string(),
                column: "status".to_string()
            })
        );
        assert_eq!(
            parse_filter_key("f-locale-title"),
            Some(ParsedFilter::LocaleEq {
                column: "title".to_string()
            })
        );
        assert_eq!(
            parse_filter_key("f-locale-like-title"),
            Some(ParsedFilter::LocaleLike {
                column: "title".to_string()
            })
        );
        assert_eq!(
            parse_filter_key("f-locale-has-user__profile-display_name"),
            Some(ParsedFilter::LocaleHas {
                relation: "user__profile".to_string(),
                column: "display_name".to_string()
            })
        );
        assert_eq!(
            parse_filter_key("f-has-user__profile-display_name"),
            Some(ParsedFilter::Has {
                relation: "user__profile".to_string(),
                column: "display_name".to_string()
            })
        );
        assert!(parse_filter_key("foo").is_none());
    }
}
