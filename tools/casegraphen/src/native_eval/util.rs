use super::{NativeCompletionCandidateType, NativeObstructionType};
use higher_graphen_core::{Confidence, Id, Provenance, ReviewStatus, SourceKind, SourceRef};
use std::collections::BTreeSet;

pub(super) fn candidate_type_stem(candidate_type: NativeCompletionCandidateType) -> &'static str {
    match candidate_type {
        NativeCompletionCandidateType::NativeCompletionCell => "native-completion-cell",
        NativeCompletionCandidateType::MissingEvidence => "missing-evidence",
        NativeCompletionCandidateType::MissingProof => "missing-proof",
        NativeCompletionCandidateType::MissingReview => "missing-review",
        NativeCompletionCandidateType::MissingDependencyResolution => {
            "missing-dependency-resolution"
        }
        NativeCompletionCandidateType::ContradictionResolution => "contradiction-resolution",
    }
}

pub(super) fn obstruction_type_stem(obstruction_type: NativeObstructionType) -> &'static str {
    match obstruction_type {
        NativeObstructionType::UnresolvedDependency => "unresolved-dependency",
        NativeObstructionType::ExternalWait => "external-wait",
        NativeObstructionType::MissingEvidence => "missing-evidence",
        NativeObstructionType::MissingProof => "missing-proof",
        NativeObstructionType::Contradiction => "contradiction",
        NativeObstructionType::ReviewRequired => "review-required",
        NativeObstructionType::ProjectionLoss => "projection-loss",
        NativeObstructionType::InvalidClose => "invalid-close",
    }
}

pub(super) fn generated_id(prefix: &str, parts: &[&str]) -> Id {
    let suffix = parts
        .iter()
        .map(|part| sanitize(part))
        .collect::<Vec<_>>()
        .join(":");
    id(&format!("{prefix}:{suffix}"))
}

pub(super) fn generated_provenance(title: &str, value: f64) -> Provenance {
    Provenance::new(SourceRef::new(SourceKind::Ai), confidence(value))
        .with_review_status(ReviewStatus::Unreviewed)
        .with_title(title)
}

pub(super) fn confidence(value: f64) -> Confidence {
    Confidence::new(value).expect("static confidence")
}

pub(super) fn dedupe_ids(ids: Vec<Id>) -> Vec<Id> {
    ids.into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub(super) fn id(value: &str) -> Id {
    Id::new(value).expect("static or generated id")
}

fn sanitize(value: &str) -> String {
    value
        .chars()
        .map(|character| match character {
            'a'..='z' | 'A'..='Z' | '0'..='9' => character,
            _ => '-',
        })
        .collect()
}

trait ProvenanceTitle {
    fn with_title(self, title: &str) -> Self;
}

impl ProvenanceTitle for Provenance {
    fn with_title(mut self, title: &str) -> Self {
        self.source.title = Some(title.to_owned());
        self.extraction_method = Some("casegraphen.native_eval.v1".to_owned());
        self
    }
}
