//! Bounded DDD review workflow.

use crate::{RuntimeError, RuntimeResult};
use serde_json::{json, Map, Value};

#[path = "ddd_review_case_space.rs"]
mod ddd_review_case_space;
#[path = "ddd_review_json.rs"]
mod ddd_review_json;
pub use ddd_review_case_space::ddd_input_from_case_space;
use ddd_review_json::*;

const INPUT_SCHEMA: &str = "highergraphen.ddd_review.input.v1";
const REPORT_SCHEMA: &str = "highergraphen.ddd_review.report.v1";
const CASE_SPACE_SCHEMA: &str = "highergraphen.case.space.v1";

/// Runs the bounded DDD review workflow and emits the stable DDD review report contract.
pub fn run_ddd_review(input: Value) -> RuntimeResult<Value> {
    require_schema(&input, INPUT_SCHEMA)?;
    require_object(&input, "source_boundary")?;
    require_object(&input, "lift_morphism")?;
    require_object(&input, "operation_gate")?;
    require_source_boundary_consistency(&input)?;

    let data = ddd_review_data(&input);
    Ok(ddd_review_report(input, data))
}

struct DddReviewData {
    accepted_facts: Vec<Value>,
    constraints: Vec<Value>,
    reviews: Vec<Value>,
    inferred_claims: Vec<Value>,
    completion_hints: Vec<Value>,
    space_id: String,
    interpretation_mappings: Vec<Value>,
    interpretation_mapping_ids: Vec<String>,
    obstructions: Vec<Value>,
    completion_candidates: Vec<Value>,
    completion_morphisms: Vec<Value>,
    projection_loss: Vec<Value>,
    review_gaps: Vec<Value>,
    blocker_ids: Vec<String>,
}

fn ddd_review_data(input: &Value) -> DddReviewData {
    let accepted_facts = array_at(input, &["accepted_facts"]);
    let constraints = array_at(input, &["constraints"]);
    let reviews = array_at(input, &["reviews"]);
    let inferred_claims = array_at(input, &["inferred_claims"]);
    let completion_hints = array_at(input, &["completion_hints"]);
    let review_subject_id = string_at(input, &["review_subject", "id"])
        .unwrap_or_else(|| "review_subject:ddd".to_owned());
    let space_id = string_at(input, &["lift_morphism", "target_space_id"])
        .unwrap_or_else(|| format!("space:ddd-review:{}", id_tail(&review_subject_id)));
    let interpretation_mappings = interpretation_mappings(input);
    let interpretation_mapping_ids = ids_from_records(&interpretation_mappings);
    let obstructions = obstructions_from_input(input);
    let completion_candidates = completion_candidates_from_input(&completion_hints, &obstructions);
    let completion_morphisms = completion_morphisms_from_candidates(&completion_candidates);
    let projection_loss = projection_loss_from_input(input);
    let review_gaps = review_gaps_from_input(&reviews);
    let mut blocker_ids = ids_from_records(&obstructions);
    blocker_ids.extend(ids_from_records(&review_gaps));

    DddReviewData {
        accepted_facts,
        constraints,
        reviews,
        inferred_claims,
        completion_hints,
        space_id,
        interpretation_mappings,
        interpretation_mapping_ids,
        obstructions,
        completion_candidates,
        completion_morphisms,
        projection_loss,
        review_gaps,
        blocker_ids,
    }
}

fn ddd_review_report(input: Value, data: DddReviewData) -> Value {
    json!({
        "schema": REPORT_SCHEMA,
        "report_type": "ddd_review",
        "report_version": 1,
        "metadata": {
            "command": "highergraphen ddd review",
            "runtime_package": "higher-graphen-runtime",
            "runtime_crate": "higher_graphen_runtime",
            "cli_package": "highergraphen-cli"
        },
        "scenario": {
            "input_schema": INPUT_SCHEMA,
            "review_subject": input["review_subject"].clone(),
            "source_boundary": input["source_boundary"].clone(),
            "lift_morphism": input["lift_morphism"].clone(),
            "operation_gate": input["operation_gate"].clone(),
            "accepted_facts": compact_records(&data.accepted_facts, "record_type"),
            "constraints": compact_records(&data.constraints, "record_type"),
            "reviews": compact_records(&data.reviews, "record_type"),
            "inferred_claims": compact_records(&data.inferred_claims, "claim_type"),
            "completion_hints": compact_records(&data.completion_hints, "candidate_type"),
            "interpretation_mappings": data.interpretation_mappings,
            "lifted_structure": {
                "space_id": data.space_id,
                "cell_ids": ids_from_values(
                    data.accepted_facts.iter()
                        .chain(&data.constraints)
                        .chain(&data.reviews)
                        .chain(&data.inferred_claims)
                        .chain(&data.completion_hints)
                        .collect::<Vec<_>>()
                ),
                "incidence_ids": ["incidence:ddd-review-source-support"],
                "context_ids": context_ids_from_records(&data.accepted_facts),
                "invariant_ids": invariant_ids(),
                "interpretation_mapping_ids": data.interpretation_mapping_ids,
                "morphism_summary_ids": [input["lift_morphism"]["id"].clone()]
            }
        },
        "result": {
            "status": if data.obstructions.is_empty() && data.review_gaps.is_empty() && data.projection_loss.is_empty() {
                "no_issues_in_snapshot"
            } else {
                "issues_detected"
            },
            "accepted_fact_ids": ids_from_values(
                data.accepted_facts.iter().chain(&data.constraints).chain(&data.reviews).collect::<Vec<_>>()
            ),
            "inferred_claim_ids": ids_from_records(&data.inferred_claims),
            "evaluated_invariant_ids": invariant_ids(),
            "interpretation_mapping_ids": ids_from_records(&data.interpretation_mappings),
            "obstructions": data.obstructions,
            "completion_candidates": data.completion_candidates,
            "completion_morphisms": data.completion_morphisms,
            "evidence_boundaries": evidence_boundaries_from_input(&data.accepted_facts, &data.inferred_claims),
            "projection_loss": data.projection_loss,
            "review_gaps": data.review_gaps,
            "closeability": {
                "closeable": data.blocker_ids.is_empty(),
                "blocker_ids": data.blocker_ids,
                "required_actions": required_actions()
            },
            "source_ids": source_ids_from_input(&input)
        },
        "projection": projection_from_input(&input)
    })
}

fn require_schema(value: &Value, expected: &str) -> RuntimeResult<()> {
    let schema = value
        .get("schema")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if schema == expected {
        Ok(())
    } else {
        Err(RuntimeError::unsupported_input_schema(schema, expected))
    }
}

fn require_object(value: &Value, field: &str) -> RuntimeResult<()> {
    if value.get(field).and_then(Value::as_object).is_some() {
        Ok(())
    } else {
        Err(RuntimeError::workflow_construction(
            "ddd_review",
            format!("missing required object {field}"),
        ))
    }
}

fn require_source_boundary_consistency(input: &Value) -> RuntimeResult<()> {
    let source_boundary_id = string_at(input, &["source_boundary", "id"]).unwrap_or_default();
    let lift_boundary_id =
        string_at(input, &["lift_morphism", "source_boundary_id"]).unwrap_or_default();
    let gate_boundary_id =
        string_at(input, &["operation_gate", "source_boundary_id"]).unwrap_or_default();
    if source_boundary_id == lift_boundary_id && source_boundary_id == gate_boundary_id {
        Ok(())
    } else {
        Err(RuntimeError::workflow_construction(
            "ddd_review",
            "source_boundary.id must match lift_morphism.source_boundary_id and operation_gate.source_boundary_id",
        ))
    }
}

fn ddd_record_from_cell(cell: &Value) -> Option<Value> {
    let id = string_at(cell, &["id"])?;
    let record_type = record_type_for_cell(cell)?;
    Some(json!({
        "id": id,
        "record_type": record_type,
        "label": string_at(cell, &["title"]).unwrap_or_else(|| id_tail(&id)),
        "context_ids": context_ids_for_cell(cell),
        "source_ids": string_array_at(cell, &["source_ids"]),
        "target_ids": string_array_at(cell, &["structure_ids"]),
        "properties": properties_for_cell(cell),
        "confidence": number_at(cell, &["provenance", "confidence"]).unwrap_or(1.0),
        "review_status": "accepted",
        "evidence_boundary": if record_type == "review" { "accepted_review" } else { "source_backed" }
    }))
}

fn ddd_record_from_relation(relation: &Value) -> Option<Value> {
    let id = string_at(relation, &["id"])?;
    let relation_type =
        string_at(relation, &["relation_type"]).unwrap_or_else(|| "relation".to_owned());
    Some(json!({
        "id": id,
        "record_type": "relation",
        "label": relation_type,
        "context_ids": context_ids_for_relation(relation),
        "source_ids": string_array_at(relation, &["source_ids"]),
        "target_ids": relation_endpoint_ids(relation),
        "properties": {
            "relation_type": string_at(relation, &["relation_type"]).unwrap_or_else(|| "relation".to_owned()),
            "relation_strength": string_at(relation, &["relation_strength"]).unwrap_or_else(|| "diagnostic".to_owned()),
            "from_id": string_at(relation, &["from_id"]).unwrap_or_else(|| "unknown".to_owned()),
            "to_id": string_at(relation, &["to_id"]).unwrap_or_else(|| "unknown".to_owned()),
            "evidence_ids": string_array_at(relation, &["evidence_ids"])
        },
        "confidence": number_at(relation, &["provenance", "confidence"]).unwrap_or(1.0),
        "review_status": "accepted",
        "evidence_boundary": "source_backed"
    }))
}

fn record_type_for_cell(cell: &Value) -> Option<&'static str> {
    match string_at(cell, &["cell_type"]).as_deref()? {
        "custom:context" => Some("bounded_context"),
        "custom:entity" => Some("entity"),
        "custom:constraint" => Some("constraint"),
        "custom:semantic_case" => Some("boundary"),
        "decision" => Some("decision"),
        "evidence" => Some("evidence"),
        "review" => Some("review"),
        _ => None,
    }
}

fn claim_type_for_cell(cell: &Value) -> &'static str {
    let id = string_at(cell, &["id"]).unwrap_or_default();
    if id.contains("equivalence") {
        "equivalence_proof"
    } else if id.contains("mapping") {
        "missing_mapping"
    } else if id.contains("projection") {
        "projection_loss"
    } else {
        "boundary_risk"
    }
}

fn claim_type_for_relation(relation: &Value) -> &'static str {
    match string_at(relation, &["relation_type"]).as_deref() {
        Some("blocks") => "boundary_risk",
        Some("requires_evidence") => "review_gap",
        Some("translates" | "maps_to") => "missing_mapping",
        _ => "boundary_risk",
    }
}

fn is_completion(cell: &Value) -> bool {
    string_at(cell, &["id"]).is_some_and(|id| id.starts_with("completion:"))
        || string_at(cell, &["cell_type"]).as_deref() == Some("completion")
}

fn is_inferred(cell: &Value) -> bool {
    evidence_boundary(cell)
        .is_some_and(|boundary| boundary == "inferred" || boundary == "ai_inference")
        || string_at(cell, &["provenance", "source", "kind"]).as_deref() == Some("ai")
        || string_at(cell, &["provenance", "review_status"]).as_deref() == Some("unreviewed")
}

fn evidence_boundary(cell: &Value) -> Option<String> {
    string_at(cell, &["metadata", "evidence_boundary"]).map(|value| {
        if value == "inferred" {
            "ai_inference".to_owned()
        } else {
            value
        }
    })
}

fn is_inferred_relation(relation: &Value) -> bool {
    string_at(relation, &["provenance", "source", "kind"]).as_deref() == Some("ai")
        || string_at(relation, &["provenance", "review_status"]).as_deref() == Some("unreviewed")
}

fn properties_for_cell(cell: &Value) -> Value {
    let mut properties = Map::new();
    if let Some(summary) = string_at(cell, &["summary"]) {
        properties.insert("summary".to_owned(), Value::String(summary));
    }
    if let Some(metadata) = cell.get("metadata").and_then(Value::as_object) {
        for (key, value) in metadata {
            properties.insert(key.clone(), value.clone());
        }
    }
    Value::Object(properties)
}

fn context_ids_for_cell(cell: &Value) -> Vec<String> {
    string_at(cell, &["metadata", "context_id"])
        .map(|context_id| vec![context_id])
        .unwrap_or_else(|| {
            string_array_at(cell, &["structure_ids"])
                .into_iter()
                .filter(|id| id.starts_with("context:"))
                .collect()
        })
}

fn context_ids_for_relation(relation: &Value) -> Vec<String> {
    let mut ids = string_array_at(relation, &["metadata", "context_ids"]);
    ids.extend(
        relation_endpoint_ids(relation)
            .into_iter()
            .filter(|id| id.starts_with("context:")),
    );
    ids.sort();
    ids.dedup();
    ids
}

fn interpretation_mappings(input: &Value) -> Vec<Value> {
    let source_ids = source_ids_from_input(input);
    vec![
        json!({
            "id": "mapping:ddd-bounded-context-to-context-cell",
            "domain_type": "bounded_context",
            "highergraphen_target": "context",
            "source_ids": source_ids,
            "review_status": "accepted"
        }),
        json!({
            "id": "mapping:ddd-boundary-risk-to-obstruction",
            "domain_type": "boundary_issue",
            "highergraphen_target": "obstruction",
            "source_ids": ids_from_values(array_at(input, &["accepted_facts"]).iter().collect()),
            "review_status": "accepted"
        }),
        json!({
            "id": "mapping:ddd-acl-gap-to-completion",
            "domain_type": "anti_corruption_mapping_gap",
            "highergraphen_target": "completion_candidate",
            "source_ids": ids_from_records(&array_at(input, &["completion_hints"])),
            "review_status": "unreviewed"
        }),
        json!({
            "id": "mapping:ddd-relation-to-incidence",
            "domain_type": "ddd_relation",
            "highergraphen_target": "incidence",
            "source_ids": ids_from_values(array_at(input, &["accepted_facts"]).iter().collect()),
            "review_status": "accepted"
        }),
    ]
}

fn obstructions_from_input(input: &Value) -> Vec<Value> {
    let mut obstructions = Vec::new();
    let boundary_ids: Vec<_> = array_at(input, &["accepted_facts"])
        .into_iter()
        .filter(|record| string_at(record, &["record_type"]).as_deref() == Some("boundary"))
        .filter_map(|record| string_at(&record, &["id"]))
        .collect();
    if !boundary_ids.is_empty() {
        obstructions.push(json!({
            "id": "obstruction:customer-boundary-semantic-loss",
            "obstruction_type": "boundary_semantic_loss",
            "title": "Shared model may drop context-specific semantics",
            "target_ids": boundary_ids,
            "witness": {"source": "accepted boundary record"},
            "invariant_ids": ["invariant:context-language-preserved", "invariant:cross-context-identity-not-collapsed"],
            "evidence_ids": ids_from_values(array_at(input, &["accepted_facts"]).iter().collect()),
            "severity": "high",
            "confidence": 0.9,
            "review_status": "unreviewed"
        }));
    }
    let inferred_boundary_risk_ids: Vec<_> = array_at(input, &["inferred_claims"])
        .into_iter()
        .filter(|claim| string_at(claim, &["claim_type"]).as_deref() == Some("boundary_risk"))
        .filter_map(|claim| string_at(&claim, &["id"]))
        .collect();
    if !inferred_boundary_risk_ids.is_empty() {
        obstructions.push(json!({
            "id": "obstruction:ddd-inferred-boundary-risk-unaccepted",
            "obstruction_type": "boundary_semantic_loss",
            "title": "Boundary risk is inferred but not accepted",
            "target_ids": inferred_boundary_risk_ids,
            "witness": {"source": "unreviewed boundary risk claim"},
            "invariant_ids": ["invariant:context-language-preserved", "invariant:inference-not-accepted-evidence"],
            "evidence_ids": ids_from_records(&array_at(input, &["inferred_claims"])),
            "severity": "high",
            "confidence": 0.85,
            "review_status": "unreviewed"
        }));
    }
    if let Some(obstruction) = language_conflict_obstruction(input) {
        obstructions.push(obstruction);
    }
    if let Some(obstruction) = missing_mapping_obstruction(input) {
        obstructions.push(obstruction);
    }
    if let Some(obstruction) = ownership_evidence_obstruction(input) {
        obstructions.push(obstruction);
    }
    if !array_at(input, &["inferred_claims"]).is_empty() {
        obstructions.push(json!({
            "id": "obstruction:customer-equivalence-evidence-unaccepted",
            "obstruction_type": "missing_evidence",
            "title": "Inferred claims cannot satisfy accepted DDD evidence",
            "target_ids": ids_from_records(&array_at(input, &["inferred_claims"])),
            "witness": {"required_evidence": "accepted reviewed evidence"},
            "invariant_ids": ["invariant:inference-not-accepted-evidence"],
            "evidence_ids": ids_from_records(&array_at(input, &["inferred_claims"])),
            "severity": "high",
            "confidence": 0.9,
            "review_status": "unreviewed"
        }));
    }
    if !review_gaps_from_input(&array_at(input, &["reviews"])).is_empty() {
        obstructions.push(json!({
            "id": "obstruction:domain-model-review-required",
            "obstruction_type": "review_required",
            "title": "Domain model acceptance review is not accepted",
            "target_ids": ids_from_records(&array_at(input, &["reviews"])),
            "witness": {"required_status": "accepted"},
            "invariant_ids": ["invariant:review-gates-satisfied-before-close"],
            "evidence_ids": ids_from_records(&array_at(input, &["reviews"])),
            "severity": "high",
            "confidence": 1.0,
            "review_status": "unreviewed"
        }));
    }
    obstructions
}

fn language_conflict_obstruction(input: &Value) -> Option<Value> {
    let entities: Vec<_> = array_at(input, &["accepted_facts"])
        .into_iter()
        .filter(|record| {
            matches!(
                string_at(record, &["record_type"]).as_deref(),
                Some("entity" | "aggregate" | "value_object")
            )
        })
        .collect();
    let mut conflict_ids = Vec::new();
    for left in &entities {
        for right in &entities {
            let left_id = string_at(left, &["id"])?;
            let right_id = string_at(right, &["id"])?;
            if left_id >= right_id {
                continue;
            }
            let left_contexts = string_array_at(left, &["context_ids"]);
            let right_contexts = string_array_at(right, &["context_ids"]);
            if left_contexts.is_empty()
                || right_contexts.is_empty()
                || left_contexts == right_contexts
            {
                continue;
            }
            if domain_term(left) == domain_term(right) {
                conflict_ids.push(left_id);
                conflict_ids.push(right_id);
            }
        }
    }
    conflict_ids.sort();
    conflict_ids.dedup();
    if conflict_ids.is_empty() {
        return None;
    }
    Some(json!({
        "id": "obstruction:ddd-cross-context-language-conflict",
        "obstruction_type": "cross_context_identity_collapse",
        "title": "Same domain term appears in multiple bounded contexts",
        "target_ids": conflict_ids,
        "witness": {"pattern": "same normalized entity term with different context_ids"},
        "invariant_ids": ["invariant:cross-context-identity-not-collapsed", "invariant:context-language-preserved"],
        "evidence_ids": ids_from_values(array_at(input, &["accepted_facts"]).iter().collect()),
        "severity": "high",
        "confidence": 0.8,
        "review_status": "unreviewed"
    }))
}

fn missing_mapping_obstruction(input: &Value) -> Option<Value> {
    let hint_ids: Vec<_> = array_at(input, &["completion_hints"])
        .into_iter()
        .filter(|hint| string_at(hint, &["candidate_type"]).as_deref() == Some("boundary_mapping"))
        .filter_map(|hint| string_at(&hint, &["id"]))
        .collect();
    let claim_ids: Vec<_> = array_at(input, &["inferred_claims"])
        .into_iter()
        .filter(|claim| string_at(claim, &["claim_type"]).as_deref() == Some("missing_mapping"))
        .filter_map(|claim| string_at(&claim, &["id"]))
        .collect();
    let mut target_ids = hint_ids;
    target_ids.extend(claim_ids);
    target_ids.sort();
    target_ids.dedup();
    if target_ids.is_empty() {
        return None;
    }
    Some(json!({
        "id": "obstruction:ddd-boundary-mapping-missing",
        "obstruction_type": "missing_boundary_mapping",
        "title": "Cross-context model requires an explicit boundary mapping",
        "target_ids": target_ids,
        "witness": {"required_mapping": "anti-corruption or translation rule"},
        "invariant_ids": ["invariant:boundary-translation-explicit"],
        "evidence_ids": source_ids_from_input(input),
        "severity": "high",
        "confidence": 0.88,
        "review_status": "unreviewed"
    }))
}

fn ownership_evidence_obstruction(input: &Value) -> Option<Value> {
    let target_ids: Vec<_> = array_at(input, &["accepted_facts"])
        .into_iter()
        .filter(|record| {
            matches!(
                string_at(record, &["record_type"]).as_deref(),
                Some(
                    "aggregate"
                        | "api"
                        | "database"
                        | "service"
                        | "team"
                        | "domain_event"
                        | "external_message"
                )
            )
        })
        .filter(|record| string_array_at(record, &["context_ids"]).is_empty())
        .filter_map(|record| string_at(&record, &["id"]))
        .collect();
    if target_ids.is_empty() {
        return None;
    }
    Some(json!({
        "id": "obstruction:ddd-ownership-evidence-missing",
        "obstruction_type": "missing_evidence",
        "title": "DDD structural elements need explicit context ownership evidence",
        "target_ids": target_ids,
        "witness": {"missing": "context_ids or ownership mapping"},
        "invariant_ids": ["invariant:context-ownership-explicit"],
        "evidence_ids": source_ids_from_input(input),
        "severity": "medium",
        "confidence": 0.75,
        "review_status": "unreviewed"
    }))
}

fn completion_candidates_from_input(
    completion_hints: &[Value],
    obstructions: &[Value],
) -> Vec<Value> {
    completion_hints
        .iter()
        .map(|hint| {
            json!({
                "id": string_at(hint, &["id"]).unwrap_or_else(|| "completion:ddd".to_owned()),
                "candidate_type": string_at(hint, &["candidate_type"]).unwrap_or_else(|| "boundary_mapping".to_owned()),
                "target_ids": string_array_at(hint, &["target_ids"]),
                "obstruction_ids": ids_from_records(obstructions),
                "suggested_change": hint.get("suggested_change").cloned().unwrap_or_else(|| json!({"summary": "Review DDD completion candidate."})),
                "rationale": "The bounded DDD review found a reviewable structural gap.",
                "provenance": {"source_ids": string_array_at(hint, &["source_ids"]), "extraction_method": "ddd_review.v1"},
                "severity": string_at(hint, &["severity"]).unwrap_or_else(|| "high".to_owned()),
                "confidence": number_at(hint, &["confidence"]).unwrap_or(0.7),
                "review_status": "unreviewed"
            })
        })
        .collect()
}

fn completion_morphisms_from_candidates(candidates: &[Value]) -> Vec<Value> {
    candidates
        .iter()
        .map(|candidate| {
            let candidate_id =
                string_at(candidate, &["id"]).unwrap_or_else(|| "completion:ddd".to_owned());
            json!({
                "id": format!("morphism:complete-{}", id_tail(&candidate_id)),
                "morphism_type": "completion_candidate_to_casegraphen_patch",
                "completion_candidate_id": candidate_id,
                "source_ids": string_array_at(candidate, &["provenance", "source_ids"]),
                "target_ids": string_array_at(candidate, &["target_ids"]),
                "operation": {
                    "op": "upsert_ontology_record",
                    "record_kind": completion_record_kind(candidate),
                    "review_required": true
                },
                "review_status": "unreviewed"
            })
        })
        .collect()
}

fn completion_record_kind(candidate: &Value) -> &'static str {
    match string_at(candidate, &["candidate_type"]).as_deref() {
        Some("domain_review") => "review",
        Some("evidence_request") => "evidence",
        Some("model_split") => "boundary",
        Some("constraint_update") => "constraint",
        _ => "transformation",
    }
}

fn evidence_boundaries_from_input(
    accepted_facts: &[Value],
    inferred_claims: &[Value],
) -> Vec<Value> {
    accepted_facts
        .iter()
        .filter(|record| string_at(record, &["record_type"]).as_deref() == Some("evidence"))
        .map(|record| {
            json!({
                "id": format!("evidence-boundary:{}", id_tail(&string_at(record, &["id"]).unwrap_or_else(|| "evidence".to_owned()))),
                "boundary_type": string_at(record, &["evidence_boundary"]).unwrap_or_else(|| "source_backed".to_owned()),
                "source_ids": [string_at(record, &["id"]).unwrap_or_else(|| "evidence:ddd".to_owned())],
                "accepted": true,
                "review_status": "accepted"
            })
        })
        .chain(inferred_claims.iter().map(|claim| {
            json!({
                "id": format!("evidence-boundary:{}", id_tail(&string_at(claim, &["id"]).unwrap_or_else(|| "inference".to_owned()))),
                "boundary_type": string_at(claim, &["evidence_boundary"]).unwrap_or_else(|| "ai_inference".to_owned()),
                "source_ids": [string_at(claim, &["id"]).unwrap_or_else(|| "inference:ddd".to_owned())],
                "accepted": false,
                "review_status": "unreviewed"
            })
        }))
        .collect()
}

fn projection_loss_from_input(input: &Value) -> Vec<Value> {
    let implementation_projection =
        array_at(input, &["projection_requests"])
            .into_iter()
            .find(|projection| {
                string_at(projection, &["view"]).as_deref() == Some("implementation_view")
            });
    implementation_projection
        .map(|projection| {
            vec![json!({
                "projection_id": string_at(&projection, &["id"]).unwrap_or_else(|| "projection:implementation-view".to_owned()),
                "omitted_ids": ids_from_values(
                    array_at(input, &["accepted_facts"])
                        .iter()
                        .chain(&array_at(input, &["inferred_claims"]))
                        .collect()
                ),
                "loss_type": "boundary_semantics_hidden",
                "summary": "Implementation view may hide DDD boundary risk, evidence, or review state.",
                "review_status": "unreviewed"
            })]
        })
        .unwrap_or_default()
}

fn review_gaps_from_input(reviews: &[Value]) -> Vec<Value> {
    reviews
        .iter()
        .filter(|review| {
            string_at(review, &["properties", "current_status"]).as_deref() != Some("accepted")
        })
        .map(|review| {
            json!({
                "id": format!("review-gap:{}", id_tail(&string_at(review, &["id"]).unwrap_or_else(|| "domain-review".to_owned()))),
                "gap_type": "unaccepted_review",
                "target_ids": string_array_at(review, &["target_ids"]),
                "required_review": string_at(review, &["properties", "required_status"]).unwrap_or_else(|| "accepted domain model review".to_owned()),
                "current_status": string_at(review, &["properties", "current_status"]).unwrap_or_else(|| "unreviewed".to_owned()),
                "review_status": "unreviewed"
            })
        })
        .collect()
}

fn projection_from_input(input: &Value) -> Value {
    let source_ids = source_ids_from_input(input);
    let audit_represented_ids = audit_represented_ids(input);
    json!({
        "human_review": {
            "summary": "DDD review reports bounded-context, evidence, review, and projection-loss risks.",
            "recommended_actions": required_actions(),
            "represented_ids": ids_from_records(&obstructions_from_input(input)),
            "source_ids": source_ids,
            "information_loss": ["Summarizes the bounded DDD review input for human review."]
        },
        "ai_view": {
            "summary": "Preserves stable DDD review IDs for agent follow-up.",
            "represented_ids": source_ids_from_input(input),
            "source_ids": [string_at(input, &["review_subject", "id"]).unwrap_or_else(|| "review_subject:ddd".to_owned())],
            "information_loss": ["Does not read omitted source material beyond the bounded input."]
        },
        "audit_trace": {
            "summary": "Preserves source boundary, lift morphism, operation gate, and interpretation mappings.",
            "represented_ids": audit_represented_ids,
            "source_ids": source_ids_from_input(input),
            "information_loss": ["Records omitted material and projection loss instead of treating absent data as safe."]
        }
    })
}

fn audit_represented_ids(input: &Value) -> Vec<String> {
    let mut ids = vec![
        string_at(input, &["source_boundary", "id"])
            .unwrap_or_else(|| "source_boundary:ddd-review".to_owned()),
        string_at(input, &["lift_morphism", "id"])
            .unwrap_or_else(|| "morphism:lift-ddd-review".to_owned()),
        string_at(input, &["operation_gate", "operation_scope_id"])
            .unwrap_or_else(|| "operation_scope:ddd-review".to_owned()),
    ];
    ids.extend(ids_from_records(&interpretation_mappings(input)));
    ids
}
