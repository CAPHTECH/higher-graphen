use super::pr_review_target_lift::LiftedPrReviewTarget;
use crate::pr_review_reports::{
    PrReviewTargetChangedFile, PrReviewTargetContext, PrReviewTargetDependencyEdge,
    PrReviewTargetEvidence, PrReviewTargetInputDocument, PrReviewTargetOwner,
    PrReviewTargetRiskSignal, PrReviewTargetScenario, PrReviewTargetSymbol, PrReviewTargetTest,
};
use higher_graphen_core::ReviewStatus;

pub(super) fn report_scenario(
    input: &PrReviewTargetInputDocument,
    lifted: LiftedPrReviewTarget,
) -> PrReviewTargetScenario {
    PrReviewTargetScenario {
        input_schema: input.schema.clone(),
        source: input.source.clone(),
        repository: input.repository.clone(),
        pull_request: input.pull_request.clone(),
        changed_files: scenario_changed_files(input),
        symbols: scenario_symbols(input),
        owners: scenario_owners(input),
        contexts: scenario_contexts(input),
        tests: scenario_tests(input),
        dependency_edges: scenario_dependency_edges(input),
        evidence: scenario_evidence(input),
        signals: scenario_signals(input),
        reviewer_context: input.reviewer_context.clone(),
        lifted_structure: lifted.structure,
    }
}

fn scenario_changed_files(input: &PrReviewTargetInputDocument) -> Vec<PrReviewTargetChangedFile> {
    input
        .changed_files
        .iter()
        .map(|file| PrReviewTargetChangedFile {
            id: file.id.clone(),
            path: file.path.clone(),
            change_type: file.change_type,
            language: file.language.clone(),
            additions: file.additions,
            deletions: file.deletions,
            symbol_ids: file.symbol_ids.clone(),
            owner_ids: file.owner_ids.clone(),
            context_ids: file.context_ids.clone(),
            review_status: ReviewStatus::Accepted,
            confidence: input.source.confidence,
        })
        .collect()
}

fn scenario_symbols(input: &PrReviewTargetInputDocument) -> Vec<PrReviewTargetSymbol> {
    input
        .symbols
        .iter()
        .map(|symbol| PrReviewTargetSymbol {
            id: symbol.id.clone(),
            file_id: symbol.file_id.clone(),
            name: symbol.name.clone(),
            kind: symbol.kind,
            path: symbol.path.clone(),
            line_start: symbol.line_start,
            line_end: symbol.line_end,
            owner_ids: symbol.owner_ids.clone(),
            context_ids: symbol.context_ids.clone(),
            review_status: ReviewStatus::Accepted,
            confidence: input.source.confidence,
        })
        .collect()
}

fn scenario_owners(input: &PrReviewTargetInputDocument) -> Vec<PrReviewTargetOwner> {
    input
        .owners
        .iter()
        .map(|owner| PrReviewTargetOwner {
            id: owner.id.clone(),
            owner_type: owner.owner_type,
            name: owner.name.clone(),
            source_ids: owner.source_ids.clone(),
            review_status: ReviewStatus::Accepted,
            confidence: input.source.confidence,
        })
        .collect()
}

fn scenario_contexts(input: &PrReviewTargetInputDocument) -> Vec<PrReviewTargetContext> {
    input
        .contexts
        .iter()
        .map(|context| PrReviewTargetContext {
            id: context.id.clone(),
            name: context.name.clone(),
            context_type: context.context_type,
            source_ids: context.source_ids.clone(),
            review_status: ReviewStatus::Accepted,
            confidence: input.source.confidence,
        })
        .collect()
}

fn scenario_tests(input: &PrReviewTargetInputDocument) -> Vec<PrReviewTargetTest> {
    input
        .tests
        .iter()
        .map(|test| PrReviewTargetTest {
            id: test.id.clone(),
            name: test.name.clone(),
            test_type: test.test_type,
            file_id: test.file_id.clone(),
            symbol_ids: test.symbol_ids.clone(),
            context_ids: test.context_ids.clone(),
            source_ids: test.source_ids.clone(),
            review_status: ReviewStatus::Accepted,
            confidence: input.source.confidence,
        })
        .collect()
}

fn scenario_dependency_edges(
    input: &PrReviewTargetInputDocument,
) -> Vec<PrReviewTargetDependencyEdge> {
    input
        .dependency_edges
        .iter()
        .map(|edge| PrReviewTargetDependencyEdge {
            id: edge.id.clone(),
            from_id: edge.from_id.clone(),
            to_id: edge.to_id.clone(),
            relation_type: edge.relation_type,
            orientation: edge.orientation,
            source_ids: edge.source_ids.clone(),
            review_status: ReviewStatus::Accepted,
            confidence: edge.confidence.unwrap_or(input.source.confidence),
        })
        .collect()
}

fn scenario_evidence(input: &PrReviewTargetInputDocument) -> Vec<PrReviewTargetEvidence> {
    input
        .evidence
        .iter()
        .map(|evidence| PrReviewTargetEvidence {
            id: evidence.id.clone(),
            evidence_type: evidence.evidence_type,
            summary: evidence.summary.clone(),
            source_ids: evidence.source_ids.clone(),
            review_status: ReviewStatus::Accepted,
            confidence: evidence.confidence.unwrap_or(input.source.confidence),
        })
        .collect()
}

fn scenario_signals(input: &PrReviewTargetInputDocument) -> Vec<PrReviewTargetRiskSignal> {
    input
        .signals
        .iter()
        .map(|signal| PrReviewTargetRiskSignal {
            id: signal.id.clone(),
            signal_type: signal.signal_type,
            summary: signal.summary.clone(),
            source_ids: signal.source_ids.clone(),
            severity: signal.severity,
            confidence: signal.confidence,
        })
        .collect()
}
