//! Rebuilds semantic proof inputs from insufficient proof and verification reports.

use higher_graphen_core::{Confidence, Id, SourceKind};
use higher_graphen_runtime::{
    SemanticProofCell, SemanticProofInputDocument, SemanticProofLaw, SemanticProofMorphism,
    SemanticProofReport, SemanticProofSource, SemanticProofStatus, SemanticProofTheorem,
    SemanticProofVerificationPolicy,
};
use serde_json::Value;

const ADAPTER_NAME: &str = "semantic-proof-reinput-from-report.v1";
const TEST_SEMANTICS_VERIFICATION_ADAPTER_NAME: &str =
    "test-semantics-verification-to-semantic-proof-input.v1";
const TEST_SEMANTICS_VERIFICATION_REPORT_SCHEMA: &str =
    "highergraphen.test_semantics.verification.report.v1";

pub(crate) fn input_from_report_value(report: Value) -> Result<SemanticProofInputDocument, String> {
    match report.get("schema").and_then(Value::as_str) {
        Some("highergraphen.semantic_proof.report.v1") => {
            let report = serde_json::from_value(report).map_err(|error| error.to_string())?;
            input_from_report(report)
        }
        Some(TEST_SEMANTICS_VERIFICATION_REPORT_SCHEMA) => {
            input_from_test_semantics_verification_report(&report)
        }
        Some(schema) => Err(format!(
            "unsupported semantic-proof input report schema {schema}; expected highergraphen.semantic_proof.report.v1 or {TEST_SEMANTICS_VERIFICATION_REPORT_SCHEMA}"
        )),
        None => Err("semantic-proof input report needs schema".to_owned()),
    }
}

pub(crate) fn input_from_report(
    report: SemanticProofReport,
) -> Result<SemanticProofInputDocument, String> {
    if report.result.status != SemanticProofStatus::InsufficientProof {
        return Err(format!(
            "semantic proof report status {:?} has no proof obligations to reinput",
            report.result.status
        ));
    }

    let scenario = report.scenario;
    let mut law_ids = Vec::new();
    let mut morphism_ids = Vec::new();
    for issue in report.result.issues {
        match issue.issue_type.as_str() {
            "missing_law_proof" => push_ids(&mut law_ids, issue.target_ids),
            "missing_morphism_proof" => push_ids(&mut morphism_ids, issue.target_ids),
            _ => {}
        }
    }

    if law_ids.is_empty() && morphism_ids.is_empty() {
        law_ids = scenario.theorem.law_ids.clone();
        morphism_ids = scenario.theorem.morphism_ids.clone();
    }
    if law_ids.is_empty() && morphism_ids.is_empty() {
        return Err(
            "semantic proof report contains no reinputable law or morphism obligations".to_owned(),
        );
    }

    for morphism in &scenario.morphisms {
        if morphism_ids.contains(&morphism.id) {
            push_ids(&mut law_ids, morphism.law_ids.clone());
        }
    }
    for law in &scenario.laws {
        if law_ids.contains(&law.id) {
            push_ids(&mut morphism_ids, law.applies_to_ids.clone());
        }
    }

    let morphisms = scenario
        .morphisms
        .into_iter()
        .filter(|morphism| morphism_ids.contains(&morphism.id))
        .collect::<Vec<_>>();
    let laws = scenario
        .laws
        .into_iter()
        .filter(|law| law_ids.contains(&law.id))
        .collect::<Vec<_>>();
    let semantic_cells = cells_for_morphisms(scenario.semantic_cells, &morphisms);
    let mut theorem = scenario.theorem;
    theorem.law_ids = laws.iter().map(|law| law.id.clone()).collect();
    theorem.morphism_ids = morphisms
        .iter()
        .map(|morphism| morphism.id.clone())
        .collect();

    Ok(SemanticProofInputDocument {
        schema: "highergraphen.semantic_proof.input.v1".to_owned(),
        source: reinput_source(scenario.source, theorem.id.as_str()),
        theorem,
        semantic_cells,
        morphisms,
        laws,
        proof_certificates: Vec::new(),
        counterexamples: Vec::new(),
        verification_policy: scenario.verification_policy,
    })
}

fn cells_for_morphisms(
    cells: Vec<SemanticProofCell>,
    morphisms: &[SemanticProofMorphism],
) -> Vec<SemanticProofCell> {
    let mut cell_ids = Vec::new();
    for morphism in morphisms {
        push_ids(&mut cell_ids, morphism.source_ids.clone());
        push_ids(&mut cell_ids, morphism.target_ids.clone());
    }
    cells
        .into_iter()
        .filter(|cell| cell_ids.contains(&cell.id))
        .collect()
}

fn reinput_source(mut source: SemanticProofSource, theorem_id: &str) -> SemanticProofSource {
    if !source
        .adapters
        .iter()
        .any(|adapter| adapter == ADAPTER_NAME)
    {
        source.adapters.push(ADAPTER_NAME.to_owned());
    }
    if source.uri.is_none() {
        source.uri = Some(format!("semantic-proof-reinput:{theorem_id}"));
    }
    if source.title.is_none() {
        source.title = Some("Semantic proof reinput obligations".to_owned());
    }
    source.kind = SourceKind::Code;
    source
}

fn push_ids(target: &mut Vec<Id>, ids: Vec<Id>) {
    for id in ids {
        if !target.contains(&id) {
            target.push(id);
        }
    }
}

fn input_from_test_semantics_verification_report(
    report: &Value,
) -> Result<SemanticProofInputDocument, String> {
    let result = report
        .get("result")
        .ok_or_else(|| "test semantics verification report needs result".to_owned())?;
    if result.get("status").and_then(Value::as_str) != Some("verified") {
        return Err(
            "test semantics verification report must have result.status == verified".to_owned(),
        );
    }

    let scenario = report
        .get("scenario")
        .ok_or_else(|| "test semantics verification report needs scenario".to_owned())?;
    let candidate_id = required_string(scenario, "candidate_id")?;
    let candidate = scenario
        .get("candidate")
        .ok_or_else(|| "test semantics verification report needs scenario.candidate".to_owned())?;
    let law_ids = candidate_target_ids(candidate, result)?;
    if law_ids.is_empty() {
        return Err(
            "test semantics verification report contains no semantic proof target laws".to_owned(),
        );
    }

    let morphism_ids = verified_morphism_ids(result, candidate_id, &law_ids)?;
    let confidence = Confidence::new(0.74).map_err(|error| error.to_string())?;
    let source_ids = string_array(candidate.get("source_ids"))?;
    let theorem_id = id(format!("theorem:test-semantics:{}", slug(candidate_id)))?;
    let proof_parts = test_semantics_proof_parts(
        candidate_id,
        &source_ids,
        &law_ids,
        &morphism_ids,
        confidence,
    )?;

    Ok(SemanticProofInputDocument {
        schema: "highergraphen.semantic_proof.input.v1".to_owned(),
        source: test_semantics_source(candidate_id, confidence),
        theorem: SemanticProofTheorem {
            id: theorem_id,
            summary: format!(
                "Reviewed test semantics candidate {candidate_id} has formal proof obligations."
            ),
            law_ids: proof_parts.theorem_law_ids,
            morphism_ids: proof_parts.theorem_morphism_ids,
        },
        semantic_cells: proof_parts.semantic_cells,
        morphisms: proof_parts.morphisms,
        laws: proof_parts.laws,
        proof_certificates: Vec::new(),
        counterexamples: Vec::new(),
        verification_policy: test_semantics_policy(),
    })
}

struct TestSemanticsProofParts {
    semantic_cells: Vec<SemanticProofCell>,
    morphisms: Vec<SemanticProofMorphism>,
    laws: Vec<SemanticProofLaw>,
    theorem_law_ids: Vec<Id>,
    theorem_morphism_ids: Vec<Id>,
}

fn test_semantics_proof_parts(
    candidate_id: &str,
    source_ids: &[String],
    law_ids: &[String],
    morphism_ids: &[String],
    confidence: Confidence,
) -> Result<TestSemanticsProofParts, String> {
    let candidate_cell_id = id(format!(
        "cell:test-semantics:candidate:{}",
        slug(candidate_id)
    ))?;
    let mut parts = TestSemanticsProofParts {
        semantic_cells: vec![SemanticProofCell {
            id: candidate_cell_id.clone(),
            cell_type: "verified_test_semantics_candidate".to_owned(),
            label: format!("Verified test semantics candidate {candidate_id}"),
            source_ids: ids(source_ids.iter().cloned())?,
            confidence: Some(confidence),
        }],
        morphisms: Vec::new(),
        laws: Vec::new(),
        theorem_law_ids: Vec::new(),
        theorem_morphism_ids: Vec::new(),
    };
    for (index, law_id_text) in law_ids.iter().enumerate() {
        push_test_semantics_law_part(
            &mut parts,
            candidate_id,
            &candidate_cell_id,
            law_id_text,
            &morphism_ids[index],
            confidence,
        )?;
    }
    Ok(parts)
}

fn push_test_semantics_law_part(
    parts: &mut TestSemanticsProofParts,
    candidate_id: &str,
    candidate_cell_id: &Id,
    law_id_text: &str,
    morphism_id_text: &str,
    confidence: Confidence,
) -> Result<(), String> {
    let law_id = id(law_id_text.to_owned())?;
    let morphism_id = id(morphism_id_text.to_owned())?;
    let law_cell_id = id(format!("cell:test-semantics:law:{}", slug(law_id.as_str())))?;
    parts.semantic_cells.push(SemanticProofCell {
        id: law_cell_id.clone(),
        cell_type: "semantic_law_target".to_owned(),
        label: format!("Semantic law target {law_id}"),
        source_ids: vec![law_id.clone()],
        confidence: Some(confidence),
    });
    parts.morphisms.push(SemanticProofMorphism {
        id: morphism_id.clone(),
        morphism_type: "verified_test_semantics_candidate_to_law".to_owned(),
        source_ids: vec![candidate_cell_id.clone()],
        target_ids: vec![law_cell_id],
        law_ids: vec![law_id.clone()],
        confidence: Some(confidence),
    });
    parts.laws.push(SemanticProofLaw {
        id: law_id.clone(),
        summary: format!(
            "Verified test semantics candidate {candidate_id} must preserve {law_id}."
        ),
        applies_to_ids: vec![morphism_id.clone()],
        confidence: Some(confidence),
    });
    parts.theorem_law_ids.push(law_id);
    parts.theorem_morphism_ids.push(morphism_id);
    Ok(())
}

fn test_semantics_source(candidate_id: &str, confidence: Confidence) -> SemanticProofSource {
    SemanticProofSource {
        kind: SourceKind::Code,
        uri: Some(format!("test-semantics-verification:{candidate_id}")),
        title: Some("Test semantics verification proof obligations".to_owned()),
        confidence,
        adapters: vec![TEST_SEMANTICS_VERIFICATION_ADAPTER_NAME.to_owned()],
    }
}

fn test_semantics_policy() -> SemanticProofVerificationPolicy {
    SemanticProofVerificationPolicy {
        accepted_backends: Vec::new(),
        require_input_hash: true,
        require_proof_hash: true,
        require_accepted_review: true,
        require_accepted_counterexample_review: true,
    }
}

fn required_string<'a>(value: &'a Value, key: &str) -> Result<&'a str, String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("missing or invalid {key}"))
}

fn candidate_target_ids(candidate: &Value, result: &Value) -> Result<Vec<String>, String> {
    let mut law_ids = string_array(candidate.get("candidate_target_ids"))?;
    law_ids.extend(string_array(candidate.get("target_ids"))?);
    if law_ids.is_empty() {
        law_ids.extend(
            string_array(result.get("proof_obligation_ids"))?
                .into_iter()
                .map(|obligation_id| {
                    obligation_id
                        .trim_start_matches("proof-obligation:test-semantics:")
                        .to_owned()
                }),
        );
    }
    law_ids.sort();
    law_ids.dedup();
    Ok(law_ids)
}

fn verified_morphism_ids(
    result: &Value,
    candidate_id: &str,
    law_ids: &[String],
) -> Result<Vec<String>, String> {
    let supplied = string_array(result.get("verified_morphism_ids"))?;
    if supplied.len() == law_ids.len() {
        return Ok(supplied);
    }
    Ok(law_ids
        .iter()
        .map(|law_id| {
            format!(
                "morphism:test-semantics-proof:{}:{}",
                slug(candidate_id),
                slug(law_id)
            )
        })
        .collect())
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

fn ids(values: impl IntoIterator<Item = String>) -> Result<Vec<Id>, String> {
    values.into_iter().map(id).collect()
}

fn id(value: impl Into<String>) -> Result<Id, String> {
    Id::new(value).map_err(|error| error.to_string())
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
        "semantic-proof".to_owned()
    } else {
        normalized
    }
}
