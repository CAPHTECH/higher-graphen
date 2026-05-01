use super::*;

pub(super) fn accepted_fact_ids(input: &TestGapInputDocument) -> Vec<Id> {
    let mut ids = Vec::new();
    push_unique(&mut ids, input.repository.id.clone());
    push_unique(&mut ids, input.change_set.id.clone());
    append_basic_fact_ids(input, &mut ids);
    append_structural_fact_ids(input, &mut ids);
    append_contextual_fact_ids(input, &mut ids);
    ids
}

fn append_basic_fact_ids(input: &TestGapInputDocument, ids: &mut Vec<Id>) {
    for file in &input.changed_files {
        push_unique(ids, file.id.clone());
    }
    for symbol in &input.symbols {
        push_unique(ids, symbol.id.clone());
    }
    for branch in &input.branches {
        push_unique(ids, branch.id.clone());
    }
    for requirement in &input.requirements {
        push_unique(ids, requirement.id.clone());
    }
    for test in &input.tests {
        push_unique(ids, test.id.clone());
    }
    for coverage in &input.coverage {
        push_unique(ids, coverage.id.clone());
    }
    for edge in &input.dependency_edges {
        push_unique(ids, edge.id.clone());
    }
}

fn append_structural_fact_ids(input: &TestGapInputDocument, ids: &mut Vec<Id>) {
    for cell in &input.higher_order_cells {
        push_unique(ids, cell.id.clone());
    }
    for incidence in &input.higher_order_incidences {
        push_unique(ids, incidence.id.clone());
    }
    for morphism in &input.morphisms {
        push_unique(ids, morphism.id.clone());
    }
    for law in &input.laws {
        push_unique(ids, law.id.clone());
    }
    for verification in &input.verification_cells {
        push_unique(ids, verification.id.clone());
    }
}

fn append_contextual_fact_ids(input: &TestGapInputDocument, ids: &mut Vec<Id>) {
    for context in &input.contexts {
        push_unique(ids, context.id.clone());
    }
    for evidence in &input.evidence {
        push_unique(ids, evidence.id.clone());
    }
    for signal in &input.signals {
        push_unique(ids, signal.id.clone());
    }
}

pub(super) fn evaluated_invariant_ids() -> RuntimeResult<Vec<Id>> {
    [
        INVARIANT_REQUIREMENT_VERIFIED,
        INVARIANT_PUBLIC_BEHAVIOR_COVERED,
        INVARIANT_BOUNDARY_CASES_REPRESENTED,
        INVARIANT_ERROR_CASES_REPRESENTED,
        INVARIANT_REGRESSION_TEST_FOR_BUG_FIX,
        INVARIANT_PROJECTION_INFORMATION_LOSS,
    ]
    .into_iter()
    .map(id)
    .collect()
}

pub(super) fn result_source_ids(
    accepted_fact_ids: &[Id],
    invariant_ids: &[Id],
    proof_objects: &[TestGapProofObject],
    counterexamples: &[TestGapCounterexample],
    obstructions: &[TestGapObstruction],
    candidates: &[TestGapCompletionCandidate],
) -> Vec<Id> {
    let mut ids = Vec::new();
    for fact_id in accepted_fact_ids {
        push_unique(&mut ids, fact_id.clone());
    }
    for invariant_id in invariant_ids {
        push_unique(&mut ids, invariant_id.clone());
    }
    for proof in proof_objects {
        push_unique(&mut ids, proof.id.clone());
        for witness_id in &proof.witness_ids {
            push_unique(&mut ids, witness_id.clone());
        }
        for verification_id in &proof.verified_by_ids {
            push_unique(&mut ids, verification_id.clone());
        }
    }
    for counterexample in counterexamples {
        push_unique(&mut ids, counterexample.id.clone());
        for path_id in &counterexample.path_ids {
            push_unique(&mut ids, path_id.clone());
        }
    }
    for obstruction in obstructions {
        push_unique(&mut ids, obstruction.id.clone());
        for source_id in obstruction_source_ids(obstruction) {
            push_unique(&mut ids, source_id);
        }
    }
    for candidate in candidates {
        push_unique(&mut ids, candidate.id.clone());
        for source_id in &candidate.provenance.source_ids {
            push_unique(&mut ids, source_id.clone());
        }
    }
    ids
}

pub(super) fn ensure_detector_output_unreviewed(
    obstructions: &[TestGapObstruction],
    candidates: &[TestGapCompletionCandidate],
) -> RuntimeResult<()> {
    if obstructions
        .iter()
        .any(|obstruction| obstruction.review_status != ReviewStatus::Unreviewed)
    {
        return Err(validation_error(
            "detector obstructions must remain unreviewed",
        ));
    }
    if candidates
        .iter()
        .any(|candidate| candidate.review_status != ReviewStatus::Unreviewed)
    {
        return Err(validation_error(
            "missing-test candidates must remain unreviewed",
        ));
    }
    Ok(())
}

pub(super) fn has_accepted_test_for_requirement(
    input: &TestGapInputDocument,
    requirement_id: &Id,
) -> bool {
    input.tests.iter().any(|test| {
        accepts_test_kind(input, test.test_type) && test.requirement_ids.contains(requirement_id)
    })
}

pub(super) fn has_requirement_for_implementation(
    input: &TestGapInputDocument,
    implementation_id: &Id,
) -> bool {
    input
        .requirements
        .iter()
        .any(|requirement| requirement.implementation_ids.contains(implementation_id))
}

pub(super) fn has_accepted_regression_test_for_requirement(
    input: &TestGapInputDocument,
    requirement_id: &Id,
) -> bool {
    input.tests.iter().any(|test| {
        accepts_test_kind(input, test.test_type)
            && test.is_regression
            && test.requirement_ids.contains(requirement_id)
    })
}

pub(super) fn has_accepted_test_for_symbol(input: &TestGapInputDocument, symbol_id: &Id) -> bool {
    input
        .tests
        .iter()
        .any(|test| accepts_test_kind(input, test.test_type) && test.target_ids.contains(symbol_id))
        || input.coverage.iter().any(|coverage| {
            &coverage.target_id == symbol_id
                && !coverage.covered_by_test_ids.is_empty()
                && coverage
                    .covered_by_test_ids
                    .iter()
                    .any(|test_id| is_accepted_test(input, test_id))
        })
}

pub(super) fn has_accepted_test_for_branch(input: &TestGapInputDocument, branch_id: &Id) -> bool {
    input
        .tests
        .iter()
        .any(|test| accepts_test_kind(input, test.test_type) && test.branch_ids.contains(branch_id))
        || input.coverage.iter().any(|coverage| {
            &coverage.target_id == branch_id
                && matches!(
                    coverage.status,
                    crate::test_gap_reports::TestGapCoverageStatus::Covered
                        | crate::test_gap_reports::TestGapCoverageStatus::Partial
                )
                && coverage
                    .covered_by_test_ids
                    .iter()
                    .any(|test_id| is_accepted_test(input, test_id))
        })
}

pub(super) fn is_accepted_test(input: &TestGapInputDocument, test_id: &Id) -> bool {
    input
        .tests
        .iter()
        .any(|test| &test.id == test_id && accepts_test_kind(input, test.test_type))
}

pub(super) fn has_accepted_verification_for_law(input: &TestGapInputDocument, law_id: &Id) -> bool {
    !accepted_verification_ids_for_law(input, law_id).is_empty()
}

pub(super) fn accepted_verification_ids_for_law(
    input: &TestGapInputDocument,
    law_id: &Id,
) -> Vec<Id> {
    let mut ids = Vec::new();
    for verification in &input.verification_cells {
        if accepts_test_kind(input, verification.test_type) && verification.law_ids.contains(law_id)
        {
            push_unique(&mut ids, verification.id.clone());
        }
    }
    for test in &input.tests {
        if accepts_test_kind(input, test.test_type) && test.target_ids.contains(law_id) {
            push_unique(&mut ids, test.id.clone());
        }
    }
    ids
}

pub(super) fn has_accepted_verification_for_morphism(
    input: &TestGapInputDocument,
    morphism_id: &Id,
) -> bool {
    !accepted_verification_ids_for_morphism(input, morphism_id).is_empty()
}

pub(super) fn accepted_verification_ids_for_morphism(
    input: &TestGapInputDocument,
    morphism_id: &Id,
) -> Vec<Id> {
    let mut ids = Vec::new();
    for verification in &input.verification_cells {
        if accepts_test_kind(input, verification.test_type)
            && verification.morphism_ids.contains(morphism_id)
        {
            push_unique(&mut ids, verification.id.clone());
        }
    }
    for test in &input.tests {
        if accepts_test_kind(input, test.test_type) && test.target_ids.contains(morphism_id) {
            push_unique(&mut ids, test.id.clone());
        }
    }
    ids
}

pub(super) fn related_test_ids_for_symbol(input: &TestGapInputDocument, symbol_id: &Id) -> Vec<Id> {
    let mut ids = Vec::new();
    for test in &input.tests {
        if test.target_ids.contains(symbol_id) {
            push_unique(&mut ids, test.id.clone());
        }
    }
    for coverage in &input.coverage {
        if &coverage.target_id == symbol_id {
            for test_id in &coverage.covered_by_test_ids {
                push_unique(&mut ids, test_id.clone());
            }
        }
    }
    ids
}

pub(super) fn coverage_ids_for_target(input: &TestGapInputDocument, target_id: &Id) -> Vec<Id> {
    input
        .coverage
        .iter()
        .filter(|coverage| &coverage.target_id == target_id)
        .map(|coverage| coverage.id.clone())
        .collect()
}

pub(super) fn requirement_target_ids(requirement: &TestGapInputRequirement) -> Vec<Id> {
    let mut ids = vec![requirement.id.clone()];
    for implementation_id in &requirement.implementation_ids {
        push_unique(&mut ids, implementation_id.clone());
    }
    ids
}

pub(super) fn nonempty_source_ids(
    input: &TestGapInputDocument,
    id: &Id,
    source_ids: &[Id],
) -> Vec<Id> {
    let mut ids = vec![id.clone()];
    for source_id in source_ids {
        push_unique(&mut ids, source_id.clone());
    }
    if ids.len() == 1 {
        push_unique(&mut ids, input.change_set.id.clone());
    }
    ids
}

pub(super) fn obstruction_source_ids(obstruction: &TestGapObstruction) -> Vec<Id> {
    let mut ids = vec![obstruction.id.clone()];
    for target_id in &obstruction.target_ids {
        push_unique(&mut ids, target_id.clone());
    }
    for evidence_id in &obstruction.evidence_ids {
        push_unique(&mut ids, evidence_id.clone());
    }
    for invariant_id in &obstruction.invariant_ids {
        push_unique(&mut ids, invariant_id.clone());
    }
    ids
}

pub(super) fn effective_context_ids(input: &TestGapInputDocument) -> RuntimeResult<Vec<Id>> {
    let mut context_ids = Vec::new();
    for context in &input.contexts {
        push_unique(&mut context_ids, context.id.clone());
    }
    if context_ids.is_empty() {
        context_ids.push(id(format!("context:test-gap:{}", input.repository.id))?);
    }
    Ok(context_ids)
}

pub(super) fn contexts_or_default(context_ids: &[Id], default_context_ids: &[Id]) -> Vec<Id> {
    if context_ids.is_empty() {
        default_context_ids.to_vec()
    } else {
        context_ids.to_vec()
    }
}

pub(super) fn fact_provenance(
    input: &TestGapInputDocument,
    confidence: Confidence,
    source_local_id: Option<&str>,
) -> RuntimeResult<Provenance> {
    let mut source = SourceRef::new(input.source.kind.clone());
    if let Some(uri) = &input.source.uri {
        source = source.with_uri(uri.clone())?;
    }
    if let Some(title) = &input.source.title {
        source = source.with_title(title.clone())?;
    }
    if let Some(captured_at) = &input.source.captured_at {
        source = source.with_captured_at(captured_at.clone())?;
    }
    if let Some(source_local_id) = source_local_id {
        source = source.with_source_local_id(source_local_id.to_owned())?;
    }
    Ok(Provenance::new(source, confidence)
        .with_review_status(ReviewStatus::Accepted)
        .with_extraction_method(EXTRACTION_METHOD)?)
}

pub(super) fn space_id(input: &TestGapInputDocument) -> RuntimeResult<Id> {
    id(format!(
        "space:test-gap:{}:{}",
        input.repository.id, input.change_set.id
    ))
}

pub(super) fn incidence_id(prefix: &str, from_id: &Id, to_id: &Id) -> RuntimeResult<Id> {
    id(format!("{prefix}:{}:{}", slug(from_id), slug(to_id)))
}

pub(super) fn id(value: impl Into<String>) -> RuntimeResult<Id> {
    Ok(Id::new(value)?)
}

pub(super) fn slug(id: &Id) -> String {
    id.as_str()
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_owned()
}

pub(super) fn file_label(path: &str) -> String {
    path.rsplit('/')
        .next()
        .filter(|name| !name.is_empty())
        .unwrap_or(path)
        .to_owned()
}

pub(super) fn obstruction_slug(obstruction_type: TestGapObstructionType) -> &'static str {
    match obstruction_type {
        TestGapObstructionType::MissingRequirementVerification => {
            "missing-requirement-verification"
        }
        TestGapObstructionType::MissingPublicBehaviorUnitTest => "missing-public-unit-test",
        TestGapObstructionType::MissingBranchUnitTest => "missing-branch-unit-test",
        TestGapObstructionType::MissingBoundaryCaseUnitTest => "missing-boundary-unit-test",
        TestGapObstructionType::MissingErrorCaseUnitTest => "missing-error-unit-test",
        TestGapObstructionType::MissingRegressionTest => "missing-regression-test",
        TestGapObstructionType::StaleOrMismatchedTestMapping => "stale-test-mapping",
        TestGapObstructionType::InsufficientTestEvidence => "insufficient-test-evidence",
        TestGapObstructionType::ProjectionInformationLossMissing => "projection-loss-missing",
    }
}

pub(super) fn serde_plain_context_type(
    context_type: crate::test_gap_reports::TestGapContextType,
) -> String {
    serde_json::to_value(context_type)
        .ok()
        .and_then(|value| value.as_str().map(ToOwned::to_owned))
        .unwrap_or_else(|| "custom".to_owned())
}

pub(super) fn serde_plain_dependency_relation_type(
    relation_type: crate::test_gap_reports::TestGapDependencyRelationType,
) -> String {
    serde_json::to_value(relation_type)
        .ok()
        .and_then(|value| value.as_str().map(ToOwned::to_owned))
        .unwrap_or_else(|| "custom".to_owned())
}

pub(super) fn push_unique(ids: &mut Vec<Id>, id: Id) {
    if !ids.contains(&id) {
        ids.push(id);
    }
}

pub(super) fn push_unique_string(values: &mut Vec<String>, value: String) {
    if !values.contains(&value) {
        values.push(value);
    }
}

pub(super) fn validation_error(reason: impl Into<String>) -> RuntimeError {
    RuntimeError::workflow_construction(WORKFLOW_NAME, reason)
}
