use serde_json::{json, Value};

use crate::test_semantics_interpretation::TEST_SEMANTICS_INTERPRETATION_SCHEMA;

pub(crate) const TEST_SEMANTICS_INTERPRETATION_REVIEW_SCHEMA: &str =
    "highergraphen.test_semantics.interpretation_review.report.v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TestSemanticsReviewDecision {
    Accepted,
    Rejected,
}

impl TestSemanticsReviewDecision {
    pub(crate) fn command_action(self) -> &'static str {
        match self {
            Self::Accepted => "accept",
            Self::Rejected => "reject",
        }
    }

    fn review_status(self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ReviewRequest {
    pub(crate) interpretation: Value,
    pub(crate) decision: TestSemanticsReviewDecision,
    pub(crate) candidate_id: String,
    pub(crate) reviewer_id: String,
    pub(crate) reason: String,
}

pub(crate) fn review(request: ReviewRequest) -> Result<Value, String> {
    let schema = request
        .interpretation
        .get("schema")
        .and_then(Value::as_str)
        .ok_or_else(|| "interpretation document needs schema".to_owned())?;
    if schema != TEST_SEMANTICS_INTERPRETATION_SCHEMA {
        return Err(format!(
            "unsupported test semantics interpretation schema {schema}; expected {TEST_SEMANTICS_INTERPRETATION_SCHEMA}"
        ));
    }

    let source = request
        .interpretation
        .get("source")
        .and_then(Value::as_object)
        .ok_or_else(|| "interpretation document needs source object".to_owned())?;
    let input_schema = source
        .get("input_schema")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let interpreter = source
        .get("interpreter")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let source_review_status = source
        .get("review_status")
        .and_then(Value::as_str)
        .unwrap_or("unreviewed");

    let candidate = find_candidate(&request.interpretation, &request.candidate_id)?;
    let mut reviewed_candidate = candidate.value.clone();
    if let Some(object) = reviewed_candidate.as_object_mut() {
        object.insert(
            "review_status".to_owned(),
            Value::String(request.decision.review_status().to_owned()),
        );
    }

    let accepted_candidate_ids = match request.decision {
        TestSemanticsReviewDecision::Accepted => vec![request.candidate_id.clone()],
        TestSemanticsReviewDecision::Rejected => Vec::new(),
    };
    let rejected_candidate_ids = match request.decision {
        TestSemanticsReviewDecision::Accepted => Vec::new(),
        TestSemanticsReviewDecision::Rejected => vec![request.candidate_id.clone()],
    };

    let command = format!(
        "highergraphen test-semantics review {}",
        request.decision.command_action()
    );
    Ok(json!({
        "schema": TEST_SEMANTICS_INTERPRETATION_REVIEW_SCHEMA,
        "report_type": "test_semantics_interpretation_review",
        "report_version": 1,
        "metadata": {
            "command": command,
            "cli_package": "highergraphen-cli"
        },
        "scenario": {
            "source_interpretation": {
                "schema": schema,
                "input_schema": input_schema,
                "interpreter": interpreter,
                "review_status": source_review_status
            },
            "candidate_kind": candidate.kind,
            "candidate": candidate.value
        },
        "result": {
            "status": request.decision.review_status(),
            "review_record": {
                "request": {
                    "candidate_id": request.candidate_id,
                    "decision": request.decision.review_status(),
                    "reviewer_id": request.reviewer_id,
                    "reason": request.reason
                },
                "candidate_kind": candidate.kind,
                "candidate": candidate.value,
                "reviewed_candidate": reviewed_candidate,
                "outcome_review_status": request.decision.review_status()
            },
            "accepted_candidate_ids": accepted_candidate_ids,
            "rejected_candidate_ids": rejected_candidate_ids,
            "accepted_fact_ids": [],
            "coverage_ids": [],
            "proof_object_ids": []
        },
        "projection": {
            "audience": "ai_agent",
            "purpose": "test_semantics_interpretation_review",
            "summary": format!(
                "{} test semantics interpretation candidate {}.",
                capitalize(request.decision.review_status()),
                request.candidate_id
            ),
            "recommended_actions": [
                "Keep the review report with the source interpretation for auditability.",
                "Promote accepted candidates into coverage or proof only through a later verification workflow."
            ],
            "source_ids": [request.candidate_id],
            "information_loss": [
                {
                    "description": "Review records an explicit decision but does not mutate the source interpretation.",
                    "source_ids": [request.candidate_id]
                },
                {
                    "description": "Accepted interpretation candidates are not accepted facts, coverage, or proof objects.",
                    "source_ids": [request.candidate_id]
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

fn capitalize(value: &str) -> String {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) => first.to_ascii_uppercase().to_string() + chars.as_str(),
        None => String::new(),
    }
}
