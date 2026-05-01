#[allow(clippy::too_many_arguments)]
fn push_rust_test_content_cells(
    content_model: &mut RustTestContentModel,
    target_model: &StructuralModel,
    change: &GitChange,
    revision: SemanticRevision,
    source_path: &str,
    contents: &str,
    binding_rules: &HgRustTestBindingRules,
    diff_evidence_id: &Id,
) -> Result<(), String> {
    include!("rust_test_content_cells_body.rs")
}

fn hg_cli_observation_label(
    binding_rules: &HgRustTestBindingRules,
    tokens: &[String],
    fallback: &str,
) -> String {
    for rule in &binding_rules.rules {
        if rule.cli_label.is_some() && contains_all_tokens(tokens, &rule.trigger_terms) {
            return rule.cli_label.clone().expect("checked label");
        }
    }
    fallback.to_owned()
}

fn contains_all_tokens(tokens: &[String], expected: &[String]) -> bool {
    expected
        .iter()
        .all(|value| tokens.iter().any(|token| token == value))
}

fn hg_rust_test_content_target_ids(
    target_model: &StructuralModel,
    binding_rules: &HgRustTestBindingRules,
    strings: &BTreeSet<String>,
) -> Result<Vec<Id>, String> {
    let mut target_ids = Vec::new();
    for value in strings {
        push_model_id_if_present(target_model, &mut target_ids, value)?;
    }

    for rule in &binding_rules.rules {
        if contains_all_owned_strings(strings, &rule.trigger_terms) {
            for target_id in &rule.target_ids {
                push_model_id_if_present(target_model, &mut target_ids, target_id)?;
            }
        }
    }
    Ok(target_ids)
}

fn contains_all_owned_strings(strings: &BTreeSet<String>, expected: &[String]) -> bool {
    expected.iter().all(|value| strings.contains(value))
        || expected
            .iter()
            .all(|value| strings.iter().any(|string| string.contains(value)))
}

fn push_rust_test_content_morphism(
    model: &mut StructuralModel,
    target_model: &StructuralModel,
    change: &GitChange,
    function_slug: &str,
    evidence_cell_ids: &[Id],
    target_id: Id,
) -> Result<(), String> {
    let morphism_id = id(format!(
        "morphism:test-gap:rust-test-content:{}:{}:{}",
        slug(&change.path),
        function_slug,
        slug(target_id.as_str())
    ))?;
    if model
        .morphisms
        .iter()
        .any(|morphism| morphism.id == morphism_id)
    {
        return Ok(());
    }
    let law_ids = law_ids_for_content_target(target_model, &target_id);
    model.morphisms.push(TestGapInputMorphism {
        id: morphism_id.clone(),
        morphism_type: "rust_test_content_evidence".to_owned(),
        source_ids: evidence_cell_ids.to_vec(),
        target_ids: vec![target_id.clone()],
        law_ids,
        requirement_ids: Vec::new(),
        expected_verification: None,
        confidence: Some(confidence(0.68)?),
    });
    Ok(())
}

fn law_ids_for_content_target(target_model: &StructuralModel, target_id: &Id) -> Vec<Id> {
    if target_model.laws.iter().any(|law| &law.id == target_id) {
        return vec![target_id.clone()];
    }
    target_model
        .morphisms
        .iter()
        .find(|morphism| &morphism.id == target_id)
        .map(|morphism| morphism.law_ids.clone())
        .unwrap_or_default()
}

fn push_model_id_if_present(
    target_model: &StructuralModel,
    target_ids: &mut Vec<Id>,
    candidate: &str,
) -> Result<(), String> {
    let Ok(candidate_id) = id(candidate) else {
        return Ok(());
    };
    if model_contains_id(target_model, &candidate_id) {
        push_unique_id(target_ids, candidate_id);
    }
    Ok(())
}

fn model_contains_id(model: &StructuralModel, candidate_id: &Id) -> bool {
    model
        .symbols
        .iter()
        .any(|symbol| &symbol.id == candidate_id)
        || model.laws.iter().any(|law| &law.id == candidate_id)
        || model
            .morphisms
            .iter()
            .any(|morphism| &morphism.id == candidate_id)
        || model
            .higher_order_cells
            .iter()
            .any(|cell| &cell.id == candidate_id)
}
