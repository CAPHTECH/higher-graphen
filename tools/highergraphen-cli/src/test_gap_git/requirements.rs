fn requirements_for_symbols(
    symbols: &[TestGapInputSymbol],
    diff_evidence_id: &Id,
    accepted_test_kinds: &[TestGapTestType],
) -> Result<Vec<TestGapInputRequirement>, String> {
    symbols
        .iter()
        .filter(|symbol| {
            symbol.id.as_str().ends_with(":changed-behavior")
                && !symbol
                    .path
                    .as_deref()
                    .is_some_and(is_highergraphen_structural_path)
        })
        .map(|symbol| {
            let requirement_id = id(format!(
                "requirement:{}:unit-verification",
                slug(symbol.path())
            ))?;
            Ok(TestGapInputRequirement {
                id: requirement_id,
                requirement_type: TestGapRequirementType::Custom,
                summary: format!(
                    "Changed behavior in {} has policy-accepted test verification",
                    symbol.path()
                ),
                in_scope: true,
                bug_fix: false,
                implementation_ids: vec![symbol.id.clone()],
                source_ids: vec![diff_evidence_id.clone(), symbol.file_id.clone()],
                expected_verification: Some(expected_verification_label(accepted_test_kinds)),
            })
        })
        .collect()
}

fn structural_requirements(
    structural: &StructuralModel,
    diff_evidence_id: &Id,
    accepted_test_kinds: &[TestGapTestType],
) -> Result<Vec<TestGapInputRequirement>, String> {
    include!("structural_requirements_body.rs")
}

fn push_structural_requirement(
    requirements: &mut Vec<TestGapInputRequirement>,
    structural: &StructuralModel,
    requirement_id: &str,
    summary: &str,
    implementation_ids: &[&str],
    diff_evidence_id: &Id,
    accepted_test_kinds: &[TestGapTestType],
) -> Result<(), String> {
    let implementation_ids = implementation_ids
        .iter()
        .map(|value| id(*value))
        .collect::<Result<Vec<_>, _>>()?;
    if implementation_ids
        .iter()
        .any(|implementation_id| !has_structural_symbol(&structural.symbols, implementation_id))
    {
        return Ok(());
    }
    let mut source_ids = vec![diff_evidence_id.clone()];
    source_ids.extend(implementation_ids.iter().cloned());
    requirements.push(TestGapInputRequirement {
        id: id(requirement_id)?,
        requirement_type: TestGapRequirementType::Custom,
        summary: summary.to_owned(),
        in_scope: true,
        bug_fix: false,
        implementation_ids,
        source_ids,
        expected_verification: Some(expected_verification_label(accepted_test_kinds)),
    });
    Ok(())
}

fn push_law_requirement(
    requirements: &mut Vec<TestGapInputRequirement>,
    structural: &StructuralModel,
    law_symbol_id: &str,
    requirement_id: &str,
    summary: &str,
    diff_evidence_id: &Id,
    accepted_test_kinds: &[TestGapTestType],
) -> Result<(), String> {
    let law_symbol_id = id(law_symbol_id)?;
    if !has_structural_symbol(&structural.symbols, &law_symbol_id) {
        return Ok(());
    }
    requirements.push(TestGapInputRequirement {
        id: id(requirement_id)?,
        requirement_type: TestGapRequirementType::Custom,
        summary: summary.to_owned(),
        in_scope: true,
        bug_fix: false,
        implementation_ids: vec![law_symbol_id.clone()],
        source_ids: vec![diff_evidence_id.clone(), law_symbol_id],
        expected_verification: Some(expected_verification_label(accepted_test_kinds)),
    });
    Ok(())
}
