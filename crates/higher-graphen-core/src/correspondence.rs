use crate::{Confidence, CoreError, Id, Result, ReviewRequirement, ReviewStatus};
use serde::de::{self, MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeMap;
use std::fmt;

/// Reference to a structure that participates in a correspondence.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(tag = "kind", content = "id", rename_all = "camelCase")]
pub enum ParticipantRef {
    /// Cell participant.
    Cell(Id),
    /// Complex participant.
    Complex(Id),
    /// Context participant.
    Context(Id),
    /// Projection participant.
    Projection(Id),
    /// Claim participant.
    Claim(Id),
    /// Evidence participant.
    Evidence(Id),
    /// Invariant participant.
    Invariant(Id),
    /// Obstruction participant.
    Obstruction(Id),
    /// Completion candidate participant.
    CompletionCandidate(Id),
}

impl ParticipantRef {
    /// Creates a participant reference from a compact prefixed identifier.
    pub fn from_compact_id(id: Id) -> Self {
        let value = id.as_str();
        if value.starts_with("complex:") {
            Self::Complex(id)
        } else if value.starts_with("ctx:") || value.starts_with("context:") {
            Self::Context(id)
        } else if value.starts_with("projection:") {
            Self::Projection(id)
        } else if value.starts_with("claim:") {
            Self::Claim(id)
        } else if value.starts_with("evidence:") {
            Self::Evidence(id)
        } else if value.starts_with("invariant:") {
            Self::Invariant(id)
        } else if value.starts_with("obstruction:") {
            Self::Obstruction(id)
        } else if value.starts_with("completion:")
            || value.starts_with("completion_candidate:")
            || value.starts_with("candidate:")
        {
            Self::CompletionCandidate(id)
        } else {
            Self::Cell(id)
        }
    }

    /// Returns the participant identifier regardless of participant kind.
    #[must_use]
    pub fn id(&self) -> &Id {
        match self {
            Self::Cell(id)
            | Self::Complex(id)
            | Self::Context(id)
            | Self::Projection(id)
            | Self::Claim(id)
            | Self::Evidence(id)
            | Self::Invariant(id)
            | Self::Obstruction(id)
            | Self::CompletionCandidate(id) => id,
        }
    }
}

impl<'de> Deserialize<'de> for ParticipantRef {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(ParticipantRefVisitor)
    }
}

struct ParticipantRefVisitor;

impl<'de> Visitor<'de> for ParticipantRefVisitor {
    type Value = ParticipantRef;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a compact participant id string or {kind, id} object")
    }

    fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
    where
        E: de::Error,
    {
        Id::new(value)
            .map(ParticipantRef::from_compact_id)
            .map_err(E::custom)
    }

    fn visit_string<E>(self, value: String) -> std::result::Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_str(&value)
    }

    fn visit_map<M>(self, mut access: M) -> std::result::Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut kind: Option<String> = None;
        let mut id: Option<Id> = None;

        while let Some(key) = access.next_key::<String>()? {
            match key.as_str() {
                "kind" => kind = Some(access.next_value()?),
                "id" => id = Some(access.next_value()?),
                unknown => {
                    return Err(de::Error::unknown_field(unknown, &["kind", "id"]));
                }
            }
        }

        let kind = kind.ok_or_else(|| de::Error::missing_field("kind"))?;
        let id = id.ok_or_else(|| de::Error::missing_field("id"))?;

        match kind.as_str() {
            "cell" => Ok(ParticipantRef::Cell(id)),
            "complex" => Ok(ParticipantRef::Complex(id)),
            "context" => Ok(ParticipantRef::Context(id)),
            "projection" => Ok(ParticipantRef::Projection(id)),
            "claim" => Ok(ParticipantRef::Claim(id)),
            "evidence" => Ok(ParticipantRef::Evidence(id)),
            "invariant" => Ok(ParticipantRef::Invariant(id)),
            "obstruction" => Ok(ParticipantRef::Obstruction(id)),
            "completionCandidate" => Ok(ParticipantRef::CompletionCandidate(id)),
            unknown => Err(de::Error::unknown_variant(
                unknown,
                &[
                    "cell",
                    "complex",
                    "context",
                    "projection",
                    "claim",
                    "evidence",
                    "invariant",
                    "obstruction",
                    "completionCandidate",
                ],
            )),
        }
    }
}

/// A role-labeled participant entry in a correspondence cell.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CorrespondenceParticipant {
    /// Participant role within this correspondence.
    pub role: String,
    /// Referenced participant.
    #[serde(rename = "ref")]
    pub participant: ParticipantRef,
}

impl CorrespondenceParticipant {
    /// Creates a role-labeled participant.
    pub fn new(role: impl Into<String>, participant: ParticipantRef) -> Result<Self> {
        Ok(Self {
            role: required_text("role", role)?,
            participant,
        })
    }
}

/// Category of relationship represented by a correspondence cell.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum CorrespondenceKind {
    /// Same target or normalized representation.
    ExactIdentity,
    /// Surface fields such as strings, labels, or tags overlap.
    SurfaceOverlap,
    /// Meaning overlaps even when representation differs.
    SemanticOverlap,
    /// Structural pattern such as a subgraph or subcomplex overlaps.
    StructuralOverlap,
    /// Same invariant or constraint area is involved.
    ConstraintOverlap,
    /// Same evidence or provenance supports the participants.
    EvidenceOverlap,
    /// Contexts share a local region.
    ContextualOverlap,
    /// Same cause, effect, or causal path is involved.
    CausalOverlap,
    /// Projections derive from the same source structure.
    ProjectionOverlap,
    /// One participant details another.
    Refinement,
    /// One participant abstracts another.
    Abstraction,
    /// Shared region exists but claims, constraints, or states conflict.
    Conflict,
    /// Meaning emerges only from the participant combination.
    Synergy,
    /// Correspondence exists but gluing or integration is blocked.
    Obstructed,
}

/// Directional or truth-status polarity of a correspondence.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum CorrespondencePolarity {
    /// Participants agree over the shared structure.
    Agreeing,
    /// Participants conflict over the shared structure.
    Conflicting,
    /// One participant refines another.
    Refining,
    /// One participant is projected from another.
    Projecting,
    /// Polarity is ambiguous.
    Ambiguous,
    /// Polarity has not been determined.
    Unknown,
}

/// Kind of overlap witness.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum OverlapWitnessKind {
    /// Shared features.
    FeatureSet,
    /// Shared predicates.
    PredicateSet,
    /// Shared normalized claim.
    NormalizedClaim,
    /// Shared subgraph pattern.
    Subgraph,
    /// Shared subcomplex pattern.
    Subcomplex,
    /// Shared constraints.
    ConstraintSet,
    /// Shared evidence references.
    EvidenceSet,
    /// Shared boundary.
    Boundary,
    /// Shared projection trace.
    ProjectionTrace,
    /// Shared causal pattern.
    CausalPattern,
    /// Shared context restriction.
    ContextRestriction,
}

impl OverlapWitnessKind {
    /// Returns the stable serde discriminant string for this witness kind.
    #[must_use]
    pub fn kind(&self) -> &'static str {
        match self {
            Self::FeatureSet => "FeatureSet",
            Self::PredicateSet => "PredicateSet",
            Self::NormalizedClaim => "NormalizedClaim",
            Self::Subgraph => "Subgraph",
            Self::Subcomplex => "Subcomplex",
            Self::ConstraintSet => "ConstraintSet",
            Self::EvidenceSet => "EvidenceSet",
            Self::Boundary => "Boundary",
            Self::ProjectionTrace => "ProjectionTrace",
            Self::CausalPattern => "CausalPattern",
            Self::ContextRestriction => "ContextRestriction",
        }
    }

    /// Returns true when this witness kind is explicit enough to support accepted semantic overlap.
    #[must_use]
    pub fn supports_accepted_semantic_overlap(self) -> bool {
        matches!(
            self,
            Self::NormalizedClaim | Self::PredicateSet | Self::FeatureSet
        )
    }
}

/// Difference category inside a correspondence.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum DifferenceKind {
    /// One participant contains extra detail.
    AdditionalDetail,
    /// One participant lacks detail present in another.
    MissingDetail,
    /// Types differ.
    TypeMismatch,
    /// Predicates differ.
    PredicateMismatch,
    /// Modalities such as observed, required, or forbidden differ.
    ModalityMismatch,
    /// Contexts differ or are incompatible.
    ContextMismatch,
    /// Evidence differs.
    EvidenceMismatch,
    /// Confidence differs materially.
    ConfidenceMismatch,
    /// Time or validity interval differs.
    TemporalMismatch,
    /// Invariant satisfaction differs.
    InvariantMismatch,
    /// Participants contradict each other.
    Contradiction,
    /// Projection omitted or collapsed information.
    ProjectionLoss,
}

/// Severity of a difference witness.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DifferenceSeverity {
    /// Difference is informational only.
    Informational,
    /// Difference is minor.
    Minor,
    /// Difference is major.
    Major,
    /// Difference blocks silent gluing or merge.
    Blocking,
}

impl DifferenceSeverity {
    /// Returns the stable serde discriminant string for this severity.
    #[must_use]
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Informational => "informational",
            Self::Minor => "minor",
            Self::Major => "major",
            Self::Blocking => "blocking",
        }
    }
}

/// Shared structure carried by an overlap witness.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "camelCase")]
pub enum SharedStructure {
    /// Feature set.
    FeatureSet(Vec<Feature>),
    /// Predicate set.
    PredicateSet(Vec<Predicate>),
    /// Normalized claim.
    NormalizedClaim(NormalizedClaim),
    /// Subgraph pattern.
    Subgraph(SubgraphPattern),
    /// Subcomplex pattern.
    Subcomplex(SubcomplexPattern),
    /// Invariant references.
    ConstraintSet(Vec<Id>),
    /// Evidence references.
    EvidenceSet(Vec<Id>),
    /// Boundary pattern.
    Boundary(BoundaryPattern),
    /// Projection trace.
    ProjectionTrace(ProjectionTrace),
    /// Causal pattern.
    CausalPattern(CausalPattern),
    /// Context restriction.
    ContextRestriction(ContextRestriction),
}

/// Structure that differs across participants.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "camelCase")]
pub enum DifferingStructure {
    /// Additional detail by participant.
    AdditionalDetail(BTreeMap<String, String>),
    /// Missing detail by participant.
    MissingDetail(BTreeMap<String, String>),
    /// Type mismatch by participant.
    TypeMismatch(BTreeMap<String, String>),
    /// Predicate mismatch by participant.
    PredicateMismatch(BTreeMap<String, String>),
    /// Modality mismatch by participant.
    ModalityMismatch(BTreeMap<String, String>),
    /// Context mismatch by participant.
    ContextMismatch(BTreeMap<String, String>),
    /// Evidence mismatch by participant.
    EvidenceMismatch(Vec<Id>),
    /// Confidence mismatch by participant.
    ConfidenceMismatch(BTreeMap<String, Confidence>),
    /// Temporal mismatch by participant.
    TemporalMismatch(BTreeMap<String, String>),
    /// Invariant mismatch by participant.
    InvariantMismatch(Vec<InvariantCheckResult>),
    /// Contradiction details.
    Contradiction(BTreeMap<String, String>),
    /// Projection loss details.
    ProjectionLoss(ProjectionLoss),
}

/// Named feature used by overlap extraction.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Feature {
    /// Feature key.
    pub key: String,
    /// Feature value.
    pub value: String,
}

/// Predicate triple or relation fragment used by overlap extraction.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Predicate {
    /// Subject identifier or normalized label.
    pub subject: String,
    /// Relation or predicate name.
    pub relation: String,
    /// Object identifier or normalized label.
    pub object: String,
}

/// Normalized claim fields used to make semantic overlap reviewable.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NormalizedClaim {
    /// Subject identifier or normalized label.
    pub subject: String,
    /// Relation or predicate name.
    pub relation: String,
    /// Object identifier or normalized label.
    pub object: String,
    /// Optional modality such as observed, inferred, required, or forbidden.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modality: Option<String>,
    /// Optional normalized time or validity interval.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temporal_scope: Option<String>,
}

/// Minimal subgraph pattern for exact structural overlap.
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SubgraphPattern {
    /// Node identifiers or normalized labels.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub node_ids: Vec<Id>,
    /// Edge identifiers or normalized labels.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub edge_ids: Vec<Id>,
}

/// Minimal subcomplex pattern for exact structural overlap.
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SubcomplexPattern {
    /// Cell identifiers in the shared subcomplex.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cell_ids: Vec<Id>,
    /// Incidence identifiers in the shared subcomplex.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub incidence_ids: Vec<Id>,
}

/// Shared boundary pattern.
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BoundaryPattern {
    /// Boundary cell identifiers.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub boundary_cell_ids: Vec<Id>,
}

/// Trace from projected view back to source structures.
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProjectionTrace {
    /// Source structure identifiers.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_ids: Vec<Id>,
    /// Projection identifiers.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub projection_ids: Vec<Id>,
}

/// Causal pattern shared across participants.
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CausalPattern {
    /// Cause identifiers.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cause_ids: Vec<Id>,
    /// Effect identifiers.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub effect_ids: Vec<Id>,
    /// Path identifiers.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub path_ids: Vec<Id>,
}

/// Context restriction shared by participants.
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContextRestriction {
    /// Source context identifiers.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_context_ids: Vec<Id>,
    /// Restricted context identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_context_id: Option<Id>,
    /// Elements retained by the restriction.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub retained_element_ids: Vec<Id>,
}

/// Mapping from a participant into a witness path.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ParticipantMapping {
    /// Participant identifier being mapped.
    pub participant: Id,
    /// Structured path inside the participant payload.
    pub path: String,
}

/// Scope in which a witness is valid.
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Scope {
    /// Structure identifiers covered by the scope.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub structure_ids: Vec<Id>,
    /// Optional textual boundary for downstream-specific scopes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub boundary: Option<String>,
}

/// Reviewable evidence for what is shared inside a correspondence.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OverlapWitness {
    /// Witness identifier.
    pub id: Id,
    /// Witness kind.
    pub witness_kind: OverlapWitnessKind,
    /// Concrete shared structure.
    pub shared_structure: SharedStructure,
    /// Participant-to-shared-structure mappings.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub participant_mappings: Vec<ParticipantMapping>,
    /// Scope in which the witness applies.
    #[serde(default, skip_serializing_if = "Scope::is_empty")]
    pub scope: Scope,
    /// Context in which the witness is valid.
    pub context: Id,
    /// Evidence identifiers supporting the witness.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<Id>,
    /// Confidence in the witness.
    pub confidence: Confidence,
    /// Review status of this witness.
    pub status: ReviewStatus,
}

impl OverlapWitness {
    /// Returns true when this witness can support an accepted semantic overlap.
    #[must_use]
    pub fn supports_accepted_semantic_overlap(&self) -> bool {
        self.witness_kind.supports_accepted_semantic_overlap()
    }
}

/// Reviewable evidence for what differs inside a correspondence.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DifferenceWitness {
    /// Witness identifier.
    pub id: Id,
    /// Difference kind.
    pub difference_kind: DifferenceKind,
    /// Concrete differing structure.
    pub differing_structure: DifferingStructure,
    /// Participant-to-difference mappings.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub participant_mappings: Vec<ParticipantMapping>,
    /// Difference severity.
    pub severity: DifferenceSeverity,
    /// Context in which the difference is valid.
    pub context: Id,
    /// Evidence identifiers supporting the difference.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<Id>,
    /// Confidence in the difference.
    pub confidence: Confidence,
    /// Review status of this witness.
    pub status: ReviewStatus,
}

/// Result of checking an invariant during gluing.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InvariantCheckResult {
    /// Invariant identifier.
    pub invariant: Id,
    /// Stable result label such as passed, failed, skipped, or unknown.
    pub result: String,
    /// Optional diagnostic detail.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// Report of structures preserved by gluing or projection.
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PreservationReport {
    /// Preserved invariant identifiers.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub preserved_invariants: Vec<Id>,
    /// Preserved structure identifiers.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub preserved_structures: Vec<Id>,
    /// Optional summary.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}

/// Declared projection loss for correspondence views.
#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProjectionLoss {
    /// Overlap witnesses omitted from a projection.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub omitted_overlap_witnesses: Vec<Id>,
    /// Difference witnesses omitted from a projection.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub omitted_difference_witnesses: Vec<Id>,
    /// Evidence omitted from a projection.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub omitted_evidence: Vec<Id>,
    /// Contexts omitted from a projection.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub omitted_contexts: Vec<Id>,
    /// Review-status collapses made by a projection.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub collapsed_statuses: Vec<ReviewStatusCollapse>,
}

/// Explicit record that a projection collapsed one review status into another.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReviewStatusCollapse {
    /// Identifier whose status was collapsed.
    pub source_id: Id,
    /// Original review status.
    pub from: ReviewStatus,
    /// Rendered review status.
    pub to: ReviewStatus,
}

/// Stable validation finding code for correspondence invariants.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CorrespondenceValidationCode {
    /// A correspondence had fewer than two participants.
    MissingParticipants,
    /// An accepted correspondence lacked supporting evidence.
    AcceptedMissingEvidence,
    /// A conflicting correspondence lacked shared structure.
    ConflictMissingSharedStructure,
    /// Accepted semantic overlap lacked an explicit semantic witness.
    SemanticOverlapMissingExplicitWitness,
    /// A gluing success lacked a preservation report.
    GluingSuccessMissingPreservationReport,
    /// A blocking difference was silently merged.
    BlockingDifferenceSilentMerge,
}

/// One validation finding for a correspondence cell.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CorrespondenceValidationFinding {
    /// Stable finding code.
    pub code: CorrespondenceValidationCode,
    /// Field where the invariant failed.
    pub field: String,
    /// Human-readable reason.
    pub reason: String,
}

impl CorrespondenceValidationFinding {
    fn new(
        code: CorrespondenceValidationCode,
        field: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            code,
            field: field.into(),
            reason: reason.into(),
        }
    }
}

/// Multi-finding validation report for correspondence cells.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CorrespondenceValidationReport {
    /// Correspondence being validated.
    pub correspondence_id: Id,
    /// Validation findings. Empty means valid for Phase 1 invariants.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub findings: Vec<CorrespondenceValidationFinding>,
}

impl CorrespondenceValidationReport {
    /// Returns true when there are no findings.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.findings.is_empty()
    }
}

/// Result of trying to glue participants over overlap witnesses.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum GluingResult {
    /// Participants were safely glued.
    Success {
        /// Merged complex identifier when a concrete structure was materialized.
        #[serde(rename = "mergedComplex", skip_serializing_if = "Option::is_none")]
        merged_complex: Option<Id>,
        /// Preservation report for the merge.
        #[serde(rename = "preservationReport")]
        preservation_report: PreservationReport,
    },
    /// Gluing is plausible but requires review.
    Candidate {
        /// Completion candidate identifier.
        #[serde(rename = "completionCandidate")]
        completion_candidate: Id,
        /// Required review before acceptance.
        #[serde(rename = "requiredReview")]
        required_review: ReviewRequirement,
    },
    /// Gluing failed with a structured obstruction.
    Failure {
        /// Obstruction identifier.
        obstruction: Id,
    },
}

impl GluingResult {
    /// Returns the stable serde discriminant string for this result variant.
    #[must_use]
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Success { .. } => "success",
            Self::Candidate { .. } => "candidate",
            Self::Failure { .. } => "failure",
        }
    }
}

/// Full gluing attempt record.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GluingAttempt {
    /// Gluing attempt identifier.
    pub id: Id,
    /// Participants involved in the attempt.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub participants: Vec<ParticipantRef>,
    /// Overlap witness identifiers used by the attempt.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub overlap_witnesses: Vec<Id>,
    /// Difference witness identifiers used by the attempt.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub difference_witnesses: Vec<Id>,
    /// Context in which gluing was checked.
    pub context: Id,
    /// Invariant checks run during gluing.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub invariant_checks: Vec<InvariantCheckResult>,
    /// Structures preserved by the attempt.
    #[serde(default, skip_serializing_if = "PreservationReport::is_empty")]
    pub preservation_report: PreservationReport,
    /// Attempt result.
    pub result: GluingResult,
    /// Evidence supporting the attempt.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<Id>,
    /// Confidence in the attempt.
    pub confidence: Confidence,
    /// Review status of the attempt.
    pub status: ReviewStatus,
    /// Explicit review override that permits success despite blocking differences.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub override_review: Option<ReviewRequirement>,
}

/// Higher-order cell describing a correspondence between structures.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CorrespondenceCell {
    /// Correspondence identifier.
    pub id: Id,
    /// Role-labeled participants.
    pub participants: Vec<CorrespondenceParticipant>,
    /// Correspondence kind.
    pub correspondence_kind: CorrespondenceKind,
    /// Correspondence polarity.
    pub polarity: CorrespondencePolarity,
    /// Reviewable overlap witnesses.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub overlap_witnesses: Vec<OverlapWitness>,
    /// Reviewable difference witnesses.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub difference_witnesses: Vec<DifferenceWitness>,
    /// Context in which the correspondence is valid.
    pub context: Id,
    /// Evidence identifiers supporting the correspondence.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<Id>,
    /// Provenance reference.
    pub provenance: Id,
    /// Confidence in the correspondence.
    pub confidence: Confidence,
    /// Review status of the correspondence.
    pub review_status: ReviewStatus,
    /// Optional gluing attempt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gluing: Option<GluingAttempt>,
}

impl CorrespondenceCell {
    /// Returns a multi-finding validation report for Phase 1 correspondence invariants.
    #[must_use]
    pub fn validate_report(&self) -> CorrespondenceValidationReport {
        let mut findings = Vec::new();

        if self.participants.len() < 2 {
            findings.push(CorrespondenceValidationFinding::new(
                CorrespondenceValidationCode::MissingParticipants,
                "participants",
                "correspondence requires at least two participants",
            ));
        }

        if self.review_status.is_accepted() && self.evidence.is_empty() {
            findings.push(CorrespondenceValidationFinding::new(
                CorrespondenceValidationCode::AcceptedMissingEvidence,
                "evidence",
                "accepted correspondence requires supporting evidence",
            ));
        }

        if matches!(self.polarity, CorrespondencePolarity::Conflicting)
            && self.overlap_witnesses.is_empty()
        {
            findings.push(CorrespondenceValidationFinding::new(
                CorrespondenceValidationCode::ConflictMissingSharedStructure,
                "overlap_witnesses",
                "conflicting correspondence requires explicit shared structure",
            ));
        }

        if matches!(
            self.correspondence_kind,
            CorrespondenceKind::SemanticOverlap
        ) && self.review_status.is_accepted()
            && !self
                .overlap_witnesses
                .iter()
                .any(OverlapWitness::supports_accepted_semantic_overlap)
        {
            findings.push(CorrespondenceValidationFinding::new(
                CorrespondenceValidationCode::SemanticOverlapMissingExplicitWitness,
                "overlap_witnesses",
                "accepted semantic overlap requires normalized claim, predicate set, or feature set witness",
            ));
        }

        if let Some(gluing) = &self.gluing {
            findings.extend(gluing.validation_findings(&self.difference_witnesses));
        }

        CorrespondenceValidationReport {
            correspondence_id: self.id.clone(),
            findings,
        }
    }

    /// Validates Phase 1 correspondence invariants.
    pub fn validate(&self) -> Result<()> {
        let report = self.validate_report();
        if let Some(finding) = report.findings.first() {
            return Err(malformed_field(&finding.field, finding.reason.clone()));
        }

        Ok(())
    }
}

impl GluingAttempt {
    fn validation_findings(
        &self,
        differences: &[DifferenceWitness],
    ) -> Vec<CorrespondenceValidationFinding> {
        let mut findings = Vec::new();

        if let GluingResult::Success {
            preservation_report,
            ..
        } = &self.result
        {
            if preservation_report.is_empty() && self.preservation_report.is_empty() {
                findings.push(CorrespondenceValidationFinding::new(
                    CorrespondenceValidationCode::GluingSuccessMissingPreservationReport,
                    "preservation_report",
                    "gluing success requires a preservation report",
                ));
            }

            if self.override_review.is_none()
                && differences
                    .iter()
                    .any(|difference| matches!(difference.severity, DifferenceSeverity::Blocking))
            {
                findings.push(CorrespondenceValidationFinding::new(
                    CorrespondenceValidationCode::BlockingDifferenceSilentMerge,
                    "result",
                    "blocking difference prevents gluing success without explicit override review",
                ));
            }
        }

        findings
    }

    /// Validates gluing invariants that do not require external graph lookup.
    pub fn validate_with_differences(&self, differences: &[DifferenceWitness]) -> Result<()> {
        let findings = self.validation_findings(differences);
        if let Some(finding) = findings.first() {
            return Err(malformed_field(&finding.field, finding.reason.clone()));
        }

        Ok(())
    }
}

impl Scope {
    fn is_empty(&self) -> bool {
        self.structure_ids.is_empty() && self.boundary.is_none()
    }
}

impl PreservationReport {
    fn is_empty(&self) -> bool {
        self.preserved_invariants.is_empty()
            && self.preserved_structures.is_empty()
            && self.summary.is_none()
    }
}

fn required_text(field: impl Into<String>, value: impl Into<String>) -> Result<String> {
    let raw = value.into();
    let normalized = raw.trim().to_owned();

    if normalized.is_empty() {
        return Err(malformed_field(
            field,
            "value must not be empty after trimming",
        ));
    }

    Ok(normalized)
}

fn malformed_field(field: impl Into<String>, reason: impl Into<String>) -> CoreError {
    CoreError::MalformedField {
        field: field.into(),
        reason: reason.into(),
    }
}
