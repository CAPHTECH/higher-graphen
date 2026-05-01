fn changed_files_for_input(
    changes: &[GitChange],
    symbols: &[TestGapInputSymbol],
    diff_evidence_id: &Id,
) -> Result<Vec<TestGapInputChangedFile>, String> {
    let symbol_ids_by_file = symbols
        .iter()
        .map(|symbol| (symbol.file_id.clone(), symbol.id.clone()))
        .fold(
            BTreeMap::<Id, Vec<Id>>::new(),
            |mut by_file, (file_id, symbol_id)| {
                by_file.entry(file_id).or_default().push(symbol_id);
                by_file
            },
        );

    changes
        .iter()
        .map(|change| {
            let file_id = file_id(&change.path)?;
            Ok(TestGapInputChangedFile {
                id: file_id.clone(),
                path: change.path.clone(),
                change_type: map_change_type(change.change_type),
                old_path: change.old_path.clone(),
                language: language_for_path(&change.path),
                additions: change.additions,
                deletions: change.deletions,
                symbol_ids: symbol_ids_by_file
                    .get(&file_id)
                    .cloned()
                    .unwrap_or_default(),
                context_ids: test_gap_context_ids_for_path(&change.path)?,
                source_ids: vec![diff_evidence_id.clone()],
            })
        })
        .collect()
}

fn symbols_for_changes(
    changes: &[GitChange],
    diff_evidence_id: &Id,
    diff_analysis: &GitDiffAnalysis,
) -> Result<Vec<TestGapInputSymbol>, String> {
    changes
        .iter()
        .filter(|change| is_source_code_path(&change.path))
        .map(|change| {
            let file_id = file_id(&change.path)?;
            let symbol_id = id(format!("symbol:{}:changed-behavior", slug(&change.path)))?;
            let structural_path = is_highergraphen_structural_path(&change.path);
            let public_api = !structural_path && diff_analysis.public_api_ids.contains(&file_id);
            Ok(TestGapInputSymbol {
                id: symbol_id,
                file_id,
                name: format!("Changed behavior in {}", change.path),
                kind: if public_api {
                    TestGapSymbolKind::PublicApi
                } else {
                    TestGapSymbolKind::Module
                },
                visibility: if public_api {
                    TestGapVisibility::Public
                } else {
                    TestGapVisibility::Unknown
                },
                public_api,
                path: Some(change.path.clone()),
                line_start: None,
                line_end: None,
                branch_ids: Vec::new(),
                requirement_ids: if structural_path {
                    Vec::new()
                } else {
                    vec![id(format!(
                        "requirement:{}:unit-verification",
                        slug(&change.path)
                    ))?]
                },
                context_ids: test_gap_context_ids_for_path(&change.path)?,
                source_ids: vec![diff_evidence_id.clone()],
            })
        })
        .collect()
}
