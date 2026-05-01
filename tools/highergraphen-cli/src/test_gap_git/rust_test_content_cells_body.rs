{
    let Some(document) = extract_rust_test_semantics(contents) else {
        return Ok(());
    };
    if document.functions.is_empty() {
        return Ok(());
    }
    content_model.test_files.insert(change.path.clone());
    let file_cell_id = id(format!(
        "semantic:rust-test:file:{}:{}",
        slug(&change.path),
        revision.as_str()
    ))?;
    push_higher_order_cell(
        &mut content_model.structural,
        file_cell_id.clone(),
        "rust_test_file_revision",
        format!(
            "Rust test file {} at {} from {}",
            change.path,
            revision.as_str(),
            source_path
        ),
        0,
        change,
        diff_evidence_id,
        0.74,
    )?;

    for function in &document.functions {
        let function_slug = slug(&function.name);
        let function_id = id(format!(
            "semantic:rust-test:function:{}:{}:{}",
            slug(&change.path),
            revision.as_str(),
            function_slug
        ))?;
        push_higher_order_cell(
            &mut content_model.structural,
            function_id.clone(),
            "rust_test_function",
            format!("Rust test function {}", function.name),
            0,
            change,
            diff_evidence_id,
            0.76,
        )?;
        push_higher_order_incidence(
            &mut content_model.structural,
            format!(
                "incidence:semantic:rust-test:file-contains-test:{}:{}:{}",
                slug(&change.path),
                revision.as_str(),
                function_slug
            ),
            file_cell_id.clone(),
            function_id.clone(),
            "contains_test_function",
            diff_evidence_id,
            0.76,
        )?;

        let mut evidence_cell_ids = vec![function_id.clone()];
        for (index, macro_name) in function.assertion_macros.iter().enumerate() {
            let assertion_number = index + 1;
            let assertion_id = id(format!(
                "semantic:rust-test:assertion:{}:{}:{}:{}",
                slug(&change.path),
                revision.as_str(),
                function_slug,
                assertion_number
            ))?;
            push_higher_order_cell(
                &mut content_model.structural,
                assertion_id.clone(),
                "rust_test_assertion",
                format!("Rust test assertion {macro_name}! in {}", function.name),
                0,
                change,
                diff_evidence_id,
                0.72,
            )?;
            push_higher_order_incidence(
                &mut content_model.structural,
                format!(
                    "incidence:semantic:rust-test:test-contains-assertion:{}:{}:{}:{}",
                    slug(&change.path),
                    revision.as_str(),
                    function_slug,
                    assertion_number
                ),
                function_id.clone(),
                assertion_id.clone(),
                "contains_assertion",
                diff_evidence_id,
                0.72,
            )?;
            evidence_cell_ids.push(assertion_id);
        }

        for observation in &function.cli_observations {
            let observation_label =
                hg_cli_observation_label(binding_rules, &observation.tokens, &observation.label);
            let cli_id = id(format!(
                "semantic:rust-test:cli-invocation:{}:{}:{}:{}",
                slug(&change.path),
                revision.as_str(),
                function_slug,
                slug(&observation_label)
            ))?;
            push_higher_order_cell(
                &mut content_model.structural,
                cli_id.clone(),
                "rust_test_cli_invocation",
                format!("Rust test observes CLI invocation {observation_label}"),
                0,
                change,
                diff_evidence_id,
                0.74,
            )?;
            push_higher_order_incidence(
                &mut content_model.structural,
                format!(
                    "incidence:semantic:rust-test:test-observes-cli:{}:{}:{}:{}",
                    slug(&change.path),
                    revision.as_str(),
                    function_slug,
                    slug(&observation_label)
                ),
                function_id.clone(),
                cli_id.clone(),
                "observes_cli_invocation",
                diff_evidence_id,
                0.74,
            )?;
            evidence_cell_ids.push(cli_id);
        }

        for observation in &function.json_observations {
            let observation_id = id(format!(
                "semantic:rust-test:json-observation:{}:{}:{}:{}",
                slug(&change.path),
                revision.as_str(),
                function_slug,
                slug(&observation.label)
            ))?;
            push_higher_order_cell(
                &mut content_model.structural,
                observation_id.clone(),
                "rust_test_json_observation",
                format!("Rust test observes JSON {}", observation.label),
                0,
                change,
                diff_evidence_id,
                0.74,
            )?;
            push_higher_order_incidence(
                &mut content_model.structural,
                format!(
                    "incidence:semantic:rust-test:test-observes-json:{}:{}:{}:{}",
                    slug(&change.path),
                    revision.as_str(),
                    function_slug,
                    slug(&observation.label)
                ),
                function_id.clone(),
                observation_id.clone(),
                "observes_json_contract",
                diff_evidence_id,
                0.74,
            )?;
            evidence_cell_ids.push(observation_id);
        }

        let mut binding_terms = function.string_literals.clone();
        binding_terms.insert(function.name.clone());
        let target_ids =
            hg_rust_test_content_target_ids(target_model, binding_rules, &binding_terms)?;
        for target_id in target_ids {
            push_unique_id(
                content_model
                    .target_ids_by_file
                    .entry(change.path.clone())
                    .or_default(),
                target_id.clone(),
            );
            push_rust_test_content_morphism(
                &mut content_model.structural,
                target_model,
                change,
                &function_slug,
                &evidence_cell_ids,
                target_id,
            )?;
        }
    }
    Ok(())
}
