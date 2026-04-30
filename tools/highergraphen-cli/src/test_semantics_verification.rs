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
    validate_schema(
        &request.interpretation,
        TEST_SEMANTICS_INTERPRETATION_SCHEMA,
        "interpretation",
    )?;
    validate_schema(
        &request.review,
        TEST_SEMANTICS_INTERPRETATION_REVIEW_SCHEMA,
        "review",
    )?;

    let review_record = request
        .review
        .get("result")
        .and_then(|value| value.get("review_record"))
        .ok_or_else(|| "review report needs result.review_record".to_owned())?;
    let review_request = review_record
        .get("request")
        .ok_or_else(|| "review report needs result.review_record.request".to_owned())?;
    let candidate_id = required_string(review_request, "candidate_id")?;
    let decision = required_string(review_request, "decision")?;
    let reviewer_id = required_string(review_request, "reviewer_id")?;

    let candidate = find_candidate(&request.interpretation, candidate_id)?;
    let source_ids = string_array(candidate.value.get("source_ids"));
    let target_ids = candidate_target_ids(&candidate.value);
    let matching_evidence = matching_passed_evidence(&request.interpretation, &source_ids);

    let review_gate_passed = decision == "accepted"
        && accepted_candidate_ids(&request.review)
            .iter()
            .any(|accepted| accepted == candidate_id);
    let evidence_gate_passed = !matching_evidence.is_empty();
    let binding_gate_passed = !target_ids.is_empty();
    let verified = review_gate_passed && evidence_gate_passed && binding_gate_passed;

    let fact_id = format!("fact:test-semantics:{}", slug(candidate_id));
    let coverage_id = format!("coverage:test-semantics:{}", slug(candidate_id));
    let proof_obligation_ids = target_ids
        .iter()
        .map(|target_id| format!("proof-obligation:test-semantics:{}", slug(target_id)))
        .collect::<Vec<_>>();
    let semantic_proof_input_ids = target_ids
        .iter()
        .map(|target_id| format!("semantic-proof-input:test-semantics:{}", slug(target_id)))
        .collect::<Vec<_>>();
    let verified_morphism_ids = target_ids
        .iter()
        .map(|target_id| {
            format!(
                "verified-morphism:test-semantics:{}:{}",
                slug(candidate_id),
                slug(target_id)
            )
        })
        .collect::<Vec<_>>();

    let accepted_fact_ids = if verified {
        vec![fact_id.clone()]
    } else {
        Vec::new()
    };
    let coverage_ids = if verified {
        vec![coverage_id.clone()]
    } else {
        Vec::new()
    };
    let emitted_proof_obligation_ids = if verified {
        proof_obligation_ids.clone()
    } else {
        Vec::new()
    };
    let emitted_semantic_proof_input_ids = if verified {
        semantic_proof_input_ids.clone()
    } else {
        Vec::new()
    };
    let emitted_verified_morphism_ids = if verified {
        verified_morphism_ids.clone()
    } else {
        Vec::new()
    };

    Ok(json!({
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
            "candidate_id": candidate_id,
            "candidate_kind": candidate.kind,
            "reviewer_id": reviewer_id,
            "test_run_path": request.test_run_path,
            "candidate": candidate.value
        },
        "result": {
            "status": if verified { "verified" } else { "not_verified" },
            "gates": [
                gate(
                    "gate:test-semantics:review",
                    "review",
                    review_gate_passed,
                    "Review report accepts the selected interpretation candidate.",
                    &[candidate_id.to_owned()],
                ),
                gate(
                    "gate:test-semantics:evidence",
                    "evidence",
                    evidence_gate_passed,
                    "Passed execution evidence links to the selected candidate source IDs.",
                    &source_ids,
                ),
                gate(
                    "gate:test-semantics:semantic-binding",
                    "semantic_binding",
                    binding_gate_passed,
                    "Candidate binds to at least one target law, morphism, or semantic role.",
                    &target_ids,
                )
            ],
            "verified_candidate_ids": if verified { vec![candidate_id.to_owned()] } else { Vec::<String>::new() },
            "unverified_candidate_ids": if verified { Vec::<String>::new() } else { vec![candidate_id.to_owned()] },
            "accepted_fact_ids": accepted_fact_ids,
            "coverage_ids": coverage_ids,
            "proof_obligation_ids": emitted_proof_obligation_ids,
            "semantic_proof_input_ids": emitted_semantic_proof_input_ids,
            "proof_object_ids": [],
            "verified_morphism_ids": emitted_verified_morphism_ids,
            "evidence_links": matching_evidence,
            "verification_cells": if verified {
                vec![json!({
                    "id": fact_id,
                    "cell_type": "verified_test_semantics_candidate",
                    "candidate_id": candidate_id,
                    "coverage_id": coverage_id,
                    "source_ids": source_ids,
                    "target_ids": target_ids,
                    "review_status": "accepted",
                    "confidence": 0.74
                })]
            } else {
                Vec::<Value>::new()
            }
        },
        "projection": {
            "audience": "ai_agent",
            "purpose": "test_semantics_verification",
            "summary": if verified {
                format!("Verified reviewed test semantics candidate {candidate_id} with passed execution evidence.")
            } else {
                format!("Reviewed test semantics candidate {candidate_id} was not verified.")
            },
            "recommended_actions": if verified {
                vec![
                    "Use semantic_proof_input_ids to enqueue formal proof input generation when a proof backend is required.",
                    "Keep proof_object_ids empty until a proof backend verifies the obligation."
                ]
            } else {
                vec![
                    "Add or attach passed execution evidence for the reviewed candidate.",
                    "Ensure the candidate declares candidate_target_ids or target_ids before proof input generation."
                ]
            },
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
        }
    }))
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
