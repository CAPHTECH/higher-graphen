use super::*;

pub(super) fn report_scenario(
    input: &TestGapInputDocument,
    lifted_structure: TestGapLiftedStructure,
) -> TestGapScenario {
    TestGapScenario {
        input_schema: input.schema.clone(),
        source_boundary: source_boundary(input),
        source: input.source.clone(),
        repository: input.repository.clone(),
        change_set: input.change_set.clone(),
        changed_files: observed_changed_files(input),
        symbols: observed_symbols(input),
        branches: observed_branches(input),
        requirements: observed_requirements(input),
        tests: observed_tests(input),
        coverage: observed_coverage(input),
        dependency_edges: observed_dependency_edges(input),
        higher_order_cells: observed_higher_order_cells(input),
        higher_order_incidences: observed_higher_order_incidences(input),
        morphisms: observed_morphisms(input),
        laws: observed_laws(input),
        verification_cells: observed_verification_cells(input),
        contexts: observed_contexts(input),
        evidence: observed_evidence(input),
        signals: observed_signals(input),
        detector_context: input.detector_context.clone(),
        lifted_structure,
    }
}

fn observed_changed_files(input: &TestGapInputDocument) -> Vec<TestGapObservedChangedFile> {
    input
        .changed_files
        .iter()
        .cloned()
        .map(|record| TestGapObservedChangedFile {
            record,
            review_status: ReviewStatus::Accepted,
            confidence: input.source.confidence,
        })
        .collect()
}

fn observed_symbols(input: &TestGapInputDocument) -> Vec<TestGapObservedSymbol> {
    input
        .symbols
        .iter()
        .cloned()
        .map(|record| TestGapObservedSymbol {
            record,
            review_status: ReviewStatus::Accepted,
            confidence: input.source.confidence,
        })
        .collect()
}

fn observed_branches(input: &TestGapInputDocument) -> Vec<TestGapObservedBranch> {
    input
        .branches
        .iter()
        .cloned()
        .map(|record| TestGapObservedBranch {
            record,
            review_status: ReviewStatus::Accepted,
            confidence: input.source.confidence,
        })
        .collect()
}

fn observed_requirements(input: &TestGapInputDocument) -> Vec<TestGapObservedRequirement> {
    input
        .requirements
        .iter()
        .cloned()
        .map(|record| TestGapObservedRequirement {
            record,
            review_status: ReviewStatus::Accepted,
            confidence: input.source.confidence,
        })
        .collect()
}

fn observed_tests(input: &TestGapInputDocument) -> Vec<TestGapObservedTest> {
    input
        .tests
        .iter()
        .cloned()
        .map(|record| TestGapObservedTest {
            record,
            review_status: ReviewStatus::Accepted,
            confidence: input.source.confidence,
        })
        .collect()
}

fn observed_coverage(input: &TestGapInputDocument) -> Vec<TestGapObservedCoverage> {
    input
        .coverage
        .iter()
        .cloned()
        .map(|record| TestGapObservedCoverage {
            confidence: record.confidence.unwrap_or(input.source.confidence),
            record,
            review_status: ReviewStatus::Accepted,
        })
        .collect()
}

fn observed_dependency_edges(input: &TestGapInputDocument) -> Vec<TestGapObservedDependencyEdge> {
    input
        .dependency_edges
        .iter()
        .cloned()
        .map(|record| TestGapObservedDependencyEdge {
            confidence: record.confidence.unwrap_or(input.source.confidence),
            record,
            review_status: ReviewStatus::Accepted,
        })
        .collect()
}

fn observed_higher_order_cells(
    input: &TestGapInputDocument,
) -> Vec<TestGapObservedHigherOrderCell> {
    input
        .higher_order_cells
        .iter()
        .cloned()
        .map(|record| TestGapObservedHigherOrderCell {
            confidence: record.confidence.unwrap_or(input.source.confidence),
            record,
            review_status: ReviewStatus::Accepted,
        })
        .collect()
}

fn observed_higher_order_incidences(
    input: &TestGapInputDocument,
) -> Vec<TestGapObservedHigherOrderIncidence> {
    input
        .higher_order_incidences
        .iter()
        .cloned()
        .map(|record| TestGapObservedHigherOrderIncidence {
            confidence: record.confidence.unwrap_or(input.source.confidence),
            record,
            review_status: ReviewStatus::Accepted,
        })
        .collect()
}

fn observed_morphisms(input: &TestGapInputDocument) -> Vec<TestGapObservedInputMorphism> {
    input
        .morphisms
        .iter()
        .cloned()
        .map(|record| TestGapObservedInputMorphism {
            confidence: record.confidence.unwrap_or(input.source.confidence),
            record,
            review_status: ReviewStatus::Accepted,
        })
        .collect()
}

fn observed_laws(input: &TestGapInputDocument) -> Vec<TestGapObservedInputLaw> {
    input
        .laws
        .iter()
        .cloned()
        .map(|record| TestGapObservedInputLaw {
            confidence: record.confidence.unwrap_or(input.source.confidence),
            record,
            review_status: ReviewStatus::Accepted,
        })
        .collect()
}

fn observed_verification_cells(
    input: &TestGapInputDocument,
) -> Vec<TestGapObservedVerificationCell> {
    input
        .verification_cells
        .iter()
        .cloned()
        .map(|record| TestGapObservedVerificationCell {
            confidence: record.confidence.unwrap_or(input.source.confidence),
            record,
            review_status: ReviewStatus::Accepted,
        })
        .collect()
}

fn observed_contexts(input: &TestGapInputDocument) -> Vec<TestGapObservedContext> {
    input
        .contexts
        .iter()
        .cloned()
        .map(|record| TestGapObservedContext {
            record,
            review_status: ReviewStatus::Accepted,
            confidence: input.source.confidence,
        })
        .collect()
}

fn observed_evidence(input: &TestGapInputDocument) -> Vec<TestGapObservedEvidence> {
    input
        .evidence
        .iter()
        .cloned()
        .map(|record| TestGapObservedEvidence {
            confidence: record.confidence.unwrap_or(input.source.confidence),
            record,
            review_status: ReviewStatus::Accepted,
        })
        .collect()
}

fn observed_signals(input: &TestGapInputDocument) -> Vec<TestGapObservedRiskSignal> {
    input
        .signals
        .iter()
        .cloned()
        .map(|record| TestGapObservedRiskSignal {
            record,
            review_status: ReviewStatus::Accepted,
        })
        .collect()
}
