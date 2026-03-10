use crate::config::ConfigsFile;
use crate::schema::{
    parse_attachments, parse_computed, parse_fields, parse_meta, parse_relations, to_snake,
    to_title_case, AttachmentFieldSpec, EnumOrOther, FieldSpec, MetaFieldSpec, MetaType, ModelSpec,
    RelationKind, RelationSpec, Schema, SpecialType,
};
use crate::template::{render_template, TemplateContext};
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt::Write;
use std::fs;
use std::path::Path;

const DATATABLE_REL_FILTER_MAX_DEPTH: usize = 3;

#[derive(Debug, Clone, Copy)]
pub struct GenerateModelsOptions {
    pub include_datatable: bool,
    pub include_extensions_imports: bool,
}

impl Default for GenerateModelsOptions {
    fn default() -> Self {
        Self {
            include_datatable: true,
            include_extensions_imports: true,
        }
    }
}

#[derive(Debug, Clone)]
struct RelationPathSpec {
    path: Vec<String>,
    target_model: String,
}

#[derive(Debug, Clone)]
struct EnumExplainedFieldSpec {
    name: String,
    explained_name: String,
    optional: bool,
}

fn parse_option_inner_type(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if !trimmed.starts_with("Option<") || !trimmed.ends_with('>') {
        return None;
    }
    let inner = trimmed
        .strip_prefix("Option<")
        .and_then(|value| value.strip_suffix('>'))?
        .trim();
    if inner.is_empty() {
        None
    } else {
        Some(inner.to_string())
    }
}

fn enum_field_spec(
    field_name: &str,
    field_type: &str,
    enum_type_names: &BTreeSet<String>,
) -> Option<EnumExplainedFieldSpec> {
    if enum_type_names.contains(field_type.trim()) {
        return Some(EnumExplainedFieldSpec {
            name: field_name.to_string(),
            explained_name: format!("{field_name}_explained"),
            optional: false,
        });
    }

    let inner = parse_option_inner_type(field_type)?;
    if !enum_type_names.contains(inner.as_str()) {
        return None;
    }

    Some(EnumExplainedFieldSpec {
        name: field_name.to_string(),
        explained_name: format!("{field_name}_explained"),
        optional: true,
    })
}

fn collect_relation_paths(
    schema: &Schema,
    model_name: &str,
    max_depth: usize,
) -> Vec<RelationPathSpec> {
    if max_depth == 0 {
        return Vec::new();
    }

    fn walk(
        schema: &Schema,
        current_model: &str,
        path: &mut Vec<String>,
        out: &mut Vec<RelationPathSpec>,
        seen: &mut BTreeSet<String>,
        max_depth: usize,
    ) {
        if path.len() >= max_depth {
            return;
        }

        let Some(cfg) = schema.models.get(current_model) else {
            return;
        };

        let pk = cfg.pk.clone().unwrap_or_else(|| "id".to_string());
        let fields = parse_fields(cfg, &pk);
        let relations = parse_relations(schema, cfg, current_model, &fields);

        for rel in relations {
            let rel_snake = to_snake(&rel.name);
            path.push(rel_snake);
            let key = path.join("__");

            if seen.insert(key) {
                out.push(RelationPathSpec {
                    path: path.clone(),
                    target_model: rel.target_model.clone(),
                });
            }

            walk(schema, &rel.target_model, path, out, seen, max_depth);
            path.pop();
        }
    }

    let mut out = Vec::new();
    let mut seen = BTreeSet::new();
    let mut path = Vec::new();
    walk(
        schema, model_name, &mut path, &mut out, &mut seen, max_depth,
    );
    out
}

fn build_nested_where_has_expr(
    path: &[String],
    leaf_expr_template: &str,
    root_var: &str,
) -> String {
    fn render(path: &[String], leaf_expr_template: &str, current_var: &str) -> String {
        if path.is_empty() {
            return leaf_expr_template.replace("{var}", current_var);
        }
        let rel = &path[0];
        let nested = render(&path[1..], leaf_expr_template, "rq");
        format!("{current_var}.where_has_{rel}(|rq| {nested})")
    }

    render(path, leaf_expr_template, root_var)
}

fn render_query_field_where_methods(db_fields: &[FieldSpec], col_ident: &str) -> String {
    let mut out = String::new();
    for f in db_fields {
        let fn_name = format!("where_{}", to_snake(&f.name));
        writeln!(
            out,
            "    pub fn {fn_name}(mut self, op: Op, val: {typ}) -> Self {{",
            typ = f.ty
        )
        .unwrap();
        writeln!(out, "        let idx = self.binds.len() + 1;").unwrap();
        writeln!(
            out,
            "        self.where_sql.push(format!(\"{{}} {{}} ${{}}\", {col_ident}::{}.as_sql(), op.as_sql(), idx));",
            to_title_case(&f.name)
        )
        .unwrap();
        writeln!(out, "        self.binds.push(val.into());").unwrap();
        writeln!(out, "        self").unwrap();
        writeln!(out, "    }}").unwrap();

        let fn_name_raw = format!("where_{}_raw", to_snake(&f.name));
        writeln!(
            out,
            "    pub fn {fn_name_raw}<T: Into<BindValue>>(mut self, op: Op, val: T) -> Self {{"
        )
        .unwrap();
        writeln!(out, "        let idx = self.binds.len() + 1;").unwrap();
        writeln!(
            out,
            "        self.where_sql.push(format!(\"{{}} {{}} ${{}}\", {col_ident}::{}.as_sql(), op.as_sql(), idx));",
            to_title_case(&f.name)
        )
        .unwrap();
        writeln!(out, "        self.binds.push(val.into());").unwrap();
        writeln!(out, "        self").unwrap();
        writeln!(out, "    }}").unwrap();
    }
    out
}

fn render_query_relation_filter_methods(relations: &[RelationSpec], table: &str) -> String {
    let mut out = String::new();
    for rel in relations {
        let fn_name = format!("where_has_{}", to_snake(&rel.name));
        let target_query = format!("{}Query", to_title_case(&rel.target_model));
        let link_clause = match rel.kind {
            RelationKind::HasMany => format!(
                "{}.{} = {}.{}",
                rel.target_table, rel.foreign_key, table, rel.local_key
            ),
            RelationKind::BelongsTo => format!(
                "{}.{} = {}.{}",
                rel.target_table, rel.target_pk, table, rel.foreign_key
            ),
        };
        writeln!(
            out,
            "    pub fn {fn_name}(mut self, scope: impl FnOnce({target_query}<'db>) -> {target_query}<'db>) -> Self {{"
        )
        .unwrap();
        writeln!(out, "        let start_idx = self.binds.len() + 1;").unwrap();
        writeln!(
            out,
            "        let scoped = scope({target_query}::new(self.db.clone(), None));"
        )
        .unwrap();
        writeln!(
            out,
            "        let (mut sub_where, mut sub_binds) = scoped.into_where_parts();"
        )
        .unwrap();
        writeln!(
            out,
            "        sub_where.insert(0, \"{link}\".to_string());",
            link = link_clause
        )
        .unwrap();
        writeln!(
            out,
            "        let mut clause = String::from(\"EXISTS (SELECT 1 FROM {rel_table} WHERE \");",
            rel_table = rel.target_table
        )
        .unwrap();
        writeln!(out, "        clause.push_str(&sub_where.join(\" AND \"));").unwrap();
        writeln!(out, "        clause.push(')');").unwrap();
        writeln!(
            out,
            "        let clause = renumber_placeholders(&clause, start_idx);"
        )
        .unwrap();
        writeln!(out, "        self.where_sql.push(clause);").unwrap();
        writeln!(out, "        self.binds.extend(sub_binds);").unwrap();
        writeln!(out, "        self").unwrap();
        writeln!(out, "    }}").unwrap();
    }

    for rel in relations {
        let fn_name = format!("where_doesnt_have_{}", to_snake(&rel.name));
        let target_query = format!("{}Query", to_title_case(&rel.target_model));
        let link_clause = match rel.kind {
            RelationKind::HasMany => format!(
                "{}.{} = {}.{}",
                rel.target_table, rel.foreign_key, table, rel.local_key
            ),
            RelationKind::BelongsTo => format!(
                "{}.{} = {}.{}",
                rel.target_table, rel.target_pk, table, rel.foreign_key
            ),
        };
        writeln!(
            out,
            "    pub fn {fn_name}(mut self, scope: impl FnOnce({target_query}<'db>) -> {target_query}<'db>) -> Self {{"
        )
        .unwrap();
        writeln!(out, "        let start_idx = self.binds.len() + 1;").unwrap();
        writeln!(
            out,
            "        let scoped = scope({target_query}::new(self.db.clone(), None));"
        )
        .unwrap();
        writeln!(
            out,
            "        let (mut sub_where, mut sub_binds) = scoped.into_where_parts();"
        )
        .unwrap();
        writeln!(
            out,
            "        sub_where.insert(0, \"{link}\".to_string());",
            link = link_clause
        )
        .unwrap();
        writeln!(
            out,
            "        let mut clause = String::from(\"NOT EXISTS (SELECT 1 FROM {rel_table} WHERE \");",
            rel_table = rel.target_table
        )
        .unwrap();
        writeln!(out, "        clause.push_str(&sub_where.join(\" AND \"));").unwrap();
        writeln!(out, "        clause.push(')');").unwrap();
        writeln!(
            out,
            "        let clause = renumber_placeholders(&clause, start_idx);"
        )
        .unwrap();
        writeln!(out, "        self.where_sql.push(clause);").unwrap();
        writeln!(out, "        self.binds.extend(sub_binds);").unwrap();
        writeln!(out, "        self").unwrap();
        writeln!(out, "    }}").unwrap();
    }

    for rel in relations {
        let fn_name = format!("or_where_has_{}", to_snake(&rel.name));
        let target_query = format!("{}Query", to_title_case(&rel.target_model));
        let link_clause = match rel.kind {
            RelationKind::HasMany => format!(
                "{}.{} = {}.{}",
                rel.target_table, rel.foreign_key, table, rel.local_key
            ),
            RelationKind::BelongsTo => format!(
                "{}.{} = {}.{}",
                rel.target_table, rel.target_pk, table, rel.foreign_key
            ),
        };
        writeln!(
            out,
            "    pub fn {fn_name}(mut self, scope: impl FnOnce({target_query}<'db>) -> {target_query}<'db>) -> Self {{"
        )
        .unwrap();
        writeln!(out, "        let start_idx = self.binds.len() + 1;").unwrap();
        writeln!(
            out,
            "        let scoped = scope({target_query}::new(self.db.clone(), None));"
        )
        .unwrap();
        writeln!(
            out,
            "        let (mut sub_where, mut sub_binds) = scoped.into_where_parts();"
        )
        .unwrap();
        writeln!(
            out,
            "        sub_where.insert(0, \"{link}\".to_string());",
            link = link_clause
        )
        .unwrap();
        writeln!(
            out,
            "        let mut clause = String::from(\"EXISTS (SELECT 1 FROM {rel_table} WHERE \");",
            rel_table = rel.target_table
        )
        .unwrap();
        writeln!(out, "        clause.push_str(&sub_where.join(\" AND \"));").unwrap();
        writeln!(out, "        clause.push(')');").unwrap();
        writeln!(
            out,
            "        let clause = renumber_placeholders(&clause, start_idx);"
        )
        .unwrap();
        writeln!(out, "        if let Some(last) = self.where_sql.pop() {{").unwrap();
        writeln!(
            out,
            "            self.where_sql.push(format!(\"({{}} OR {{}})\", last, clause));"
        )
        .unwrap();
        writeln!(out, "        }} else {{").unwrap();
        writeln!(out, "            self.where_sql.push(clause);").unwrap();
        writeln!(out, "        }}").unwrap();
        writeln!(out, "        self.binds.extend(sub_binds);").unwrap();
        writeln!(out, "        self").unwrap();
        writeln!(out, "    }}").unwrap();
    }
    out
}

fn render_query_select_method(col_ident: &str) -> String {
    let mut out = String::new();
    writeln!(
        out,
        "    pub fn select(mut self, cols: &[{col_ident}]) -> Self {{"
    )
    .unwrap();
    writeln!(
        out,
        "        let names: Vec<&str> = cols.iter().map(|c| c.as_sql()).collect();"
    )
    .unwrap();
    writeln!(out, "        self.select_sql = Some(names.join(\", \"));").unwrap();
    writeln!(out, "        self").unwrap();
    writeln!(out, "    }}").unwrap();
    out
}

fn render_simple_join_method(method_name: &str, sql_keyword: &str) -> String {
    let mut out = String::new();
    writeln!(
        out,
        "    fn {method_name}(mut self, table: &str, first: &str, op: &str, second: &str) -> Self {{"
    )
    .unwrap();
    writeln!(
        out,
        "        self.join_sql.push(format!(\"{sql_keyword} {{}} ON {{}} {{}} {{}}\", table, first, op, second));"
    )
    .unwrap();
    writeln!(out, "        self").unwrap();
    writeln!(out, "    }}").unwrap();
    out
}

fn render_query_source_methods() -> String {
    let mut out = String::new();
    writeln!(out, "    fn from_raw(mut self, sql: &str) -> Self {{").unwrap();
    writeln!(out, "        self.from_sql = Some(sql.to_string());").unwrap();
    writeln!(out, "        self").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(out, "    fn count_sql(mut self, sql: &str) -> Self {{").unwrap();
    writeln!(out, "        self.count_sql = Some(sql.to_string());").unwrap();
    writeln!(out, "        self").unwrap();
    writeln!(out, "    }}").unwrap();
    out
}

fn render_where_exists_method() -> String {
    let mut out = String::new();
    writeln!(
        out,
        "    fn where_exists<T: Into<BindValue>>(mut self, clause: impl Into<String>, binds: impl IntoIterator<Item = T>) -> Self {{"
    )
    .unwrap();
    writeln!(out, "        let mut clause = clause.into();").unwrap();
    writeln!(
        out,
        "        let incoming: Vec<BindValue> = binds.into_iter().map(Into::into).collect();"
    )
    .unwrap();
    writeln!(out, "        let mut idx = self.binds.len() + 1;").unwrap();
    writeln!(out, "        while let Some(pos) = clause.find('?') {{").unwrap();
    writeln!(out, "            let ph = format!(\"${{}}\", idx);").unwrap();
    writeln!(out, "            clause.replace_range(pos..pos + 1, &ph);").unwrap();
    writeln!(out, "            idx += 1;").unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(
        out,
        "        self.where_sql.push(format!(\"EXISTS ({{}})\", clause));"
    )
    .unwrap();
    writeln!(out, "        self.binds.extend(incoming);").unwrap();
    writeln!(out, "        self").unwrap();
    writeln!(out, "    }}").unwrap();
    out
}

fn render_select_subquery_method() -> String {
    let mut out = String::new();
    writeln!(
        out,
        "    fn select_subquery(mut self, alias: &str, sql: &str) -> Self {{"
    )
    .unwrap();
    writeln!(
        out,
        "        let current = self.select_sql.get_or_insert_with(|| \"*\".to_string());"
    )
    .unwrap();
    writeln!(
        out,
        "        current.push_str(&format!(\", ({{}}) AS {{}}\", sql, alias));"
    )
    .unwrap();
    writeln!(out, "        self").unwrap();
    writeln!(out, "    }}").unwrap();
    out
}

fn render_select_list_methods(col_ident: &str, base_select: &str) -> String {
    let mut out = String::new();
    writeln!(
        out,
        "    pub fn select_cols(mut self, cols: &[{col_ident}]) -> Self {{"
    )
    .unwrap();
    writeln!(out, "        if cols.is_empty() {{").unwrap();
    writeln!(
        out,
        "            self.select_sql = Some(\"{base_select}\".to_string());"
    )
    .unwrap();
    writeln!(out, "        }} else {{").unwrap();
    writeln!(
        out,
        "            let mut seen = std::collections::BTreeSet::new();"
    )
    .unwrap();
    writeln!(
        out,
        "            let mut list: Vec<String> = \"{base_select}\".split(',').map(|s| s.trim().to_string()).collect();"
    )
    .unwrap();
    writeln!(
        out,
        "            for s in &list {{ seen.insert(s.clone()); }}"
    )
    .unwrap();
    writeln!(
        out,
        "            for c in cols {{ let s = c.as_sql().to_string(); if seen.insert(s.clone()) {{ list.push(s); }} }}"
    )
    .unwrap();
    writeln!(
        out,
        "            self.select_sql = Some(list.join(\", \"));"
    )
    .unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(out, "        self").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(
        out,
        "    pub fn add_select_cols(mut self, cols: &[{col_ident}]) -> Self {{"
    )
    .unwrap();
    writeln!(
        out,
        "        let mut seen = std::collections::BTreeSet::new();"
    )
    .unwrap();
    writeln!(
        out,
        "        let mut list: Vec<String> = match self.select_sql.take() {{"
    )
    .unwrap();
    writeln!(
        out,
        "            Some(s) if !s.is_empty() => s.split(',').map(|s| s.trim().to_string()).collect(),"
    )
    .unwrap();
    writeln!(
        out,
        "            _ => \"{base_select}\".split(',').map(|s| s.trim().to_string()).collect(),"
    )
    .unwrap();
    writeln!(out, "        }};").unwrap();
    writeln!(out, "        for s in &list {{ seen.insert(s.clone()); }}").unwrap();
    writeln!(
        out,
        "        for c in cols {{ let s = c.as_sql().to_string(); if seen.insert(s.clone()) {{ list.push(s); }} }}"
    )
    .unwrap();
    writeln!(out, "        self.select_sql = Some(list.join(\", \"));").unwrap();
    writeln!(out, "        self").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(
        out,
        "    fn select_raw(mut self, sql: impl Into<String>) -> Self {{"
    )
    .unwrap();
    writeln!(out, "        let s = sql.into();").unwrap();
    writeln!(out, "        if s.is_empty() {{").unwrap();
    writeln!(
        out,
        "            self.select_sql = Some(\"{base_select}\".to_string());"
    )
    .unwrap();
    writeln!(out, "        }} else {{").unwrap();
    writeln!(
        out,
        "            self.select_sql = Some(format!(\"{base_select}, {{}}\", s));"
    )
    .unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(out, "        self").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(
        out,
        "    fn add_select_raw(mut self, sql: impl Into<String>) -> Self {{"
    )
    .unwrap();
    writeln!(out, "        let s = sql.into();").unwrap();
    writeln!(out, "        if s.is_empty() {{ return self; }}").unwrap();
    writeln!(
        out,
        "        let mut base = self.select_sql.take().unwrap_or_else(|| \"{base_select}\".to_string());"
    )
    .unwrap();
    writeln!(
        out,
        "        if !base.is_empty() {{ base.push_str(\", \"); }}"
    )
    .unwrap();
    writeln!(out, "        base.push_str(&s);").unwrap();
    writeln!(out, "        self.select_sql = Some(base);").unwrap();
    writeln!(out, "        self").unwrap();
    writeln!(out, "    }}").unwrap();
    out
}

fn render_raw_join_method(method_name: &str, sql_keyword: &str) -> String {
    let mut out = String::new();
    writeln!(
        out,
        "    fn {method_name}<T: Into<BindValue>>(mut self, table: impl Into<String>, on_clause: impl Into<String>, binds: impl IntoIterator<Item = T>) -> Self {{"
    )
    .unwrap();
    writeln!(
        out,
        "        let mut clause = format!(\"{sql_keyword} {{}} ON {{}}\", table.into(), on_clause.into());"
    )
    .unwrap();
    writeln!(
        out,
        "        let mut incoming: Vec<BindValue> = binds.into_iter().map(Into::into).collect();"
    )
    .unwrap();
    writeln!(
        out,
        "        let mut idx = self.join_binds.len() + self.binds.len() + 1;"
    )
    .unwrap();
    writeln!(out, "        while let Some(pos) = clause.find('?') {{").unwrap();
    writeln!(out, "            let ph = format!(\"${{}}\", idx);").unwrap();
    writeln!(out, "            clause.replace_range(pos..pos+1, &ph);").unwrap();
    writeln!(out, "            idx += 1;").unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(out, "        self.join_sql.push(clause);").unwrap();
    writeln!(out, "        self.join_binds.append(&mut incoming);").unwrap();
    writeln!(out, "        self").unwrap();
    writeln!(out, "    }}").unwrap();
    out
}

fn render_group_by_method(col_ident: &str) -> String {
    let mut out = String::new();
    writeln!(
        out,
        "    pub fn group_by(mut self, cols: &[{col_ident}]) -> Self {{"
    )
    .unwrap();
    writeln!(out, "        for c in cols {{").unwrap();
    writeln!(
        out,
        "            self.group_by_sql.push(c.as_sql().to_string());"
    )
    .unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(out, "        self").unwrap();
    writeln!(out, "    }}").unwrap();
    out
}

fn render_having_raw_method() -> String {
    let mut out = String::new();
    writeln!(
        out,
        "    pub fn having_raw<T: Into<BindValue>>(mut self, clause: impl Into<String>, binds: impl IntoIterator<Item = T>) -> Self {{"
    )
    .unwrap();
    writeln!(out, "        let mut clause = clause.into();").unwrap();
    writeln!(
        out,
        "        let incoming: Vec<BindValue> = binds.into_iter().map(Into::into).collect();"
    )
    .unwrap();
    writeln!(out, "        let mut idx = self.having_binds.len() + 1;").unwrap();
    writeln!(out, "        while let Some(pos) = clause.find('?') {{").unwrap();
    writeln!(out, "            let ph = format!(\"${{}}\", idx);").unwrap();
    writeln!(out, "            clause.replace_range(pos..pos + 1, &ph);").unwrap();
    writeln!(out, "            idx += 1;").unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(out, "        self.having_sql.push(clause);").unwrap();
    writeln!(out, "        self.having_binds.extend(incoming);").unwrap();
    writeln!(out, "        self").unwrap();
    writeln!(out, "    }}").unwrap();
    out
}

fn render_limit_offset_methods() -> String {
    let mut out = String::new();
    writeln!(out, "    pub fn limit(mut self, n: i64) -> Self {{").unwrap();
    writeln!(out, "        self.limit = Some(n);").unwrap();
    writeln!(out, "        self").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(out, "    pub fn offset(mut self, n: i64) -> Self {{").unwrap();
    writeln!(out, "        self.offset = Some(n);").unwrap();
    writeln!(out, "        self").unwrap();
    writeln!(out, "    }}").unwrap();
    out
}

fn render_null_check_methods(col_ident: &str) -> String {
    let mut out = String::new();
    writeln!(
        out,
        "    pub fn where_null(mut self, col: {col_ident}) -> Self {{"
    )
    .unwrap();
    writeln!(
        out,
        "        self.where_sql.push(format!(\"{{}} IS NULL\", col.as_sql()));"
    )
    .unwrap();
    writeln!(out, "        self").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(
        out,
        "    pub fn where_not_null(mut self, col: {col_ident}) -> Self {{"
    )
    .unwrap();
    writeln!(
        out,
        "        self.where_sql.push(format!(\"{{}} IS NOT NULL\", col.as_sql()));"
    )
    .unwrap();
    writeln!(out, "        self").unwrap();
    writeln!(out, "    }}").unwrap();
    out
}

fn render_where_set_methods(col_ident: &str) -> String {
    let mut out = String::new();
    writeln!(
        out,
        "    pub fn where_in<T: Clone + Into<BindValue>>(mut self, col: {col_ident}, vals: &[T]) -> Self {{"
    )
    .unwrap();
    writeln!(out, "        if vals.is_empty() {{").unwrap();
    writeln!(out, "            self.where_sql.push(\"1=0\".to_string());").unwrap();
    writeln!(out, "            return self;").unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(out, "        let start = self.binds.len() + 1;").unwrap();
    writeln!(
        out,
        "        let mut placeholders = Vec::with_capacity(vals.len());"
    )
    .unwrap();
    writeln!(out, "        for (i, v) in vals.iter().enumerate() {{").unwrap();
    writeln!(
        out,
        "            placeholders.push(format!(\"${{}}\", start + i));"
    )
    .unwrap();
    writeln!(out, "            self.binds.push(v.clone().into());").unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(
        out,
        "        let clause = format!(\"{{}} IN ({{}})\", col.as_sql(), placeholders.join(\", \"));"
    )
    .unwrap();
    writeln!(out, "        self.where_sql.push(clause);").unwrap();
    writeln!(out, "        self").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(
        out,
        "    pub fn where_not_in<T: Clone + Into<BindValue>>(mut self, col: {col_ident}, vals: &[T]) -> Self {{"
    )
    .unwrap();
    writeln!(out, "        if vals.is_empty() {{ return self; }}").unwrap();
    writeln!(out, "        let start = self.binds.len() + 1;").unwrap();
    writeln!(
        out,
        "        let mut placeholders = Vec::with_capacity(vals.len());"
    )
    .unwrap();
    writeln!(out, "        for (i, v) in vals.iter().enumerate() {{").unwrap();
    writeln!(
        out,
        "            placeholders.push(format!(\"${{}}\", start + i));"
    )
    .unwrap();
    writeln!(out, "            self.binds.push(v.clone().into());").unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(
        out,
        "        let clause = format!(\"{{}} NOT IN ({{}})\", col.as_sql(), placeholders.join(\", \"));"
    )
    .unwrap();
    writeln!(out, "        self.where_sql.push(clause);").unwrap();
    writeln!(out, "        self").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(
        out,
        "    pub fn where_between<T: Into<BindValue>>(mut self, col: {col_ident}, low: T, high: T) -> Self {{"
    )
    .unwrap();
    writeln!(out, "        let idx1 = self.binds.len() + 1;").unwrap();
    writeln!(out, "        let idx2 = idx1 + 1;").unwrap();
    writeln!(
        out,
        "        self.where_sql.push(format!(\"{{}} BETWEEN ${{}} AND ${{}}\", col.as_sql(), idx1, idx2));"
    )
    .unwrap();
    writeln!(out, "        self.binds.push(low.into());").unwrap();
    writeln!(out, "        self.binds.push(high.into());").unwrap();
    writeln!(out, "        self").unwrap();
    writeln!(out, "    }}").unwrap();
    out
}

fn render_or_where_methods(col_ident: &str) -> String {
    let mut out = String::new();
    writeln!(
        out,
        "    pub fn or_where_col<T: Into<BindValue>>(mut self, col: {col_ident}, op: Op, val: T) -> Self {{"
    )
    .unwrap();
    writeln!(out, "        let idx = self.binds.len() + 1;").unwrap();
    writeln!(
        out,
        "        let clause = format!(\"{{}} {{}} ${{}}\", col.as_sql(), op.as_sql(), idx);"
    )
    .unwrap();
    writeln!(out, "        if let Some(last) = self.where_sql.pop() {{").unwrap();
    writeln!(
        out,
        "            self.where_sql.push(format!(\"({{}} OR {{}})\", last, clause));"
    )
    .unwrap();
    writeln!(out, "        }} else {{").unwrap();
    writeln!(out, "            self.where_sql.push(clause);").unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(out, "        self.binds.push(val.into());").unwrap();
    writeln!(out, "        self").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(
        out,
        "    fn or_where_raw<T: Into<BindValue>>(mut self, clause: impl Into<String>, binds: impl IntoIterator<Item = T>) -> Self {{"
    )
    .unwrap();
    writeln!(out, "        let mut clause = clause.into();").unwrap();
    writeln!(
        out,
        "        let incoming: Vec<BindValue> = binds.into_iter().map(Into::into).collect();"
    )
    .unwrap();
    writeln!(out, "        let mut idx = self.binds.len() + 1;").unwrap();
    writeln!(out, "        while let Some(pos) = clause.find('?') {{").unwrap();
    writeln!(out, "            let ph = format!(\"${{}}\", idx);").unwrap();
    writeln!(out, "            clause.replace_range(pos..pos + 1, &ph);").unwrap();
    writeln!(out, "            idx += 1;").unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(out, "        if let Some(last) = self.where_sql.pop() {{").unwrap();
    writeln!(
        out,
        "            self.where_sql.push(format!(\"({{}} OR {{}})\", last, clause));"
    )
    .unwrap();
    writeln!(out, "        }} else {{").unwrap();
    writeln!(out, "            self.where_sql.push(clause);").unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(out, "        self.binds.extend(incoming);").unwrap();
    writeln!(out, "        self").unwrap();
    writeln!(out, "    }}").unwrap();
    out
}

fn render_where_group_methods() -> String {
    let mut out = String::new();
    writeln!(
        out,
        "    pub fn where_group(self, f: impl FnOnce(Self) -> Self) -> Self {{"
    )
    .unwrap();
    writeln!(out, "        let start_where = self.where_sql.len();").unwrap();
    writeln!(out, "        let grouped = f(self);").unwrap();
    writeln!(out, "        let mut result = grouped;").unwrap();
    writeln!(out, "        if result.where_sql.len() > start_where {{").unwrap();
    writeln!(
        out,
        "            let group_clauses: Vec<String> = result.where_sql.drain(start_where..).collect();"
    )
    .unwrap();
    writeln!(
        out,
        "            let grouped_sql = format!(\"({{}})\", group_clauses.join(\" AND \"));"
    )
    .unwrap();
    writeln!(out, "            result.where_sql.push(grouped_sql);").unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(out, "        result").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(
        out,
        "    pub fn or_where_group(self, f: impl FnOnce(Self) -> Self) -> Self {{"
    )
    .unwrap();
    writeln!(out, "        let start_where = self.where_sql.len();").unwrap();
    writeln!(out, "        let grouped = f(self);").unwrap();
    writeln!(out, "        let mut result = grouped;").unwrap();
    writeln!(out, "        if result.where_sql.len() > start_where {{").unwrap();
    writeln!(
        out,
        "            let group_clauses: Vec<String> = result.where_sql.drain(start_where..).collect();"
    )
    .unwrap();
    writeln!(
        out,
        "            let grouped_sql = format!(\"({{}})\", group_clauses.join(\" AND \"));"
    )
    .unwrap();
    writeln!(
        out,
        "            if let Some(last) = result.where_sql.pop() {{"
    )
    .unwrap();
    writeln!(
        out,
        "                result.where_sql.push(format!(\"({{}} OR {{}})\", last, grouped_sql));"
    )
    .unwrap();
    writeln!(out, "            }} else {{").unwrap();
    writeln!(out, "                result.where_sql.push(grouped_sql);").unwrap();
    writeln!(out, "            }}").unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(out, "        result").unwrap();
    writeln!(out, "    }}").unwrap();
    out
}

fn render_distinct_methods(col_ident: &str) -> String {
    let mut out = String::new();
    writeln!(
        out,
        "    pub fn distinct(mut self) -> Self {{ self.distinct = true; self }}"
    )
    .unwrap();
    writeln!(
        out,
        "    pub fn distinct_on(mut self, cols: &[{col_ident}]) -> Self {{"
    )
    .unwrap();
    writeln!(out, "        if cols.is_empty() {{ return self; }}").unwrap();
    writeln!(
        out,
        "        let list: Vec<&'static str> = cols.iter().map(|c| c.as_sql()).collect();"
    )
    .unwrap();
    writeln!(out, "        self.distinct_on = Some(list.join(\", \"));").unwrap();
    writeln!(out, "        self").unwrap();
    writeln!(out, "    }}").unwrap();
    out
}

fn render_order_methods(col_ident: &str) -> String {
    let mut out = String::new();
    writeln!(
        out,
        "    pub fn order_by(mut self, col: {col_ident}, dir: OrderDir) -> Self {{"
    )
    .unwrap();
    writeln!(
        out,
        "        self.order_sql.push(format!(\"{{}} {{}}\", col.as_sql(), dir.as_sql()));"
    )
    .unwrap();
    writeln!(out, "        self").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(
        out,
        "    pub fn order_by_nulls_first(mut self, col: {col_ident}, dir: OrderDir) -> Self {{"
    )
    .unwrap();
    writeln!(
        out,
        "        self.order_sql.push(format!(\"{{}} {{}} NULLS FIRST\", col.as_sql(), dir.as_sql()));"
    )
    .unwrap();
    writeln!(out, "        self").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(
        out,
        "    pub fn order_by_nulls_last(mut self, col: {col_ident}, dir: OrderDir) -> Self {{"
    )
    .unwrap();
    writeln!(
        out,
        "        self.order_sql.push(format!(\"{{}} {{}} NULLS LAST\", col.as_sql(), dir.as_sql()));"
    )
    .unwrap();
    writeln!(out, "        self").unwrap();
    writeln!(out, "    }}").unwrap();
    out
}

fn render_lock_methods() -> String {
    let mut out = String::new();
    writeln!(
        out,
        "    pub fn for_update(mut self) -> Self {{ self.lock_sql = Some(\"FOR UPDATE\"); self }}"
    )
    .unwrap();
    writeln!(
        out,
        "    pub fn for_update_skip_locked(mut self) -> Self {{ self.lock_sql = Some(\"FOR UPDATE SKIP LOCKED\"); self }}"
    )
    .unwrap();
    writeln!(
        out,
        "    pub fn for_no_key_update(mut self) -> Self {{ self.lock_sql = Some(\"FOR NO KEY UPDATE\"); self }}"
    )
    .unwrap();
    writeln!(
        out,
        "    pub fn for_share(mut self) -> Self {{ self.lock_sql = Some(\"FOR SHARE\"); self }}"
    )
    .unwrap();
    writeln!(
        out,
        "    pub fn for_key_share(mut self) -> Self {{ self.lock_sql = Some(\"FOR KEY SHARE\"); self }}"
    )
    .unwrap();
    out
}

fn render_find_methods(model_title: &str, parent_pk_ty: &str, table: &str, pk: &str) -> String {
    let wr = format!("{model_title}WithRelations");
    let mut out = String::new();
    writeln!(
        out,
        "    pub async fn first(self) -> Result<Option<{wr}>> {{"
    )
    .unwrap();
    writeln!(out, "        let mut v = self.limit(1).get().await?;").unwrap();
    writeln!(out, "        Ok(v.pop())").unwrap();
    writeln!(out, "    }}\n").unwrap();
    writeln!(
        out,
        "    pub async fn first_or_fail(self) -> Result<{wr}> {{"
    )
    .unwrap();
    writeln!(
        out,
        "        self.first().await?.ok_or_else(|| anyhow::anyhow!(\"{table}: record not found\"))"
    )
    .unwrap();
    writeln!(out, "    }}\n").unwrap();
    writeln!(
        out,
        "    pub async fn find(self, id: {parent_pk_ty}) -> Result<Option<{wr}>> {{"
    )
    .unwrap();
    writeln!(
        out,
        "        self.where_{}(Op::Eq, id).first().await",
        to_snake(pk)
    )
    .unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(
        out,
        "    pub async fn find_or_fail(self, id: {parent_pk_ty}) -> Result<{wr}> {{"
    )
    .unwrap();
    writeln!(
        out,
        "        self.find(id).await?.ok_or_else(|| anyhow::anyhow!(\"{table}: record not found\"))"
    )
    .unwrap();
    writeln!(out, "    }}").unwrap();
    out
}

fn render_small_terminal_methods(
    query_ident: &str,
    model_title: &str,
    col_ident: &str,
    pk_col_variant: &str,
    has_created_at: bool,
) -> String {
    let view_ident = format!("{model_title}WithRelations");
    let mut out = String::new();
    writeln!(out, "    pub async fn exists(self) -> Result<bool> {{").unwrap();
    writeln!(out, "        Ok(self.count().await? > 0)").unwrap();
    writeln!(out, "    }}\n").unwrap();
    writeln!(
        out,
        "    pub async fn chunk<F, Fut>(mut self, size: i64, mut callback: F) -> Result<()>"
    )
    .unwrap();
    writeln!(out, "    where").unwrap();
    writeln!(out, "        F: FnMut(Vec<{view_ident}>) -> Fut,").unwrap();
    writeln!(
        out,
        "        Fut: std::future::Future<Output = Result<bool>>,"
    )
    .unwrap();
    writeln!(out, "    {{").unwrap();
    writeln!(out, "        let mut page = 0i64;").unwrap();
    writeln!(out, "        let db = self.db.clone();").unwrap();
    writeln!(out, "        loop {{").unwrap();
    writeln!(
        out,
        "            let mut query = {query_ident}::new(db.clone(), self.base_url.clone());"
    )
    .unwrap();
    writeln!(out, "            query.where_sql = self.where_sql.clone();").unwrap();
    writeln!(out, "            query.binds = self.binds.clone();").unwrap();
    writeln!(out, "            query.order_sql = self.order_sql.clone();").unwrap();
    writeln!(
        out,
        "            let rows = query.limit(size).offset(page * size).get().await?;"
    )
    .unwrap();
    writeln!(out, "            if rows.is_empty() {{ break; }}").unwrap();
    writeln!(
        out,
        "            let should_continue = callback(rows).await?;"
    )
    .unwrap();
    writeln!(out, "            if !should_continue {{ break; }}").unwrap();
    writeln!(out, "            page += 1;").unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(out, "        Ok(())").unwrap();
    writeln!(out, "    }}\n").unwrap();
    writeln!(out, "    pub fn latest(self) -> Self {{").unwrap();
    if has_created_at {
        writeln!(
            out,
            "        self.order_by({col_ident}::CreatedAt, OrderDir::Desc)"
        )
        .unwrap();
    } else {
        writeln!(
            out,
            "        self.order_by({col_ident}::{pk_col_variant}, OrderDir::Desc)"
        )
        .unwrap();
    }
    writeln!(out, "    }}\n").unwrap();
    writeln!(out, "    pub fn oldest(self) -> Self {{").unwrap();
    if has_created_at {
        writeln!(
            out,
            "        self.order_by({col_ident}::CreatedAt, OrderDir::Asc)"
        )
        .unwrap();
    } else {
        writeln!(
            out,
            "        self.order_by({col_ident}::{pk_col_variant}, OrderDir::Asc)"
        )
        .unwrap();
    }
    writeln!(out, "    }}\n").unwrap();
    writeln!(out, "    pub fn take(self, n: i64) -> Self {{").unwrap();
    writeln!(out, "        self.limit(n)").unwrap();
    writeln!(out, "    }}\n").unwrap();
    writeln!(out, "    pub fn skip(self, n: i64) -> Self {{").unwrap();
    writeln!(out, "        self.offset(n)").unwrap();
    writeln!(out, "    }}\n").unwrap();
    writeln!(
        out,
        "    pub async fn sole(self) -> Result<{view_ident}> {{"
    )
    .unwrap();
    writeln!(out, "        let mut rows = self.limit(2).get().await?;").unwrap();
    writeln!(out, "        match rows.len() {{").unwrap();
    writeln!(
        out,
        "            0 => anyhow::bail!(\"sole: no record found\"),"
    )
    .unwrap();
    writeln!(out, "            1 => Ok(rows.remove(0)),").unwrap();
    writeln!(
        out,
        "            _ => anyhow::bail!(\"sole: multiple records found\"),"
    )
    .unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(out, "    }}\n").unwrap();
    writeln!(
        out,
        "    fn order_by_raw(mut self, sql: impl Into<String>) -> Self {{"
    )
    .unwrap();
    writeln!(out, "        self.order_sql.push(sql.into());").unwrap();
    writeln!(out, "        self").unwrap();
    writeln!(out, "    }}\n").unwrap();
    writeln!(
        out,
        "    fn group_by_raw(mut self, sql: impl Into<String>) -> Self {{"
    )
    .unwrap();
    writeln!(out, "        self.group_by_sql.push(sql.into());").unwrap();
    writeln!(out, "        self").unwrap();
    writeln!(out, "    }}\n").unwrap();
    writeln!(
        out,
        "    pub async fn pluck_pair<K, V>(self, extract: impl Fn(&{view_ident}) -> (K, V)) -> Result<std::collections::HashMap<K, V>>"
    )
    .unwrap();
    writeln!(out, "    where").unwrap();
    writeln!(out, "        K: Eq + std::hash::Hash,").unwrap();
    writeln!(out, "    {{").unwrap();
    writeln!(out, "        let rows = self.get().await?;").unwrap();
    writeln!(
        out,
        "        Ok(rows.into_iter().map(|r| extract(&r)).collect())"
    )
    .unwrap();
    writeln!(out, "    }}\n").unwrap();
    out
}

fn render_to_sql_method(has_soft_delete: bool, col_ident: &str, table: &str) -> String {
    let mut out = String::new();
    writeln!(
        out,
        "    pub fn to_sql(&self) -> (String, Vec<BindValue>) {{"
    )
    .unwrap();
    writeln!(out, "        let select_sql = self.select_sql.clone();").unwrap();
    writeln!(out, "        let from_sql = self.from_sql.clone();").unwrap();
    writeln!(out, "        let distinct = self.distinct;").unwrap();
    writeln!(out, "        let distinct_on = self.distinct_on.clone();").unwrap();
    writeln!(out, "        let lock_sql = self.lock_sql;").unwrap();
    writeln!(out, "        let join_sql = self.join_sql.clone();").unwrap();
    writeln!(out, "        let join_binds = self.join_binds.clone();").unwrap();
    writeln!(out, "        let mut where_sql = self.where_sql.clone();").unwrap();
    writeln!(out, "        let order_sql = self.order_sql.clone();").unwrap();
    writeln!(out, "        let group_by_sql = self.group_by_sql.clone();").unwrap();
    writeln!(out, "        let having_sql = self.having_sql.clone();").unwrap();
    writeln!(out, "        let having_binds = self.having_binds.clone();").unwrap();
    writeln!(out, "        let offset = self.offset;").unwrap();
    writeln!(out, "        let limit = self.limit;").unwrap();
    writeln!(out, "        let binds = self.binds.clone();").unwrap();
    if has_soft_delete {
        writeln!(out, "        let with_deleted = self.with_deleted;").unwrap();
        writeln!(out, "        let only_deleted = self.only_deleted;").unwrap();
        writeln!(out, "        if HAS_SOFT_DELETE {{").unwrap();
        writeln!(out, "            if only_deleted {{").unwrap();
        writeln!(
            out,
            "                where_sql.push(format!(\"{{}} IS NOT NULL\", {col_ident}::DeletedAt.as_sql()));"
        )
        .unwrap();
        writeln!(out, "            }} else if !with_deleted {{").unwrap();
        writeln!(
            out,
            "                where_sql.push(format!(\"{{}} IS NULL\", {col_ident}::DeletedAt.as_sql()));"
        )
        .unwrap();
        writeln!(out, "            }}").unwrap();
        writeln!(out, "        }}").unwrap();
    }
    // Build select clause
    writeln!(
        out,
        "        let select_clause = match (distinct, distinct_on.as_ref()) {{"
    )
    .unwrap();
    writeln!(
        out,
        "            (false, None) => select_sql.unwrap_or_else(|| \"*\".to_string()),"
    )
    .unwrap();
    writeln!(
        out,
        "            (true, None) => format!(\"DISTINCT {{}}\", select_sql.unwrap_or_else(|| \"*\".to_string())),"
    )
    .unwrap();
    writeln!(
        out,
        "            (_, Some(col)) => format!(\"DISTINCT ON ({{}}) {{}}\", col, select_sql.unwrap_or_else(|| \"*\".to_string())),"
    )
    .unwrap();
    writeln!(out, "        }};").unwrap();
    writeln!(
        out,
        "        let table_name = from_sql.unwrap_or_else(|| \"{table}\".to_string());"
    )
    .unwrap();
    writeln!(
        out,
        "        let mut sql = format!(\"SELECT {{}} FROM {{}}\", select_clause, table_name);"
    )
    .unwrap();
    writeln!(
        out,
        "        if !join_sql.is_empty() {{ sql.push(' '); sql.push_str(&join_sql.join(\" \")); }}"
    )
    .unwrap();
    writeln!(out, "        if !where_sql.is_empty() {{").unwrap();
    writeln!(out, "            sql.push_str(\" WHERE \");").unwrap();
    writeln!(out, "            sql.push_str(&where_sql.join(\" AND \"));").unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(out, "        if !group_by_sql.is_empty() {{").unwrap();
    writeln!(out, "            sql.push_str(\" GROUP BY \");").unwrap();
    writeln!(out, "            sql.push_str(&group_by_sql.join(\", \"));").unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(out, "        if !having_sql.is_empty() {{").unwrap();
    writeln!(out, "            sql.push_str(\" HAVING \");").unwrap();
    writeln!(
        out,
        "            sql.push_str(&having_sql.join(\" AND \"));"
    )
    .unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(out, "        if !order_sql.is_empty() {{").unwrap();
    writeln!(out, "            sql.push_str(\" ORDER BY \");").unwrap();
    writeln!(out, "            sql.push_str(&order_sql.join(\", \"));").unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(out, "        if let Some(off) = offset {{").unwrap();
    writeln!(out, "            sql.push_str(\" OFFSET \");").unwrap();
    writeln!(out, "            sql.push_str(&off.to_string());").unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(out, "        if let Some(l) = limit {{").unwrap();
    writeln!(out, "            sql.push_str(\" LIMIT \");").unwrap();
    writeln!(out, "            sql.push_str(&l.to_string());").unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(
        out,
        "        if let Some(lock) = lock_sql {{ sql.push(' '); sql.push_str(lock); }}"
    )
    .unwrap();
    writeln!(out, "        let mut all_binds = binds;").unwrap();
    writeln!(out, "        all_binds.extend(join_binds);").unwrap();
    writeln!(out, "        all_binds.extend(having_binds);").unwrap();
    writeln!(out, "        (sql, all_binds)").unwrap();
    writeln!(out, "    }}\n").unwrap();
    out
}

fn render_into_where_parts_method(col_ident: &str, has_soft_delete: bool) -> String {
    let mut out = String::new();
    writeln!(
        out,
        "    pub fn into_where_parts(self) -> (Vec<String>, Vec<BindValue>) {{"
    )
    .unwrap();
    if has_soft_delete {
        writeln!(
            out,
            "        let Self {{ where_sql, binds, with_deleted, only_deleted, .. }} = self;"
        )
        .unwrap();
    } else {
        writeln!(out, "        let Self {{ where_sql, binds, .. }} = self;").unwrap();
    }
    writeln!(out, "        let mut where_sql = where_sql;").unwrap();
    if has_soft_delete {
        writeln!(out, "        if HAS_SOFT_DELETE {{").unwrap();
        writeln!(out, "            if only_deleted {{").unwrap();
        writeln!(
            out,
            "                where_sql.push(format!(\"{{}} IS NOT NULL\", {col_ident}::DeletedAt.as_sql()));"
        )
        .unwrap();
        writeln!(out, "            }} else if !with_deleted {{").unwrap();
        writeln!(
            out,
            "                where_sql.push(format!(\"{{}} IS NULL\", {col_ident}::DeletedAt.as_sql()));"
        )
        .unwrap();
        writeln!(out, "            }}").unwrap();
        writeln!(out, "        }}").unwrap();
    }
    writeln!(out, "        (where_sql, binds)").unwrap();
    writeln!(out, "    }}").unwrap();
    out
}

fn render_delete_method(table: &str, col_ident: &str, has_soft_delete: bool, audit: bool, row_ident: &str, pk_snake: &str, skip_profiler: bool) -> String {
    let mut out = String::new();
    writeln!(out, "    pub async fn delete(self) -> Result<u64> {{").unwrap();
    writeln!(out, "        if self.limit.is_some() {{").unwrap();
    writeln!(
        out,
        "            anyhow::bail!(\"delete() does not support limit; add where clauses\");"
    )
    .unwrap();
    writeln!(out, "        }}").unwrap();
    if has_soft_delete {
        writeln!(
            out,
            "        let Self {{ db, where_sql, binds, with_deleted, only_deleted, .. }} = self;"
        )
        .unwrap();
    } else {
        writeln!(
            out,
            "        let Self {{ db, where_sql, binds, .. }} = self;"
        )
        .unwrap();
    }
    writeln!(
        out,
        "        if where_sql.is_empty() {{ anyhow::bail!(\"delete(): no conditions set\"); }}"
    )
    .unwrap();
    writeln!(out, "        let __profiler_binds = if is_sql_profiler_enabled() {{ binds.iter().map(|b| format!(\"{{}}\", b)).collect::<Vec<_>>().join(\", \") }} else {{ String::new() }};").unwrap();
    // Fetch old rows before delete for audit hooks
    if audit {
        writeln!(out, "        let __observer_active = try_get_observer().is_some();").unwrap();
        writeln!(out, "        let __old_rows_json: Vec<(i64, serde_json::Value)> = if __observer_active {{").unwrap();
        writeln!(out, "            let select_sql = format!(\"SELECT * FROM {table} WHERE {{}}\", where_sql.join(\" AND \"));").unwrap();
        writeln!(out, "            let mut fq = sqlx::query_as::<_, {row_ident}>(&select_sql);").unwrap();
        writeln!(out, "            for b in &binds {{ fq = bind(fq, b.clone()); }}").unwrap();
        writeln!(out, "            let rows: Vec<{row_ident}> = db.fetch_all(fq).await.unwrap_or_default();").unwrap();
        writeln!(out, "            rows.into_iter().map(|r| (r.{pk_snake}, serde_json::to_value(&r).unwrap_or_default())).collect()").unwrap();
        writeln!(out, "        }} else {{").unwrap();
        writeln!(out, "            Vec::new()").unwrap();
        writeln!(out, "        }};").unwrap();
    }
    if has_soft_delete {
        writeln!(out, "        if HAS_SOFT_DELETE {{").unwrap();
        writeln!(out, "            let mut where_sql = where_sql;").unwrap();
        writeln!(out, "            if only_deleted {{").unwrap();
        writeln!(
            out,
            "                where_sql.push(format!(\"{{}} IS NOT NULL\", {col_ident}::DeletedAt.as_sql()));"
        )
        .unwrap();
        writeln!(out, "            }} else if !with_deleted {{").unwrap();
        writeln!(
            out,
            "                where_sql.push(format!(\"{{}} IS NULL\", {col_ident}::DeletedAt.as_sql()));"
        )
        .unwrap();
        writeln!(out, "            }}").unwrap();
        writeln!(out, "            let idx = binds.len() + 1;").unwrap();
        writeln!(
            out,
            "            let mut sql = format!(\"UPDATE {table} SET {{}} = ${{}}\", {col_ident}::DeletedAt.as_sql(), idx);"
        )
        .unwrap();
        writeln!(out, "            if !where_sql.is_empty() {{").unwrap();
        writeln!(out, "                sql.push_str(\" WHERE \");").unwrap();
        writeln!(
            out,
            "                sql.push_str(&where_sql.join(\" AND \"));"
        )
        .unwrap();
        writeln!(out, "            }}").unwrap();
        if !skip_profiler {
            writeln!(out, "            let __profiler_start = std::time::Instant::now();").unwrap();
        }
        writeln!(out, "            let mut q = sqlx::query(&sql);").unwrap();
        writeln!(
            out,
            "            for b in binds {{ q = bind_query(q, b); }}"
        )
        .unwrap();
        writeln!(
            out,
            "            q = bind_query(q, time::OffsetDateTime::now_utc().into());"
        )
        .unwrap();
        writeln!(out, "            let res = db.execute(q).await?;").unwrap();
        if !skip_profiler {
            writeln!(out, "            record_profiled_query(\"{table}\", \"UPDATE\", &sql, &__profiler_binds, __profiler_start.elapsed());").unwrap();
        }
        if audit {
            writeln!(out, "            if !__old_rows_json.is_empty() && res.rows_affected() > 0 {{").unwrap();
            writeln!(out, "                if let Some(observer) = try_get_observer() {{").unwrap();
            writeln!(out, "                    for (record_id, old_data) in &__old_rows_json {{").unwrap();
            writeln!(out, "                        let event = ModelEvent {{ table: \"{table}\", record_id: *record_id }};").unwrap();
            writeln!(out, "                        let _ = observer.on_deleted(&event, old_data).await;").unwrap();
            writeln!(out, "                    }}").unwrap();
            writeln!(out, "                }}").unwrap();
            writeln!(out, "            }}").unwrap();
        }
        writeln!(out, "            return Ok(res.rows_affected());").unwrap();
        writeln!(out, "        }}").unwrap();
    }
    writeln!(
        out,
        "        let mut sql = String::from(\"DELETE FROM {table}\");"
    )
    .unwrap();
    writeln!(out, "        if !where_sql.is_empty() {{").unwrap();
    writeln!(out, "            sql.push_str(\" WHERE \");").unwrap();
    writeln!(out, "            sql.push_str(&where_sql.join(\" AND \"));").unwrap();
    writeln!(out, "        }}").unwrap();
    out.push_str(&render_profiler_start(skip_profiler));
    writeln!(out, "        let mut q = sqlx::query(&sql);").unwrap();
    writeln!(out, "        for b in binds {{ q = bind_query(q, b); }}").unwrap();
    writeln!(out, "        let res = db.execute(q).await?;").unwrap();
    out.push_str(&render_profiler_log(table, "DELETE", "&sql", "&__profiler_binds", skip_profiler));
    if audit {
        writeln!(out, "        if !__old_rows_json.is_empty() && res.rows_affected() > 0 {{").unwrap();
        writeln!(out, "            if let Some(observer) = try_get_observer() {{").unwrap();
        writeln!(out, "                for (record_id, old_data) in &__old_rows_json {{").unwrap();
        writeln!(out, "                    let event = ModelEvent {{ table: \"{table}\", record_id: *record_id }};").unwrap();
        writeln!(out, "                    let _ = observer.on_deleted(&event, old_data).await;").unwrap();
        writeln!(out, "                }}").unwrap();
        writeln!(out, "            }}").unwrap();
        writeln!(out, "        }}").unwrap();
    }
    writeln!(out, "        Ok(res.rows_affected())").unwrap();
    writeln!(out, "    }}").unwrap();
    out
}

fn render_restore_method(table: &str, col_ident: &str, skip_profiler: bool) -> String {
    let mut out = String::new();
    writeln!(out, "    pub async fn restore(self) -> Result<u64> {{").unwrap();
    writeln!(
        out,
        "        if !HAS_SOFT_DELETE {{ anyhow::bail!(\"restore() not supported\"); }}"
    )
    .unwrap();
    writeln!(out, "        if self.limit.is_some() {{").unwrap();
    writeln!(
        out,
        "            anyhow::bail!(\"restore() does not support limit; add where clauses\");"
    )
    .unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(
        out,
        "        let Self {{ db, where_sql, binds, with_deleted, only_deleted, .. }} = self;"
    )
    .unwrap();
    writeln!(
        out,
        "        if where_sql.is_empty() {{ anyhow::bail!(\"restore(): no conditions set\"); }}"
    )
    .unwrap();
    writeln!(out, "        let mut where_sql = where_sql;").unwrap();
    writeln!(out, "        if !with_deleted && !only_deleted {{").unwrap();
    writeln!(
        out,
        "            where_sql.push(format!(\"{{}} IS NOT NULL\", {col_ident}::DeletedAt.as_sql()));"
    )
    .unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(
        out,
        "        let mut sql = format!(\"UPDATE {table} SET {{}} = NULL\", {col_ident}::DeletedAt.as_sql());"
    )
    .unwrap();
    writeln!(out, "        if !where_sql.is_empty() {{").unwrap();
    writeln!(out, "            sql.push_str(\" WHERE \");").unwrap();
    writeln!(out, "            sql.push_str(&where_sql.join(\" AND \"));").unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(out, "        let __profiler_binds = if is_sql_profiler_enabled() {{ binds.iter().map(|b| format!(\"{{}}\", b)).collect::<Vec<_>>().join(\", \") }} else {{ String::new() }};").unwrap();
    out.push_str(&render_profiler_start(skip_profiler));
    writeln!(out, "        let mut q = sqlx::query(&sql);").unwrap();
    writeln!(out, "        for b in binds {{ q = bind_query(q, b); }}").unwrap();
    writeln!(out, "        let res = db.execute(q).await?;").unwrap();
    out.push_str(&render_profiler_log(table, "UPDATE", "&sql", "&__profiler_binds", skip_profiler));
    writeln!(out, "        Ok(res.rows_affected())").unwrap();
    writeln!(out, "    }}").unwrap();
    out
}

fn render_soft_delete_scope_methods() -> String {
    let mut out = String::new();
    writeln!(
        out,
        "    pub fn with_deleted(mut self) -> Self {{ self.with_deleted = true; self }}"
    )
    .unwrap();
    writeln!(
        out,
        "    pub fn only_deleted(mut self) -> Self {{ self.only_deleted = true; self }}"
    )
    .unwrap();
    out
}

fn render_insert_field_setters(db_fields: &[FieldSpec], col_ident: &str) -> String {
    let mut out = String::new();
    for f in db_fields {
        let fn_name = format!("set_{}", to_snake(&f.name));
        if let Some(SpecialType::Hashed) = f.special_type {
            writeln!(
                out,
                "    pub fn {fn_name}(mut self, val: &str) -> anyhow::Result<Self> {{",
            )
            .unwrap();
            writeln!(
                out,
                "        let hashed = core_db::common::auth::hash::hash_password(val)?;"
            )
            .unwrap();
            writeln!(
                out,
                "        self.cols.push({col_ident}::{});",
                to_title_case(&f.name)
            )
            .unwrap();
            writeln!(out, "        self.binds.push(hashed.into());").unwrap();
            writeln!(out, "        Ok(self)").unwrap();
            writeln!(out, "    }}").unwrap();

            let fn_name_raw = format!("{}_raw", fn_name);
            writeln!(
                out,
                "    pub fn {fn_name_raw}(mut self, val: String) -> Self {{",
            )
            .unwrap();
            writeln!(
                out,
                "        self.cols.push({col_ident}::{});",
                to_title_case(&f.name)
            )
            .unwrap();
            writeln!(out, "        self.binds.push(val.into());").unwrap();
            writeln!(out, "        self").unwrap();
            writeln!(out, "    }}").unwrap();
        } else {
            writeln!(
                out,
                "    pub fn {fn_name}(mut self, val: {typ}) -> Self {{",
                typ = f.ty
            )
            .unwrap();
            writeln!(
                out,
                "        self.cols.push({col_ident}::{});",
                to_title_case(&f.name)
            )
            .unwrap();
            writeln!(out, "        self.binds.push(val.into());").unwrap();
            writeln!(out, "        self").unwrap();
            writeln!(out, "    }}").unwrap();
        }
    }
    out
}

fn render_localized_setters(localized_fields: &[String], cfgs: &ConfigsFile) -> String {
    let mut out = String::new();
    for f in localized_fields {
        let fn_name = format!("set_{}_lang", to_snake(f));
        writeln!(
            out,
            "    pub fn {fn_name}(mut self, locale: localized::Locale, val: impl Into<String>) -> Self {{"
        )
        .unwrap();
        writeln!(
            out,
            "        self.translations.entry(\"{f}\").or_default().insert(locale.into(), val.into());"
        )
        .unwrap();
        writeln!(out, "        self").unwrap();
        writeln!(out, "    }}").unwrap();

        let fn_name_bulk = format!("set_{}_langs", to_snake(f));
        writeln!(
            out,
            "    pub fn {fn_name_bulk}(mut self, langs: localized::LocalizedText) -> Self {{"
        )
        .unwrap();
        for lang in &cfgs.languages.supported {
            let variant = to_title_case(lang);
            writeln!(
                out,
                "        if !langs.{lang}.is_empty() {{ self = self.{fn_name}(localized::Locale::{variant}, langs.{lang}); }}"
            )
            .unwrap();
        }
        writeln!(out, "        self").unwrap();
        writeln!(out, "    }}").unwrap();

        let fn_name_input = format!("set_{}_input", to_snake(f));
        writeln!(
            out,
            "    pub fn {fn_name_input}(mut self, input: Option<localized::LocalizedInput>) -> Self {{"
        )
        .unwrap();
        writeln!(out, "        let Some(input) = input else {{ return self; }};").unwrap();
        writeln!(out, "        if input.is_empty() {{ return self; }}").unwrap();
        writeln!(out, "        let map = input.to_hashmap();").unwrap();
        writeln!(out, "        for (locale, val) in map {{").unwrap();
        writeln!(
            out,
            "            self.translations.entry(\"{f}\").or_default().insert(locale, val);"
        )
        .unwrap();
        writeln!(out, "        }}").unwrap();
        writeln!(out, "        self").unwrap();
        writeln!(out, "    }}").unwrap();
    }
    out
}

fn render_meta_setters(meta_fields: &[MetaFieldSpec]) -> String {
    let mut out = String::new();
    for m in meta_fields {
        let fn_name = format!("set_meta_{}", m.name);
        match &m.ty {
            MetaType::String => {
                writeln!(
                    out,
                    "    pub fn {fn_name}(mut self, val: impl Into<String>) -> Self {{"
                )
                .unwrap();
                writeln!(
                    out,
                    "        self.meta.insert(\"{name}\".to_string(), JsonValue::String(val.into()));",
                    name = m.name
                )
                .unwrap();
                writeln!(out, "        self").unwrap();
                writeln!(out, "    }}").unwrap();
            }
            MetaType::Bool => {
                writeln!(out, "    pub fn {fn_name}(mut self, val: bool) -> Self {{").unwrap();
                writeln!(
                    out,
                    "        self.meta.insert(\"{name}\".to_string(), JsonValue::Bool(val));",
                    name = m.name
                )
                .unwrap();
                writeln!(out, "        self").unwrap();
                writeln!(out, "    }}").unwrap();
            }
            MetaType::I32 => {
                writeln!(out, "    pub fn {fn_name}(mut self, val: i32) -> Self {{").unwrap();
                writeln!(
                    out,
                    "        self.meta.insert(\"{name}\".to_string(), JsonValue::from(val));",
                    name = m.name
                )
                .unwrap();
                writeln!(out, "        self").unwrap();
                writeln!(out, "    }}").unwrap();
            }
            MetaType::I64 => {
                writeln!(out, "    pub fn {fn_name}(mut self, val: i64) -> Self {{").unwrap();
                writeln!(
                    out,
                    "        self.meta.insert(\"{name}\".to_string(), JsonValue::from(val));",
                    name = m.name
                )
                .unwrap();
                writeln!(out, "        self").unwrap();
                writeln!(out, "    }}").unwrap();
            }
            MetaType::F64 => {
                writeln!(out, "    pub fn {fn_name}(mut self, val: f64) -> Self {{").unwrap();
                writeln!(
                    out,
                    "        self.meta.insert(\"{name}\".to_string(), JsonValue::from(val));",
                    name = m.name
                )
                .unwrap();
                writeln!(out, "        self").unwrap();
                writeln!(out, "    }}").unwrap();
            }
            MetaType::Decimal => {
                writeln!(out, "    pub fn {fn_name}(mut self, val: rust_decimal::Decimal) -> Self {{").unwrap();
                writeln!(
                    out,
                    "        self.meta.insert(\"{name}\".to_string(), JsonValue::from(val.to_string()));",
                    name = m.name
                )
                .unwrap();
                writeln!(out, "        self").unwrap();
                writeln!(out, "    }}").unwrap();
            }
            MetaType::Json => {
                writeln!(
                    out,
                    "    pub fn {fn_name}(mut self, val: JsonValue) -> Self {{"
                )
                .unwrap();
                writeln!(
                    out,
                    "        self.meta.insert(\"{name}\".to_string(), val);",
                    name = m.name
                )
                .unwrap();
                writeln!(out, "        self").unwrap();
                writeln!(out, "    }}").unwrap();
                let typed_fn_name = format!("set_meta_{}_as", m.name);
                writeln!(
                    out,
                    "    pub fn {typed_fn_name}<T: serde::Serialize>(mut self, val: &T) -> anyhow::Result<Self> {{"
                )
                .unwrap();
                writeln!(
                    out,
                    "        self.meta.insert(\"{name}\".to_string(), serde_json::to_value(val)?);",
                    name = m.name
                )
                .unwrap();
                writeln!(out, "        Ok(self)").unwrap();
                writeln!(out, "    }}").unwrap();
            }
            MetaType::DateTime => {
                writeln!(
                    out,
                    "    pub fn {fn_name}(mut self, val: time::OffsetDateTime) -> Self {{"
                )
                .unwrap();
                writeln!(
                    out,
                    "        self.meta.insert(\"{name}\".to_string(), JsonValue::String(val.to_string()));",
                    name = m.name
                )
                .unwrap();
                writeln!(out, "        self").unwrap();
                writeln!(out, "    }}").unwrap();
            }
            MetaType::Custom(ty) => {
                writeln!(
                    out,
                    "    pub fn {fn_name}(mut self, val: &{ty}) -> anyhow::Result<Self> {{",
                    ty = ty
                )
                .unwrap();
                writeln!(
                    out,
                    "        self.meta.insert(\"{name}\".to_string(), serde_json::to_value(val)?);",
                    name = m.name
                )
                .unwrap();
                writeln!(out, "        Ok(self)").unwrap();
                writeln!(out, "    }}").unwrap();
            }
        }
    }
    out
}

fn render_insert_attachment_setters(
    single_attachments: &[AttachmentFieldSpec],
    multi_attachments: &[AttachmentFieldSpec],
) -> String {
    let mut out = String::new();
    for a in single_attachments {
        let fn_name = format!("set_attachment_{}", to_snake(&a.name));
        writeln!(
            out,
            "    pub fn {fn_name}(mut self, att: AttachmentInput) -> Self {{"
        )
        .unwrap();
        writeln!(
            out,
            "        self.attachments_single.insert(\"{name}\", att);",
            name = a.name
        )
        .unwrap();
        writeln!(out, "        self").unwrap();
        writeln!(out, "    }}").unwrap();
    }
    for a in multi_attachments {
        let fn_name = format!("add_attachment_{}", to_snake(&a.name));
        writeln!(
            out,
            "    pub fn {fn_name}(mut self, att: AttachmentInput) -> Self {{"
        )
        .unwrap();
        writeln!(
            out,
            "        self.attachments_multi.entry(\"{name}\").or_default().push(att);",
            name = a.name
        )
        .unwrap();
        writeln!(out, "        self").unwrap();
        writeln!(out, "    }}").unwrap();
    }
    out
}

fn is_incrementable_type(ty: &str) -> bool {
    matches!(
        ty,
        "i16" | "i32" | "i64" | "f64" | "rust_decimal::Decimal"
    )
}

fn render_update_field_setters(db_fields: &[FieldSpec], col_ident: &str) -> String {
    let mut out = String::new();
    for f in db_fields {
        let fn_name = format!("set_{}", to_snake(&f.name));
        let col_variant = to_title_case(&f.name);
        if let Some(SpecialType::Hashed) = f.special_type {
            writeln!(
                out,
                "    pub fn {fn_name}(mut self, val: &str) -> anyhow::Result<Self> {{",
            )
            .unwrap();
            writeln!(
                out,
                "        let hashed = core_db::common::auth::hash::hash_password(val)?;"
            )
            .unwrap();
            writeln!(
                out,
                "        self.sets.push(({col_ident}::{col_variant}, hashed.into(), SetMode::Assign));",
            )
            .unwrap();
            writeln!(out, "        Ok(self)").unwrap();
            writeln!(out, "    }}").unwrap();

            let fn_name_raw = format!("{}_raw", fn_name);
            writeln!(
                out,
                "    pub fn {fn_name_raw}(mut self, val: String) -> Self {{",
            )
            .unwrap();
            writeln!(
                out,
                "        self.sets.push(({col_ident}::{col_variant}, val.into(), SetMode::Assign));",
            )
            .unwrap();
            writeln!(out, "        self").unwrap();
            writeln!(out, "    }}").unwrap();
        } else {
            writeln!(
                out,
                "    pub fn {fn_name}(mut self, val: {typ}) -> Self {{",
                typ = f.ty
            )
            .unwrap();
            writeln!(
                out,
                "        self.sets.push(({col_ident}::{col_variant}, val.into(), SetMode::Assign));",
            )
            .unwrap();
            writeln!(out, "        self").unwrap();
            writeln!(out, "    }}").unwrap();

            // Generate increment/decrement for non-optional numeric fields
            if is_incrementable_type(&f.ty) {
                let snake = to_snake(&f.name);
                let inc_fn = format!("increment_{snake}");
                let dec_fn = format!("decrement_{snake}");
                writeln!(
                    out,
                    "    pub fn {inc_fn}(mut self, val: {typ}) -> Self {{",
                    typ = f.ty
                )
                .unwrap();
                writeln!(
                    out,
                    "        self.sets.push(({col_ident}::{col_variant}, val.into(), SetMode::Increment));",
                )
                .unwrap();
                writeln!(out, "        self").unwrap();
                writeln!(out, "    }}").unwrap();

                writeln!(
                    out,
                    "    pub fn {dec_fn}(mut self, val: {typ}) -> Self {{",
                    typ = f.ty
                )
                .unwrap();
                writeln!(
                    out,
                    "        self.sets.push(({col_ident}::{col_variant}, val.into(), SetMode::Decrement));",
                )
                .unwrap();
                writeln!(out, "        self").unwrap();
                writeln!(out, "    }}").unwrap();
            }
        }
    }
    out
}

fn render_update_attachment_setters(
    single_attachments: &[AttachmentFieldSpec],
    multi_attachments: &[AttachmentFieldSpec],
) -> String {
    let mut out = String::new();
    for a in single_attachments {
        let fn_name = format!("set_attachment_{}", to_snake(&a.name));
        writeln!(
            out,
            "    pub fn {fn_name}(mut self, att: AttachmentInput) -> Self {{"
        )
        .unwrap();
        writeln!(
            out,
            "        self.attachments_single.insert(\"{name}\", att);",
            name = a.name
        )
        .unwrap();
        writeln!(out, "        self").unwrap();
        writeln!(out, "    }}").unwrap();

        let clear_fn = format!("clear_attachment_{}", to_snake(&a.name));
        writeln!(out, "    pub fn {clear_fn}(mut self) -> Self {{").unwrap();
        writeln!(
            out,
            "        if !self.attachments_clear_single.contains(&\"{name}\") {{ self.attachments_clear_single.push(\"{name}\"); }}",
            name = a.name
        )
        .unwrap();
        writeln!(out, "        self").unwrap();
        writeln!(out, "    }}").unwrap();
    }
    for a in multi_attachments {
        let add_fn = format!("add_attachment_{}", to_snake(&a.name));
        writeln!(
            out,
            "    pub fn {add_fn}(mut self, att: AttachmentInput) -> Self {{"
        )
        .unwrap();
        writeln!(
            out,
            "        self.attachments_multi.entry(\"{name}\").or_default().push(att);",
            name = a.name
        )
        .unwrap();
        writeln!(out, "        self").unwrap();
        writeln!(out, "    }}").unwrap();

        let del_fn = format!("delete_attachment_{}", to_snake(&a.name));
        writeln!(
            out,
            "    pub fn {del_fn}(mut self, ids: impl IntoIterator<Item = Uuid>) -> Self {{"
        )
        .unwrap();
        writeln!(
            out,
            "        self.attachments_delete_multi.entry(\"{name}\").or_default().extend(ids);",
            name = a.name
        )
        .unwrap();
        writeln!(out, "        self").unwrap();
        writeln!(out, "    }}").unwrap();
    }
    out
}

fn render_support_data_loaders(
    model_snake: &str,
    pk: &str,
    parent_pk_ty: &str,
    localized_fields: &[String],
    has_meta: bool,
    has_attachments: bool,
    rows_ident: &str,
    db_expr: &str,
) -> String {
    let mut out = String::new();
    writeln!(
        out,
        "        let ids: Vec<{parent_pk_ty}> = {rows_ident}.iter().map(|r| r.{pk}.clone()).collect();"
    )
    .unwrap();
    if localized_fields.is_empty() {
        writeln!(out, "        let localized = LocalizedMap::default();").unwrap();
    } else {
        writeln!(
            out,
            "        let localized = localized::load_{model_snake}_localized({db_expr}, &ids).await?;"
        )
        .unwrap();
    }
    if has_meta {
        writeln!(
            out,
            "        let meta_map = localized::load_{model_snake}_meta({db_expr}, &ids).await?;"
        )
        .unwrap();
    }
    if has_attachments {
        writeln!(
            out,
            "        let attachments = localized::load_{model_snake}_attachments({db_expr}, &ids).await?;"
        )
        .unwrap();
    }
    out
}

fn build_hydrate_view_expr(
    row_expr: &str,
    localized_fields: &[String],
    has_meta: bool,
    has_attachments: bool,
    base_url_expr: &str,
) -> String {
    match (has_meta, has_attachments) {
        (true, true) => format!(
            "hydrate_view({row_expr}, &localized, &meta_map, &attachments, {base_url_expr})"
        ),
        (true, false) => {
            format!("hydrate_view({row_expr}, &localized, &meta_map, {base_url_expr})")
        }
        (false, true) => {
            format!("hydrate_view({row_expr}, &localized, &attachments, {base_url_expr})")
        }
        (false, false) => {
            if localized_fields.is_empty() {
                format!("hydrate_view({row_expr}, &LocalizedMap::default(), {base_url_expr})")
            } else {
                format!("hydrate_view({row_expr}, &localized, {base_url_expr})")
            }
        }
    }
}

fn render_view_collection_build(
    out_ident: &str,
    row_var: &str,
    rows_ident: &str,
    localized_fields: &[String],
    has_meta: bool,
    has_attachments: bool,
    base_url_expr: &str,
) -> String {
    let mut out = String::new();
    writeln!(
        out,
        "        let mut {out_ident} = Vec::with_capacity({rows_ident}.len());"
    )
    .unwrap();
    writeln!(out, "        for {row_var} in {rows_ident} {{").unwrap();
    writeln!(
        out,
        "            {out_ident}.push({});",
        build_hydrate_view_expr(
            row_var,
            localized_fields,
            has_meta,
            has_attachments,
            base_url_expr
        )
    )
    .unwrap();
    writeln!(out, "        }}").unwrap();
    out
}

fn render_relation_loader_bindings(relations: &[RelationSpec]) -> String {
    let mut out = String::new();
    for rel in relations {
        let rel_name = to_snake(&rel.name);
        writeln!(
            out,
            "        let {rel_name} = m.load_{rel_name}(&rows).await?;"
        )
        .unwrap();
    }
    out
}

fn render_with_relations_collection_build(
    model_title: &str,
    relations: &[RelationSpec],
    pk: &str,
    rows_ident: &str,
    row_var: &str,
    out_ident: &str,
    localized_fields: &[String],
    has_meta: bool,
    has_attachments: bool,
    base_url_expr: &str,
) -> String {
    let mut out = String::new();
    writeln!(
        out,
        "        let mut {out_ident} = Vec::with_capacity({rows_ident}.len());"
    )
    .unwrap();
    writeln!(out, "        for {row_var} in {rows_ident} {{").unwrap();
    writeln!(out, "            let key = {row_var}.{pk}.clone();").unwrap();
    writeln!(
        out,
        "            let view = {};",
        build_hydrate_view_expr(
            &format!("{row_var}.clone()"),
            localized_fields,
            has_meta,
            has_attachments,
            base_url_expr
        )
    )
    .unwrap();
    writeln!(
        out,
        "            {out_ident}.push({model_title}WithRelations {{"
    )
    .unwrap();
    writeln!(out, "                row: view,").unwrap();
    for rel in relations {
        let field = to_snake(&rel.name);
        match rel.kind {
            RelationKind::HasMany => {
                writeln!(
                    out,
                    "                {field}: {field}.get(&key).cloned().unwrap_or_default(),"
                )
                .unwrap();
            }
            RelationKind::BelongsTo => {
                writeln!(
                    out,
                    "                {field}: {field}.get(&key).cloned().unwrap_or(None),"
                )
                .unwrap();
            }
        }
    }
    writeln!(out, "            }});").unwrap();
    writeln!(out, "        }}").unwrap();
    out
}

/// Generate the `let __profiler_start = ...` line.
/// When `skip` is true (model has `profile = false`), returns empty.
fn render_profiler_start(skip: bool) -> String {
    if skip {
        return String::new();
    }
    "        let __profiler_start = std::time::Instant::now();\n".to_string()
}

/// Generate the profiler log block. `sql_var` is the variable name holding the SQL string,
/// `binds_expr` is the expression for the bind values to display.
/// When `skip` is true (model has `profile = false`), returns empty.
fn render_profiler_log(table: &str, op: &str, sql_var: &str, binds_expr: &str, skip: bool) -> String {
    if skip {
        return String::new();
    }
    let mut out = String::new();
    writeln!(out, "        record_profiled_query(\"{table}\", \"{op}\", {sql_var}, {binds_expr}, __profiler_start.elapsed());").unwrap();
    out
}

fn render_scoped_where_setup(has_soft_delete: bool, col_ident: &str) -> String {
    let mut out = String::new();
    writeln!(out, "        let mut where_sql = where_sql;").unwrap();
    if has_soft_delete {
        writeln!(out, "        if HAS_SOFT_DELETE {{").unwrap();
        writeln!(out, "            if only_deleted {{").unwrap();
        writeln!(
            out,
            "                where_sql.push(format!(\"{{}} IS NOT NULL\", {col_ident}::DeletedAt.as_sql()));"
        )
        .unwrap();
        writeln!(out, "            }} else if !with_deleted {{").unwrap();
        writeln!(
            out,
            "                where_sql.push(format!(\"{{}} IS NULL\", {col_ident}::DeletedAt.as_sql()));"
        )
        .unwrap();
        writeln!(out, "            }}").unwrap();
        writeln!(out, "        }}").unwrap();
    }
    out
}

fn render_from_and_where_clause_setup(table: &str) -> String {
    let mut out = String::new();
    writeln!(
        out,
        "        let table_name = from_sql.unwrap_or_else(|| \"{table}\".to_string());"
    )
    .unwrap();
    writeln!(out, "        let from_clause = if join_sql.is_empty() {{").unwrap();
    writeln!(out, "            format!(\"FROM {{}}\", table_name)").unwrap();
    writeln!(out, "        }} else {{").unwrap();
    writeln!(
        out,
        "            format!(\"FROM {{}} {{}}\", table_name, join_sql.join(\" \"))"
    )
    .unwrap();
    writeln!(out, "        }};").unwrap();
    writeln!(
        out,
        "        let where_clause = if where_sql.is_empty() {{ String::new() }} else {{ format!(\" WHERE {{}}\", where_sql.join(\" AND \")) }};"
    )
    .unwrap();
    out
}

fn render_select_clause_setup() -> String {
    let mut out = String::new();
    writeln!(
        out,
        "        let select_clause = match (distinct, distinct_on.as_ref()) {{"
    )
    .unwrap();
    writeln!(
        out,
        "            (false, None) => select_sql.unwrap_or_else(|| \"*\".to_string()),"
    )
    .unwrap();
    writeln!(
        out,
        "            (true, None) => format!(\"DISTINCT {{}}\", select_sql.unwrap_or_else(|| \"*\".to_string())),"
    )
    .unwrap();
    writeln!(
        out,
        "            (_, Some(on)) => format!(\"DISTINCT ON ({{}}) {{}}\", on, select_sql.unwrap_or_else(|| \"*\".to_string())),"
    )
    .unwrap();
    writeln!(out, "        }};").unwrap();
    out
}

fn render_paginate_count_sql_setup() -> String {
    let mut out = String::new();
    writeln!(
        out,
        "        let count_expr = count_sql.unwrap_or_else(|| \"COUNT(*)\".to_string());"
    )
    .unwrap();
    writeln!(
        out,
        "        let count_sql = if distinct || distinct_on.is_some() {{"
    )
    .unwrap();
    writeln!(
        out,
        "            format!(\"SELECT COUNT(*) FROM (SELECT {{}} {{}}{{}}) AS sub\", select_clause, from_clause, where_clause)"
    )
    .unwrap();
    writeln!(out, "        }} else {{").unwrap();
    writeln!(
        out,
        "            format!(\"SELECT {{}} {{}}{{}}\", count_expr, from_clause, where_clause)"
    )
    .unwrap();
    writeln!(out, "        }};").unwrap();
    out
}

fn render_get_as_method(table: &str) -> String {
    let mut out = String::new();
    writeln!(out, "    pub async fn get_as<T>(self) -> Result<Vec<T>>").unwrap();
    writeln!(out, "    where").unwrap();
    writeln!(
        out,
        "        T: for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin + 'static,"
    )
    .unwrap();
    writeln!(out, "    {{").unwrap();
    writeln!(
        out,
        "        let Self {{ db, select_sql, from_sql, distinct, distinct_on, lock_sql, join_sql, join_binds, where_sql, order_sql, group_by_sql, having_sql, having_binds, offset, limit, binds, .. }} = self;"
    )
    .unwrap();
    writeln!(out, "        let mut where_sql = where_sql;").unwrap();
    out.push_str(&render_select_clause_setup());
    out.push_str(&render_from_and_where_clause_setup(table));
    writeln!(
        out,
        "        let mut sql = format!(\"SELECT {{}} {{}}{{}}\", select_clause, from_clause, where_clause);"
    )
    .unwrap();
    writeln!(out, "        if !group_by_sql.is_empty() {{").unwrap();
    writeln!(out, "            sql.push_str(\" GROUP BY \");").unwrap();
    writeln!(out, "            sql.push_str(&group_by_sql.join(\", \"));").unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(out, "        if !having_sql.is_empty() {{").unwrap();
    writeln!(out, "            sql.push_str(\" HAVING \");").unwrap();
    writeln!(
        out,
        "            sql.push_str(&having_sql.join(\" AND \"));"
    )
    .unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(out, "        if !order_sql.is_empty() {{").unwrap();
    writeln!(out, "            sql.push_str(\" ORDER BY \");").unwrap();
    writeln!(out, "            sql.push_str(&order_sql.join(\", \"));").unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(out, "        if let Some(off) = offset {{").unwrap();
    writeln!(out, "            sql.push_str(\" OFFSET \");").unwrap();
    writeln!(out, "            sql.push_str(&off.to_string());").unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(out, "        if let Some(l) = limit {{").unwrap();
    writeln!(out, "            sql.push_str(\" LIMIT \");").unwrap();
    writeln!(out, "            sql.push_str(&l.to_string());").unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(
        out,
        "        if let Some(lock) = lock_sql {{ sql.push(' '); sql.push_str(lock); }}"
    )
    .unwrap();
    writeln!(out, "        let mut q = sqlx::query_as::<_, T>(&sql);").unwrap();
    writeln!(out, "        for b in binds {{ q = bind(q, b); }}").unwrap();
    writeln!(out, "        for b in join_binds {{ q = bind(q, b); }}").unwrap();
    writeln!(out, "        for b in having_binds {{ q = bind(q, b); }}").unwrap();
    writeln!(out, "        Ok(db.fetch_all(q).await?)").unwrap();
    writeln!(out, "    }}").unwrap();
    out
}

fn render_count_method(has_soft_delete: bool, col_ident: &str, table: &str, skip_profiler: bool) -> String {
    let mut out = String::new();
    writeln!(out, "    pub async fn count(self) -> Result<i64> {{").unwrap();
    writeln!(
        out,
        "        let Self {{ db, from_sql, count_sql, join_sql, join_binds, where_sql, binds {extra} , .. }} = self;",
        extra = if has_soft_delete { ", with_deleted, only_deleted" } else { "" }
    )
    .unwrap();
    out.push_str(&render_scoped_where_setup(has_soft_delete, col_ident));
    out.push_str(&render_from_and_where_clause_setup(table));
    writeln!(
        out,
        "        let count_expr = count_sql.unwrap_or_else(|| \"COUNT(*)\".to_string());"
    )
    .unwrap();
    writeln!(
        out,
        "        let sql = format!(\"SELECT {{}} {{}}{{}}\", count_expr, from_clause, where_clause);"
    )
    .unwrap();
    writeln!(out, "        let __profiler_binds = if is_sql_profiler_enabled() {{ binds.iter().chain(join_binds.iter()).map(|b| format!(\"{{}}\", b)).collect::<Vec<_>>().join(\", \") }} else {{ String::new() }};").unwrap();
    out.push_str(&render_profiler_start(skip_profiler));
    writeln!(
        out,
        "        let mut q = sqlx::query_scalar::<_, i64>(&sql);"
    )
    .unwrap();
    writeln!(out, "        for b in binds {{ q = bind_scalar(q, b); }}").unwrap();
    writeln!(
        out,
        "        for b in join_binds {{ q = bind_scalar(q, b); }}"
    )
    .unwrap();
    writeln!(out, "        let count = db.fetch_scalar(q).await?;").unwrap();
    out.push_str(&render_profiler_log(table, "COUNT", "&sql", "&__profiler_binds", skip_profiler));
    writeln!(out, "        Ok(count)").unwrap();
    writeln!(out, "    }}\n").unwrap();
    out
}

fn render_pluck_ids_method(
    parent_pk_ty: &str,
    pk: &str,
    has_soft_delete: bool,
    col_ident: &str,
    table: &str,
    skip_profiler: bool,
) -> String {
    let mut out = String::new();
    writeln!(
        out,
        "    pub async fn pluck_ids(self) -> Result<Vec<{parent_pk_ty}>> {{"
    )
    .unwrap();
    writeln!(
        out,
        "        let Self {{ db, from_sql, join_sql, join_binds, where_sql, binds, order_sql, limit, offset {extra} , .. }} = self;",
        extra = if has_soft_delete { ", with_deleted, only_deleted" } else { "" }
    )
    .unwrap();
    out.push_str(&render_scoped_where_setup(has_soft_delete, col_ident));
    out.push_str(&render_from_and_where_clause_setup(table));
    writeln!(
        out,
        "        let order_clause = if order_sql.is_empty() {{ String::new() }} else {{ format!(\" ORDER BY {{}}\", order_sql.join(\", \")) }};"
    )
    .unwrap();
    writeln!(
        out,
        "        let limit_clause = limit.map(|n| format!(\" LIMIT {{}}\", n)).unwrap_or_default();"
    )
    .unwrap();
    writeln!(
        out,
        "        let offset_clause = offset.map(|n| format!(\" OFFSET {{}}\", n)).unwrap_or_default();"
    )
    .unwrap();
    writeln!(
        out,
        "        let sql = format!(\"SELECT {pk} {{}}{{}}{{}}{{}}{{}}\", from_clause, where_clause, order_clause, limit_clause, offset_clause);"
    )
    .unwrap();
    writeln!(out, "        let __profiler_binds = if is_sql_profiler_enabled() {{ binds.iter().chain(join_binds.iter()).map(|b| format!(\"{{}}\", b)).collect::<Vec<_>>().join(\", \") }} else {{ String::new() }};").unwrap();
    out.push_str(&render_profiler_start(skip_profiler));
    writeln!(
        out,
        "        let mut q = sqlx::query_scalar::<_, {parent_pk_ty}>(&sql);"
    )
    .unwrap();
    writeln!(out, "        for b in binds {{ q = bind_scalar(q, b); }}").unwrap();
    writeln!(
        out,
        "        for b in join_binds {{ q = bind_scalar(q, b); }}"
    )
    .unwrap();
    writeln!(out, "        let ids = db.fetch_all_scalar(q).await?;").unwrap();
    out.push_str(&render_profiler_log(table, "SELECT", "&sql", "&__profiler_binds", skip_profiler));
    writeln!(out, "        Ok(ids)").unwrap();
    writeln!(out, "    }}\n").unwrap();
    out
}

fn render_scalar_aggregate_method(
    method_name: &str,
    result_ty: &str,
    select_expr: &str,
    has_soft_delete: bool,
    col_ident: &str,
    table: &str,
    skip_profiler: bool,
) -> String {
    let mut out = String::new();
    writeln!(
        out,
        "    pub async fn {method_name}(self, col: {col_ident}) -> Result<{result_ty}> {{"
    )
    .unwrap();
    writeln!(
        out,
        "        let Self {{ db, from_sql, join_sql, join_binds, where_sql, binds {extra} , .. }} = self;",
        extra = if has_soft_delete { ", with_deleted, only_deleted" } else { "" }
    )
    .unwrap();
    out.push_str(&render_scoped_where_setup(has_soft_delete, col_ident));
    out.push_str(&render_from_and_where_clause_setup(table));
    writeln!(
        out,
        "        let sql = format!(\"SELECT {select_expr} {{}}{{}}\", col.as_sql(), from_clause, where_clause);"
    )
    .unwrap();
    writeln!(out, "        let __profiler_binds = if is_sql_profiler_enabled() {{ binds.iter().chain(join_binds.iter()).map(|b| format!(\"{{}}\", b)).collect::<Vec<_>>().join(\", \") }} else {{ String::new() }};").unwrap();
    out.push_str(&render_profiler_start(skip_profiler));
    writeln!(
        out,
        "        let mut q = sqlx::query_scalar::<_, {result_ty}>(&sql);"
    )
    .unwrap();
    writeln!(out, "        for b in binds {{ q = bind_scalar(q, b); }}").unwrap();
    writeln!(
        out,
        "        for b in join_binds {{ q = bind_scalar(q, b); }}"
    )
    .unwrap();
    writeln!(out, "        let result = db.fetch_scalar(q).await?;").unwrap();
    let op = method_name.to_uppercase();
    out.push_str(&render_profiler_log(table, &op, "&sql", "&__profiler_binds", skip_profiler));
    writeln!(out, "        Ok(result)").unwrap();
    writeln!(out, "    }}\n").unwrap();
    out
}

fn render_paginate_method(
    model_title: &str,
    model_ident: &str,
    row_ident: &str,
    has_soft_delete: bool,
    col_ident: &str,
    table: &str,
    model_snake: &str,
    pk: &str,
    parent_pk_ty: &str,
    relations: &[RelationSpec],
    localized_fields: &[String],
    has_meta: bool,
    has_attachments: bool,
    skip_profiler: bool,
) -> String {
    let mut out = String::new();
    writeln!(
        out,
        "    pub async fn paginate(self, page: i64, per_page: i64) -> Result<Page<{model_title}WithRelations>> {{"
    )
    .unwrap();
    writeln!(
        out,
        "        let page = if page < 1 {{ 1 }} else {{ page }};"
    )
    .unwrap();
    writeln!(out, "        let per_page = resolve_per_page(per_page);").unwrap();
    writeln!(
        out,
        "        let Self {{ db, base_url, select_sql, from_sql, count_sql, distinct, distinct_on, lock_sql, join_sql, join_binds, where_sql, order_sql, group_by_sql, having_sql, having_binds, offset: _, limit: _, binds {extra}, .. }} = self;",
        extra = if has_soft_delete { ", with_deleted, only_deleted" } else { "" }
    )
    .unwrap();
    out.push_str(&render_scoped_where_setup(has_soft_delete, col_ident));
    out.push_str(&render_select_clause_setup());
    out.push_str(&render_from_and_where_clause_setup(table));
    out.push_str(&render_paginate_count_sql_setup());
    writeln!(out, "        let __profiler_binds = if is_sql_profiler_enabled() {{ binds.iter().chain(join_binds.iter()).map(|b| format!(\"{{}}\", b)).collect::<Vec<_>>().join(\", \") }} else {{ String::new() }};").unwrap();
    out.push_str(&render_profiler_start(skip_profiler));
    writeln!(
        out,
        "        let mut count_q = sqlx::query_scalar::<_, i64>(&count_sql);"
    )
    .unwrap();
    writeln!(
        out,
        "        for b in binds.iter().cloned() {{ count_q = bind_scalar(count_q, b); }}"
    )
    .unwrap();
    writeln!(
        out,
        "        for b in join_binds.iter().cloned() {{ count_q = bind_scalar(count_q, b); }}"
    )
    .unwrap();
    writeln!(
        out,
        "        let total: i64 = db.fetch_scalar(count_q).await?;"
    )
    .unwrap();
    out.push_str(&render_profiler_log(table, "COUNT", "&count_sql", "&__profiler_binds", skip_profiler));
    writeln!(
        out,
        "        let last_page = ((total + per_page - 1) / per_page).max(1);"
    )
    .unwrap();
    writeln!(out, "        let current_page = page.min(last_page);").unwrap();
    writeln!(
        out,
        "        let offset_val = (current_page - 1) * per_page;"
    )
    .unwrap();
    writeln!(
        out,
        "        let mut sql = format!(\"SELECT {{}} {{}}{{}}\", select_clause, from_clause, where_clause);"
    )
    .unwrap();
    writeln!(out, "        if !order_sql.is_empty() {{").unwrap();
    writeln!(out, "            sql.push_str(\" ORDER BY \");").unwrap();
    writeln!(out, "            sql.push_str(&order_sql.join(\", \"));").unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(
        out,
        "        sql.push_str(&format!(\" OFFSET {{}}\", offset_val));"
    )
    .unwrap();
    writeln!(
        out,
        "        sql.push_str(&format!(\" LIMIT {{}}\", per_page));"
    )
    .unwrap();
    writeln!(
        out,
        "        if let Some(lock) = lock_sql {{ sql.push(' '); sql.push_str(lock); }}"
    )
    .unwrap();
    out.push_str(&render_profiler_start(skip_profiler));
    writeln!(
        out,
        "        let mut q = sqlx::query_as::<_, {row_ident}>(&sql);"
    )
    .unwrap();
    writeln!(
        out,
        "        for b in binds.iter().cloned() {{ q = bind(q, b); }}"
    )
    .unwrap();
    writeln!(out, "        for b in join_binds {{ q = bind(q, b); }}").unwrap();
    writeln!(out, "        let rows = db.fetch_all(q).await?;").unwrap();
    out.push_str(&render_profiler_log(table, "SELECT", "&sql", "&__profiler_binds", skip_profiler));
    if !relations.is_empty() {
        writeln!(
            out,
            "        let m = {model_ident} {{ db: db.clone(), base_url: base_url.clone() }};"
        )
        .unwrap();
        out.push_str(&render_relation_loader_bindings(relations));
    }
    out.push_str(&render_support_data_loaders(
        model_snake,
        pk,
        parent_pk_ty,
        localized_fields,
        has_meta,
        has_attachments,
        "rows",
        "db",
    ));
    if !relations.is_empty() {
        out.push_str(&render_with_relations_collection_build(
            model_title,
            relations,
            pk,
            "rows",
            "row",
            "data",
            localized_fields,
            has_meta,
            has_attachments,
            "base_url.as_deref()",
        ));
    } else {
        out.push_str(&render_view_collection_build(
            "data",
            "r",
            "rows",
            localized_fields,
            has_meta,
            has_attachments,
            "base_url.as_deref()",
        ));
        writeln!(out, "        let data: Vec<{model_title}WithRelations> = data.into_iter().map(|v| {model_title}WithRelations {{ row: v }}).collect();").unwrap();
    }
    writeln!(
        out,
        "        Ok(Page {{ data, total, per_page, current_page, last_page }})"
    )
    .unwrap();
    writeln!(out, "    }}").unwrap();
    out
}

fn render_get_method(
    model_title: &str,
    model_ident: &str,
    row_ident: &str,
    has_soft_delete: bool,
    col_ident: &str,
    table: &str,
    model_snake: &str,
    pk: &str,
    parent_pk_ty: &str,
    relations: &[RelationSpec],
    localized_fields: &[String],
    has_meta: bool,
    has_attachments: bool,
    skip_profiler: bool,
) -> String {
    let mut out = String::new();
    writeln!(
        out,
        "    pub async fn get(self) -> Result<Vec<{model_title}WithRelations>> {{"
    )
    .unwrap();
    writeln!(
        out,
        "        let Self {{ db, base_url, select_sql, from_sql, distinct, distinct_on, lock_sql, join_sql, join_binds, where_sql, order_sql, group_by_sql, having_sql, having_binds, offset, limit, binds {extra}, .. }} = self;",
        extra = if has_soft_delete { ", with_deleted, only_deleted" } else { "" }
    )
    .unwrap();
    out.push_str(&render_scoped_where_setup(has_soft_delete, col_ident));
    out.push_str(&render_select_clause_setup());
    writeln!(
        out,
        "        let table_name = from_sql.unwrap_or_else(|| \"{table}\".to_string());"
    )
    .unwrap();
    writeln!(
        out,
        "        let mut sql = format!(\"SELECT {{}} FROM {{}}\", select_clause, table_name);"
    )
    .unwrap();
    writeln!(
        out,
        "        if !join_sql.is_empty() {{ sql.push(' '); sql.push_str(&join_sql.join(\" \")); }}"
    )
    .unwrap();
    writeln!(out, "        if !where_sql.is_empty() {{").unwrap();
    writeln!(out, "            sql.push_str(\" WHERE \");").unwrap();
    writeln!(out, "            sql.push_str(&where_sql.join(\" AND \"));").unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(out, "        if !group_by_sql.is_empty() {{").unwrap();
    writeln!(out, "            sql.push_str(\" GROUP BY \");").unwrap();
    writeln!(out, "            sql.push_str(&group_by_sql.join(\", \"));").unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(out, "        if !having_sql.is_empty() {{").unwrap();
    writeln!(out, "            sql.push_str(\" HAVING \");").unwrap();
    writeln!(
        out,
        "            sql.push_str(&having_sql.join(\" AND \"));"
    )
    .unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(out, "        if !order_sql.is_empty() {{").unwrap();
    writeln!(out, "            sql.push_str(\" ORDER BY \");").unwrap();
    writeln!(out, "            sql.push_str(&order_sql.join(\", \"));").unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(out, "        if let Some(off) = offset {{").unwrap();
    writeln!(out, "            sql.push_str(\" OFFSET \");").unwrap();
    writeln!(out, "            sql.push_str(&off.to_string());").unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(out, "        if let Some(l) = limit {{").unwrap();
    writeln!(out, "            sql.push_str(\" LIMIT \");").unwrap();
    writeln!(out, "            sql.push_str(&l.to_string());").unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(
        out,
        "        if let Some(lock) = lock_sql {{ sql.push(' '); sql.push_str(lock); }}"
    )
    .unwrap();
    writeln!(out, "        let __profiler_binds = if is_sql_profiler_enabled() {{ binds.iter().chain(join_binds.iter()).chain(having_binds.iter()).map(|b| format!(\"{{}}\", b)).collect::<Vec<_>>().join(\", \") }} else {{ String::new() }};").unwrap();
    out.push_str(&render_profiler_start(skip_profiler));
    writeln!(
        out,
        "        let mut q = sqlx::query_as::<_, {row_ident}>(&sql);"
    )
    .unwrap();
    writeln!(out, "        for b in binds {{").unwrap();
    writeln!(out, "            q = bind(q, b);").unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(out, "        for b in join_binds {{ q = bind(q, b); }}").unwrap();
    writeln!(out, "        for b in having_binds {{ q = bind(q, b); }}").unwrap();
    writeln!(out, "        let rows = db.fetch_all(q).await?;").unwrap();
    out.push_str(&render_profiler_log(table, "SELECT", "&sql", "&__profiler_binds", skip_profiler));
    if !relations.is_empty() {
        writeln!(
            out,
            "        let m = {model_ident} {{ db: db.clone(), base_url: base_url.clone() }};"
        )
        .unwrap();
        out.push_str(&render_relation_loader_bindings(relations));
    }
    out.push_str(&render_support_data_loaders(
        model_snake,
        pk,
        parent_pk_ty,
        localized_fields,
        has_meta,
        has_attachments,
        "rows",
        "db.clone()",
    ));
    if !relations.is_empty() {
        out.push_str(&render_with_relations_collection_build(
            model_title,
            relations,
            pk,
            "rows",
            "r",
            "out_vec",
            localized_fields,
            has_meta,
            has_attachments,
            "base_url.as_deref()",
        ));
    } else {
        out.push_str(&render_view_collection_build(
            "out_vec",
            "r",
            "rows",
            localized_fields,
            has_meta,
            has_attachments,
            "base_url.as_deref()",
        ));
        // Wrap views into WithRelations
        writeln!(out, "        let out_vec: Vec<{model_title}WithRelations> = out_vec.into_iter().map(|v| {model_title}WithRelations {{ row: v }}).collect();").unwrap();
    }
    writeln!(out, "        Ok(out_vec)").unwrap();
    writeln!(out, "    }}\n").unwrap();
    out
}

fn render_with_counts_method(
    model_title: &str,
    view_ident: &str,
    has_many_rels: &[&RelationSpec],
    pk: &str,
) -> String {
    let mut out = String::new();
    if has_many_rels.is_empty() {
        return out;
    }

    let rel_ident = format!("{}Rel", model_title);
    writeln!(
        out,
        "    pub async fn with_counts(self, rels: &[{rel_ident}]) -> Result<(Vec<{view_ident}>, std::collections::HashMap<String, std::collections::HashMap<i64, i64>>)> {{"
    )
    .unwrap();
    writeln!(out, "        let db = self.db.clone();").unwrap();
    writeln!(out, "        let rows = self.get().await?;").unwrap();
    writeln!(
        out,
        "        let ids: Vec<i64> = rows.iter().map(|r| r.{pk}).collect();"
    )
    .unwrap();
    writeln!(
        out,
        "        if ids.is_empty() {{ return Ok((rows, std::collections::HashMap::new())); }}"
    )
    .unwrap();
    writeln!(
        out,
        "        let mut all_counts: std::collections::HashMap<String, std::collections::HashMap<i64, i64>> = std::collections::HashMap::new();"
    )
    .unwrap();
    writeln!(out, "        for rel in rels {{").unwrap();
    writeln!(
        out,
        "            let placeholders: Vec<String> = (1..=ids.len()).map(|i| format!(\"${{}}\", i)).collect();"
    )
    .unwrap();
    writeln!(out, "            let sql = format!(").unwrap();
    writeln!(
        out,
        "                \"SELECT {{}}, COUNT(*) as cnt FROM {{}} WHERE {{}} IN ({{}}) GROUP BY {{}}\","
    )
    .unwrap();
    writeln!(
        out,
        "                rel.foreign_key(), rel.target_table(), rel.foreign_key(), placeholders.join(\", \"), rel.foreign_key()"
    )
    .unwrap();
    writeln!(out, "            );").unwrap();
    writeln!(
        out,
        "            let mut q = sqlx::query_as::<_, (i64, i64)>(&sql);"
    )
    .unwrap();
    writeln!(out, "            for id in &ids {{ q = q.bind(*id); }}").unwrap();
    writeln!(
        out,
        "            let count_rows: Vec<(i64, i64)> = db.fetch_all(q).await?;"
    )
    .unwrap();
    writeln!(
        out,
        "            let counts: std::collections::HashMap<i64, i64> = count_rows.into_iter().collect();"
    )
    .unwrap();
    writeln!(
        out,
        "            all_counts.insert(rel.name().to_string(), counts);"
    )
    .unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(out, "        Ok((rows, all_counts))").unwrap();
    writeln!(out, "    }}\n").unwrap();
    out
}

fn render_first_or_create_method(insert_ident: &str, model_title: &str, model_ident: &str, pk: &str) -> String {
    let wr = format!("{model_title}WithRelations");
    let pk_snake = to_snake(pk);
    let mut out = String::new();
    writeln!(
        out,
        "    pub async fn first_or_create(self, create: impl FnOnce({insert_ident}<'db>) -> {insert_ident}<'db>) -> Result<{wr}> {{"
    )
    .unwrap();
    writeln!(out, "        let db = self.db.clone();").unwrap();
    writeln!(out, "        let base_url = self.base_url.clone();").unwrap();
    writeln!(
        out,
        "        if let Some(existing) = self.first().await? {{"
    )
    .unwrap();
    writeln!(out, "            return Ok(existing);").unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(
        out,
        "        let insert_builder = create({insert_ident}::new(db.clone(), base_url.clone()));"
    )
    .unwrap();
    writeln!(out, "        let view = insert_builder.save().await?;").unwrap();
    writeln!(
        out,
        "        {model_ident}::new(db, base_url).query().find(view.{pk_snake}).await.map(|r| r.unwrap())"
    )
    .unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(out).unwrap();
    out
}

fn render_update_or_create_method(
    update_ident: &str,
    insert_ident: &str,
    model_title: &str,
    model_ident: &str,
    pk: &str,
) -> String {
    let wr = format!("{model_title}WithRelations");
    let pk_snake = to_snake(pk);
    let mut out = String::new();
    writeln!(out, "    pub async fn update_or_create(").unwrap();
    writeln!(out, "        self,").unwrap();
    writeln!(
        out,
        "        on_update: impl FnOnce({update_ident}<'db>) -> {update_ident}<'db>,"
    )
    .unwrap();
    writeln!(
        out,
        "        on_create: impl FnOnce({insert_ident}<'db>) -> {insert_ident}<'db>,"
    )
    .unwrap();
    writeln!(out, "    ) -> Result<{wr}> {{").unwrap();
    writeln!(out, "        let db = self.db.clone();").unwrap();
    writeln!(out, "        let base_url = self.base_url.clone();").unwrap();
    writeln!(out, "        let where_sql = self.where_sql.clone();").unwrap();
    writeln!(out, "        let binds = self.binds.clone();").unwrap();
    writeln!(
        out,
        "        if let Some(existing) = self.first().await? {{"
    )
    .unwrap();
    writeln!(
        out,
        "            let mut update_builder = {update_ident}::new(db.clone(), base_url.clone());"
    )
    .unwrap();
    writeln!(out, "            update_builder.where_sql = where_sql;").unwrap();
    writeln!(out, "            update_builder.binds = binds;").unwrap();
    writeln!(
        out,
        "            let update_builder = on_update(update_builder);"
    )
    .unwrap();
    writeln!(out, "            update_builder.save().await?;").unwrap();
    writeln!(
        out,
        "            return {model_ident}::new(db, base_url.clone()).query().find(existing.{pk_snake}.clone()).await.map(|r| r.unwrap());"
    )
    .unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(
        out,
        "        let insert_builder = on_create({insert_ident}::new(db.clone(), base_url.clone()));"
    )
    .unwrap();
    writeln!(out, "        let view = insert_builder.save().await?;").unwrap();
    writeln!(
        out,
        "        {model_ident}::new(db, base_url).query().find(view.{pk_snake}).await.map(|r| r.unwrap())"
    )
    .unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(out).unwrap();
    out
}

fn render_increment_method(col_ident: &str, table: &str, has_soft_delete: bool, skip_profiler: bool) -> String {
    let mut out = String::new();
    writeln!(
        out,
        "    pub async fn increment(self, col: {col_ident}, amount: i64) -> Result<u64> {{"
    )
    .unwrap();
    writeln!(out, "        let db = self.db.clone();").unwrap();
    writeln!(out, "        let mut where_sql = self.where_sql;").unwrap();
    writeln!(out, "        let binds = self.binds;").unwrap();
    if has_soft_delete {
        writeln!(out, "        if HAS_SOFT_DELETE && !self.with_deleted {{").unwrap();
        writeln!(
            out,
            "            where_sql.push(format!(\"{{}} IS NULL\", {col_ident}::DeletedAt.as_sql()));"
        )
        .unwrap();
        writeln!(out, "        }}").unwrap();
    }
    writeln!(
        out,
        "        let where_clause = if where_sql.is_empty() {{ String::new() }} else {{ format!(\" WHERE {{}}\", where_sql.join(\" AND \")) }};"
    )
    .unwrap();
    writeln!(
        out,
        "        let sql = format!(\"UPDATE {table} SET {{}} = {{}} + {{}} {{}}\", col.as_sql(), col.as_sql(), amount, where_clause);"
    )
    .unwrap();
    writeln!(out, "        let __profiler_binds = if is_sql_profiler_enabled() {{ binds.iter().map(|b| format!(\"{{}}\", b)).collect::<Vec<_>>().join(\", \") }} else {{ String::new() }};").unwrap();
    out.push_str(&render_profiler_start(skip_profiler));
    writeln!(out, "        let mut q = sqlx::query(&sql);").unwrap();
    writeln!(out, "        for b in binds {{ q = bind_query(q, b); }}").unwrap();
    writeln!(out, "        let res = db.execute(q).await?;").unwrap();
    out.push_str(&render_profiler_log(table, "UPDATE", "&sql", "&__profiler_binds", skip_profiler));
    writeln!(out, "        Ok(res.rows_affected())").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(out).unwrap();
    out
}

fn render_decrement_method(col_ident: &str) -> String {
    let mut out = String::new();
    writeln!(
        out,
        "    pub async fn decrement(self, col: {col_ident}, amount: i64) -> Result<u64> {{"
    )
    .unwrap();
    writeln!(out, "        self.increment(col, -amount).await").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(out).unwrap();
    out
}

pub fn generate_models(
    schema: &Schema,
    cfgs: &ConfigsFile,
    out_dir: &Path,
) -> Result<(), Box<dyn Error>> {
    generate_models_with_options(schema, cfgs, out_dir, GenerateModelsOptions::default())
}

pub fn generate_models_with_options(
    schema: &Schema,
    cfgs: &ConfigsFile,
    out_dir: &Path,
    options: GenerateModelsOptions,
) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(out_dir)?;

    let mut model_module_exports = String::new();

    for (name, cfg) in &schema.models {
        let file_stem = to_snake(name);
        let model_title = to_title_case(&file_stem);
        let code = render_model(name, cfg, schema, cfgs, options);
        fs::write(out_dir.join(format!("{file_stem}.rs")), code)?;

        let pk = cfg.pk.clone().unwrap_or_else(|| "id".to_string());
        let fields = parse_fields(cfg, &pk);
        let has_many_rel = parse_relations(schema, cfg, name, &fields)
            .iter()
            .any(|r| matches!(r.kind, RelationKind::HasMany));

        let mut exports = vec![
            model_title.clone(),
            format!("{model_title}View"),
            format!("{model_title}WithRelations"),
            format!("{model_title}Query"),
            format!("{model_title}Insert"),
            format!("{model_title}Update"),
            format!("{model_title}Col"),
            format!("{model_title}ViewsExt"),
        ];
        if options.include_datatable {
            exports.push(format!("{model_title}TableAdapter"));
            exports.push(format!("{model_title}DataTable"));
            exports.push(format!("{model_title}DataTableConfig"));
            exports.push(format!("{model_title}DataTableHooks"));
            exports.push(format!("{model_title}DefaultDataTableHooks"));
        }
        if has_many_rel {
            exports.push(format!("{model_title}Rel"));
        }

        writeln!(model_module_exports, "pub mod {};", file_stem)?;
        writeln!(
            model_module_exports,
            "pub use {}::{{{}}};",
            file_stem,
            exports.join(", ")
        )?;
    }

    fs::write(out_dir.join("common.rs"), generate_common())?;
    let mut mod_context = TemplateContext::new();
    mod_context.insert(
        "model_module_exports",
        model_module_exports.trim_end().to_string(),
    )?;
    fs::write(
        out_dir.join("mod.rs"),
        render_template("models/mod.rs.tpl", &mod_context)?,
    )?;
    Ok(())
}

fn render_model(
    name: &str,
    cfg: &ModelSpec,
    schema: &Schema,
    cfgs: &ConfigsFile,
    options: GenerateModelsOptions,
) -> String {
    let model_snake = to_snake(name);
    let model_title = to_title_case(&model_snake);
    let row_ident = format!("{}Row", model_title);
    let view_ident = format!("{}View", model_title);
    let col_ident = format!("{}Col", model_title);
    let model_ident = model_title.clone();
    let query_ident = format!("{}Query", model_title);
    let unsafe_query_ident = format!("{}UnsafeQuery", model_title);
    let insert_ident = format!("{}Insert", model_title);
    let update_ident = format!("{}Update", model_title);
    let unsafe_update_ident = format!("{}UnsafeUpdate", model_title);
    let json_ident = format!("{}Json", model_title);
    let model_snake_upper = model_snake.to_uppercase();

    let table = cfg.table.as_deref().unwrap_or(&model_snake).to_string();
    let pk = cfg.pk.clone().unwrap_or_else(|| "id".to_string());
    let pk_col_variant = to_title_case(&pk);

    let fields = parse_fields(cfg, &pk);
    let meta_fields = parse_meta(cfg);
    let has_meta = !meta_fields.is_empty();
    let attachment_fields = parse_attachments(cfg);
    for att in &attachment_fields {
        if !cfgs.attachment_types.contains_key(att.typ.as_str()) {
            panic!(
                "Attachment type '{}' on model '{}' field '{}' is not defined in configs.toml attachment_type.*",
                att.typ, name, att.name
            );
        }
    }
    let single_attachments: Vec<AttachmentFieldSpec> = attachment_fields
        .iter()
        .cloned()
        .filter(|a| !a.multiple)
        .collect();
    let multi_attachments: Vec<AttachmentFieldSpec> = attachment_fields
        .iter()
        .cloned()
        .filter(|a| a.multiple)
        .collect();
    let has_attachments = !attachment_fields.is_empty();
    let localized_fields: Vec<String> = cfg
        .localized
        .clone()
        .unwrap_or_default()
        .into_iter()
        .map(|s| to_snake(&s))
        .collect();
    let localized_set: BTreeSet<String> = localized_fields.iter().cloned().collect();
    let db_fields: Vec<FieldSpec> = fields
        .iter()
        .cloned()
        .filter(|f| !localized_set.contains(&f.name))
        .collect();
    let enum_type_names: BTreeSet<String> = schema
        .extra_sections
        .iter()
        .filter_map(|(name, section)| match section {
            EnumOrOther::Enum(spec) if spec.type_name == "enum" => Some(name.clone()),
            _ => None,
        })
        .collect();
    let enum_explained_fields: Vec<EnumExplainedFieldSpec> = db_fields
        .iter()
        .filter_map(|field| enum_field_spec(&field.name, &field.ty, &enum_type_names))
        .collect();

    let parent_pk_ty = fields
        .iter()
        .find(|f| f.name == pk)
        .map(|f| f.ty.clone())
        .unwrap_or_else(|| "i64".to_string());
    let id_strategy = cfg
        .id_strategy
        .as_deref()
        .unwrap_or(if parent_pk_ty == "i64" {
            "snowflake"
        } else {
            "manual"
        });
    if id_strategy != "snowflake" && id_strategy != "manual" {
        panic!(
            "Model '{}' has invalid id_strategy '{}'. Supported: snowflake, manual",
            name, id_strategy
        );
    }
    if id_strategy == "snowflake" && parent_pk_ty != "i64" {
        panic!(
            "Model '{}' uses id_strategy='snowflake' but pk_type='{}'. Snowflake currently requires i64 PK.",
            name, parent_pk_ty
        );
    }
    let use_snowflake_id = !cfg.disable_id && id_strategy == "snowflake" && parent_pk_ty == "i64";
    // Emit lifecycle hooks only for audit-enabled models with i64 PKs
    let emit_hooks = cfg.audit && parent_pk_ty == "i64";
    // Skip profiler instrumentation for models with `profile = false`
    let skip_profiler = !cfg.profile;
    let has_created_at = fields.iter().any(|f| f.name == "created_at");
    let has_updated_at = fields.iter().any(|f| f.name == "updated_at");
    let has_soft_delete = fields.iter().any(|f| f.name == "deleted_at");
    let relations = parse_relations(schema, cfg, name, &fields);
    let relation_paths = collect_relation_paths(schema, name, DATATABLE_REL_FILTER_MAX_DEPTH);
    let computed_fields = parse_computed(cfg);
    let hidden_fields: BTreeSet<String> = cfg
        .hidden
        .clone()
        .unwrap_or_default()
        .into_iter()
        .map(|s| to_snake(&s))
        .collect();

    // Parse touch relations
    let mut touch_targets: Vec<(String, String, String, String, String)> = Vec::new(); // (fk_col, target_model_snake, target_model_title, target_pk, target_pk_ty)
    if let Some(touches) = &cfg.touch {
        for rel_name in touches {
            if let Some(rel) = relations
                .iter()
                .find(|r| &r.name == rel_name && matches!(r.kind, RelationKind::BelongsTo))
            {
                let target_snake = to_snake(&rel.target_model);
                let target_title = to_title_case(&rel.target_model);
                touch_targets.push((
                    rel.foreign_key.clone(),
                    target_snake,
                    target_title,
                    rel.target_pk.clone(),
                    rel.target_pk_ty.clone(),
                ));
            } else {
                panic!(
                    "Model '{}' configures touch='{}' but no such belongs_to relation found.",
                    name, rel_name
                );
            }
        }
    }
    let base_select = fields
        .iter()
        .map(|f| f.name.clone())
        .collect::<Vec<_>>()
        .join(", ");

    let mut imports = String::new();
    writeln!(imports, "use anyhow::Result;").unwrap();
    if !relations.is_empty() || !localized_fields.is_empty() || has_meta {
        writeln!(imports, "use std::collections::HashMap;").unwrap();
    }
    if has_meta {
        writeln!(imports, "use serde_json::Value as JsonValue;").unwrap();
    }
    writeln!(imports, "use serde::{{Deserialize, Serialize}};").unwrap();
    writeln!(imports, "use schemars::JsonSchema;").unwrap();

    writeln!(imports, "use sqlx::FromRow;").unwrap();
    if use_snowflake_id {
        writeln!(
            imports,
            "use core_db::common::sql::{{BindValue, Op, OrderDir, RawClause, RawGroupExpr, RawJoinKind, RawJoinSpec, RawOrderExpr, RawSelectExpr, SetMode, bind, bind_query, bind_scalar, generate_snowflake_i64, is_sql_profiler_enabled, format_duration, record_profiled_query, DbConn}};"
        )
        .unwrap();
    } else {
        writeln!(
            imports,
            "use core_db::common::sql::{{BindValue, Op, OrderDir, RawClause, RawGroupExpr, RawJoinKind, RawJoinSpec, RawOrderExpr, RawSelectExpr, SetMode, bind, bind_query, bind_scalar, is_sql_profiler_enabled, format_duration, record_profiled_query, DbConn}};"
        )
        .unwrap();
    }
    writeln!(
        imports,
        "use core_db::common::pagination::resolve_per_page;"
    )
    .unwrap();
    if options.include_datatable {
        writeln!(
            imports,
            "use core_datatable::{{AutoDataTable, BoxFuture, DataTableColumnDescriptor, DataTableContext, DataTableInput, DataTableRelationColumnDescriptor, GeneratedTableAdapter, ParsedFilter, SortDirection}};"
        )
        .unwrap();
    }
    if has_attachments {
        writeln!(imports, "use core_db::platform::attachments::types::{{Attachment, AttachmentInput, AttachmentMap}};").unwrap();
        writeln!(imports, "use uuid::Uuid;").unwrap();
    }
    writeln!(
        imports,
        "use core_db::platform::localized::types::LocalizedMap;"
    )
    .unwrap();
    writeln!(
        imports,
        "use crate::generated::models::common::{{Page, renumber_placeholders}};"
    )
    .unwrap();
    writeln!(
        imports,
        "use core_db::common::collection::TypedCollectionExt;"
    )
    .unwrap();
    if has_meta {
        writeln!(imports, "use core_db::platform::meta::types::MetaMap;").unwrap();
    }
    if !localized_fields.is_empty() || has_meta {
        writeln!(imports, "use crate::generated::localized;").unwrap();
    }
    if !localized_fields.is_empty() {
        writeln!(imports, "use core_i18n::current_locale;").unwrap();
    }
    {
        let mut imported_models = std::collections::BTreeSet::new();
        for rel in &relations {
            let target_mod = to_snake(&rel.target_model);
            if imported_models.insert(target_mod.clone()) {
                let target_title = to_title_case(&rel.target_model);
                writeln!(
                    imports,
                    "use crate::generated::models::{}::{{{}Col, {}Query, {}Row}};",
                    target_mod, target_title, target_title, target_title
                )
                .unwrap();
            }
        }
    }
    // Check if any field uses a custom type (not a built-in, no "::" in name)
    let builtin_types = [
        "i8",
        "i16",
        "i32",
        "i64",
        "u8",
        "u16",
        "u32",
        "u64",
        "f32",
        "f64",
        "rust_decimal::Decimal",
        "bool",
        "String",
        "Vec",
        "Option",
        "uuid::Uuid",
        "time::OffsetDateTime",
    ];
    let has_custom_types = db_fields.iter().any(|f| {
        let ty = f.ty.trim();
        if enum_type_names.contains(ty) {
            return false;
        }
        if let Some(inner) = parse_option_inner_type(ty) {
            if enum_type_names.contains(inner.as_str()) {
                return false;
            }
        }
        !ty.contains("::") && !builtin_types.iter().any(|b| ty.starts_with(b))
    });
    let has_custom_meta_types = meta_fields
        .iter()
        .any(|m| matches!(&m.ty, MetaType::Custom(ty) if !ty.contains("::")));
    if options.include_extensions_imports && (has_custom_types || has_custom_meta_types) {
        writeln!(imports, "use crate::extensions::{}::types::*;", model_snake).unwrap();
    }
    if options.include_extensions_imports && !computed_fields.is_empty() {
        writeln!(
            imports,
            "use crate::extensions::{}::types::{}Computed;",
            model_snake, model_title
        )
        .unwrap();
    }
    if !localized_fields.is_empty() {
        writeln!(
            imports,
            "use crate::generated::localized::LocalizedMapHelper;"
        )
        .unwrap();
    }
    if !enum_explained_fields.is_empty() {
        writeln!(imports, "use super::enums::*;").unwrap();
    }
    if emit_hooks {
        writeln!(
            imports,
            "use core_db::common::model_observer::{{ModelEvent, try_get_observer}};"
        )
        .unwrap();
    }

    let mut constants = String::new();
    writeln!(
        constants,
        "const HAS_CREATED_AT: bool = {};",
        has_created_at
    )
    .unwrap();
    writeln!(
        constants,
        "const HAS_UPDATED_AT: bool = {};",
        has_updated_at
    )
    .unwrap();
    writeln!(
        constants,
        "const HAS_SOFT_DELETE: bool = {};",
        has_soft_delete
    )
    .unwrap();

    let mut out = String::new();

    // Row
    writeln!(
        out,
        "#[derive(Debug, Clone, FromRow, Serialize, Deserialize, JsonSchema)]"
    )
    .unwrap();
    writeln!(out, "#[doc(hidden)]").unwrap();
    writeln!(out, "pub struct {row_ident} {{").unwrap();
    for f in &db_fields {
        if let Some(attr) = f.serde_attr {
            writeln!(out, "    {attr}").unwrap();
        }
        if f.ty.contains("OffsetDateTime") {
            writeln!(out, "    #[schemars(with = \"String\")]").unwrap();
        }
        writeln!(out, "    pub {}: {},", f.name, f.ty).unwrap();
    }
    writeln!(out, "}}\n").unwrap();

    let mut view_fields: Vec<String> = Vec::new();
    for f in &db_fields {
        if f.ty.contains("OffsetDateTime") {
            view_fields.push("    #[schemars(with = \"String\")]".to_string());
        }
        view_fields.push(format!("    pub {}: {},", f.name, f.ty));
    }
    for enum_field in &enum_explained_fields {
        if enum_field.optional {
            view_fields.push(format!(
                "    pub {}: Option<String>,",
                enum_field.explained_name
            ));
        } else {
            view_fields.push(format!("    pub {}: String,", enum_field.explained_name));
        }
    }
    for f in &localized_fields {
        view_fields.push(format!("    pub {}: Option<String>,", f));
        view_fields.push(format!(
            "    pub {f}_translations: Option<localized::LocalizedText>,"
        ));
    }
    for a in &single_attachments {
        view_fields.push(format!("    pub {}: Option<Attachment>,", a.name));
        view_fields.push(format!(
            "    pub {name}_url: Option<String>,",
            name = a.name
        ));
    }
    for a in &multi_attachments {
        view_fields.push(format!("    pub {}: Vec<Attachment>,", a.name));
        view_fields.push(format!("    pub {name}_urls: Vec<String>,", name = a.name));
    }
    if has_meta {
        view_fields.push("    pub meta: std::collections::HashMap<String, JsonValue>,".to_string());
    }
    writeln!(
        out,
        "#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]"
    )
    .unwrap();
    writeln!(out, "pub struct {view_ident} {{").unwrap();
    for line in view_fields {
        writeln!(out, "{}", line).unwrap();
    }
    writeln!(out, "}}\n").unwrap();

    writeln!(out, "impl {view_ident} {{").unwrap();
    writeln!(
        out,
        "    pub fn update<'db>(&self, db: impl Into<DbConn<'db>>) -> {update_ident}<'db> {{"
    )
    .unwrap();
    writeln!(
        out,
        "        {model_ident}::new(db.into(), None).update().where_{pk}(Op::Eq, self.{pk}.clone())",
        pk = to_snake(&pk)
    )
    .unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(
        out,
        "    pub fn update_with<'db>(&self, model: &{model_ident}<'db>) -> {update_ident}<'db> {{"
    )
    .unwrap();
    writeln!(
        out,
        "        model.update().where_{pk}(Op::Eq, self.{pk}.clone())",
        pk = to_snake(&pk)
    )
    .unwrap();
    writeln!(out, "    }}").unwrap();

    writeln!(out, "    pub fn to_json(&self) -> {json_ident} {{").unwrap();
    writeln!(out, "        {json_ident} {{").unwrap();
    for f in &db_fields {
        if !hidden_fields.contains(&f.name) {
            writeln!(out, "            {}: self.{}.clone(),", f.name, f.name).unwrap();
        }
    }
    for enum_field in &enum_explained_fields {
        if !hidden_fields.contains(&enum_field.name) {
            writeln!(
                out,
                "            {}: self.{}.clone(),",
                enum_field.explained_name, enum_field.explained_name
            )
            .unwrap();
        }
    }
    for f in &localized_fields {
        if !hidden_fields.contains(f) {
            writeln!(out, "            {}: self.{}.clone(),", f, f).unwrap();
            writeln!(
                out,
                "            {f}_translations: self.{f}_translations.clone(),"
            )
            .unwrap();
        }
    }
    for a in &single_attachments {
        if !hidden_fields.contains(&a.name) {
            writeln!(out, "            {}: self.{}.clone(),", a.name, a.name).unwrap();
            writeln!(
                out,
                "            {name}_url: self.{name}_url.clone(),",
                name = a.name
            )
            .unwrap();
        }
    }
    for a in &multi_attachments {
        if !hidden_fields.contains(&a.name) {
            writeln!(out, "            {}: self.{}.clone(),", a.name, a.name).unwrap();
            writeln!(
                out,
                "            {name}_urls: self.{name}_urls.clone(),",
                name = a.name
            )
            .unwrap();
        }
    }
    if has_meta && !hidden_fields.contains("meta") {
        writeln!(out, "            meta: self.meta.clone(),").unwrap();
    }
    for c in &computed_fields {
        writeln!(out, "            {}: self.{}(),", c.name, c.name).unwrap();
    }
    writeln!(out, "        }}").unwrap();
    writeln!(out, "    }}").unwrap();
    if has_meta && !hidden_fields.contains("meta") {
        for m in &meta_fields {
            match &m.ty {
                MetaType::String => {
                    writeln!(
                        out,
                        "    pub fn meta_{name}(&self) -> Option<String> {{ self.meta.get(\"{name}\").and_then(|v| v.as_str().map(|s| s.to_string())) }}",
                        name = m.name
                    )
                    .unwrap();
                }
                MetaType::Bool => {
                    writeln!(
                        out,
                        "    pub fn meta_{name}(&self) -> Option<bool> {{ self.meta.get(\"{name}\").and_then(|v| v.as_bool()) }}",
                        name = m.name
                    )
                    .unwrap();
                }
                MetaType::I32 => {
                    writeln!(
                        out,
                        "    pub fn meta_{name}(&self) -> Option<i32> {{ self.meta.get(\"{name}\").and_then(|v| v.as_i64()).and_then(|n| i32::try_from(n).ok()) }}",
                        name = m.name
                    )
                    .unwrap();
                }
                MetaType::I64 => {
                    writeln!(
                        out,
                        "    pub fn meta_{name}(&self) -> Option<i64> {{ self.meta.get(\"{name}\").and_then(|v| v.as_i64()) }}",
                        name = m.name
                    )
                    .unwrap();
                }
                MetaType::F64 => {
                    writeln!(
                        out,
                        "    pub fn meta_{name}(&self) -> Option<f64> {{ self.meta.get(\"{name}\").and_then(|v| v.as_f64()) }}",
                        name = m.name
                    )
                    .unwrap();
                }
                MetaType::Decimal => {
                    writeln!(
                        out,
                        "    pub fn meta_{name}(&self) -> Option<rust_decimal::Decimal> {{ self.meta.get(\"{name}\").and_then(|v| v.as_str()).and_then(|s| s.parse().ok()) }}",
                        name = m.name
                    )
                    .unwrap();
                }
                MetaType::DateTime => {
                    writeln!(
                        out,
                        "    pub fn meta_{name}(&self) -> Option<time::OffsetDateTime> {{ self.meta.get(\"{name}\").and_then(|v| v.as_str()).and_then(|s| time::OffsetDateTime::parse(s, &time::format_description::well_known::Rfc3339).ok()) }}",
                        name = m.name
                    )
                    .unwrap();
                }
                MetaType::Json => {
                    writeln!(
                        out,
                        "    pub fn meta_{name}_as<T: serde::de::DeserializeOwned>(&self) -> anyhow::Result<Option<T>> {{ match self.meta.get(\"{name}\") {{ None => Ok(None), Some(v) => Ok(Some(serde_json::from_value(v.clone())?)), }} }}",
                        name = m.name
                    )
                    .unwrap();
                }
                MetaType::Custom(ty) => {
                    writeln!(
                        out,
                        "    pub fn meta_{name}(&self) -> anyhow::Result<Option<{ty}>> {{ match self.meta.get(\"{name}\") {{ None => Ok(None), Some(v) => Ok(Some(serde_json::from_value(v.clone())?)), }} }}",
                        name = m.name,
                        ty = ty
                    )
                    .unwrap();
                }
            }
        }
    }
    if !localized_fields.is_empty() {
        let model_snake_upper = model_snake.to_uppercase();
        for f in &localized_fields {
            let fn_upsert = format!("upsert_{}", to_snake(f));
            writeln!(
                out,
                "    pub async fn {fn_upsert}<'a>(&self, db: DbConn<'a>, input: Option<localized::LocalizedInput>) -> Result<()> {{"
            )
            .unwrap();
            writeln!(out, "        let Some(input) = input else {{ return Ok(()); }};").unwrap();
            writeln!(out, "        if input.is_empty() {{ return Ok(()); }}").unwrap();
            writeln!(out, "        let map = input.to_hashmap();").unwrap();
            writeln!(
                out,
                "        localized::upsert_localized_many(db, localized::{model_snake_upper}_OWNER_TYPE, self.{pk}, \"{f}\", &map).await",
                pk = to_snake(&pk)
            )
            .unwrap();
            writeln!(out, "    }}").unwrap();

            let fn_clear = format!("clear_{}", to_snake(f));
            writeln!(
                out,
                "    pub async fn {fn_clear}<'a>(&self, db: DbConn<'a>) -> Result<()> {{"
            )
            .unwrap();
            writeln!(
                out,
                "        localized::delete_localized_field(db, localized::{model_snake_upper}_OWNER_TYPE, self.{pk}, \"{f}\").await",
                pk = to_snake(&pk)
            )
            .unwrap();
            writeln!(out, "    }}").unwrap();
        }
    }
    writeln!(out, "}}\n").unwrap();

    // Collection extensions for Vec<View>
    let views_ext_ident = format!("{}ViewsExt", model_title);
    writeln!(out, "pub trait {views_ext_ident} {{").unwrap();
    writeln!(out, "    fn ids(&self) -> Vec<{parent_pk_ty}>;").unwrap();
    writeln!(
        out,
        "    fn pluck<R>(&self, f: impl Fn(&{view_ident}) -> R) -> Vec<R>;"
    )
    .unwrap();
    writeln!(
        out,
        "    fn key_by<K>(&self, f: impl Fn(&{view_ident}) -> K) -> std::collections::HashMap<K, {view_ident}> where K: Eq + std::hash::Hash;"
    )
    .unwrap();
    writeln!(
        out,
        "    fn group_by<K>(&self, f: impl Fn(&{view_ident}) -> K) -> std::collections::HashMap<K, Vec<{view_ident}>> where K: Eq + std::hash::Hash;"
    )
    .unwrap();
    writeln!(out, "}}\n").unwrap();
    writeln!(out, "impl {views_ext_ident} for Vec<{view_ident}> {{").unwrap();
    writeln!(
        out,
        "    fn ids(&self) -> Vec<{parent_pk_ty}> {{ self.as_slice().pluck_typed(|v| v.{pk}.clone()) }}",
        pk = to_snake(&pk)
    )
    .unwrap();
    writeln!(
        out,
        "    fn pluck<R>(&self, f: impl Fn(&{view_ident}) -> R) -> Vec<R> {{ self.as_slice().pluck_typed(f) }}"
    )
    .unwrap();
    writeln!(
        out,
        "    fn key_by<K>(&self, f: impl Fn(&{view_ident}) -> K) -> std::collections::HashMap<K, {view_ident}> where K: Eq + std::hash::Hash {{ self.as_slice().key_by_typed(f) }}"
    )
    .unwrap();
    writeln!(
        out,
        "    fn group_by<K>(&self, f: impl Fn(&{view_ident}) -> K) -> std::collections::HashMap<K, Vec<{view_ident}>> where K: Eq + std::hash::Hash {{ self.as_slice().group_by_typed(f) }}"
    )
    .unwrap();
    writeln!(out, "}}\n").unwrap();

    // Json Struct
    writeln!(
        out,
        "#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]"
    )
    .unwrap();
    writeln!(out, "#[doc(hidden)]").unwrap();
    writeln!(out, "pub struct {json_ident} {{").unwrap();
    for f in &db_fields {
        if !hidden_fields.contains(&f.name) {
            if f.ty.contains("OffsetDateTime") {
                writeln!(out, "    #[schemars(with = \"String\")]").unwrap();
            }
            writeln!(out, "    pub {}: {},", f.name, f.ty).unwrap();
        }
    }
    for enum_field in &enum_explained_fields {
        if !hidden_fields.contains(&enum_field.name) {
            if enum_field.optional {
                writeln!(
                    out,
                    "    pub {}: Option<String>,",
                    enum_field.explained_name
                )
                .unwrap();
            } else {
                writeln!(out, "    pub {}: String,", enum_field.explained_name).unwrap();
            }
        }
    }
    for f in &localized_fields {
        if !hidden_fields.contains(f) {
            writeln!(out, "    pub {}: Option<String>,", f).unwrap();
            writeln!(
                out,
                "    pub {f}_translations: Option<localized::LocalizedText>,"
            )
            .unwrap();
        }
    }
    for a in &single_attachments {
        if !hidden_fields.contains(&a.name) {
            writeln!(out, "    pub {}: Option<Attachment>,", a.name).unwrap();
            writeln!(out, "    pub {name}_url: Option<String>,", name = a.name).unwrap();
        }
    }
    for a in &multi_attachments {
        if !hidden_fields.contains(&a.name) {
            writeln!(out, "    pub {}: Vec<Attachment>,", a.name).unwrap();
            writeln!(out, "    pub {name}_urls: Vec<String>,", name = a.name).unwrap();
        }
    }
    if has_meta && !hidden_fields.contains("meta") {
        writeln!(
            out,
            "    pub meta: std::collections::HashMap<String, JsonValue>,"
        )
        .unwrap();
    }
    for c in &computed_fields {
        writeln!(out, "    pub {}: {},", c.name, c.ty).unwrap();
    }
    writeln!(out, "}}\n").unwrap();

    // Hydrate helper: combine DB row + localized/meta/attachments maps into view
    let loc_ident = if !localized_fields.is_empty() {
        "loc"
    } else {
        "_loc"
    };
    let base_url_ident = if has_attachments {
        "base_url"
    } else {
        "_base_url"
    };
    if has_meta && has_attachments {
        writeln!(
            out,
            "fn hydrate_view(row: {row_ident}, {loc_ident}: &LocalizedMap, meta: &MetaMap, attachments: &AttachmentMap, {base_url_ident}: Option<&str>) -> {view_ident} {{"
        )
        .unwrap();
    } else if has_meta {
        writeln!(
            out,
            "fn hydrate_view(row: {row_ident}, {loc_ident}: &LocalizedMap, meta: &MetaMap, {base_url_ident}: Option<&str>) -> {view_ident} {{"
        )
        .unwrap();
    } else if has_attachments {
        writeln!(
            out,
            "fn hydrate_view(row: {row_ident}, {loc_ident}: &LocalizedMap, attachments: &AttachmentMap, {base_url_ident}: Option<&str>) -> {view_ident} {{"
        )
        .unwrap();
    } else {
        writeln!(
            out,
            "fn hydrate_view(row: {row_ident}, {loc_ident}: &LocalizedMap, {base_url_ident}: Option<&str>) -> {view_ident} {{"
        )
        .unwrap();
    }
    if !localized_fields.is_empty() {
        writeln!(out, "    let locale = current_locale();").unwrap();
    }
    if !localized_fields.is_empty() || has_meta || has_attachments {
        writeln!(out, "    let mut view = {view_ident} {{").unwrap();
    } else {
        writeln!(out, "    let view = {view_ident} {{").unwrap();
    }
    for f in &db_fields {
        writeln!(out, "        {}: row.{},", f.name, f.name).unwrap();
    }
    for enum_field in &enum_explained_fields {
        if enum_field.optional {
            writeln!(
                out,
                "        {}: row.{}.map(|value| value.explained_label()),",
                enum_field.explained_name, enum_field.name
            )
            .unwrap();
        } else {
            writeln!(
                out,
                "        {}: row.{}.explained_label(),",
                enum_field.explained_name, enum_field.name
            )
            .unwrap();
        }
    }
    for f in &localized_fields {
        writeln!(out, "        {f}: None,").unwrap();
        writeln!(out, "        {f}_translations: None,").unwrap();
    }
    for a in &single_attachments {
        writeln!(out, "        {}: None,", a.name).unwrap();
        writeln!(out, "        {name}_url: None,", name = a.name).unwrap();
    }
    for a in &multi_attachments {
        writeln!(out, "        {}: Vec::new(),", a.name).unwrap();
        writeln!(out, "        {name}_urls: Vec::new(),", name = a.name).unwrap();
    }
    if has_meta {
        writeln!(out, "        meta: HashMap::new(),").unwrap();
    }
    writeln!(out, "    }};").unwrap();
    for f in &localized_fields {
        writeln!(
            out,
            "    let ml_{f} = {loc_ident}.get_localized_text(\"{f}\", view.id);"
        )
        .unwrap();
        writeln!(out, "    if let Some(ref ml) = ml_{f} {{").unwrap();
        writeln!(out, "        view.{f} = Some(ml.get(locale).to_string());").unwrap();
        writeln!(out, "    }}").unwrap();
        writeln!(out, "    view.{f}_translations = ml_{f};").unwrap();
    }
    if has_meta {
        writeln!(out, "    view.meta = meta.get_all_for_owner(view.id);").unwrap();
    }
    if has_attachments {
        for a in &single_attachments {
            writeln!(
                out,
                "    view.{name} = attachments.get_single(\"{name}\", view.id);",
                name = a.name
            )
            .unwrap();
            writeln!(
                out,
                "    view.{name}_url = view.{name}.as_ref().map(|a| a.url_with_base({base_url_ident}));",
                name = a.name
            )
            .unwrap();
        }
        for a in &multi_attachments {
            writeln!(
                out,
                "    view.{name} = attachments.get_many(\"{name}\", view.id);",
                name = a.name
            )
            .unwrap();
            writeln!(
                out,
                "    view.{name}_urls = view.{name}.iter().map(|a| a.url_with_base({base_url_ident})).collect();",
                name = a.name
            )
            .unwrap();
        }
    }
    writeln!(out, "    view").unwrap();
    writeln!(out, "}}\n").unwrap();

    writeln!(
        out,
        "#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]"
    )
    .unwrap();
    writeln!(out, "#[doc(hidden)]").unwrap();
    writeln!(out, "pub struct {model_title}WithRelations {{").unwrap();
    writeln!(out, "    #[serde(flatten)]").unwrap();
    writeln!(out, "    pub row: {view_ident},").unwrap();
    for rel in &relations {
        let rel_field = to_snake(&rel.name);
        let target_title = to_title_case(&rel.target_model);
        let target_row = format!("{}Row", target_title);
        match rel.kind {
            RelationKind::HasMany => {
                writeln!(out, "    pub {rel_field}: Vec<{target_row}>,").unwrap();
            }
            RelationKind::BelongsTo => {
                writeln!(out, "    pub {rel_field}: Option<{target_row}>,").unwrap();
            }
        }
    }
    writeln!(out, "}}\n").unwrap();

    writeln!(out, "impl {model_title}WithRelations {{").unwrap();
    writeln!(out, "    pub fn into_row(self) -> {view_ident} {{ self.row }}").unwrap();
    writeln!(out, "}}\n").unwrap();

    writeln!(out, "impl std::ops::Deref for {model_title}WithRelations {{").unwrap();
    writeln!(out, "    type Target = {view_ident};").unwrap();
    writeln!(out, "    fn deref(&self) -> &Self::Target {{ &self.row }}").unwrap();
    writeln!(out, "}}\n").unwrap();

    writeln!(out, "impl std::ops::DerefMut for {model_title}WithRelations {{").unwrap();
    writeln!(out, "    fn deref_mut(&mut self) -> &mut Self::Target {{ &mut self.row }}").unwrap();
    writeln!(out, "}}\n").unwrap();

    let row_view_json_section = out;
    let mut out = String::new();

    // Col enum
    writeln!(out, "#[derive(Debug, Clone, Copy, JsonSchema)]").unwrap();
    writeln!(out, "pub enum {col_ident} {{").unwrap();
    for f in &db_fields {
        writeln!(out, "    {},", to_title_case(&f.name)).unwrap();
    }
    writeln!(out, "}}\n").unwrap();

    writeln!(out, "impl {col_ident} {{").unwrap();
    writeln!(out, "    pub const fn all() -> &'static [{col_ident}] {{").unwrap();
    writeln!(
        out,
        "        &[{}]",
        db_fields
            .iter()
            .map(|f| format!("{col_ident}::{}", to_title_case(&f.name)))
            .collect::<Vec<_>>()
            .join(", ")
    )
    .unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(out, "    pub const fn as_sql(self) -> &'static str {{").unwrap();
    writeln!(out, "        match self {{").unwrap();
    for f in &db_fields {
        writeln!(
            out,
            "            {col_ident}::{} => \"{}\",",
            to_title_case(&f.name),
            f.name
        )
        .unwrap();
    }
    writeln!(out, "        }}").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(out, "}}\n").unwrap();

    // Rel enum for HasMany relations (typed with_counts)
    let has_many_rels: Vec<_> = relations
        .iter()
        .filter(|r| matches!(r.kind, RelationKind::HasMany))
        .collect();
    if !has_many_rels.is_empty() {
        let rel_ident = format!("{}Rel", model_title);
        writeln!(out, "#[derive(Debug, Clone, Copy, JsonSchema)]").unwrap();
        writeln!(out, "pub enum {rel_ident} {{").unwrap();
        for r in &has_many_rels {
            writeln!(out, "    {},", to_title_case(&r.name)).unwrap();
        }
        writeln!(out, "}}\n").unwrap();

        writeln!(out, "impl {rel_ident} {{").unwrap();
        writeln!(out, "    pub const fn name(self) -> &'static str {{").unwrap();
        writeln!(out, "        match self {{").unwrap();
        for r in &has_many_rels {
            writeln!(
                out,
                "            {rel_ident}::{} => \"{}\",",
                to_title_case(&r.name),
                r.name
            )
            .unwrap();
        }
        writeln!(out, "        }}").unwrap();
        writeln!(out, "    }}").unwrap();
        writeln!(
            out,
            "    pub const fn target_table(self) -> &'static str {{"
        )
        .unwrap();
        writeln!(out, "        match self {{").unwrap();
        for r in &has_many_rels {
            writeln!(
                out,
                "            {rel_ident}::{} => \"{}\",",
                to_title_case(&r.name),
                r.target_table
            )
            .unwrap();
        }
        writeln!(out, "        }}").unwrap();
        writeln!(out, "    }}").unwrap();
        writeln!(out, "    pub const fn foreign_key(self) -> &'static str {{").unwrap();
        writeln!(out, "        match self {{").unwrap();
        for r in &has_many_rels {
            writeln!(
                out,
                "            {rel_ident}::{} => \"{}\",",
                to_title_case(&r.name),
                r.foreign_key
            )
            .unwrap();
        }
        writeln!(out, "        }}").unwrap();
        writeln!(out, "    }}").unwrap();
        writeln!(out, "}}\n").unwrap();
    }

    // Model
    writeln!(out, "pub struct {model_ident}<'db> {{").unwrap();
    writeln!(out, "    db: DbConn<'db>,").unwrap();
    writeln!(out, "    base_url: Option<String>,").unwrap();
    writeln!(out, "}}\n").unwrap();

    writeln!(out, "impl<'db> {model_ident}<'db> {{").unwrap();
    writeln!(out, "    pub const TABLE: &'static str = \"{table}\";").unwrap();
    writeln!(out, "    pub const PK: &'static str = \"{pk}\";").unwrap();
    writeln!(out, "    pub fn new(db: impl Into<DbConn<'db>>, base_url: Option<String>) -> Self {{ Self {{ db: db.into(), base_url }} }}").unwrap();
    writeln!(
        out,
        "    pub fn query(&self) -> {query_ident}<'db> {{ {query_ident}::new(self.db.clone(), self.base_url.clone()) }}"
    )
    .unwrap();
    writeln!(
        out,
        "    pub fn insert(&self) -> {insert_ident}<'db> {{ {insert_ident}::new(self.db.clone(), self.base_url.clone()) }}"
    )
    .unwrap();
    writeln!(
        out,
        "    pub fn update(&self) -> {update_ident}<'db> {{ {update_ident}::new(self.db.clone(), self.base_url.clone()) }}"
    )
    .unwrap();
    writeln!(
        out,
        "    pub async fn find(&self, id: {parent_pk_ty}) -> Result<Option<{model_title}WithRelations>> {{"
    )
    .unwrap();
    writeln!(out, "        self.query().find(id).await").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(
        out,
        "    pub async fn delete(&self, id: {parent_pk_ty}) -> Result<u64> {{"
    )
    .unwrap();
    writeln!(
        out,
        "        self.query().where_{}(Op::Eq, id).delete().await",
        to_snake(&pk)
    )
    .unwrap();
    writeln!(out, "    }}").unwrap();
    if has_soft_delete {
        writeln!(
            out,
            "    pub async fn restore(&self, id: {parent_pk_ty}) -> Result<u64> {{"
        )
        .unwrap();
        writeln!(
            out,
            "        self.query().where_{}(Op::Eq, id).restore().await",
            to_snake(&pk)
        )
        .unwrap();
        writeln!(out, "    }}").unwrap();
    }

    // localized setters are provided on insert/update builders via *_lang methods
    for rel in &relations {
        let fn_name = format!("load_{}", to_snake(&rel.name));
        let target_title = to_title_case(&rel.target_model);
        let target_row = format!("{}Row", target_title);
        let is_fk_optional = fields
            .iter()
            .any(|f| f.name == rel.foreign_key && f.ty.starts_with("Option<"));
        match rel.kind {
            RelationKind::HasMany => {
                writeln!(out, "    pub async fn {fn_name}(&self, parents: &[{row_ident}]) -> Result<HashMap<{parent_pk_ty}, Vec<{target_row}>>> {{").unwrap();
                writeln!(
                    out,
                    "        if parents.is_empty() {{ return Ok(HashMap::new()); }}"
                )
                .unwrap();
                writeln!(out, "        let ids: Vec<{parent_pk_ty}> = parents.iter().map(|p| p.{pk}.clone()).collect();").unwrap();
                writeln!(out, "        let placeholders: Vec<String> = (1..=ids.len()).map(|i| format!(\"${{}}\", i)).collect();").unwrap();
                writeln!(out, "        let sql = format!(\"SELECT * FROM {rel_table} WHERE {fk} IN ({{}})\", placeholders.join(\", \"));", rel_table = rel.target_table, fk = rel.foreign_key).unwrap();
                writeln!(
                    out,
                    "        let mut q = sqlx::query_as::<_, {target_row}>(&sql);"
                )
                .unwrap();
                writeln!(out, "        for id in ids {{ q = bind(q, id.into()); }}").unwrap();
                writeln!(out, "        let rows = self.db.fetch_all(q).await?;").unwrap();
                writeln!(out, "        let mut map: HashMap<{parent_pk_ty}, Vec<{target_row}>> = HashMap::new();").unwrap();
                writeln!(out, "        for row in rows {{").unwrap();
                writeln!(
                    out,
                    "            map.entry(row.{fk}.clone()).or_default().push(row);",
                    fk = rel.foreign_key
                )
                .unwrap();
                writeln!(out, "        }}").unwrap();
                writeln!(out, "        Ok(map)").unwrap();
                writeln!(out, "    }}").unwrap();
            }
            RelationKind::BelongsTo => {
                writeln!(out, "    pub async fn {fn_name}(&self, parents: &[{row_ident}]) -> Result<HashMap<{parent_pk_ty}, Option<{target_row}>>> {{").unwrap();
                writeln!(
                    out,
                    "        if parents.is_empty() {{ return Ok(HashMap::new()); }}"
                )
                .unwrap();
                writeln!(out, "        let mut fk_vals = Vec::new();").unwrap();
                writeln!(out, "        let mut parent_pairs = Vec::new();").unwrap();
                writeln!(out, "        for p in parents {{").unwrap();
                if is_fk_optional {
                    writeln!(
                        out,
                        "            if let Some(fk_val) = p.{fk}.clone() {{ fk_vals.push(fk_val); parent_pairs.push((p.{pk}.clone(), Some(fk_val))); }} else {{ parent_pairs.push((p.{pk}.clone(), None)); }}",
                        fk = rel.foreign_key,
                        pk = pk
                    )
                    .unwrap();
                } else {
                    writeln!(
                        out,
                        "            fk_vals.push(p.{fk}.clone());",
                        fk = rel.foreign_key
                    )
                    .unwrap();
                    writeln!(
                        out,
                        "            parent_pairs.push((p.{pk}.clone(), Some(p.{fk}.clone())));",
                        pk = pk,
                        fk = rel.foreign_key
                    )
                    .unwrap();
                }
                writeln!(out, "        }}").unwrap();
                writeln!(
                    out,
                    "        if fk_vals.is_empty() {{ return Ok(HashMap::new()); }}"
                )
                .unwrap();
                writeln!(out, "        let placeholders: Vec<String> = (1..=fk_vals.len()).map(|i| format!(\"${{}}\", i)).collect();").unwrap();
                writeln!(out, "        let sql = format!(\"SELECT * FROM {rel_table} WHERE {target_pk} IN ({{}})\", placeholders.join(\", \"));", rel_table = rel.target_table, target_pk = rel.target_pk).unwrap();
                writeln!(
                    out,
                    "        let mut q = sqlx::query_as::<_, {target_row}>(&sql);"
                )
                .unwrap();
                writeln!(
                    out,
                    "        for fk in fk_vals {{ q = bind(q, fk.into()); }}"
                )
                .unwrap();
                writeln!(out, "        let rows = self.db.fetch_all(q).await?;").unwrap();
                writeln!(out, "        let mut by_pk: HashMap<{target_pk_ty}, {target_row}> = HashMap::new();", target_pk_ty = rel.target_pk_ty).unwrap();
                writeln!(
                    out,
                    "        for row in rows {{ by_pk.insert(row.{target_pk}.clone(), row); }}",
                    target_pk = rel.target_pk
                )
                .unwrap();
                writeln!(out, "        let mut out = HashMap::new();").unwrap();
                writeln!(out, "        for (pid, fk) in parent_pairs {{").unwrap();
                writeln!(out, "            out.insert(pid, fk.and_then(|k| by_pk.get(&k).cloned()));").unwrap();
                writeln!(out, "        }}").unwrap();
                writeln!(out, "        Ok(out)").unwrap();
                writeln!(out, "    }}").unwrap();
            }
        }
    }
    writeln!(out, "}}\n").unwrap();

    let column_model_section = out;
    let mut out = String::new();

    // Query
    writeln!(out, "#[derive(Clone)]").unwrap();
    writeln!(out, "pub struct {query_ident}<'db> {{").unwrap();
    writeln!(out, "    db: DbConn<'db>,").unwrap();
    writeln!(out, "    base_url: Option<String>,").unwrap();
    writeln!(out, "    select_sql: Option<String>,").unwrap();
    writeln!(out, "    from_sql: Option<String>,").unwrap();
    writeln!(out, "    count_sql: Option<String>,").unwrap();
    writeln!(out, "    distinct: bool,").unwrap();
    writeln!(out, "    distinct_on: Option<String>,").unwrap();
    writeln!(out, "    lock_sql: Option<&'static str>,").unwrap();
    writeln!(out, "    join_sql: Vec<String>,").unwrap();
    writeln!(out, "    join_binds: Vec<BindValue>,").unwrap();
    writeln!(out, "    where_sql: Vec<String>,").unwrap();
    writeln!(out, "    order_sql: Vec<String>,").unwrap();
    writeln!(out, "    group_by_sql: Vec<String>,").unwrap();
    writeln!(out, "    having_sql: Vec<String>,").unwrap();
    writeln!(out, "    having_binds: Vec<BindValue>,").unwrap();
    writeln!(out, "    offset: Option<i64>,").unwrap();
    writeln!(out, "    limit: Option<i64>,").unwrap();
    writeln!(out, "    binds: Vec<BindValue>,").unwrap();
    if has_soft_delete {
        writeln!(out, "    with_deleted: bool,").unwrap();
        writeln!(out, "    only_deleted: bool,").unwrap();
    }
    writeln!(out, "}}\n").unwrap();

    let query_struct_section = out;
    let mut out = String::new();

    writeln!(out, "impl<'db> {query_ident}<'db> {{").unwrap();
    writeln!(
        out,
        "    pub fn new(db: DbConn<'db>, base_url: Option<String>) -> Self {{"
    )
    .unwrap();
    if has_soft_delete {
        writeln!(
            out,
            "        Self {{ db, base_url, select_sql: Some(\"{base_select}\".to_string()), from_sql: None, count_sql: None, distinct: false, distinct_on: None, lock_sql: None, join_sql: vec![], join_binds: vec![], where_sql: vec![], order_sql: vec![], group_by_sql: vec![], having_sql: vec![], having_binds: vec![], offset: None, limit: None, binds: vec![], with_deleted: false, only_deleted: false }}"
        )
        .unwrap();
    } else {
        writeln!(
            out,
            "        Self {{ db, base_url, select_sql: Some(\"{base_select}\".to_string()), from_sql: None, count_sql: None, distinct: false, distinct_on: None, lock_sql: None, join_sql: vec![], join_binds: vec![], where_sql: vec![], order_sql: vec![], group_by_sql: vec![], having_sql: vec![], having_binds: vec![], offset: None, limit: None, binds: vec![] }}"
        )
        .unwrap();
    }
    writeln!(out, "    }}").unwrap();
    writeln!(
        out,
        "    pub fn unsafe_sql(self) -> {unsafe_query_ident}<'db> {{ {unsafe_query_ident}::new(self) }}"
    )
    .unwrap();

    out.push_str(&render_query_field_where_methods(&db_fields, &col_ident));

    writeln!(
        out,
        "    pub fn where_key(self, id: {parent_pk_ty}) -> Self {{ self.where_{pk}(Op::Eq, id) }}",
        pk = to_snake(&pk)
    )
    .unwrap();
    writeln!(
        out,
        "    pub fn where_key_in<T: Clone + Into<BindValue>>(self, vals: &[T]) -> Self {{ self.where_in({col_ident}::{pk_col_variant}, vals) }}"
    )
    .unwrap();

    writeln!(
        out,
        "    pub fn where_col<T: Into<BindValue>>(mut self, col: {col_ident}, op: Op, val: T) -> Self {{"
    )
    .unwrap();
    writeln!(out, "        let idx = self.binds.len() + 1;").unwrap();
    writeln!(
        out,
        "        self.where_sql.push(format!(\"{{}} {{}} ${{}}\", col.as_sql(), op.as_sql(), idx));"
    )
    .unwrap();
    writeln!(out, "        self.binds.push(val.into());").unwrap();
    writeln!(out, "        self").unwrap();
    writeln!(out, "    }}").unwrap();

    writeln!(
        out,
        "    fn where_raw<T: Into<BindValue>>(mut self, clause: impl Into<String>, binds: impl IntoIterator<Item = T>) -> Self {{"
    )
    .unwrap();
    writeln!(out, "        let mut clause = clause.into();").unwrap();
    writeln!(
        out,
        "        let incoming: Vec<BindValue> = binds.into_iter().map(Into::into).collect();"
    )
    .unwrap();
    writeln!(out, "        let mut idx = self.binds.len() + 1;").unwrap();
    writeln!(out, "        while let Some(pos) = clause.find('?') {{").unwrap();
    writeln!(out, "            let ph = format!(\"${{}}\", idx);").unwrap();
    writeln!(out, "            clause.replace_range(pos..pos + 1, &ph);").unwrap();
    writeln!(out, "            idx += 1;").unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(out, "        self.where_sql.push(clause);").unwrap();
    writeln!(out, "        self.binds.extend(incoming);").unwrap();
    writeln!(out, "        self").unwrap();
    writeln!(out, "    }}").unwrap();

    out.push_str(&render_where_set_methods(&col_ident));
    out.push_str(&render_null_check_methods(&col_ident));
    out.push_str(&render_or_where_methods(&col_ident));
    out.push_str(&render_where_group_methods());
    out.push_str(&render_select_list_methods(&col_ident, &base_select));
    out.push_str(&render_raw_join_method("inner_join_raw", "INNER JOIN"));
    out.push_str(&render_raw_join_method("left_join_raw", "LEFT JOIN"));
    out.push_str(&render_raw_join_method("right_join_raw", "RIGHT JOIN"));
    out.push_str(&render_raw_join_method("full_join_raw", "FULL OUTER JOIN"));
    out.push_str(&render_order_methods(&col_ident));
    out.push_str(&render_distinct_methods(&col_ident));
    out.push_str(&render_query_select_method(&col_ident));
    out.push_str(&render_simple_join_method("join", "JOIN"));
    out.push_str(&render_simple_join_method("left_join", "LEFT JOIN"));
    out.push_str(&render_simple_join_method("right_join", "RIGHT JOIN"));
    out.push_str(&render_query_source_methods());
    out.push_str(&render_where_exists_method());
    out.push_str(&render_select_subquery_method());
    out.push_str(&render_lock_methods());
    out.push_str(&render_group_by_method(&col_ident));
    out.push_str(&render_having_raw_method());
    out.push_str(&render_limit_offset_methods());
    if has_soft_delete {
        out.push_str(&render_soft_delete_scope_methods());
    }

    out.push_str(&render_query_relation_filter_methods(&relations, &table));

    let query_builder_methods_section = out;
    let mut out = String::new();

    out.push_str(&render_get_as_method(&table));
    out.push_str(&render_get_method(
        &model_title,
        &model_ident,
        &row_ident,
        has_soft_delete,
        &col_ident,
        &table,
        &model_snake,
        &pk,
        &parent_pk_ty,
        &relations,
        &localized_fields,
        has_meta,
        has_attachments,
        skip_profiler,
    ));

    out.push_str(&render_find_methods(
        &model_title,
        &parent_pk_ty,
        &table,
        &pk,
    ));

    out.push_str(&render_first_or_create_method(&insert_ident, &model_title, &model_ident, &pk));
    out.push_str(&render_update_or_create_method(
        &update_ident,
        &insert_ident,
        &model_title,
        &model_ident,
        &pk,
    ));
    out.push_str(&render_increment_method(
        &col_ident,
        &table,
        has_soft_delete,
        skip_profiler,
    ));
    out.push_str(&render_decrement_method(&col_ident));

    out.push_str(&render_count_method(has_soft_delete, &col_ident, &table, skip_profiler));

    out.push_str(&render_pluck_ids_method(
        &parent_pk_ty,
        &pk,
        has_soft_delete,
        &col_ident,
        &table,
        skip_profiler,
    ));
    out.push_str(&render_small_terminal_methods(
        &query_ident,
        &model_title,
        &col_ident,
        &pk_col_variant,
        has_created_at,
    ));
    out.push_str(&render_scalar_aggregate_method(
        "sum",
        "Option<f64>",
        "SUM({}::DOUBLE PRECISION)",
        has_soft_delete,
        &col_ident,
        &table,
        skip_profiler,
    ));

    out.push_str(&render_with_counts_method(
        &model_title,
        &view_ident,
        &has_many_rels,
        &pk,
    ));
    out.push_str(&render_scalar_aggregate_method(
        "avg",
        "Option<f64>",
        "AVG({}::DOUBLE PRECISION)",
        has_soft_delete,
        &col_ident,
        &table,
        skip_profiler,
    ));
    out.push_str(&render_scalar_aggregate_method(
        "min_val",
        "Option<i64>",
        "MIN({})",
        has_soft_delete,
        &col_ident,
        &table,
        skip_profiler,
    ));
    out.push_str(&render_scalar_aggregate_method(
        "max_val",
        "Option<i64>",
        "MAX({})",
        has_soft_delete,
        &col_ident,
        &table,
        skip_profiler,
    ));

    out.push_str(&render_paginate_method(
        &model_title,
        &model_ident,
        &row_ident,
        has_soft_delete,
        &col_ident,
        &table,
        &model_snake,
        &pk,
        &parent_pk_ty,
        &relations,
        &localized_fields,
        has_meta,
        has_attachments,
        skip_profiler,
    ));

    out.push_str(&render_to_sql_method(has_soft_delete, &col_ident, &table));
    out.push_str(&render_into_where_parts_method(&col_ident, has_soft_delete));
    out.push_str(&render_delete_method(&table, &col_ident, has_soft_delete, emit_hooks, &row_ident, &to_snake(&pk), skip_profiler));
    if has_soft_delete {
        out.push_str(&render_restore_method(&table, &col_ident, skip_profiler));
    }
    writeln!(out, "}}\n").unwrap();

    let query_terminal_methods_section = out;
    let mut out = String::new();

    writeln!(out, "#[doc(hidden)]").unwrap();
    writeln!(out, "pub struct {unsafe_query_ident}<'db> {{").unwrap();
    writeln!(out, "    inner: {query_ident}<'db>,").unwrap();
    writeln!(out, "}}\n").unwrap();
    writeln!(out, "impl<'db> {unsafe_query_ident}<'db> {{").unwrap();
    writeln!(
        out,
        "    fn new(inner: {query_ident}<'db>) -> Self {{ Self {{ inner }} }}"
    )
    .unwrap();
    writeln!(
        out,
        "    pub fn where_raw(mut self, clause: RawClause) -> Self {{ let (sql, binds) = clause.into_parts(); self.inner = self.inner.where_raw(sql, binds); self }}"
    )
    .unwrap();
    writeln!(
        out,
        "    pub fn or_where_raw(mut self, clause: RawClause) -> Self {{ let (sql, binds) = clause.into_parts(); self.inner = self.inner.or_where_raw(sql, binds); self }}"
    )
    .unwrap();
    writeln!(
        out,
        "    pub fn join_raw(mut self, spec: RawJoinSpec) -> Self {{ let (kind, table, on, binds) = spec.into_parts(); self.inner = match kind {{ RawJoinKind::Inner => self.inner.inner_join_raw(table, on, binds), RawJoinKind::Left => self.inner.left_join_raw(table, on, binds), RawJoinKind::Right => self.inner.right_join_raw(table, on, binds), RawJoinKind::Full => self.inner.full_join_raw(table, on, binds), }}; self }}"
    )
    .unwrap();
    writeln!(
        out,
        "    pub fn select_raw(mut self, expr: RawSelectExpr) -> Self {{ self.inner = self.inner.select_raw(expr.into_inner()); self }}"
    )
    .unwrap();
    writeln!(
        out,
        "    pub fn add_select_raw(mut self, expr: RawSelectExpr) -> Self {{ self.inner = self.inner.add_select_raw(expr.into_inner()); self }}"
    )
    .unwrap();
    writeln!(
        out,
        "    pub fn select_subquery(mut self, alias: impl Into<String>, sql: RawSelectExpr) -> Self {{ let alias = alias.into(); let raw = sql.into_inner(); self.inner = self.inner.select_subquery(&alias, &raw); self }}"
    )
    .unwrap();
    writeln!(
        out,
        "    pub fn from_raw(mut self, expr: RawSelectExpr) -> Self {{ let raw = expr.into_inner(); self.inner = self.inner.from_raw(&raw); self }}"
    )
    .unwrap();
    writeln!(
        out,
        "    pub fn count_sql(mut self, expr: RawSelectExpr) -> Self {{ let raw = expr.into_inner(); self.inner = self.inner.count_sql(&raw); self }}"
    )
    .unwrap();
    writeln!(
        out,
        "    pub fn where_exists(mut self, clause: RawClause) -> Self {{ let (sql, binds) = clause.into_parts(); self.inner = self.inner.where_exists(sql, binds); self }}"
    )
    .unwrap();
    writeln!(
        out,
        "    pub fn order_by_raw(mut self, expr: RawOrderExpr) -> Self {{ self.inner = self.inner.order_by_raw(expr.into_inner()); self }}"
    )
    .unwrap();
    writeln!(
        out,
        "    pub fn group_by_raw(mut self, expr: RawGroupExpr) -> Self {{ self.inner = self.inner.group_by_raw(expr.into_inner()); self }}"
    )
    .unwrap();
    writeln!(
        out,
        "    pub fn done(self) -> {query_ident}<'db> {{ self.inner }}"
    )
    .unwrap();
    writeln!(out, "}}\n").unwrap();

    let unsafe_query_section = out;
    let mut query_context = TemplateContext::new();
    query_context
        .insert(
            "query_struct_section",
            query_struct_section.trim_start().to_string(),
        )
        .unwrap();
    query_context
        .insert(
            "query_builder_methods_section",
            query_builder_methods_section.trim_start().to_string(),
        )
        .unwrap();
    query_context
        .insert(
            "query_terminal_methods_section",
            query_terminal_methods_section.trim_start().to_string(),
        )
        .unwrap();
    query_context
        .insert(
            "unsafe_query_section",
            unsafe_query_section.trim_start().to_string(),
        )
        .unwrap();
    let query_section = render_template("models/query.rs.tpl", &query_context).unwrap();
    let mut out = String::new();

    // Insert builder (DB columns + localized setters)
    writeln!(out, "pub struct {insert_ident}<'db> {{").unwrap();
    writeln!(out, "    db: DbConn<'db>,").unwrap();
    writeln!(out, "    base_url: Option<String>,").unwrap();
    writeln!(out, "    cols: Vec<{col_ident}>,").unwrap();
    writeln!(out, "    binds: Vec<BindValue>,").unwrap();
    if !localized_fields.is_empty() {
        writeln!(
            out,
            "    translations: HashMap<&'static str, HashMap<String, String>>,"
        )
        .unwrap();
    }
    if has_meta {
        writeln!(out, "    meta: HashMap<String, JsonValue>,").unwrap();
    }
    if has_attachments {
        writeln!(
            out,
            "    attachments_single: HashMap<&'static str, AttachmentInput>,"
        )
        .unwrap();
        writeln!(
            out,
            "    attachments_multi: HashMap<&'static str, Vec<AttachmentInput>>,"
        )
        .unwrap();
    }
    writeln!(out, "    conflict_action: Option<&'static str>,").unwrap();
    writeln!(out, "    conflict_cols: Vec<{col_ident}>,").unwrap();
    writeln!(out, "}}\n").unwrap();

    writeln!(out, "impl<'db> {insert_ident}<'db> {{").unwrap();
    writeln!(
        out,
        "    pub fn new(db: DbConn<'db>, base_url: Option<String>) -> Self {{"
    )
    .unwrap();
    writeln!(out, "        Self {{").unwrap();
    writeln!(out, "            db,").unwrap();
    writeln!(out, "            base_url,").unwrap();
    writeln!(out, "            cols: vec![],").unwrap();
    writeln!(out, "            binds: vec![],").unwrap();
    if !localized_fields.is_empty() {
        writeln!(out, "            translations: HashMap::new(),").unwrap();
    }
    if has_meta {
        writeln!(out, "            meta: HashMap::new(),").unwrap();
    }
    if has_attachments {
        writeln!(out, "            attachments_single: HashMap::new(),").unwrap();
        writeln!(out, "            attachments_multi: HashMap::new(),").unwrap();
    }
    writeln!(out, "            conflict_action: None,").unwrap();
    writeln!(out, "            conflict_cols: vec![],").unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(out, "    }}").unwrap();

    let insert_struct_section = out;
    let mut out = String::new();

    out.push_str(&render_insert_field_setters(&db_fields, &col_ident));
    if !localized_fields.is_empty() {
        out.push_str(&render_localized_setters(&localized_fields, cfgs));
    }
    if has_meta {
        out.push_str(&render_meta_setters(&meta_fields));
    }
    if has_attachments {
        out.push_str(&render_insert_attachment_setters(
            &single_attachments,
            &multi_attachments,
        ));
    }

    // on_conflict_do_nothing - INSERT ... ON CONFLICT DO NOTHING
    writeln!(
        out,
        "    pub fn on_conflict_do_nothing(mut self, conflict_cols: &[{col_ident}]) -> Self {{"
    )
    .unwrap();
    writeln!(out, "        self.conflict_action = Some(\"DO NOTHING\");").unwrap();
    writeln!(out, "        self.conflict_cols = conflict_cols.to_vec();").unwrap();
    writeln!(out, "        self").unwrap();
    writeln!(out, "    }}").unwrap();

    // on_conflict_update - INSERT ... ON CONFLICT (cols) DO UPDATE SET ...
    writeln!(
        out,
        "    pub fn on_conflict_update(mut self, conflict_cols: &[{col_ident}]) -> Self {{"
    )
    .unwrap();
    writeln!(out, "        self.conflict_action = Some(\"DO UPDATE\");").unwrap();
    writeln!(out, "        self.conflict_cols = conflict_cols.to_vec();").unwrap();
    writeln!(out, "        self").unwrap();
    writeln!(out, "    }}").unwrap();

    let insert_builder_methods_section = out;
    let mut out = String::new();

    writeln!(
        out,
        "    pub async fn save(self) -> Result<{view_ident}> {{"
    )
    .unwrap();
    writeln!(out, "        let db_conn = self.db.clone();").unwrap();
    writeln!(out, "        match db_conn {{").unwrap();
    writeln!(out, "            DbConn::Pool(pool) => {{").unwrap();
    writeln!(out, "                let tx = pool.begin().await?;").unwrap();
    writeln!(
        out,
        "                let tx_lock = std::sync::Arc::new(tokio::sync::Mutex::new(tx));"
    )
    .unwrap();
    writeln!(out, "                let view = {{").unwrap();
    writeln!(
        out,
        "                    let db = DbConn::tx(tx_lock.clone());"
    )
    .unwrap();
    writeln!(out, "                    self.save_with_db(db).await?").unwrap();
    writeln!(out, "                }};").unwrap();
    writeln!(
        out,
        "                let tx = std::sync::Arc::try_unwrap(tx_lock)"
    )
    .unwrap();
    writeln!(out, "                    .map_err(|_| anyhow::anyhow!(\"transaction scope still has active handles\"))?")
        .unwrap();
    writeln!(out, "                    .into_inner();").unwrap();
    writeln!(out, "                tx.commit().await?;").unwrap();
    if emit_hooks {
        writeln!(out, "                if let Some(observer) = try_get_observer() {{").unwrap();
        writeln!(out, "                    let event = ModelEvent {{ table: \"{table}\", record_id: view.{pk} }};", pk = to_snake(&pk)).unwrap();
        writeln!(out, "                    if let Ok(data) = serde_json::to_value(&view) {{").unwrap();
        writeln!(out, "                        let _ = observer.on_created(&event, &data).await;").unwrap();
        writeln!(out, "                    }}").unwrap();
        writeln!(out, "                }}").unwrap();
    }
    writeln!(out, "                Ok(view)").unwrap();
    writeln!(out, "            }}").unwrap();
    if emit_hooks {
        writeln!(out, "            DbConn::Tx(_) => {{").unwrap();
        writeln!(out, "                let view = self.save_with_db(db_conn).await?;").unwrap();
        writeln!(out, "                if let Some(observer) = try_get_observer() {{").unwrap();
        writeln!(out, "                    let event = ModelEvent {{ table: \"{table}\", record_id: view.{pk} }};", pk = to_snake(&pk)).unwrap();
        writeln!(out, "                    if let Ok(data) = serde_json::to_value(&view) {{").unwrap();
        writeln!(out, "                        let _ = observer.on_created(&event, &data).await;").unwrap();
        writeln!(out, "                    }}").unwrap();
        writeln!(out, "                }}").unwrap();
        writeln!(out, "                Ok(view)").unwrap();
        writeln!(out, "            }}").unwrap();
    } else {
        writeln!(
            out,
            "            DbConn::Tx(_) => self.save_with_db(db_conn).await,"
        )
        .unwrap();
    }
    writeln!(out, "        }}").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(out).unwrap();
    writeln!(
        out,
        "    async fn save_with_db<'tx>(self, db: DbConn<'tx>) -> Result<{view_ident}> {{"
    )
    .unwrap();
    writeln!(out, "        let mut cols = self.cols;").unwrap();
    writeln!(out, "        let mut binds = self.binds;").unwrap();
    if use_snowflake_id {
        writeln!(
            out,
            "        if !cols.iter().any(|c| matches!(c, {col_ident}::{pk_col_variant})) {{"
        )
        .unwrap();
        writeln!(out, "            cols.push({col_ident}::{pk_col_variant});").unwrap();
        writeln!(
            out,
            "            binds.push(generate_snowflake_i64().into());"
        )
        .unwrap();
        writeln!(out, "        }}").unwrap();
    }
    if has_created_at {
        writeln!(
            out,
            "        if HAS_CREATED_AT && !cols.iter().any(|c| matches!(c, {col_ident}::CreatedAt)) {{"
        )
        .unwrap();
        writeln!(
            out,
            "            let now = time::OffsetDateTime::now_utc();"
        )
        .unwrap();
        writeln!(out, "            cols.push({col_ident}::CreatedAt);").unwrap();
        writeln!(out, "            binds.push(now.into());").unwrap();
        writeln!(out, "        }}").unwrap();
    }
    if has_updated_at {
        writeln!(
            out,
            "        if HAS_UPDATED_AT && !cols.iter().any(|c| matches!(c, {col_ident}::UpdatedAt)) {{"
        )
        .unwrap();
        writeln!(
            out,
            "            let now = time::OffsetDateTime::now_utc();"
        )
        .unwrap();
        writeln!(out, "            cols.push({col_ident}::UpdatedAt);").unwrap();
        writeln!(out, "            binds.push(now.into());").unwrap();
        writeln!(out, "        }}").unwrap();
    }
    writeln!(out, "        if cols.is_empty() {{").unwrap();
    writeln!(
        out,
        "            anyhow::bail!(\"insert: no columns set\");"
    )
    .unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(
        out,
        "        let col_sql: Vec<&'static str> = cols.iter().map(|c| c.as_sql()).collect();"
    )
    .unwrap();
    writeln!(
        out,
        "        let placeholders: Vec<String> = (1..=binds.len()).map(|i| format!(\"${{}}\", i)).collect();"
    )
    .unwrap();
    writeln!(out, "        let mut sql = format!(\"INSERT INTO {{}} ({{}}) VALUES ({{}})\", \"{table}\", col_sql.join(\", \"), placeholders.join(\", \"));").unwrap();
    writeln!(out, "        if let Some(action) = self.conflict_action {{").unwrap();
    writeln!(out, "            if !self.conflict_cols.is_empty() {{").unwrap();
    writeln!(out, "                let conflict_col_sql: Vec<&'static str> = self.conflict_cols.iter().map(|c| c.as_sql()).collect();").unwrap();
    writeln!(out, "                sql.push_str(&format!(\" ON CONFLICT ({{}}) {{}}\", conflict_col_sql.join(\", \"), action));").unwrap();
    writeln!(out, "                if action == \"DO UPDATE\" {{").unwrap();
    writeln!(out, "                    let set_clauses: Vec<String> = col_sql.iter().zip(placeholders.iter())").unwrap();
    writeln!(
        out,
        "                        .filter(|(col, _)| !conflict_col_sql.contains(col))"
    )
    .unwrap();
    writeln!(
        out,
        "                        .map(|(col, ph)| format!(\"{{}} = {{}}\", col, ph))"
    )
    .unwrap();
    writeln!(out, "                        .collect();").unwrap();
    writeln!(out, "                    if !set_clauses.is_empty() {{").unwrap();
    writeln!(
        out,
        "                        sql.push_str(&format!(\" SET {{}}\", set_clauses.join(\", \")));"
    )
    .unwrap();
    writeln!(out, "                    }}").unwrap();
    writeln!(out, "                }}").unwrap();
    writeln!(out, "            }}").unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(out, "        sql.push_str(\" RETURNING *\");").unwrap();
    writeln!(out, "        let __profiler_binds = if is_sql_profiler_enabled() {{ binds.iter().map(|b| format!(\"{{}}\", b)).collect::<Vec<_>>().join(\", \") }} else {{ String::new() }};").unwrap();
    out.push_str(&render_profiler_start(skip_profiler));
    writeln!(
        out,
        "        let mut q = sqlx::query_as::<_, {row_ident}>(&sql);"
    )
    .unwrap();
    writeln!(out, "        for b in binds {{").unwrap();
    writeln!(out, "            q = bind(q, b);").unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(out, "        let row = db.fetch_one(q).await?;").unwrap();
    out.push_str(&render_profiler_log(&table, "INSERT", "&sql", "&__profiler_binds", skip_profiler));
    if !localized_fields.is_empty() {
        writeln!(out, "        if !self.translations.is_empty() {{").unwrap();
        writeln!(
            out,
            "            let supported = localized::SUPPORTED_LOCALES;"
        )
        .unwrap();
        for f in &localized_fields {
            writeln!(
                out,
                "            if let Some(map) = self.translations.get(\"{f}\") {{"
            )
            .unwrap();
            writeln!(out, "                let mut filtered = HashMap::new();").unwrap();
            writeln!(out, "                for (loc, val) in map {{").unwrap();
            writeln!(out, "                    if supported.contains(&loc.as_str()) {{ filtered.insert(loc.clone(), val.clone()); }}").unwrap();
            writeln!(out, "                }}").unwrap();
            writeln!(out, "                if !filtered.is_empty() {{").unwrap();
            writeln!(out, "                    localized::upsert_localized_many(db.clone(), localized::{}_OWNER_TYPE, row.{pk}, \"{f}\", &filtered).await?;", model_snake.to_uppercase()).unwrap();
            writeln!(out, "                }}").unwrap();
            writeln!(out, "            }}").unwrap();
        }
        writeln!(out, "        }}").unwrap();
    }
    if has_meta {
        writeln!(out, "        if !self.meta.is_empty() {{").unwrap();
        writeln!(out, "            localized::upsert_meta_many(db.clone(), localized::{model_snake_upper}_OWNER_TYPE, row.{pk}, &self.meta).await?;", model_snake_upper = model_snake.to_uppercase()).unwrap();
        writeln!(out, "        }}").unwrap();
    }
    if has_attachments {
        writeln!(out, "        if !self.attachments_single.is_empty() || !self.attachments_multi.is_empty() {{").unwrap();
        writeln!(
            out,
            "            for (field, att) in &self.attachments_single {{"
        )
        .unwrap();
        writeln!(out, "                localized::replace_single_attachment(db.clone(), localized::{}_OWNER_TYPE, row.{pk}, field, att).await?;", model_snake_upper).unwrap();
        writeln!(out, "            }}").unwrap();
        writeln!(
            out,
            "            for (field, list) in &self.attachments_multi {{"
        )
        .unwrap();
        writeln!(out, "                localized::add_attachments(db.clone(), localized::{}_OWNER_TYPE, row.{pk}, field, list).await?;", model_snake_upper).unwrap();
        writeln!(out, "            }}").unwrap();
        writeln!(out, "        }}").unwrap();
    }
    // touch parent timestamps
    for (fk, target_snake, target_title, target_pk, _target_pk_ty) in &touch_targets {
        writeln!(out, "        if let Some(parent_id) = row.{} {{", fk).unwrap();
        writeln!(
            out,
            "            crate::generated::models::{}::{}::new(db.clone()).update()",
            target_snake, target_title
        )
        .unwrap();
        writeln!(
            out,
            "                .where_{target_pk}(Op::Eq, parent_id)",
            target_pk = target_pk
        )
        .unwrap();
        writeln!(
            out,
            "                .set_updated_at(time::OffsetDateTime::now_utc())"
        )
        .unwrap();
        writeln!(out, "                .save().await?;").unwrap();
        writeln!(out, "        }}").unwrap();
    }
    if localized_fields.is_empty() {
        writeln!(out, "        let localized = LocalizedMap::default();").unwrap();
    } else {
        writeln!(out, "        let localized = localized::load_{model_snake}_localized(db, &[row.{pk}]).await?;").unwrap();
    }
    if has_attachments {
        writeln!(out, "        let attachments = localized::load_{model_snake}_attachments(db, &[row.{pk}]).await?;").unwrap();
    }
    if has_meta {
        writeln!(
            out,
            "        let meta_map = localized::load_{model_snake}_meta(db, &[row.{pk}]).await?;"
        )
        .unwrap();
        match has_attachments {
            true => writeln!(out, "        Ok(hydrate_view(row, &localized, &meta_map, &attachments, self.base_url.as_deref()))").unwrap(),
            false => writeln!(out, "        Ok(hydrate_view(row, &localized, &meta_map, self.base_url.as_deref()))").unwrap(),
        }
    } else {
        match has_attachments {
            true => writeln!(
                out,
                "        Ok(hydrate_view(row, &localized, &attachments, self.base_url.as_deref()))"
            )
            .unwrap(),
            false => {
                if localized_fields.is_empty() {
                    writeln!(out, "        Ok(hydrate_view(row, &LocalizedMap::default(), self.base_url.as_deref()))").unwrap();
                } else {
                    writeln!(
                        out,
                        "        Ok(hydrate_view(row, &localized, self.base_url.as_deref()))"
                    )
                    .unwrap();
                }
            }
        }
    }
    writeln!(out, "    }}").unwrap();
    writeln!(out, "}}").unwrap();

    let insert_save_methods_section = out;
    let mut insert_context = TemplateContext::new();
    insert_context
        .insert(
            "insert_struct_section",
            insert_struct_section.trim_start().to_string(),
        )
        .unwrap();
    insert_context
        .insert(
            "insert_builder_methods_section",
            insert_builder_methods_section.trim_start().to_string(),
        )
        .unwrap();
    insert_context
        .insert(
            "insert_save_methods_section",
            insert_save_methods_section.trim_start().to_string(),
        )
        .unwrap();
    let insert_section = render_template("models/insert.rs.tpl", &insert_context).unwrap();
    let mut out = String::new();

    // Update builder (DB columns only)
    writeln!(out, "pub struct {update_ident}<'db> {{").unwrap();
    writeln!(out, "    db: DbConn<'db>,").unwrap();
    writeln!(out, "    base_url: Option<String>,").unwrap();
    writeln!(out, "    sets: Vec<({col_ident}, BindValue, SetMode)>,").unwrap();
    writeln!(out, "    where_sql: Vec<String>,").unwrap();
    writeln!(out, "    binds: Vec<BindValue>,").unwrap();
    if !localized_fields.is_empty() {
        writeln!(
            out,
            "    translations: HashMap<&'static str, HashMap<String, String>>,"
        )
        .unwrap();
    }
    if has_meta {
        writeln!(out, "    meta: HashMap<String, JsonValue>,").unwrap();
    }
    if has_attachments {
        writeln!(
            out,
            "    attachments_single: HashMap<&'static str, AttachmentInput>,"
        )
        .unwrap();
        writeln!(
            out,
            "    attachments_multi: HashMap<&'static str, Vec<AttachmentInput>>,"
        )
        .unwrap();
        writeln!(out, "    attachments_clear_single: Vec<&'static str>,").unwrap();
        writeln!(
            out,
            "    attachments_delete_multi: HashMap<&'static str, Vec<Uuid>>,"
        )
        .unwrap();
    }
    writeln!(out, "}}\n").unwrap();

    writeln!(out, "impl<'db> {update_ident}<'db> {{").unwrap();
    writeln!(
        out,
        "    pub fn new(db: DbConn<'db>, base_url: Option<String>) -> Self {{"
    )
    .unwrap();
    writeln!(out, "        Self {{").unwrap();
    writeln!(out, "            db,").unwrap();
    writeln!(out, "            base_url,").unwrap();
    writeln!(out, "            sets: vec![],").unwrap();
    writeln!(out, "            where_sql: vec![],").unwrap();
    writeln!(out, "            binds: vec![],").unwrap();
    if !localized_fields.is_empty() {
        writeln!(out, "            translations: HashMap::new(),").unwrap();
    }
    if has_meta {
        writeln!(out, "            meta: HashMap::new(),").unwrap();
    }
    if has_attachments {
        writeln!(out, "            attachments_single: HashMap::new(),").unwrap();
        writeln!(out, "            attachments_multi: HashMap::new(),").unwrap();
        writeln!(out, "            attachments_clear_single: Vec::new(),").unwrap();
        writeln!(out, "            attachments_delete_multi: HashMap::new(),").unwrap();
    }
    writeln!(out, "        }}").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(
        out,
        "    pub fn unsafe_sql(self) -> {unsafe_update_ident}<'db> {{ {unsafe_update_ident}::new(self) }}"
    )
    .unwrap();

    let update_struct_section = out;
    let mut out = String::new();

    out.push_str(&render_update_field_setters(&db_fields, &col_ident));
    if !localized_fields.is_empty() {
        out.push_str(&render_localized_setters(&localized_fields, cfgs));
    }
    if has_meta {
        out.push_str(&render_meta_setters(&meta_fields));
    }
    if has_attachments {
        out.push_str(&render_update_attachment_setters(
            &single_attachments,
            &multi_attachments,
        ));
    }

    for f in &db_fields {
        let fn_name = format!("where_{}", to_snake(&f.name));
        writeln!(
            out,
            "    pub fn {fn_name}(mut self, op: Op, val: {typ}) -> Self {{",
            typ = f.ty
        )
        .unwrap();
        writeln!(out, "        let idx = self.binds.len() + 1;").unwrap();
        writeln!(
            out,
            "        self.where_sql.push(format!(\"{{}} {{}} ${{}}\", {col_ident}::{}.as_sql(), op.as_sql(), idx));",
            to_title_case(&f.name)
        )
        .unwrap();
        writeln!(out, "        self.binds.push(val.into());").unwrap();
        writeln!(out, "        self").unwrap();
        writeln!(out, "    }}").unwrap();
    }

    writeln!(
        out,
        "    pub fn where_col<T: Into<BindValue>>(mut self, col: {col_ident}, op: Op, val: T) -> Self {{"
    )
    .unwrap();
    writeln!(out, "        let idx = self.binds.len() + 1;").unwrap();
    writeln!(
        out,
        "        self.where_sql.push(format!(\"{{}} {{}} ${{}}\", col.as_sql(), op.as_sql(), idx));"
    )
    .unwrap();
    writeln!(out, "        self.binds.push(val.into());").unwrap();
    writeln!(out, "        self").unwrap();
    writeln!(out, "    }}").unwrap();

    writeln!(
        out,
        "    fn where_raw<T: Into<BindValue>>(mut self, clause: impl Into<String>, binds: impl IntoIterator<Item = T>) -> Self {{"
    )
    .unwrap();
    writeln!(out, "        let mut clause = clause.into();").unwrap();
    writeln!(
        out,
        "        let incoming: Vec<BindValue> = binds.into_iter().map(Into::into).collect();"
    )
    .unwrap();
    writeln!(out, "        let mut idx = self.binds.len() + 1;").unwrap();
    writeln!(out, "        while let Some(pos) = clause.find('?') {{").unwrap();
    writeln!(out, "            let ph = format!(\"${{}}\", idx);").unwrap();
    writeln!(out, "            clause.replace_range(pos..pos + 1, &ph);").unwrap();
    writeln!(out, "            idx += 1;").unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(out, "        self.where_sql.push(clause);").unwrap();
    writeln!(out, "        self.binds.extend(incoming);").unwrap();
    writeln!(out, "        self").unwrap();
    writeln!(out, "    }}").unwrap();

    let update_builder_methods_section = out;
    let mut out = String::new();

    writeln!(out, "    pub async fn save(self) -> Result<u64> {{").unwrap();
    writeln!(
        out,
        "        if self.sets.is_empty() {{ anyhow::bail!(\"update: no columns set\"); }}"
    )
    .unwrap();
    writeln!(
        out,
        "        if self.where_sql.is_empty() {{ anyhow::bail!(\"update: no conditions set\"); }}"
    )
    .unwrap();
    writeln!(out, "        let db_conn = self.db.clone();").unwrap();
    writeln!(out, "        match db_conn {{").unwrap();
    writeln!(out, "            DbConn::Pool(pool) => {{").unwrap();
    writeln!(out, "                let tx = pool.begin().await?;").unwrap();
    writeln!(
        out,
        "                let tx_lock = std::sync::Arc::new(tokio::sync::Mutex::new(tx));"
    )
    .unwrap();
    writeln!(out, "                let affected = {{").unwrap();
    writeln!(
        out,
        "                    let db = DbConn::tx(tx_lock.clone());"
    )
    .unwrap();
    writeln!(out, "                    self.save_with_db(db).await?").unwrap();
    writeln!(out, "                }};").unwrap();
    writeln!(
        out,
        "                let tx = std::sync::Arc::try_unwrap(tx_lock)"
    )
    .unwrap();
    writeln!(out, "                    .map_err(|_| anyhow::anyhow!(\"transaction scope still has active handles\"))?")
        .unwrap();
    writeln!(out, "                    .into_inner();").unwrap();
    writeln!(out, "                tx.commit().await?;").unwrap();
    writeln!(out, "                Ok(affected)").unwrap();
    writeln!(out, "            }}").unwrap();
    writeln!(
        out,
        "            DbConn::Tx(_) => self.save_with_db(db_conn).await,"
    )
    .unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(out).unwrap();
    writeln!(
        out,
        "    async fn save_with_db<'tx>(self, db: DbConn<'tx>) -> Result<u64> {{"
    )
    .unwrap();
    writeln!(
        out,
        "        let mut cols = Vec::new();\n        let mut set_binds = Vec::new();\n        let mut set_modes = Vec::new();\n        for (col, bind, mode) in self.sets {{ cols.push(col); set_binds.push(bind); set_modes.push(mode); }}"
    )
    .unwrap();
    if has_updated_at {
        writeln!(out, "        if HAS_UPDATED_AT && !cols.iter().any(|c| matches!(c, {col_ident}::UpdatedAt)) {{").unwrap();
        writeln!(
            out,
            "            let now = time::OffsetDateTime::now_utc();"
        )
        .unwrap();
        writeln!(out, "            cols.push({col_ident}::UpdatedAt);").unwrap();
        writeln!(out, "            set_binds.push(now.into());").unwrap();
        writeln!(out, "            set_modes.push(SetMode::Assign);").unwrap();
        writeln!(out, "        }}").unwrap();
    }
    writeln!(out, "        // find target ids for localized updates").unwrap();
    writeln!(out, "        let select_sql = format!(\"SELECT {pk} FROM {table} WHERE {{}}\", self.where_sql.join(\" AND \"));").unwrap();
    writeln!(
        out,
        "        let mut select_q = sqlx::query_scalar::<_, {parent_pk_ty}>(&select_sql);"
    )
    .unwrap();
    writeln!(
        out,
        "        for b in &self.binds {{ select_q = bind_scalar(select_q, b.clone()); }}"
    )
    .unwrap();
    writeln!(
        out,
        "        let target_ids = db.fetch_all_scalar(select_q).await?;"
    )
    .unwrap();
    if emit_hooks {
        let pk_snake = to_snake(&pk);
        writeln!(out, "        let __observer_active = try_get_observer().is_some();").unwrap();
        writeln!(out, "        let __old_rows_json: Vec<({parent_pk_ty}, serde_json::Value)> = if __observer_active && !target_ids.is_empty() {{").unwrap();
        writeln!(out, "            let phs: Vec<String> = (1..=target_ids.len()).map(|i| format!(\"${{}}\", i)).collect();").unwrap();
        writeln!(out, "            let fetch_sql = format!(\"SELECT * FROM {table} WHERE {pk} IN ({{}})\", phs.join(\", \"));").unwrap();
        writeln!(out, "            let mut fq = sqlx::query_as::<_, {row_ident}>(&fetch_sql);").unwrap();
        writeln!(out, "            for id in &target_ids {{ fq = fq.bind(id); }}").unwrap();
        writeln!(out, "            let rows: Vec<{row_ident}> = db.fetch_all(fq).await.unwrap_or_default();").unwrap();
        writeln!(out, "            rows.into_iter().map(|r| (r.{pk_snake}, serde_json::to_value(&r).unwrap_or_default())).collect()").unwrap();
        writeln!(out, "        }} else {{").unwrap();
        writeln!(out, "            Vec::new()").unwrap();
        writeln!(out, "        }};").unwrap();
    }
    writeln!(out, "        let mut parts: Vec<String> = Vec::new();").unwrap();
    writeln!(out, "        for (i, (c, mode)) in cols.iter().zip(set_modes.iter()).enumerate() {{").unwrap();
    writeln!(
        out,
        "            let col = c.as_sql();"
    )
    .unwrap();
    writeln!(
        out,
        "            let part = match mode {{"
    )
    .unwrap();
    writeln!(
        out,
        "                SetMode::Assign => format!(\"{{}} = ${{}}\", col, i + 1),"
    )
    .unwrap();
    writeln!(
        out,
        "                SetMode::Increment => format!(\"{{}} = {{}} + ${{}}\", col, col, i + 1),"
    )
    .unwrap();
    writeln!(
        out,
        "                SetMode::Decrement => format!(\"{{}} = {{}} - ${{}}\", col, col, i + 1),"
    )
    .unwrap();
    writeln!(
        out,
        "            }};"
    )
    .unwrap();
    writeln!(
        out,
        "            parts.push(part);"
    )
    .unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(out, "        let offset = parts.len();").unwrap();
    writeln!(out, "        let mut where_sql = self.where_sql;").unwrap();
    writeln!(out, "        let binds = self.binds;").unwrap();
    writeln!(
        out,
        "        let mut renumbered = Vec::with_capacity(where_sql.len());"
    )
    .unwrap();
    writeln!(out, "        for clause in where_sql.drain(..) {{").unwrap();
    writeln!(
        out,
        "            renumbered.push(renumber_placeholders(&clause, offset + 1));"
    )
    .unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(out, "        where_sql = renumbered;").unwrap();
    writeln!(
        out,
        "        let mut sql = String::from(\"UPDATE {table} SET \");"
    )
    .unwrap();
    writeln!(out, "        sql.push_str(&parts.join(\", \"));").unwrap();
    writeln!(out, "        if !where_sql.is_empty() {{").unwrap();
    writeln!(out, "            sql.push_str(\" WHERE \");").unwrap();
    writeln!(out, "            sql.push_str(&where_sql.join(\" AND \"));").unwrap();
    writeln!(out, "        }}").unwrap();
    writeln!(out, "        let mut q = sqlx::query(&sql);").unwrap();
    // touch parent timestamps
    if !touch_targets.is_empty() {
        writeln!(out, "        if !target_ids.is_empty() {{").unwrap();
        writeln!(out, "            fn to_params(len: usize) -> String {{ (1..=len).map(|i| format!(\"${{}}\", i)).collect::<Vec<_>>().join(\", \") }}").unwrap();
        for (fk, target_snake, target_title, target_pk, target_pk_ty) in &touch_targets {
            writeln!(
                out,
                "            let placeholders = to_params(target_ids.len());"
            )
            .unwrap();
            writeln!(out, "            let sql = format!(\"SELECT DISTINCT {} FROM {} WHERE {{}} IN ({{}})\", \"{pk}\", placeholders);", fk, table).unwrap();
            writeln!(
                out,
                "            let mut q = sqlx::query_scalar::<_, {target_pk_ty}>(&sql);"
            )
            .unwrap();
            writeln!(
                out,
                "            for id in &target_ids {{ q = bind_scalar(q, id.clone()); }}"
            )
            .unwrap();
            writeln!(
                out,
                "            let parent_ids: Vec<{target_pk_ty}> = db.fetch_all_scalar(q).await?;"
            )
            .unwrap();
            writeln!(out, "            for pid in parent_ids {{").unwrap();
            writeln!(
                out,
                "                crate::generated::models::{}::{}::new(db.clone()).update()",
                target_snake, target_title
            )
            .unwrap();
            writeln!(
                out,
                "                    .where_{target_pk}(Op::Eq, pid)",
                target_pk = target_pk
            )
            .unwrap();
            writeln!(
                out,
                "                    .set_updated_at(time::OffsetDateTime::now_utc())"
            )
            .unwrap();
            writeln!(out, "                    .save().await?;").unwrap();
            writeln!(out, "            }}").unwrap();
        }
        writeln!(out, "        }}").unwrap();
    }
    writeln!(out, "        let __profiler_binds = if is_sql_profiler_enabled() {{ set_binds.iter().chain(binds.iter()).map(|b| format!(\"{{}}\", b)).collect::<Vec<_>>().join(\", \") }} else {{ String::new() }};").unwrap();
    out.push_str(&render_profiler_start(skip_profiler));
    writeln!(
        out,
        "        for b in &set_binds {{ q = bind_query(q, b.clone()); }}"
    )
    .unwrap();
    writeln!(
        out,
        "        for b in &binds {{ q = bind_query(q, b.clone()); }}"
    )
    .unwrap();
    writeln!(out, "        let res = db.execute(q).await?;").unwrap();
    out.push_str(&render_profiler_log(&table, "UPDATE", "&sql", "&__profiler_binds", skip_profiler));
    if !localized_fields.is_empty() {
        writeln!(out, "        if res.rows_affected() > 0 && !self.translations.is_empty() && !target_ids.is_empty() {{").unwrap();
        writeln!(
            out,
            "            let supported = localized::SUPPORTED_LOCALES;"
        )
        .unwrap();
        for f in &localized_fields {
            writeln!(
                out,
                "            if let Some(map) = self.translations.get(\"{f}\") {{"
            )
            .unwrap();
            writeln!(out, "                let mut filtered = HashMap::new();").unwrap();
            writeln!(out, "                for (loc, val) in map {{").unwrap();
            writeln!(out, "                    if supported.contains(&loc.as_str()) {{ filtered.insert(loc.clone(), val.clone()); }}").unwrap();
            writeln!(out, "                }}").unwrap();
            writeln!(out, "                if !filtered.is_empty() {{").unwrap();
            writeln!(out, "                    for id in &target_ids {{").unwrap();
            writeln!(out, "                        localized::upsert_localized_many(db.clone(), localized::{}_OWNER_TYPE, id.clone(), \"{f}\", &filtered).await?;", model_snake.to_uppercase()).unwrap();
            writeln!(out, "                    }}").unwrap();
            writeln!(out, "                }}").unwrap();
            writeln!(out, "            }}").unwrap();
        }
        writeln!(out, "        }}").unwrap();
    }
    if has_meta {
        writeln!(out, "        if res.rows_affected() > 0 && !self.meta.is_empty() && !target_ids.is_empty() {{").unwrap();
        writeln!(out, "            for id in &target_ids {{").unwrap();
        writeln!(out, "                localized::upsert_meta_many(db.clone(), localized::{model_snake_upper}_OWNER_TYPE, id.clone(), &self.meta).await?;", model_snake_upper = model_snake.to_uppercase()).unwrap();
        writeln!(out, "            }}").unwrap();
        writeln!(out, "        }}").unwrap();
    }
    if has_attachments {
        writeln!(
            out,
            "        if res.rows_affected() > 0 && !target_ids.is_empty() {{"
        )
        .unwrap();
        writeln!(out, "            for id in &target_ids {{").unwrap();
        writeln!(
            out,
            "                for field in &self.attachments_clear_single {{"
        )
        .unwrap();
        writeln!(
            out,
            "                    localized::clear_attachment_field(db.clone(), localized::{}_OWNER_TYPE, id.clone(), field).await?;",
            model_snake_upper
        )
        .unwrap();
        writeln!(out, "                }}").unwrap();
        writeln!(
            out,
            "                for (field, att) in &self.attachments_single {{"
        )
        .unwrap();
        writeln!(out, "                    localized::replace_single_attachment(db.clone(), localized::{}_OWNER_TYPE, id.clone(), field, att).await?;", model_snake_upper).unwrap();
        writeln!(out, "                }}").unwrap();
        writeln!(
            out,
            "                for (field, list) in &self.attachments_multi {{"
        )
        .unwrap();
        writeln!(
            out,
            "                    localized::add_attachments(db.clone(), localized::{}_OWNER_TYPE, id.clone(), field, list).await?;",
            model_snake_upper
        )
        .unwrap();
        writeln!(out, "                }}").unwrap();
        writeln!(
            out,
            "                for (field, ids) in &self.attachments_delete_multi {{"
        )
        .unwrap();
        writeln!(out, "                    localized::delete_attachment_ids(db.clone(), localized::{}_OWNER_TYPE, id.clone(), field, ids).await?;", model_snake_upper).unwrap();
        writeln!(out, "                }}").unwrap();
        writeln!(out, "            }}").unwrap();
        writeln!(out, "        }}").unwrap();
    }
    if emit_hooks {
        writeln!(out, "        if !__old_rows_json.is_empty() && res.rows_affected() > 0 {{").unwrap();
        writeln!(out, "            if let Some(observer) = try_get_observer() {{").unwrap();
        writeln!(out, "                for (record_id, old_data) in &__old_rows_json {{").unwrap();
        writeln!(out, "                    let fetch_sql = format!(\"SELECT * FROM {table} WHERE {pk} = $1\");").unwrap();
        writeln!(out, "                    if let Ok(Some(new_row)) = db.fetch_optional(sqlx::query_as::<_, {row_ident}>(&fetch_sql).bind(record_id)).await {{").unwrap();
        writeln!(out, "                        if let Ok(new_data) = serde_json::to_value(&new_row) {{").unwrap();
        writeln!(out, "                            let event = ModelEvent {{ table: \"{table}\", record_id: *record_id }};").unwrap();
        writeln!(out, "                            let _ = observer.on_updated(&event, old_data, &new_data).await;").unwrap();
        writeln!(out, "                        }}").unwrap();
        writeln!(out, "                    }}").unwrap();
        writeln!(out, "                }}").unwrap();
        writeln!(out, "            }}").unwrap();
        writeln!(out, "        }}").unwrap();
    }
    writeln!(out, "        Ok(res.rows_affected())").unwrap();
    writeln!(out, "    }}").unwrap();

    // returning_row/returning_view intentionally omitted.
    // save() is the single mutation path with side-effect parity and atomic behavior.

    writeln!(out, "}}").unwrap();

    let update_save_methods_section = out;
    let mut out = String::new();

    writeln!(out, "#[doc(hidden)]").unwrap();
    writeln!(out, "pub struct {unsafe_update_ident}<'db> {{").unwrap();
    writeln!(out, "    inner: {update_ident}<'db>,").unwrap();
    writeln!(out, "}}\n").unwrap();
    writeln!(out, "impl<'db> {unsafe_update_ident}<'db> {{").unwrap();
    writeln!(
        out,
        "    fn new(inner: {update_ident}<'db>) -> Self {{ Self {{ inner }} }}"
    )
    .unwrap();
    writeln!(
        out,
        "    pub fn where_raw(mut self, clause: RawClause) -> Self {{ let (sql, binds) = clause.into_parts(); self.inner = self.inner.where_raw(sql, binds); self }}"
    )
    .unwrap();
    writeln!(
        out,
        "    pub fn done(self) -> {update_ident}<'db> {{ self.inner }}"
    )
    .unwrap();
    writeln!(out, "}}").unwrap();

    let unsafe_update_section = out;
    let mut update_context = TemplateContext::new();
    update_context
        .insert(
            "update_struct_section",
            update_struct_section.trim_start().to_string(),
        )
        .unwrap();
    update_context
        .insert(
            "update_builder_methods_section",
            update_builder_methods_section.trim_start().to_string(),
        )
        .unwrap();
    update_context
        .insert(
            "update_save_methods_section",
            update_save_methods_section.trim_start().to_string(),
        )
        .unwrap();
    update_context
        .insert(
            "unsafe_update_section",
            unsafe_update_section.trim_start().to_string(),
        )
        .unwrap();
    let update_section = render_template("models/update.rs.tpl", &update_context).unwrap();
    let mut out = String::new();

    if options.include_datatable {
        let table_adapter_ident = format!("{}TableAdapter", model_title);
        let sortable_cols_lit = db_fields
            .iter()
            .filter(|f| !f.ty.contains("serde_json"))
            .map(|f| format!("\"{}\"", f.name))
            .collect::<Vec<_>>()
            .join(", ");
        let timestamp_cols_lit = db_fields
            .iter()
            .filter(|f| f.ty.contains("OffsetDateTime"))
            .map(|f| format!("\"{}\"", f.name))
            .collect::<Vec<_>>()
            .join(", ");

        let column_filter_ops_lit = |f: &FieldSpec| -> &'static str {
            if f.ty.contains("String") {
                "&[\"eq\", \"like\", \"gte\", \"lte\"]"
            } else if f.ty.contains("OffsetDateTime") {
                "&[\"eq\", \"gte\", \"lte\", \"date_from\", \"date_to\"]"
            } else {
                "&[\"eq\", \"gte\", \"lte\"]"
            }
        };

        writeln!(out, "pub struct {table_adapter_ident};").unwrap();
        writeln!(out, "impl {table_adapter_ident} {{").unwrap();
        writeln!(
            out,
            "    fn parse_col(name: &str) -> Option<{col_ident}> {{"
        )
        .unwrap();
        writeln!(out, "        match name {{").unwrap();
        for f in &db_fields {
            writeln!(
                out,
                "            \"{name}\" => Some({col_ident}::{variant}),",
                name = f.name,
                variant = to_title_case(&f.name)
            )
            .unwrap();
        }
        writeln!(out, "            _ => None,").unwrap();
        writeln!(out, "        }}").unwrap();
        writeln!(out, "    }}").unwrap();

        writeln!(
        out,
        "    fn parse_locale_field_for_relation(relation: &str, column: &str) -> Option<&'static str> {{"
    )
    .unwrap();
        writeln!(out, "        match (relation, column) {{").unwrap();
        for rel_path in &relation_paths {
            let rel_key = rel_path.path.join("__");
            let target_cfg = schema
                .models
                .get(&rel_path.target_model)
                .unwrap_or_else(|| {
                    panic!(
                        "Relation path '{}' target model not found",
                        rel_path.target_model
                    )
                });
            let target_localized_fields: Vec<String> = target_cfg
                .localized
                .clone()
                .unwrap_or_default()
                .into_iter()
                .map(|s| to_snake(&s))
                .collect();
            for tf in &target_localized_fields {
                writeln!(
                    out,
                    "            (\"{rel}\", \"{col}\") => Some(\"{col}\"),",
                    rel = rel_key,
                    col = tf
                )
                .unwrap();
            }
        }
        writeln!(out, "            _ => None,").unwrap();
        writeln!(out, "        }}").unwrap();
        writeln!(out, "    }}").unwrap();

        writeln!(
            out,
            "    fn parse_locale_field(name: &str) -> Option<&'static str> {{"
        )
        .unwrap();
        writeln!(out, "        match name {{").unwrap();
        for f in &localized_fields {
            writeln!(out, "            \"{f}\" => Some(\"{f}\"),").unwrap();
        }
        writeln!(out, "            _ => None,").unwrap();
        writeln!(out, "        }}").unwrap();
        writeln!(out, "    }}").unwrap();

        writeln!(
            out,
            "    fn parse_like_col(name: &str) -> Option<{col_ident}> {{"
        )
        .unwrap();
        writeln!(out, "        match name {{").unwrap();
        for f in &db_fields {
            if f.ty.contains("String") {
                writeln!(
                    out,
                    "            \"{name}\" => Some({col_ident}::{variant}),",
                    name = f.name,
                    variant = to_title_case(&f.name)
                )
                .unwrap();
            }
        }
        writeln!(out, "            _ => None,").unwrap();
        writeln!(out, "        }}").unwrap();
        writeln!(out, "    }}").unwrap();

        let parse_bind_expr = |ty: &str, raw: &str| -> String {
            match ty {
                "String" => format!("Some({raw}.trim().to_string().into())"),
                "bool" => format!("{raw}.trim().parse::<bool>().ok().map(Into::into)"),
                "i8" => format!("{raw}.trim().parse::<i8>().ok().map(|v| (v as i64).into())"),
                "i16" => format!("{raw}.trim().parse::<i16>().ok().map(Into::into)"),
                "i32" => format!("{raw}.trim().parse::<i32>().ok().map(Into::into)"),
                "i64" => format!("{raw}.trim().parse::<i64>().ok().map(Into::into)"),
                "u8" => format!("{raw}.trim().parse::<u8>().ok().map(|v| (v as i64).into())"),
                "u16" => format!("{raw}.trim().parse::<u16>().ok().map(|v| (v as i64).into())"),
                "u32" => format!("{raw}.trim().parse::<u32>().ok().map(|v| (v as i64).into())"),
                "u64" => format!("{raw}.trim().parse::<u64>().ok().map(|v| (v as i64).into())"),
                "f32" => format!("{raw}.trim().parse::<f32>().ok().map(|v| (v as f64).into())"),
                "f64" => format!("{raw}.trim().parse::<f64>().ok().map(Into::into)"),
                "rust_decimal::Decimal" => format!("{raw}.trim().parse::<rust_decimal::Decimal>().ok().map(Into::into)"),
                "uuid::Uuid" => format!("uuid::Uuid::parse_str({raw}.trim()).ok().map(Into::into)"),
                "time::OffsetDateTime" => {
                    format!("Self::parse_datetime({raw}.trim(), false).map(Into::into)")
                }
                "Option<time::OffsetDateTime>" => {
                    format!("Self::parse_datetime({raw}.trim(), false).map(Into::into)")
                }
                _ => format!("Some(Self::parse_bind({raw}.trim()))"),
            }
        };

        let cursor_value_expr = |ty: &str, field_name: &str| -> Option<String> {
            match ty {
            "String" => Some(format!("Some(row.{field_name}.clone())")),
            "bool" | "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" | "f32"
            | "f64" | "rust_decimal::Decimal" | "uuid::Uuid" => Some(format!("Some(row.{field_name}.to_string())")),
            "Option<String>" | "Option<bool>" | "Option<i8>" | "Option<i16>" | "Option<i32>"
            | "Option<i64>" | "Option<u8>" | "Option<u16>" | "Option<u32>" | "Option<u64>"
            | "Option<f32>" | "Option<f64>" | "Option<rust_decimal::Decimal>" | "Option<uuid::Uuid>" => {
                Some(format!("row.{field_name}.as_ref().map(|v| v.to_string())"))
            }
            "time::OffsetDateTime" => Some(format!(
                "row.{field_name}.format(&time::format_description::well_known::Rfc3339).ok()"
            )),
            "Option<time::OffsetDateTime>" => Some(format!(
                "row.{field_name}.as_ref().and_then(|v| v.format(&time::format_description::well_known::Rfc3339).ok())"
            )),
            _ => None,
        }
        };

        writeln!(
            out,
            "    fn parse_bind_for_col(name: &str, raw: &str) -> Option<BindValue> {{"
        )
        .unwrap();
        writeln!(out, "        match name {{").unwrap();
        for f in &db_fields {
            writeln!(
                out,
                "            \"{name}\" => {expr},",
                name = f.name,
                expr = parse_bind_expr(&f.ty, "raw")
            )
            .unwrap();
        }
        writeln!(out, "            _ => None,").unwrap();
        writeln!(out, "        }}").unwrap();
        writeln!(out, "    }}").unwrap();

        writeln!(
        out,
        "    fn parse_bind_for_relation(relation: &str, column: &str, raw: &str) -> Option<BindValue> {{"
    )
    .unwrap();
        writeln!(out, "        match (relation, column) {{").unwrap();
        for rel_path in &relation_paths {
            let rel_key = rel_path.path.join("__");
            let target_cfg = schema
                .models
                .get(&rel_path.target_model)
                .unwrap_or_else(|| {
                    panic!(
                        "Relation path '{}' target model not found",
                        rel_path.target_model
                    )
                });
            let target_pk = target_cfg.pk.clone().unwrap_or_else(|| "id".to_string());
            let target_fields = parse_fields(target_cfg, &target_pk);
            for tf in &target_fields {
                writeln!(
                    out,
                    "            (\"{rel}\", \"{col}\") => {expr},",
                    rel = rel_key,
                    col = tf.name,
                    expr = parse_bind_expr(&tf.ty, "raw")
                )
                .unwrap();
            }
        }
        writeln!(out, "            _ => None,").unwrap();
        writeln!(out, "        }}").unwrap();
        writeln!(out, "    }}").unwrap();

        writeln!(out, "    fn parse_bind(raw: &str) -> BindValue {{").unwrap();
        writeln!(out, "        let trimmed = raw.trim();").unwrap();
        writeln!(
        out,
        "        let lower = trimmed.to_ascii_lowercase(); if lower == \"true\" {{ return true.into(); }} if lower == \"false\" {{ return false.into(); }}"
    )
    .unwrap();
        writeln!(
            out,
            "        if let Ok(v) = trimmed.parse::<i64>() {{ return v.into(); }}"
        )
        .unwrap();
        writeln!(
            out,
            "        if let Ok(v) = trimmed.parse::<rust_decimal::Decimal>() {{ return v.into(); }}"
        )
        .unwrap();
        writeln!(
            out,
            "        if let Ok(v) = trimmed.parse::<f64>() {{ return v.into(); }}"
        )
        .unwrap();
        writeln!(
            out,
            "        if let Ok(v) = uuid::Uuid::parse_str(trimmed) {{ return v.into(); }}"
        )
        .unwrap();
        writeln!(
            out,
            "        if let Some(v) = Self::parse_datetime(trimmed, false) {{ return v.into(); }}"
        )
        .unwrap();
        writeln!(out, "        trimmed.to_string().into()").unwrap();
        writeln!(out, "    }}").unwrap();

        writeln!(
            out,
            "    fn parse_datetime(raw: &str, end_of_day: bool) -> Option<time::OffsetDateTime> {{"
        )
        .unwrap();
        writeln!(out, "        let trimmed = raw.trim();").unwrap();
        writeln!(
        out,
        "        if let Ok(dt) = time::OffsetDateTime::parse(trimmed, &time::format_description::well_known::Rfc3339) {{ return Some(dt); }}"
    )
    .unwrap();
        writeln!(out, "        if trimmed.len() == 10 {{").unwrap();
        writeln!(
        out,
        "            let date = time::Date::parse(trimmed, &time::macros::format_description!(\"[year]-[month]-[day]\")).ok()?;"
    )
    .unwrap();
        writeln!(
        out,
        "            let t = if end_of_day {{ time::Time::from_hms(23, 59, 59).ok()? }} else {{ time::Time::MIDNIGHT }};"
    )
    .unwrap();
        writeln!(
            out,
            "            return Some(date.with_time(t).assume_offset(time::UtcOffset::UTC));"
        )
        .unwrap();
        writeln!(out, "        }}").unwrap();
        writeln!(out, "        None").unwrap();
        writeln!(out, "    }}").unwrap();
        writeln!(out, "}}").unwrap();

        writeln!(
            out,
            "impl GeneratedTableAdapter for {table_adapter_ident} {{"
        )
        .unwrap();
        writeln!(out, "    type Query<'db> = {query_ident}<'db>;").unwrap();
        let wr_ident = format!("{model_title}WithRelations");
        writeln!(out, "    type Row = {wr_ident};").unwrap();
        writeln!(
            out,
            "    fn model_key(&self) -> &'static str {{ \"{model_title}\" }}"
        )
        .unwrap();
        writeln!(
        out,
        "    fn sortable_columns(&self) -> &'static [&'static str] {{ &[{sortable_cols_lit}] }}"
    )
        .unwrap();
        writeln!(
        out,
        "    fn timestamp_columns(&self) -> &'static [&'static str] {{ &[{timestamp_cols_lit}] }}"
    )
        .unwrap();
        writeln!(
            out,
            "    fn column_descriptors(&self) -> &'static [DataTableColumnDescriptor] {{"
        )
        .unwrap();
        writeln!(out, "        &[").unwrap();
        for f in &db_fields {
            let ops = column_filter_ops_lit(f);
            let label = crate::schema::to_label(&f.name);
            let sortable = !f.ty.contains("serde_json");
            writeln!(
            out,
            "            DataTableColumnDescriptor {{ name: \"{name}\", label: \"{label}\", data_type: \"{ty}\", sortable: {sortable}, localized: false, filter_ops: {ops} }},",
            name = f.name,
            ty = f.ty,
        )
        .unwrap();
        }
        for f in &localized_fields {
            let label = crate::schema::to_label(f);
            writeln!(
            out,
            "            DataTableColumnDescriptor {{ name: \"{name}\", label: \"{label}\", data_type: \"String\", sortable: false, localized: true, filter_ops: &[\"locale_eq\", \"locale_like\"] }},",
            name = f,
        )
        .unwrap();
        }
        writeln!(out, "        ]").unwrap();
        writeln!(out, "    }}").unwrap();
        writeln!(
        out,
        "    fn relation_column_descriptors(&self) -> &'static [DataTableRelationColumnDescriptor] {{"
    )
    .unwrap();
        writeln!(out, "        &[").unwrap();
        for rel_path in &relation_paths {
            let rel_key = rel_path.path.join("__");
            let target_cfg = schema
                .models
                .get(&rel_path.target_model)
                .unwrap_or_else(|| {
                    panic!(
                        "Relation path '{}' target model not found",
                        rel_path.target_model
                    )
                });
            let target_pk = target_cfg.pk.clone().unwrap_or_else(|| "id".to_string());
            let target_fields = parse_fields(target_cfg, &target_pk);
            for tf in &target_fields {
                let ops = if tf.ty.contains("String") {
                    "&[\"has_eq\", \"has_like\"]"
                } else {
                    "&[\"has_eq\"]"
                };
                writeln!(
                out,
                "            DataTableRelationColumnDescriptor {{ relation: \"{relation}\", column: \"{column}\", data_type: \"{ty}\", filter_ops: {ops} }},",
                relation = rel_key,
                column = tf.name,
                ty = tf.ty
            )
            .unwrap();
            }
            let target_localized_fields: Vec<String> = target_cfg
                .localized
                .clone()
                .unwrap_or_default()
                .into_iter()
                .map(|s| to_snake(&s))
                .collect();
            for tf in &target_localized_fields {
                writeln!(
                out,
                "            DataTableRelationColumnDescriptor {{ relation: \"{relation}\", column: \"{column}\", data_type: \"String\", filter_ops: &[\"locale_has_eq\", \"locale_has_like\"] }},",
                relation = rel_key,
                column = tf
            )
            .unwrap();
            }
        }
        writeln!(out, "        ]").unwrap();
        writeln!(out, "    }}").unwrap();
        writeln!(
            out,
            "    fn filter_patterns(&self) -> &'static [&'static str] {{"
        )
        .unwrap();
        writeln!(out, "        &[").unwrap();
        writeln!(out, "            \"f-<col>\",").unwrap();
        writeln!(out, "            \"f-like-<col>\",").unwrap();
        writeln!(out, "            \"f-gte-<col>\",").unwrap();
        writeln!(out, "            \"f-lte-<col>\",").unwrap();
        writeln!(out, "            \"f-date-from-<col>\",").unwrap();
        writeln!(out, "            \"f-date-to-<col>\",").unwrap();
        writeln!(out, "            \"f-like-any-<col1|col2|...>\",").unwrap();
        writeln!(out, "            \"f-any-<col1|col2|...>\",").unwrap();
        writeln!(out, "            \"f-has-<relation>-<col>\",").unwrap();
        writeln!(out, "            \"f-has-like-<relation>-<col>\",").unwrap();
        if !localized_fields.is_empty() {
            writeln!(out, "            \"f-locale-<col>\",").unwrap();
            writeln!(out, "            \"f-locale-like-<col>\",").unwrap();
        }
        let has_relation_locale = relation_paths.iter().any(|rel_path| {
            let Some(target_cfg) = schema.models.get(&rel_path.target_model) else {
                return false;
            };
            !target_cfg.localized.clone().unwrap_or_default().is_empty()
        });
        if has_relation_locale {
            writeln!(out, "            \"f-locale-has-<relation>-<col>\",").unwrap();
            writeln!(out, "            \"f-locale-has-like-<relation>-<col>\",").unwrap();
        }
        writeln!(out, "        ]").unwrap();
        writeln!(out, "    }}").unwrap();
        writeln!(
        out,
        "    fn apply_auto_filter<'db>(&self, query: {query_ident}<'db>, filter: &ParsedFilter, value: &str) -> anyhow::Result<Option<{query_ident}<'db>>> where Self: 'db {{"
    )
    .unwrap();
        writeln!(out, "        let trimmed = value.trim();").unwrap();
        writeln!(
            out,
            "        if trimmed.is_empty() {{ return Ok(Some(query)); }}"
        )
        .unwrap();
        writeln!(out, "        match filter {{").unwrap();
        writeln!(out, "            ParsedFilter::Eq {{ column }} => {{").unwrap();
        writeln!(
        out,
        "                let Some(col) = Self::parse_col(column.as_str()) else {{ return Ok(None); }};"
    )
    .unwrap();
        writeln!(
        out,
        "                let Some(bind) = Self::parse_bind_for_col(column.as_str(), trimmed) else {{ return Ok(None); }};"
    )
    .unwrap();
        writeln!(
            out,
            "                Ok(Some(query.where_col(col, Op::Eq, bind)))"
        )
        .unwrap();
        writeln!(out, "            }}").unwrap();
        writeln!(out, "            ParsedFilter::Like {{ column }} => {{").unwrap();
        writeln!(
        out,
        "                let Some(col) = Self::parse_like_col(column.as_str()) else {{ return Ok(None); }};"
    )
    .unwrap();
        writeln!(
        out,
        "                Ok(Some(query.where_col(col, Op::Like, format!(\"%{{}}%\", trimmed))))"
    )
        .unwrap();
        writeln!(out, "            }}").unwrap();
        writeln!(out, "            ParsedFilter::Gte {{ column }} => {{").unwrap();
        writeln!(
        out,
        "                let Some(col) = Self::parse_col(column.as_str()) else {{ return Ok(None); }};"
    )
    .unwrap();
        writeln!(
        out,
        "                let Some(bind) = Self::parse_bind_for_col(column.as_str(), trimmed) else {{ return Ok(None); }};"
    )
    .unwrap();
        writeln!(
            out,
            "                Ok(Some(query.where_col(col, Op::Ge, bind)))"
        )
        .unwrap();
        writeln!(out, "            }}").unwrap();
        writeln!(out, "            ParsedFilter::Lte {{ column }} => {{").unwrap();
        writeln!(
        out,
        "                let Some(col) = Self::parse_col(column.as_str()) else {{ return Ok(None); }};"
    )
    .unwrap();
        writeln!(
        out,
        "                let Some(bind) = Self::parse_bind_for_col(column.as_str(), trimmed) else {{ return Ok(None); }};"
    )
    .unwrap();
        writeln!(
            out,
            "                Ok(Some(query.where_col(col, Op::Le, bind)))"
        )
        .unwrap();
        writeln!(out, "            }}").unwrap();
        writeln!(out, "            ParsedFilter::DateFrom {{ column }} => {{").unwrap();
        writeln!(
        out,
        "                let Some(col) = Self::parse_col(column.as_str()) else {{ return Ok(None); }};"
    )
    .unwrap();
        writeln!(
        out,
        "                let Some(ts) = Self::parse_datetime(trimmed, false) else {{ return Ok(None); }};"
    )
    .unwrap();
        writeln!(
            out,
            "                Ok(Some(query.where_col(col, Op::Ge, ts)))"
        )
        .unwrap();
        writeln!(out, "            }}").unwrap();
        writeln!(out, "            ParsedFilter::DateTo {{ column }} => {{").unwrap();
        writeln!(
        out,
        "                let Some(col) = Self::parse_col(column.as_str()) else {{ return Ok(None); }};"
    )
    .unwrap();
        writeln!(
        out,
        "                let Some(ts) = Self::parse_datetime(trimmed, true) else {{ return Ok(None); }};"
    )
    .unwrap();
        writeln!(
            out,
            "                Ok(Some(query.where_col(col, Op::Le, ts)))"
        )
        .unwrap();
        writeln!(out, "            }}").unwrap();
        writeln!(out, "            ParsedFilter::LocaleEq {{ column }} => {{").unwrap();
        if localized_fields.is_empty() {
            writeln!(out, "                Ok(None)").unwrap();
        } else {
            writeln!(
            out,
            "                let Some(field) = Self::parse_locale_field(column.as_str()) else {{ return Ok(None); }};"
        )
        .unwrap();
            writeln!(
                out,
                "                let locale = core_i18n::current_locale().to_string();"
            )
            .unwrap();
            writeln!(
            out,
            "                let clause = \"EXISTS (SELECT 1 FROM localized l WHERE l.owner_type = ? AND l.owner_id = {table}.{pk} AND l.field = ? AND l.locale = ? AND l.value = ?)\".to_string();",
        )
        .unwrap();
            writeln!(
            out,
            "                Ok(Some(query.where_exists(clause, vec![localized::{model_snake_upper}_OWNER_TYPE.to_string(), field.to_string(), locale, trimmed.to_string()])))",
        )
        .unwrap();
        }
        writeln!(out, "            }}").unwrap();
        writeln!(
            out,
            "            ParsedFilter::LocaleLike {{ column }} => {{"
        )
        .unwrap();
        if localized_fields.is_empty() {
            writeln!(out, "                Ok(None)").unwrap();
        } else {
            writeln!(
            out,
            "                let Some(field) = Self::parse_locale_field(column.as_str()) else {{ return Ok(None); }};"
        )
        .unwrap();
            writeln!(
                out,
                "                let locale = core_i18n::current_locale().to_string();"
            )
            .unwrap();
            writeln!(
                out,
                "                let pattern = format!(\"%{{}}%\", trimmed);"
            )
            .unwrap();
            writeln!(
            out,
            "                let clause = \"EXISTS (SELECT 1 FROM localized l WHERE l.owner_type = ? AND l.owner_id = {table}.{pk} AND l.field = ? AND l.locale = ? AND l.value LIKE ?)\".to_string();",
        )
        .unwrap();
            writeln!(
            out,
            "                Ok(Some(query.where_exists(clause, vec![localized::{model_snake_upper}_OWNER_TYPE.to_string(), field.to_string(), locale, pattern])))",
        )
        .unwrap();
        }
        writeln!(out, "            }}").unwrap();
        writeln!(out, "            ParsedFilter::LikeAny {{ columns }} => {{").unwrap();
        writeln!(out, "                let mut applied = false;").unwrap();
        writeln!(
            out,
            "                let pattern = format!(\"%{{}}%\", trimmed);"
        )
        .unwrap();
        writeln!(
            out,
            "                let next = query.where_group(|group| {{"
        )
        .unwrap();
        writeln!(out, "                    let mut q = group;").unwrap();
        writeln!(out, "                    for column in columns {{").unwrap();
        writeln!(
            out,
            "                        if let Some(col) = Self::parse_like_col(column.as_str()) {{"
        )
        .unwrap();
        writeln!(
        out,
        "                            if applied {{ q = q.or_where_col(col, Op::Like, pattern.clone()); }} else {{ q = q.where_col(col, Op::Like, pattern.clone()); applied = true; }}"
    )
    .unwrap();
        writeln!(out, "                        }}").unwrap();
        writeln!(out, "                    }}").unwrap();
        writeln!(out, "                    q").unwrap();
        writeln!(out, "                }});").unwrap();
        writeln!(
            out,
            "                if applied {{ Ok(Some(next)) }} else {{ Ok(None) }}"
        )
        .unwrap();
        writeln!(out, "            }}").unwrap();
        writeln!(out, "            ParsedFilter::Any {{ columns }} => {{").unwrap();
        writeln!(out, "                let mut applied = false;").unwrap();
        writeln!(
            out,
            "                let next = query.where_group(|group| {{"
        )
        .unwrap();
        writeln!(out, "                    let mut q = group;").unwrap();
        writeln!(out, "                    for column in columns {{").unwrap();
        writeln!(
            out,
            "                        if let Some(col) = Self::parse_col(column.as_str()) {{"
        )
        .unwrap();
        writeln!(
        out,
        "                            if let Some(bind) = Self::parse_bind_for_col(column.as_str(), trimmed) {{ if applied {{ q = q.or_where_col(col, Op::Eq, bind.clone()); }} else {{ q = q.where_col(col, Op::Eq, bind.clone()); applied = true; }} }}"
    )
    .unwrap();
        writeln!(out, "                        }}").unwrap();
        writeln!(out, "                    }}").unwrap();
        writeln!(out, "                    q").unwrap();
        writeln!(out, "                }});").unwrap();
        writeln!(
            out,
            "                if applied {{ Ok(Some(next)) }} else {{ Ok(None) }}"
        )
        .unwrap();
        writeln!(out, "            }}").unwrap();
        writeln!(
            out,
            "            ParsedFilter::Has {{ relation, column }} => {{"
        )
        .unwrap();
        writeln!(
            out,
            "                match (relation.as_str(), column.as_str()) {{"
        )
        .unwrap();
        for rel_path in &relation_paths {
            let rel_key = rel_path.path.join("__");
            let target_title = to_title_case(&rel_path.target_model);
            let target_col_ident = format!("{}Col", target_title);
            let target_cfg = schema
                .models
                .get(&rel_path.target_model)
                .unwrap_or_else(|| {
                    panic!(
                        "Relation path '{}' target model not found",
                        rel_path.target_model
                    )
                });
            let target_pk = target_cfg.pk.clone().unwrap_or_else(|| "id".to_string());
            let target_fields = parse_fields(target_cfg, &target_pk);
            for tf in &target_fields {
                let leaf_expr = format!(
                    "{{var}}.where_col({target_col_ident}::{target_variant}, Op::Eq, bind)",
                    target_variant = to_title_case(&tf.name)
                );
                let has_expr = build_nested_where_has_expr(&rel_path.path, &leaf_expr, "query");
                writeln!(
                out,
                "                    (\"{rel_name}\", \"{col}\") => {{ let Some(bind) = Self::parse_bind_for_relation(\"{rel_name}\", \"{col}\", trimmed) else {{ return Ok(None); }}; Ok(Some({has_expr})) }},",
                rel_name = rel_key,
                col = tf.name,
            )
            .unwrap();
            }
        }
        writeln!(out, "                    _ => Ok(None),").unwrap();
        writeln!(out, "                }}").unwrap();
        writeln!(out, "            }}").unwrap();
        writeln!(
            out,
            "            ParsedFilter::HasLike {{ relation, column }} => {{"
        )
        .unwrap();
        writeln!(
            out,
            "                let pattern = format!(\"%{{}}%\", trimmed);"
        )
        .unwrap();
        writeln!(
            out,
            "                match (relation.as_str(), column.as_str()) {{"
        )
        .unwrap();
        for rel_path in &relation_paths {
            let rel_key = rel_path.path.join("__");
            let target_title = to_title_case(&rel_path.target_model);
            let target_col_ident = format!("{}Col", target_title);
            let target_cfg = schema
                .models
                .get(&rel_path.target_model)
                .unwrap_or_else(|| {
                    panic!(
                        "Relation path '{}' target model not found",
                        rel_path.target_model
                    )
                });
            let target_pk = target_cfg.pk.clone().unwrap_or_else(|| "id".to_string());
            let target_fields = parse_fields(target_cfg, &target_pk);
            for tf in &target_fields {
                if tf.ty.contains("String") {
                    let leaf_expr = format!(
                    "{{var}}.where_col({target_col_ident}::{target_variant}, Op::Like, pattern.clone())",
                    target_variant = to_title_case(&tf.name)
                );
                    let has_like_expr =
                        build_nested_where_has_expr(&rel_path.path, &leaf_expr, "query");
                    writeln!(
                    out,
                    "                    (\"{rel_name}\", \"{col}\") => Ok(Some({has_like_expr})),",
                    rel_name = rel_key,
                    col = tf.name,
                )
                    .unwrap();
                }
            }
        }
        writeln!(out, "                    _ => Ok(None),").unwrap();
        writeln!(out, "                }}").unwrap();
        writeln!(out, "            }}").unwrap();
        writeln!(
            out,
            "            ParsedFilter::LocaleHas {{ relation, column }} => {{"
        )
        .unwrap();
        writeln!(
        out,
        "                let Some(field) = Self::parse_locale_field_for_relation(relation.as_str(), column.as_str()) else {{ return Ok(None); }};"
    )
    .unwrap();
        writeln!(
            out,
            "                let locale = core_i18n::current_locale().to_string();"
        )
        .unwrap();
        writeln!(
            out,
            "                match (relation.as_str(), column.as_str()) {{"
        )
        .unwrap();
        for rel_path in &relation_paths {
            let rel_key = rel_path.path.join("__");
            let target_cfg = schema
                .models
                .get(&rel_path.target_model)
                .unwrap_or_else(|| {
                    panic!(
                        "Relation path '{}' target model not found",
                        rel_path.target_model
                    )
                });
            let target_table = target_cfg
                .table
                .as_ref()
                .map(|s| s.to_string())
                .unwrap_or_else(|| to_snake(&rel_path.target_model));
            let target_pk = target_cfg
                .pk
                .as_ref()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "id".to_string());
            let target_localized_fields: Vec<String> = target_cfg
                .localized
                .clone()
                .unwrap_or_default()
                .into_iter()
                .map(|s| to_snake(&s))
                .collect();
            let target_owner_const = format!(
                "{}_OWNER_TYPE",
                to_snake(&rel_path.target_model).to_uppercase()
            );
            for tf in &target_localized_fields {
                let leaf_expr = format!(
                "{{var}}.where_exists(\"EXISTS (SELECT 1 FROM localized l WHERE l.owner_type = ? AND l.owner_id = {target_table}.{target_pk} AND l.field = ? AND l.locale = ? AND l.value = ?)\".to_string(), vec![localized::{target_owner_const}.to_string(), field.to_string(), locale.clone(), trimmed.to_string()])"
            );
                let has_expr = build_nested_where_has_expr(&rel_path.path, &leaf_expr, "query");
                writeln!(
                    out,
                    "                    (\"{rel}\", \"{col}\") => Ok(Some({has_expr})),",
                    rel = rel_key,
                    col = tf,
                )
                .unwrap();
            }
        }
        writeln!(out, "                    _ => Ok(None),").unwrap();
        writeln!(out, "                }}").unwrap();
        writeln!(out, "            }}").unwrap();
        writeln!(
            out,
            "            ParsedFilter::LocaleHasLike {{ relation, column }} => {{"
        )
        .unwrap();
        writeln!(
        out,
        "                let Some(field) = Self::parse_locale_field_for_relation(relation.as_str(), column.as_str()) else {{ return Ok(None); }};"
    )
    .unwrap();
        writeln!(
            out,
            "                let locale = core_i18n::current_locale().to_string();"
        )
        .unwrap();
        writeln!(
            out,
            "                let pattern = format!(\"%{{}}%\", trimmed);"
        )
        .unwrap();
        writeln!(
            out,
            "                match (relation.as_str(), column.as_str()) {{"
        )
        .unwrap();
        for rel_path in &relation_paths {
            let rel_key = rel_path.path.join("__");
            let target_cfg = schema
                .models
                .get(&rel_path.target_model)
                .unwrap_or_else(|| {
                    panic!(
                        "Relation path '{}' target model not found",
                        rel_path.target_model
                    )
                });
            let target_table = target_cfg
                .table
                .as_ref()
                .map(|s| s.to_string())
                .unwrap_or_else(|| to_snake(&rel_path.target_model));
            let target_pk = target_cfg
                .pk
                .as_ref()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "id".to_string());
            let target_localized_fields: Vec<String> = target_cfg
                .localized
                .clone()
                .unwrap_or_default()
                .into_iter()
                .map(|s| to_snake(&s))
                .collect();
            let target_owner_const = format!(
                "{}_OWNER_TYPE",
                to_snake(&rel_path.target_model).to_uppercase()
            );
            for tf in &target_localized_fields {
                let leaf_expr = format!(
                "{{var}}.where_exists(\"EXISTS (SELECT 1 FROM localized l WHERE l.owner_type = ? AND l.owner_id = {target_table}.{target_pk} AND l.field = ? AND l.locale = ? AND l.value LIKE ?)\".to_string(), vec![localized::{target_owner_const}.to_string(), field.to_string(), locale.clone(), pattern.clone()])"
            );
                let has_expr = build_nested_where_has_expr(&rel_path.path, &leaf_expr, "query");
                writeln!(
                    out,
                    "                    (\"{rel}\", \"{col}\") => Ok(Some({has_expr})),",
                    rel = rel_key,
                    col = tf,
                )
                .unwrap();
            }
        }
        writeln!(out, "                    _ => Ok(None),").unwrap();
        writeln!(out, "                }}").unwrap();
        writeln!(out, "            }}").unwrap();
        writeln!(out, "        }}").unwrap();
        writeln!(out, "    }}").unwrap();

        writeln!(
        out,
        "    fn apply_sort<'db>(&self, query: {query_ident}<'db>, column: &str, dir: SortDirection) -> anyhow::Result<{query_ident}<'db>> where Self: 'db {{"
    )
    .unwrap();
        writeln!(
        out,
        "        let dir = match dir {{ SortDirection::Asc => OrderDir::Asc, SortDirection::Desc => OrderDir::Desc }};"
    )
    .unwrap();
        writeln!(out, "        let next = match column {{").unwrap();
        for f in &db_fields {
            writeln!(
                out,
                "            \"{name}\" => query.order_by({col_ident}::{variant}, dir),",
                name = f.name,
                variant = to_title_case(&f.name)
            )
            .unwrap();
        }
        writeln!(out, "            _ => query,").unwrap();
        writeln!(out, "        }};").unwrap();
        writeln!(out, "        Ok(next)").unwrap();
        writeln!(out, "    }}").unwrap();

        writeln!(
        out,
        "    fn apply_cursor<'db>(&self, query: {query_ident}<'db>, column: &str, dir: SortDirection, cursor: &str) -> anyhow::Result<Option<{query_ident}<'db>>> where Self: 'db {{"
    )
    .unwrap();
        writeln!(
            out,
            "        let Some(col) = Self::parse_col(column) else {{ return Ok(None); }};"
        )
        .unwrap();
        writeln!(
        out,
        "        let Some(bind) = Self::parse_bind_for_col(column, cursor) else {{ return Ok(None); }};"
    )
    .unwrap();
        writeln!(
        out,
        "        let op = match dir {{ SortDirection::Asc => Op::Gt, SortDirection::Desc => Op::Lt }};"
    )
    .unwrap();
        writeln!(out, "        Ok(Some(query.where_col(col, op, bind)))").unwrap();
        writeln!(out, "    }}").unwrap();

        writeln!(
            out,
            "    fn cursor_from_row(&self, row: &{wr_ident}, column: &str) -> Option<String> {{"
        )
        .unwrap();
        writeln!(out, "        match column {{").unwrap();
        for f in &db_fields {
            let Some(expr) = cursor_value_expr(&f.ty, &f.name) else {
                continue;
            };
            writeln!(
                out,
                "            \"{name}\" => {expr},",
                name = f.name,
                expr = expr
            )
            .unwrap();
        }
        writeln!(out, "            _ => None,").unwrap();
        writeln!(out, "        }}").unwrap();
        writeln!(out, "    }}").unwrap();

        writeln!(
        out,
        "    fn count<'db>(&self, query: {query_ident}<'db>) -> BoxFuture<'db, anyhow::Result<i64>> where Self: 'db {{"
    )
    .unwrap();
        writeln!(
            out,
            "        Box::pin(async move {{ query.count().await }})"
        )
        .unwrap();
        writeln!(out, "    }}").unwrap();

        writeln!(
        out,
        "    fn fetch_page<'db>(&self, query: {query_ident}<'db>, page: i64, per_page: i64) -> BoxFuture<'db, anyhow::Result<Vec<{wr_ident}>>> where Self: 'db {{"
    )
    .unwrap();
        writeln!(
            out,
            "        Box::pin(async move {{ Ok(query.paginate(page, per_page).await?.data) }})"
        )
        .unwrap();
        writeln!(out, "    }}").unwrap();
        writeln!(out, "}}").unwrap();

        let data_table_config_ident = format!("{}DataTableConfig", model_title);
        let data_table_hooks_ident = format!("{}DataTableHooks", model_title);
        let default_data_table_hooks_ident = format!("{}DefaultDataTableHooks", model_title);
        let data_table_ident = format!("{}DataTable", model_title);

        writeln!(out, "#[derive(Debug, Clone, Copy)]").unwrap();
        writeln!(out, "pub struct {data_table_config_ident} {{").unwrap();
        writeln!(out, "    pub default_sorting_column: &'static str,").unwrap();
        writeln!(out, "    pub default_sorted: SortDirection,").unwrap();
        writeln!(
            out,
            "    pub default_export_ignore_columns: &'static [&'static str],"
        )
        .unwrap();
        writeln!(
            out,
            "    pub default_timestamp_columns: &'static [&'static str],"
        )
        .unwrap();
        writeln!(out, "    pub default_unsortable: &'static [&'static str],").unwrap();
        writeln!(out, "    pub default_row_per_page: Option<i64>,").unwrap();
        writeln!(out, "}}").unwrap();

        writeln!(out, "impl Default for {data_table_config_ident} {{").unwrap();
        writeln!(out, "    fn default() -> Self {{").unwrap();
        writeln!(out, "        Self {{").unwrap();
        writeln!(out, "            default_sorting_column: \"{pk}\",").unwrap();
        writeln!(out, "            default_sorted: SortDirection::Desc,").unwrap();
        writeln!(
            out,
            "            default_export_ignore_columns: &[\"actions\", \"action\"],"
        )
        .unwrap();
        writeln!(
            out,
            "            default_timestamp_columns: &[{timestamp_cols_lit}],"
        )
        .unwrap();
        writeln!(out, "            default_unsortable: &[],").unwrap();
        writeln!(out, "            default_row_per_page: None,").unwrap();
        writeln!(out, "        }}").unwrap();
        writeln!(out, "    }}").unwrap();
        writeln!(out, "}}").unwrap();

        writeln!(
            out,
            "pub trait {data_table_hooks_ident}: Send + Sync + 'static {{"
        )
        .unwrap();
        writeln!(
        out,
        "    fn scope<'db>(&'db self, query: {query_ident}<'db>, _input: &DataTableInput, _ctx: &DataTableContext) -> {query_ident}<'db> {{ query }}"
    )
    .unwrap();
        writeln!(
        out,
        "    fn authorize(&self, _input: &DataTableInput, _ctx: &DataTableContext) -> anyhow::Result<bool> {{ Ok(true) }}"
    )
    .unwrap();
        writeln!(
        out,
        "    fn filter_query<'db>(&'db self, _query: {query_ident}<'db>, _filter_key: &str, _value: &str, _input: &DataTableInput, _ctx: &DataTableContext) -> anyhow::Result<Option<{query_ident}<'db>>> {{ Ok(None) }}"
    )
    .unwrap();
        writeln!(
        out,
        "    fn filters<'db>(&'db self, query: {query_ident}<'db>, _input: &DataTableInput, _ctx: &DataTableContext) -> anyhow::Result<{query_ident}<'db>> {{ Ok(query) }}"
    )
    .unwrap();
        writeln!(
        out,
        "    fn map_row(&self, _row: &mut {wr_ident}, _input: &DataTableInput, _ctx: &DataTableContext) -> anyhow::Result<()> {{ Ok(()) }}"
    )
    .unwrap();
        writeln!(
        out,
        "    fn default_row_to_record(&self, row: {wr_ident}) -> anyhow::Result<serde_json::Map<String, serde_json::Value>> {{"
    )
    .unwrap();
        writeln!(out, "        let value = serde_json::to_value(row)?;").unwrap();
        writeln!(
        out,
        "        let mut record = match value {{ serde_json::Value::Object(map) => map, _ => anyhow::bail!(\"Generated row must serialize to a JSON object\"), }};"
    )
    .unwrap();
        if use_snowflake_id {
            writeln!(
                out,
                "        if let Some(id_value) = record.get(\"{pk}\").cloned() {{"
            )
            .unwrap();
            writeln!(out, "            let id_text = match id_value {{").unwrap();
            writeln!(
                out,
                "                serde_json::Value::Number(number) => number.to_string(),"
            )
            .unwrap();
            writeln!(
                out,
                "                serde_json::Value::String(text) => text,"
            )
            .unwrap();
            writeln!(out, "                other => other.to_string(),").unwrap();
            writeln!(out, "            }};").unwrap();
            writeln!(
            out,
            "            record.insert(\"{pk}\".to_string(), serde_json::Value::String(id_text));"
        )
            .unwrap();
            writeln!(out, "        }}").unwrap();
        }
        writeln!(out, "        Ok(record)").unwrap();
        writeln!(out, "    }}").unwrap();
        writeln!(
        out,
        "    fn row_to_record(&self, row: {wr_ident}, _input: &DataTableInput, _ctx: &DataTableContext) -> anyhow::Result<serde_json::Map<String, serde_json::Value>> {{"
    )
    .unwrap();
        writeln!(out, "        self.default_row_to_record(row)").unwrap();
        writeln!(out, "    }}").unwrap();
        writeln!(
        out,
        "    fn summary<'db>(&'db self, _query: {query_ident}<'db>, _input: &DataTableInput, _ctx: &DataTableContext) -> BoxFuture<'db, anyhow::Result<Option<serde_json::Value>>> {{ Box::pin(async {{ Ok(None) }}) }}"
    )
    .unwrap();
        writeln!(out, "}}").unwrap();

        writeln!(out, "#[derive(Default)]").unwrap();
        writeln!(out, "pub struct {default_data_table_hooks_ident};").unwrap();
        writeln!(
            out,
            "impl {data_table_hooks_ident} for {default_data_table_hooks_ident} {{}}"
        )
        .unwrap();

        writeln!(
        out,
        "pub struct {data_table_ident}<H = {default_data_table_hooks_ident}> where H: {data_table_hooks_ident} {{"
    )
    .unwrap();
        writeln!(out, "    pub db: sqlx::PgPool,").unwrap();
        writeln!(out, "    pub hooks: H,").unwrap();
        writeln!(out, "    pub config: {data_table_config_ident},").unwrap();
        writeln!(out, "    adapter: {table_adapter_ident},").unwrap();
        writeln!(out, "}}").unwrap();

        writeln!(
            out,
            "impl {data_table_ident}<{default_data_table_hooks_ident}> {{"
        )
        .unwrap();
        writeln!(out, "    pub fn new(db: sqlx::PgPool) -> Self {{").unwrap();
        writeln!(out, "        Self {{").unwrap();
        writeln!(out, "            db,").unwrap();
        writeln!(out, "            hooks: {default_data_table_hooks_ident},").unwrap();
        writeln!(
            out,
            "            config: {data_table_config_ident}::default(),"
        )
        .unwrap();
        writeln!(out, "            adapter: {table_adapter_ident},").unwrap();
        writeln!(out, "        }}").unwrap();
        writeln!(out, "    }}").unwrap();
        writeln!(out, "}}").unwrap();

        writeln!(
            out,
            "impl<H: {data_table_hooks_ident}> {data_table_ident}<H> {{"
        )
        .unwrap();
        writeln!(
        out,
        "    pub fn with_hooks<NH: {data_table_hooks_ident}>(self, hooks: NH) -> {data_table_ident}<NH> {{"
    )
    .unwrap();
        writeln!(out, "        {data_table_ident} {{").unwrap();
        writeln!(out, "            db: self.db,").unwrap();
        writeln!(out, "            hooks,").unwrap();
        writeln!(out, "            config: self.config,").unwrap();
        writeln!(out, "            adapter: {table_adapter_ident},").unwrap();
        writeln!(out, "        }}").unwrap();
        writeln!(out, "    }}").unwrap();
        writeln!(
            out,
            "    pub fn with_config(mut self, config: {data_table_config_ident}) -> Self {{"
        )
        .unwrap();
        writeln!(out, "        self.config = config;").unwrap();
        writeln!(out, "        self").unwrap();
        writeln!(out, "    }}").unwrap();
        writeln!(out, "}}").unwrap();

        writeln!(
            out,
            "impl<H: {data_table_hooks_ident}> AutoDataTable for {data_table_ident}<H> {{"
        )
        .unwrap();
        writeln!(out, "    type Adapter = {table_adapter_ident};").unwrap();
        writeln!(
            out,
            "    fn adapter(&self) -> &Self::Adapter {{ &self.adapter }}"
        )
        .unwrap();
        writeln!(
        out,
        "    fn base_query<'db>(&'db self, input: &DataTableInput, ctx: &DataTableContext) -> {query_ident}<'db> {{"
    )
    .unwrap();
        writeln!(
            out,
            "        self.hooks.scope({model_ident}::new(&self.db, None).query(), input, ctx)"
        )
        .unwrap();
        writeln!(out, "    }}").unwrap();
        writeln!(
        out,
        "    fn authorize(&self, input: &DataTableInput, ctx: &DataTableContext) -> anyhow::Result<bool> {{ self.hooks.authorize(input, ctx) }}"
    )
    .unwrap();
        writeln!(
        out,
        "    fn filter_query<'db>(&'db self, query: {query_ident}<'db>, filter_key: &str, value: &str, input: &DataTableInput, ctx: &DataTableContext) -> anyhow::Result<Option<{query_ident}<'db>>> {{ self.hooks.filter_query(query, filter_key, value, input, ctx) }}"
    )
    .unwrap();
        writeln!(
        out,
        "    fn filters<'db>(&'db self, query: {query_ident}<'db>, input: &DataTableInput, ctx: &DataTableContext) -> anyhow::Result<{query_ident}<'db>> {{ self.hooks.filters(query, input, ctx) }}"
    )
    .unwrap();
        writeln!(
        out,
        "    fn map_row(&self, row: &mut {wr_ident}, input: &DataTableInput, ctx: &DataTableContext) -> anyhow::Result<()> {{ self.hooks.map_row(row, input, ctx) }}"
    )
    .unwrap();
        writeln!(
        out,
        "    fn row_to_record(&self, row: {wr_ident}, input: &DataTableInput, ctx: &DataTableContext) -> anyhow::Result<serde_json::Map<String, serde_json::Value>> {{ self.hooks.row_to_record(row, input, ctx) }}"
    )
    .unwrap();
        writeln!(
        out,
        "    fn summary<'db>(&'db self, query: {query_ident}<'db>, input: &DataTableInput, ctx: &DataTableContext) -> BoxFuture<'db, anyhow::Result<Option<serde_json::Value>>> where Self: 'db {{ self.hooks.summary(query, input, ctx) }}"
    )
    .unwrap();
        writeln!(
        out,
        "    fn default_sorting_column(&self) -> &'static str {{ self.config.default_sorting_column }}"
    )
    .unwrap();
        writeln!(
            out,
            "    fn default_sorted(&self) -> SortDirection {{ self.config.default_sorted }}"
        )
        .unwrap();
        writeln!(
        out,
        "    fn default_export_ignore_columns(&self) -> &'static [&'static str] {{ self.config.default_export_ignore_columns }}"
    )
    .unwrap();
        writeln!(
        out,
        "    fn default_timestamp_columns(&self) -> &'static [&'static str] {{ self.config.default_timestamp_columns }}"
    )
    .unwrap();
        writeln!(
        out,
        "    fn default_unsortable(&self) -> &'static [&'static str] {{ self.config.default_unsortable }}"
    )
    .unwrap();
        writeln!(
        out,
        "    fn default_row_per_page(&self, ctx: &DataTableContext) -> i64 {{ self.config.default_row_per_page.unwrap_or(ctx.default_per_page) }}"
    )
        .unwrap();
        writeln!(out, "}}").unwrap();
    }

    let datatable_section = out;
    let mut out = String::new();

    // Implement ActiveRecord for View
    writeln!(out).unwrap();
    writeln!(out, "use core_db::common::active_record::ActiveRecord;").unwrap();
    writeln!(out, "#[async_trait::async_trait]").unwrap();
    writeln!(out, "impl ActiveRecord for {view_ident} {{").unwrap();
    writeln!(out, "    type Id = {parent_pk_ty};").unwrap();
    writeln!(
        out,
        "    async fn find(db: &sqlx::PgPool, id: Self::Id) -> anyhow::Result<Option<Self>> {{"
    )
    .unwrap();
    writeln!(
        out,
        "        {model_ident}::new(db, None).find(id).await.map(|opt| opt.map(|r| r.into_row())).map_err(|e| e.into())"
    )
    .unwrap();
    writeln!(out, "    }}").unwrap();
    writeln!(out, "}}").unwrap();

    let active_record_section = out;

    let mut context = TemplateContext::new();
    context
        .insert("imports", imports.trim_end().to_string())
        .unwrap();
    context
        .insert("constants", constants.trim_end().to_string())
        .unwrap();
    context
        .insert(
            "row_view_json_section",
            row_view_json_section.trim_start().to_string(),
        )
        .unwrap();
    context
        .insert(
            "column_model_section",
            column_model_section.trim_start().to_string(),
        )
        .unwrap();
    context
        .insert("query_section", query_section.trim_start().to_string())
        .unwrap();
    context
        .insert("insert_section", insert_section.trim_start().to_string())
        .unwrap();
    context
        .insert("update_section", update_section.trim_start().to_string())
        .unwrap();
    context
        .insert(
            "datatable_section",
            datatable_section.trim_start().to_string(),
        )
        .unwrap();
    context
        .insert(
            "active_record_section",
            active_record_section.trim_start().to_string(),
        )
        .unwrap();
    render_template("models/model.rs.tpl", &context).unwrap()
}

fn generate_common() -> String {
    render_template("models/common.rs.tpl", &TemplateContext::new()).unwrap()
}
