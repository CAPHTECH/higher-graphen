use std::collections::BTreeSet;
use syn::visit::Visit;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RustTestSemanticDocument {
    pub(crate) functions: Vec<RustTestSemanticFunction>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RustTestSemanticFunction {
    pub(crate) name: String,
    pub(crate) assertion_macros: Vec<String>,
    pub(crate) string_literals: BTreeSet<String>,
    pub(crate) cli_observations: Vec<RustTestCliObservation>,
    pub(crate) json_observations: Vec<RustTestJsonObservation>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RustTestCliObservation {
    pub(crate) label: String,
    pub(crate) tokens: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RustTestJsonObservation {
    pub(crate) label: String,
    pub(crate) observation_type: RustTestJsonObservationType,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RustTestJsonObservationType {
    Field,
    SchemaId,
}

pub(crate) fn extract_rust_test_semantics(contents: &str) -> Option<RustTestSemanticDocument> {
    let file = syn::parse_file(contents).ok()?;
    let mut collector = RustTestFunctionCollector::default();
    collector.visit_file(&file);
    Some(RustTestSemanticDocument {
        functions: collector.functions,
    })
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct RustTestFunctionCollector {
    functions: Vec<RustTestSemanticFunction>,
}

impl Visit<'_> for RustTestFunctionCollector {
    fn visit_item_fn(&mut self, item_fn: &syn::ItemFn) {
        if has_rust_test_attribute(&item_fn.attrs) {
            let mut visitor = RustTestSemanticVisitor::default();
            visitor.visit_block(&item_fn.block);
            self.functions.push(RustTestSemanticFunction {
                name: item_fn.sig.ident.to_string(),
                assertion_macros: visitor.assertion_macros.clone(),
                cli_observations: cli_observations(&visitor.cli_token_sequences),
                json_observations: json_observations(&visitor),
                string_literals: visitor.string_literals,
            });
        } else {
            syn::visit::visit_item_fn(self, item_fn);
        }
    }
}

fn has_rust_test_attribute(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attr| {
        let segments = attr
            .path()
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect::<Vec<_>>();
        segments == ["test"] || segments == ["tokio", "test"] || segments == ["async_std", "test"]
    })
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct RustTestSemanticVisitor {
    assertion_macros: Vec<String>,
    string_literals: BTreeSet<String>,
    cli_token_sequences: Vec<Vec<String>>,
    json_field_labels: BTreeSet<String>,
}

impl Visit<'_> for RustTestSemanticVisitor {
    fn visit_expr_call(&mut self, node: &syn::ExprCall) {
        self.record_call_cli_tokens(node);
        syn::visit::visit_expr_call(self, node);
    }

    fn visit_expr_index(&mut self, node: &syn::ExprIndex) {
        if let syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(lit_str),
            ..
        }) = node.index.as_ref()
        {
            self.json_field_labels.insert(lit_str.value());
        }
        syn::visit::visit_expr_index(self, node);
    }

    fn visit_expr_lit(&mut self, node: &syn::ExprLit) {
        if let syn::Lit::Str(lit_str) = &node.lit {
            self.string_literals.insert(lit_str.value());
        }
        syn::visit::visit_expr_lit(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &syn::ExprMethodCall) {
        self.record_method_cli_tokens(node);
        syn::visit::visit_expr_method_call(self, node);
    }

    fn visit_macro(&mut self, node: &syn::Macro) {
        self.record_macro(node);
        syn::visit::visit_macro(self, node);
    }
}

impl RustTestSemanticVisitor {
    fn record_macro(&mut self, node: &syn::Macro) {
        let macro_name = node
            .path
            .segments
            .last()
            .map(|segment| segment.ident.to_string())
            .unwrap_or_default();
        if matches!(
            macro_name.as_str(),
            "assert" | "assert_eq" | "assert_ne" | "assert_matches" | "matches"
        ) {
            self.assertion_macros.push(macro_name);
        }
        let token_text = node.tokens.to_string();
        for field in bracketed_string_indices_from_token_text(&token_text) {
            self.json_field_labels.insert(field);
        }
        for literal in quoted_strings_from_token_text(&token_text) {
            self.string_literals.insert(literal);
        }
    }

    fn record_call_cli_tokens(&mut self, node: &syn::ExprCall) {
        if !callee_indicates_cli(&node.func) {
            return;
        }
        for arg in &node.args {
            if let Some(tokens) = string_array_tokens_from_expr(arg) {
                self.cli_token_sequences.push(tokens);
            }
        }
    }

    fn record_method_cli_tokens(&mut self, node: &syn::ExprMethodCall) {
        if node.method != "args" {
            return;
        }
        for arg in &node.args {
            if let Some(tokens) = string_array_tokens_from_expr(arg) {
                self.cli_token_sequences.push(tokens);
            }
        }
    }
}

fn cli_observations(token_sequences: &[Vec<String>]) -> Vec<RustTestCliObservation> {
    let mut seen = BTreeSet::new();
    let mut observations = Vec::new();
    for tokens in token_sequences {
        if tokens.len() < 2 || tokens.iter().all(|token| token.starts_with('-')) {
            continue;
        }
        let label = tokens.join(" ");
        if seen.insert(label.clone()) {
            observations.push(RustTestCliObservation {
                label,
                tokens: tokens.clone(),
            });
        }
    }
    observations
}

fn json_observations(visitor: &RustTestSemanticVisitor) -> Vec<RustTestJsonObservation> {
    let mut observations = Vec::new();
    for field in &visitor.json_field_labels {
        observations.push(RustTestJsonObservation {
            label: format!("field:{field}"),
            observation_type: RustTestJsonObservationType::Field,
        });
    }
    for value in &visitor.string_literals {
        if looks_like_schema_id(value) {
            observations.push(RustTestJsonObservation {
                label: format!("schema:{value}"),
                observation_type: RustTestJsonObservationType::SchemaId,
            });
        }
    }
    observations
}

pub(crate) fn contains_all_strings(strings: &BTreeSet<String>, expected: &[&str]) -> bool {
    expected.iter().all(|value| strings.contains(*value))
        || expected
            .iter()
            .all(|value| strings.iter().any(|string| string.contains(value)))
}

fn callee_indicates_cli(function: &syn::Expr) -> bool {
    let syn::Expr::Path(path) = function else {
        return false;
    };
    path.path.segments.last().is_some_and(|segment| {
        let name = segment.ident.to_string().to_ascii_lowercase();
        name.contains("cli") || name.contains("command") || name.contains("cmd")
    })
}

fn string_array_tokens_from_expr(expr: &syn::Expr) -> Option<Vec<String>> {
    match expr {
        syn::Expr::Reference(reference) => string_array_tokens_from_expr(&reference.expr),
        syn::Expr::Array(array) => string_array_tokens_from_array(array),
        _ => None,
    }
}

fn string_array_tokens_from_array(array: &syn::ExprArray) -> Option<Vec<String>> {
    let mut tokens = Vec::new();
    for element in &array.elems {
        let syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(lit_str),
            ..
        }) = element
        else {
            return None;
        };
        tokens.push(lit_str.value());
    }
    Some(tokens)
}

fn looks_like_schema_id(value: &str) -> bool {
    let Some((prefix, version)) = value.rsplit_once(".v") else {
        return false;
    };
    !prefix.is_empty()
        && prefix.contains('.')
        && version.chars().all(|character| character.is_ascii_digit())
        && value.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-' | ':' | '/')
        })
}

fn quoted_strings_from_token_text(text: &str) -> Vec<String> {
    let mut strings = Vec::new();
    let mut chars = text.chars().peekable();
    while let Some(character) = chars.next() {
        if character != '"' {
            continue;
        }
        let mut value = String::new();
        let mut escaped = false;
        for next in chars.by_ref() {
            if escaped {
                value.push(next);
                escaped = false;
                continue;
            }
            if next == '\\' {
                escaped = true;
                continue;
            }
            if next == '"' {
                strings.push(value);
                break;
            }
            value.push(next);
        }
    }
    strings
}

fn bracketed_string_indices_from_token_text(text: &str) -> Vec<String> {
    let mut strings = Vec::new();
    let mut chars = text.chars().peekable();
    while let Some(character) = chars.next() {
        if character != '[' {
            continue;
        }
        while chars.peek().is_some_and(|next| next.is_whitespace()) {
            chars.next();
        }
        if !chars.peek().is_some_and(|next| *next == '"') {
            continue;
        }
        chars.next();
        let mut value = String::new();
        let mut escaped = false;
        for next in chars.by_ref() {
            if escaped {
                value.push(next);
                escaped = false;
                continue;
            }
            if next == '\\' {
                escaped = true;
                continue;
            }
            if next == '"' {
                while chars.peek().is_some_and(|after| after.is_whitespace()) {
                    chars.next();
                }
                if chars.peek().is_some_and(|after| *after == ']') {
                    strings.push(value);
                }
                break;
            }
            value.push(next);
        }
    }
    strings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_rust_test_semantics_without_project_binding() {
        let document = extract_rust_test_semantics(
            r##"
#[cfg(test)]
mod tests {
use super::*;

#[test]
fn emits_json() {
    let output = run_cli(&["acme", "audit", "--format", "json"]);
    assert!(output.status.success());
    assert_eq!(value["schema"], json!("acme.audit.input.v2"));
}
}
"##,
        )
        .expect("parse test document");

        assert_eq!(document.functions.len(), 1);
        let function = &document.functions[0];
        assert_eq!(function.name, "emits_json");
        assert_eq!(function.assertion_macros, vec!["assert", "assert_eq"]);
        assert!(function
            .cli_observations
            .iter()
            .any(|observation| observation.tokens
                == vec![
                    "acme".to_owned(),
                    "audit".to_owned(),
                    "--format".to_owned(),
                    "json".to_owned()
                ]));
        assert!(function.json_observations.iter().any(|observation| {
            observation.label == "field:schema"
                && observation.observation_type == RustTestJsonObservationType::Field
        }));
        assert!(function.json_observations.iter().any(|observation| {
            observation.label == "schema:acme.audit.input.v2"
                && observation.observation_type == RustTestJsonObservationType::SchemaId
        }));
    }
}
