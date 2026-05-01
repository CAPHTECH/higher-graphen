use std::collections::BTreeSet;

use serde_json::{json, Value};

use crate::test_semantics_verification::TEST_SEMANTICS_VERIFICATION_REPORT_SCHEMA;

pub(crate) const TEST_SEMANTICS_EXPECTED_OBLIGATIONS_SCHEMA: &str =
    "highergraphen.test_semantics.expected_obligations.input.v1";
pub(crate) const TEST_SEMANTICS_GAP_REPORT_SCHEMA: &str =
    "highergraphen.test_semantics.gap.report.v1";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GapRequest {
    pub(crate) expected: Value,
    pub(crate) verified_reports: Vec<Value>,
}

pub(crate) fn detect(request: GapRequest) -> Result<Value, String> {
    validate_schema(
        &request.expected,
        TEST_SEMANTICS_EXPECTED_OBLIGATIONS_SCHEMA,
        "expected obligations",
    )?;
    if request.verified_reports.is_empty() {
        return Err("at least one verified report is required".to_owned());
    }

    let expected_obligations = expected_obligations(&request.expected)?;
    let mut covered_ids = BTreeSet::new();
    let mut verified_report_ids = Vec::new();
    for report in &request.verified_reports {
        validate_schema(
            report,
            TEST_SEMANTICS_VERIFICATION_REPORT_SCHEMA,
            "verified report",
        )?;
        let report_covered_ids = covered_ids_from_verified_report(report);
        covered_ids.extend(report_covered_ids);
        if let Some(candidate_id) = report
            .get("scenario")
            .and_then(|scenario| scenario.get("candidate_id"))
            .and_then(Value::as_str)
        {
            verified_report_ids.push(candidate_id.to_owned());
        }
    }

    let mut covered_obligations = Vec::new();
    let mut missing_obligations = Vec::new();
    for obligation in expected_obligations {
        if obligation.is_covered_by(&covered_ids) {
            covered_obligations.push(obligation.to_value("covered"));
        } else {
            missing_obligations.push(obligation.to_value("missing"));
        }
    }

    let obstructions = missing_obligations
        .iter()
        .map(obstruction_for_obligation)
        .collect::<Vec<_>>();
    let completion_candidates = missing_obligations
        .iter()
        .map(completion_candidate_for_obligation)
        .collect::<Vec<_>>();
    let status = if missing_obligations.is_empty() {
        "no_gaps_detected"
    } else {
        "gaps_detected"
    };

    Ok(json!({
        "schema": TEST_SEMANTICS_GAP_REPORT_SCHEMA,
        "report_type": "test_semantics_gap",
        "report_version": 1,
        "metadata": {
            "command": "highergraphen test-semantics gap",
            "cli_package": "highergraphen-cli"
        },
        "scenario": {
            "expected_schema": TEST_SEMANTICS_EXPECTED_OBLIGATIONS_SCHEMA,
            "verified_schema": TEST_SEMANTICS_VERIFICATION_REPORT_SCHEMA,
            "verified_report_ids": verified_report_ids
        },
        "result": {
            "status": status,
            "total_expected": covered_obligations.len() + missing_obligations.len(),
            "covered_count": covered_obligations.len(),
            "missing_count": missing_obligations.len(),
            "covered_obligations": covered_obligations,
            "missing_obligations": missing_obligations,
            "obstructions": obstructions,
            "completion_candidates": completion_candidates
        },
        "projection": {
            "audience": "ai_agent",
            "purpose": "test_semantics_gap_detection",
            "summary": if status == "gaps_detected" {
                format!("Detected {} missing test semantics obligations.", missing_obligations.len())
            } else {
                "No missing test semantics obligations detected.".to_owned()
            },
            "recommended_actions": if status == "gaps_detected" {
                vec![
                    "Add tests that cover each missing obligation target ID.",
                    "Run test-semantics interpret, review, and verify again after adding tests."
                ]
            } else {
                vec![
                    "Keep verified reports with the expected obligations for auditability."
                ]
            },
            "source_ids": missing_obligations
                .iter()
                .filter_map(|obligation| obligation.get("id").and_then(Value::as_str))
                .collect::<Vec<_>>(),
            "information_loss": [
                {
                    "description": "Gap detection compares expected obligation IDs and target IDs against verified semantic coverage; it does not inspect full source bodies.",
                    "source_ids": verified_report_ids
                },
                {
                    "description": "Missing-test completion candidates are unreviewed suggestions until a later review workflow accepts or rejects them.",
                    "source_ids": completion_candidates
                        .iter()
                        .filter_map(|candidate| candidate.get("id").and_then(Value::as_str))
                        .collect::<Vec<_>>()
                }
            ]
        }
    }))
}

#[derive(Clone, Debug, PartialEq)]
struct ExpectedObligation {
    id: String,
    obligation_type: String,
    summary: String,
    target_ids: Vec<String>,
    severity: String,
    source_ids: Vec<String>,
    review_status: String,
    confidence: f64,
}

impl ExpectedObligation {
    fn is_covered_by(&self, covered_ids: &BTreeSet<String>) -> bool {
        covered_ids.contains(&self.id)
            || self
                .target_ids
                .iter()
                .any(|target_id| covered_ids.contains(target_id))
    }

    fn to_value(&self, coverage_status: &str) -> Value {
        json!({
            "id": self.id,
            "obligation_type": self.obligation_type,
            "summary": self.summary,
            "target_ids": self.target_ids,
            "severity": self.severity,
            "source_ids": self.source_ids,
            "coverage_status": coverage_status,
            "review_status": self.review_status,
            "confidence": self.confidence
        })
    }
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

fn expected_obligations(input: &Value) -> Result<Vec<ExpectedObligation>, String> {
    input
        .get("obligations")
        .and_then(Value::as_array)
        .ok_or_else(|| "expected obligations input needs obligations array".to_owned())?
        .iter()
        .map(expected_obligation)
        .collect()
}

fn expected_obligation(value: &Value) -> Result<ExpectedObligation, String> {
    let id = required_string(value, "id")?;
    Ok(ExpectedObligation {
        id,
        obligation_type: required_string(value, "obligation_type")?,
        summary: required_string(value, "summary")?,
        target_ids: string_array(value.get("target_ids"))?,
        severity: value
            .get("severity")
            .and_then(Value::as_str)
            .unwrap_or("medium")
            .to_owned(),
        source_ids: string_array(value.get("source_ids"))?,
        review_status: value
            .get("review_status")
            .and_then(Value::as_str)
            .unwrap_or("accepted")
            .to_owned(),
        confidence: value
            .get("confidence")
            .and_then(Value::as_f64)
            .unwrap_or(0.72),
    })
}

fn required_string(value: &Value, key: &str) -> Result<String, String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|text| !text.trim().is_empty())
        .map(str::to_owned)
        .ok_or_else(|| format!("expected obligation needs {key}"))
}

fn string_array(value: Option<&Value>) -> Result<Vec<String>, String> {
    match value {
        Some(Value::Array(values)) => values
            .iter()
            .map(|value| {
                value
                    .as_str()
                    .map(str::to_owned)
                    .ok_or_else(|| "expected string array entries".to_owned())
            })
            .collect(),
        Some(_) => Err("expected string array".to_owned()),
        None => Ok(Vec::new()),
    }
}

fn covered_ids_from_verified_report(report: &Value) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    if report
        .get("result")
        .and_then(|result| result.get("status"))
        .and_then(Value::as_str)
        != Some("verified")
    {
        return ids;
    }

    push_array(&mut ids, report, &["result", "verified_candidate_ids"]);
    push_array(&mut ids, report, &["result", "accepted_fact_ids"]);
    push_array(&mut ids, report, &["result", "coverage_ids"]);
    push_array(&mut ids, report, &["result", "proof_obligation_ids"]);
    push_array(&mut ids, report, &["result", "semantic_proof_input_ids"]);
    push_array(&mut ids, report, &["result", "verified_morphism_ids"]);

    if let Some(candidate_id) = report
        .get("scenario")
        .and_then(|scenario| scenario.get("candidate_id"))
        .and_then(Value::as_str)
    {
        ids.insert(candidate_id.to_owned());
    }
    if let Some(candidate) = report
        .get("scenario")
        .and_then(|scenario| scenario.get("candidate"))
    {
        push_value_array(&mut ids, candidate.get("candidate_target_ids"));
        push_value_array(&mut ids, candidate.get("target_ids"));
        push_value_array(&mut ids, candidate.get("source_ids"));
    }
    for cell in report
        .get("result")
        .and_then(|result| result.get("verification_cells"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        if let Some(id) = cell.get("id").and_then(Value::as_str) {
            ids.insert(id.to_owned());
        }
        if let Some(candidate_id) = cell.get("candidate_id").and_then(Value::as_str) {
            ids.insert(candidate_id.to_owned());
        }
        if let Some(coverage_id) = cell.get("coverage_id").and_then(Value::as_str) {
            ids.insert(coverage_id.to_owned());
        }
        push_value_array(&mut ids, cell.get("source_ids"));
        push_value_array(&mut ids, cell.get("target_ids"));
    }
    ids
}

fn push_array(ids: &mut BTreeSet<String>, value: &Value, path: &[&str]) {
    let mut current = value;
    for segment in path {
        let Some(next) = current.get(*segment) else {
            return;
        };
        current = next;
    }
    push_value_array(ids, Some(current));
}

fn push_value_array(ids: &mut BTreeSet<String>, value: Option<&Value>) {
    if let Some(values) = value.and_then(Value::as_array) {
        for value in values {
            if let Some(text) = value.as_str() {
                ids.insert(text.to_owned());
                if let Some(stripped) = text.strip_prefix("proof-obligation:test-semantics:") {
                    ids.insert(stripped.to_owned());
                }
                if let Some(stripped) = text.strip_prefix("semantic-proof-input:test-semantics:") {
                    ids.insert(stripped.to_owned());
                }
            }
        }
    }
}

fn obstruction_for_obligation(obligation: &Value) -> Value {
    let id = obligation
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    json!({
        "id": format!("obstruction:test-semantics:missing-test:{}", slug(id)),
        "obstruction_type": "missing_test_semantics_coverage",
        "summary": format!("Missing verified test coverage for {id}."),
        "target_ids": obligation.get("target_ids").cloned().unwrap_or_else(|| json!([])),
        "severity": obligation.get("severity").cloned().unwrap_or_else(|| json!("medium")),
        "source_ids": [id],
        "review_status": "unreviewed",
        "confidence": 0.78
    })
}

fn completion_candidate_for_obligation(obligation: &Value) -> Value {
    let id = obligation
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    json!({
        "id": format!("candidate:test-semantics:missing-test:{}", slug(id)),
        "candidate_type": "missing_test",
        "summary": format!("Add a test that verifies {id}."),
        "suggested_test": {
            "name": format!("covers_{}", slug(id).replace('-', "_")),
            "target_ids": obligation.get("target_ids").cloned().unwrap_or_else(|| json!([])),
            "test_type": "unit_or_integration"
        },
        "source_ids": [id],
        "review_status": "unreviewed",
        "confidence": 0.74
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
    let normalized = slug.trim_matches('-').to_owned();
    if normalized.is_empty() {
        "obligation".to_owned()
    } else {
        normalized
    }
}
