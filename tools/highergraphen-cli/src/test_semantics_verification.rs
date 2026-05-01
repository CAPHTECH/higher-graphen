use serde_json::{json, Value};

use crate::{
    test_semantics_interpretation::TEST_SEMANTICS_INTERPRETATION_SCHEMA,
    test_semantics_review::TEST_SEMANTICS_INTERPRETATION_REVIEW_SCHEMA,
};

pub(crate) const TEST_SEMANTICS_VERIFICATION_REPORT_SCHEMA: &str =
    "highergraphen.test_semantics.verification.report.v1";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct VerifyRequest {
    pub(crate) interpretation: Value,
    pub(crate) review: Value,
    pub(crate) test_run_path: Option<String>,
}

pub(crate) fn verify(request: VerifyRequest) -> Result<Value, String> {
    validate_verify_request(&request)?;
    let review = review_fields(&request.review)?;
    let candidate = find_candidate(&request.interpretation, &review.candidate_id)?;
    let source_ids = string_array(candidate.value.get("source_ids"));
    let target_ids = candidate_target_ids(&candidate.value);
    let matching_evidence = matching_passed_evidence(&request.interpretation, &source_ids);

    let review_gate_passed = review.decision == "accepted"
        && accepted_candidate_ids(&request.review)
            .iter()
            .any(|accepted| accepted == &review.candidate_id);
    let evidence_gate_passed = !matching_evidence.is_empty();
    let binding_gate_passed = !target_ids.is_empty();
    let verified = review_gate_passed && evidence_gate_passed && binding_gate_passed;

    let ids = VerificationIds::new(&review.candidate_id, &target_ids);
    let gates = VerificationGates {
        review: review_gate_passed,
        evidence: evidence_gate_passed,
        binding: binding_gate_passed,
    };
    Ok(verification_report(VerificationReportInput {
        request,
        candidate,
        review,
        source_ids,
        target_ids,
        matching_evidence,
        ids,
        gates,
        verified,
    }))
}

struct ReviewFields {
    candidate_id: String,
    decision: String,
    reviewer_id: String,
}

struct VerificationGates {
    review: bool,
    evidence: bool,
    binding: bool,
}

#[derive(Clone)]
struct VerificationIds {
    fact_id: String,
    coverage_id: String,
    proof_obligation_ids: Vec<String>,
    semantic_proof_input_ids: Vec<String>,
    verified_morphism_ids: Vec<String>,
}

struct VerificationReportInput {
    request: VerifyRequest,
    candidate: Candidate,
    review: ReviewFields,
    source_ids: Vec<String>,
    target_ids: Vec<String>,
    matching_evidence: Vec<Value>,
    ids: VerificationIds,
    gates: VerificationGates,
    verified: bool,
}

impl VerificationIds {
    fn new(candidate_id: &str, target_ids: &[String]) -> Self {
        Self {
            fact_id: format!("fact:test-semantics:{}", slug(candidate_id)),
            coverage_id: format!("coverage:test-semantics:{}", slug(candidate_id)),
            proof_obligation_ids: prefixed_ids("proof-obligation:test-semantics", target_ids),
            semantic_proof_input_ids: prefixed_ids(
                "semantic-proof-input:test-semantics",
                target_ids,
            ),
            verified_morphism_ids: target_ids
                .iter()
                .map(|target_id| {
                    format!(
                        "verified-morphism:test-semantics:{}:{}",
                        slug(candidate_id),
                        slug(target_id)
                    )
                })
                .collect(),
        }
    }
}

fn validate_verify_request(request: &VerifyRequest) -> Result<(), String> {
    validate_schema(
        &request.interpretation,
        TEST_SEMANTICS_INTERPRETATION_SCHEMA,
        "interpretation",
    )?;
    validate_schema(
        &request.review,
        TEST_SEMANTICS_INTERPRETATION_REVIEW_SCHEMA,
        "review",
    )
}

fn review_fields(review: &Value) -> Result<ReviewFields, String> {
    let review_record = review
        .get("result")
        .and_then(|value| value.get("review_record"))
        .ok_or_else(|| "review report needs result.review_record".to_owned())?;
    let request = review_record
        .get("request")
        .ok_or_else(|| "review report needs result.review_record.request".to_owned())?;
    Ok(ReviewFields {
        candidate_id: required_string(request, "candidate_id")?.to_owned(),
        decision: required_string(request, "decision")?.to_owned(),
        reviewer_id: required_string(request, "reviewer_id")?.to_owned(),
    })
}

fn prefixed_ids(prefix: &str, target_ids: &[String]) -> Vec<String> {
    target_ids
        .iter()
        .map(|target_id| format!("{prefix}:{}", slug(target_id)))
        .collect()
}

fn verification_report(input: VerificationReportInput) -> Value {
    let result = verification_result(
        &input.review.candidate_id,
        input.source_ids.clone(),
        input.target_ids.clone(),
        input.matching_evidence,
        input.ids,
        input.gates,
        input.verified,
    );
    json!({
        "schema": TEST_SEMANTICS_VERIFICATION_REPORT_SCHEMA,
        "report_type": "test_semantics_verification",
        "report_version": 1,
        "metadata": {
            "command": "highergraphen test-semantics verify",
            "cli_package": "highergraphen-cli"
        },
        "scenario": {
            "interpretation_schema": TEST_SEMANTICS_INTERPRETATION_SCHEMA,
            "review_schema": TEST_SEMANTICS_INTERPRETATION_REVIEW_SCHEMA,
            "candidate_id": input.review.candidate_id,
            "candidate_kind": input.candidate.kind,
            "reviewer_id": input.review.reviewer_id,
            "test_run_path": input.request.test_run_path,
            "candidate": input.candidate.value
        },
        "result": result,
        "projection": verification_projection(input.verified, &input.review.candidate_id)
    })
}

fn verification_result(
    candidate_id: &str,
    source_ids: Vec<String>,
    target_ids: Vec<String>,
    matching_evidence: Vec<Value>,
    ids: VerificationIds,
    gates: VerificationGates,
    verified: bool,
) -> Value {
    json!({
        "status": if verified { "verified" } else { "not_verified" },
        "gates": verification_gates(candidate_id, &source_ids, &target_ids, &gates),
        "verified_candidate_ids": if verified { vec![candidate_id.to_owned()] } else { Vec::new() },
        "unverified_candidate_ids": if verified { Vec::new() } else { vec![candidate_id.to_owned()] },
        "accepted_fact_ids": if verified { vec![ids.fact_id.clone()] } else { Vec::new() },
        "coverage_ids": if verified { vec![ids.coverage_id.clone()] } else { Vec::new() },
        "proof_obligation_ids": if verified { ids.proof_obligation_ids.clone() } else { Vec::new() },
        "semantic_proof_input_ids": if verified { ids.semantic_proof_input_ids.clone() } else { Vec::new() },
        "proof_object_ids": Vec::<String>::new(),
        "verified_morphism_ids": if verified { ids.verified_morphism_ids.clone() } else { Vec::new() },
        "evidence_links": matching_evidence,
        "verification_cells": verification_cells(candidate_id, source_ids, target_ids, ids, verified)
    })
}

fn verification_gates(
    candidate_id: &str,
    source_ids: &[String],
    target_ids: &[String],
    gates: &VerificationGates,
) -> Vec<Value> {
    vec![
        gate(
            "gate:test-semantics:review",
            "review",
            gates.review,
            "Review report accepts the selected interpretation candidate.",
            &[candidate_id.to_owned()],
        ),
        gate(
            "gate:test-semantics:evidence",
            "evidence",
            gates.evidence,
            "Passed execution evidence links to the selected candidate source IDs.",
            source_ids,
        ),
        gate(
            "gate:test-semantics:semantic-binding",
            "semantic_binding",
            gates.binding,
            "Candidate binds to at least one target law, morphism, or semantic role.",
            target_ids,
        ),
    ]
}

fn verification_cells(
    candidate_id: &str,
    source_ids: Vec<String>,
    target_ids: Vec<String>,
    ids: VerificationIds,
    verified: bool,
) -> Vec<Value> {
    if !verified {
        return Vec::new();
    }
    vec![json!({
        "id": ids.fact_id,
        "cell_type": "verified_test_semantics_candidate",
        "candidate_id": candidate_id,
        "coverage_id": ids.coverage_id,
        "source_ids": source_ids,
        "target_ids": target_ids,
        "review_status": "accepted",
        "confidence": 0.74
    })]
}

fn verification_projection(verified: bool, candidate_id: &str) -> Value {
    json!({
        "audience": "ai_agent",
        "purpose": "test_semantics_verification",
        "summary": verification_summary(verified, candidate_id),
        "recommended_actions": verification_actions(verified),
        "source_ids": [candidate_id],
        "information_loss": [
            {
                "description": "Verification consumes bounded interpretation and review reports; it does not inspect full source bodies.",
                "source_ids": [candidate_id]
            },
            {
                "description": "This workflow creates proof obligations and semantic proof input IDs, not proof objects.",
                "source_ids": [candidate_id]
            }
        ]
    })
}

fn verification_summary(verified: bool, candidate_id: &str) -> String {
    if verified {
        format!(
            "Verified reviewed test semantics candidate {candidate_id} with passed execution evidence."
        )
    } else {
        format!("Reviewed test semantics candidate {candidate_id} was not verified.")
    }
}

fn verification_actions(verified: bool) -> Vec<&'static str> {
    if verified {
        vec![
            "Use semantic_proof_input_ids to enqueue formal proof input generation when a proof backend is required.",
            "Keep proof_object_ids empty until a proof backend verifies the obligation.",
        ]
    } else {
        vec![
            "Add or attach passed execution evidence for the reviewed candidate.",
            "Ensure the candidate declares candidate_target_ids or target_ids before proof input generation.",
        ]
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Candidate {
    kind: &'static str,
    value: Value,
}

fn validate_schema(value: &Value, expected: &str, label: &str) -> Result<(), String> {
    let schema = value
        .get("schema")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{label} document needs schema"))?;
    if schema != expected {
        return Err(format!(
            "unsupported {label} schema {schema}; expected {expected}"
        ));
    }
    Ok(())
}

fn required_string<'a>(value: &'a Value, field: &str) -> Result<&'a str, String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("review request needs {field}"))
}

fn find_candidate(interpretation: &Value, candidate_id: &str) -> Result<Candidate, String> {
    for (field, kind) in [
        ("interpreted_cells", "interpreted_cell"),
        ("interpreted_morphisms", "interpreted_morphism"),
        ("candidate_laws", "candidate_law"),
        ("binding_candidates", "binding_candidate"),
        ("evidence_links", "evidence_link"),
    ] {
        for value in interpretation
            .get(field)
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            if value.get("id").and_then(Value::as_str) == Some(candidate_id) {
                return Ok(Candidate {
                    kind,
                    value: value.clone(),
                });
            }
        }
    }

    Err(format!(
        "candidate {candidate_id} was not found in test semantics interpretation"
    ))
}

fn accepted_candidate_ids(review: &Value) -> Vec<String> {
    review
        .get("result")
        .and_then(|value| value.get("accepted_candidate_ids"))
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn candidate_target_ids(candidate: &Value) -> Vec<String> {
    let mut target_ids = string_array(candidate.get("candidate_target_ids"));
    target_ids.extend(string_array(candidate.get("target_ids")));
    target_ids.sort();
    target_ids.dedup();
    target_ids
}

fn matching_passed_evidence(interpretation: &Value, source_ids: &[String]) -> Vec<Value> {
    interpretation
        .get("evidence_links")
        .and_then(Value::as_array)
        .map(|links| {
            links
                .iter()
                .filter(|link| link.get("status").and_then(Value::as_str) == Some("passed"))
                .filter(|link| {
                    let target_id = link
                        .get("target_id")
                        .and_then(Value::as_str)
                        .unwrap_or_default();
                    source_ids
                        .iter()
                        .any(|source_id| evidence_target_matches_source(target_id, source_id))
                })
                .cloned()
                .collect()
        })
        .unwrap_or_default()
}

fn evidence_target_matches_source(target_id: &str, source_id: &str) -> bool {
    let target = comparable_id(target_id);
    let source = comparable_id(source_id);
    !target.is_empty()
        && !source.is_empty()
        && (target.contains(&source) || source.contains(&target))
}

fn comparable_id(value: &str) -> String {
    value
        .trim_start_matches("rust-test:function-ref:")
        .trim_start_matches("test:function-ref:")
        .trim_start_matches("rust-test:function:")
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .map(|character| character.to_ascii_lowercase())
        .collect()
}

fn string_array(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn gate(id: &str, gate_type: &str, passed: bool, summary: &str, source_ids: &[String]) -> Value {
    json!({
        "id": id,
        "gate_type": gate_type,
        "status": if passed { "passed" } else { "failed" },
        "summary": summary,
        "source_ids": source_ids
    })
}

fn slug(value: &str) -> String {
    let mut slug = String::new();
    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character.to_ascii_lowercase());
        } else if !slug.ends_with('-') {
            slug.push('-');
        }
    }
    slug.trim_matches('-').to_owned()
}
