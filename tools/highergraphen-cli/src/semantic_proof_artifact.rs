//! Adapter from bounded backend artifacts into semantic proof input documents.

use higher_graphen_core::{Confidence, Id, ReviewStatus, Severity, SourceKind};
use higher_graphen_runtime::{
    SemanticProofCell, SemanticProofCertificate, SemanticProofCounterexample,
    SemanticProofInputDocument, SemanticProofLaw, SemanticProofMorphism, SemanticProofSource,
    SemanticProofTheorem, SemanticProofVerificationPolicy,
};
use serde_json::Value;
use std::{fs, path::PathBuf, str::FromStr};

const ADAPTER_NAME: &str = "semantic-proof-from-artifact.v1";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ArtifactInputRequest {
    pub(crate) artifact: PathBuf,
    pub(crate) backend: String,
    pub(crate) backend_version: String,
    pub(crate) theorem_id: String,
    pub(crate) theorem_summary: String,
    pub(crate) law_id: String,
    pub(crate) law_summary: String,
    pub(crate) morphism_id: String,
    pub(crate) morphism_type: String,
    pub(crate) base_cell: String,
    pub(crate) base_label: String,
    pub(crate) head_cell: String,
    pub(crate) head_label: String,
}

pub(crate) fn input_from_artifact(
    request: ArtifactInputRequest,
) -> Result<SemanticProofInputDocument, String> {
    let artifact = read_artifact(&request.artifact)?;
    let status = required_string(&artifact, "status")?;
    let confidence = confidence(optional_f64(&artifact, "confidence").unwrap_or(0.8))?;
    let theorem_id = id(&request.theorem_id)?;
    let law_id = id(&request.law_id)?;
    let morphism_id = id(&request.morphism_id)?;
    let base_cell = id(&request.base_cell)?;
    let head_cell = id(&request.head_cell)?;

    let source = source_from_request(&request, confidence);
    let theorem = theorem_from_request(&request, &theorem_id, &law_id, &morphism_id);
    let semantic_cells = semantic_cells_from_request(&request, &base_cell, &head_cell, confidence);
    let morphisms = morphisms_from_request(
        &request,
        &morphism_id,
        &base_cell,
        &head_cell,
        &law_id,
        confidence,
    );
    let laws = laws_from_request(&request, &law_id, &morphism_id, confidence);
    let verification_policy = verification_policy_from_request(&request);
    let outcome = artifact_outcome(
        &request,
        &artifact,
        &status,
        ArtifactIds {
            theorem: theorem_id.clone(),
            law: law_id.clone(),
            morphism: morphism_id.clone(),
            base_cell: base_cell.clone(),
            head_cell: head_cell.clone(),
        },
        confidence,
    )?;

    Ok(SemanticProofInputDocument {
        schema: "highergraphen.semantic_proof.input.v1".to_owned(),
        source,
        theorem,
        semantic_cells,
        morphisms,
        laws,
        proof_certificates: outcome.proof_certificates,
        counterexamples: outcome.counterexamples,
        verification_policy,
    })
}

#[derive(Clone)]
struct ArtifactIds {
    theorem: Id,
    law: Id,
    morphism: Id,
    base_cell: Id,
    head_cell: Id,
}

struct ArtifactOutcome {
    proof_certificates: Vec<SemanticProofCertificate>,
    counterexamples: Vec<SemanticProofCounterexample>,
}

fn artifact_outcome(
    request: &ArtifactInputRequest,
    artifact: &Value,
    status: &str,
    ids: ArtifactIds,
    confidence: Confidence,
) -> Result<ArtifactOutcome, String> {
    match status {
        "proved" => Ok(ArtifactOutcome {
            proof_certificates: vec![certificate_from_artifact(
                request, artifact, ids, confidence,
            )?],
            counterexamples: Vec::new(),
        }),
        "counterexample" | "counterexample_found" => Ok(ArtifactOutcome {
            proof_certificates: Vec::new(),
            counterexamples: vec![counterexample_from_artifact(
                request, artifact, ids, confidence,
            )?],
        }),
        _ => Err(format!(
            "unsupported semantic proof artifact status {status:?}; expected proved, counterexample, or counterexample_found"
        )),
    }
}

fn certificate_from_artifact(
    request: &ArtifactInputRequest,
    artifact: &Value,
    ids: ArtifactIds,
    confidence: Confidence,
) -> Result<SemanticProofCertificate, String> {
    Ok(SemanticProofCertificate {
        id: id(format!(
            "certificate:semantic:{}:{}",
            slug(&request.backend),
            slug(ids.theorem.as_str())
        ))?,
        certificate_type: optional_string(artifact, "certificate_type")
            .unwrap_or_else(|| "formal_proof".to_owned()),
        backend: request.backend.clone(),
        backend_version: request.backend_version.clone(),
        theorem_id: ids.theorem,
        law_ids: vec![ids.law],
        morphism_ids: vec![ids.morphism],
        witness_ids: artifact_ids(artifact, "witness_ids")?
            .unwrap_or_else(|| vec![ids.base_cell, ids.head_cell]),
        input_hash: optional_string(artifact, "input_hash"),
        proof_hash: optional_string(artifact, "proof_hash"),
        confidence,
        review_status: optional_review_status(artifact)?.unwrap_or(ReviewStatus::Accepted),
    })
}

fn counterexample_from_artifact(
    request: &ArtifactInputRequest,
    artifact: &Value,
    ids: ArtifactIds,
    confidence: Confidence,
) -> Result<SemanticProofCounterexample, String> {
    Ok(SemanticProofCounterexample {
        id: id(format!(
            "counterexample:semantic:{}:{}",
            slug(&request.backend),
            slug(ids.theorem.as_str())
        ))?,
        counterexample_type: optional_string(artifact, "counterexample_type")
            .unwrap_or_else(|| "backend_counterexample".to_owned()),
        theorem_id: ids.theorem,
        law_ids: vec![ids.law],
        morphism_ids: vec![ids.morphism],
        path_ids: artifact_ids(artifact, "path_ids")?
            .unwrap_or_else(|| vec![ids.base_cell, ids.head_cell]),
        summary: optional_string(artifact, "summary")
            .unwrap_or_else(|| "Backend artifact supplied a counterexample.".to_owned()),
        severity: optional_severity(artifact)?.unwrap_or(Severity::High),
        confidence,
        review_status: optional_review_status(artifact)?.unwrap_or(ReviewStatus::Accepted),
    })
}

fn read_artifact(path: &PathBuf) -> Result<Value, String> {
    let text = fs::read_to_string(path).map_err(|error| {
        format!(
            "failed to read semantic proof artifact {}: {error}",
            path.display()
        )
    })?;
    serde_json::from_str(&text).map_err(|error| {
        format!(
            "failed to parse semantic proof artifact {}: {error}",
            path.display()
        )
    })
}

fn source_from_request(
    request: &ArtifactInputRequest,
    confidence: Confidence,
) -> SemanticProofSource {
    SemanticProofSource {
        kind: SourceKind::Code,
        uri: Some(format!("artifact:{}", request.artifact.display())),
        title: Some(format!("Semantic proof artifact from {}", request.backend)),
        confidence,
        adapters: vec![ADAPTER_NAME.to_owned()],
    }
}

fn theorem_from_request(
    request: &ArtifactInputRequest,
    theorem_id: &Id,
    law_id: &Id,
    morphism_id: &Id,
) -> SemanticProofTheorem {
    SemanticProofTheorem {
        id: theorem_id.clone(),
        summary: request.theorem_summary.clone(),
        law_ids: vec![law_id.clone()],
        morphism_ids: vec![morphism_id.clone()],
    }
}

fn semantic_cells_from_request(
    request: &ArtifactInputRequest,
    base_cell: &Id,
    head_cell: &Id,
    confidence: Confidence,
) -> Vec<SemanticProofCell> {
    vec![
        SemanticProofCell {
            id: base_cell.clone(),
            cell_type: "backend_semantic_cell".to_owned(),
            label: request.base_label.clone(),
            source_ids: Vec::new(),
            confidence: Some(confidence),
        },
        SemanticProofCell {
            id: head_cell.clone(),
            cell_type: "backend_semantic_cell".to_owned(),
            label: request.head_label.clone(),
            source_ids: Vec::new(),
            confidence: Some(confidence),
        },
    ]
}

fn morphisms_from_request(
    request: &ArtifactInputRequest,
    morphism_id: &Id,
    base_cell: &Id,
    head_cell: &Id,
    law_id: &Id,
    confidence: Confidence,
) -> Vec<SemanticProofMorphism> {
    vec![SemanticProofMorphism {
        id: morphism_id.clone(),
        morphism_type: request.morphism_type.clone(),
        source_ids: vec![base_cell.clone()],
        target_ids: vec![head_cell.clone()],
        law_ids: vec![law_id.clone()],
        confidence: Some(confidence),
    }]
}

fn laws_from_request(
    request: &ArtifactInputRequest,
    law_id: &Id,
    morphism_id: &Id,
    confidence: Confidence,
) -> Vec<SemanticProofLaw> {
    vec![SemanticProofLaw {
        id: law_id.clone(),
        summary: request.law_summary.clone(),
        applies_to_ids: vec![morphism_id.clone()],
        confidence: Some(confidence),
    }]
}

fn verification_policy_from_request(
    request: &ArtifactInputRequest,
) -> SemanticProofVerificationPolicy {
    SemanticProofVerificationPolicy {
        accepted_backends: vec![request.backend.clone()],
        require_input_hash: true,
        require_proof_hash: true,
        require_accepted_review: true,
        require_accepted_counterexample_review: true,
    }
}

fn required_string(value: &Value, key: &'static str) -> Result<String, String> {
    match value.get(key).and_then(Value::as_str) {
        Some(text) if !text.trim().is_empty() => Ok(text.trim().to_owned()),
        Some(_) => Err(format!("{key} must be a non-empty string")),
        None => Err(format!("missing {key}")),
    }
}

fn optional_string(value: &Value, key: &'static str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(str::to_owned)
}

fn optional_f64(value: &Value, key: &'static str) -> Option<f64> {
    value.get(key).and_then(Value::as_f64)
}

fn artifact_ids(value: &Value, key: &'static str) -> Result<Option<Vec<Id>>, String> {
    let Some(raw_ids) = value.get(key) else {
        return Ok(None);
    };
    let ids = raw_ids
        .as_array()
        .ok_or_else(|| format!("{key} must be an array of strings"))?
        .iter()
        .map(|raw| {
            raw.as_str()
                .ok_or_else(|| format!("{key} entries must be strings"))
                .and_then(id)
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Some(ids))
}

fn optional_review_status(value: &Value) -> Result<Option<ReviewStatus>, String> {
    optional_string(value, "review_status")
        .map(|status| ReviewStatus::from_str(&status).map_err(|error| error.to_string()))
        .transpose()
}

fn optional_severity(value: &Value) -> Result<Option<Severity>, String> {
    optional_string(value, "severity")
        .map(|severity| Severity::from_str(&severity).map_err(|error| error.to_string()))
        .transpose()
}

fn id(value: impl Into<String>) -> Result<Id, String> {
    Id::new(value).map_err(|error| error.to_string())
}

fn confidence(value: f64) -> Result<Confidence, String> {
    Confidence::new(value).map_err(|error| error.to_string())
}

fn slug(value: &str) -> String {
    let mut slug = String::new();
    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character.to_ascii_lowercase());
        } else if matches!(character, '-' | '_' | ':' | '.') {
            slug.push('-');
        }
    }

    let normalized = slug.trim_matches('-').to_owned();
    if normalized.is_empty() {
        "artifact".to_owned()
    } else {
        normalized
    }
}
