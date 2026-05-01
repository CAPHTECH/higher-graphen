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
    assert!(function.cli_observations.iter().any(|observation| {
        observation.tokens
            == vec![
                "acme".to_owned(),
                "audit".to_owned(),
                "--format".to_owned(),
                "json".to_owned(),
            ]
    }));
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
