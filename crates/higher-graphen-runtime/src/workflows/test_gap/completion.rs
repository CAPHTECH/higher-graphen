use super::*;

pub(super) fn completion_candidates(
    input: &TestGapInputDocument,
    obstructions: &[TestGapObstruction],
) -> RuntimeResult<Vec<TestGapCompletionCandidate>> {
    obstructions
        .iter()
        .filter(|obstruction| {
            !matches!(
                obstruction.obstruction_type,
                TestGapObstructionType::ProjectionInformationLossMissing
                    | TestGapObstructionType::InsufficientTestEvidence
                    | TestGapObstructionType::StaleOrMismatchedTestMapping
            )
        })
        .map(|obstruction| completion_candidate(input, obstruction))
        .collect()
}

pub(super) fn completion_candidate(
    input: &TestGapInputDocument,
    obstruction: &TestGapObstruction,
) -> RuntimeResult<TestGapCompletionCandidate> {
    let primary_target = obstruction
        .target_ids
        .last()
        .cloned()
        .unwrap_or_else(|| obstruction.id.clone());
    let target_label = target_label(input, &primary_target);
    let test_name = suggested_test_name(obstruction, &target_label);
    let source_ids = obstruction_source_ids(obstruction);
    Ok(TestGapCompletionCandidate {
        id: id(format!(
            "candidate:test-gap:{}:{}",
            missing_candidate_slug(obstruction.obstruction_type),
            slug(&primary_target)
        ))?,
        candidate_type: "missing_test".to_owned(),
        missing_type: TestGapMissingType::UnitTest,
        target_ids: obstruction.target_ids.clone(),
        obstruction_ids: vec![obstruction.id.clone()],
        suggested_test_shape: TestGapSuggestedTestShape {
            test_name,
            test_kind: preferred_test_kind(input),
            setup: format!("Construct the minimal accepted-test inputs for {target_label}."),
            inputs: suggested_inputs(obstruction),
            expected_behavior: suggested_expected_behavior(obstruction),
            assertions: vec![suggested_assertion(obstruction)],
            fixture_notes: Some(
                "Use only bounded snapshot facts as accepted evidence; candidate details remain unreviewed until explicit review."
                    .to_owned(),
            ),
        },
        rationale: format!(
            "The bounded structure violates {:?}; an accepted test linked to the witness would close this gap.",
            obstruction.obstruction_type
        ),
        provenance: TestGapCandidateProvenance {
            source_ids,
            extraction_method: EXTRACTION_METHOD.to_owned(),
        },
        severity: obstruction.severity,
        confidence: obstruction.confidence,
        review_status: ReviewStatus::Unreviewed,
    })
}

pub(super) fn accepts_test_kind(input: &TestGapInputDocument, test_type: TestGapTestType) -> bool {
    accepted_test_kinds(input).contains(&test_type)
}

pub(super) fn accepted_test_kinds(input: &TestGapInputDocument) -> BTreeSet<TestGapTestType> {
    input
        .detector_context
        .as_ref()
        .map(|context| context.test_kinds.clone())
        .filter(|test_kinds| !test_kinds.is_empty())
        .unwrap_or_else(|| vec![TestGapTestType::Unit])
        .into_iter()
        .collect()
}

pub(super) fn accepted_test_kind_names(input: &TestGapInputDocument) -> Vec<String> {
    accepted_test_kinds(input)
        .into_iter()
        .filter_map(|test_kind| {
            serde_json::to_value(test_kind)
                .ok()
                .and_then(|value| value.as_str().map(ToOwned::to_owned))
        })
        .collect()
}

pub(super) fn preferred_test_kind(input: &TestGapInputDocument) -> TestGapTestType {
    input
        .detector_context
        .as_ref()
        .and_then(|context| context.test_kinds.first().copied())
        .unwrap_or(TestGapTestType::Unit)
}

pub(super) fn suggested_test_name(obstruction: &TestGapObstruction, target_label: &str) -> String {
    let target = target_label
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_owned();
    match obstruction.obstruction_type {
        TestGapObstructionType::MissingRegressionTest => {
            format!("regresses_{target}_bug_fix")
        }
        TestGapObstructionType::MissingBoundaryCaseUnitTest => {
            format!("covers_{target}_boundary_case")
        }
        TestGapObstructionType::MissingErrorCaseUnitTest => {
            format!("covers_{target}_error_case")
        }
        _ => format!("covers_{target}_unit_behavior"),
    }
}

pub(super) fn suggested_inputs(obstruction: &TestGapObstruction) -> Value {
    match obstruction.obstruction_type {
        TestGapObstructionType::MissingBoundaryCaseUnitTest => obstruction
            .witness
            .get("representative_value")
            .cloned()
            .unwrap_or_else(|| json!({"boundary": "representative boundary value"})),
        TestGapObstructionType::MissingRegressionTest => {
            json!({"regression_case": "minimal bug-fix reproduction"})
        }
        TestGapObstructionType::MissingErrorCaseUnitTest => {
            json!({"trigger": "representative error trigger"})
        }
        _ => json!({"case": "representative unit behavior"}),
    }
}

pub(super) fn suggested_expected_behavior(obstruction: &TestGapObstruction) -> String {
    match obstruction.obstruction_type {
        TestGapObstructionType::MissingBoundaryCaseUnitTest => {
            "The boundary condition preserves the declared behavior.".to_owned()
        }
        TestGapObstructionType::MissingRegressionTest => {
            "The bug-fix behavior fails before the change and passes after it.".to_owned()
        }
        TestGapObstructionType::MissingErrorCaseUnitTest => {
            "The unit returns or raises the expected error behavior.".to_owned()
        }
        _ => "The changed unit behavior is asserted directly.".to_owned(),
    }
}

pub(super) fn suggested_assertion(obstruction: &TestGapObstruction) -> String {
    match obstruction.obstruction_type {
        TestGapObstructionType::MissingBoundaryCaseUnitTest => {
            "assert the boundary output matches the expected behavior".to_owned()
        }
        TestGapObstructionType::MissingRegressionTest => {
            "assert the regression expectation is satisfied after the fix".to_owned()
        }
        TestGapObstructionType::MissingErrorCaseUnitTest => {
            "assert the expected error path result".to_owned()
        }
        _ => "assert the unit-level observable result".to_owned(),
    }
}

pub(super) fn target_label(input: &TestGapInputDocument, target_id: &Id) -> String {
    input
        .symbols
        .iter()
        .find(|symbol| &symbol.id == target_id)
        .map(|symbol| symbol.name.clone())
        .or_else(|| {
            input
                .branches
                .iter()
                .find(|branch| &branch.id == target_id)
                .map(|branch| branch.summary.clone())
        })
        .or_else(|| {
            input
                .requirements
                .iter()
                .find(|requirement| &requirement.id == target_id)
                .map(|requirement| requirement.summary.clone())
        })
        .or_else(|| {
            input
                .laws
                .iter()
                .find(|law| &law.id == target_id)
                .map(|law| law.summary.clone())
        })
        .or_else(|| {
            input
                .morphisms
                .iter()
                .find(|morphism| &morphism.id == target_id)
                .map(|morphism| morphism.morphism_type.clone())
        })
        .or_else(|| {
            input
                .higher_order_cells
                .iter()
                .find(|cell| &cell.id == target_id)
                .map(|cell| cell.label.clone())
        })
        .unwrap_or_else(|| target_id.to_string())
}

pub(super) fn missing_candidate_slug(obstruction_type: TestGapObstructionType) -> &'static str {
    match obstruction_type {
        TestGapObstructionType::MissingBoundaryCaseUnitTest => "boundary-unit-test",
        TestGapObstructionType::MissingBranchUnitTest => "branch-unit-test",
        TestGapObstructionType::MissingErrorCaseUnitTest => "error-unit-test",
        TestGapObstructionType::MissingRegressionTest => "regression-unit-test",
        TestGapObstructionType::MissingPublicBehaviorUnitTest => "public-behavior-unit-test",
        TestGapObstructionType::MissingRequirementVerification => "requirement-unit-test",
        _ => "unit-test",
    }
}
