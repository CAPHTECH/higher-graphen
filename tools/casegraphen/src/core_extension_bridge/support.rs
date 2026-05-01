use super::CaseGraphenCoreExtensions;
use crate::workflow_model::{EvidenceType, WorkflowProvenance};
use higher_graphen_core::{
    Confidence, Id, PayloadRef, Provenance, ReviewStatus, SourceKind, SourceRef, WitnessType,
};
use serde_json::{Map, Value};
use std::str::FromStr;

const EXTENSIONS_METADATA_KEY: &str = "higher_graphen_extensions";

pub(crate) fn metadata_extensions(metadata: &Map<String, Value>) -> CaseGraphenCoreExtensions {
    metadata
        .get(EXTENSIONS_METADATA_KEY)
        .cloned()
        .map(serde_json::from_value)
        .transpose()
        .expect("metadata.higher_graphen_extensions must match CaseGraphenCoreExtensions")
        .unwrap_or_default()
}

pub(crate) fn workflow_provenance(provenance: &WorkflowProvenance, title: &str) -> Provenance {
    let kind = SourceKind::from_str(&provenance.source.kind)
        .or_else(|_| SourceKind::custom(&provenance.source.kind))
        .unwrap_or(SourceKind::Ai);
    let mut source = SourceRef::new(kind);
    if let Some(uri) = &provenance.source.uri {
        source = source.with_uri(uri).expect("workflow source uri is valid");
    }
    source = source
        .with_title(title)
        .expect("workflow source title is valid");
    if let Some(captured_at) = &provenance.source.captured_at {
        source = source
            .with_captured_at(captured_at)
            .expect("workflow source captured_at is valid");
    }
    if let Some(local_id) = &provenance.source.source_local_id {
        source = source
            .with_source_local_id(local_id)
            .expect("workflow source local id is valid");
    }

    let mut core =
        Provenance::new(source, provenance.confidence).with_review_status(provenance.review_status);
    if let Some(actor_id) = &provenance.actor_id {
        core = core
            .with_extractor_id(actor_id.to_string())
            .expect("workflow actor id is valid");
    }
    if let Some(method) = &provenance.extraction_method {
        core = core
            .with_extraction_method(method)
            .expect("workflow extraction method is valid");
    }
    if let Some(recorded_at) = &provenance.recorded_at {
        core = core
            .with_reviewed_at(recorded_at)
            .expect("workflow recorded_at is valid");
    }
    core
}

pub(crate) fn witness_type_for_workflow(evidence_type: EvidenceType) -> WitnessType {
    match evidence_type {
        EvidenceType::Document => WitnessType::DocumentExcerpt,
        EvidenceType::CommandOutput => WitnessType::LogEntry,
        EvidenceType::TestResult => WitnessType::TestResult,
        EvidenceType::ReviewRecord => WitnessType::HumanReview,
        EvidenceType::HumanObservation => WitnessType::Observation,
        EvidenceType::AiInference | EvidenceType::Proof => WitnessType::MachineCheckResult,
        EvidenceType::TransitionWitness => WitnessType::Observation,
    }
}

pub(crate) fn generated_provenance(
    uri: String,
    title: &str,
    review_status: ReviewStatus,
    score: f64,
) -> Provenance {
    Provenance::new(
        SourceRef::new(SourceKind::Code)
            .with_uri(uri)
            .expect("generated source uri is valid")
            .with_title(title)
            .expect("generated source title is valid"),
        confidence(score),
    )
    .with_review_status(review_status)
    .with_extraction_method("casegraphen-core-extension-bridge")
    .expect("generated extraction method is valid")
}

pub(crate) fn payload_ref(kind: &str, uri: String) -> PayloadRef {
    PayloadRef::new(kind, uri).expect("generated payload ref is valid")
}

pub(crate) fn source_uri(namespace: &str, root: &Id, segment: &str, id: &Id) -> String {
    format!(
        "casegraphen://{namespace}/{}/{segment}/{}",
        uri_token(root),
        uri_token(id)
    )
}

pub(crate) fn generated_id(prefix: &str, parts: &[&str]) -> Id {
    let suffix = parts
        .iter()
        .map(|part| sanitize_id_part(part))
        .collect::<Vec<_>>()
        .join(":");
    Id::new(format!("{prefix}:{suffix}")).expect("generated id is valid")
}

pub(crate) fn sanitize_id_part(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, ':' | '-' | '_' | '.') {
                character
            } else {
                '-'
            }
        })
        .collect()
}

pub(crate) fn confidence(value: f64) -> Confidence {
    Confidence::new(value).expect("static confidence is valid")
}

fn uri_token(id: &Id) -> String {
    sanitize_id_part(id.as_str()).replace(':', "/")
}
