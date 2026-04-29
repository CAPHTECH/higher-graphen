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
use higher_graphen_space::IncidenceOrientation;
use serde_json::{json, Value};
use std::collections::BTreeSet;

const WORKFLOW_NAME: &str = "test_gap";
const INPUT_SCHEMA: &str = "highergraphen.test_gap.input.v1";
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

    let mut source_ids = result_source_ids(
        &accepted_fact_ids,
        &evaluated_invariant_ids,
        &[],
        &[],
        &obstructions,
        &initial_completion_candidates,
    );
    if source_ids.is_empty() {
        source_ids = accepted_fact_ids.clone();
    }

    let status = if obstructions.is_empty() {
        crate::test_gap_reports::TestGapStatus::NoGapsInSnapshot
    } else {
        crate::test_gap_reports::TestGapStatus::GapsDetected
    };

    let scenario = report_scenario(&input, lifted_structure);
    let projection = report_projection(
        &scenario,
        &TestGapResult {
            status,
            accepted_fact_ids: accepted_fact_ids.clone(),
            evaluated_invariant_ids: evaluated_invariant_ids.clone(),
            morphism_summaries: Vec::new(),
            proof_objects: Vec::new(),
            counterexamples: Vec::new(),
            obstructions: Vec::new(),
            completion_candidates: Vec::new(),
            source_ids: source_ids.clone(),
        },
        &[],
    )?;

    if projection.human_review.information_loss.is_empty()
        || projection.ai_view.information_loss.is_empty()
        || projection.audit_trace.information_loss.is_empty()
    {
        obstructions.push(TestGapObstruction {
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
        });
    }

    let completion_candidates = completion_candidates(&input, &obstructions)?;
    ensure_detector_output_unreviewed(&obstructions, &completion_candidates)?;
    let proof_objects = proof_objects(&input)?;
    let counterexamples = counterexamples(&input)?;
    let source_ids = result_source_ids(
        &accepted_fact_ids,
        &evaluated_invariant_ids,
        &proof_objects,
        &counterexamples,
        &obstructions,
        &completion_candidates,
    );
    let morphism_summaries = morphism_summaries(&input, &completion_candidates)?;
    let status = if obstructions.is_empty() {
        crate::test_gap_reports::TestGapStatus::NoGapsInSnapshot
    } else {
        crate::test_gap_reports::TestGapStatus::GapsDetected
    };
    let result = TestGapResult {
        status,
        accepted_fact_ids,
        evaluated_invariant_ids,
        morphism_summaries,
        proof_objects,
        counterexamples,
        obstructions,
        completion_candidates,
        source_ids,
    };
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

fn validate_input_schema(input: &TestGapInputDocument) -> RuntimeResult<()> {
    if input.schema == INPUT_SCHEMA {
        return Ok(());
    }
    Err(RuntimeError::unsupported_input_schema(
        input.schema.clone(),
        INPUT_SCHEMA,
    ))
}

fn validate_input_references(input: &TestGapInputDocument) -> RuntimeResult<()> {
    if input.changed_files.is_empty() {
        return Err(validation_error(
            "changed_files must contain at least one file",
        ));
    }
    ensure_unique_input_ids(input)?;
    let ids = ReferenceIds::from_input(input);
    for file in &input.changed_files {
        ensure_known_ids(
            &ids.symbol_ids,
            &file.symbol_ids,
            "changed_file",
            &file.id,
            "symbol",
        )?;
        ensure_known_ids(
            &ids.context_ids,
            &file.context_ids,
            "changed_file",
            &file.id,
            "context",
        )?;
        ensure_known_ids(
            &ids.accepted_ids,
            &file.source_ids,
            "changed_file",
            &file.id,
            "source",
        )?;
    }
    for symbol in &input.symbols {
        ensure_known_id(&ids.file_ids, &symbol.file_id, "symbol", &symbol.id, "file")?;
        ensure_known_ids(
            &ids.branch_ids,
            &symbol.branch_ids,
            "symbol",
            &symbol.id,
            "branch",
        )?;
        ensure_known_ids(
            &ids.requirement_ids,
            &symbol.requirement_ids,
            "symbol",
            &symbol.id,
            "requirement",
        )?;
        ensure_known_ids(
            &ids.context_ids,
            &symbol.context_ids,
            "symbol",
            &symbol.id,
            "context",
        )?;
        ensure_known_ids(
            &ids.accepted_ids,
            &symbol.source_ids,
            "symbol",
            &symbol.id,
            "source",
        )?;
    }
    for branch in &input.branches {
        ensure_known_id(
            &ids.symbol_ids,
            &branch.symbol_id,
            "branch",
            &branch.id,
            "symbol",
        )?;
        ensure_known_ids(
            &ids.requirement_ids,
            &branch.requirement_ids,
            "branch",
            &branch.id,
            "requirement",
        )?;
        ensure_known_ids(
            &ids.accepted_ids,
            &branch.source_ids,
            "branch",
            &branch.id,
            "source",
        )?;
    }
    for requirement in &input.requirements {
        ensure_known_ids(
            &ids.implementation_ids,
            &requirement.implementation_ids,
            "requirement",
            &requirement.id,
            "implementation",
        )?;
        ensure_known_ids(
            &ids.accepted_ids,
            &requirement.source_ids,
            "requirement",
            &requirement.id,
            "source",
        )?;
    }
    for test in &input.tests {
        if let Some(file_id) = &test.file_id {
            ensure_known_id(&ids.file_ids, file_id, "test", &test.id, "file")?;
        }
        ensure_known_ids(
            &ids.implementation_ids,
            &test.target_ids,
            "test",
            &test.id,
            "target",
        )?;
        ensure_known_ids(
            &ids.branch_ids,
            &test.branch_ids,
            "test",
            &test.id,
            "branch",
        )?;
        ensure_known_ids(
            &ids.requirement_ids,
            &test.requirement_ids,
            "test",
            &test.id,
            "requirement",
        )?;
        ensure_known_ids(
            &ids.context_ids,
            &test.context_ids,
            "test",
            &test.id,
            "context",
        )?;
        ensure_known_ids(
            &ids.accepted_ids,
            &test.source_ids,
            "test",
            &test.id,
            "source",
        )?;
    }
    for coverage in &input.coverage {
        ensure_known_id(
            &ids.coverage_target_ids,
            &coverage.target_id,
            "coverage",
            &coverage.id,
            "target",
        )?;
        ensure_known_ids(
            &ids.test_ids,
            &coverage.covered_by_test_ids,
            "coverage",
            &coverage.id,
            "test",
        )?;
        ensure_known_ids(
            &ids.accepted_ids,
            &coverage.source_ids,
            "coverage",
            &coverage.id,
            "source",
        )?;
    }
    for edge in &input.dependency_edges {
        ensure_known_id(
            &ids.accepted_ids,
            &edge.from_id,
            "dependency_edge",
            &edge.id,
            "from endpoint",
        )?;
        ensure_known_id(
            &ids.accepted_ids,
            &edge.to_id,
            "dependency_edge",
            &edge.id,
            "to endpoint",
        )?;
        ensure_known_ids(
            &ids.accepted_ids,
            &edge.source_ids,
            "dependency_edge",
            &edge.id,
            "source",
        )?;
    }
    for cell in &input.higher_order_cells {
        ensure_known_ids(
            &ids.context_ids,
            &cell.context_ids,
            "higher_order_cell",
            &cell.id,
            "context",
        )?;
        ensure_known_ids(
            &ids.accepted_ids,
            &cell.source_ids,
            "higher_order_cell",
            &cell.id,
            "source",
        )?;
    }
    for incidence in &input.higher_order_incidences {
        ensure_known_id(
            &ids.high_order_endpoint_ids,
            &incidence.from_id,
            "higher_order_incidence",
            &incidence.id,
            "from endpoint",
        )?;
        ensure_known_id(
            &ids.high_order_endpoint_ids,
            &incidence.to_id,
            "higher_order_incidence",
            &incidence.id,
            "to endpoint",
        )?;
        ensure_known_ids(
            &ids.accepted_ids,
            &incidence.source_ids,
            "higher_order_incidence",
            &incidence.id,
            "source",
        )?;
    }
    for morphism in &input.morphisms {
        ensure_known_ids(
            &ids.high_order_endpoint_ids,
            &morphism.source_ids,
            "morphism",
            &morphism.id,
            "source endpoint",
        )?;
        ensure_known_ids(
            &ids.high_order_endpoint_ids,
            &morphism.target_ids,
            "morphism",
            &morphism.id,
            "target endpoint",
        )?;
        ensure_known_ids(
            &ids.law_ids,
            &morphism.law_ids,
            "morphism",
            &morphism.id,
            "law",
        )?;
        ensure_known_ids(
            &ids.requirement_ids,
            &morphism.requirement_ids,
            "morphism",
            &morphism.id,
            "requirement",
        )?;
    }
    for law in &input.laws {
        ensure_known_ids(
            &ids.high_order_endpoint_ids,
            &law.applies_to_ids,
            "law",
            &law.id,
            "applies-to endpoint",
        )?;
        ensure_known_ids(
            &ids.requirement_ids,
            &law.requirement_ids,
            "law",
            &law.id,
            "requirement",
        )?;
        ensure_known_ids(&ids.accepted_ids, &law.source_ids, "law", &law.id, "source")?;
    }
    for verification in &input.verification_cells {
        ensure_known_ids(
            &ids.high_order_endpoint_ids,
            &verification.target_ids,
            "verification_cell",
            &verification.id,
            "target",
        )?;
        ensure_known_ids(
            &ids.requirement_ids,
            &verification.requirement_ids,
            "verification_cell",
            &verification.id,
            "requirement",
        )?;
        ensure_known_ids(
            &ids.law_ids,
            &verification.law_ids,
            "verification_cell",
            &verification.id,
            "law",
        )?;
        ensure_known_ids(
            &ids.morphism_ids,
            &verification.morphism_ids,
            "verification_cell",
            &verification.id,
            "morphism",
        )?;
        ensure_known_ids(
            &ids.accepted_ids,
            &verification.source_ids,
            "verification_cell",
            &verification.id,
            "source",
        )?;
    }
    for context in &input.contexts {
        ensure_known_ids(
            &ids.accepted_ids,
            &context.source_ids,
            "context",
            &context.id,
            "source",
        )?;
    }
    for evidence in &input.evidence {
        ensure_known_ids(
            &ids.accepted_ids,
            &evidence.source_ids,
            "evidence",
            &evidence.id,
            "source",
        )?;
    }
    for signal in &input.signals {
        ensure_known_ids(
            &ids.accepted_ids,
            &signal.source_ids,
            "signal",
            &signal.id,
            "source",
        )?;
    }
    Ok(())
}

struct ReferenceIds {
    file_ids: Vec<Id>,
    symbol_ids: Vec<Id>,
    branch_ids: Vec<Id>,
    requirement_ids: Vec<Id>,
    test_ids: Vec<Id>,
    law_ids: Vec<Id>,
    morphism_ids: Vec<Id>,
    context_ids: Vec<Id>,
    implementation_ids: Vec<Id>,
    coverage_target_ids: Vec<Id>,
    high_order_endpoint_ids: Vec<Id>,
    accepted_ids: Vec<Id>,
}

impl ReferenceIds {
    fn from_input(input: &TestGapInputDocument) -> Self {
        let file_ids = input
            .changed_files
            .iter()
            .map(|file| file.id.clone())
            .collect();
        let symbol_ids = input
            .symbols
            .iter()
            .map(|symbol| symbol.id.clone())
            .collect();
        let branch_ids = input
            .branches
            .iter()
            .map(|branch| branch.id.clone())
            .collect();
        let requirement_ids = input
            .requirements
            .iter()
            .map(|requirement| requirement.id.clone())
            .collect();
        let test_ids = input.tests.iter().map(|test| test.id.clone()).collect();
        let law_ids = input.laws.iter().map(|law| law.id.clone()).collect();
        let morphism_ids = input
            .morphisms
            .iter()
            .map(|morphism| morphism.id.clone())
            .collect();
        let context_ids = input
            .contexts
            .iter()
            .map(|context| context.id.clone())
            .collect();
        let mut implementation_ids = Vec::new();
        implementation_ids.extend(input.changed_files.iter().map(|file| file.id.clone()));
        implementation_ids.extend(input.symbols.iter().map(|symbol| symbol.id.clone()));
        implementation_ids.extend(input.higher_order_cells.iter().map(|cell| cell.id.clone()));
        implementation_ids.extend(input.laws.iter().map(|law| law.id.clone()));
        implementation_ids.extend(input.morphisms.iter().map(|morphism| morphism.id.clone()));
        let mut coverage_target_ids = implementation_ids.clone();
        coverage_target_ids.extend(input.branches.iter().map(|branch| branch.id.clone()));
        coverage_target_ids.extend(
            input
                .requirements
                .iter()
                .map(|requirement| requirement.id.clone()),
        );
        let mut high_order_endpoint_ids = accepted_fact_ids(input);
        high_order_endpoint_ids.extend(input.higher_order_cells.iter().map(|cell| cell.id.clone()));
        high_order_endpoint_ids.extend(input.laws.iter().map(|law| law.id.clone()));
        high_order_endpoint_ids.extend(input.morphisms.iter().map(|morphism| morphism.id.clone()));
        Self {
            file_ids,
            symbol_ids,
            branch_ids,
            requirement_ids,
            test_ids,
            law_ids,
            morphism_ids,
            context_ids,
            implementation_ids,
            coverage_target_ids,
            high_order_endpoint_ids,
            accepted_ids: accepted_fact_ids(input),
        }
    }
}

fn lift_input(input: &TestGapInputDocument) -> RuntimeResult<TestGapLiftedStructure> {
    let space_id = space_id(input)?;
    let context_ids = effective_context_ids(input)?;
    let contexts = lifted_contexts(input, &space_id, &context_ids);
    let mut cells = Vec::new();
    append_cells(input, &space_id, &context_ids, &mut cells)?;
    let incidences = lifted_incidences(input, &space_id)?;
    let space = TestGapLiftedSpace {
        id: space_id.clone(),
        name: format!("Test gap space for {}", input.repository.name),
        description: Some(format!(
            "Bounded structural view of {} between {} and {}.",
            input.change_set.boundary, input.change_set.base_ref, input.change_set.head_ref
        )),
        cell_ids: cells.iter().map(|cell| cell.id.clone()).collect(),
        incidence_ids: incidences
            .iter()
            .map(|incidence| incidence.id.clone())
            .collect(),
        context_ids,
    };
    Ok(TestGapLiftedStructure {
        structural_summary: TestGapStructuralSummary {
            accepted_cell_count: cells.len(),
            accepted_incidence_count: incidences.len(),
            context_count: contexts.len(),
            branch_count: input.branches.len(),
            requirement_count: input.requirements.len(),
            test_count: input.tests.len(),
            coverage_record_count: input.coverage.len(),
            higher_order_cell_count: input.higher_order_cells.len(),
            higher_order_incidence_count: input.higher_order_incidences.len(),
            morphism_count: input.morphisms.len(),
            law_count: input.laws.len(),
            verification_cell_count: input.verification_cells.len(),
        },
        space,
        contexts,
        cells,
        incidences,
    })
}

fn lifted_contexts(
    input: &TestGapInputDocument,
    space_id: &Id,
    context_ids: &[Id],
) -> Vec<TestGapLiftedContext> {
    context_ids
        .iter()
        .map(|context_id| {
            if let Some(context) = input
                .contexts
                .iter()
                .find(|context| &context.id == context_id)
            {
                TestGapLiftedContext {
                    id: context.id.clone(),
                    space_id: space_id.clone(),
                    name: context.name.clone(),
                    context_type: serde_plain_context_type(context.context_type),
                    provenance: fact_provenance(input, input.source.confidence, Some("contexts"))
                        .expect("valid context provenance"),
                }
            } else {
                TestGapLiftedContext {
                    id: context_id.clone(),
                    space_id: space_id.clone(),
                    name: input.repository.name.clone(),
                    context_type: "repository".to_owned(),
                    provenance: fact_provenance(input, input.source.confidence, Some("repository"))
                        .expect("valid repository provenance"),
                }
            }
        })
        .collect()
}

fn append_cells(
    input: &TestGapInputDocument,
    space_id: &Id,
    default_context_ids: &[Id],
    cells: &mut Vec<TestGapLiftedCell>,
) -> RuntimeResult<()> {
    for file in &input.changed_files {
        cells.push(lifted_cell(
            input,
            space_id,
            file.id.clone(),
            0,
            "test_gap.changed_file",
            file_label(&file.path),
            contexts_or_default(&file.context_ids, default_context_ids),
            input.source.confidence,
            Some("changed_files"),
        )?);
    }
    for symbol in &input.symbols {
        cells.push(lifted_cell(
            input,
            space_id,
            symbol.id.clone(),
            0,
            "test_gap.symbol",
            symbol.name.clone(),
            contexts_or_default(&symbol.context_ids, default_context_ids),
            input.source.confidence,
            Some("symbols"),
        )?);
    }
    for branch in &input.branches {
        cells.push(lifted_cell(
            input,
            space_id,
            branch.id.clone(),
            0,
            "test_gap.branch",
            branch.summary.clone(),
            default_context_ids.to_vec(),
            input.source.confidence,
            Some("branches"),
        )?);
    }
    for requirement in &input.requirements {
        cells.push(lifted_cell(
            input,
            space_id,
            requirement.id.clone(),
            0,
            "test_gap.requirement",
            requirement.summary.clone(),
            default_context_ids.to_vec(),
            input.source.confidence,
            Some("requirements"),
        )?);
    }
    for test in &input.tests {
        cells.push(lifted_cell(
            input,
            space_id,
            test.id.clone(),
            0,
            "test_gap.test",
            test.name.clone(),
            contexts_or_default(&test.context_ids, default_context_ids),
            input.source.confidence,
            Some("tests"),
        )?);
    }
    for coverage in &input.coverage {
        cells.push(lifted_cell(
            input,
            space_id,
            coverage.id.clone(),
            1,
            "test_gap.coverage",
            coverage
                .summary
                .clone()
                .unwrap_or_else(|| format!("Coverage for {}", coverage.target_id)),
            default_context_ids.to_vec(),
            coverage.confidence.unwrap_or(input.source.confidence),
            Some("coverage"),
        )?);
    }
    for evidence in &input.evidence {
        cells.push(lifted_cell(
            input,
            space_id,
            evidence.id.clone(),
            1,
            "test_gap.evidence",
            evidence.summary.clone(),
            default_context_ids.to_vec(),
            evidence.confidence.unwrap_or(input.source.confidence),
            Some("evidence"),
        )?);
    }
    for signal in &input.signals {
        cells.push(lifted_cell(
            input,
            space_id,
            signal.id.clone(),
            1,
            "test_gap.risk_signal",
            signal.summary.clone(),
            default_context_ids.to_vec(),
            signal.confidence,
            Some("signals"),
        )?);
    }
    for cell in &input.higher_order_cells {
        cells.push(lifted_cell(
            input,
            space_id,
            cell.id.clone(),
            cell.dimension,
            &format!("test_gap.higher_order.{}", cell.cell_type),
            cell.label.clone(),
            contexts_or_default(&cell.context_ids, default_context_ids),
            cell.confidence.unwrap_or(input.source.confidence),
            Some("higher_order_cells"),
        )?);
    }
    for law in &input.laws {
        cells.push(lifted_cell(
            input,
            space_id,
            law.id.clone(),
            1,
            "test_gap.law",
            law.summary.clone(),
            default_context_ids.to_vec(),
            law.confidence.unwrap_or(input.source.confidence),
            Some("laws"),
        )?);
    }
    for morphism in &input.morphisms {
        cells.push(lifted_cell(
            input,
            space_id,
            morphism.id.clone(),
            2,
            "test_gap.morphism",
            morphism.morphism_type.clone(),
            default_context_ids.to_vec(),
            morphism.confidence.unwrap_or(input.source.confidence),
            Some("morphisms"),
        )?);
    }
    for verification in &input.verification_cells {
        cells.push(lifted_cell(
            input,
            space_id,
            verification.id.clone(),
            1,
            &format!("test_gap.verification.{}", verification.verification_type),
            verification.name.clone(),
            default_context_ids.to_vec(),
            verification.confidence.unwrap_or(input.source.confidence),
            Some("verification_cells"),
        )?);
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn lifted_cell(
    input: &TestGapInputDocument,
    space_id: &Id,
    id: Id,
    dimension: u32,
    cell_type: &str,
    label: String,
    context_ids: Vec<Id>,
    confidence: Confidence,
    source_local_id: Option<&str>,
) -> RuntimeResult<TestGapLiftedCell> {
    Ok(TestGapLiftedCell {
        id,
        space_id: space_id.clone(),
        dimension,
        cell_type: cell_type.to_owned(),
        label,
        context_ids,
        provenance: fact_provenance(input, confidence, source_local_id)?,
    })
}

fn lifted_incidences(
    input: &TestGapInputDocument,
    space_id: &Id,
) -> RuntimeResult<Vec<TestGapLiftedIncidence>> {
    let mut incidences = Vec::new();
    for file in &input.changed_files {
        for symbol_id in &file.symbol_ids {
            incidences.push(lifted_incidence(
                input,
                space_id,
                incidence_id("contains-symbol", &file.id, symbol_id)?,
                file.id.clone(),
                symbol_id.clone(),
                "contains_symbol",
                input.source.confidence,
            )?);
        }
    }
    for symbol in &input.symbols {
        for branch_id in &symbol.branch_ids {
            incidences.push(lifted_incidence(
                input,
                space_id,
                incidence_id("has-branch", &symbol.id, branch_id)?,
                symbol.id.clone(),
                branch_id.clone(),
                "has_branch",
                input.source.confidence,
            )?);
        }
        for requirement_id in &symbol.requirement_ids {
            incidences.push(lifted_incidence(
                input,
                space_id,
                incidence_id("implements-requirement", &symbol.id, requirement_id)?,
                symbol.id.clone(),
                requirement_id.clone(),
                "implements_requirement",
                input.source.confidence,
            )?);
        }
    }
    for test in &input.tests {
        for target_id in &test.target_ids {
            incidences.push(lifted_incidence(
                input,
                space_id,
                incidence_id("covered-by-test", target_id, &test.id)?,
                target_id.clone(),
                test.id.clone(),
                "covered_by_test",
                input.source.confidence,
            )?);
        }
        for branch_id in &test.branch_ids {
            incidences.push(lifted_incidence(
                input,
                space_id,
                incidence_id("exercises-condition", &test.id, branch_id)?,
                test.id.clone(),
                branch_id.clone(),
                "exercises_condition",
                input.source.confidence,
            )?);
        }
        for requirement_id in &test.requirement_ids {
            incidences.push(lifted_incidence(
                input,
                space_id,
                incidence_id("verifies-requirement", &test.id, requirement_id)?,
                test.id.clone(),
                requirement_id.clone(),
                "verifies_requirement",
                input.source.confidence,
            )?);
        }
    }
    for coverage in &input.coverage {
        incidences.push(lifted_incidence(
            input,
            space_id,
            incidence_id("coverage-supports", &coverage.id, &coverage.target_id)?,
            coverage.id.clone(),
            coverage.target_id.clone(),
            "supports",
            coverage.confidence.unwrap_or(input.source.confidence),
        )?);
    }
    for edge in &input.dependency_edges {
        incidences.push(TestGapLiftedIncidence {
            id: edge.id.clone(),
            space_id: space_id.clone(),
            from_cell_id: edge.from_id.clone(),
            to_cell_id: edge.to_id.clone(),
            relation_type: serde_plain_dependency_relation_type(edge.relation_type),
            orientation: edge.orientation.unwrap_or(IncidenceOrientation::Directed),
            weight: None,
            provenance: fact_provenance(
                input,
                edge.confidence.unwrap_or(input.source.confidence),
                Some("dependency_edges"),
            )?,
        });
    }
    for incidence in &input.higher_order_incidences {
        incidences.push(TestGapLiftedIncidence {
            id: incidence.id.clone(),
            space_id: space_id.clone(),
            from_cell_id: incidence.from_id.clone(),
            to_cell_id: incidence.to_id.clone(),
            relation_type: incidence.relation_type.clone(),
            orientation: incidence
                .orientation
                .unwrap_or(IncidenceOrientation::Directed),
            weight: None,
            provenance: fact_provenance(
                input,
                incidence.confidence.unwrap_or(input.source.confidence),
                Some("higher_order_incidences"),
            )?,
        });
    }
    for morphism in &input.morphisms {
        for source_id in &morphism.source_ids {
            incidences.push(lifted_incidence(
                input,
                space_id,
                incidence_id("morphism-source", &morphism.id, source_id)?,
                morphism.id.clone(),
                source_id.clone(),
                "morphism_source",
                morphism.confidence.unwrap_or(input.source.confidence),
            )?);
        }
        for target_id in &morphism.target_ids {
            incidences.push(lifted_incidence(
                input,
                space_id,
                incidence_id("morphism-target", &morphism.id, target_id)?,
                morphism.id.clone(),
                target_id.clone(),
                "morphism_target",
                morphism.confidence.unwrap_or(input.source.confidence),
            )?);
        }
        for law_id in &morphism.law_ids {
            incidences.push(lifted_incidence(
                input,
                space_id,
                incidence_id("morphism-preserves-law", &morphism.id, law_id)?,
                morphism.id.clone(),
                law_id.clone(),
                "preserves_law",
                morphism.confidence.unwrap_or(input.source.confidence),
            )?);
        }
    }
    for law in &input.laws {
        for applies_to_id in &law.applies_to_ids {
            incidences.push(lifted_incidence(
                input,
                space_id,
                incidence_id("law-applies-to", &law.id, applies_to_id)?,
                law.id.clone(),
                applies_to_id.clone(),
                "applies_to",
                law.confidence.unwrap_or(input.source.confidence),
            )?);
        }
    }
    for verification in &input.verification_cells {
        for law_id in &verification.law_ids {
            incidences.push(lifted_incidence(
                input,
                space_id,
                incidence_id("verification-closes-law", &verification.id, law_id)?,
                verification.id.clone(),
                law_id.clone(),
                "verifies_law",
                verification.confidence.unwrap_or(input.source.confidence),
            )?);
        }
        for morphism_id in &verification.morphism_ids {
            incidences.push(lifted_incidence(
                input,
                space_id,
                incidence_id(
                    "verification-closes-morphism",
                    &verification.id,
                    morphism_id,
                )?,
                verification.id.clone(),
                morphism_id.clone(),
                "verifies_morphism",
                verification.confidence.unwrap_or(input.source.confidence),
            )?);
        }
    }
    Ok(incidences)
}

fn lifted_incidence(
    input: &TestGapInputDocument,
    space_id: &Id,
    id: Id,
    from_cell_id: Id,
    to_cell_id: Id,
    relation_type: &str,
    confidence: Confidence,
) -> RuntimeResult<TestGapLiftedIncidence> {
    Ok(TestGapLiftedIncidence {
        id,
        space_id: space_id.clone(),
        from_cell_id,
        to_cell_id,
        relation_type: relation_type.to_owned(),
        orientation: IncidenceOrientation::Directed,
        weight: None,
        provenance: fact_provenance(input, confidence, Some("lifted_incidences"))?,
    })
}

fn detect_obstructions(input: &TestGapInputDocument) -> RuntimeResult<Vec<TestGapObstruction>> {
    let mut obstructions = Vec::new();
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
    for symbol in &input.symbols {
        if symbol_is_public_behavior(symbol) && !has_accepted_test_for_symbol(input, &symbol.id) {
            obstructions.push(missing_public_behavior_obstruction(input, symbol)?);
        }
    }
    for branch in &input.branches {
        if branch_needs_unit_test(branch) && !has_accepted_test_for_branch(input, &branch.id) {
            obstructions.push(missing_branch_obstruction(input, branch)?);
        }
    }
    for law in &input.laws {
        if law.expected_verification.is_some() && !has_accepted_verification_for_law(input, &law.id)
        {
            obstructions.push(missing_law_obstruction(input, law)?);
        }
    }
    for morphism in &input.morphisms {
        if morphism.expected_verification.is_some()
            && !has_accepted_verification_for_morphism(input, &morphism.id)
        {
            obstructions.push(missing_morphism_obstruction(input, morphism)?);
        }
    }
    Ok(obstructions)
}

fn missing_requirement_obstruction(
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

fn missing_regression_obstruction(
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

fn missing_public_behavior_obstruction(
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

fn missing_branch_obstruction(
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

fn missing_law_obstruction(
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

fn missing_morphism_obstruction(
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

fn completion_candidates(
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

fn completion_candidate(
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

fn morphism_summaries(
    input: &TestGapInputDocument,
    candidates: &[TestGapCompletionCandidate],
) -> RuntimeResult<Vec<TestGapMorphismSummary>> {
    let mut summaries = Vec::new();
    for requirement in &input.requirements {
        let has_impl = !requirement.implementation_ids.is_empty();
        let has_unit_test = has_accepted_test_for_requirement(input, &requirement.id);
        let mut loss = Vec::new();
        if !has_impl {
            loss.push("requirement has no supplied implementation target".to_owned());
        }
        if !has_unit_test {
            loss.push("requirement has no accepted unit-test verification".to_owned());
        }
        summaries.push(TestGapMorphismSummary {
            id: id(format!(
                "morphism:test-gap:requirement-to-implementation:{}",
                slug(&requirement.id)
            ))?,
            morphism_type: TestGapMorphismType::RequirementToImplementation,
            source_ids: vec![requirement.id.clone()],
            target_ids: requirement.implementation_ids.clone(),
            preservation_status: if loss.is_empty() {
                TestGapPreservationStatus::Preserved
            } else if has_impl {
                TestGapPreservationStatus::Partial
            } else {
                TestGapPreservationStatus::Lost
            },
            preserved: if has_impl {
                vec!["requirement identity and implementation target IDs".to_owned()]
            } else {
                Vec::new()
            },
            loss,
            review_status: ReviewStatus::Accepted,
        });
    }
    for symbol in &input.symbols {
        let has_unit_test = has_accepted_test_for_symbol(input, &symbol.id);
        summaries.push(TestGapMorphismSummary {
            id: id(format!(
                "morphism:test-gap:implementation-to-test:{}",
                slug(&symbol.id)
            ))?,
            morphism_type: TestGapMorphismType::ImplementationToTest,
            source_ids: vec![symbol.id.clone()],
            target_ids: related_test_ids_for_symbol(input, &symbol.id),
            preservation_status: if has_unit_test {
                TestGapPreservationStatus::Preserved
            } else {
                TestGapPreservationStatus::Lost
            },
            preserved: if has_unit_test {
                vec!["implementation target has accepted unit-test relation".to_owned()]
            } else {
                Vec::new()
            },
            loss: if has_unit_test {
                Vec::new()
            } else {
                vec!["implementation target has no accepted unit-test relation".to_owned()]
            },
            review_status: ReviewStatus::Accepted,
        });
    }
    for morphism in &input.morphisms {
        let has_verification = has_accepted_verification_for_morphism(input, &morphism.id);
        summaries.push(TestGapMorphismSummary {
            id: morphism.id.clone(),
            morphism_type: TestGapMorphismType::RequirementToImplementation,
            source_ids: morphism.source_ids.clone(),
            target_ids: morphism.target_ids.clone(),
            preservation_status: if has_verification {
                TestGapPreservationStatus::Preserved
            } else {
                TestGapPreservationStatus::Lost
            },
            preserved: if has_verification {
                vec![format!(
                    "native morphism {} has accepted verification cell",
                    morphism.morphism_type
                )]
            } else {
                Vec::new()
            },
            loss: if has_verification {
                Vec::new()
            } else {
                vec![format!(
                    "native morphism {} has no accepted verification cell",
                    morphism.morphism_type
                )]
            },
            review_status: ReviewStatus::Accepted,
        });
    }
    summaries.push(TestGapMorphismSummary {
        id: id(format!(
            "morphism:test-gap:before-to-after:{}",
            slug(&input.change_set.id)
        ))?,
        morphism_type: TestGapMorphismType::BeforeToAfter,
        source_ids: input
            .changed_files
            .iter()
            .map(|file| file.id.clone())
            .collect(),
        target_ids: input
            .symbols
            .iter()
            .map(|symbol| symbol.id.clone())
            .collect(),
        preservation_status: TestGapPreservationStatus::Partial,
        preserved: vec!["changed file and symbol identities from bounded input".to_owned()],
        loss: vec!["raw pre-change and post-change source bodies are omitted".to_owned()],
        review_status: ReviewStatus::Accepted,
    });
    for candidate in candidates {
        summaries.push(TestGapMorphismSummary {
            id: id(format!(
                "morphism:test-gap:candidate-to-accepted-test:{}",
                slug(&candidate.id)
            ))?,
            morphism_type: TestGapMorphismType::CandidateToAcceptedTest,
            source_ids: vec![candidate.id.clone()],
            target_ids: Vec::new(),
            preservation_status: TestGapPreservationStatus::NotEvaluated,
            preserved: Vec::new(),
            loss: vec!["candidate remains unreviewed and has no accepted test mapping".to_owned()],
            review_status: ReviewStatus::Unreviewed,
        });
    }
    Ok(summaries)
}

fn proof_objects(input: &TestGapInputDocument) -> RuntimeResult<Vec<TestGapProofObject>> {
    let mut proofs = Vec::new();
    for law in &input.laws {
        let verified_by_ids = accepted_verification_ids_for_law(input, &law.id);
        if verified_by_ids.is_empty() {
            continue;
        }
        let mut witness_ids = vec![law.id.clone()];
        witness_ids.extend(law.applies_to_ids.iter().cloned());
        proofs.push(TestGapProofObject {
            id: id(format!("proof:test-gap:law:{}", slug(&law.id)))?,
            proof_type: "law_verification".to_owned(),
            law_ids: vec![law.id.clone()],
            morphism_ids: Vec::new(),
            verified_by_ids,
            witness_ids,
            summary: format!("Law {} has accepted verification evidence.", law.summary),
            confidence: law.confidence.unwrap_or(Confidence::new(0.82)?),
            review_status: ReviewStatus::Accepted,
        });
    }
    for morphism in &input.morphisms {
        let verified_by_ids = accepted_verification_ids_for_morphism(input, &morphism.id);
        if verified_by_ids.is_empty() {
            continue;
        }
        let mut witness_ids = vec![morphism.id.clone()];
        witness_ids.extend(morphism.source_ids.iter().cloned());
        witness_ids.extend(morphism.target_ids.iter().cloned());
        witness_ids.extend(morphism.law_ids.iter().cloned());
        proofs.push(TestGapProofObject {
            id: id(format!("proof:test-gap:morphism:{}", slug(&morphism.id)))?,
            proof_type: "morphism_verification".to_owned(),
            law_ids: morphism.law_ids.clone(),
            morphism_ids: vec![morphism.id.clone()],
            verified_by_ids,
            witness_ids,
            summary: format!(
                "Morphism {} has accepted verification evidence.",
                morphism.morphism_type
            ),
            confidence: morphism.confidence.unwrap_or(Confidence::new(0.8)?),
            review_status: ReviewStatus::Accepted,
        });
    }
    Ok(proofs)
}

fn counterexamples(input: &TestGapInputDocument) -> RuntimeResult<Vec<TestGapCounterexample>> {
    let mut counterexamples = Vec::new();
    for law in &input.laws {
        if law.expected_verification.is_some()
            && accepted_verification_ids_for_law(input, &law.id).is_empty()
        {
            let mut path_ids = vec![law.id.clone()];
            path_ids.extend(law.applies_to_ids.iter().cloned());
            counterexamples.push(TestGapCounterexample {
                id: id(format!("counterexample:test-gap:law:{}", slug(&law.id)))?,
                counterexample_type: "missing_law_verification".to_owned(),
                law_ids: vec![law.id.clone()],
                morphism_ids: Vec::new(),
                path_ids,
                summary: format!("Law {} has no accepted verification path.", law.summary),
                confidence: law.confidence.unwrap_or(Confidence::new(0.82)?),
                review_status: ReviewStatus::Unreviewed,
            });
        }
    }
    for morphism in &input.morphisms {
        if morphism.expected_verification.is_some()
            && accepted_verification_ids_for_morphism(input, &morphism.id).is_empty()
        {
            let mut path_ids = vec![morphism.id.clone()];
            path_ids.extend(morphism.source_ids.iter().cloned());
            path_ids.extend(morphism.target_ids.iter().cloned());
            path_ids.extend(morphism.law_ids.iter().cloned());
            counterexamples.push(TestGapCounterexample {
                id: id(format!(
                    "counterexample:test-gap:morphism:{}",
                    slug(&morphism.id)
                ))?,
                counterexample_type: "missing_morphism_verification".to_owned(),
                law_ids: morphism.law_ids.clone(),
                morphism_ids: vec![morphism.id.clone()],
                path_ids,
                summary: format!(
                    "Morphism {} has no accepted verification path.",
                    morphism.morphism_type
                ),
                confidence: morphism.confidence.unwrap_or(Confidence::new(0.8)?),
                review_status: ReviewStatus::Unreviewed,
            });
        }
    }
    Ok(counterexamples)
}

fn report_scenario(
    input: &TestGapInputDocument,
    lifted_structure: TestGapLiftedStructure,
) -> TestGapScenario {
    TestGapScenario {
        input_schema: input.schema.clone(),
        source_boundary: source_boundary(input),
        source: input.source.clone(),
        repository: input.repository.clone(),
        change_set: input.change_set.clone(),
        changed_files: input
            .changed_files
            .iter()
            .cloned()
            .map(|record| TestGapObservedChangedFile {
                record,
                review_status: ReviewStatus::Accepted,
                confidence: input.source.confidence,
            })
            .collect(),
        symbols: input
            .symbols
            .iter()
            .cloned()
            .map(|record| TestGapObservedSymbol {
                record,
                review_status: ReviewStatus::Accepted,
                confidence: input.source.confidence,
            })
            .collect(),
        branches: input
            .branches
            .iter()
            .cloned()
            .map(|record| TestGapObservedBranch {
                record,
                review_status: ReviewStatus::Accepted,
                confidence: input.source.confidence,
            })
            .collect(),
        requirements: input
            .requirements
            .iter()
            .cloned()
            .map(|record| TestGapObservedRequirement {
                record,
                review_status: ReviewStatus::Accepted,
                confidence: input.source.confidence,
            })
            .collect(),
        tests: input
            .tests
            .iter()
            .cloned()
            .map(|record| TestGapObservedTest {
                record,
                review_status: ReviewStatus::Accepted,
                confidence: input.source.confidence,
            })
            .collect(),
        coverage: input
            .coverage
            .iter()
            .cloned()
            .map(|record| TestGapObservedCoverage {
                confidence: record.confidence.unwrap_or(input.source.confidence),
                record,
                review_status: ReviewStatus::Accepted,
            })
            .collect(),
        dependency_edges: input
            .dependency_edges
            .iter()
            .cloned()
            .map(|record| TestGapObservedDependencyEdge {
                confidence: record.confidence.unwrap_or(input.source.confidence),
                record,
                review_status: ReviewStatus::Accepted,
            })
            .collect(),
        higher_order_cells: input
            .higher_order_cells
            .iter()
            .cloned()
            .map(|record| TestGapObservedHigherOrderCell {
                confidence: record.confidence.unwrap_or(input.source.confidence),
                record,
                review_status: ReviewStatus::Accepted,
            })
            .collect(),
        higher_order_incidences: input
            .higher_order_incidences
            .iter()
            .cloned()
            .map(|record| TestGapObservedHigherOrderIncidence {
                confidence: record.confidence.unwrap_or(input.source.confidence),
                record,
                review_status: ReviewStatus::Accepted,
            })
            .collect(),
        morphisms: input
            .morphisms
            .iter()
            .cloned()
            .map(|record| TestGapObservedInputMorphism {
                confidence: record.confidence.unwrap_or(input.source.confidence),
                record,
                review_status: ReviewStatus::Accepted,
            })
            .collect(),
        laws: input
            .laws
            .iter()
            .cloned()
            .map(|record| TestGapObservedInputLaw {
                confidence: record.confidence.unwrap_or(input.source.confidence),
                record,
                review_status: ReviewStatus::Accepted,
            })
            .collect(),
        verification_cells: input
            .verification_cells
            .iter()
            .cloned()
            .map(|record| TestGapObservedVerificationCell {
                confidence: record.confidence.unwrap_or(input.source.confidence),
                record,
                review_status: ReviewStatus::Accepted,
            })
            .collect(),
        contexts: input
            .contexts
            .iter()
            .cloned()
            .map(|record| TestGapObservedContext {
                record,
                review_status: ReviewStatus::Accepted,
                confidence: input.source.confidence,
            })
            .collect(),
        evidence: input
            .evidence
            .iter()
            .cloned()
            .map(|record| TestGapObservedEvidence {
                confidence: record.confidence.unwrap_or(input.source.confidence),
                record,
                review_status: ReviewStatus::Accepted,
            })
            .collect(),
        signals: input
            .signals
            .iter()
            .cloned()
            .map(|record| TestGapObservedRiskSignal {
                record,
                review_status: ReviewStatus::Accepted,
            })
            .collect(),
        detector_context: input.detector_context.clone(),
        lifted_structure,
    }
}

fn report_projection(
    scenario: &TestGapScenario,
    result: &TestGapResult,
    candidates: &[TestGapCompletionCandidate],
) -> RuntimeResult<ProjectionViewSet> {
    let source_ids = if result.source_ids.is_empty() {
        result.accepted_fact_ids.clone()
    } else {
        result.source_ids.clone()
    };
    let human_loss = InformationLoss::declared(
        "Projection summarizes bounded files, symbols, branches, requirements, tests, coverage, obstructions, and candidates without embedding raw source bodies or full diffs.",
        source_ids.clone(),
    )?;
    let ai_loss = InformationLoss::declared(
        "AI view preserves stable IDs, severity, confidence, review status, and source IDs, but candidate suggestions remain unreviewed detector inference.",
        source_ids.clone(),
    )?;
    let audit_loss = InformationLoss::declared(
        "Audit trace records source identifiers, adapter roles, represented views, and review boundary, but unsupported coverage dimensions and full test bodies are omitted.",
        source_ids.clone(),
    )?;
    let human_review = HumanReviewProjectionView {
        audience: ProjectionAudience::Human,
        purpose: ProjectionPurpose::TestGapDetection,
        summary: human_summary(result),
        recommended_actions: recommended_actions(result),
        source_ids: source_ids.clone(),
        information_loss: vec![human_loss],
    };
    let ai_view = crate::reports::AiProjectionView {
        audience: ProjectionAudience::AiAgent,
        purpose: ProjectionPurpose::TestGapDetection,
        records: ai_projection_records(scenario, result, candidates),
        source_ids: source_ids.clone(),
        information_loss: vec![ai_loss],
    };
    let audit_trace = AuditProjectionView {
        audience: ProjectionAudience::Audit,
        purpose: ProjectionPurpose::AuditTrace,
        source_ids,
        information_loss: vec![audit_loss],
        traces: audit_traces(result.source_ids.clone()),
    };
    Ok(ProjectionViewSet {
        audience: human_review.audience,
        purpose: human_review.purpose,
        summary: human_review.summary.clone(),
        recommended_actions: human_review.recommended_actions.clone(),
        information_loss: human_review.information_loss.clone(),
        source_ids: human_review.source_ids.clone(),
        human_review,
        ai_view,
        audit_trace,
    })
}

fn ai_projection_records(
    scenario: &TestGapScenario,
    result: &TestGapResult,
    candidates: &[TestGapCompletionCandidate],
) -> Vec<AiProjectionRecord> {
    let mut records = Vec::new();
    for file in &scenario.changed_files {
        records.push(AiProjectionRecord {
            id: file.record.id.clone(),
            record_type: AiProjectionRecordType::ChangedFile,
            summary: format!("Changed file {}.", file.record.path),
            source_ids: vec![file.record.id.clone()],
            confidence: Some(file.confidence),
            review_status: Some(file.review_status),
            severity: None,
            provenance: None,
        });
    }
    for symbol in &scenario.symbols {
        records.push(AiProjectionRecord {
            id: symbol.record.id.clone(),
            record_type: AiProjectionRecordType::Symbol,
            summary: format!("Changed symbol {}.", symbol.record.name),
            source_ids: vec![symbol.record.id.clone(), symbol.record.file_id.clone()],
            confidence: Some(symbol.confidence),
            review_status: Some(symbol.review_status),
            severity: None,
            provenance: None,
        });
    }
    for branch in &scenario.branches {
        records.push(AiProjectionRecord {
            id: branch.record.id.clone(),
            record_type: AiProjectionRecordType::Cell,
            summary: format!("Changed branch {}.", branch.record.summary),
            source_ids: vec![branch.record.id.clone(), branch.record.symbol_id.clone()],
            confidence: Some(branch.confidence),
            review_status: Some(branch.review_status),
            severity: None,
            provenance: None,
        });
    }
    for test in &scenario.tests {
        records.push(AiProjectionRecord {
            id: test.record.id.clone(),
            record_type: AiProjectionRecordType::Test,
            summary: format!("Existing test {}.", test.record.name),
            source_ids: vec![test.record.id.clone()],
            confidence: Some(test.confidence),
            review_status: Some(test.review_status),
            severity: None,
            provenance: None,
        });
    }
    for cell in &scenario.higher_order_cells {
        records.push(AiProjectionRecord {
            id: cell.record.id.clone(),
            record_type: AiProjectionRecordType::Cell,
            summary: format!(
                "Higher-order {} cell {}.",
                cell.record.cell_type, cell.record.label
            ),
            source_ids: vec![cell.record.id.clone()],
            confidence: Some(cell.confidence),
            review_status: Some(cell.review_status),
            severity: None,
            provenance: None,
        });
    }
    for law in &scenario.laws {
        records.push(AiProjectionRecord {
            id: law.record.id.clone(),
            record_type: AiProjectionRecordType::CheckResult,
            summary: format!("Law obligation {}.", law.record.summary),
            source_ids: vec![law.record.id.clone()],
            confidence: Some(law.confidence),
            review_status: Some(law.review_status),
            severity: None,
            provenance: None,
        });
    }
    for morphism in &scenario.morphisms {
        records.push(AiProjectionRecord {
            id: morphism.record.id.clone(),
            record_type: AiProjectionRecordType::CheckResult,
            summary: format!("Morphism obligation {}.", morphism.record.morphism_type),
            source_ids: vec![morphism.record.id.clone()],
            confidence: Some(morphism.confidence),
            review_status: Some(morphism.review_status),
            severity: None,
            provenance: None,
        });
    }
    for verification in &scenario.verification_cells {
        records.push(AiProjectionRecord {
            id: verification.record.id.clone(),
            record_type: AiProjectionRecordType::Test,
            summary: format!("Verification cell {}.", verification.record.name),
            source_ids: vec![verification.record.id.clone()],
            confidence: Some(verification.confidence),
            review_status: Some(verification.review_status),
            severity: None,
            provenance: None,
        });
    }
    for invariant_id in &result.evaluated_invariant_ids {
        records.push(AiProjectionRecord {
            id: invariant_id.clone(),
            record_type: AiProjectionRecordType::CheckResult,
            summary: "Evaluated test-gap invariant.".to_owned(),
            source_ids: result.accepted_fact_ids.clone(),
            confidence: None,
            review_status: Some(ReviewStatus::Accepted),
            severity: None,
            provenance: None,
        });
    }
    for obstruction in &result.obstructions {
        records.push(AiProjectionRecord {
            id: obstruction.id.clone(),
            record_type: AiProjectionRecordType::Obstruction,
            summary: obstruction.title.clone(),
            source_ids: obstruction_source_ids(obstruction),
            confidence: Some(obstruction.confidence),
            review_status: Some(obstruction.review_status),
            severity: Some(obstruction.severity),
            provenance: None,
        });
    }
    for candidate in candidates {
        records.push(AiProjectionRecord {
            id: candidate.id.clone(),
            record_type: AiProjectionRecordType::CompletionCandidate,
            summary: candidate.suggested_test_shape.test_name.clone(),
            source_ids: candidate.provenance.source_ids.clone(),
            confidence: Some(candidate.confidence),
            review_status: Some(candidate.review_status),
            severity: Some(candidate.severity),
            provenance: None,
        });
    }
    records
}

fn source_boundary(input: &TestGapInputDocument) -> TestGapSourceBoundary {
    let mut excluded_paths = input.change_set.excluded_paths.clone();
    if let Some(context) = &input.detector_context {
        for path in &context.excluded_paths {
            push_unique_string(&mut excluded_paths, path.clone());
        }
    }
    TestGapSourceBoundary {
        repository_id: input.repository.id.clone(),
        change_set_id: input.change_set.id.clone(),
        base_ref: input.change_set.base_ref.clone(),
        head_ref: input.change_set.head_ref.clone(),
        base_commit: input.change_set.base_commit.clone(),
        head_commit: input.change_set.head_commit.clone(),
        boundary: input.change_set.boundary.clone(),
        adapters: input.source.adapters.clone(),
        excluded_paths,
        coverage_dimensions: unique_coverage_dimensions(input),
        symbol_source: if input.symbols.is_empty() {
            TestGapFactSource::Unavailable
        } else {
            TestGapFactSource::AdapterSupplied
        },
        branch_source: if input.branches.is_empty() {
            TestGapFactSource::Unavailable
        } else {
            TestGapFactSource::AdapterSupplied
        },
        test_mapping_source: if input.tests.iter().any(|test| {
            !test.target_ids.is_empty()
                || !test.branch_ids.is_empty()
                || !test.requirement_ids.is_empty()
        }) {
            TestGapFactSource::AdapterSupplied
        } else {
            TestGapFactSource::Unavailable
        },
        requirement_mapping_source: if input
            .requirements
            .iter()
            .any(|requirement| !requirement.implementation_ids.is_empty())
        {
            TestGapFactSource::AdapterSupplied
        } else {
            TestGapFactSource::Unavailable
        },
        information_loss: source_boundary_information_loss(input),
    }
}

fn source_boundary_information_loss(input: &TestGapInputDocument) -> Vec<String> {
    let mut loss = vec![
        "raw source bodies omitted".to_owned(),
        "full diffs summarized to changed files, symbols, and supplied branch metadata".to_owned(),
        "candidate suggestions are unreviewed detector inference".to_owned(),
    ];
    if input.coverage.is_empty() {
        loss.push("coverage data absent from bounded snapshot".to_owned());
    }
    if input.branches.is_empty() {
        loss.push("branch extraction unavailable in bounded snapshot".to_owned());
    }
    if input
        .tests
        .iter()
        .any(|test| !accepts_test_kind(input, test.test_type))
    {
        loss.push("some represented tests are outside the accepted verification policy".to_owned());
    }
    loss
}

fn accepted_fact_ids(input: &TestGapInputDocument) -> Vec<Id> {
    let mut ids = Vec::new();
    push_unique(&mut ids, input.repository.id.clone());
    push_unique(&mut ids, input.change_set.id.clone());
    for file in &input.changed_files {
        push_unique(&mut ids, file.id.clone());
    }
    for symbol in &input.symbols {
        push_unique(&mut ids, symbol.id.clone());
    }
    for branch in &input.branches {
        push_unique(&mut ids, branch.id.clone());
    }
    for requirement in &input.requirements {
        push_unique(&mut ids, requirement.id.clone());
    }
    for test in &input.tests {
        push_unique(&mut ids, test.id.clone());
    }
    for coverage in &input.coverage {
        push_unique(&mut ids, coverage.id.clone());
    }
    for edge in &input.dependency_edges {
        push_unique(&mut ids, edge.id.clone());
    }
    for cell in &input.higher_order_cells {
        push_unique(&mut ids, cell.id.clone());
    }
    for incidence in &input.higher_order_incidences {
        push_unique(&mut ids, incidence.id.clone());
    }
    for morphism in &input.morphisms {
        push_unique(&mut ids, morphism.id.clone());
    }
    for law in &input.laws {
        push_unique(&mut ids, law.id.clone());
    }
    for verification in &input.verification_cells {
        push_unique(&mut ids, verification.id.clone());
    }
    for context in &input.contexts {
        push_unique(&mut ids, context.id.clone());
    }
    for evidence in &input.evidence {
        push_unique(&mut ids, evidence.id.clone());
    }
    for signal in &input.signals {
        push_unique(&mut ids, signal.id.clone());
    }
    ids
}

fn evaluated_invariant_ids() -> RuntimeResult<Vec<Id>> {
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

fn result_source_ids(
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

fn ensure_detector_output_unreviewed(
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

fn requirement_needs_verification(requirement: &TestGapInputRequirement) -> bool {
    requirement.in_scope || requirement.bug_fix
}

fn requirement_needs_regression(requirement: &TestGapInputRequirement) -> bool {
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

fn symbol_is_public_behavior(symbol: &TestGapInputSymbol) -> bool {
    symbol.public_api
        || matches!(
            symbol.visibility,
            crate::test_gap_reports::TestGapVisibility::Public
        )
}

fn branch_needs_unit_test(branch: &TestGapInputBranch) -> bool {
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

fn has_accepted_test_for_requirement(input: &TestGapInputDocument, requirement_id: &Id) -> bool {
    input.tests.iter().any(|test| {
        accepts_test_kind(input, test.test_type) && test.requirement_ids.contains(requirement_id)
    })
}

fn has_accepted_regression_test_for_requirement(
    input: &TestGapInputDocument,
    requirement_id: &Id,
) -> bool {
    input.tests.iter().any(|test| {
        accepts_test_kind(input, test.test_type)
            && test.is_regression
            && test.requirement_ids.contains(requirement_id)
    })
}

fn has_accepted_test_for_symbol(input: &TestGapInputDocument, symbol_id: &Id) -> bool {
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

fn has_accepted_test_for_branch(input: &TestGapInputDocument, branch_id: &Id) -> bool {
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

fn is_accepted_test(input: &TestGapInputDocument, test_id: &Id) -> bool {
    input
        .tests
        .iter()
        .any(|test| &test.id == test_id && accepts_test_kind(input, test.test_type))
}

fn has_accepted_verification_for_law(input: &TestGapInputDocument, law_id: &Id) -> bool {
    !accepted_verification_ids_for_law(input, law_id).is_empty()
}

fn accepted_verification_ids_for_law(input: &TestGapInputDocument, law_id: &Id) -> Vec<Id> {
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

fn has_accepted_verification_for_morphism(input: &TestGapInputDocument, morphism_id: &Id) -> bool {
    !accepted_verification_ids_for_morphism(input, morphism_id).is_empty()
}

fn accepted_verification_ids_for_morphism(
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

fn accepts_test_kind(input: &TestGapInputDocument, test_type: TestGapTestType) -> bool {
    accepted_test_kinds(input).contains(&test_type)
}

fn accepted_test_kinds(input: &TestGapInputDocument) -> BTreeSet<TestGapTestType> {
    input
        .detector_context
        .as_ref()
        .map(|context| context.test_kinds.clone())
        .filter(|test_kinds| !test_kinds.is_empty())
        .unwrap_or_else(|| vec![TestGapTestType::Unit])
        .into_iter()
        .collect()
}

fn accepted_test_kind_names(input: &TestGapInputDocument) -> Vec<String> {
    accepted_test_kinds(input)
        .into_iter()
        .filter_map(|test_kind| {
            serde_json::to_value(test_kind)
                .ok()
                .and_then(|value| value.as_str().map(ToOwned::to_owned))
        })
        .collect()
}

fn preferred_test_kind(input: &TestGapInputDocument) -> TestGapTestType {
    input
        .detector_context
        .as_ref()
        .and_then(|context| context.test_kinds.first().copied())
        .unwrap_or(TestGapTestType::Unit)
}

fn related_test_ids_for_symbol(input: &TestGapInputDocument, symbol_id: &Id) -> Vec<Id> {
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

fn coverage_ids_for_target(input: &TestGapInputDocument, target_id: &Id) -> Vec<Id> {
    input
        .coverage
        .iter()
        .filter(|coverage| &coverage.target_id == target_id)
        .map(|coverage| coverage.id.clone())
        .collect()
}

fn requirement_target_ids(requirement: &TestGapInputRequirement) -> Vec<Id> {
    let mut ids = vec![requirement.id.clone()];
    for implementation_id in &requirement.implementation_ids {
        push_unique(&mut ids, implementation_id.clone());
    }
    ids
}

fn nonempty_source_ids(input: &TestGapInputDocument, id: &Id, source_ids: &[Id]) -> Vec<Id> {
    let mut ids = vec![id.clone()];
    for source_id in source_ids {
        push_unique(&mut ids, source_id.clone());
    }
    if ids.len() == 1 {
        push_unique(&mut ids, input.change_set.id.clone());
    }
    ids
}

fn obstruction_source_ids(obstruction: &TestGapObstruction) -> Vec<Id> {
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

fn suggested_test_name(obstruction: &TestGapObstruction, target_label: &str) -> String {
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

fn suggested_inputs(obstruction: &TestGapObstruction) -> Value {
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

fn suggested_expected_behavior(obstruction: &TestGapObstruction) -> String {
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

fn suggested_assertion(obstruction: &TestGapObstruction) -> String {
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

fn target_label(input: &TestGapInputDocument, target_id: &Id) -> String {
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

fn human_summary(result: &TestGapResult) -> String {
    match result.status {
        crate::test_gap_reports::TestGapStatus::GapsDetected => format!(
            "Detected {} unreviewed unit-test gaps and proposed {} missing-test candidates.",
            result.obstructions.len(),
            result.completion_candidates.len()
        ),
        crate::test_gap_reports::TestGapStatus::NoGapsInSnapshot => {
            "No unit-test gaps were detected in the bounded snapshot.".to_owned()
        }
        crate::test_gap_reports::TestGapStatus::UnsupportedInput => {
            "The bounded snapshot could not be evaluated by the first test-gap detector slice."
                .to_owned()
        }
    }
}

fn recommended_actions(result: &TestGapResult) -> Vec<String> {
    if result.completion_candidates.is_empty() {
        return vec![
            "Review the source boundary before treating the bounded snapshot as complete."
                .to_owned(),
        ];
    }
    vec![
        "Review each unreviewed missing_test candidate before implementing or accepting it."
            .to_owned(),
        "Add bounded unit-test evidence in a later snapshot or explicit completion review."
            .to_owned(),
    ]
}

fn audit_traces(source_ids: Vec<Id>) -> Vec<ProjectionTrace> {
    source_ids
        .into_iter()
        .map(|source_id| ProjectionTrace {
            role: source_role(&source_id).to_owned(),
            source_id,
            represented_in: vec![
                "human_review".to_owned(),
                "ai_view".to_owned(),
                "audit_trace".to_owned(),
            ],
        })
        .collect()
}

fn source_role(source_id: &Id) -> &'static str {
    let value = source_id.as_str();
    if value.starts_with("file:") {
        "changed_file"
    } else if value.starts_with("symbol:") || value.starts_with("function:") {
        "symbol"
    } else if value.starts_with("branch:") {
        "branch"
    } else if value.starts_with("requirement:") {
        "requirement"
    } else if value.starts_with("test:") {
        "test"
    } else if value.starts_with("coverage:") {
        "coverage"
    } else if value.starts_with("obstruction:") {
        "obstruction"
    } else if value.starts_with("candidate:") {
        "completion_candidate"
    } else if value.starts_with("invariant:") {
        "invariant"
    } else {
        "source"
    }
}

fn effective_context_ids(input: &TestGapInputDocument) -> RuntimeResult<Vec<Id>> {
    let mut context_ids = Vec::new();
    for context in &input.contexts {
        push_unique(&mut context_ids, context.id.clone());
    }
    if context_ids.is_empty() {
        context_ids.push(id(format!("context:test-gap:{}", input.repository.id))?);
    }
    Ok(context_ids)
}

fn contexts_or_default(context_ids: &[Id], default_context_ids: &[Id]) -> Vec<Id> {
    if context_ids.is_empty() {
        default_context_ids.to_vec()
    } else {
        context_ids.to_vec()
    }
}

fn unique_coverage_dimensions(
    input: &TestGapInputDocument,
) -> Vec<crate::test_gap_reports::TestGapCoverageType> {
    let mut dimensions = Vec::new();
    for coverage in &input.coverage {
        if !dimensions.contains(&coverage.coverage_type) {
            dimensions.push(coverage.coverage_type);
        }
    }
    dimensions
}

fn fact_provenance(
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

fn ensure_unique_input_ids(input: &TestGapInputDocument) -> RuntimeResult<()> {
    let mut seen: BTreeSet<Id> = BTreeSet::new();
    for (kind, ids) in [
        (
            "changed_file",
            input
                .changed_files
                .iter()
                .map(|file| file.id.clone())
                .collect::<Vec<_>>(),
        ),
        (
            "symbol",
            input
                .symbols
                .iter()
                .map(|symbol| symbol.id.clone())
                .collect::<Vec<_>>(),
        ),
        (
            "branch",
            input
                .branches
                .iter()
                .map(|branch| branch.id.clone())
                .collect::<Vec<_>>(),
        ),
        (
            "requirement",
            input
                .requirements
                .iter()
                .map(|requirement| requirement.id.clone())
                .collect::<Vec<_>>(),
        ),
        (
            "test",
            input
                .tests
                .iter()
                .map(|test| test.id.clone())
                .collect::<Vec<_>>(),
        ),
        (
            "coverage",
            input
                .coverage
                .iter()
                .map(|coverage| coverage.id.clone())
                .collect::<Vec<_>>(),
        ),
        (
            "dependency_edge",
            input
                .dependency_edges
                .iter()
                .map(|edge| edge.id.clone())
                .collect::<Vec<_>>(),
        ),
        (
            "higher_order_incidence",
            input
                .higher_order_incidences
                .iter()
                .map(|incidence| incidence.id.clone())
                .collect::<Vec<_>>(),
        ),
        (
            "morphism",
            input
                .morphisms
                .iter()
                .map(|morphism| morphism.id.clone())
                .collect::<Vec<_>>(),
        ),
        (
            "verification_cell",
            input
                .verification_cells
                .iter()
                .map(|verification| verification.id.clone())
                .collect::<Vec<_>>(),
        ),
        (
            "context",
            input
                .contexts
                .iter()
                .map(|context| context.id.clone())
                .collect::<Vec<_>>(),
        ),
        (
            "evidence",
            input
                .evidence
                .iter()
                .map(|evidence| evidence.id.clone())
                .collect::<Vec<_>>(),
        ),
        (
            "signal",
            input
                .signals
                .iter()
                .map(|signal| signal.id.clone())
                .collect::<Vec<_>>(),
        ),
    ] {
        for id in ids {
            if !seen.insert(id.clone()) {
                return Err(validation_error(format!(
                    "{kind} id {id} duplicates existing input id"
                )));
            }
        }
    }
    ensure_unique_ids_within_kind(
        "higher_order_cell",
        input
            .higher_order_cells
            .iter()
            .map(|cell| cell.id.clone())
            .collect(),
    )?;
    ensure_unique_ids_within_kind("law", input.laws.iter().map(|law| law.id.clone()).collect())?;
    Ok(())
}

fn ensure_unique_ids_within_kind(kind: &str, ids: Vec<Id>) -> RuntimeResult<()> {
    let mut seen = BTreeSet::new();
    for id in ids {
        if !seen.insert(id.clone()) {
            return Err(validation_error(format!(
                "{kind} id {id} duplicates existing {kind} id"
            )));
        }
    }
    Ok(())
}

fn ensure_known_ids(
    known_ids: &[Id],
    referenced_ids: &[Id],
    owner_kind: &str,
    owner_id: &Id,
    referenced_kind: &str,
) -> RuntimeResult<()> {
    for referenced_id in referenced_ids {
        ensure_known_id(
            known_ids,
            referenced_id,
            owner_kind,
            owner_id,
            referenced_kind,
        )?;
    }
    Ok(())
}

fn ensure_known_id(
    known_ids: &[Id],
    referenced_id: &Id,
    owner_kind: &str,
    owner_id: &Id,
    referenced_kind: &str,
) -> RuntimeResult<()> {
    if known_ids.contains(referenced_id) {
        return Ok(());
    }
    Err(validation_error(format!(
        "{owner_kind} id {owner_id} references unknown {referenced_kind} {referenced_id}"
    )))
}

fn space_id(input: &TestGapInputDocument) -> RuntimeResult<Id> {
    id(format!(
        "space:test-gap:{}:{}",
        input.repository.id, input.change_set.id
    ))
}

fn incidence_id(prefix: &str, from_id: &Id, to_id: &Id) -> RuntimeResult<Id> {
    id(format!("{prefix}:{}:{}", slug(from_id), slug(to_id)))
}

fn id(value: impl Into<String>) -> RuntimeResult<Id> {
    Ok(Id::new(value)?)
}

fn slug(id: &Id) -> String {
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

fn file_label(path: &str) -> String {
    path.rsplit('/')
        .next()
        .filter(|name| !name.is_empty())
        .unwrap_or(path)
        .to_owned()
}

fn obstruction_slug(obstruction_type: TestGapObstructionType) -> &'static str {
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

fn missing_candidate_slug(obstruction_type: TestGapObstructionType) -> &'static str {
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

fn serde_plain_context_type(context_type: crate::test_gap_reports::TestGapContextType) -> String {
    serde_json::to_value(context_type)
        .ok()
        .and_then(|value| value.as_str().map(ToOwned::to_owned))
        .unwrap_or_else(|| "custom".to_owned())
}

fn serde_plain_dependency_relation_type(
    relation_type: crate::test_gap_reports::TestGapDependencyRelationType,
) -> String {
    serde_json::to_value(relation_type)
        .ok()
        .and_then(|value| value.as_str().map(ToOwned::to_owned))
        .unwrap_or_else(|| "custom".to_owned())
}

fn push_unique(ids: &mut Vec<Id>, id: Id) {
    if !ids.contains(&id) {
        ids.push(id);
    }
}

fn push_unique_string(values: &mut Vec<String>, value: String) {
    if !values.contains(&value) {
        values.push(value);
    }
}

fn validation_error(reason: impl Into<String>) -> RuntimeError {
    RuntimeError::workflow_construction(WORKFLOW_NAME, reason)
}
