use serde_json::{json, Value};

pub(super) fn required_actions() -> Vec<&'static str> {
    vec![
        "Review context-specific domain meanings before accepting a shared model.",
        "Add accepted equivalence evidence or split the model by bounded context.",
        "Review or implement anti-corruption mapping candidates before close.",
    ]
}

pub(super) fn invariant_ids() -> Vec<&'static str> {
    vec![
        "invariant:context-language-preserved",
        "invariant:cross-context-identity-not-collapsed",
        "invariant:boundary-translation-explicit",
        "invariant:review-gates-satisfied-before-close",
        "invariant:inference-not-accepted-evidence",
        "invariant:projection-declares-loss",
        "invariant:context-ownership-explicit",
    ]
}

pub(super) fn source_ids_from_input(input: &Value) -> Vec<String> {
    ids_from_values(
        array_at(input, &["accepted_facts"])
            .iter()
            .chain(&array_at(input, &["constraints"]))
            .chain(&array_at(input, &["reviews"]))
            .chain(&array_at(input, &["inferred_claims"]))
            .chain(&array_at(input, &["completion_hints"]))
            .collect(),
    )
}

pub(super) fn context_ids_from_records(records: &[Value]) -> Vec<String> {
    records
        .iter()
        .filter(|record| string_at(record, &["record_type"]).as_deref() == Some("bounded_context"))
        .filter_map(|record| string_at(record, &["id"]))
        .collect()
}

pub(super) fn compact_records(records: &[Value], kind_field: &str) -> Vec<Value> {
    records
        .iter()
        .filter_map(|record| {
            Some(json!({
                "id": string_at(record, &["id"])?,
                kind_field: string_at(record, &[kind_field])?,
                "review_status": string_at(record, &["review_status"]).unwrap_or_else(|| "unreviewed".to_owned())
            }))
        })
        .collect()
}

pub(super) fn cells(case_space: &Value) -> Vec<&Value> {
    case_space
        .get("case_cells")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .collect()
}

pub(super) fn ids_from_records(records: &[Value]) -> Vec<String> {
    records
        .iter()
        .filter_map(|record| string_at(record, &["id"]))
        .collect()
}

pub(super) fn ids_from_values(records: Vec<&Value>) -> Vec<String> {
    records
        .into_iter()
        .filter_map(|record| string_at(record, &["id"]))
        .collect()
}

pub(super) fn id_tail(id: &str) -> String {
    id.rsplit(':').next().unwrap_or(id).replace('_', "-")
}

pub(super) fn domain_term(record: &Value) -> String {
    string_at(record, &["label"])
        .or_else(|| string_at(record, &["id"]))
        .unwrap_or_default()
        .split([':', '.', '-', '_', ' '])
        .next_back()
        .unwrap_or_default()
        .to_ascii_lowercase()
}

pub(super) fn array_at(value: &Value, path: &[&str]) -> Vec<Value> {
    value_at(value, path)
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

pub(super) fn relations(case_space: &Value) -> Vec<&Value> {
    case_space
        .get("case_relations")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .collect()
}

pub(super) fn relation_endpoint_ids(relation: &Value) -> Vec<String> {
    ["from_id", "to_id"]
        .iter()
        .filter_map(|field| string_at(relation, &[field]))
        .collect()
}

pub(super) fn string_array_at(value: &Value, path: &[&str]) -> Vec<String> {
    value_at(value, path)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(ToOwned::to_owned)
        .collect()
}

pub(super) fn string_at(value: &Value, path: &[&str]) -> Option<String> {
    value_at(value, path)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
}

pub(super) fn number_at(value: &Value, path: &[&str]) -> Option<f64> {
    value_at(value, path).and_then(Value::as_f64)
}

fn value_at<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut current = value;
    for segment in path {
        current = current.get(*segment)?;
    }
    Some(current)
}
