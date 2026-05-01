//! Bounded PR review target recommender workflow.

#[path = "pr_review_target_lift.rs"]
mod pr_review_target_lift;
#[path = "pr_review_target_recommend.rs"]
mod pr_review_target_recommend;
#[path = "pr_review_target_scenario.rs"]
mod pr_review_target_scenario;
#[path = "pr_review_target_validate.rs"]
mod pr_review_target_validate;

use crate::error::{RuntimeError, RuntimeResult};
use crate::pr_review_reports::{
    PrReviewTarget, PrReviewTargetInputDocument, PrReviewTargetObstruction, PrReviewTargetReport,
    PrReviewTargetResult, PrReviewTargetScenario, PrReviewTargetStatus,
};
use crate::reports::{
    AiProjectionRecord, AiProjectionRecordType, AiProjectionView, AuditProjectionView,
    HumanReviewProjectionView, ProjectionAudience, ProjectionPurpose, ProjectionTrace,
    ProjectionViewSet, ReportEnvelope, ReportMetadata,
};
use higher_graphen_core::{Confidence, Id, ReviewStatus};
use higher_graphen_projection::InformationLoss;
use higher_graphen_reasoning::completion::{CompletionCandidate, MissingType, SuggestedStructure};

use self::pr_review_target_lift::lift_input;
use self::pr_review_target_recommend::{recommend_review_targets, review_obstructions};
use self::pr_review_target_scenario::report_scenario;
use self::pr_review_target_validate::{validate_input_references, validate_input_schema};

const WORKFLOW_NAME: &str = "pr_review_target";
const INPUT_SCHEMA: &str = "highergraphen.pr_review_target.input.v1";
const REPORT_SCHEMA: &str = "highergraphen.pr_review_target.report.v1";
const REPORT_TYPE: &str = "pr_review_target";
const REPORT_VERSION: u32 = 1;
const EXTRACTION_METHOD: &str = "pr_review_target_lift.v1";

/// Runs the bounded PR review target recommender workflow.
pub fn run_pr_review_target_recommend(
    input: PrReviewTargetInputDocument,
) -> RuntimeResult<PrReviewTargetReport> {
    validate_input_schema(&input)?;
    validate_input_references(&input)?;

    let lifted = lift_input(&input)?;
    let accepted_change_ids = accepted_change_ids(&input);
    let review_targets = recommend_review_targets(&input)?;
    let obstructions = review_obstructions(&input, &review_targets)?;
    let completion_candidates = completion_candidates(&input, &obstructions)?;
    ensure_ai_proposals_are_unreviewed(&review_targets, &obstructions, &completion_candidates)?;

    let mut source_ids = result_source_ids(
        &accepted_change_ids,
        &review_targets,
        &obstructions,
        &completion_candidates,
    );
    if source_ids.is_empty() {
        source_ids = accepted_change_ids.clone();
    }

    let status = if review_targets.is_empty() && obstructions.is_empty() {
        PrReviewTargetStatus::NoTargets
    } else {
        PrReviewTargetStatus::TargetsRecommended
    };
    let result = PrReviewTargetResult {
        status,
        accepted_change_ids,
        review_targets,
        obstructions,
        completion_candidates,
        source_ids,
    };
    let scenario = report_scenario(&input, lifted);
    let projection = report_projection(&input, &scenario, &result)?;

    Ok(ReportEnvelope {
        schema: REPORT_SCHEMA.to_owned(),
        report_type: REPORT_TYPE.to_owned(),
        report_version: REPORT_VERSION,
        metadata: ReportMetadata::pr_review_target(),
        scenario,
        result,
        projection,
    })
}

fn completion_candidates(
    input: &PrReviewTargetInputDocument,
    obstructions: &[PrReviewTargetObstruction],
) -> RuntimeResult<Vec<CompletionCandidate>> {
    if obstructions.is_empty() {
        return Ok(Vec::new());
    }
    let space_id = space_id(input)?;
    let candidate_id = id(format!(
        "candidate:{}:review-checklist",
        slug(&input.pull_request.id)
    ))?;
    let structure_id = id(format!(
        "section:{}:review-checklist",
        slug(&input.pull_request.id)
    ))?;
    let suggested = SuggestedStructure::new(
        "review_checklist",
        "Add a checklist section for the unreviewed PR review targets.",
    )?
    .with_structure_id(structure_id)
    .with_related_ids(
        obstructions
            .iter()
            .map(|obstruction| obstruction.id.clone())
            .collect(),
    );
    let candidate = CompletionCandidate::new(
        candidate_id,
        space_id,
        MissingType::Section,
        suggested,
        obstructions
            .iter()
            .map(|obstruction| obstruction.id.clone())
            .collect(),
        "The recommender found unresolved review risks that should be tracked explicitly.",
        Confidence::new(0.71)?,
    )?;
    Ok(vec![candidate])
}

fn report_projection(
    input: &PrReviewTargetInputDocument,
    scenario: &PrReviewTargetScenario,
    result: &PrReviewTargetResult,
) -> RuntimeResult<ProjectionViewSet> {
    let source_ids = if result.source_ids.is_empty() {
        result.accepted_change_ids.clone()
    } else {
        result.source_ids.clone()
    };
    let human_loss = InformationLoss::declared(
        "Projection summarizes changed files, symbols, risk signals, targets, obstructions, and completion candidates without embedding raw provider payloads.",
        source_ids.clone(),
    )?;
    let ai_loss = InformationLoss::declared(
        "AI view preserves stable IDs, severity, confidence, and review status but omits full changed-file payloads.",
        source_ids.clone(),
    )?;
    let audit_loss = InformationLoss::declared(
        "Audit trace records represented source identifiers and view coverage but omits raw diff hunks and provider payloads.",
        source_ids.clone(),
    )?;
    let human_review = HumanReviewProjectionView {
        audience: ProjectionAudience::Human,
        purpose: ProjectionPurpose::PrReviewTargeting,
        summary: human_summary(result),
        recommended_actions: vec![
            "Review the unreviewed targets before treating this PR as covered.".to_owned(),
            "Record explicit accept or reject decisions outside this recommendation report."
                .to_owned(),
        ],
        source_ids: source_ids.clone(),
        information_loss: vec![human_loss],
    };
    let ai_view = AiProjectionView {
        audience: ProjectionAudience::AiAgent,
        purpose: ProjectionPurpose::PrReviewTargeting,
        records: ai_projection_records(input, scenario, result),
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
    input: &PrReviewTargetInputDocument,
    scenario: &PrReviewTargetScenario,
    result: &PrReviewTargetResult,
) -> Vec<AiProjectionRecord> {
    let mut records = Vec::new();
    for file in &scenario.changed_files {
        records.push(AiProjectionRecord {
            id: file.id.clone(),
            record_type: AiProjectionRecordType::ChangedFile,
            summary: format!("{} changed file.", file_label(&file.path)),
            source_ids: nonempty_source_ids(input, &file.id),
            confidence: Some(file.confidence),
            review_status: Some(file.review_status),
            severity: None,
            provenance: None,
        });
    }
    for symbol in &scenario.symbols {
        records.push(AiProjectionRecord {
            id: symbol.id.clone(),
            record_type: AiProjectionRecordType::Symbol,
            summary: format!("Changed symbol {}.", symbol.name),
            source_ids: vec![symbol.id.clone(), symbol.file_id.clone()],
            confidence: Some(symbol.confidence),
            review_status: Some(symbol.review_status),
            severity: None,
            provenance: None,
        });
    }
    for signal in &scenario.signals {
        records.push(AiProjectionRecord {
            id: signal.id.clone(),
            record_type: AiProjectionRecordType::RiskSignal,
            summary: signal.summary.clone(),
            source_ids: signal.source_ids.clone(),
            confidence: Some(signal.confidence),
            review_status: Some(ReviewStatus::Accepted),
            severity: Some(signal.severity),
            provenance: None,
        });
    }
    for target in &result.review_targets {
        records.push(AiProjectionRecord {
            id: target.id.clone(),
            record_type: AiProjectionRecordType::ReviewTarget,
            summary: target.title.clone(),
            source_ids: target_source_ids(target),
            confidence: Some(target.confidence),
            review_status: Some(target.review_status),
            severity: Some(target.severity),
            provenance: None,
        });
    }
    for obstruction in &result.obstructions {
        records.push(AiProjectionRecord {
            id: obstruction.id.clone(),
            record_type: AiProjectionRecordType::Obstruction,
            summary: obstruction.summary.clone(),
            source_ids: obstruction.source_ids.clone(),
            confidence: Some(obstruction.confidence),
            review_status: Some(obstruction.review_status),
            severity: Some(obstruction.severity),
            provenance: None,
        });
    }
    for candidate in &result.completion_candidates {
        records.push(AiProjectionRecord {
            id: candidate.id.clone(),
            record_type: AiProjectionRecordType::CompletionCandidate,
            summary: candidate.suggested_structure.summary.clone(),
            source_ids: completion_candidate_source_ids(candidate),
            confidence: Some(candidate.confidence),
            review_status: Some(candidate.review_status),
            severity: None,
            provenance: None,
        });
    }
    records
}

fn accepted_change_ids(input: &PrReviewTargetInputDocument) -> Vec<Id> {
    let mut ids = Vec::new();
    ids.extend(input.changed_files.iter().map(|file| file.id.clone()));
    ids.extend(input.symbols.iter().map(|symbol| symbol.id.clone()));
    ids.extend(input.tests.iter().map(|test| test.id.clone()));
    ids.extend(input.dependency_edges.iter().map(|edge| edge.id.clone()));
    ids
}

fn result_source_ids(
    accepted_change_ids: &[Id],
    review_targets: &[PrReviewTarget],
    obstructions: &[PrReviewTargetObstruction],
    completion_candidates: &[CompletionCandidate],
) -> Vec<Id> {
    let mut ids = Vec::new();
    for id in accepted_change_ids {
        push_unique(&mut ids, id.clone());
    }
    for target in review_targets {
        push_unique(&mut ids, target.id.clone());
        for evidence_id in &target.evidence_ids {
            push_unique(&mut ids, evidence_id.clone());
        }
    }
    for obstruction in obstructions {
        push_unique(&mut ids, obstruction.id.clone());
        for source_id in &obstruction.source_ids {
            push_unique(&mut ids, source_id.clone());
        }
    }
    for candidate in completion_candidates {
        push_unique(&mut ids, candidate.id.clone());
        for source_id in &candidate.inferred_from {
            push_unique(&mut ids, source_id.clone());
        }
    }
    ids
}

fn ensure_ai_proposals_are_unreviewed(
    review_targets: &[PrReviewTarget],
    obstructions: &[PrReviewTargetObstruction],
    completion_candidates: &[CompletionCandidate],
) -> RuntimeResult<()> {
    if review_targets
        .iter()
        .any(|target| target.review_status != ReviewStatus::Unreviewed)
    {
        return Err(validation_error("review targets must remain unreviewed"));
    }
    if obstructions
        .iter()
        .any(|obstruction| obstruction.review_status != ReviewStatus::Unreviewed)
    {
        return Err(validation_error("obstructions must remain unreviewed"));
    }
    if completion_candidates
        .iter()
        .any(|candidate| candidate.review_status != ReviewStatus::Unreviewed)
    {
        return Err(validation_error(
            "completion candidates must remain unreviewed",
        ));
    }
    Ok(())
}

fn space_id(input: &PrReviewTargetInputDocument) -> RuntimeResult<Id> {
    id(format!("space:pr-review-target:{}", input.pull_request.id))
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

fn human_summary(result: &PrReviewTargetResult) -> String {
    match result.status {
        PrReviewTargetStatus::TargetsRecommended => format!(
            "Recommended {} unreviewed PR review targets with {} unresolved obstructions.",
            result.review_targets.len(),
            result.obstructions.len()
        ),
        PrReviewTargetStatus::NoTargets => {
            "No PR review targets were recommended from the bounded snapshot.".to_owned()
        }
        PrReviewTargetStatus::UnsupportedInput => {
            "The bounded snapshot could not be mapped into review targets.".to_owned()
        }
    }
}

fn nonempty_source_ids(input: &PrReviewTargetInputDocument, source_id: &Id) -> Vec<Id> {
    let mut ids = vec![source_id.clone()];
    push_unique(&mut ids, input.pull_request.id.clone());
    ids
}

fn target_source_ids(target: &PrReviewTarget) -> Vec<Id> {
    let mut ids = vec![target.id.clone()];
    for evidence_id in &target.evidence_ids {
        push_unique(&mut ids, evidence_id.clone());
    }
    ids
}

fn completion_candidate_source_ids(candidate: &CompletionCandidate) -> Vec<Id> {
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
    } else if value.starts_with("symbol:") {
        "symbol"
    } else if value.starts_with("signal:") {
        "risk_signal"
    } else if value.starts_with("target:") {
        "review_target"
    } else if value.starts_with("obstruction:") {
        "obstruction"
    } else if value.starts_with("candidate:") {
        "completion_candidate"
    } else if value.starts_with("dependency:") {
        "dependency_edge"
    } else {
        "source"
    }
}

fn push_unique(ids: &mut Vec<Id>, id: Id) {
    if !ids.contains(&id) {
        ids.push(id);
    }
}

fn id(value: impl Into<String>) -> RuntimeResult<Id> {
    Ok(Id::new(value)?)
}

fn validation_error(reason: impl Into<String>) -> RuntimeError {
    RuntimeError::workflow_construction(WORKFLOW_NAME, reason)
}
