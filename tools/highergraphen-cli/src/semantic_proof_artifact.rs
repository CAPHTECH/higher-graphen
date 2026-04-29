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
    let text = fs::read_to_string(&request.artifact).map_err(|error| {
        format!(
            "failed to read semantic proof artifact {}: {error}",
            request.artifact.display()
        )
    })?;
    let artifact: Value = serde_json::from_str(&text).map_err(|error| {
        format!(
            "failed to parse semantic proof artifact {}: {error}",
            request.artifact.display()
        )
    })?;

    let status = required_string(&artifact, "status")?;
    let confidence = confidence(optional_f64(&artifact, "confidence").unwrap_or(0.8))?;
    let theorem_id = id(&request.theorem_id)?;
    let law_id = id(&request.law_id)?;
    let morphism_id = id(&request.morphism_id)?;
    let base_cell = id(&request.base_cell)?;
    let head_cell = id(&request.head_cell)?;

    let source = SemanticProofSource {
        kind: SourceKind::Code,
        uri: Some(format!("artifact:{}", request.artifact.display())),
        title: Some(format!("Semantic proof artifact from {}", request.backend)),
        confidence,
        adapters: vec![ADAPTER_NAME.to_owned()],
    };
    let theorem = SemanticProofTheorem {
        id: theorem_id.clone(),
        summary: request.theorem_summary,
        law_ids: vec![law_id.clone()],
        morphism_ids: vec![morphism_id.clone()],
    };
    let semantic_cells = vec![
        SemanticProofCell {
            id: base_cell.clone(),
            cell_type: "backend_semantic_cell".to_owned(),
            label: request.base_label,
            source_ids: Vec::new(),
            confidence: Some(confidence),
        },
        SemanticProofCell {
            id: head_cell.clone(),
            cell_type: "backend_semantic_cell".to_owned(),
            label: request.head_label,
            source_ids: Vec::new(),
            confidence: Some(confidence),
        },
    ];
    let morphisms = vec![SemanticProofMorphism {
        id: morphism_id.clone(),
        morphism_type: request.morphism_type,
        source_ids: vec![base_cell.clone()],
        target_ids: vec![head_cell.clone()],
        law_ids: vec![law_id.clone()],
        confidence: Some(confidence),
    }];
    let laws = vec![SemanticProofLaw {
        id: law_id.clone(),
        summary: request.law_summary,
        applies_to_ids: vec![morphism_id.clone()],
        confidence: Some(confidence),
    }];
    let verification_policy = SemanticProofVerificationPolicy {
        accepted_backends: vec![request.backend.clone()],
        require_input_hash: true,
        require_proof_hash: true,
        require_accepted_review: true,
    };

    let mut proof_certificates = Vec::new();
    let mut counterexamples = Vec::new();
    match status.as_str() {
        "proved" => proof_certificates.push(SemanticProofCertificate {
            id: id(format!(
                "certificate:semantic:{}:{}",
                slug(&request.backend),
                slug(theorem_id.as_str())
            ))?,
            certificate_type: optional_string(&artifact, "certificate_type")
                .unwrap_or_else(|| "formal_proof".to_owned()),
            backend: request.backend,
            backend_version: request.backend_version,
            theorem_id: theorem_id.clone(),
            law_ids: vec![law_id.clone()],
            morphism_ids: vec![morphism_id.clone()],
            witness_ids: artifact_ids(&artifact, "witness_ids")?
                .unwrap_or_else(|| vec![base_cell.clone(), head_cell.clone()]),
            input_hash: optional_string(&artifact, "input_hash"),
            proof_hash: optional_string(&artifact, "proof_hash"),
            confidence,
            review_status: optional_review_status(&artifact)?.unwrap_or(ReviewStatus::Accepted),
        }),
        "counterexample" | "counterexample_found" => {
            counterexamples.push(SemanticProofCounterexample {
                id: id(format!(
                    "counterexample:semantic:{}:{}",
                    slug(&request.backend),
                    slug(theorem_id.as_str())
                ))?,
                counterexample_type: optional_string(&artifact, "counterexample_type")
                    .unwrap_or_else(|| "backend_counterexample".to_owned()),
                theorem_id: theorem_id.clone(),
                law_ids: vec![law_id.clone()],
                morphism_ids: vec![morphism_id.clone()],
                path_ids: artifact_ids(&artifact, "path_ids")?
                    .unwrap_or_else(|| vec![base_cell.clone(), head_cell.clone()]),
                summary: optional_string(&artifact, "summary")
                    .unwrap_or_else(|| "Backend artifact supplied a counterexample.".to_owned()),
                severity: optional_severity(&artifact)?.unwrap_or(Severity::High),
                confidence,
                review_status: optional_review_status(&artifact)?.unwrap_or(ReviewStatus::Accepted),
            });
        }
        _ => {
            return Err(format!(
                "unsupported semantic proof artifact status {status:?}; expected proved, counterexample, or counterexample_found"
            ));
        }
    }

    Ok(SemanticProofInputDocument {
        schema: "highergraphen.semantic_proof.input.v1".to_owned(),
        source,
        theorem,
        semantic_cells,
        morphisms,
        laws,
        proof_certificates,
        counterexamples,
        verification_policy,
    })
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
