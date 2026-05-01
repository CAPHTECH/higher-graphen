//! Explicit completion candidate review workflow.

use crate::error::{RuntimeError, RuntimeResult};
use crate::reports::{
    AiProjectionRecord, AiProjectionRecordType, AiProjectionView, AuditProjectionView,
    CompletionReviewProjection, CompletionReviewReport, CompletionReviewResult,
    CompletionReviewScenario, CompletionReviewSnapshot, CompletionReviewSourceReport,
    CompletionReviewStatus, HumanReviewProjectionView, ProjectionAudience, ProjectionPurpose,
    ProjectionTrace, ProjectionViewSet, ReportEnvelope, ReportMetadata,
};
use higher_graphen_core::{Id, ReviewStatus};
use higher_graphen_projection::InformationLoss;
use higher_graphen_reasoning::completion::{
    review_completion, CompletionCandidate, CompletionReviewDecision, CompletionReviewRequest,
};

const WORKFLOW_NAME: &str = "completion_review";
const REPORT_SCHEMA: &str = "highergraphen.completion.review.report.v1";
const REPORT_TYPE: &str = "completion_review";
const REPORT_VERSION: u32 = 1;

/// Reviews one completion candidate from a report or snapshot.
///
/// The workflow accepts or rejects only the explicitly selected candidate,
/// preserves the source candidate snapshot, and emits a review report instead
/// of mutating or promoting the candidate in place.
pub fn run_completion_review(
    snapshot: CompletionReviewSnapshot,
    request: CompletionReviewRequest,
) -> RuntimeResult<CompletionReviewReport> {
    validate_source_report(&snapshot.source_report)?;
    let candidate = find_candidate(&snapshot.completion_candidates, &request.candidate_id)?;
    ensure_unreviewed_candidate(&candidate)?;

    let record = review_completion(&candidate, request)?;
    let status = status_from_decision(record.decision());
    let scenario = CompletionReviewScenario {
        source_report: snapshot.source_report,
        candidate,
    };
    let result = CompletionReviewResult {
        status,
        review_record: record,
    };
    let projection = report_projection(&scenario, &result)?;

    Ok(ReportEnvelope {
        schema: REPORT_SCHEMA.to_owned(),
        report_type: REPORT_TYPE.to_owned(),
        report_version: REPORT_VERSION,
        metadata: ReportMetadata::completion_review(command_action(status)),
        scenario,
        result,
        projection,
    })
}

fn validate_source_report(source_report: &CompletionReviewSourceReport) -> RuntimeResult<()> {
    required_text("source_report.schema", &source_report.schema)?;
    required_text("source_report.report_type", &source_report.report_type)?;
    required_text("source_report.command", &source_report.command)?;
    if source_report.report_version == 0 {
        return Err(RuntimeError::workflow_construction(
            WORKFLOW_NAME,
            "source_report.report_version must be greater than zero",
        ));
    }

    Ok(())
}

fn find_candidate(
    candidates: &[CompletionCandidate],
    candidate_id: &Id,
) -> RuntimeResult<CompletionCandidate> {
    let mut matches = candidates
        .iter()
        .filter(|candidate| candidate.id == *candidate_id);
    let candidate = matches.next().ok_or_else(|| {
        RuntimeError::workflow_construction(
            WORKFLOW_NAME,
            format!("candidate {candidate_id} was not found in the source snapshot"),
        )
    })?;
    if matches.next().is_some() {
        return Err(RuntimeError::workflow_construction(
            WORKFLOW_NAME,
            format!("candidate {candidate_id} appears more than once in the source snapshot"),
        ));
    }

    Ok(candidate.clone())
}

fn ensure_unreviewed_candidate(candidate: &CompletionCandidate) -> RuntimeResult<()> {
    if candidate.review_status != ReviewStatus::Unreviewed {
        return Err(RuntimeError::workflow_construction(
            WORKFLOW_NAME,
            format!(
                "candidate {} has review status {:?}; only unreviewed candidates can be reviewed",
                candidate.id, candidate.review_status
            ),
        ));
    }

    Ok(())
}

fn status_from_decision(decision: CompletionReviewDecision) -> CompletionReviewStatus {
    match decision {
        CompletionReviewDecision::Accepted => CompletionReviewStatus::Accepted,
        CompletionReviewDecision::Rejected => CompletionReviewStatus::Rejected,
    }
}

fn command_action(status: CompletionReviewStatus) -> &'static str {
    match status {
        CompletionReviewStatus::Accepted => "accept",
        CompletionReviewStatus::Rejected => "reject",
    }
}

fn report_projection(
    scenario: &CompletionReviewScenario,
    result: &CompletionReviewResult,
) -> RuntimeResult<CompletionReviewProjection> {
    let candidate = &scenario.candidate;
    let source_ids = projection_source_ids(result);
    let human_loss = InformationLoss::declared(
        "Projection summarizes the selected completion candidate, review request, and review outcome.",
        source_ids.clone(),
    )?;
    let ai_loss = InformationLoss::declared(
        "AI view preserves candidate and review records with IDs, confidence, and review status but omits full source report payloads.",
        source_ids.clone(),
    )?;
    let audit_loss = InformationLoss::declared(
        "Audit trace records represented source identifiers and view coverage but omits full object payloads.",
        source_ids.clone(),
    )?;
    let (summary, recommended_actions) = match result.status {
        CompletionReviewStatus::Accepted => (
            format!(
                "Accepted completion candidate {} from {}.",
                candidate.id, scenario.source_report.report_type
            ),
            vec![
                "Create or promote the accepted structure through an explicit downstream workflow."
                    .to_owned(),
                "Keep the review report with the source report for auditability.".to_owned(),
            ],
        ),
        CompletionReviewStatus::Rejected => (
            format!(
                "Rejected completion candidate {} from {}.",
                candidate.id, scenario.source_report.report_type
            ),
            vec![
                "Do not promote the rejected candidate into accepted structure.".to_owned(),
                "Keep the rejection rationale with the source report for auditability.".to_owned(),
            ],
        ),
    };
    let human_review = HumanReviewProjectionView {
        audience: ProjectionAudience::Human,
        purpose: ProjectionPurpose::CompletionReview,
        summary,
        recommended_actions,
        source_ids: source_ids.clone(),
        information_loss: vec![human_loss],
    };
    let ai_view = AiProjectionView {
        audience: ProjectionAudience::AiAgent,
        purpose: ProjectionPurpose::CompletionReview,
        records: ai_projection_records(result)?,
        source_ids: source_ids.clone(),
        information_loss: vec![ai_loss],
    };
    let audit_trace = AuditProjectionView {
        audience: ProjectionAudience::Audit,
        purpose: ProjectionPurpose::AuditTrace,
        source_ids,
        information_loss: vec![audit_loss],
        traces: audit_traces(ai_view.source_ids.clone()),
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

fn projection_source_ids(result: &CompletionReviewResult) -> Vec<Id> {
    let mut ids = candidate_source_ids(&result.review_record.candidate);
    push_unique(&mut ids, result.review_record.request.reviewer_id.clone());
    match result.status {
        CompletionReviewStatus::Accepted => {
            if let Some(accepted) = &result.review_record.accepted_completion {
                push_unique(&mut ids, accepted.candidate_id.clone());
                push_unique(&mut ids, accepted.space_id.clone());
                if let Some(structure_id) = &accepted.accepted_structure.structure_id {
                    push_unique(&mut ids, structure_id.clone());
                }
                for inferred_from in &accepted.inferred_from {
                    push_unique(&mut ids, inferred_from.clone());
                }
            }
        }
        CompletionReviewStatus::Rejected => {
            if let Some(rejected) = &result.review_record.rejected_completion {
                push_unique(&mut ids, rejected.candidate_id.clone());
                push_unique(&mut ids, rejected.space_id.clone());
                if let Some(structure_id) = &rejected.rejected_structure.structure_id {
                    push_unique(&mut ids, structure_id.clone());
                }
                for inferred_from in &rejected.inferred_from {
                    push_unique(&mut ids, inferred_from.clone());
                }
            }
        }
    }
    ids
}

fn ai_projection_records(
    result: &CompletionReviewResult,
) -> RuntimeResult<Vec<AiProjectionRecord>> {
    let candidate = &result.review_record.candidate;
    let candidate_record = AiProjectionRecord {
        id: candidate.id.clone(),
        record_type: AiProjectionRecordType::CompletionCandidate,
        summary: candidate.suggested_structure.summary.clone(),
        source_ids: candidate_source_ids(candidate),
        confidence: Some(candidate.confidence),
        review_status: Some(candidate.review_status),
        severity: None,
        provenance: None,
    };
    let review_record = AiProjectionRecord {
        id: id(format!(
            "review:{}",
            result.review_record.request.candidate_id
        ))?,
        record_type: AiProjectionRecordType::CompletionReview,
        summary: format!(
            "{:?} completion candidate {}",
            result.review_record.request.decision, result.review_record.request.candidate_id
        ),
        source_ids: projection_source_ids(result),
        confidence: review_confidence(result),
        review_status: Some(result.review_record.outcome_review_status),
        severity: None,
        provenance: None,
    };
    Ok(vec![candidate_record, review_record])
}

fn review_confidence(result: &CompletionReviewResult) -> Option<higher_graphen_core::Confidence> {
    result
        .review_record
        .accepted_completion
        .as_ref()
        .map(|accepted| accepted.confidence)
        .or_else(|| {
            result
                .review_record
                .rejected_completion
                .as_ref()
                .map(|rejected| rejected.confidence)
        })
}

fn candidate_source_ids(candidate: &CompletionCandidate) -> Vec<Id> {
    let mut ids = vec![candidate.id.clone(), candidate.space_id.clone()];
    if let Some(structure_id) = &candidate.suggested_structure.structure_id {
        push_unique(&mut ids, structure_id.clone());
    }
    for related_id in &candidate.suggested_structure.related_ids {
        push_unique(&mut ids, related_id.clone());
    }
    for inferred_from in &candidate.inferred_from {
        push_unique(&mut ids, inferred_from.clone());
    }
    ids
}

fn push_unique(ids: &mut Vec<Id>, id: Id) {
    if !ids.contains(&id) {
        ids.push(id);
    }
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
    if source_id.as_str().starts_with("candidate:") {
        "completion_candidate"
    } else if source_id.as_str().starts_with("cell:") {
        "cell"
    } else if source_id.as_str().starts_with("incidence:") {
        "incidence"
    } else if source_id.as_str().starts_with("reviewer:") {
        "reviewer"
    } else if source_id.as_str().starts_with("space:") {
        "space"
    } else {
        "source"
    }
}

fn id(value: impl Into<String>) -> RuntimeResult<Id> {
    Ok(Id::new(value)?)
}

fn required_text(field: &'static str, value: &str) -> RuntimeResult<()> {
    if value.trim().is_empty() {
        return Err(RuntimeError::workflow_construction(
            WORKFLOW_NAME,
            format!("{field} must not be empty"),
        ));
    }

    Ok(())
}
