//! Bounded semantic proof certificate verification workflow.

use crate::error::{RuntimeError, RuntimeResult};
use crate::reports::{
    AiProjectionRecord, AiProjectionRecordType, AuditProjectionView, HumanReviewProjectionView,
    ProjectionAudience, ProjectionPurpose, ProjectionTrace, ProjectionViewSet, ReportEnvelope,
    ReportMetadata,
};
use crate::semantic_proof_reports::{
    SemanticProofCertificate, SemanticProofCounterexample, SemanticProofInputDocument,
    SemanticProofIssue, SemanticProofObject, SemanticProofReport, SemanticProofResult,
    SemanticProofScenario, SemanticProofStatus, SEMANTIC_PROOF_INPUT_SCHEMA,
    SEMANTIC_PROOF_REPORT_SCHEMA,
};
use higher_graphen_core::{Confidence, Id, ReviewStatus, Severity};
use higher_graphen_projection::InformationLoss;
use std::collections::BTreeSet;

const REPORT_TYPE: &str = "semantic_proof";
const REPORT_VERSION: u32 = 1;

/// Verifies a bounded semantic proof certificate bundle.
pub fn run_semantic_proof_verify(
    input: SemanticProofInputDocument,
) -> RuntimeResult<SemanticProofReport> {
    validate_input_schema(&input)?;
    validate_references(&input)?;

    let accepted_certificates = accepted_certificates(&input);
    let rejected_certificate_ids = rejected_certificate_ids(&input, &accepted_certificates);
    let accepted_counterexamples = accepted_counterexamples(&input);
    let proof_objects = proof_objects(&input, &accepted_certificates)?;
    let issues = issues(&input, &accepted_certificates, &accepted_counterexamples)?;
    let counterexamples = accepted_counterexamples.clone();
    let status = if !counterexamples.is_empty() {
        SemanticProofStatus::CounterexampleFound
    } else if issues.is_empty() {
        SemanticProofStatus::Proved
    } else {
        SemanticProofStatus::InsufficientProof
    };
    let accepted_certificate_ids = accepted_certificates
        .iter()
        .map(|certificate| certificate.id.clone())
        .collect::<Vec<_>>();
    let source_ids = result_source_ids(
        &input,
        &accepted_certificate_ids,
        &rejected_certificate_ids,
        &proof_objects,
        &counterexamples,
        &issues,
    );
    let scenario = SemanticProofScenario {
        input_schema: input.schema.clone(),
        source: input.source.clone(),
        theorem: input.theorem.clone(),
        semantic_cells: input.semantic_cells.clone(),
        morphisms: input.morphisms.clone(),
        laws: input.laws.clone(),
        proof_certificates: input.proof_certificates.clone(),
        counterexamples: input.counterexamples.clone(),
        verification_policy: input.verification_policy.clone(),
    };
    let result = SemanticProofResult {
        status,
        accepted_certificate_ids,
        rejected_certificate_ids,
        proof_objects,
        counterexamples,
        issues,
        source_ids,
    };
    let projection = projection(&scenario, &result)?;

    Ok(ReportEnvelope {
        schema: SEMANTIC_PROOF_REPORT_SCHEMA.to_owned(),
        report_type: REPORT_TYPE.to_owned(),
        report_version: REPORT_VERSION,
        metadata: ReportMetadata::semantic_proof_verify(),
        scenario,
        result,
        projection,
    })
}

fn validate_input_schema(input: &SemanticProofInputDocument) -> RuntimeResult<()> {
    if input.schema == SEMANTIC_PROOF_INPUT_SCHEMA {
        return Ok(());
    }
    Err(RuntimeError::unsupported_input_schema(
        input.schema.clone(),
        SEMANTIC_PROOF_INPUT_SCHEMA,
    ))
}

fn validate_references(input: &SemanticProofInputDocument) -> RuntimeResult<()> {
    let cell_ids = input
        .semantic_cells
        .iter()
        .map(|cell| cell.id.clone())
        .collect::<BTreeSet<_>>();
    let law_ids = input
        .laws
        .iter()
        .map(|law| law.id.clone())
        .collect::<BTreeSet<_>>();
    let morphism_ids = input
        .morphisms
        .iter()
        .map(|morphism| morphism.id.clone())
        .collect::<BTreeSet<_>>();
    ensure_known_ids(&law_ids, &input.theorem.law_ids, "theorem", "law")?;
    ensure_known_ids(
        &morphism_ids,
        &input.theorem.morphism_ids,
        "theorem",
        "morphism",
    )?;
    for morphism in &input.morphisms {
        ensure_known_ids(&cell_ids, &morphism.source_ids, "morphism", "source cell")?;
        ensure_known_ids(&cell_ids, &morphism.target_ids, "morphism", "target cell")?;
        ensure_known_ids(&law_ids, &morphism.law_ids, "morphism", "law")?;
    }
    for law in &input.laws {
        ensure_known_ids(
            &morphism_ids,
            &law.applies_to_ids,
            "law",
            "applies-to morphism",
        )?;
    }
    for certificate in &input.proof_certificates {
        ensure_known_id(
            &input.theorem.id,
            &certificate.theorem_id,
            "certificate theorem",
        )?;
        ensure_known_ids(&law_ids, &certificate.law_ids, "certificate", "law")?;
        ensure_known_ids(
            &morphism_ids,
            &certificate.morphism_ids,
            "certificate",
            "morphism",
        )?;
        ensure_known_witnesses(input, &certificate.witness_ids, "certificate")?;
    }
    for counterexample in &input.counterexamples {
        ensure_known_id(
            &input.theorem.id,
            &counterexample.theorem_id,
            "counterexample theorem",
        )?;
        ensure_known_ids(&law_ids, &counterexample.law_ids, "counterexample", "law")?;
        ensure_known_ids(
            &morphism_ids,
            &counterexample.morphism_ids,
            "counterexample",
            "morphism",
        )?;
        ensure_known_witnesses(input, &counterexample.path_ids, "counterexample")?;
    }
    Ok(())
}

fn ensure_known_id(expected: &Id, actual: &Id, context: &str) -> RuntimeResult<()> {
    if expected == actual {
        return Ok(());
    }
    Err(validation_error(format!(
        "{context} references unknown theorem {actual}"
    )))
}

fn ensure_known_ids(
    known: &BTreeSet<Id>,
    ids: &[Id],
    context: &str,
    target: &str,
) -> RuntimeResult<()> {
    for id in ids {
        if !known.contains(id) {
            return Err(validation_error(format!(
                "{context} references unknown {target} {id}"
            )));
        }
    }
    Ok(())
}

fn ensure_known_witnesses(
    input: &SemanticProofInputDocument,
    ids: &[Id],
    context: &str,
) -> RuntimeResult<()> {
    let mut known = BTreeSet::new();
    known.insert(input.theorem.id.clone());
    known.extend(input.semantic_cells.iter().map(|cell| cell.id.clone()));
    known.extend(input.morphisms.iter().map(|morphism| morphism.id.clone()));
    known.extend(input.laws.iter().map(|law| law.id.clone()));
    known.extend(
        input
            .proof_certificates
            .iter()
            .map(|certificate| certificate.id.clone()),
    );
    for id in ids {
        if !known.contains(id) {
            return Err(validation_error(format!(
                "{context} references unknown witness {id}"
            )));
        }
    }
    Ok(())
}

fn accepted_certificates(input: &SemanticProofInputDocument) -> Vec<&SemanticProofCertificate> {
    input
        .proof_certificates
        .iter()
        .filter(|certificate| certificate_is_accepted(input, certificate))
        .collect()
}

fn certificate_is_accepted(
    input: &SemanticProofInputDocument,
    certificate: &SemanticProofCertificate,
) -> bool {
    if input.verification_policy.require_accepted_review
        && certificate.review_status != ReviewStatus::Accepted
    {
        return false;
    }
    if input.verification_policy.require_input_hash && certificate.input_hash.is_none() {
        return false;
    }
    if input.verification_policy.require_proof_hash && certificate.proof_hash.is_none() {
        return false;
    }
    input.verification_policy.accepted_backends.is_empty()
        || input
            .verification_policy
            .accepted_backends
            .iter()
            .any(|backend| backend == &certificate.backend)
}

fn rejected_certificate_ids(
    input: &SemanticProofInputDocument,
    accepted: &[&SemanticProofCertificate],
) -> Vec<Id> {
    input
        .proof_certificates
        .iter()
        .filter(|certificate| {
            !accepted
                .iter()
                .any(|accepted| accepted.id == certificate.id)
        })
        .map(|certificate| certificate.id.clone())
        .collect()
}

fn accepted_counterexamples(
    input: &SemanticProofInputDocument,
) -> Vec<SemanticProofCounterexample> {
    input
        .counterexamples
        .iter()
        .filter(|counterexample| counterexample_is_accepted(input, counterexample))
        .cloned()
        .collect()
}

fn counterexample_is_accepted(
    input: &SemanticProofInputDocument,
    counterexample: &SemanticProofCounterexample,
) -> bool {
    !input
        .verification_policy
        .require_accepted_counterexample_review
        || counterexample.review_status == ReviewStatus::Accepted
}

fn proof_objects(
    input: &SemanticProofInputDocument,
    accepted: &[&SemanticProofCertificate],
) -> RuntimeResult<Vec<SemanticProofObject>> {
    accepted
        .iter()
        .map(|certificate| {
            Ok(SemanticProofObject {
                id: id(format!("proof:semantic:{}", slug(certificate.id.as_str())))?,
                theorem_ids: vec![input.theorem.id.clone()],
                law_ids: certificate.law_ids.clone(),
                morphism_ids: certificate.morphism_ids.clone(),
                certificate_ids: vec![certificate.id.clone()],
                witness_ids: certificate.witness_ids.clone(),
                summary: format!(
                    "Certificate {} proves {} with backend {} {}.",
                    certificate.id,
                    input.theorem.summary,
                    certificate.backend,
                    certificate.backend_version
                ),
                confidence: certificate.confidence,
                review_status: ReviewStatus::Accepted,
            })
        })
        .collect()
}

fn issues(
    input: &SemanticProofInputDocument,
    accepted: &[&SemanticProofCertificate],
    accepted_counterexamples: &[SemanticProofCounterexample],
) -> RuntimeResult<Vec<SemanticProofIssue>> {
    let mut issues = Vec::new();
    let proved_laws = proved_laws(accepted);
    let proved_morphisms = proved_morphisms(accepted);

    issues.extend(missing_law_proof_issues(input, &proved_laws)?);
    issues.extend(missing_morphism_proof_issues(input, &proved_morphisms)?);
    issues.extend(rejected_certificate_issues(input)?);
    issues.extend(unaccepted_counterexample_issues(
        input,
        accepted_counterexamples,
    )?);

    Ok(issues)
}

fn proved_laws(accepted: &[&SemanticProofCertificate]) -> BTreeSet<Id> {
    accepted
        .iter()
        .flat_map(|certificate| certificate.law_ids.iter().cloned())
        .collect()
}

fn proved_morphisms(accepted: &[&SemanticProofCertificate]) -> BTreeSet<Id> {
    accepted
        .iter()
        .flat_map(|certificate| certificate.morphism_ids.iter().cloned())
        .collect()
}

fn missing_law_proof_issues(
    input: &SemanticProofInputDocument,
    proved_laws: &BTreeSet<Id>,
) -> RuntimeResult<Vec<SemanticProofIssue>> {
    input
        .theorem
        .law_ids
        .iter()
        .filter(|law_id| !proved_laws.contains(*law_id))
        .map(|law_id| {
            issue(
                format!(
                    "issue:semantic-proof:missing-law-proof:{}",
                    slug(law_id.as_str())
                ),
                "missing_law_proof",
                vec![law_id.clone()],
                format!("No accepted proof certificate covers law {law_id}."),
                Severity::High,
            )
        })
        .collect()
}

fn missing_morphism_proof_issues(
    input: &SemanticProofInputDocument,
    proved_morphisms: &BTreeSet<Id>,
) -> RuntimeResult<Vec<SemanticProofIssue>> {
    input
        .theorem
        .morphism_ids
        .iter()
        .filter(|morphism_id| !proved_morphisms.contains(*morphism_id))
        .map(|morphism_id| {
            issue(
                format!(
                    "issue:semantic-proof:missing-morphism-proof:{}",
                    slug(morphism_id.as_str())
                ),
                "missing_morphism_proof",
                vec![morphism_id.clone()],
                format!("No accepted proof certificate covers morphism {morphism_id}."),
                Severity::High,
            )
        })
        .collect()
}

fn rejected_certificate_issues(
    input: &SemanticProofInputDocument,
) -> RuntimeResult<Vec<SemanticProofIssue>> {
    let mut issues = Vec::new();
    for certificate in &input.proof_certificates {
        if !certificate_is_accepted(input, certificate) {
            issues.push(issue(
                format!(
                    "issue:semantic-proof:rejected-certificate:{}",
                    slug(certificate.id.as_str())
                ),
                "certificate_not_accepted_by_policy",
                vec![certificate.id.clone()],
                format!(
                    "Certificate {} does not satisfy verification policy.",
                    certificate.id
                ),
                Severity::Medium,
            )?);
        }
    }
    Ok(issues)
}

fn unaccepted_counterexample_issues(
    input: &SemanticProofInputDocument,
    accepted_counterexamples: &[SemanticProofCounterexample],
) -> RuntimeResult<Vec<SemanticProofIssue>> {
    let mut issues = Vec::new();
    for counterexample in &input.counterexamples {
        if !accepted_counterexamples
            .iter()
            .any(|accepted| accepted.id == counterexample.id)
        {
            issues.push(issue(
                format!(
                    "issue:semantic-proof:counterexample-not-accepted:{}",
                    slug(counterexample.id.as_str())
                ),
                "counterexample_not_accepted_by_policy",
                vec![counterexample.id.clone()],
                format!(
                    "Counterexample {} does not satisfy verification policy.",
                    counterexample.id
                ),
                Severity::Medium,
            )?);
        }
    }
    Ok(issues)
}

fn issue(
    id_value: String,
    issue_type: &str,
    target_ids: Vec<Id>,
    summary: String,
    severity: Severity,
) -> RuntimeResult<SemanticProofIssue> {
    Ok(SemanticProofIssue {
        id: id(id_value)?,
        issue_type: issue_type.to_owned(),
        target_ids,
        summary,
        severity,
        confidence: Confidence::new(0.9)?,
        review_status: ReviewStatus::Unreviewed,
    })
}

fn result_source_ids(
    input: &SemanticProofInputDocument,
    accepted_certificate_ids: &[Id],
    rejected_certificate_ids: &[Id],
    proof_objects: &[SemanticProofObject],
    counterexamples: &[SemanticProofCounterexample],
    issues: &[SemanticProofIssue],
) -> Vec<Id> {
    let mut ids = Vec::new();
    push_unique(&mut ids, input.theorem.id.clone());
    for id in input
        .theorem
        .law_ids
        .iter()
        .chain(input.theorem.morphism_ids.iter())
    {
        push_unique(&mut ids, id.clone());
    }
    for id in accepted_certificate_ids
        .iter()
        .chain(rejected_certificate_ids.iter())
    {
        push_unique(&mut ids, id.clone());
    }
    for proof in proof_objects {
        push_unique(&mut ids, proof.id.clone());
        for id in &proof.witness_ids {
            push_unique(&mut ids, id.clone());
        }
    }
    for counterexample in counterexamples {
        push_unique(&mut ids, counterexample.id.clone());
        for id in &counterexample.path_ids {
            push_unique(&mut ids, id.clone());
        }
    }
    for issue in issues {
        push_unique(&mut ids, issue.id.clone());
        for id in &issue.target_ids {
            push_unique(&mut ids, id.clone());
        }
    }
    ids
}

fn projection(
    scenario: &SemanticProofScenario,
    result: &SemanticProofResult,
) -> RuntimeResult<ProjectionViewSet> {
    let source_ids = result.source_ids.clone();
    let loss = InformationLoss::declared(
        "Semantic proof verification validates HG proof certificates and policy references; external backend proof checking, MIR extraction, and SMT/model-check execution occur before this bounded input.",
        source_ids.clone(),
    )?;
    let human_review = HumanReviewProjectionView {
        audience: ProjectionAudience::Human,
        purpose: ProjectionPurpose::TestGapDetection,
        summary: format!(
            "Semantic proof status {:?} with {} proof objects, {} counterexamples, and {} issues.",
            result.status,
            result.proof_objects.len(),
            result.counterexamples.len(),
            result.issues.len()
        ),
        recommended_actions: recommended_actions(result),
        source_ids: source_ids.clone(),
        information_loss: vec![loss.clone()],
    };
    let ai_view = crate::reports::AiProjectionView {
        audience: ProjectionAudience::AiAgent,
        purpose: ProjectionPurpose::TestGapDetection,
        records: ai_records(scenario, result),
        source_ids: source_ids.clone(),
        information_loss: vec![loss.clone()],
    };
    let audit_trace = AuditProjectionView {
        audience: ProjectionAudience::Audit,
        purpose: ProjectionPurpose::AuditTrace,
        source_ids,
        information_loss: vec![loss.clone()],
        traces: result
            .source_ids
            .iter()
            .map(|source_id| ProjectionTrace {
                source_id: source_id.clone(),
                role: "semantic_proof_source".to_owned(),
                represented_in: vec![
                    "human_review".to_owned(),
                    "ai_view".to_owned(),
                    "audit_trace".to_owned(),
                ],
            })
            .collect(),
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

fn recommended_actions(result: &SemanticProofResult) -> Vec<String> {
    match result.status {
        SemanticProofStatus::Proved => vec![
            "Use accepted proof_objects as formal verification cells for downstream HG workflows."
                .to_owned(),
        ],
        SemanticProofStatus::CounterexampleFound => {
            vec!["Inspect counterexample path_ids before accepting the semantic change.".to_owned()]
        }
        SemanticProofStatus::InsufficientProof => vec![
            "Run or attach accepted proof certificates for every theorem law and morphism."
                .to_owned(),
        ],
    }
}

fn ai_records(
    scenario: &SemanticProofScenario,
    result: &SemanticProofResult,
) -> Vec<AiProjectionRecord> {
    let mut records = Vec::new();
    records.push(AiProjectionRecord {
        id: scenario.theorem.id.clone(),
        record_type: AiProjectionRecordType::CheckResult,
        summary: scenario.theorem.summary.clone(),
        source_ids: vec![scenario.theorem.id.clone()],
        confidence: Some(scenario.source.confidence),
        review_status: Some(ReviewStatus::Accepted),
        severity: None,
        provenance: None,
    });
    for proof in &result.proof_objects {
        records.push(AiProjectionRecord {
            id: proof.id.clone(),
            record_type: AiProjectionRecordType::CheckResult,
            summary: proof.summary.clone(),
            source_ids: proof.certificate_ids.clone(),
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
            severity: Some(counterexample.severity),
            provenance: None,
        });
    }
    for issue in &result.issues {
        records.push(AiProjectionRecord {
            id: issue.id.clone(),
            record_type: AiProjectionRecordType::Obstruction,
            summary: issue.summary.clone(),
            source_ids: issue.target_ids.clone(),
            confidence: Some(issue.confidence),
            review_status: Some(issue.review_status),
            severity: Some(issue.severity),
            provenance: None,
        });
    }
    records
}

fn validation_error(reason: impl Into<String>) -> RuntimeError {
    RuntimeError::workflow_construction("semantic_proof", reason)
}

fn id(value: impl Into<String>) -> RuntimeResult<Id> {
    Ok(Id::new(value.into())?)
}

fn slug(value: impl AsRef<str>) -> String {
    value
        .as_ref()
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn push_unique(ids: &mut Vec<Id>, id: Id) {
    if !ids.contains(&id) {
        ids.push(id);
    }
}
