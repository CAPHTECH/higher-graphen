use super::*;

pub(super) fn report_projection(
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

pub(super) fn ai_projection_records(
    scenario: &TestGapScenario,
    result: &TestGapResult,
    candidates: &[TestGapCompletionCandidate],
) -> Vec<AiProjectionRecord> {
    let mut records = Vec::new();
    append_identity_records(scenario, &mut records);
    append_semantic_records(scenario, &mut records);
    append_result_records(result, &mut records);
    append_candidate_records(candidates, &mut records);
    records
}

fn append_identity_records(scenario: &TestGapScenario, records: &mut Vec<AiProjectionRecord>) {
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
}

fn append_semantic_records(scenario: &TestGapScenario, records: &mut Vec<AiProjectionRecord>) {
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
}

fn append_result_records(result: &TestGapResult, records: &mut Vec<AiProjectionRecord>) {
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
    for proof in &result.proof_objects {
        records.push(AiProjectionRecord {
            id: proof.id.clone(),
            record_type: AiProjectionRecordType::CheckResult,
            summary: proof.summary.clone(),
            source_ids: proof.witness_ids.clone(),
            confidence: Some(proof.confidence),
            review_status: Some(proof.review_status),
            severity: None,
            provenance: None,
        });
    }
    for counterexample in &result.counterexamples {
        records.push(AiProjectionRecord {
            id: counterexample.id.clone(),
            record_type: AiProjectionRecordType::Obstruction,
            summary: counterexample.summary.clone(),
            source_ids: counterexample.path_ids.clone(),
            confidence: Some(counterexample.confidence),
            review_status: Some(counterexample.review_status),
            severity: Some(Severity::High),
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
}

fn append_candidate_records(
    candidates: &[TestGapCompletionCandidate],
    records: &mut Vec<AiProjectionRecord>,
) {
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
}

pub(super) fn human_summary(result: &TestGapResult) -> String {
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

pub(super) fn recommended_actions(result: &TestGapResult) -> Vec<String> {
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

pub(super) fn audit_traces(source_ids: Vec<Id>) -> Vec<ProjectionTrace> {
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

pub(super) fn source_role(source_id: &Id) -> &'static str {
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
