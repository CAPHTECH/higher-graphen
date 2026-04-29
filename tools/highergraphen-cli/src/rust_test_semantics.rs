use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::{
    fs,
    path::{Path, PathBuf},
};
use syn::visit::Visit;

pub(crate) const RUST_TEST_SEMANTICS_SCHEMA: &str = "highergraphen.rust_test_semantics.input.v1";
const RUST_TEST_SEMANTICS_ADAPTER: &str = "rust-test-semantics-from-path.v1";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RustTestSemanticsPathRequest {
    pub(crate) repo: PathBuf,
    pub(crate) paths: Vec<PathBuf>,
    pub(crate) test_run: Option<PathBuf>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RustTestSemanticPathDocument {
    pub(crate) selected_paths: Vec<String>,
    pub(crate) files: Vec<RustTestSemanticFile>,
    pub(crate) execution_cases: Vec<RustTestExecutionCase>,
}

impl RustTestSemanticPathDocument {
    pub(crate) fn to_json_value(&self) -> Value {
        json!({
            "schema": RUST_TEST_SEMANTICS_SCHEMA,
            "source": {
                "kind": "code",
                "adapter": RUST_TEST_SEMANTICS_ADAPTER,
                "boundary": "selected_paths"
            },
            "selected_paths": self.selected_paths,
            "files": self.files.iter().map(RustTestSemanticFile::to_json_value).collect::<Vec<_>>(),
            "execution_cases": self.execution_cases.iter().map(RustTestExecutionCase::to_json_value).collect::<Vec<_>>(),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RustTestSemanticFile {
    pub(crate) path: String,
    pub(crate) functions: Vec<RustTestSemanticFunction>,
}

impl RustTestSemanticFile {
    fn to_json_value(&self) -> Value {
        json!({
            "path": self.path,
            "functions": self.functions.iter().map(RustTestSemanticFunction::to_json_value).collect::<Vec<_>>(),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RustTestExecutionCase {
    pub(crate) name: String,
    pub(crate) status: String,
    pub(crate) matched_functions: Vec<String>,
}

impl RustTestExecutionCase {
    fn to_json_value(&self) -> Value {
        json!({
            "name": self.name,
            "status": self.status,
            "matched_functions": self.matched_functions,
        })
    }
}

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

impl RustTestSemanticFunction {
    fn to_json_value(&self) -> Value {
        json!({
            "name": self.name,
            "assertion_macros": self.assertion_macros,
            "string_literals": self.string_literals.iter().collect::<Vec<_>>(),
            "cli_observations": self.cli_observations.iter().map(RustTestCliObservation::to_json_value).collect::<Vec<_>>(),
            "json_observations": self.json_observations.iter().map(RustTestJsonObservation::to_json_value).collect::<Vec<_>>(),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RustTestCliObservation {
    pub(crate) label: String,
    pub(crate) tokens: Vec<String>,
}

impl RustTestCliObservation {
    fn to_json_value(&self) -> Value {
        json!({
            "label": self.label,
            "tokens": self.tokens,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RustTestJsonObservation {
    pub(crate) label: String,
    pub(crate) observation_type: RustTestJsonObservationType,
}

impl RustTestJsonObservation {
    fn to_json_value(&self) -> Value {
        json!({
            "label": self.label,
            "observation_type": self.observation_type.as_str(),
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RustTestJsonObservationType {
    Field,
    SchemaId,
}

impl RustTestJsonObservationType {
    fn as_str(self) -> &'static str {
        match self {
            Self::Field => "field",
            Self::SchemaId => "schema_id",
        }
    }
}

pub(crate) fn document_from_path(
    request: RustTestSemanticsPathRequest,
) -> Result<RustTestSemanticPathDocument, String> {
    if request.paths.is_empty() {
        return Err("--path <path> is required".to_owned());
    }
    let repo = fs::canonicalize(&request.repo).map_err(|error| {
        format!(
            "failed to resolve repository {}: {error}",
            request.repo.display()
        )
    })?;
    let mut relative_paths = BTreeSet::new();
    let mut selected_paths = Vec::new();
    for path in &request.paths {
        let (relative_path, absolute_path) = resolve_selected_path(&repo, path)?;
        selected_paths.push(path_to_string(&relative_path));
        collect_rust_paths(&repo, &absolute_path, &mut relative_paths)?;
    }

    let mut files = Vec::new();
    for relative_path in relative_paths {
        let absolute_path = repo.join(&relative_path);
        let contents = fs::read_to_string(&absolute_path)
            .map_err(|error| format!("failed to read {}: {error}", relative_path.display()))?;
        let Some(document) = extract_rust_test_semantics(&contents) else {
            continue;
        };
        if document.functions.is_empty() {
            continue;
        }
        files.push(RustTestSemanticFile {
            path: path_to_string(&relative_path),
            functions: document.functions,
        });
    }

    let execution_cases = match &request.test_run {
        Some(test_run) => execution_cases_from_test_run(test_run, &files)?,
        None => Vec::new(),
    };

    Ok(RustTestSemanticPathDocument {
        selected_paths,
        files,
        execution_cases,
    })
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
        segments == ["test"]
            || segments == ["tokio", "test"]
            || segments == ["async_std", "test"]
            || segments == ["rstest"]
            || segments == ["test_case"]
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
            "assert"
                | "assert_eq"
                | "assert_ne"
                | "assert_matches"
                | "assert_json_eq"
                | "assert_json_include"
                | "assert_snapshot"
                | "assert_debug_snapshot"
                | "assert_display_snapshot"
                | "assert_json_snapshot"
                | "assert_yaml_snapshot"
                | "matches"
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

fn resolve_selected_path(repo: &Path, path: &Path) -> Result<(PathBuf, PathBuf), String> {
    let candidate = if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo.join(path)
    };
    let absolute_path = fs::canonicalize(&candidate)
        .map_err(|error| format!("failed to resolve path {}: {error}", candidate.display()))?;
    let relative_path = relative_path(repo, &absolute_path)?;
    let relative_text = path_to_string(&relative_path);
    if relative_text.is_empty() || path_has_disallowed_component(&relative_path) {
        return Err(format!(
            "unsupported path {} relative to {}",
            relative_text,
            repo.display()
        ));
    }
    Ok((relative_path, absolute_path))
}

fn collect_rust_paths(
    repo: &Path,
    absolute_path: &Path,
    output: &mut BTreeSet<PathBuf>,
) -> Result<(), String> {
    let metadata = fs::metadata(absolute_path)
        .map_err(|error| format!("failed to read {}: {error}", absolute_path.display()))?;
    if metadata.is_file() {
        if is_rust_path(absolute_path) {
            output.insert(relative_path(repo, absolute_path)?);
        }
        return Ok(());
    }
    if !metadata.is_dir() {
        return Ok(());
    }

    let mut children = fs::read_dir(absolute_path)
        .map_err(|error| {
            format!(
                "failed to read directory {}: {error}",
                absolute_path.display()
            )
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| {
            format!(
                "failed to read directory {}: {error}",
                absolute_path.display()
            )
        })?;
    children.sort_by_key(|entry| entry.path());
    for child in children {
        let path = child.path();
        if should_skip_directory(&path) {
            continue;
        }
        collect_rust_paths(repo, &path, output)?;
    }
    Ok(())
}

fn relative_path(repo: &Path, absolute_path: &Path) -> Result<PathBuf, String> {
    absolute_path
        .strip_prefix(repo)
        .map(Path::to_path_buf)
        .map_err(|_| format!("{} is outside {}", absolute_path.display(), repo.display()))
}

fn path_has_disallowed_component(path: &Path) -> bool {
    path.components().any(|component| {
        let value = component.as_os_str().to_string_lossy();
        matches!(value.as_ref(), "." | ".." | ".git" | "target")
    })
}

fn should_skip_directory(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| matches!(name, ".git" | "target"))
}

fn is_rust_path(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension == "rs")
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn execution_cases_from_test_run(
    test_run: &Path,
    files: &[RustTestSemanticFile],
) -> Result<Vec<RustTestExecutionCase>, String> {
    let text = fs::read_to_string(test_run)
        .map_err(|error| format!("failed to read test run {}: {error}", test_run.display()))?;
    Ok(parse_test_run_cases(&text)
        .into_iter()
        .map(|case| RustTestExecutionCase {
            matched_functions: matching_function_names(files, &case.name),
            name: case.name,
            status: case.status,
        })
        .collect())
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ParsedTestRunCase {
    name: String,
    status: String,
}

fn parse_test_run_cases(text: &str) -> Vec<ParsedTestRunCase> {
    if let Ok(value) = serde_json::from_str::<Value>(text) {
        let cases = parse_json_test_cases(&value);
        if !cases.is_empty() {
            return cases;
        }
    }
    let jsonl_cases = text
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .flat_map(|value| parse_json_test_cases(&value))
        .collect::<Vec<_>>();
    if !jsonl_cases.is_empty() {
        return jsonl_cases;
    }
    parse_plain_test_cases(text)
}

fn parse_json_test_cases(value: &Value) -> Vec<ParsedTestRunCase> {
    if let Some(cases) = value.get("tests").and_then(Value::as_array) {
        return cases.iter().filter_map(test_case_from_json).collect();
    }
    if let Some(cases) = value.as_array() {
        return cases.iter().filter_map(test_case_from_json).collect();
    }
    test_case_from_json(value).into_iter().collect()
}

fn test_case_from_json(value: &Value) -> Option<ParsedTestRunCase> {
    let name = value
        .get("name")
        .or_else(|| value.get("test"))
        .and_then(Value::as_str)?
        .trim();
    if name.is_empty() {
        return None;
    }
    let status = value
        .get("status")
        .or_else(|| value.get("event"))
        .or_else(|| value.get("result"))
        .and_then(Value::as_str)
        .and_then(normalize_status)?;
    Some(ParsedTestRunCase {
        name: name.to_owned(),
        status: status.to_owned(),
    })
}

fn parse_plain_test_cases(text: &str) -> Vec<ParsedTestRunCase> {
    text.lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            let body = trimmed.strip_prefix("test ")?;
            let (name, status_text) = body.rsplit_once(" ... ")?;
            Some(ParsedTestRunCase {
                name: name.trim().to_owned(),
                status: normalize_status(status_text.trim())?.to_owned(),
            })
        })
        .collect()
}

fn normalize_status(value: &str) -> Option<&'static str> {
    match value.trim().to_ascii_lowercase().as_str() {
        "ok" | "passed" | "pass" | "success" | "succeeded" => Some("passed"),
        "failed" | "fail" | "failure" | "error" => Some("failed"),
        "ignored" | "ignore" | "skipped" | "skip" => Some("ignored"),
        _ => None,
    }
}

fn matching_function_names(files: &[RustTestSemanticFile], test_name: &str) -> Vec<String> {
    let candidates = test_name_candidates(test_name);
    files
        .iter()
        .flat_map(|file| {
            file.functions.iter().filter_map(|function| {
                if candidates.contains(&slug(&function.name)) {
                    Some(format!("{}::{}", file.path, function.name))
                } else {
                    None
                }
            })
        })
        .collect()
}

fn test_name_candidates(test_name: &str) -> BTreeSet<String> {
    let mut candidates = BTreeSet::new();
    candidates.insert(slug(test_name));
    for segment in test_name.split("::") {
        candidates.insert(slug(segment));
    }
    if let Some(last) = test_name.split("::").last() {
        candidates.insert(slug(last));
    }
    candidates
}

fn slug(value: &str) -> String {
    let mut slug = String::new();
    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character.to_ascii_lowercase());
        } else if !slug.ends_with('-') {
            slug.push('-');
        }
    }
    slug.trim_matches('-').to_owned()
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
    if value.starts_with("schema:") || value.starts_with("field:") {
        return false;
    }
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

#[rstest]
fn snapshot_contract() {
    assert_json_snapshot!("contract", json!({"schema": "acme.snapshot.v1"}));
}
}
"##,
        )
        .expect("parse test document");

        assert_eq!(document.functions.len(), 2);
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
        let snapshot_function = &document.functions[1];
        assert_eq!(snapshot_function.name, "snapshot_contract");
        assert_eq!(
            snapshot_function.assertion_macros,
            vec!["assert_json_snapshot"]
        );
        assert!(snapshot_function
            .json_observations
            .iter()
            .any(|observation| {
                observation.label == "schema:acme.snapshot.v1"
                    && observation.observation_type == RustTestJsonObservationType::SchemaId
            }));
    }
}
