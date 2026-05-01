//! Bounded missing unit test detector workflow.

use crate::error::{RuntimeError, RuntimeResult};
use crate::reports::{
    AiProjectionRecord, AiProjectionRecordType, AuditProjectionView, HumanReviewProjectionView,
    ProjectionAudience, ProjectionPurpose, ProjectionTrace, ProjectionViewSet, ReportEnvelope,
    ReportMetadata,
};
use crate::test_gap_reports::{
    TestGapBranchType, TestGapCandidateProvenance, TestGapCompletionCandidate,
    TestGapCounterexample, TestGapFactSource, TestGapInputBranch, TestGapInputDocument,
    TestGapInputRequirement, TestGapInputSymbol, TestGapLiftedCell, TestGapLiftedContext,
    TestGapLiftedIncidence, TestGapLiftedSpace, TestGapLiftedStructure, TestGapMissingType,
    TestGapMorphismSummary, TestGapMorphismType, TestGapObservedBranch, TestGapObservedChangedFile,
    TestGapObservedContext, TestGapObservedCoverage, TestGapObservedDependencyEdge,
    TestGapObservedEvidence, TestGapObservedHigherOrderCell, TestGapObservedHigherOrderIncidence,
    TestGapObservedInputLaw, TestGapObservedInputMorphism, TestGapObservedRequirement,
    TestGapObservedRiskSignal, TestGapObservedSymbol, TestGapObservedTest,
    TestGapObservedVerificationCell, TestGapObstruction, TestGapObstructionType,
    TestGapPreservationStatus, TestGapProofObject, TestGapReport, TestGapResult, TestGapScenario,
    TestGapSourceBoundary, TestGapStructuralSummary, TestGapSuggestedTestShape, TestGapTestType,
};
use higher_graphen_core::{Confidence, Id, Provenance, ReviewStatus, Severity, SourceRef};
use higher_graphen_projection::InformationLoss;
use higher_graphen_structure::space::IncidenceOrientation;
use serde_json::{json, Value};
use std::collections::BTreeSet;

mod boundary;
mod completion;
mod detection;
mod lift;
mod morphology;
mod projection;
mod scenario;
mod support;
mod validation;

use boundary::*;
use completion::*;
use detection::*;
use lift::*;
use morphology::*;
use projection::*;
use scenario::*;
use support::*;
use validation::*;

pub(super) const WORKFLOW_NAME: &str = "test_gap";
pub(super) const INPUT_SCHEMA: &str = "highergraphen.test_gap.input.v1";
const REPORT_SCHEMA: &str = "highergraphen.test_gap.report.v1";
const REPORT_TYPE: &str = "test_gap";
const REPORT_VERSION: u32 = 1;
const EXTRACTION_METHOD: &str = "test_gap_detect.v1";

const INVARIANT_REQUIREMENT_VERIFIED: &str = "invariant:test-gap:requirement-verified";
const INVARIANT_PUBLIC_BEHAVIOR_COVERED: &str = "invariant:test-gap:public-behavior-covered";
const INVARIANT_BOUNDARY_CASES_REPRESENTED: &str = "invariant:test-gap:boundary-cases-represented";
const INVARIANT_ERROR_CASES_REPRESENTED: &str = "invariant:test-gap:error-cases-represented";
const INVARIANT_REGRESSION_TEST_FOR_BUG_FIX: &str =
    "invariant:test-gap:regression-test-for-bug-fix";
const INVARIANT_PROJECTION_INFORMATION_LOSS: &str =
    "invariant:test-gap:projection-declares-information-loss";

/// Runs the bounded missing unit test detector workflow.
pub fn run_test_gap_detect(input: TestGapInputDocument) -> RuntimeResult<TestGapReport> {
    validate_input_schema(&input)?;
    validate_input_references(&input)?;

    let lifted_structure = lift_input(&input)?;
    let accepted_fact_ids = accepted_fact_ids(&input);
    let evaluated_invariant_ids = evaluated_invariant_ids()?;
    let mut obstructions = detect_obstructions(&input)?;
    let initial_completion_candidates = completion_candidates(&input, &obstructions)?;
    ensure_detector_output_unreviewed(&obstructions, &initial_completion_candidates)?;

    let scenario = report_scenario(&input, lifted_structure);
    append_projection_information_loss_if_missing(
        &scenario,
        &accepted_fact_ids,
        &evaluated_invariant_ids,
        &initial_completion_candidates,
        &mut obstructions,
    )?;

    let completion_candidates = completion_candidates(&input, &obstructions)?;
    ensure_detector_output_unreviewed(&obstructions, &completion_candidates)?;
    let result = build_result(
        &input,
        accepted_fact_ids,
        evaluated_invariant_ids,
        obstructions,
        completion_candidates,
    )?;
    let projection = report_projection(&scenario, &result, &result.completion_candidates)?;

    Ok(ReportEnvelope {
        schema: REPORT_SCHEMA.to_owned(),
        report_type: REPORT_TYPE.to_owned(),
        report_version: REPORT_VERSION,
        metadata: ReportMetadata::test_gap_detection(),
        scenario,
        result,
        projection,
    })
}

fn append_projection_information_loss_if_missing(
    scenario: &TestGapScenario,
    accepted_fact_ids: &[Id],
    evaluated_invariant_ids: &[Id],
    initial_completion_candidates: &[TestGapCompletionCandidate],
    obstructions: &mut Vec<TestGapObstruction>,
) -> RuntimeResult<()> {
    let source_ids = initial_source_ids(
        accepted_fact_ids,
        evaluated_invariant_ids,
        obstructions,
        initial_completion_candidates,
    );
    let projection = report_projection(
        scenario,
        &initial_result(
            accepted_fact_ids,
            evaluated_invariant_ids,
            obstructions,
            source_ids.clone(),
        ),
        &[],
    )?;
    if projection_declares_information_loss(&projection) {
        return Ok(());
    }
    obstructions.push(projection_information_loss_obstruction(source_ids)?);
    Ok(())
}

fn initial_source_ids(
    accepted_fact_ids: &[Id],
    evaluated_invariant_ids: &[Id],
    obstructions: &[TestGapObstruction],
    initial_completion_candidates: &[TestGapCompletionCandidate],
) -> Vec<Id> {
    let source_ids = result_source_ids(
        accepted_fact_ids,
        evaluated_invariant_ids,
        &[],
        &[],
        obstructions,
        initial_completion_candidates,
    );
    if source_ids.is_empty() {
        accepted_fact_ids.to_vec()
    } else {
        source_ids
    }
}

fn initial_result(
    accepted_fact_ids: &[Id],
    evaluated_invariant_ids: &[Id],
    obstructions: &[TestGapObstruction],
    source_ids: Vec<Id>,
) -> TestGapResult {
    TestGapResult {
        status: result_status(obstructions),
        accepted_fact_ids: accepted_fact_ids.to_vec(),
        evaluated_invariant_ids: evaluated_invariant_ids.to_vec(),
        morphism_summaries: Vec::new(),
        proof_objects: Vec::new(),
        counterexamples: Vec::new(),
        obstructions: Vec::new(),
        completion_candidates: Vec::new(),
        source_ids,
    }
}

fn projection_declares_information_loss(projection: &ProjectionViewSet) -> bool {
    !projection.human_review.information_loss.is_empty()
        && !projection.ai_view.information_loss.is_empty()
        && !projection.audit_trace.information_loss.is_empty()
}

fn projection_information_loss_obstruction(
    source_ids: Vec<Id>,
) -> RuntimeResult<TestGapObstruction> {
    Ok(TestGapObstruction {
        id: id("obstruction:test-gap:projection-information-loss")?,
        obstruction_type: TestGapObstructionType::ProjectionInformationLossMissing,
        title: "Projection information loss is missing".to_owned(),
        target_ids: vec![id("projection:test-gap")?],
        witness: json!({
            "projection_id": "projection:test-gap",
            "affected_source_ids": source_ids,
        }),
        invariant_ids: vec![id(INVARIANT_PROJECTION_INFORMATION_LOSS)?],
        evidence_ids: Vec::new(),
        severity: Severity::Medium,
        confidence: Confidence::new(0.95)?,
        review_status: ReviewStatus::Unreviewed,
    })
}

fn build_result(
    input: &TestGapInputDocument,
    accepted_fact_ids: Vec<Id>,
    evaluated_invariant_ids: Vec<Id>,
    obstructions: Vec<TestGapObstruction>,
    completion_candidates: Vec<TestGapCompletionCandidate>,
) -> RuntimeResult<TestGapResult> {
    let proof_objects = proof_objects(input)?;
    let counterexamples = counterexamples(input)?;
    let source_ids = result_source_ids(
        &accepted_fact_ids,
        &evaluated_invariant_ids,
        &proof_objects,
        &counterexamples,
        &obstructions,
        &completion_candidates,
    );
    Ok(TestGapResult {
        status: result_status(&obstructions),
        accepted_fact_ids,
        evaluated_invariant_ids,
        morphism_summaries: morphism_summaries(input, &completion_candidates)?,
        proof_objects,
        counterexamples,
        obstructions,
        completion_candidates,
        source_ids,
    })
}

fn result_status(obstructions: &[TestGapObstruction]) -> crate::test_gap_reports::TestGapStatus {
    if obstructions.is_empty() {
        crate::test_gap_reports::TestGapStatus::NoGapsInSnapshot
    } else {
        crate::test_gap_reports::TestGapStatus::GapsDetected
    }
}
