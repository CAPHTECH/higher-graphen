use super::*;

pub(super) fn detect_obstructions(
    input: &TestGapInputDocument,
) -> RuntimeResult<Vec<TestGapObstruction>> {
    let mut obstructions = Vec::new();
    detect_requirement_obstructions(input, &mut obstructions)?;
    detect_symbol_obstructions(input, &mut obstructions)?;
    detect_branch_obstructions(input, &mut obstructions)?;
    detect_law_obstructions(input, &mut obstructions)?;
    detect_morphism_obstructions(input, &mut obstructions)?;
    Ok(obstructions)
}

fn detect_requirement_obstructions(
    input: &TestGapInputDocument,
    obstructions: &mut Vec<TestGapObstruction>,
) -> RuntimeResult<()> {
    for requirement in &input.requirements {
        if requirement_needs_verification(requirement)
            && !has_accepted_test_for_requirement(input, &requirement.id)
        {
            obstructions.push(missing_requirement_obstruction(input, requirement)?);
        }
        if requirement_needs_regression(requirement)
            && !has_accepted_regression_test_for_requirement(input, &requirement.id)
        {
            obstructions.push(missing_regression_obstruction(input, requirement)?);
        }
    }
    Ok(())
}

fn detect_symbol_obstructions(
    input: &TestGapInputDocument,
    obstructions: &mut Vec<TestGapObstruction>,
) -> RuntimeResult<()> {
    for symbol in &input.symbols {
        if symbol_is_public_behavior(symbol)
            && !symbol_is_higher_order_obligation_surface(symbol)
            && !has_accepted_test_for_symbol(input, &symbol.id)
        {
            obstructions.push(missing_public_behavior_obstruction(input, symbol)?);
        }
    }
    Ok(())
}

fn detect_branch_obstructions(
    input: &TestGapInputDocument,
    obstructions: &mut Vec<TestGapObstruction>,
) -> RuntimeResult<()> {
    for branch in &input.branches {
        if branch_needs_unit_test(branch) && !has_accepted_test_for_branch(input, &branch.id) {
            obstructions.push(missing_branch_obstruction(input, branch)?);
        }
    }
    Ok(())
}

fn detect_law_obstructions(
    input: &TestGapInputDocument,
    obstructions: &mut Vec<TestGapObstruction>,
) -> RuntimeResult<()> {
    for law in &input.laws {
        if law.expected_verification.is_some()
            && !has_requirement_for_implementation(input, &law.id)
            && !has_accepted_verification_for_law(input, &law.id)
        {
            obstructions.push(missing_law_obstruction(input, law)?);
        }
    }
    Ok(())
}

fn detect_morphism_obstructions(
    input: &TestGapInputDocument,
    obstructions: &mut Vec<TestGapObstruction>,
) -> RuntimeResult<()> {
    for morphism in &input.morphisms {
        if morphism.expected_verification.is_some()
            && !has_accepted_verification_for_morphism(input, &morphism.id)
        {
            obstructions.push(missing_morphism_obstruction(input, morphism)?);
        }
    }
    Ok(())
}

pub(super) fn missing_requirement_obstruction(
    input: &TestGapInputDocument,
    requirement: &TestGapInputRequirement,
) -> RuntimeResult<TestGapObstruction> {
    Ok(TestGapObstruction {
        id: id(format!(
            "obstruction:test-gap:missing-requirement-verification:{}",
            slug(&requirement.id)
        ))?,
        obstruction_type: TestGapObstructionType::MissingRequirementVerification,
        title: format!("Missing verification for {}", requirement.summary),
        target_ids: requirement_target_ids(requirement),
        witness: json!({
            "requirement_id": requirement.id,
            "implementation_ids": requirement.implementation_ids,
            "missing_test_ids": [],
            "requirement_source_ids": requirement.source_ids,
            "expected_verification_kind": requirement.expected_verification.clone().unwrap_or_else(|| "unit_test".to_owned()),
            "accepted_test_kinds": accepted_test_kind_names(input),
        }),
        invariant_ids: vec![id(INVARIANT_REQUIREMENT_VERIFIED)?],
        evidence_ids: nonempty_source_ids(input, &requirement.id, &requirement.source_ids),
        severity: Severity::High,
        confidence: Confidence::new(0.82)?,
        review_status: ReviewStatus::Unreviewed,
    })
}

pub(super) fn missing_regression_obstruction(
    input: &TestGapInputDocument,
    requirement: &TestGapInputRequirement,
) -> RuntimeResult<TestGapObstruction> {
    Ok(TestGapObstruction {
        id: id(format!(
            "obstruction:test-gap:missing-regression-test:{}",
            slug(&requirement.id)
        ))?,
        obstruction_type: TestGapObstructionType::MissingRegressionTest,
        title: format!(
            "Missing accepted regression test for {}",
            requirement.summary
        ),
        target_ids: requirement_target_ids(requirement),
        witness: json!({
            "bug_fix_requirement_id": requirement.id,
            "changed_implementation_ids": requirement.implementation_ids,
            "failing_before_passing_after_expectation": "A unit regression test should fail before the bounded change and pass after it.",
            "related_issue_or_evidence_ids": requirement.source_ids,
        }),
        invariant_ids: vec![id(INVARIANT_REGRESSION_TEST_FOR_BUG_FIX)?],
        evidence_ids: nonempty_source_ids(input, &requirement.id, &requirement.source_ids),
        severity: Severity::High,
        confidence: Confidence::new(0.84)?,
        review_status: ReviewStatus::Unreviewed,
    })
}

pub(super) fn missing_public_behavior_obstruction(
    input: &TestGapInputDocument,
    symbol: &TestGapInputSymbol,
) -> RuntimeResult<TestGapObstruction> {
    Ok(TestGapObstruction {
        id: id(format!(
            "obstruction:test-gap:missing-public-unit-test:{}",
            slug(&symbol.id)
        ))?,
        obstruction_type: TestGapObstructionType::MissingPublicBehaviorUnitTest,
        title: format!("Missing accepted test for public behavior {}", symbol.name),
        target_ids: vec![symbol.id.clone(), symbol.file_id.clone()],
        witness: json!({
            "symbol_id": symbol.id,
            "visibility": symbol.visibility,
            "changed_behavior_summary": format!("{} changed in the bounded snapshot.", symbol.name),
            "existing_related_tests": related_test_ids_for_symbol(input, &symbol.id),
            "expected_unit_test_obligation": "public behavior covered",
            "accepted_test_kinds": accepted_test_kind_names(input),
        }),
        invariant_ids: vec![id(INVARIANT_PUBLIC_BEHAVIOR_COVERED)?],
        evidence_ids: nonempty_source_ids(input, &symbol.id, &symbol.source_ids),
        severity: Severity::Medium,
        confidence: Confidence::new(0.78)?,
        review_status: ReviewStatus::Unreviewed,
    })
}

pub(super) fn missing_branch_obstruction(
    input: &TestGapInputDocument,
    branch: &TestGapInputBranch,
) -> RuntimeResult<TestGapObstruction> {
    let (obstruction_type, invariant_id, title) = match branch.branch_type {
        TestGapBranchType::Boundary => (
            TestGapObstructionType::MissingBoundaryCaseUnitTest,
            INVARIANT_BOUNDARY_CASES_REPRESENTED,
            format!("Missing boundary accepted test for {}", branch.summary),
        ),
        TestGapBranchType::ErrorPath => (
            TestGapObstructionType::MissingErrorCaseUnitTest,
            INVARIANT_ERROR_CASES_REPRESENTED,
            format!("Missing error-case accepted test for {}", branch.summary),
        ),
        _ => (
            TestGapObstructionType::MissingBranchUnitTest,
            INVARIANT_BOUNDARY_CASES_REPRESENTED,
            format!("Missing branch accepted test for {}", branch.summary),
        ),
    };
    let coverage_ids = coverage_ids_for_target(input, &branch.id);
    Ok(TestGapObstruction {
        id: id(format!(
            "obstruction:test-gap:{}:{}",
            obstruction_slug(obstruction_type),
            slug(&branch.id)
        ))?,
        obstruction_type,
        title,
        target_ids: vec![branch.symbol_id.clone(), branch.id.clone()],
        witness: json!({
            "branch_id": branch.id,
            "parent_symbol_id": branch.symbol_id,
            "condition_summary": branch.summary,
            "boundary_type": branch.boundary_kind,
            "representative_value": branch.representative_value,
            "observed_branch_or_coverage_evidence": coverage_ids,
            "missing_test_relation": "No policy-accepted test exercises this branch in the bounded snapshot.",
            "accepted_test_kinds": accepted_test_kind_names(input),
        }),
        invariant_ids: vec![id(invariant_id)?],
        evidence_ids: nonempty_source_ids(input, &branch.id, &branch.source_ids),
        severity: Severity::Medium,
        confidence: Confidence::new(0.86)?,
        review_status: ReviewStatus::Unreviewed,
    })
}

pub(super) fn missing_law_obstruction(
    input: &TestGapInputDocument,
    law: &crate::test_gap_reports::TestGapInputLaw,
) -> RuntimeResult<TestGapObstruction> {
    let mut target_ids = vec![law.id.clone()];
    target_ids.extend(law.applies_to_ids.iter().cloned());
    Ok(TestGapObstruction {
        id: id(format!(
            "obstruction:test-gap:missing-law-verification:{}",
            slug(&law.id)
        ))?,
        obstruction_type: TestGapObstructionType::MissingRequirementVerification,
        title: format!("Missing accepted verification for law {}", law.summary),
        target_ids,
        witness: json!({
            "law_id": law.id,
            "applies_to_ids": law.applies_to_ids,
            "expected_verification_kind": law.expected_verification.clone().unwrap_or_else(|| "policy_accepted_verification".to_owned()),
            "accepted_test_kinds": accepted_test_kind_names(input),
            "missing_relation": "No policy-accepted verification cell closes this law in the bounded snapshot.",
        }),
        invariant_ids: vec![id(INVARIANT_REQUIREMENT_VERIFIED)?],
        evidence_ids: nonempty_source_ids(input, &law.id, &law.source_ids),
        severity: Severity::High,
        confidence: law.confidence.unwrap_or(Confidence::new(0.82)?),
        review_status: ReviewStatus::Unreviewed,
    })
}

pub(super) fn missing_morphism_obstruction(
    input: &TestGapInputDocument,
    morphism: &crate::test_gap_reports::TestGapInputMorphism,
) -> RuntimeResult<TestGapObstruction> {
    let mut target_ids = vec![morphism.id.clone()];
    target_ids.extend(morphism.source_ids.iter().cloned());
    target_ids.extend(morphism.target_ids.iter().cloned());
    Ok(TestGapObstruction {
        id: id(format!(
            "obstruction:test-gap:missing-morphism-verification:{}",
            slug(&morphism.id)
        ))?,
        obstruction_type: TestGapObstructionType::MissingRequirementVerification,
        title: format!(
            "Missing accepted verification for morphism {}",
            morphism.morphism_type
        ),
        target_ids,
        witness: json!({
            "morphism_id": morphism.id,
            "morphism_type": morphism.morphism_type,
            "source_ids": morphism.source_ids,
            "target_ids": morphism.target_ids,
            "law_ids": morphism.law_ids,
            "expected_verification_kind": morphism.expected_verification.clone().unwrap_or_else(|| "policy_accepted_verification".to_owned()),
            "accepted_test_kinds": accepted_test_kind_names(input),
            "missing_relation": "No policy-accepted verification cell closes this morphism in the bounded snapshot.",
        }),
        invariant_ids: vec![id(INVARIANT_REQUIREMENT_VERIFIED)?],
        evidence_ids: nonempty_source_ids(input, &morphism.id, &morphism.source_ids),
        severity: Severity::High,
        confidence: morphism.confidence.unwrap_or(Confidence::new(0.82)?),
        review_status: ReviewStatus::Unreviewed,
    })
}

pub(super) fn requirement_needs_verification(requirement: &TestGapInputRequirement) -> bool {
    requirement.in_scope || requirement.bug_fix
}

pub(super) fn requirement_needs_regression(requirement: &TestGapInputRequirement) -> bool {
    requirement.bug_fix
        || matches!(
            requirement.requirement_type,
            crate::test_gap_reports::TestGapRequirementType::BugFix
                | crate::test_gap_reports::TestGapRequirementType::Issue
        )
        || requirement
            .expected_verification
            .as_deref()
            .is_some_and(|value| value.contains("regression"))
}

pub(super) fn symbol_is_public_behavior(symbol: &TestGapInputSymbol) -> bool {
    symbol.public_api
        || matches!(
            symbol.visibility,
            crate::test_gap_reports::TestGapVisibility::Public
        )
}

pub(super) fn symbol_is_higher_order_obligation_surface(symbol: &TestGapInputSymbol) -> bool {
    let id = symbol.id.as_str();
    id.starts_with("law:")
        || id.starts_with("adapter:")
        || id.starts_with("command:")
        || id.starts_with("runner:")
        || id.starts_with("schema:")
        || id.starts_with("fixture:")
        || id.starts_with("validator:")
        || id.starts_with("theorem:")
        || id.starts_with("contract:")
        || id.starts_with("projection:")
        || id.starts_with("registry:")
        || id.starts_with("export:")
        || id.starts_with("test:")
}

pub(super) fn branch_needs_unit_test(branch: &TestGapInputBranch) -> bool {
    matches!(
        branch.branch_type,
        TestGapBranchType::Boundary
            | TestGapBranchType::Condition
            | TestGapBranchType::ErrorPath
            | TestGapBranchType::Branch
            | TestGapBranchType::StateTransition
            | TestGapBranchType::PatternArm
    )
}
