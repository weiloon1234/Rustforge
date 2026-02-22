use std::collections::BTreeMap;

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote, ToTokens};
use rustforge_contract_meta::{builtin_rule_meta, render_template, BuiltinRuleKind};
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

    let mut container_attrs_for_shadow: Vec<Attribute> = Vec::new();
    let mut original_container_attrs: Vec<Attribute> = Vec::new();
    for attr in item.attrs.iter() {
        if attr.path().is_ident("rustforge_contract") {
            continue;
        }
        if attr.path().is_ident("derive") {
            if let Some(derive_attr) = strip_jsonschema_from_derive_attr(attr)? {
                original_container_attrs.push(derive_attr);
            }
            continue;
        }
        if attr.path().is_ident("schemars") {
            container_attrs_for_shadow.push(attr.clone());
            continue;
        }
        original_container_attrs.push(attr.clone());
        container_attrs_for_shadow.push(attr.clone());
    }
    item.attrs = original_container_attrs;

    let mut original_fields_tokens = Vec::new();
    let mut shadow_fields_tokens = Vec::new();
    let mut patch_blocks = Vec::new();
    let mut helper_fns = Vec::new();

    for (idx, field) in named_fields.iter().enumerate() {
        let field_ident = field.ident.clone().expect("named field");
        let field_ty = field.ty.clone();
        let field_vis = field.vis.clone();

        let (rf_cfg, non_rf_attrs, rf_attrs_present) = parse_rf_field(field)?;

        let mut original_field_attrs: Vec<Attribute> = Vec::new();
        let mut shadow_field_attrs: Vec<Attribute> = Vec::new();
        for attr in non_rf_attrs {
            if attr.path().is_ident("schemars") {
                shadow_field_attrs.push(attr);
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
        if let Some(length) = &rf_cfg.length {
            generated_validate_attrs.push(build_validate_length_attr(length, &rf_cfg)?);
            field_rule_extensions.push(RuleExtensionSpec::from_length(length));
            field_desc_parts.push(length.openapi_hint());
        }

        if let Some(range) = &rf_cfg.range {
            generated_validate_attrs.push(build_validate_range_attr(range, &rf_cfg)?);
            field_rule_extensions.push(RuleExtensionSpec::from_range(range));
            field_desc_parts.push(range.openapi_hint());
        }

        if rf_cfg.email {
            generated_validate_attrs.push(build_validate_simple_attr("email", &rf_cfg)?);
            field_rule_extensions.push(RuleExtensionSpec::simple_validator("email"));
            field_desc_parts.push("Must be a valid email address.".to_string());
        }

        if rf_cfg.url {
            generated_validate_attrs.push(build_validate_simple_attr("url", &rf_cfg)?);
            field_rule_extensions.push(RuleExtensionSpec::simple_validator("url"));
            field_desc_parts.push("Must be a valid URL.".to_string());
        }

        if rf_cfg.required {
            generated_validate_attrs.push(build_validate_simple_attr("required", &rf_cfg)?);
            field_rule_extensions.push(RuleExtensionSpec::simple_validator("required"));
            field_desc_parts.push("This field is required.".to_string());
        }

        if let Some(pattern) = &rf_cfg.regex_pattern {
            let helper_ident = format_ident!(
                "__rf_contract_{}_{}_regex_{}",
                struct_ident.to_string().to_lowercase(),
                field_ident,
                idx
            );
            helper_fns.push(generate_regex_wrapper_fn(&helper_ident, pattern));
            generated_validate_attrs.push(build_validate_custom_fn_attr(
                &helper_ident,
                &rf_cfg.message,
                &rf_cfg.code,
            )?);
            generated_shadow_schemars_attrs.push(build_schemars_regex_attr(pattern)?);
            field_rule_extensions.push(RuleExtensionSpec::regex(pattern));
            field_desc_parts.push(format!("Must match pattern `{}`.", pattern));
            field_pattern_patch = Some(pattern.clone());
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
                    let path: Path = syn::parse_str(path_str)?;
                    generated_validate_attrs.push(build_validate_custom_path_attr(
                        &path,
                        &rf_cfg.message,
                        &rf_cfg.code,
                    )?);
                }
                BuiltinRuleKind::PhoneNumberByIso2Field => {
                    let field_name = builtin.field.as_ref().ok_or_else(|| {
                        syn::Error::new_spanned(
                            &field_ident,
                            "#[rf(rule = \"phonenumber\", field = \"...\")] requires field",
                        )
                    })?;
                    let other_ident = Ident::new(field_name, Span::call_site());
                    generated_validate_attrs.push(build_validate_phonenumber_attr(
                        &other_ident,
                        &rf_cfg.message,
                        &rf_cfg.code,
                    )?);
                }
                BuiltinRuleKind::GeneratedOneOf => {
                    if builtin.values.is_empty() {
                        return Err(syn::Error::new_spanned(
                            &field_ident,
                            "#[rf(rule = \"one_of\", values(...))] requires at least one value",
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
                    generated_validate_attrs.push(build_validate_custom_fn_attr(
                        &helper_ident,
                        &rf_cfg.message,
                        &rf_cfg.code,
                    )?);
                    field_enum_values_patch = Some(builtin.values.clone());
                }
                BuiltinRuleKind::GeneratedNoneOf => {
                    if builtin.values.is_empty() {
                        return Err(syn::Error::new_spanned(
                            &field_ident,
                            "#[rf(rule = \"none_of\", values(...))] requires at least one value",
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
                    generated_validate_attrs.push(build_validate_custom_fn_attr(
                        &helper_ident,
                        &rf_cfg.message,
                        &rf_cfg.code,
                    )?);
                }
                BuiltinRuleKind::GeneratedDate | BuiltinRuleKind::GeneratedDateTime => {
                    let format = builtin.format.clone().ok_or_else(|| {
                        syn::Error::new_spanned(
                            &field_ident,
                            format!(
                                "#[rf(rule = \"{}\", format = \"...\")] requires format",
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
                    generated_validate_attrs.push(build_validate_custom_fn_attr(
                        &helper_ident,
                        &rf_cfg.message,
                        &rf_cfg.code,
                    )?);
                }
            }

            field_rule_extensions.push(RuleExtensionSpec::builtin(
                builtin,
                meta.default_message.to_string(),
                default_desc,
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
                .map(|ex| quote! { #ex.to_string() });
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

    let original_attrs = item.attrs.clone();
    let original_generics = item.generics.clone();

    let expanded = quote! {
        #(#helper_fns)*

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
    rf_cfg: &FieldRfConfig,
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
    if let Some(msg) = &rf_cfg.message {
        nested.push(quote! { message = #msg });
    }
    if let Some(code) = &rf_cfg.code {
        nested.push(quote! { code = #code });
    }
    mk_attr(quote! { #[validate(length(#(#nested),*))] })
}

fn build_validate_range_attr(range: &RangeArgs, rf_cfg: &FieldRfConfig) -> syn::Result<Attribute> {
    let mut nested = Vec::<TokenStream2>::new();
    if let Some(min) = &range.min {
        nested.push(quote! { min = #min });
    }
    if let Some(max) = &range.max {
        nested.push(quote! { max = #max });
    }
    if let Some(msg) = &rf_cfg.message {
        nested.push(quote! { message = #msg });
    }
    if let Some(code) = &rf_cfg.code {
        nested.push(quote! { code = #code });
    }
    mk_attr(quote! { #[validate(range(#(#nested),*))] })
}

fn build_validate_simple_attr(name: &str, rf_cfg: &FieldRfConfig) -> syn::Result<Attribute> {
    let ident = Ident::new(name, Span::call_site());
    let mut nested = Vec::<TokenStream2>::new();
    if let Some(msg) = &rf_cfg.message {
        nested.push(quote! { message = #msg });
    }
    if let Some(code) = &rf_cfg.code {
        nested.push(quote! { code = #code });
    }
    if nested.is_empty() {
        mk_attr(quote! { #[validate(#ident)] })
    } else {
        mk_attr(quote! { #[validate(#ident(#(#nested),*))] })
    }
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
        let mut pending_builtin: Option<BuiltinRuleUse> = None;
        let mut local_length: Option<LengthArgs> = None;
        let mut local_range: Option<RangeArgs> = None;
        let mut local_regex: Option<String> = None;
        let mut local_email = false;
        let mut local_url = false;
        let mut local_required = false;
        let mut local_values: Option<Vec<String>> = None;

        for meta in metas {
            match meta {
                Meta::Path(path) if path.is_ident("email") => {
                    local_email = true;
                }
                Meta::Path(path) if path.is_ident("url") => {
                    local_url = true;
                }
                Meta::Path(path) if path.is_ident("required") => {
                    local_required = true;
                }
                Meta::List(list) if list.path.is_ident("length") => {
                    if local_length.is_some() || cfg.length.is_some() {
                        return Err(syn::Error::new_spanned(list, "duplicate rf length"));
                    }
                    local_length = Some(parse_length_args(&list)?);
                }
                Meta::List(list) if list.path.is_ident("range") => {
                    if local_range.is_some() || cfg.range.is_some() {
                        return Err(syn::Error::new_spanned(list, "duplicate rf range"));
                    }
                    local_range = Some(parse_range_args(&list)?);
                }
                Meta::List(list) if list.path.is_ident("regex") => {
                    if local_regex.is_some() || cfg.regex_pattern.is_some() {
                        return Err(syn::Error::new_spanned(list, "duplicate rf regex"));
                    }
                    local_regex = Some(parse_regex_pattern(&list)?);
                }
                Meta::List(list) if list.path.is_ident("values") => {
                    local_values = Some(parse_values_list(&list)?);
                }
                Meta::NameValue(nv) if nv.path.is_ident("rule") => {
                    let key = lit_str_from_expr(&nv.value, "rule")?.value();
                    pending_builtin = Some(BuiltinRuleUse {
                        key,
                        values: Vec::new(),
                        format: None,
                        field: None,
                    });
                }
                Meta::NameValue(nv) if nv.path.is_ident("format") => {
                    let value = lit_str_from_expr(&nv.value, "format")?.value();
                    if let Some(rule) = pending_builtin.as_mut() {
                        rule.format = Some(value);
                    } else {
                        cfg.openapi_format = Some(value);
                    }
                }
                Meta::NameValue(nv) if nv.path.is_ident("field") => {
                    let value = lit_str_from_expr(&nv.value, "field")?.value();
                    if let Some(rule) = pending_builtin.as_mut() {
                        rule.field = Some(value);
                    } else {
                        return Err(syn::Error::new_spanned(
                            nv,
                            "rf field = ... is only valid with rf(rule = \"phonenumber\")",
                        ));
                    }
                }
                Meta::NameValue(nv) if nv.path.is_ident("message") => {
                    cfg.message = Some(lit_str_from_expr(&nv.value, "message")?.value());
                }
                Meta::NameValue(nv) if nv.path.is_ident("code") => {
                    cfg.code = Some(lit_str_from_expr(&nv.value, "code")?.value());
                }
                Meta::NameValue(nv) if nv.path.is_ident("openapi_description") => {
                    cfg.openapi_description =
                        Some(lit_str_from_expr(&nv.value, "openapi_description")?.value());
                }
                Meta::NameValue(nv) if nv.path.is_ident("openapi_hint") => {
                    cfg.openapi_hint = Some(lit_str_from_expr(&nv.value, "openapi_hint")?.value());
                }
                Meta::NameValue(nv) if nv.path.is_ident("openapi_example") => {
                    cfg.openapi_example =
                        Some(lit_str_from_expr(&nv.value, "openapi_example")?.value());
                }
                Meta::NameValue(nv) if nv.path.is_ident("openapi_format") => {
                    cfg.openapi_format =
                        Some(lit_str_from_expr(&nv.value, "openapi_format")?.value());
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
        cfg.email |= local_email;
        cfg.url |= local_url;
        cfg.required |= local_required;

        if let Some(mut builtin) = pending_builtin {
            if let Some(values) = local_values {
                builtin.values = values;
            }
            cfg.builtin_rules.push(builtin);
        } else if local_values.is_some() {
            return Err(syn::Error::new_spanned(
                attr,
                "rf values(...) requires rf(rule = \"one_of\"|\"none_of\") in the same attribute",
            ));
        }
    }

    if cfg.regex_pattern.is_some() && cfg.builtin_rules.iter().any(|r| r.key == "one_of") {
        return Err(syn::Error::new_spanned(
            field,
            "rf regex(...) cannot be combined with rf(rule = \"one_of\") on the same field",
        ));
    }

    Ok((cfg, keep_attrs, rf_attr_tokens))
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

fn parse_values_list(list: &MetaList) -> syn::Result<Vec<String>> {
    let exprs = list.parse_args_with(Punctuated::<Expr, Token![,]>::parse_terminated)?;
    let mut out = Vec::new();
    for expr in exprs {
        let lit = lit_str_from_expr(&expr, "values")?;
        out.push(lit.value());
    }
    Ok(out)
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

fn strip_jsonschema_from_derive_attr(attr: &Attribute) -> syn::Result<Option<Attribute>> {
    let metas = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
    let kept: Vec<Meta> = metas
        .into_iter()
        .filter(|m| match m {
            Meta::Path(p) => !path_ends_with_ident(p, "JsonSchema"),
            _ => true,
        })
        .collect();
    if kept.is_empty() {
        return Ok(None);
    }
    let tokens = quote! { #[derive(#(#kept),*)] };
    mk_attr(tokens).map(Some)
}

fn path_ends_with_ident(path: &Path, ident: &str) -> bool {
    path.segments
        .last()
        .map(|s| s.ident == ident)
        .unwrap_or(false)
}

#[derive(Default)]
struct FieldRfConfig {
    length: Option<LengthArgs>,
    range: Option<RangeArgs>,
    email: bool,
    url: bool,
    required: bool,
    regex_pattern: Option<String>,
    builtin_rules: Vec<BuiltinRuleUse>,
    message: Option<String>,
    code: Option<String>,
    openapi_description: Option<String>,
    openapi_hint: Option<String>,
    openapi_example: Option<String>,
    openapi_format: Option<String>,
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
