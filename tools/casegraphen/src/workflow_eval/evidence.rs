use super::{
    dedupe_ids, sanitize, EvidenceBoundaryViolation, EvidenceBoundaryViolationType,
    EvidenceFinding, EvidenceFindingType, EvidenceFindings, ObstructionRecord, ObstructionType,
};
use crate::workflow_model::{
    EvidenceBoundary, EvidenceRecord, EvidenceType, WorkflowCaseGraph, WorkflowSeverity,
};
use higher_graphen_core::{Id, ReviewStatus};

#[derive(Default)]
struct EvidenceAccum {
    accepted_evidence_ids: Vec<Id>,
    source_backed_evidence_ids: Vec<Id>,
    inference_record_ids: Vec<Id>,
    unreviewed_inference_ids: Vec<Id>,
    promoted_evidence_ids: Vec<Id>,
    boundary_violations: Vec<EvidenceBoundaryViolation>,
    findings: Vec<EvidenceFinding>,
}

impl EvidenceAccum {
    fn finish(self) -> EvidenceFindings {
        EvidenceFindings {
            accepted_evidence_ids: dedupe_ids(self.accepted_evidence_ids),
            source_backed_evidence_ids: dedupe_ids(self.source_backed_evidence_ids),
            inference_record_ids: dedupe_ids(self.inference_record_ids),
            unreviewed_inference_ids: dedupe_ids(self.unreviewed_inference_ids),
            promoted_evidence_ids: dedupe_ids(self.promoted_evidence_ids),
            boundary_violations: self.boundary_violations,
            findings: self.findings,
        }
    }
}

pub(super) fn evidence_findings(
    graph: &WorkflowCaseGraph,
    obstructions: &[ObstructionRecord],
) -> EvidenceFindings {
    let mut accum = EvidenceAccum::default();
    for evidence in &graph.evidence_records {
        record_evidence(&mut accum, evidence);
    }
    record_missing_evidence(&mut accum, obstructions);
    accum.finish()
}

pub(super) fn acceptable_evidence(record: &EvidenceRecord) -> bool {
    !inference_record(record)
        && record.provenance.review_status != ReviewStatus::Rejected
        && matches!(
            record.evidence_boundary,
            EvidenceBoundary::AcceptedEvidence
                | EvidenceBoundary::SourceBackedEvidence
                | EvidenceBoundary::ReviewPromotion
        )
}

fn record_evidence(accum: &mut EvidenceAccum, evidence: &EvidenceRecord) {
    record_accepted_evidence(accum, evidence);
    record_source_backed_evidence(accum, evidence);
    record_inference_evidence(accum, evidence);
    record_review_promotion(accum, evidence);
    record_rejected_evidence(accum, evidence);
}

fn record_accepted_evidence(accum: &mut EvidenceAccum, evidence: &EvidenceRecord) {
    if evidence.provenance.review_status != ReviewStatus::Accepted
        && evidence.evidence_boundary != EvidenceBoundary::AcceptedEvidence
    {
        return;
    }
    accum.accepted_evidence_ids.push(evidence.id.clone());
    accum.findings.push(EvidenceFinding {
        id: Id::new(format!(
            "finding:{}:accepted",
            sanitize(evidence.id.as_str())
        ))
        .expect("generated evidence finding id"),
        finding_type: EvidenceFindingType::AcceptedEvidencePresent,
        evidence_ids: vec![evidence.id.clone()],
        summary: format!("{} is accepted or accepted-boundary evidence.", evidence.id),
        review_status: evidence.provenance.review_status,
    });
}

fn record_source_backed_evidence(accum: &mut EvidenceAccum, evidence: &EvidenceRecord) {
    if !matches!(
        evidence.evidence_boundary,
        EvidenceBoundary::SourceBackedEvidence | EvidenceBoundary::AcceptedEvidence
    ) {
        return;
    }
    accum.source_backed_evidence_ids.push(evidence.id.clone());
    if evidence.source_ids.is_empty() {
        accum.boundary_violations.push(EvidenceBoundaryViolation {
            id: Id::new(format!(
                "violation:{}:missing-source",
                sanitize(evidence.id.as_str())
            ))
            .expect("generated evidence violation id"),
            evidence_id: evidence.id.clone(),
            violation_type: EvidenceBoundaryViolationType::MissingSource,
            explanation: "Source-backed evidence must retain at least one source id.".to_owned(),
            severity: WorkflowSeverity::High,
        });
    } else if evidence.provenance.review_status != ReviewStatus::Accepted {
        accum.findings.push(EvidenceFinding {
            id: Id::new(format!(
                "finding:{}:source-backed-pending-review",
                sanitize(evidence.id.as_str())
            ))
            .expect("generated evidence finding id"),
            finding_type: EvidenceFindingType::SourceBackedPendingReview,
            evidence_ids: vec![evidence.id.clone()],
            summary: format!("{} is source-backed but not accepted.", evidence.id),
            review_status: evidence.provenance.review_status,
        });
    }
}

fn record_inference_evidence(accum: &mut EvidenceAccum, evidence: &EvidenceRecord) {
    if !inference_record(evidence) {
        return;
    }
    accum.inference_record_ids.push(evidence.id.clone());
    if evidence.provenance.review_status == ReviewStatus::Unreviewed {
        accum.unreviewed_inference_ids.push(evidence.id.clone());
    }
    accum.findings.push(EvidenceFinding {
        id: Id::new(format!(
            "finding:{}:inference-separated",
            sanitize(evidence.id.as_str())
        ))
        .expect("generated evidence finding id"),
        finding_type: EvidenceFindingType::InferenceSeparated,
        evidence_ids: vec![evidence.id.clone()],
        summary: format!(
            "{} is an inference record and is not treated as accepted evidence.",
            evidence.id
        ),
        review_status: evidence.provenance.review_status,
    });
}

fn record_review_promotion(accum: &mut EvidenceAccum, evidence: &EvidenceRecord) {
    if evidence.evidence_boundary != EvidenceBoundary::ReviewPromotion {
        return;
    }
    accum.promoted_evidence_ids.push(evidence.id.clone());
    if evidence.provenance.review_status == ReviewStatus::Accepted {
        return;
    }
    accum.findings.push(EvidenceFinding {
        id: Id::new(format!(
            "finding:{}:promotion-required",
            sanitize(evidence.id.as_str())
        ))
        .expect("generated evidence finding id"),
        finding_type: EvidenceFindingType::PromotionRequired,
        evidence_ids: vec![evidence.id.clone()],
        summary: format!("{} requires accepted review before promotion.", evidence.id),
        review_status: evidence.provenance.review_status,
    });
}

fn record_rejected_evidence(accum: &mut EvidenceAccum, evidence: &EvidenceRecord) {
    if evidence.provenance.review_status != ReviewStatus::Rejected {
        return;
    }
    accum.boundary_violations.push(EvidenceBoundaryViolation {
        id: Id::new(format!(
            "violation:{}:rejected-used",
            sanitize(evidence.id.as_str())
        ))
        .expect("generated evidence violation id"),
        evidence_id: evidence.id.clone(),
        violation_type: EvidenceBoundaryViolationType::RejectedEvidenceUsed,
        explanation: "Rejected evidence is present and must not satisfy readiness.".to_owned(),
        severity: WorkflowSeverity::High,
    });
}

fn record_missing_evidence(accum: &mut EvidenceAccum, obstructions: &[ObstructionRecord]) {
    for obstruction in obstructions
        .iter()
        .filter(|record| record.obstruction_type == ObstructionType::MissingEvidence)
    {
        accum.findings.push(EvidenceFinding {
            id: Id::new(format!(
                "finding:{}:evidence-missing",
                sanitize(obstruction.id.as_str())
            ))
            .expect("generated evidence finding id"),
            finding_type: EvidenceFindingType::EvidenceMissing,
            evidence_ids: obstruction.witness_ids.clone(),
            summary: obstruction.explanation.clone(),
            review_status: ReviewStatus::Unreviewed,
        });
    }
}

fn inference_record(record: &EvidenceRecord) -> bool {
    record.evidence_type == EvidenceType::AiInference
        || record.evidence_boundary == EvidenceBoundary::AiInference
        || record.provenance.source.kind == "agent_inference"
}
