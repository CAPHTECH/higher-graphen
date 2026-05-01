fn signals_for_changes(
    changes: &[GitChange],
    tests: &[TestGapInputTest],
    accepted_test_kinds: &[TestGapTestType],
    diff_analysis: &GitDiffAnalysis,
    diff_evidence_id: &Id,
) -> Result<Vec<TestGapInputRiskSignal>, String> {
    let mut signals = Vec::new();
    push_missing_test_signal(&mut signals, changes, tests, accepted_test_kinds)?;
    push_diff_signal(
        &mut signals,
        "signal:test-gap:public-api-change",
        TestGapRiskSignalType::PublicApiChange,
        "Diff changes public Rust API-like declarations.",
        &diff_analysis.public_api_ids,
        Severity::High,
        0.74,
    )?;
    push_diff_signal(
        &mut signals,
        "signal:test-gap:error-path-change",
        TestGapRiskSignalType::ErrorPathChange,
        "Diff adds panic, unwrap/expect, or placeholder control-flow paths.",
        &diff_analysis.panic_or_placeholder_ids,
        Severity::Medium,
        0.7,
    )?;
    push_diff_signal(
        &mut signals,
        "signal:test-gap:boundary-change",
        TestGapRiskSignalType::BoundaryChange,
        "Diff changes boundary, incidence, or composition structure between finite code elements.",
        &diff_analysis.structural_boundary_ids,
        Severity::High,
        0.73,
    )?;
    push_size_signal(&mut signals, changes, diff_evidence_id)?;
    Ok(signals)
}

fn push_missing_test_signal(
    signals: &mut Vec<TestGapInputRiskSignal>,
    changes: &[GitChange],
    tests: &[TestGapInputTest],
    accepted_test_kinds: &[TestGapTestType],
) -> Result<(), String> {
    let changed_accepted_test_targets = tests
        .iter()
        .filter(|test| accepted_test_kinds.contains(&test.test_type))
        .flat_map(|test| test.target_ids.iter().cloned())
        .collect::<BTreeSet<_>>();
    let uncovered = changes
        .iter()
        .filter(|change| is_source_code_path(&change.path))
        .map(|change| {
            let symbol_id = id(format!("symbol:{}:changed-behavior", slug(&change.path)))?;
            Ok((file_id(&change.path)?, symbol_id))
        })
        .filter_map(|result: Result<(Id, Id), String>| match result {
            Ok((file_id, symbol_id)) if !changed_accepted_test_targets.contains(&symbol_id) => {
                Some(Ok(file_id))
            }
            Ok(_) => None,
            Err(error) => Some(Err(error)),
        })
        .collect::<Result<Vec<_>, _>>()?;

    if uncovered.is_empty() {
        return Ok(());
    }

    signals.push(TestGapInputRiskSignal {
        id: id("signal:test-gap:changed-source-without-accepted-test")?,
        signal_type: TestGapRiskSignalType::TestGap,
        summary: format!(
            "Input snapshot contains {} source files without a matched policy-accepted test.",
            uncovered.len()
        ),
        source_ids: uncovered,
        severity: Severity::Medium,
        confidence: confidence(0.72)?,
    });
    Ok(())
}

fn push_size_signal(
    signals: &mut Vec<TestGapInputRiskSignal>,
    changes: &[GitChange],
    diff_evidence_id: &Id,
) -> Result<(), String> {
    let total_lines = changes
        .iter()
        .map(|change| change.additions + change.deletions)
        .sum::<u32>();
    if changes.len() < 6 && total_lines < 500 {
        return Ok(());
    }
    signals.push(TestGapInputRiskSignal {
        id: id("signal:test-gap:large-git-change")?,
        signal_type: TestGapRiskSignalType::Custom,
        summary: format!(
            "Git range changes {} files and {} lines.",
            changes.len(),
            total_lines
        ),
        source_ids: vec![diff_evidence_id.clone()],
        severity: if total_lines >= 1200 {
            Severity::High
        } else {
            Severity::Medium
        },
        confidence: confidence(0.82)?,
    });
    Ok(())
}

fn push_diff_signal(
    signals: &mut Vec<TestGapInputRiskSignal>,
    id_value: &str,
    signal_type: TestGapRiskSignalType,
    summary: &str,
    source_ids: &[Id],
    severity: Severity,
    confidence_value: f64,
) -> Result<(), String> {
    if source_ids.is_empty() {
        return Ok(());
    }
    signals.push(TestGapInputRiskSignal {
        id: id(id_value)?,
        signal_type,
        summary: summary.to_owned(),
        source_ids: source_ids.to_vec(),
        severity,
        confidence: confidence(confidence_value)?,
    });
    Ok(())
}

fn matching_symbol_ids(test_path: &str, symbols: &[TestGapInputSymbol]) -> Vec<Id> {
    include!("matching_symbol_ids_body.rs")
}

fn push_matching_symbols(ids: &mut Vec<Id>, symbols: &[TestGapInputSymbol], symbol_ids: &[&str]) {
    for symbol_id in symbol_ids {
        if symbols
            .iter()
            .any(|symbol| symbol.id.as_str() == *symbol_id)
        {
            if let Ok(id) = id(*symbol_id) {
                push_unique_id(ids, id);
            }
        }
    }
}

fn push_unique_id(ids: &mut Vec<Id>, id: Id) {
    if !ids.contains(&id) {
        ids.push(id);
    }
}
