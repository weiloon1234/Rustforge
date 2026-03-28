use crate::traits::DataTableColumnResolver;
use core_db::common::model_api::QueryState;
use core_db::common::sql::{BindValue, Op};

/// Apply standard (non-relation) filters generically using [`QueryState`]
/// string-based methods.
///
/// Returns `Ok(Some(query))` when the filter was applied, `Ok(None)` when
/// the column was not recognised (caller should fall through to model-specific
/// handling).
///
/// Relation filters (`Has`, `HasLike`, `LocaleHas`, `LocaleHasLike`) are
/// **not** handled here — those remain per-model because they require typed
/// `where_has` calls.
pub fn apply_standard_filter<'db>(
    query_state: QueryState<'db>,
    filter: &ParsedFilter,
    value: &str,
    resolver: &dyn DataTableColumnResolver,
) -> anyhow::Result<Option<QueryState<'db>>> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(Some(query_state));
    }

    match filter {
        // ── single-column scalar filters ────────────────────────────
        ParsedFilter::Eq { column } | ParsedFilter::Gte { column } | ParsedFilter::Lte { column } => {
            let Some(col_sql) = resolver.resolve_col_sql(column) else {
                return Ok(None);
            };
            let Some(bind) = resolver.parse_bind_for_col(column, trimmed) else {
                return Ok(None);
            };
            let op = match filter {
                ParsedFilter::Eq { .. } => Op::Eq,
                ParsedFilter::Gte { .. } => Op::Ge,
                ParsedFilter::Lte { .. } => Op::Le,
                _ => unreachable!(),
            };
            Ok(Some(query_state.where_col_str(col_sql, op, bind)))
        }
        ParsedFilter::Like { column } => {
            let Some(col_sql) = resolver.resolve_like_col_sql(column) else {
                return Ok(None);
            };
            Ok(Some(query_state.where_col_str(
                col_sql,
                Op::Like,
                BindValue::String(format!("%{trimmed}%")),
            )))
        }
        ParsedFilter::DateFrom { column } | ParsedFilter::DateTo { column } => {
            let Some(col_sql) = resolver.resolve_col_sql(column) else {
                return Ok(None);
            };
            let end_of_day = matches!(filter, ParsedFilter::DateTo { .. });
            let Some(dt) = resolver.parse_datetime(trimmed, end_of_day) else {
                // Column exists but value is not a valid date — skip filter silently
                // (matches legacy behavior where invalid dates are treated as no-op)
                return Ok(Some(query_state));
            };
            let op = if end_of_day { Op::Le } else { Op::Ge };
            Ok(Some(
                query_state.where_col_str(col_sql, op, BindValue::from(dt)),
            ))
        }

        // ── locale filters (own model only) ─────────────────────────
        ParsedFilter::LocaleEq { column } | ParsedFilter::LocaleLike { column } => {
            let Some(field) = resolver.resolve_locale_field(column) else {
                return Ok(None);
            };
            let Some(owner_type) = resolver.locale_owner_type() else {
                return Ok(None);
            };
            let table = resolver.table_name();
            let pk = resolver.pk_column();
            let locale = core_i18n::current_locale().to_string();
            let is_like = matches!(filter, ParsedFilter::LocaleLike { .. });
            let comparator = if is_like { "LIKE" } else { "=" };
            let match_value = if is_like {
                format!("%{trimmed}%")
            } else {
                trimmed.to_string()
            };
            let clause = format!(
                "EXISTS (SELECT 1 FROM localized l WHERE l.owner_type = ? AND l.owner_id = {table}.{pk} AND l.field = ? AND l.locale = ? AND l.value {comparator} ?)"
            );
            Ok(Some(query_state.where_exists_raw(
                clause,
                vec![
                    BindValue::String(owner_type.to_string()),
                    BindValue::String(field.to_string()),
                    BindValue::String(locale),
                    BindValue::String(match_value),
                ],
            )))
        }

        // ── multi-column OR filters ─────────────────────────────────
        ParsedFilter::LikeAny { columns } => {
            let pattern = format!("%{trimmed}%");
            let mut applied = false;
            let next = query_state.where_group(|group| {
                let mut q = group;
                for col_name in columns {
                    if let Some(col_sql) = resolver.resolve_like_col_sql(col_name) {
                        let bind = BindValue::String(pattern.clone());
                        if applied {
                            q = q.or_where_col_str(col_sql, Op::Like, bind);
                        } else {
                            q = q.where_col_str(col_sql, Op::Like, bind);
                            applied = true;
                        }
                    }
                }
                q
            });
            if applied {
                Ok(Some(next))
            } else {
                Ok(None)
            }
        }
        ParsedFilter::Any { columns } => {
            let mut applied = false;
            let next = query_state.where_group(|group| {
                let mut q = group;
                for col_name in columns {
                    if let Some(col_sql) = resolver.resolve_col_sql(col_name) {
                        if let Some(bind) = resolver.parse_bind_for_col(col_name, trimmed) {
                            if applied {
                                q = q.or_where_col_str(col_sql, Op::Eq, bind);
                            } else {
                                q = q.where_col_str(col_sql, Op::Eq, bind);
                                applied = true;
                            }
                        }
                    }
                }
                q
            });
            if applied {
                Ok(Some(next))
            } else {
                Ok(None)
            }
        }

        // ── relation filters — handled per-model ────────────────────
        ParsedFilter::Has { .. }
        | ParsedFilter::HasLike { .. }
        | ParsedFilter::LocaleHas { .. }
        | ParsedFilter::LocaleHasLike { .. } => Ok(None),
    }
}

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
