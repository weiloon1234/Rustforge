use core_i18n::t_args;
use generated::models::UserCreditTransactionView;

pub trait UserCreditTransactionViewExt {
    /// Enrich the generated `transaction_type_explained` in-place.
    ///
    /// Priority:
    /// 1. `custom_description == true` → use localized `custom_description_text`
    /// 2. `params` non-empty → re-translate with `t_args` to interpolate placeholders
    /// 3. otherwise → keep the generated `explained_label()` as-is
    fn enrich_transaction_type_explained(&mut self);
}

impl UserCreditTransactionViewExt for UserCreditTransactionView {
    fn enrich_transaction_type_explained(&mut self) {
        if self.custom_description {
            if let Some(ref text) = self.custom_description_text {
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    self.transaction_type_explained = trimmed.to_string();
                    return;
                }
            }
        }

        if let Some(serde_json::Value::Object(ref map)) = self.params {
            if !map.is_empty() {
                let args: Vec<(&str, String)> = map
                    .iter()
                    .map(|(k, v)| {
                        let s = match v {
                            serde_json::Value::String(s) => s.clone(),
                            other => other.to_string(),
                        };
                        (k.as_str(), s)
                    })
                    .collect();
                let refs: Vec<(&str, &str)> =
                    args.iter().map(|(k, v)| (*k, v.as_str())).collect();
                self.transaction_type_explained =
                    t_args(self.transaction_type.i18n_key(), &refs);
            }
        }
    }
}
