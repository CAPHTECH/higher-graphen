use serde_json::{json, Value};

pub(crate) const TEST_SEMANTICS_INTERPRETATION_SCHEMA: &str =
    "highergraphen.test_semantics.interpretation.v1";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct InterpretRequest {
    pub(crate) input: Value,
    pub(crate) interpreter: String,
}

pub(crate) fn interpret(request: InterpretRequest) -> Result<Value, String> {
    let schema = request
        .input
        .get("schema")
        .and_then(Value::as_str)
        .ok_or_else(|| "input document needs schema".to_owned())?;
    if schema != "highergraphen.rust_test_semantics.input.v1"
        && schema != "highergraphen.test_semantics.input.v1"
    {
        return Err(format!(
            "unsupported test semantics schema {schema}; expected highergraphen.rust_test_semantics.input.v1 or highergraphen.test_semantics.input.v1"
        ));
    }

    let mut interpreted_cells = Vec::new();
    let mut interpreted_morphisms = Vec::new();
    let mut candidate_laws = Vec::new();
    let mut binding_candidates = Vec::new();
    let mut evidence_links = Vec::new();

    if schema == "highergraphen.rust_test_semantics.input.v1" {
        interpret_rust_semantics(
            &request.input,
            &mut interpreted_cells,
            &mut interpreted_morphisms,
            &mut candidate_laws,
            &mut binding_candidates,
            &mut evidence_links,
        )?;
    } else {
        interpret_generic_semantics(
            &request.input,
            &mut interpreted_cells,
            &mut interpreted_morphisms,
            &mut candidate_laws,
            &mut binding_candidates,
            &mut evidence_links,
        )?;
    }

    Ok(json!({
        "schema": TEST_SEMANTICS_INTERPRETATION_SCHEMA,
        "source": {
            "kind": "ai_agent",
            "input_schema": schema,
            "interpreter": request.interpreter,
            "review_status": "unreviewed"
        },
        "interpreted_cells": interpreted_cells,
        "interpreted_morphisms": interpreted_morphisms,
        "candidate_laws": candidate_laws,
        "binding_candidates": binding_candidates,
        "evidence_links": evidence_links,
        "information_loss": [
            "Interpretation candidates are not accepted coverage.",
            "No LLM reasoning transcript is embedded in this bounded document.",
            "Function-name execution matching is heuristic and requires review.",
            "Semantic roles require explicit binding and verification before proof use."
        ]
    }))
}

fn interpret_rust_semantics(
    input: &Value,
    interpreted_cells: &mut Vec<Value>,
    interpreted_morphisms: &mut Vec<Value>,
    candidate_laws: &mut Vec<Value>,
    binding_candidates: &mut Vec<Value>,
    evidence_links: &mut Vec<Value>,
) -> Result<(), String> {
    let files = input
        .get("files")
        .and_then(Value::as_array)
        .ok_or_else(|| "rust semantics input needs files array".to_owned())?;
    for file in files {
        let path = required_str(file, "path", "rust semantics file")?;
        let functions = file
            .get("functions")
            .and_then(Value::as_array)
            .ok_or_else(|| format!("rust semantics file {path} needs functions array"))?;
        for function in functions {
            interpret_rust_function(
                path,
                function,
                interpreted_cells,
                interpreted_morphisms,
                candidate_laws,
                binding_candidates,
            )?;
        }
    }
    push_execution_links(
        input,
        "matched_functions",
        "rust-test:function-ref",
        "execution_case_matches_test_function",
        evidence_links,
    );
    Ok(())
}

fn interpret_rust_function(
    path: &str,
    function: &Value,
    interpreted_cells: &mut Vec<Value>,
    interpreted_morphisms: &mut Vec<Value>,
    candidate_laws: &mut Vec<Value>,
    binding_candidates: &mut Vec<Value>,
) -> Result<(), String> {
    let name = required_str(function, "name", &format!("rust semantics file {path}"))?;
    let source_id = rust_function_source_id(path, name);
    push_test_obligation_cell(interpreted_cells, &source_id, path, name);
    push_command_observations(
        function,
        "cli_observations",
        "CLI observation",
        interpreted_morphisms,
        candidate_laws,
        binding_candidates,
        &source_id,
    );
    push_data_observations(
        function,
        "json_observations",
        "JSON observation",
        interpreted_morphisms,
        candidate_laws,
        binding_candidates,
        &source_id,
    );
    Ok(())
}

fn interpret_generic_semantics(
    input: &Value,
    interpreted_cells: &mut Vec<Value>,
    interpreted_morphisms: &mut Vec<Value>,
    candidate_laws: &mut Vec<Value>,
    binding_candidates: &mut Vec<Value>,
    evidence_links: &mut Vec<Value>,
) -> Result<(), String> {
    let files = input
        .get("files")
        .and_then(Value::as_array)
        .ok_or_else(|| "test semantics input needs files array".to_owned())?;
    for file in files {
        let path = required_str(file, "path", "test semantics file")?;
        let tests = file
            .get("tests")
            .and_then(Value::as_array)
            .ok_or_else(|| format!("test semantics file {path} needs tests array"))?;
        for test in tests {
            interpret_generic_test(
                path,
                test,
                interpreted_cells,
                interpreted_morphisms,
                candidate_laws,
                binding_candidates,
            )?;
        }
    }
    push_execution_links(
        input,
        "matched_tests",
        "test:function-ref",
        "execution_case_matches_test",
        evidence_links,
    );
    Ok(())
}

fn interpret_generic_test(
    path: &str,
    test: &Value,
    interpreted_cells: &mut Vec<Value>,
    interpreted_morphisms: &mut Vec<Value>,
    candidate_laws: &mut Vec<Value>,
    binding_candidates: &mut Vec<Value>,
) -> Result<(), String> {
    let name = required_str(test, "name", &format!("test semantics file {path}"))?;
    let source_id = rust_function_source_id(path, name);
    push_test_obligation_cell(interpreted_cells, &source_id, path, name);
    push_command_observations(
        test,
        "command_observations",
        "command observation",
        interpreted_morphisms,
        candidate_laws,
        binding_candidates,
        &source_id,
    );
    push_data_observations(
        test,
        "data_observations",
        "data observation",
        interpreted_morphisms,
        candidate_laws,
        binding_candidates,
        &source_id,
    );
    Ok(())
}

fn push_command_observations(
    value: &Value,
    field: &str,
    fallback_label: &'static str,
    interpreted_morphisms: &mut Vec<Value>,
    candidate_laws: &mut Vec<Value>,
    binding_candidates: &mut Vec<Value>,
    source_id: &str,
) {
    for observation in array_field(value, field) {
        let label = observation
            .get("label")
            .and_then(Value::as_str)
            .unwrap_or(fallback_label);
        let tokens = observation
            .get("tokens")
            .and_then(Value::as_array)
            .map(|values| string_array(values))
            .unwrap_or_default();
        push_command_contract_candidate(
            interpreted_morphisms,
            candidate_laws,
            binding_candidates,
            source_id,
            label,
            tokens,
        );
    }
}

fn push_data_observations(
    value: &Value,
    field: &str,
    fallback_label: &'static str,
    interpreted_morphisms: &mut Vec<Value>,
    candidate_laws: &mut Vec<Value>,
    binding_candidates: &mut Vec<Value>,
    source_id: &str,
) {
    for observation in array_field(value, field) {
        let label = observation
            .get("label")
            .and_then(Value::as_str)
            .unwrap_or(fallback_label);
        let observation_type = observation
            .get("observation_type")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        push_data_contract_candidate(
            interpreted_morphisms,
            candidate_laws,
            binding_candidates,
            source_id,
            label,
            observation_type,
        );
    }
}

fn push_execution_links(
    input: &Value,
    matched_field: &str,
    target_prefix: &str,
    relation_type: &str,
    evidence_links: &mut Vec<Value>,
) {
    for case in array_field(input, "execution_cases") {
        let name = case
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("unnamed execution case");
        let status = case
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        let case_id = format!("execution-case:{}", slug(name));
        for matched in array_field(case, matched_field) {
            push_execution_link(
                evidence_links,
                &case_id,
                matched,
                target_prefix,
                relation_type,
                status,
            );
        }
    }
}

fn push_execution_link(
    evidence_links: &mut Vec<Value>,
    case_id: &str,
    matched: &Value,
    target_prefix: &str,
    relation_type: &str,
    status: &str,
) {
    let matched_name = matched.as_str().unwrap_or_default();
    if matched_name.is_empty() {
        return;
    }
    evidence_links.push(json!({
        "id": format!("evidence-link:{}:{}", slug(case_id), slug(matched_name)),
        "source_id": case_id,
        "target_id": format!("{target_prefix}:{}", slug(matched_name)),
        "relation_type": relation_type,
        "status": status,
        "confidence": 0.62
    }));
}

fn required_str<'a>(value: &'a Value, field: &str, label: &str) -> Result<&'a str, String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{label} needs {field}"))
}

fn push_test_obligation_cell(cells: &mut Vec<Value>, source_id: &str, path: &str, name: &str) {
    cells.push(json!({
        "id": format!("interpreted-cell:{}", slug(source_id)),
        "cell_type": "interpreted_test_obligation_candidate",
        "label": format!("Interpreted test obligation {name}"),
        "source_ids": [source_id],
        "interpretation": format!("AI agent candidate: test {name} in {path} may verify behavior named by its assertions and observations."),
        "confidence": 0.58,
        "review_status": "unreviewed"
    }));
}

fn push_command_contract_candidate(
    morphisms: &mut Vec<Value>,
    laws: &mut Vec<Value>,
    bindings: &mut Vec<Value>,
    source_id: &str,
    label: &str,
    tokens: Vec<String>,
) {
    let law_id = format!("candidate-law:command-contract:{}", slug(label));
    laws.push(json!({
        "id": law_id,
        "summary": format!("Command observation {label} should preserve its documented contract."),
        "source_ids": [source_id],
        "confidence": 0.56,
        "review_status": "unreviewed"
    }));
    morphisms.push(json!({
        "id": format!("interpreted-morphism:{}:{}", slug(source_id), slug(label)),
        "morphism_type": "interprets_test_as_command_contract_candidate",
        "source_ids": [source_id],
        "target_ids": [law_id],
        "interpretation": format!("AI agent candidate: CLI observation {label} suggests command-contract verification."),
        "confidence": 0.57,
        "review_status": "unreviewed"
    }));
    bindings.push(json!({
        "id": format!("binding-candidate:{}", slug(label)),
        "semantic_role": "command_contract_verification",
        "trigger_terms": tokens,
        "candidate_target_ids": [law_id],
        "source_ids": [source_id],
        "rationale": format!("Observed CLI tokens for {label}."),
        "confidence": 0.55,
        "review_status": "unreviewed"
    }));
}

fn push_data_contract_candidate(
    morphisms: &mut Vec<Value>,
    laws: &mut Vec<Value>,
    bindings: &mut Vec<Value>,
    source_id: &str,
    label: &str,
    observation_type: &str,
) {
    let role = match observation_type {
        "schema_id" => "schema_identity_preservation",
        "field" | "json_field" => "json_field_contract_observation",
        _ => "data_contract_observation",
    };
    let law_id = format!("candidate-law:{role}:{}", slug(label));
    laws.push(json!({
        "id": law_id,
        "summary": format!("Data observation {label} should preserve the {role} property."),
        "source_ids": [source_id],
        "confidence": 0.6,
        "review_status": "unreviewed"
    }));
    morphisms.push(json!({
        "id": format!("interpreted-morphism:{}:{}", slug(source_id), slug(label)),
        "morphism_type": "interprets_test_as_data_contract_candidate",
        "source_ids": [source_id],
        "target_ids": [law_id],
        "interpretation": format!("AI agent candidate: data observation {label} suggests {role}."),
        "confidence": 0.6,
        "review_status": "unreviewed"
    }));
    bindings.push(json!({
        "id": format!("binding-candidate:{}", slug(label)),
        "semantic_role": role,
        "trigger_terms": [label],
        "candidate_target_ids": [law_id],
        "source_ids": [source_id],
        "rationale": format!("Observed structured data assertion {label}."),
        "confidence": 0.58,
        "review_status": "unreviewed"
    }));
}

fn rust_function_source_id(path: &str, name: &str) -> String {
    format!("rust-test:function:{}:{}", slug(path), slug(name))
}

fn array_field<'a>(value: &'a Value, field: &str) -> Vec<&'a Value> {
    value
        .get(field)
        .and_then(Value::as_array)
        .map(|values| values.iter().collect())
        .unwrap_or_default()
}

fn string_array(values: &[Value]) -> Vec<String> {
    values
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect()
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
