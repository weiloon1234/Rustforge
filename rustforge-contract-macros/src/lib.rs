use std::collections::BTreeMap;

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote, ToTokens};
use rustforge_contract_meta::{builtin_rule_meta, render_template, BuiltinRuleArgs, BuiltinRuleKind};
use syn::{
    parse::Parser, parse_macro_input, punctuated::Punctuated, Attribute, Expr, Field, Fields,
    Ident, ItemStruct, Lit, LitBool, LitFloat, LitStr, Meta, MetaList, MetaNameValue, Path, Token,
};

#[proc_macro_attribute]
pub fn rustforge_contract(_attr: TokenStream, item: TokenStream) -> TokenStream {
    match expand_rustforge_contract(parse_macro_input!(item as ItemStruct)) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn expand_rustforge_contract(mut item: ItemStruct) -> syn::Result<TokenStream2> {
    if !item.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(
            &item.generics,
            "#[rustforge_contract] currently supports non-generic structs only",
        ));
    }

    let named_fields = match &mut item.fields {
        Fields::Named(named) => &mut named.named,
        _ => {
            return Err(syn::Error::new_spanned(
                &item,
                "#[rustforge_contract] currently supports named-field structs only",
            ))
        }
    };

    let struct_ident = item.ident.clone();
    let vis = item.vis.clone();
    let shadow_ident = format_ident!("__RustforgeContractSchema_{}", struct_ident);
    let (container_rf_cfg, item_attrs_without_rf) = parse_rf_container_attrs(&item.attrs)?;

    let auto_inject_set = ["Debug", "Clone", "Deserialize", "Validate", "JsonSchema"];
    let mut user_extra_derives: Vec<Meta> = Vec::new();
    let mut container_attrs_for_shadow: Vec<Attribute> = Vec::new();
    let mut original_container_attrs: Vec<Attribute> = Vec::new();
    for attr in item_attrs_without_rf.iter() {
        if attr.path().is_ident("rustforge_contract") {
            continue;
        }
        if attr.path().is_ident("derive") {
            let metas =
                attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
            for m in metas {
                if let Meta::Path(p) = &m {
                    let ident = p
                        .segments
                        .last()
                        .map(|s| s.ident.to_string())
                        .unwrap_or_default();
                    if !auto_inject_set.contains(&ident.as_str()) {
                        user_extra_derives.push(m);
                    }
                }
            }
            continue;
        }
        if attr.path().is_ident("validate") {
            original_container_attrs.push(attr.clone());
            continue;
        }
        if attr.path().is_ident("schemars") {
            container_attrs_for_shadow.push(attr.clone());
            continue;
        }
        if attr.path().is_ident("ts") {
            original_container_attrs.push(attr.clone());
            continue;
        }
        original_container_attrs.push(attr.clone());
        container_attrs_for_shadow.push(attr.clone());
    }
    for schema_rule in &container_rf_cfg.schema_rules {
        original_container_attrs.push(build_validate_schema_attr(schema_rule)?);
    }
    let extra_derives = if user_extra_derives.is_empty() {
        quote! {}
    } else {
        quote! { , #(#user_extra_derives),* }
    };
    item.attrs = original_container_attrs;

    let mut original_fields_tokens = Vec::new();
    let mut shadow_fields_tokens = Vec::new();
    let mut patch_blocks = Vec::new();
    let mut helper_fns = Vec::new();
    let mut async_validation_blocks_all = Vec::<TokenStream2>::new();

    for (idx, field) in named_fields.iter().enumerate() {
        let field_ident = field.ident.clone().expect("named field");
        let field_ty = field.ty.clone();
        let field_vis = field.vis.clone();
        let field_kind = classify_field_kind(&field_ty);

        let (rf_cfg, non_rf_attrs, rf_attrs_present) = parse_rf_field(field)?;

        let mut original_field_attrs: Vec<Attribute> = Vec::new();
        let mut shadow_field_attrs: Vec<Attribute> = Vec::new();
        for attr in non_rf_attrs {
            if attr.path().is_ident("schemars") {
                shadow_field_attrs.push(attr);
            } else if attr.path().is_ident("ts") {
                original_field_attrs.push(attr);
            } else {
                original_field_attrs.push(attr.clone());
                shadow_field_attrs.push(attr);
            }
        }

        let mut generated_validate_attrs = Vec::new();
        let mut generated_shadow_schemars_attrs = Vec::new();
        let mut field_rule_extensions = Vec::<RuleExtensionSpec>::new();
        let mut field_desc_parts = Vec::<String>::new();
        let mut field_pattern_patch: Option<String> = None;
        let mut field_enum_values_patch: Option<Vec<String>> = None;
        let mut async_validate_blocks = Vec::<TokenStream2>::new();
        if let Some(length) = &rf_cfg.length {
            let (msg, code) = rf_cfg.message_code_for("length");
            generated_validate_attrs.push(build_validate_length_attr(length, &msg, &code)?);
            field_rule_extensions.push(RuleExtensionSpec::from_length(length));
            field_desc_parts.push(length.openapi_hint());
        }

        if let Some(range) = &rf_cfg.range {
            let (msg, code) = rf_cfg.message_code_for("range");
            generated_validate_attrs.push(build_validate_range_attr(range, &msg, &code)?);
            field_rule_extensions.push(RuleExtensionSpec::from_range(range));
            field_desc_parts.push(range.openapi_hint());
        }

        if rf_cfg.email {
            let (msg, code) = rf_cfg.message_code_for("email");
            generated_validate_attrs.push(build_validate_simple_attr("email", &msg, &code)?);
            field_rule_extensions.push(RuleExtensionSpec::simple_validator("email"));
            field_desc_parts.push("Must be a valid email address.".to_string());
        }

        if rf_cfg.url {
            let (msg, code) = rf_cfg.message_code_for("url");
            generated_validate_attrs.push(build_validate_simple_attr("url", &msg, &code)?);
            field_rule_extensions.push(RuleExtensionSpec::simple_validator("url"));
            field_desc_parts.push("Must be a valid URL.".to_string());
        }

        if rf_cfg.required {
            let (msg, code) = rf_cfg.message_code_for("required");
            generated_validate_attrs.push(build_validate_simple_attr("required", &msg, &code)?);
            field_rule_extensions.push(RuleExtensionSpec::simple_validator("required"));
            field_desc_parts.push("This field is required.".to_string());
        }

        if rf_cfg.nested {
            generated_validate_attrs.push(build_validate_nested_attr()?);
            field_rule_extensions.push(RuleExtensionSpec::simple_validator("nested"));
        }

        if let Some(other) = &rf_cfg.must_match_other {
            let (msg, code) = rf_cfg.message_code_for("must_match");
            generated_validate_attrs.push(build_validate_must_match_attr(other, &msg, &code)?);
            field_rule_extensions.push(RuleExtensionSpec::must_match(other));
            field_desc_parts.push(format!("Must match `{other}`."));
        }

        if let Some(pattern) = &rf_cfg.regex_pattern {
            ensure_string_like(field_kind, &field_ident, "regex")?;
            let helper_ident = format_ident!(
                "__rf_contract_{}_{}_regex_{}",
                struct_ident.to_string().to_lowercase(),
                field_ident,
                idx
            );
            helper_fns.push(generate_regex_wrapper_fn(&helper_ident, pattern));
            let (msg, code) = rf_cfg.message_code_for("regex");
            generated_validate_attrs.push(build_validate_custom_fn_attr(
                &helper_ident,
                &msg,
                &code,
            )?);
            generated_shadow_schemars_attrs.push(build_schemars_regex_attr(pattern)?);
            field_rule_extensions.push(RuleExtensionSpec::regex(pattern));
            field_desc_parts.push(format!("Must match pattern `{}`.", pattern));
            field_pattern_patch = Some(pattern.clone());
        }

        if let Some(pattern) = &rf_cfg.contains_pattern {
            ensure_string_like(field_kind, &field_ident, "contains")?;
            let (msg, code) = rf_cfg.message_code_for("contains");
            generated_validate_attrs.push(build_validate_contains_attr(pattern, &msg, &code)?);
            field_rule_extensions.push(RuleExtensionSpec::contains(pattern));
            field_desc_parts.push(format!("Must contain `{pattern}`."));
        }

        if let Some(pattern) = &rf_cfg.does_not_contain_pattern {
            ensure_string_like(field_kind, &field_ident, "does_not_contain")?;
            let (msg, code) = rf_cfg.message_code_for("does_not_contain");
            generated_validate_attrs.push(build_validate_does_not_contain_attr(pattern, &msg, &code)?);
            field_rule_extensions.push(RuleExtensionSpec::does_not_contain(pattern));
            field_desc_parts.push(format!("Must not contain `{pattern}`."));
        }

        for (rule_i, builtin) in rf_cfg.builtin_rules.iter().enumerate() {
            let meta = builtin_rule_meta(&builtin.key).ok_or_else(|| {
                syn::Error::new_spanned(
                    rf_attrs_present
                        .first()
                        .cloned()
                        .unwrap_or_else(|| field_ident.clone().into_token_stream()),
                    format!("Unknown Rustforge built-in rule `{}`", builtin.key),
                )
            })?;

            let mut params_for_desc = Vec::<(&str, String)>::new();
            if let Some(format) = &builtin.format {
                params_for_desc.push(("format", format.clone()));
            }
            if let Some(field_ref) = &builtin.field {
                params_for_desc.push(("field", field_ref.clone()));
            }
            if !builtin.values.is_empty() {
                params_for_desc.push(("values", builtin.values.join(", ")));
            }

            let default_desc = render_template(meta.openapi_description_template, &params_for_desc);
            field_desc_parts.push(default_desc.clone());

            if let Some(pattern) = meta.pattern {
                field_pattern_patch.get_or_insert_with(|| pattern.to_string());
            }

            if let Some(fmt) = meta.format {
                if rf_cfg.openapi_format.is_none() {
                    // only default if user did not override
                }
                if builtin.key == "phonenumber" {
                    // no schema format attr helper; patched later via format patch
                } else {
                    let _ = fmt;
                }
            }

            match meta.kind {
                BuiltinRuleKind::CustomFnPath(path_str) => {
                    ensure_string_like(field_kind, &field_ident, &builtin.key)?;
                    let path: Path = syn::parse_str(path_str)?;
                    let (msg, code) = rf_cfg.message_code_for(&builtin.key);
                    generated_validate_attrs.push(build_validate_custom_path_attr(
                        &path,
                        &msg,
                        &code,
                    )?);
                }
                BuiltinRuleKind::PhoneNumberByIso2Field => {
                    let field_name = builtin.field.as_ref().ok_or_else(|| {
                        syn::Error::new_spanned(
                            &field_ident,
                            "#[rf(phonenumber(field = \"...\"))] requires field",
                        )
                    })?;
                    let other_ident = Ident::new(field_name, Span::call_site());
                    let (msg, code) = rf_cfg.message_code_for(&builtin.key);
                    generated_validate_attrs.push(build_validate_phonenumber_attr(
                        &other_ident,
                        &msg,
                        &code,
                    )?);
                }
                BuiltinRuleKind::GeneratedOneOf => {
                    ensure_string_like(field_kind, &field_ident, "one_of")?;
                    if builtin.values.is_empty() {
                        return Err(syn::Error::new_spanned(
                            &field_ident,
                            "#[rf(one_of(\"a\", \"b\", ...))] requires at least one value",
                        ));
                    }
                    let helper_ident = format_ident!(
                        "__rf_contract_{}_{}_one_of_{}",
                        struct_ident.to_string().to_lowercase(),
                        field_ident,
                        rule_i
                    );
                    helper_fns.push(generate_values_wrapper_fn(
                        &helper_ident,
                        true,
                        &builtin.values,
                    ));
                    let (msg, code) = rf_cfg.message_code_for(&builtin.key);
                    generated_validate_attrs.push(build_validate_custom_fn_attr(
                        &helper_ident,
                        &msg,
                        &code,
                    )?);
                    field_enum_values_patch = Some(builtin.values.clone());
                }
                BuiltinRuleKind::GeneratedNoneOf => {
                    ensure_string_like(field_kind, &field_ident, "none_of")?;
                    if builtin.values.is_empty() {
                        return Err(syn::Error::new_spanned(
                            &field_ident,
                            "#[rf(none_of(\"a\", \"b\", ...))] requires at least one value",
                        ));
                    }
                    let helper_ident = format_ident!(
                        "__rf_contract_{}_{}_none_of_{}",
                        struct_ident.to_string().to_lowercase(),
                        field_ident,
                        rule_i
                    );
                    helper_fns.push(generate_values_wrapper_fn(
                        &helper_ident,
                        false,
                        &builtin.values,
                    ));
                    let (msg, code) = rf_cfg.message_code_for(&builtin.key);
                    generated_validate_attrs.push(build_validate_custom_fn_attr(
                        &helper_ident,
                        &msg,
                        &code,
                    )?);
                }
                BuiltinRuleKind::GeneratedDate | BuiltinRuleKind::GeneratedDateTime => {
                    ensure_string_like(field_kind, &field_ident, &builtin.key)?;
                    let format = builtin.format.clone().ok_or_else(|| {
                        syn::Error::new_spanned(
                            &field_ident,
                            format!(
                                "#[rf({}(format = \"...\"))] requires format",
                                builtin.key
                            ),
                        )
                    })?;
                    let helper_ident = format_ident!(
                        "__rf_contract_{}_{}_{}_{}",
                        struct_ident.to_string().to_lowercase(),
                        field_ident,
                        builtin.key,
                        rule_i
                    );
                    helper_fns.push(generate_date_wrapper_fn(
                        &helper_ident,
                        &builtin.key,
                        &format,
                    ));
                    let (msg, code) = rf_cfg.message_code_for(&builtin.key);
                    generated_validate_attrs.push(build_validate_custom_fn_attr(
                        &helper_ident,
                        &msg,
                        &code,
                    )?);
                }
            }

            field_rule_extensions.push(RuleExtensionSpec::builtin(
                builtin,
                meta.default_message.to_string(),
                default_desc,
            ));
        }

        for custom in &rf_cfg.custom_rules {
            let fn_path = &custom.function;
            let mut nested = vec![quote! { function = #fn_path }];
            if let Some(msg) = &custom.message {
                nested.push(quote! { message = #msg });
            }
            if let Some(code) = &custom.code {
                nested.push(quote! { code = #code });
            }
            generated_validate_attrs
                .push(mk_attr(quote! { #[validate(custom(#(#nested),*))] })?);

            let mut params = BTreeMap::new();
            params.insert(
                "function".to_string(),
                JsonParam::String(fn_path.to_token_stream().to_string()),
            );
            field_rule_extensions.push(RuleExtensionSpec {
                key: "custom".to_string(),
                source: "app".to_string(),
                params,
                default_message: custom.message.clone(),
                description: custom.description.clone(),
            });
            if let Some(desc) = &custom.description {
                field_desc_parts.push(desc.clone());
            }
            if let Some(pat) = &custom.pattern {
                field_pattern_patch.get_or_insert_with(|| pat.clone());
            }
        }

        for async_rule in &rf_cfg.async_rules {
            field_desc_parts.push(async_rule.kind.default_desc(&async_rule.table, &async_rule.column));
            field_rule_extensions.push(RuleExtensionSpec::async_db(async_rule));
            async_validate_blocks.push(generate_async_db_rule_block(
                &field_ident,
                option_inner_type(&field_ty).is_some(),
                async_rule,
                &rf_cfg,
            ));
        }

        // explicit OpenAPI format override (escape hatch)
        if let Some(fmt) = &rf_cfg.openapi_format {
            generated_shadow_schemars_attrs.push(build_schemars_format_attr(fmt)?);
        }

        original_field_attrs.extend(generated_validate_attrs.clone());
        shadow_field_attrs.extend(generated_validate_attrs);
        shadow_field_attrs.extend(generated_shadow_schemars_attrs);

        let field_tokens_orig = quote! {
            #(#original_field_attrs)*
            #field_vis #field_ident: #field_ty
        };
        let field_tokens_shadow = quote! {
            #(#shadow_field_attrs)*
            #field_vis #field_ident: #field_ty
        };
        original_fields_tokens.push(field_tokens_orig);
        shadow_fields_tokens.push(field_tokens_shadow);

        if !field_rule_extensions.is_empty()
            || rf_cfg.openapi_description.is_some()
            || rf_cfg.openapi_hint.is_some()
            || rf_cfg.openapi_example.is_some()
            || rf_cfg.openapi_format.is_some()
            || field_pattern_patch.is_some()
            || field_enum_values_patch.is_some()
        {
            let prop_name = field_serde_rename(field).unwrap_or_else(|| field_ident.to_string());
            let explicit_schemars_desc = has_schemars_description(field);
            let doc_desc = doc_comment_description(field);
            let desc_expr = build_description_expr(
                explicit_schemars_desc,
                &rf_cfg.openapi_description,
                &doc_desc,
                &rf_cfg.openapi_hint,
                &field_desc_parts,
            );
            let rules_json_expr = build_rules_json_expr(&field_rule_extensions);
            let summary = field_rule_extensions
                .iter()
                .map(|r| r.key.clone())
                .collect::<Vec<_>>()
                .join(", ");
            let summary_expr = if summary.is_empty() {
                None
            } else {
                Some(quote! { #summary })
            };
            let example_expr = rf_cfg
                .openapi_example
                .as_ref()
                .map(|ex| quote! { #ex });
            let format_expr = rf_cfg
                .openapi_format
                .clone()
                .or_else(|| {
                    rf_cfg
                        .builtin_rules
                        .iter()
                        .filter_map(|r| {
                            builtin_rule_meta(&r.key).and_then(|m| m.format.map(str::to_string))
                        })
                        .next()
                })
                .map(|fmt| quote! { #fmt.to_string() });
            let pattern_expr = field_pattern_patch.map(|p| quote! { #p.to_string() });
            let enum_values_expr = field_enum_values_patch.map(|vals| {
                let vals_tokens = vals
                    .iter()
                    .map(|v| quote! { ::schemars::_serde_json::json!(#v) });
                quote! { vec![#(#vals_tokens),*] }
            });

            let patch = build_patch_block(
                &prop_name,
                desc_expr,
                example_expr,
                format_expr,
                pattern_expr,
                enum_values_expr,
                &summary_expr,
                rules_json_expr,
            );
            patch_blocks.push(patch);
        }

        async_validation_blocks_all.extend(async_validate_blocks);
    }

    let original_ident = struct_ident.clone();
    let schema_name_literal = struct_ident.to_string();
    let impl_block = quote! {
        impl ::schemars::JsonSchema for #original_ident {
            fn is_referenceable() -> bool {
                <#shadow_ident as ::schemars::JsonSchema>::is_referenceable()
            }

            fn schema_name() -> String {
                #schema_name_literal.to_string()
            }

            fn schema_id() -> ::std::borrow::Cow<'static, str> {
                ::std::borrow::Cow::Owned(Self::schema_name())
            }

            fn json_schema(generator: &mut ::schemars::gen::SchemaGenerator) -> ::schemars::schema::Schema {
                let mut schema = <#shadow_ident as ::schemars::JsonSchema>::json_schema(generator);
                #(#patch_blocks)*
                schema
            }

            fn _schemars_private_non_optional_json_schema(
                generator: &mut ::schemars::gen::SchemaGenerator,
            ) -> ::schemars::schema::Schema {
                <#shadow_ident as ::schemars::JsonSchema>::_schemars_private_non_optional_json_schema(generator)
            }

            fn _schemars_private_is_option() -> bool {
                <#shadow_ident as ::schemars::JsonSchema>::_schemars_private_is_option()
            }
        }
    };

    let async_validate_impl_block = if async_validation_blocks_all.is_empty() {
        quote! {}
    } else {
        quote! {
            #[::core_web::extract::async_trait]
            impl ::core_web::extract::AsyncValidate for #original_ident {
                async fn validate_async(
                    &self,
                    db: &::sqlx::PgPool,
                ) -> ::anyhow::Result<(), ::validator::ValidationErrors> {
                    let mut errors = ::validator::ValidationErrors::new();
                    #(#async_validation_blocks_all)*
                    if errors.is_empty() {
                        Ok(())
                    } else {
                        Err(errors)
                    }
                }
            }
        }
    };

    let original_attrs = item.attrs.clone();
    let original_generics = item.generics.clone();

    let expanded = quote! {
        #(#helper_fns)*

        #[derive(::std::fmt::Debug, ::std::clone::Clone, ::serde::Deserialize, ::validator::Validate #extra_derives)]
        #(#original_attrs)*
        #vis struct #struct_ident #original_generics {
            #(#original_fields_tokens,)*
        }

        #[allow(non_camel_case_types)]
        #[derive(::schemars::JsonSchema)]
        #(#container_attrs_for_shadow)*
        struct #shadow_ident {
            #(#shadow_fields_tokens,)*
        }

        #impl_block
        #async_validate_impl_block
    };

    Ok(expanded)
}

fn build_patch_block(
    prop_name: &str,
    desc_expr: TokenStream2,
    example_expr: Option<TokenStream2>,
    format_expr: Option<TokenStream2>,
    pattern_expr: Option<TokenStream2>,
    enum_values_expr: Option<TokenStream2>,
    summary_expr: &Option<TokenStream2>,
    rules_json_expr: TokenStream2,
) -> TokenStream2 {
    let example_block = example_expr.map(|example| {
        quote! {
            let meta = prop_obj.metadata.get_or_insert_with(Default::default).as_mut();
            meta.examples.push(::schemars::_serde_json::json!(#example));
        }
    });
    let format_block = format_expr.map(|fmt| {
        quote! {
            prop_obj.format = Some(#fmt);
        }
    });
    let pattern_block = pattern_expr.map(|pattern| {
        quote! {
            prop_obj.string.get_or_insert_with(Default::default).pattern = Some(#pattern);
        }
    });
    let enum_block = enum_values_expr.map(|vals| {
        quote! {
            prop_obj.enum_values = Some(#vals);
        }
    });
    let summary_block = summary_expr.as_ref().map(|summary| {
        quote! {
            prop_obj.extensions.insert(
                "x-rf-rule-summary".to_string(),
                ::schemars::_serde_json::json!(#summary),
            );
        }
    });

    quote! {
        if let ::schemars::schema::Schema::Object(root_obj) = &mut schema {
            if let Some(obj_validation) = root_obj.object.as_mut() {
                if let Some(prop_schema) = obj_validation.properties.get_mut(#prop_name) {
                    if let ::schemars::schema::Schema::Object(prop_obj) = prop_schema {
                        if let Some(desc) = #desc_expr {
                            let meta = prop_obj.metadata.get_or_insert_with(Default::default).as_mut();
                            if meta.description.is_none() {
                                meta.description = Some(desc);
                            }
                        }
                        #format_block
                        #example_block
                        #pattern_block
                        #enum_block
                        #summary_block
                        prop_obj.extensions.insert("x-rf-rules".to_string(), #rules_json_expr);
                    }
                }
            }
        }
    }
}

fn build_description_expr(
    has_manual_schemars_desc: bool,
    rf_openapi_description: &Option<String>,
    doc_desc: &Option<String>,
    rf_openapi_hint: &Option<String>,
    generated_parts: &[String],
) -> TokenStream2 {
    if has_manual_schemars_desc {
        return quote! { None::<String> };
    }

    let mut base = if let Some(desc) = rf_openapi_description {
        Some(desc.clone())
    } else {
        doc_desc.clone().filter(|s| !s.trim().is_empty())
    };

    if base.is_none() && !generated_parts.is_empty() {
        base = Some(generated_parts.join("; "));
    } else if let Some(existing) = &mut base {
        if !generated_parts.is_empty() {
            let joined = generated_parts.join("; ");
            if !joined.is_empty() {
                if !existing.ends_with('.') {
                    existing.push('.');
                }
                existing.push(' ');
                existing.push_str(&joined);
            }
        }
    }

    if let Some(hint) = rf_openapi_hint {
        if let Some(existing) = &mut base {
            if !existing.ends_with('.') {
                existing.push('.');
            }
            existing.push(' ');
            existing.push_str(hint);
        } else {
            base = Some(hint.clone());
        }
    }

    match base {
        Some(s) => quote! { Some(#s.to_string()) },
        None => quote! { None::<String> },
    }
}

fn build_rules_json_expr(specs: &[RuleExtensionSpec]) -> TokenStream2 {
    use serde_json::{json, Map, Value};

    let mut arr = Vec::<Value>::new();
    for spec in specs {
        let mut params = Map::new();
        for (k, v) in &spec.params {
            let value = match v {
                JsonParam::String(s) => Value::String(s.clone()),
                JsonParam::I64(n) => Value::Number((*n).into()),
                JsonParam::Bool(b) => Value::Bool(*b),
                JsonParam::StringList(values) => {
                    Value::Array(values.iter().cloned().map(Value::String).collect())
                }
            };
            params.insert(k.clone(), value);
        }
        arr.push(json!({
            "key": spec.key,
            "source": spec.source,
            "params": params,
            "default_message": spec.default_message,
            "description": spec.description,
        }));
    }
    let encoded = serde_json::to_string(&Value::Array(arr)).expect("serialize x-rf-rules");
    quote! {
        ::schemars::_serde_json::from_str::<::schemars::_serde_json::Value>(#encoded)
            .expect("valid x-rf-rules JSON")
    }
}

fn build_validate_length_attr(
    length: &LengthArgs,
    message: &Option<String>,
    code: &Option<String>,
) -> syn::Result<Attribute> {
    let mut nested = Vec::<TokenStream2>::new();
    if let Some(min) = &length.min {
        nested.push(quote! { min = #min });
    }
    if let Some(max) = &length.max {
        nested.push(quote! { max = #max });
    }
    if let Some(equal) = &length.equal {
        nested.push(quote! { equal = #equal });
    }
    if let Some(msg) = message {
        nested.push(quote! { message = #msg });
    }
    if let Some(code_value) = code {
        nested.push(quote! { code = #code_value });
    }
    mk_attr(quote! { #[validate(length(#(#nested),*))] })
}

fn build_validate_range_attr(
    range: &RangeArgs,
    message: &Option<String>,
    code: &Option<String>,
) -> syn::Result<Attribute> {
    let mut nested = Vec::<TokenStream2>::new();
    if let Some(min) = &range.min {
        nested.push(quote! { min = #min });
    }
    if let Some(max) = &range.max {
        nested.push(quote! { max = #max });
    }
    if let Some(msg) = message {
        nested.push(quote! { message = #msg });
    }
    if let Some(code_value) = code {
        nested.push(quote! { code = #code_value });
    }
    mk_attr(quote! { #[validate(range(#(#nested),*))] })
}

fn build_validate_simple_attr(
    name: &str,
    message: &Option<String>,
    code: &Option<String>,
) -> syn::Result<Attribute> {
    let ident = Ident::new(name, Span::call_site());
    let mut nested = Vec::<TokenStream2>::new();
    if let Some(msg) = message {
        nested.push(quote! { message = #msg });
    }
    if let Some(code_value) = code {
        nested.push(quote! { code = #code_value });
    }
    if nested.is_empty() {
        mk_attr(quote! { #[validate(#ident)] })
    } else {
        mk_attr(quote! { #[validate(#ident(#(#nested),*))] })
    }
}

fn build_validate_nested_attr() -> syn::Result<Attribute> {
    mk_attr(quote! { #[validate(nested)] })
}

fn build_validate_schema_attr(rule: &SchemaRuleRf) -> syn::Result<Attribute> {
    let function = rule
        .function
        .as_ref()
        .ok_or_else(|| syn::Error::new(Span::call_site(), "schema function missing"))?;
    let mut nested = vec![quote! { function = #function }];
    if let Some(use_context) = rule.use_context {
        nested.push(quote! { use_context = #use_context });
    }
    if let Some(skip) = rule.skip_on_field_errors {
        nested.push(quote! { skip_on_field_errors = #skip });
    }
    if let Some(msg) = &rule.message {
        nested.push(quote! { message = #msg });
    }
    if let Some(code) = &rule.code {
        nested.push(quote! { code = #code });
    }
    mk_attr(quote! { #[validate(schema(#(#nested),*))] })
}

fn build_validate_contains_attr(
    pattern: &str,
    message: &Option<String>,
    code: &Option<String>,
) -> syn::Result<Attribute> {
    let mut nested = vec![quote! { pattern = #pattern }];
    if let Some(msg) = message {
        nested.push(quote! { message = #msg });
    }
    if let Some(code_value) = code {
        nested.push(quote! { code = #code_value });
    }
    mk_attr(quote! { #[validate(contains(#(#nested),*))] })
}

fn build_validate_does_not_contain_attr(
    pattern: &str,
    message: &Option<String>,
    code: &Option<String>,
) -> syn::Result<Attribute> {
    let mut nested = vec![quote! { pattern = #pattern }];
    if let Some(msg) = message {
        nested.push(quote! { message = #msg });
    }
    if let Some(code_value) = code {
        nested.push(quote! { code = #code_value });
    }
    mk_attr(quote! { #[validate(does_not_contain(#(#nested),*))] })
}

fn build_validate_must_match_attr(
    other: &str,
    message: &Option<String>,
    code: &Option<String>,
) -> syn::Result<Attribute> {
    let mut nested = vec![quote! { other = #other }];
    if let Some(msg) = message {
        nested.push(quote! { message = #msg });
    }
    if let Some(code_value) = code {
        nested.push(quote! { code = #code_value });
    }
    mk_attr(quote! { #[validate(must_match(#(#nested),*))] })
}

fn build_validate_custom_fn_attr(
    function_ident: &Ident,
    message: &Option<String>,
    code: &Option<String>,
) -> syn::Result<Attribute> {
    let mut nested = vec![quote! { function = #function_ident }];
    if let Some(msg) = message {
        nested.push(quote! { message = #msg });
    }
    if let Some(code) = code {
        nested.push(quote! { code = #code });
    }
    mk_attr(quote! { #[validate(custom(#(#nested),*))] })
}

fn build_validate_custom_path_attr(
    function_path: &Path,
    message: &Option<String>,
    code: &Option<String>,
) -> syn::Result<Attribute> {
    let mut nested = vec![quote! { function = #function_path }];
    if let Some(msg) = message {
        nested.push(quote! { message = #msg });
    }
    if let Some(code) = code {
        nested.push(quote! { code = #code });
    }
    mk_attr(quote! { #[validate(custom(#(#nested),*))] })
}

fn build_validate_phonenumber_attr(
    field_ident: &Ident,
    message: &Option<String>,
    code: &Option<String>,
) -> syn::Result<Attribute> {
    let mut nested = vec![quote! { field = #field_ident }];
    if let Some(msg) = message {
        nested.push(quote! { message = #msg });
    }
    if let Some(code) = code {
        nested.push(quote! { code = #code });
    }
    mk_attr(quote! { #[validate(phonenumber(#(#nested),*))] })
}

fn build_schemars_regex_attr(pattern: &str) -> syn::Result<Attribute> {
    mk_attr(quote! { #[schemars(regex(pattern = #pattern))] })
}

fn build_schemars_format_attr(fmt: &str) -> syn::Result<Attribute> {
    mk_attr(quote! { #[schemars(format = #fmt)] })
}

fn generate_regex_wrapper_fn(ident: &Ident, pattern: &str) -> TokenStream2 {
    quote! {
        fn #ident(value: &str) -> Result<(), ::validator::ValidationError> {
            ::core_web::rules::regex_pattern(value, #pattern)
        }
    }
}

fn generate_values_wrapper_fn(ident: &Ident, allow_list: bool, values: &[String]) -> TokenStream2 {
    let values_lits = values.iter().map(|v| quote! { #v });
    if allow_list {
        quote! {
            fn #ident(value: &str) -> Result<(), ::validator::ValidationError> {
                ::core_web::rules::one_of(value, &[#(#values_lits),*])
            }
        }
    } else {
        quote! {
            fn #ident(value: &str) -> Result<(), ::validator::ValidationError> {
                ::core_web::rules::none_of(value, &[#(#values_lits),*])
            }
        }
    }
}

fn generate_date_wrapper_fn(ident: &Ident, key: &str, format: &str) -> TokenStream2 {
    let call = if key == "date" {
        quote! { ::core_web::rules::date(value, #format) }
    } else {
        quote! { ::core_web::rules::datetime(value, #format) }
    };
    quote! {
        fn #ident(value: &str) -> Result<(), ::validator::ValidationError> {
            #call
        }
    }
}

fn generate_async_db_rule_block(
    field_ident: &Ident,
    is_option: bool,
    rule: &AsyncDbRuleUse,
    rf_cfg: &FieldRfConfig,
) -> TokenStream2 {
    let field_name = field_ident.to_string();
    let table = rule.table.clone();
    let column = rule.column.clone();
    let mut builder_expr = match rule.kind {
        AsyncDbRuleKind::Unique => quote! { ::core_web::rules::Unique::new(#table, #column, value) },
        AsyncDbRuleKind::Exists => quote! { ::core_web::rules::Exists::new(#table, #column, value) },
        AsyncDbRuleKind::NotExists => quote! { ::core_web::rules::NotExists::new(#table, #column, value) },
    };
    for modifier in &rule.modifiers {
        builder_expr = apply_async_db_modifier(builder_expr, modifier);
    }
    let (message, code) = rf_cfg.message_code_for(rule.kind.key());
    let code_expr = code.unwrap_or_else(|| rule.kind.default_code().to_string());
    let message_expr = if let Some(msg) = message {
        quote! { #msg.to_string() }
    } else {
        quote! { ::core_web::rules::AsyncRule::message(&rule_builder) }
    };

    let body = quote! {
        let rule_builder = #builder_expr;
        let ok = match ::core_web::rules::AsyncRule::check(&rule_builder, db).await {
            Ok(value) => value,
            Err(_) => {
                let mut err = ::validator::ValidationError::new("async_validation");
                err.message = Some(::std::borrow::Cow::Borrowed("Async validation check failed."));
                errors.add(#field_name, err);
                true
            }
        };
        if !ok {
            let mut err = ::validator::ValidationError::new(#code_expr);
            err.message = Some(::std::borrow::Cow::Owned(#message_expr));
            errors.add(#field_name, err);
        }
    };

    if is_option {
        quote! {
            if let Some(value) = &self.#field_ident {
                #body
            }
        }
    } else {
        quote! {
            let value = &self.#field_ident;
            #body
        }
    }
}

fn apply_async_db_modifier(base: TokenStream2, modifier: &AsyncDbModifier) -> TokenStream2 {
    match modifier {
        AsyncDbModifier::Ignore { column, source } => {
            let source_expr = async_db_value_source_expr(source);
            quote! { (#base).ignore(#column, #source_expr) }
        }
        AsyncDbModifier::WhereEq { column, source } => {
            let source_expr = async_db_value_source_expr(source);
            quote! { (#base).where_eq(#column, #source_expr) }
        }
        AsyncDbModifier::WhereNotEq { column, source } => {
            let source_expr = async_db_value_source_expr(source);
            quote! { (#base).where_not_eq(#column, #source_expr) }
        }
        AsyncDbModifier::WhereNull { column } => quote! { (#base).where_null(#column) },
        AsyncDbModifier::WhereNotNull { column } => quote! { (#base).where_not_null(#column) },
    }
}

fn async_db_value_source_expr(source: &AsyncDbValueSource) -> TokenStream2 {
    match source {
        AsyncDbValueSource::Field(field_name) => {
            let ident = Ident::new(field_name, Span::call_site());
            quote! { &self.#ident }
        }
        AsyncDbValueSource::Expr(expr) => quote! { #expr },
    }
}

fn mk_attr(tokens: TokenStream2) -> syn::Result<Attribute> {
    let parser = Attribute::parse_outer;
    let mut attrs = parser.parse2(tokens)?;
    attrs
        .pop()
        .ok_or_else(|| syn::Error::new(Span::call_site(), "failed to parse generated attribute"))
}

fn parse_rf_field(
    field: &Field,
) -> syn::Result<(FieldRfConfig, Vec<Attribute>, Vec<TokenStream2>)> {
    let mut cfg = FieldRfConfig::default();
    let mut keep_attrs = Vec::new();
    let mut rf_attr_tokens = Vec::new();

    for attr in &field.attrs {
        if !attr.path().is_ident("rf") {
            keep_attrs.push(attr.clone());
            continue;
        }
        rf_attr_tokens.push(attr.to_token_stream());
        let metas = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
        let mut local_builtin: Option<BuiltinRuleUse> = None;
        let mut local_length: Option<LengthArgs> = None;
        let mut local_range: Option<RangeArgs> = None;
        let mut local_regex: Option<String> = None;
        let mut local_contains: Option<String> = None;
        let mut local_does_not_contain: Option<String> = None;
        let mut local_email = false;
        let mut local_url = false;
        let mut local_required = false;
        let mut local_nested = false;
        let mut local_must_match: Option<String> = None;
        let mut local_async_rules: Vec<AsyncDbRuleUse> = Vec::new();
        let mut local_message: Option<String> = None;
        let mut local_code: Option<String> = None;
        let mut local_rule_keys: Vec<String> = Vec::new();

        for meta in metas {
            match meta {
                // --- Parameterless core rules ---
                Meta::Path(ref path) if path.is_ident("email") => {
                    local_email = true;
                    local_rule_keys.push("email".to_string());
                }
                Meta::Path(ref path) if path.is_ident("url") => {
                    local_url = true;
                    local_rule_keys.push("url".to_string());
                }
                Meta::Path(ref path) if path.is_ident("required") => {
                    local_required = true;
                    local_rule_keys.push("required".to_string());
                }
                Meta::Path(ref path) if path.is_ident("nested") => {
                    local_nested = true;
                    local_rule_keys.push("nested".to_string());
                }
                // --- Parameterless builtins from registry ---
                Meta::Path(ref path) => {
                    let name = path.get_ident().map(|i| i.to_string()).unwrap_or_default();
                    if let Some(meta_entry) = builtin_rule_meta(&name) {
                        if meta_entry.args != BuiltinRuleArgs::None {
                            return Err(syn::Error::new_spanned(
                                path,
                                format!("#[rf({})] requires arguments", name),
                            ));
                        }
                        local_builtin = Some(BuiltinRuleUse {
                            key: name.clone(),
                            values: vec![],
                            format: None,
                            field: None,
                        });
                        local_rule_keys.push(name);
                    } else {
                        return Err(syn::Error::new_spanned(
                            path,
                            "unsupported #[rf(...)] syntax",
                        ));
                    }
                }
                // --- Parameterized core rules ---
                Meta::List(ref list) if list.path.is_ident("length") => {
                    if local_length.is_some() || cfg.length.is_some() {
                        return Err(syn::Error::new_spanned(list, "duplicate rf length"));
                    }
                    local_length = Some(parse_length_args(list)?);
                    local_rule_keys.push("length".to_string());
                }
                Meta::List(ref list) if list.path.is_ident("range") => {
                    if local_range.is_some() || cfg.range.is_some() {
                        return Err(syn::Error::new_spanned(list, "duplicate rf range"));
                    }
                    local_range = Some(parse_range_args(list)?);
                    local_rule_keys.push("range".to_string());
                }
                Meta::List(ref list) if list.path.is_ident("regex") => {
                    if local_regex.is_some() || cfg.regex_pattern.is_some() {
                        return Err(syn::Error::new_spanned(list, "duplicate rf regex"));
                    }
                    local_regex = Some(parse_regex_pattern(list)?);
                    local_rule_keys.push("regex".to_string());
                }
                Meta::List(ref list) if list.path.is_ident("contains") => {
                    if local_contains.is_some() || cfg.contains_pattern.is_some() {
                        return Err(syn::Error::new_spanned(list, "duplicate rf contains"));
                    }
                    local_contains = Some(parse_contains_pattern(list)?);
                    local_rule_keys.push("contains".to_string());
                }
                Meta::List(ref list) if list.path.is_ident("does_not_contain") => {
                    if local_does_not_contain.is_some() || cfg.does_not_contain_pattern.is_some() {
                        return Err(syn::Error::new_spanned(
                            list,
                            "duplicate rf does_not_contain",
                        ));
                    }
                    local_does_not_contain = Some(parse_does_not_contain_pattern(list)?);
                    local_rule_keys.push("does_not_contain".to_string());
                }
                Meta::List(ref list) if list.path.is_ident("must_match") => {
                    if local_must_match.is_some() || cfg.must_match_other.is_some() {
                        return Err(syn::Error::new_spanned(list, "duplicate rf must_match"));
                    }
                    local_must_match = Some(parse_must_match_other(list)?);
                    local_rule_keys.push("must_match".to_string());
                }
                // --- Async DB rules ---
                Meta::List(ref list) if list.path.is_ident("async_unique") => {
                    local_async_rules.push(parse_async_db_rule(list, AsyncDbRuleKind::Unique)?);
                    local_rule_keys.push("async_unique".to_string());
                }
                Meta::List(ref list) if list.path.is_ident("async_exists") => {
                    local_async_rules.push(parse_async_db_rule(list, AsyncDbRuleKind::Exists)?);
                    local_rule_keys.push("async_exists".to_string());
                }
                Meta::List(ref list) if list.path.is_ident("async_not_exists") => {
                    local_async_rules.push(parse_async_db_rule(
                        list,
                        AsyncDbRuleKind::NotExists,
                    )?);
                    local_rule_keys.push("async_not_exists".to_string());
                }
                // --- custom() rule ---
                Meta::List(ref list) if list.path.is_ident("custom") => {
                    cfg.custom_rules.push(parse_custom_rule(list)?);
                }
                // --- openapi() grouped ---
                Meta::List(ref list) if list.path.is_ident("openapi") => {
                    parse_openapi_group(list, &mut cfg)?;
                }
                // --- Core rules in list form for message/code override ---
                Meta::List(ref list)
                    if list.path.is_ident("email")
                        || list.path.is_ident("url")
                        || list.path.is_ident("required")
                        || list.path.is_ident("nested") =>
                {
                    let name = list.path.get_ident().unwrap().to_string();
                    let (msg, code) = parse_message_code_from_list(list)?;
                    match name.as_str() {
                        "email" => local_email = true,
                        "url" => local_url = true,
                        "required" => local_required = true,
                        "nested" => local_nested = true,
                        _ => unreachable!(),
                    }
                    local_rule_keys.push(name.clone());
                    let entry = cfg.rule_overrides.entry(name).or_default();
                    if let Some(m) = msg {
                        entry.message = Some(m);
                    }
                    if let Some(c) = code {
                        entry.code = Some(c);
                    }
                }
                // --- Parameterized builtins from registry ---
                Meta::List(ref list) => {
                    let name = list.path.get_ident().map(|i| i.to_string()).unwrap_or_default();
                    if let Some(meta_entry) = builtin_rule_meta(&name) {
                        let (builtin, blt_msg, blt_code) =
                            parse_builtin_from_list(list, meta_entry)?;
                        if blt_msg.is_some() || blt_code.is_some() {
                            let entry = cfg.rule_overrides.entry(name.clone()).or_default();
                            if blt_msg.is_some() {
                                entry.message = blt_msg;
                            }
                            if blt_code.is_some() {
                                entry.code = blt_code;
                            }
                        }
                        local_rule_keys.push(name);
                        local_builtin = Some(builtin);
                    } else {
                        return Err(syn::Error::new_spanned(
                            list,
                            format!("unknown #[rf({}(...))] rule", name),
                        ));
                    }
                }
                // --- Standalone message/code (field-level default) ---
                Meta::NameValue(ref nv) if nv.path.is_ident("message") => {
                    local_message = Some(lit_str_from_expr(&nv.value, "message")?.value());
                }
                Meta::NameValue(ref nv) if nv.path.is_ident("code") => {
                    local_code = Some(lit_str_from_expr(&nv.value, "code")?.value());
                }
                other => {
                    return Err(syn::Error::new_spanned(
                        other,
                        "unsupported #[rf(...)] syntax",
                    ));
                }
            }
        }

        if let Some(length) = local_length {
            if length.equal.is_some() && (length.min.is_some() || length.max.is_some()) {
                return Err(syn::Error::new_spanned(
                    attr,
                    "rf length(equal = ...) cannot be combined with min/max",
                ));
            }
            cfg.length = Some(length);
        }
        if let Some(range) = local_range {
            cfg.range = Some(range);
        }
        if let Some(regex) = local_regex {
            cfg.regex_pattern = Some(regex);
        }
        if let Some(pattern) = local_contains {
            cfg.contains_pattern = Some(pattern);
        }
        if let Some(pattern) = local_does_not_contain {
            cfg.does_not_contain_pattern = Some(pattern);
        }
        cfg.email |= local_email;
        cfg.url |= local_url;
        cfg.required |= local_required;
        cfg.nested |= local_nested;
        if let Some(other) = local_must_match {
            cfg.must_match_other = Some(other);
        }
        cfg.async_rules.extend(local_async_rules);

        if let Some(builtin) = local_builtin {
            cfg.builtin_rules.push(builtin);
        }

        if local_message.is_some() || local_code.is_some() {
            if local_rule_keys.is_empty() {
                if let Some(m) = local_message {
                    cfg.message = Some(m);
                }
                if let Some(c) = local_code {
                    cfg.code = Some(c);
                }
            } else {
                for rule_key in local_rule_keys {
                    let entry = cfg.rule_overrides.entry(rule_key).or_default();
                    if entry.message.is_none() {
                        entry.message = local_message.clone();
                    }
                    if entry.code.is_none() {
                        entry.code = local_code.clone();
                    }
                }
            }
        }
    }

    if cfg.regex_pattern.is_some() && cfg.builtin_rules.iter().any(|r| r.key == "one_of") {
        return Err(syn::Error::new_spanned(
            field,
            "rf regex(...) cannot be combined with rf(one_of(...)) on the same field",
        ));
    }

    Ok((cfg, keep_attrs, rf_attr_tokens))
}

fn parse_builtin_from_list(
    list: &MetaList,
    meta: &rustforge_contract_meta::BuiltinRuleMeta,
) -> syn::Result<(BuiltinRuleUse, Option<String>, Option<String>)> {
    let mut values = Vec::new();
    let mut format = None;
    let mut field = None;
    let mut message = None;
    let mut code = None;

    // For Values args, parse mixed positional strings + named args
    if meta.args == BuiltinRuleArgs::Values {
        let exprs = list.parse_args_with(Punctuated::<Expr, Token![,]>::parse_terminated)?;
        for expr in &exprs {
            match expr {
                Expr::Lit(expr_lit) => {
                    if let Lit::Str(s) = &expr_lit.lit {
                        values.push(s.value());
                    }
                }
                Expr::Assign(assign) => {
                    if let Expr::Path(path) = &*assign.left {
                        let key = path
                            .path
                            .get_ident()
                            .map(|i| i.to_string())
                            .unwrap_or_default();
                        match key.as_str() {
                            "message" => {
                                message =
                                    Some(lit_str_from_expr(&assign.right, "message")?.value());
                            }
                            "code" => {
                                code = Some(lit_str_from_expr(&assign.right, "code")?.value());
                            }
                            _ => {
                                return Err(syn::Error::new_spanned(
                                    expr,
                                    format!("unexpected argument `{}`", key),
                                ));
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    } else {
        let metas = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
        for m in &metas {
            match m {
                Meta::NameValue(nv) if nv.path.is_ident("format") => {
                    format = Some(lit_str_from_expr(&nv.value, "format")?.value());
                }
                Meta::NameValue(nv) if nv.path.is_ident("field") => {
                    field = Some(lit_str_from_expr(&nv.value, "field")?.value());
                }
                Meta::NameValue(nv) if nv.path.is_ident("message") => {
                    message = Some(lit_str_from_expr(&nv.value, "message")?.value());
                }
                Meta::NameValue(nv) if nv.path.is_ident("code") => {
                    code = Some(lit_str_from_expr(&nv.value, "code")?.value());
                }
                _ => {
                    return Err(syn::Error::new_spanned(m, "unexpected argument"));
                }
            }
        }
    }

    // Validate required args based on meta.args
    match meta.args {
        BuiltinRuleArgs::Format if format.is_none() => {
            return Err(syn::Error::new_spanned(
                list,
                format!("#[rf({}(...))] requires format = \"...\"", meta.key),
            ));
        }
        BuiltinRuleArgs::Field if field.is_none() => {
            return Err(syn::Error::new_spanned(
                list,
                format!("#[rf({}(...))] requires field = \"...\"", meta.key),
            ));
        }
        BuiltinRuleArgs::Values if values.is_empty() => {
            return Err(syn::Error::new_spanned(
                list,
                format!("#[rf({}(...))] requires at least one value", meta.key),
            ));
        }
        _ => {}
    }

    Ok((
        BuiltinRuleUse {
            key: meta.key.to_string(),
            values,
            format,
            field,
        },
        message,
        code,
    ))
}

fn parse_custom_rule(list: &MetaList) -> syn::Result<CustomRuleUse> {
    let metas = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
    let mut function: Option<Path> = None;
    let mut description: Option<String> = None;
    let mut pattern: Option<String> = None;
    let mut custom_code: Option<String> = None;
    let mut custom_message: Option<String> = None;

    for m in metas {
        match m {
            Meta::NameValue(nv) if nv.path.is_ident("function") => {
                function = Some(path_from_expr(&nv.value, "function")?);
            }
            Meta::NameValue(nv) if nv.path.is_ident("description") => {
                description = Some(lit_str_from_expr(&nv.value, "description")?.value());
            }
            Meta::NameValue(nv) if nv.path.is_ident("pattern") => {
                pattern = Some(lit_str_from_expr(&nv.value, "pattern")?.value());
            }
            Meta::NameValue(nv) if nv.path.is_ident("code") => {
                custom_code = Some(lit_str_from_expr(&nv.value, "code")?.value());
            }
            Meta::NameValue(nv) if nv.path.is_ident("message") => {
                custom_message = Some(lit_str_from_expr(&nv.value, "message")?.value());
            }
            other => {
                return Err(syn::Error::new_spanned(other, "unsupported custom(...) arg"))
            }
        }
    }
    let function = function
        .ok_or_else(|| syn::Error::new_spanned(list, "custom(...) requires function = ..."))?;

    Ok(CustomRuleUse {
        function,
        description,
        pattern,
        code: custom_code,
        message: custom_message,
    })
}

fn parse_openapi_group(list: &MetaList, cfg: &mut FieldRfConfig) -> syn::Result<()> {
    let metas = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
    for m in metas {
        match m {
            Meta::NameValue(nv) if nv.path.is_ident("description") => {
                cfg.openapi_description =
                    Some(lit_str_from_expr(&nv.value, "description")?.value());
            }
            Meta::NameValue(nv) if nv.path.is_ident("hint") => {
                cfg.openapi_hint = Some(lit_str_from_expr(&nv.value, "hint")?.value());
            }
            Meta::NameValue(nv) if nv.path.is_ident("example") => {
                cfg.openapi_example = Some(nv.value.clone());
            }
            Meta::NameValue(nv) if nv.path.is_ident("format") => {
                cfg.openapi_format = Some(lit_str_from_expr(&nv.value, "format")?.value());
            }
            other => {
                return Err(syn::Error::new_spanned(
                    other,
                    "unsupported openapi(...) arg",
                ))
            }
        }
    }
    Ok(())
}

fn parse_message_code_from_list(list: &MetaList) -> syn::Result<(Option<String>, Option<String>)> {
    let metas = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
    let mut message = None;
    let mut code = None;
    for m in metas {
        match m {
            Meta::NameValue(nv) if nv.path.is_ident("message") => {
                message = Some(lit_str_from_expr(&nv.value, "message")?.value());
            }
            Meta::NameValue(nv) if nv.path.is_ident("code") => {
                code = Some(lit_str_from_expr(&nv.value, "code")?.value());
            }
            other => {
                return Err(syn::Error::new_spanned(
                    other,
                    "expected message = \"...\" or code = \"...\"",
                ))
            }
        }
    }
    Ok((message, code))
}

fn parse_rf_container_attrs(attrs: &[Attribute]) -> syn::Result<(ContainerRfConfig, Vec<Attribute>)> {
    let mut cfg = ContainerRfConfig::default();
    let mut keep = Vec::new();

    for attr in attrs {
        if !attr.path().is_ident("rf") {
            keep.push(attr.clone());
            continue;
        }

        let metas = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
        for meta in metas {
            match meta {
                Meta::List(list) if list.path.is_ident("schema") => {
                    cfg.schema_rules.push(parse_container_schema_rule(&list)?);
                }
                other => {
                    return Err(syn::Error::new_spanned(
                        other,
                        "unsupported #[rf(...)] syntax on struct (supported: rf(schema(...)))",
                    ));
                }
            }
        }
    }

    Ok((cfg, keep))
}

fn parse_container_schema_rule(list: &MetaList) -> syn::Result<SchemaRuleRf> {
    let metas = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
    let mut out = SchemaRuleRf::default();
    for meta in metas {
        match meta {
            Meta::NameValue(nv) if nv.path.is_ident("function") => {
                out.function = Some(path_from_expr(&nv.value, "function")?);
            }
            Meta::NameValue(nv) if nv.path.is_ident("use_context") => {
                out.use_context = Some(bool_from_expr(&nv.value, "use_context")?);
            }
            Meta::NameValue(nv) if nv.path.is_ident("skip_on_field_errors") => {
                out.skip_on_field_errors =
                    Some(bool_from_expr(&nv.value, "skip_on_field_errors")?);
            }
            Meta::NameValue(nv) if nv.path.is_ident("message") => {
                out.message = Some(lit_str_from_expr(&nv.value, "message")?.value());
            }
            Meta::NameValue(nv) if nv.path.is_ident("code") => {
                out.code = Some(lit_str_from_expr(&nv.value, "code")?.value());
            }
            other => {
                return Err(syn::Error::new_spanned(
                    other,
                    "unsupported rf schema(...) argument",
                ));
            }
        }
    }

    if out.function.is_none() {
        return Err(syn::Error::new_spanned(
            list,
            "rf schema(...) requires function = \"path::to::fn\"",
        ));
    }

    Ok(out)
}

fn parse_async_db_rule(list: &MetaList, kind: AsyncDbRuleKind) -> syn::Result<AsyncDbRuleUse> {
    let metas = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
    let mut table: Option<String> = None;
    let mut column: Option<String> = None;
    let mut modifiers = Vec::new();
    for meta in metas {
        match meta {
            Meta::NameValue(nv) if nv.path.is_ident("table") => {
                table = Some(lit_str_from_expr(&nv.value, "table")?.value());
            }
            Meta::NameValue(nv) if nv.path.is_ident("column") => {
                column = Some(lit_str_from_expr(&nv.value, "column")?.value());
            }
            Meta::NameValue(nv) if nv.path.is_ident("message") || nv.path.is_ident("code") => {
                // Parsed at the parent rf(...) attribute level so overrides can be shared.
            }
            Meta::List(inner) if inner.path.is_ident("ignore") => {
                if !matches!(kind, AsyncDbRuleKind::Unique) {
                    return Err(syn::Error::new_spanned(
                        inner,
                        "ignore(...) is only supported for async_unique(...)",
                    ));
                }
                modifiers.push(parse_async_db_modifier(&inner)?);
            }
            Meta::List(inner)
                if inner.path.is_ident("where_eq")
                    || inner.path.is_ident("where_not_eq")
                    || inner.path.is_ident("where_null")
                    || inner.path.is_ident("where_not_null") =>
            {
                modifiers.push(parse_async_db_modifier(&inner)?);
            }
            other => {
                return Err(syn::Error::new_spanned(
                    other,
                    "unsupported async db rule argument (expected table, column, message, code, ignore/where_*)",
                ));
            }
        }
    }

    Ok(AsyncDbRuleUse {
        kind,
        table: table.ok_or_else(|| syn::Error::new_spanned(list, "async rule requires table"))?,
        column: column.ok_or_else(|| syn::Error::new_spanned(list, "async rule requires column"))?,
        modifiers,
    })
}

fn parse_async_db_modifier(list: &MetaList) -> syn::Result<AsyncDbModifier> {
    let metas = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
    let mut column: Option<String> = None;
    let mut field: Option<String> = None;
    let mut value: Option<Expr> = None;

    for meta in metas {
        match meta {
            Meta::NameValue(nv) if nv.path.is_ident("column") => {
                column = Some(lit_str_from_expr(&nv.value, "column")?.value());
            }
            Meta::NameValue(nv) if nv.path.is_ident("field") => {
                field = Some(lit_str_from_expr(&nv.value, "field")?.value());
            }
            Meta::NameValue(nv) if nv.path.is_ident("value") => {
                value = Some(nv.value.clone());
            }
            other => {
                return Err(syn::Error::new_spanned(
                    other,
                    "unsupported async db modifier argument",
                ));
            }
        }
    }

    let col = column.ok_or_else(|| syn::Error::new_spanned(list, "modifier requires column"))?;
    let source = match (field, value) {
        (Some(_), Some(_)) => {
            return Err(syn::Error::new_spanned(
                list,
                "use either field = ... or value = ..., not both",
            ))
        }
        (Some(field_name), None) => Some(AsyncDbValueSource::Field(field_name)),
        (None, Some(expr)) => Some(AsyncDbValueSource::Expr(expr)),
        (None, None) => None,
    };

    if list.path.is_ident("ignore") {
        return Ok(AsyncDbModifier::Ignore {
            column: col,
            source: source.ok_or_else(|| {
                syn::Error::new_spanned(list, "ignore(...) requires field = ... or value = ...")
            })?,
        });
    }
    if list.path.is_ident("where_eq") {
        return Ok(AsyncDbModifier::WhereEq {
            column: col,
            source: source.ok_or_else(|| {
                syn::Error::new_spanned(list, "where_eq(...) requires field = ... or value = ...")
            })?,
        });
    }
    if list.path.is_ident("where_not_eq") {
        return Ok(AsyncDbModifier::WhereNotEq {
            column: col,
            source: source.ok_or_else(|| {
                syn::Error::new_spanned(
                    list,
                    "where_not_eq(...) requires field = ... or value = ...",
                )
            })?,
        });
    }
    if list.path.is_ident("where_null") {
        if source.is_some() {
            return Err(syn::Error::new_spanned(
                list,
                "where_null(...) does not accept field/value",
            ));
        }
        return Ok(AsyncDbModifier::WhereNull { column: col });
    }
    if list.path.is_ident("where_not_null") {
        if source.is_some() {
            return Err(syn::Error::new_spanned(
                list,
                "where_not_null(...) does not accept field/value",
            ));
        }
        return Ok(AsyncDbModifier::WhereNotNull { column: col });
    }

    Err(syn::Error::new_spanned(
        list,
        "unsupported async db modifier",
    ))
}

fn parse_length_args(list: &MetaList) -> syn::Result<LengthArgs> {
    let metas = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
    let mut out = LengthArgs::default();
    for meta in metas {
        match meta {
            Meta::NameValue(nv) if nv.path.is_ident("min") => {
                out.min = Some(number_expr(nv.value)?)
            }
            Meta::NameValue(nv) if nv.path.is_ident("max") => {
                out.max = Some(number_expr(nv.value)?)
            }
            Meta::NameValue(nv) if nv.path.is_ident("equal") => {
                out.equal = Some(number_expr(nv.value)?)
            }
            other => {
                return Err(syn::Error::new_spanned(
                    other,
                    "unsupported rf length(...) argument",
                ))
            }
        }
    }
    Ok(out)
}

fn parse_range_args(list: &MetaList) -> syn::Result<RangeArgs> {
    let metas = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
    let mut out = RangeArgs::default();
    for meta in metas {
        match meta {
            Meta::NameValue(nv) if nv.path.is_ident("min") => {
                out.min = Some(number_expr(nv.value)?)
            }
            Meta::NameValue(nv) if nv.path.is_ident("max") => {
                out.max = Some(number_expr(nv.value)?)
            }
            other => {
                return Err(syn::Error::new_spanned(
                    other,
                    "unsupported rf range(...) argument",
                ))
            }
        }
    }
    Ok(out)
}

fn parse_regex_pattern(list: &MetaList) -> syn::Result<String> {
    let metas = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
    for meta in metas {
        if let Meta::NameValue(nv) = meta {
            if nv.path.is_ident("pattern") {
                return Ok(lit_str_from_expr(&nv.value, "pattern")?.value());
            }
        }
    }
    Err(syn::Error::new_spanned(
        list,
        "rf regex(...) requires pattern = \"...\"",
    ))
}

fn parse_contains_pattern(list: &MetaList) -> syn::Result<String> {
    let metas = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
    for meta in metas {
        if let Meta::NameValue(nv) = meta {
            if nv.path.is_ident("pattern") {
                return Ok(lit_str_from_expr(&nv.value, "pattern")?.value());
            }
        }
    }
    Err(syn::Error::new_spanned(
        list,
        "rf contains(...) requires pattern = \"...\"",
    ))
}

fn parse_does_not_contain_pattern(list: &MetaList) -> syn::Result<String> {
    let metas = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
    for meta in metas {
        if let Meta::NameValue(nv) = meta {
            if nv.path.is_ident("pattern") {
                return Ok(lit_str_from_expr(&nv.value, "pattern")?.value());
            }
        }
    }
    Err(syn::Error::new_spanned(
        list,
        "rf does_not_contain(...) requires pattern = \"...\"",
    ))
}

fn parse_must_match_other(list: &MetaList) -> syn::Result<String> {
    let metas = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
    for meta in metas {
        if let Meta::NameValue(nv) = meta {
            if nv.path.is_ident("other") {
                return Ok(lit_str_from_expr(&nv.value, "other")?.value());
            }
        }
    }
    Err(syn::Error::new_spanned(
        list,
        "rf must_match(...) requires other = \"field_name\"",
    ))
}

fn lit_str_from_expr<'a>(expr: &'a Expr, name: &str) -> syn::Result<&'a LitStr> {
    match expr {
        Expr::Lit(expr_lit) => match &expr_lit.lit {
            Lit::Str(s) => Ok(s),
            _ => Err(syn::Error::new_spanned(
                expr,
                format!("{name} must be a string literal"),
            )),
        },
        _ => Err(syn::Error::new_spanned(
            expr,
            format!("{name} must be a string literal"),
        )),
    }
}

fn bool_from_expr(expr: &Expr, name: &str) -> syn::Result<bool> {
    match expr {
        Expr::Lit(expr_lit) => match &expr_lit.lit {
            Lit::Bool(b) => Ok(b.value),
            _ => Err(syn::Error::new_spanned(
                expr,
                format!("{name} must be a boolean literal"),
            )),
        },
        _ => Err(syn::Error::new_spanned(
            expr,
            format!("{name} must be a boolean literal"),
        )),
    }
}

fn path_from_expr(expr: &Expr, name: &str) -> syn::Result<Path> {
    match expr {
        Expr::Path(p) => Ok(p.path.clone()),
        Expr::Lit(expr_lit) => match &expr_lit.lit {
            Lit::Str(s) => syn::parse_str::<Path>(&s.value()).map_err(|_| {
                syn::Error::new_spanned(expr, format!("{name} must be a valid path"))
            }),
            _ => Err(syn::Error::new_spanned(
                expr,
                format!("{name} must be a path or string path"),
            )),
        },
        _ => Err(syn::Error::new_spanned(
            expr,
            format!("{name} must be a path or string path"),
        )),
    }
}

fn number_expr(expr: Expr) -> syn::Result<Expr> {
    match &expr {
        Expr::Lit(expr_lit) => match &expr_lit.lit {
            Lit::Int(_) | Lit::Float(_) | Lit::Bool(_) => Ok(expr),
            _ => Err(syn::Error::new_spanned(expr, "expected numeric literal")),
        },
        _ => Ok(expr),
    }
}

fn field_serde_rename(field: &Field) -> Option<String> {
    for attr in &field.attrs {
        if !attr.path().is_ident("serde") {
            continue;
        }
        if let Ok(metas) = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated) {
            for meta in metas {
                if let Meta::NameValue(nv) = meta {
                    if nv.path.is_ident("rename") {
                        if let Ok(s) = lit_str_from_expr(&nv.value, "rename") {
                            return Some(s.value());
                        }
                    }
                }
            }
        }
    }
    None
}

fn has_schemars_description(field: &Field) -> bool {
    field.attrs.iter().any(|attr| {
        if !attr.path().is_ident("schemars") {
            return false;
        }
        attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
            .map(|metas| {
                metas.into_iter().any(|m| match m {
                    Meta::NameValue(nv) => nv.path.is_ident("description"),
                    _ => false,
                })
            })
            .unwrap_or(false)
    })
}

fn doc_comment_description(field: &Field) -> Option<String> {
    let mut lines = Vec::new();
    for attr in &field.attrs {
        if !attr.path().is_ident("doc") {
            continue;
        }
        if let Meta::NameValue(MetaNameValue { value, .. }) = &attr.meta {
            if let Expr::Lit(expr_lit) = value {
                if let Lit::Str(s) = &expr_lit.lit {
                    lines.push(s.value().trim().to_string());
                }
            }
        }
    }
    let joined = lines
        .into_iter()
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    if joined.is_empty() {
        None
    } else {
        Some(joined)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FieldKind {
    String,
    OptionString,
    Other,
}

fn classify_field_kind(ty: &syn::Type) -> FieldKind {
    if is_string_type(ty) {
        return FieldKind::String;
    }
    if let Some(inner) = option_inner_type(ty) {
        if is_string_type(inner) {
            return FieldKind::OptionString;
        }
    }
    FieldKind::Other
}

fn option_inner_type<'a>(ty: &'a syn::Type) -> Option<&'a syn::Type> {
    let syn::Type::Path(type_path) = ty else {
        return None;
    };
    let segment = type_path.path.segments.last()?;
    if segment.ident != "Option" {
        return None;
    }
    let syn::PathArguments::AngleBracketed(args) = &segment.arguments else {
        return None;
    };
    let first = args.args.first()?;
    let syn::GenericArgument::Type(inner) = first else {
        return None;
    };
    Some(inner)
}

fn is_string_type(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Path(type_path) => type_path
            .path
            .segments
            .last()
            .map(|s| s.ident == "String")
            .unwrap_or(false),
        _ => false,
    }
}

fn ensure_string_like(field_kind: FieldKind, field_ident: &Ident, rule: &str) -> syn::Result<()> {
    if matches!(field_kind, FieldKind::String | FieldKind::OptionString) {
        Ok(())
    } else {
        Err(syn::Error::new_spanned(
            field_ident,
            format!(
                "rf {} currently supports String or Option<String> fields only",
                rule
            ),
        ))
    }
}

#[derive(Default)]
struct FieldRfConfig {
    length: Option<LengthArgs>,
    range: Option<RangeArgs>,
    email: bool,
    url: bool,
    required: bool,
    nested: bool,
    must_match_other: Option<String>,
    regex_pattern: Option<String>,
    contains_pattern: Option<String>,
    does_not_contain_pattern: Option<String>,
    builtin_rules: Vec<BuiltinRuleUse>,
    custom_rules: Vec<CustomRuleUse>,
    async_rules: Vec<AsyncDbRuleUse>,
    rule_overrides: BTreeMap<String, RuleMessageCode>,
    message: Option<String>,
    code: Option<String>,
    openapi_description: Option<String>,
    openapi_hint: Option<String>,
    openapi_example: Option<Expr>,
    openapi_format: Option<String>,
}

impl FieldRfConfig {
    fn message_code_for(&self, rule_key: &str) -> (Option<String>, Option<String>) {
        let scoped = self.rule_overrides.get(rule_key);
        let message = scoped
            .and_then(|v| v.message.clone())
            .or_else(|| self.message.clone());
        let code = scoped
            .and_then(|v| v.code.clone())
            .or_else(|| self.code.clone());
        (message, code)
    }
}

#[derive(Default)]
struct ContainerRfConfig {
    schema_rules: Vec<SchemaRuleRf>,
}

#[derive(Default, Clone)]
struct SchemaRuleRf {
    function: Option<Path>,
    use_context: Option<bool>,
    skip_on_field_errors: Option<bool>,
    message: Option<String>,
    code: Option<String>,
}

#[derive(Default, Clone)]
struct LengthArgs {
    min: Option<Expr>,
    max: Option<Expr>,
    equal: Option<Expr>,
}

impl LengthArgs {
    fn openapi_hint(&self) -> String {
        if self.equal.is_some() {
            "Fixed length required.".to_string()
        } else {
            match (self.min.as_ref(), self.max.as_ref()) {
                (Some(min), Some(max)) => format!(
                    "Length must be between {} and {}.",
                    min.to_token_stream(),
                    max.to_token_stream()
                ),
                (Some(min), None) => format!("Length must be at least {}.", min.to_token_stream()),
                (None, Some(max)) => format!("Length must be at most {}.", max.to_token_stream()),
                _ => "Length constrained.".to_string(),
            }
        }
    }
}

#[derive(Default, Clone)]
struct RangeArgs {
    min: Option<Expr>,
    max: Option<Expr>,
}

impl RangeArgs {
    fn openapi_hint(&self) -> String {
        match (self.min.as_ref(), self.max.as_ref()) {
            (Some(min), Some(max)) => format!(
                "Value must be between {} and {}.",
                min.to_token_stream(),
                max.to_token_stream()
            ),
            (Some(min), None) => format!("Value must be at least {}.", min.to_token_stream()),
            (None, Some(max)) => format!("Value must be at most {}.", max.to_token_stream()),
            _ => "Value range constrained.".to_string(),
        }
    }
}

#[derive(Clone)]
struct BuiltinRuleUse {
    key: String,
    values: Vec<String>,
    format: Option<String>,
    field: Option<String>,
}

#[derive(Clone)]
struct CustomRuleUse {
    function: Path,
    description: Option<String>,
    pattern: Option<String>,
    code: Option<String>,
    message: Option<String>,
}

#[derive(Clone)]
struct AsyncDbRuleUse {
    kind: AsyncDbRuleKind,
    table: String,
    column: String,
    modifiers: Vec<AsyncDbModifier>,
}

#[derive(Clone, Copy)]
enum AsyncDbRuleKind {
    Unique,
    Exists,
    NotExists,
}

#[derive(Clone)]
enum AsyncDbModifier {
    Ignore {
        column: String,
        source: AsyncDbValueSource,
    },
    WhereEq {
        column: String,
        source: AsyncDbValueSource,
    },
    WhereNotEq {
        column: String,
        source: AsyncDbValueSource,
    },
    WhereNull {
        column: String,
    },
    WhereNotNull {
        column: String,
    },
}

#[derive(Clone)]
enum AsyncDbValueSource {
    Field(String),
    Expr(Expr),
}

impl AsyncDbRuleKind {
    fn key(self) -> &'static str {
        match self {
            AsyncDbRuleKind::Unique => "async_unique",
            AsyncDbRuleKind::Exists => "async_exists",
            AsyncDbRuleKind::NotExists => "async_not_exists",
        }
    }

    fn default_code(self) -> &'static str {
        match self {
            AsyncDbRuleKind::Unique => "unique",
            AsyncDbRuleKind::Exists => "exists",
            AsyncDbRuleKind::NotExists => "not_exists",
        }
    }

    fn default_desc(self, table: &str, column: &str) -> String {
        match self {
            AsyncDbRuleKind::Unique => {
                format!("Must be unique in `{table}.{column}`.")
            }
            AsyncDbRuleKind::Exists => {
                format!("Must exist in `{table}.{column}`.")
            }
            AsyncDbRuleKind::NotExists => {
                format!("Must not exist in `{table}.{column}`.")
            }
        }
    }
}

#[derive(Default, Clone)]
struct RuleMessageCode {
    message: Option<String>,
    code: Option<String>,
}

#[derive(Clone)]
struct RuleExtensionSpec {
    key: String,
    source: String,
    params: BTreeMap<String, JsonParam>,
    default_message: Option<String>,
    description: Option<String>,
}

impl RuleExtensionSpec {
    fn simple_validator(key: &str) -> Self {
        Self {
            key: key.to_string(),
            source: "validator".to_string(),
            params: BTreeMap::new(),
            default_message: None,
            description: None,
        }
    }

    fn regex(pattern: &str) -> Self {
        let mut params = BTreeMap::new();
        params.insert(
            "pattern".to_string(),
            JsonParam::String(pattern.to_string()),
        );
        Self {
            key: "regex".to_string(),
            source: "validator".to_string(),
            params,
            default_message: None,
            description: Some(format!("Must match pattern `{}`.", pattern)),
        }
    }

    fn contains(pattern: &str) -> Self {
        let mut params = BTreeMap::new();
        params.insert(
            "pattern".to_string(),
            JsonParam::String(pattern.to_string()),
        );
        Self {
            key: "contains".to_string(),
            source: "validator".to_string(),
            params,
            default_message: None,
            description: Some(format!("Must contain `{}`.", pattern)),
        }
    }

    fn does_not_contain(pattern: &str) -> Self {
        let mut params = BTreeMap::new();
        params.insert(
            "pattern".to_string(),
            JsonParam::String(pattern.to_string()),
        );
        Self {
            key: "does_not_contain".to_string(),
            source: "validator".to_string(),
            params,
            default_message: None,
            description: Some(format!("Must not contain `{}`.", pattern)),
        }
    }

    fn must_match(other: &str) -> Self {
        let mut params = BTreeMap::new();
        params.insert("other".to_string(), JsonParam::String(other.to_string()));
        Self {
            key: "must_match".to_string(),
            source: "validator".to_string(),
            params,
            default_message: None,
            description: Some(format!("Must match `{other}`.")),
        }
    }

    fn from_length(length: &LengthArgs) -> Self {
        let mut params = BTreeMap::new();
        if let Some(min) = &length.min {
            params.insert("min".to_string(), expr_to_json_param(min));
        }
        if let Some(max) = &length.max {
            params.insert("max".to_string(), expr_to_json_param(max));
        }
        if let Some(equal) = &length.equal {
            params.insert("equal".to_string(), expr_to_json_param(equal));
        }
        Self {
            key: "length".to_string(),
            source: "validator".to_string(),
            params,
            default_message: None,
            description: None,
        }
    }

    fn from_range(range: &RangeArgs) -> Self {
        let mut params = BTreeMap::new();
        if let Some(min) = &range.min {
            params.insert("min".to_string(), expr_to_json_param(min));
        }
        if let Some(max) = &range.max {
            params.insert("max".to_string(), expr_to_json_param(max));
        }
        Self {
            key: "range".to_string(),
            source: "validator".to_string(),
            params,
            default_message: None,
            description: None,
        }
    }

    fn builtin(builtin: &BuiltinRuleUse, default_message: String, description: String) -> Self {
        let mut params = BTreeMap::new();
        if let Some(format) = &builtin.format {
            params.insert("format".to_string(), JsonParam::String(format.clone()));
        }
        if let Some(field) = &builtin.field {
            params.insert("field".to_string(), JsonParam::String(field.clone()));
        }
        if !builtin.values.is_empty() {
            params.insert(
                "values".to_string(),
                JsonParam::StringList(builtin.values.clone()),
            );
        }
        Self {
            key: builtin.key.clone(),
            source: "rustforge".to_string(),
            params,
            default_message: Some(default_message),
            description: Some(description),
        }
    }

    fn async_db(rule: &AsyncDbRuleUse) -> Self {
        let mut params = BTreeMap::new();
        params.insert("table".to_string(), JsonParam::String(rule.table.clone()));
        params.insert("column".to_string(), JsonParam::String(rule.column.clone()));
        Self {
            key: rule.kind.key().to_string(),
            source: "rustforge_async".to_string(),
            params,
            default_message: None,
            description: Some(rule.kind.default_desc(&rule.table, &rule.column)),
        }
    }
}

#[derive(Clone)]
enum JsonParam {
    String(String),
    I64(i64),
    Bool(bool),
    StringList(Vec<String>),
}

fn expr_to_json_param(expr: &Expr) -> JsonParam {
    if let Expr::Lit(expr_lit) = expr {
        match &expr_lit.lit {
            Lit::Int(i) => return JsonParam::I64(i.base10_parse::<i64>().unwrap_or_default()),
            Lit::Bool(LitBool { value, .. }) => return JsonParam::Bool(*value),
            Lit::Str(s) => return JsonParam::String(s.value()),
            Lit::Float(LitFloat { .. }) => {
                return JsonParam::String(expr.to_token_stream().to_string())
            }
            _ => {}
        }
    }
    JsonParam::String(expr.to_token_stream().to_string())
}
