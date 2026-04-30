//! Attaches bounded backend artifacts to existing semantic proof inputs.

use higher_graphen_core::{Confidence, Id, ReviewStatus, Severity};
use higher_graphen_runtime::{
    SemanticProofCertificate, SemanticProofCounterexample, SemanticProofInputDocument,
};
use serde_json::Value;
use std::{fs, path::PathBuf, str::FromStr};

const ADAPTER_NAME: &str = "semantic-proof-attach-artifact.v1";

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AttachArtifactRequest {
    pub(crate) input: SemanticProofInputDocument,
    pub(crate) artifact: PathBuf,
    pub(crate) backend: String,
    pub(crate) backend_version: String,
}

pub(crate) fn attach_artifact(
    request: AttachArtifactRequest,
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
    let mut input = request.input;
    if !input
        .source
        .adapters
        .iter()
        .any(|adapter| adapter == ADAPTER_NAME)
    {
        input.source.adapters.push(ADAPTER_NAME.to_owned());
    }
    if !input
        .verification_policy
        .accepted_backends
        .iter()
        .any(|backend| backend == &request.backend)
    {
        input
            .verification_policy
            .accepted_backends
            .push(request.backend.clone());
    }

    match status.as_str() {
        "proved" => input.proof_certificates.push(SemanticProofCertificate {
            id: id(format!(
                "certificate:semantic:{}:{}",
                slug(&request.backend),
                slug(input.theorem.id.as_str())
            ))?,
            certificate_type: optional_string(&artifact, "certificate_type")
                .unwrap_or_else(|| "formal_proof".to_owned()),
            backend: request.backend,
            backend_version: request.backend_version,
            theorem_id: input.theorem.id.clone(),
            law_ids: input.theorem.law_ids.clone(),
            morphism_ids: input.theorem.morphism_ids.clone(),
            witness_ids: artifact_ids(&artifact, "witness_ids")?.unwrap_or_else(|| {
                input
                    .semantic_cells
                    .iter()
                    .map(|cell| cell.id.clone())
                    .collect()
            }),
            input_hash: optional_string(&artifact, "input_hash"),
            proof_hash: optional_string(&artifact, "proof_hash"),
            confidence,
            review_status: optional_review_status(&artifact)?.unwrap_or(ReviewStatus::Accepted),
        }),
        "counterexample" | "counterexample_found" => {
            input.counterexamples.push(SemanticProofCounterexample {
                id: id(format!(
                    "counterexample:semantic:{}:{}",
                    slug(&request.backend),
                    slug(input.theorem.id.as_str())
                ))?,
                counterexample_type: optional_string(&artifact, "counterexample_type")
                    .unwrap_or_else(|| "backend_counterexample".to_owned()),
                theorem_id: input.theorem.id.clone(),
                law_ids: input.theorem.law_ids.clone(),
                morphism_ids: input.theorem.morphism_ids.clone(),
                path_ids: artifact_ids(&artifact, "path_ids")?.unwrap_or_else(|| {
                    input
                        .semantic_cells
                        .iter()
                        .map(|cell| cell.id.clone())
                        .collect()
                }),
                summary: optional_string(&artifact, "summary")
                    .unwrap_or_else(|| "Backend artifact supplied a counterexample.".to_owned()),
                severity: optional_severity(&artifact)?.unwrap_or(Severity::High),
                confidence,
                review_status: optional_review_status(&artifact)?.unwrap_or(ReviewStatus::Accepted),
            })
        }
        _ => {
            return Err(format!(
                "unsupported semantic proof artifact status {status:?}; expected proved, counterexample, or counterexample_found"
            ));
        }
    }

    Ok(input)
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
