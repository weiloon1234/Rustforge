pub fn has_permission<P>(granted: &[P], required: &str) -> bool
where
    P: AsRef<str>,
{
    granted
        .iter()
        .any(|permission| permission_matches(permission.as_ref(), required))
}

pub fn permission_matches(granted: &str, required: &str) -> bool {
    let granted = granted.trim();
    let required = required.trim();

    if granted.is_empty() || required.is_empty() {
        return false;
    }

    if granted == "*" || required == "*" || granted == required {
        return true;
    }

    // Convention: `<resource>.manage` implicitly grants `<resource>.read`.
    if manage_implies_read(granted, required) {
        return true;
    }

    match_pattern(granted, required) || match_pattern(required, granted)
}

fn match_pattern(pattern: &str, value: &str) -> bool {
    let Some(prefix) = pattern.strip_suffix(".*") else {
        return false;
    };

    let prefix = prefix.trim_end_matches('.');
    if prefix.is_empty() {
        return false;
    }

    value == prefix || value.starts_with(&format!("{prefix}."))
}

fn manage_implies_read(granted: &str, required: &str) -> bool {
    let Some((granted_scope, granted_action)) = granted.rsplit_once('.') else {
        return false;
    };
    let Some((required_scope, required_action)) = required.rsplit_once('.') else {
        return false;
    };

    granted_scope == required_scope && granted_action == "manage" && required_action == "read"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wildcard_matches_all() {
        assert!(permission_matches("*", "article.manage"));
    }

    #[test]
    fn namespace_wildcard_matches() {
        assert!(permission_matches("article.*", "article.manage"));
        assert!(!permission_matches("article.*", "category.manage"));
    }

    #[test]
    fn manage_implies_read_for_same_resource() {
        assert!(permission_matches("article.manage", "article.read"));
        assert!(!permission_matches("article.manage", "article.export"));
    }

    #[test]
    fn has_permission_checks_list() {
        let granted = vec!["article.read".to_string(), "article.*".to_string()];
        assert!(has_permission(&granted, "article.export"));
        assert!(!has_permission(&granted, "category.read"));
    }
}
