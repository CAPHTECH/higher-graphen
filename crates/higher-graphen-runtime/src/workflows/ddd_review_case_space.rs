use super::*;
use serde_json::{json, Value};

/// Converts a native CaseGraphen CaseSpace JSON document into the bounded DDD review input contract.
pub fn ddd_input_from_case_space(case_space: Value, case_space_path: &str) -> RuntimeResult<Value> {
    require_schema(&case_space, CASE_SPACE_SCHEMA)?;
    let case_space_id = string_at(&case_space, &["case_space_id"])
        .unwrap_or_else(|| "case_space:ddd-review".to_owned());
    let space_id = format!("space:ddd-review:{}", id_tail(&case_space_id));
    let source_boundary = source_boundary_from_case_space(&case_space, case_space_path);
    let source_boundary_id = string_at(&source_boundary, &["id"])
        .unwrap_or_else(|| format!("source_boundary:{}", id_tail(&case_space_id)));
    let adapter_ids = string_array_at(&source_boundary, &["adapters"]);
    let mut accepted_facts = accepted_facts_from_case_space(&case_space);
    accepted_facts.extend(accepted_relations_from_case_space(&case_space));
    let constraints = records_by_type(&case_space, &["custom:constraint"]);
    let reviews = records_by_type(&case_space, &["review"]);
    let mut inferred_claims = inferred_claims_from_case_space(&case_space);
    inferred_claims.extend(inferred_relations_from_case_space(&case_space));
    let completion_hints = completion_hints_from_case_space(&case_space);

    Ok(json!({
        "schema": INPUT_SCHEMA,
        "source": {
            "kind": "case_space",
            "uri": case_space_path,
            "title": format!("DDD review CaseSpace {}", case_space_id),
            "confidence": 1.0,
            "adapters": adapter_ids
        },
        "review_subject": {
            "id": case_space_id,
            "subject_type": "case_space",
            "title": "DDD review from CaseSpace",
            "description": "Review bounded-context and domain-model risks from a native CaseGraphen CaseSpace."
        },
        "source_boundary": source_boundary,
        "lift_morphism": {
            "id": format!("morphism:lift-{}", id_tail(&case_space_id)),
            "morphism_type": "source_to_ddd_review_space",
            "source_boundary_id": source_boundary_id,
            "source_schema": CASE_SPACE_SCHEMA,
            "target_space_id": space_id,
            "adapter_ids": adapter_ids,
            "preserved_ids": ids_from_records(&accepted_facts)
                .into_iter()
                .chain(ids_from_records(&constraints))
                .chain(ids_from_records(&reviews))
                .chain(ids_from_records(&completion_hints))
                .collect::<Vec<_>>(),
            "inferred_ids": ids_from_records(&inferred_claims),
            "information_loss": [
                "Native CaseSpace records are summarized into DDD review records.",
                "Store replay, historical morphisms, and external implementation evidence are not read."
            ]
        },
        "operation_gate": {
            "actor_id": "actor:highergraphen-cli",
            "operation": "ddd_review",
            "operation_scope_id": space_id,
            "audience": "audit",
            "capability_ids": ["capability:highergraphen-cli:ddd-review"],
            "policy_ids": ["policy:ddd-review-source-boundary"],
            "source_boundary_id": source_boundary_id
        },
        "accepted_facts": accepted_facts,
        "constraints": constraints,
        "reviews": reviews,
        "inferred_claims": inferred_claims,
        "completion_hints": completion_hints,
        "projection_requests": projection_requests_from_case_space(&case_space)
    }))
}

fn source_boundary_from_case_space(case_space: &Value, case_space_path: &str) -> Value {
    if let Some(boundary) = case_space
        .get("metadata")
        .and_then(|metadata| metadata.get("source_boundary"))
    {
        return normalize_source_boundary(boundary, case_space_path);
    }
    let case_space_id = string_at(case_space, &["case_space_id"])
        .unwrap_or_else(|| "case_space:ddd-review".to_owned());
    json!({
        "id": format!("source_boundary:{}", id_tail(&case_space_id)),
        "input_paths": [case_space_path],
        "included_sources": [case_space_path],
        "excluded_sources": [
            "CaseGraphen store replay",
            "full MorphismLog history",
            "source code",
            "ADRs",
            "tickets",
            "tests"
        ],
        "adapters": ["casegraphen_case_space.v1"],
        "accepted_fact_boundaries": ["source_backed", "adapter_supplied"],
        "inference_boundaries": ["ai_inference", "unreviewed_note"],
        "omitted_material": [
            "No CaseGraphen store replay or MorphismLog history was read.",
            "Full workshop notes, ADRs, source code, tickets, and tests were not read."
        ],
        "information_loss": [
            "Native CaseSpace records are summarized into a bounded DDD review snapshot."
        ]
    })
}

fn normalize_source_boundary(boundary: &Value, case_space_path: &str) -> Value {
    let id =
        string_at(boundary, &["id"]).unwrap_or_else(|| "source_boundary:ddd-review".to_owned());
    let included_sources = string_array_at(boundary, &["included_sources"]);
    let excluded_sources = string_array_at(boundary, &["excluded_sources"]);
    let adapters = {
        let values = string_array_at(boundary, &["adapters"]);
        if values.is_empty() {
            vec!["casegraphen_case_space.v1".to_owned()]
        } else {
            values
        }
    };
    json!({
        "id": id,
        "input_paths": [case_space_path],
        "included_sources": if included_sources.is_empty() { vec![case_space_path.to_owned()] } else { included_sources },
        "excluded_sources": excluded_sources,
        "adapters": adapters,
        "accepted_fact_boundaries": ["source_backed", "adapter_supplied"],
        "inference_boundaries": ["ai_inference", "unreviewed_note"],
        "omitted_material": string_array_at(boundary, &["information_loss"]),
        "information_loss": string_array_at(boundary, &["information_loss"])
    })
}

fn accepted_facts_from_case_space(case_space: &Value) -> Vec<Value> {
    cells(case_space)
        .into_iter()
        .filter(|cell| !is_inferred(cell) && !is_completion(cell))
        .filter(|cell| {
            !matches!(
                string_at(cell, &["cell_type"]).as_deref(),
                Some("custom:constraint" | "review")
            )
        })
        .filter_map(ddd_record_from_cell)
        .collect()
}

fn records_by_type(case_space: &Value, cell_types: &[&str]) -> Vec<Value> {
    cells(case_space)
        .into_iter()
        .filter(|cell| {
            cell.get("cell_type")
                .and_then(Value::as_str)
                .is_some_and(|cell_type| cell_types.contains(&cell_type))
        })
        .filter_map(ddd_record_from_cell)
        .collect()
}

fn accepted_relations_from_case_space(case_space: &Value) -> Vec<Value> {
    relations(case_space)
        .into_iter()
        .filter(|relation| !is_inferred_relation(relation))
        .filter_map(ddd_record_from_relation)
        .collect()
}

fn inferred_relations_from_case_space(case_space: &Value) -> Vec<Value> {
    relations(case_space)
        .into_iter()
        .filter(|relation| is_inferred_relation(relation))
        .map(|relation| {
            json!({
                "id": string_at(relation, &["id"]).unwrap_or_else(|| "inference:ddd-relation".to_owned()),
                "claim_type": claim_type_for_relation(relation),
                "label": string_at(relation, &["relation_type"]).unwrap_or_else(|| "DDD inferred relation".to_owned()),
                "source_ids": string_array_at(relation, &["source_ids"]),
                "target_ids": relation_endpoint_ids(relation),
                "confidence": number_at(relation, &["provenance", "confidence"]).unwrap_or(0.6),
                "review_status": "unreviewed",
                "evidence_boundary": "ai_inference"
            })
        })
        .collect()
}

fn inferred_claims_from_case_space(case_space: &Value) -> Vec<Value> {
    cells(case_space)
        .into_iter()
        .filter(|cell| is_inferred(cell))
        .map(|cell| {
            json!({
                "id": string_at(cell, &["id"]).unwrap_or_else(|| "inference:ddd".to_owned()),
                "claim_type": claim_type_for_cell(cell),
                "label": string_at(cell, &["title"]).unwrap_or_else(|| "DDD inferred claim".to_owned()),
                "source_ids": string_array_at(cell, &["source_ids"]),
                "target_ids": string_array_at(cell, &["structure_ids"]),
                "confidence": number_at(cell, &["provenance", "confidence"]).unwrap_or(0.6),
                "review_status": "unreviewed",
                "evidence_boundary": evidence_boundary(cell).unwrap_or("ai_inference".to_owned())
            })
        })
        .collect()
}

fn completion_hints_from_case_space(case_space: &Value) -> Vec<Value> {
    cells(case_space)
        .into_iter()
        .filter(|cell| is_completion(cell))
        .map(|cell| {
            json!({
                "id": string_at(cell, &["id"]).unwrap_or_else(|| "completion:ddd".to_owned()),
                "candidate_type": "boundary_mapping",
                "target_ids": string_array_at(cell, &["structure_ids"]),
                "source_ids": string_array_at(cell, &["source_ids"]),
                "suggested_change": {
                    "summary": string_at(cell, &["summary"]).unwrap_or_else(|| "Review the DDD completion candidate.".to_owned())
                },
                "confidence": number_at(cell, &["provenance", "confidence"]).unwrap_or(0.7),
                "severity": string_at(cell, &["metadata", "severity"]).unwrap_or_else(|| "high".to_owned()),
                "review_status": "unreviewed"
            })
        })
        .collect()
}

fn projection_requests_from_case_space(case_space: &Value) -> Vec<Value> {
    case_space
        .get("projections")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|projection| {
            json!({
                "id": string_at(projection, &["projection_id"]).unwrap_or_else(|| "projection:ddd".to_owned()),
                "view": match string_at(projection, &["audience"]).as_deref() {
                    Some("audit") => "audit_trace",
                    Some("ai_agent") => "ai_view",
                    Some("human_review") => "human_review",
                    _ => "implementation_view"
                },
                "focus_ids": string_array_at(projection, &["represented_cell_ids"])
            })
        })
        .collect()
}
