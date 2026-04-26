use higher_graphen_core::{Id, Provenance, ReviewStatus, Severity};
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Map, Value};
use std::{fmt, str::FromStr};

pub const NATIVE_CASE_SPACE_SCHEMA: &str = "highergraphen.case.space.v1";
pub const NATIVE_CASE_SPACE_SCHEMA_VERSION: u32 = 1;
pub const NATIVE_MORPHISM_LOG_ENTRY_SCHEMA: &str = "highergraphen.case.morphism_log_entry.v1";

const CUSTOM_PREFIX: &str = "custom:";

pub type MorphismLog = Vec<MorphismLogEntry>;

macro_rules! impl_custom_enum {
    ($name:ident, { $($value:literal => $variant:ident),+ $(,)? }) => {
        impl $name {
            pub fn serialized_value(&self) -> String {
                match self {
                    $(Self::$variant => $value.to_owned(),)+
                    Self::Custom(extension) => format!("{CUSTOM_PREFIX}{extension}"),
                }
            }
        }

        impl FromStr for $name {
            type Err = String;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                match value {
                    $($value => Ok(Self::$variant),)+
                    custom if custom.starts_with(CUSTOM_PREFIX) => {
                        let extension = &custom[CUSTOM_PREFIX.len()..];
                        if extension.trim().is_empty() {
                            Err(format!("{value:?} has an empty custom extension"))
                        } else {
                            Ok(Self::Custom(extension.to_owned()))
                        }
                    }
                    unknown => Err(format!("unsupported {} value {unknown:?}", stringify!($name))),
                }
            }
        }

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.serialize_str(&self.serialized_value())
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value = String::deserialize(deserializer)?;
                Self::from_str(&value).map_err(de::Error::custom)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.serialized_value())
            }
        }
    };
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CaseSpace {
    pub schema: String,
    pub schema_version: u32,
    pub case_space_id: Id,
    pub space_id: Id,
    pub case_cells: Vec<CaseCell>,
    pub case_relations: Vec<CaseRelation>,
    pub morphism_log: MorphismLog,
    pub projections: Vec<Projection>,
    pub revision: Revision,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub close_policy_id: Option<Id>,
    pub metadata: Map<String, Value>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CaseCell {
    pub id: Id,
    pub cell_type: CaseCellType,
    pub space_id: Id,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    pub lifecycle: CaseCellLifecycle,
    pub source_ids: Vec<Id>,
    pub structure_ids: Vec<Id>,
    pub provenance: Provenance,
    pub metadata: Map<String, Value>,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CaseCellType {
    Case,
    Scenario,
    Goal,
    Work,
    Decision,
    Event,
    Evidence,
    Proof,
    Review,
    Obstruction,
    Completion,
    Projection,
    Revision,
    Morphism,
    ExternalRef,
    Custom(String),
}

impl_custom_enum!(
    CaseCellType,
    {
        "case" => Case,
        "scenario" => Scenario,
        "goal" => Goal,
        "work" => Work,
        "decision" => Decision,
        "event" => Event,
        "evidence" => Evidence,
        "proof" => Proof,
        "review" => Review,
        "obstruction" => Obstruction,
        "completion" => Completion,
        "projection" => Projection,
        "revision" => Revision,
        "morphism" => Morphism,
        "external_ref" => ExternalRef
    }
);

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CaseCellLifecycle {
    Proposed,
    Active,
    Waiting,
    Resolved,
    Retired,
    Accepted,
    Rejected,
    Superseded,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CaseRelation {
    pub id: Id,
    pub relation_type: CaseRelationType,
    pub relation_strength: RelationStrength,
    pub from_id: Id,
    pub to_id: Id,
    pub evidence_ids: Vec<Id>,
    pub source_ids: Vec<Id>,
    pub provenance: Provenance,
    pub metadata: Map<String, Value>,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CaseRelationType {
    DependsOn,
    WaitsFor,
    RequiresEvidence,
    RequiresProof,
    SatisfiesEvidenceRequirement,
    Verifies,
    Covers,
    Exercises,
    Blocks,
    Unblocks,
    Contradicts,
    Invalidates,
    Completes,
    DerivesFrom,
    Refines,
    ProjectsTo,
    TransitionsTo,
    CorrespondsTo,
    Accepts,
    Rejects,
    Supersedes,
    Custom(String),
}

impl_custom_enum!(
    CaseRelationType,
    {
        "depends_on" => DependsOn,
        "waits_for" => WaitsFor,
        "requires_evidence" => RequiresEvidence,
        "requires_proof" => RequiresProof,
        "satisfies_evidence_requirement" => SatisfiesEvidenceRequirement,
        "verifies" => Verifies,
        "covers" => Covers,
        "exercises" => Exercises,
        "blocks" => Blocks,
        "unblocks" => Unblocks,
        "contradicts" => Contradicts,
        "invalidates" => Invalidates,
        "completes" => Completes,
        "derives_from" => DerivesFrom,
        "refines" => Refines,
        "projects_to" => ProjectsTo,
        "transitions_to" => TransitionsTo,
        "corresponds_to" => CorrespondsTo,
        "accepts" => Accepts,
        "rejects" => Rejects,
        "supersedes" => Supersedes
    }
);

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationStrength {
    Hard,
    Soft,
    Diagnostic,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CaseMorphism {
    pub morphism_id: Id,
    pub morphism_type: CaseMorphismType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_revision_id: Option<Id>,
    pub target_revision_id: Id,
    pub added_ids: Vec<Id>,
    pub updated_ids: Vec<Id>,
    pub retired_ids: Vec<Id>,
    pub preserved_ids: Vec<Id>,
    pub violated_invariant_ids: Vec<Id>,
    pub review_status: ReviewStatus,
    pub evidence_ids: Vec<Id>,
    pub source_ids: Vec<Id>,
    pub metadata: Map<String, Value>,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CaseMorphismType {
    Create,
    Update,
    Retire,
    Relate,
    Unrelate,
    Review,
    EvidenceAttach,
    CompletionAccept,
    CompletionReject,
    Projection,
    Migration,
    Close,
    Custom(String),
}

impl_custom_enum!(
    CaseMorphismType,
    {
        "create" => Create,
        "update" => Update,
        "retire" => Retire,
        "relate" => Relate,
        "unrelate" => Unrelate,
        "review" => Review,
        "evidence_attach" => EvidenceAttach,
        "completion_accept" => CompletionAccept,
        "completion_reject" => CompletionReject,
        "projection" => Projection,
        "migration" => Migration,
        "close" => Close
    }
);

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MorphismLogEntry {
    pub schema: String,
    pub schema_version: u32,
    pub case_space_id: Id,
    pub sequence: u64,
    pub entry_id: Id,
    pub morphism_id: Id,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_revision_id: Option<Id>,
    pub target_revision_id: Id,
    pub morphism: CaseMorphism,
    pub actor_id: Id,
    pub recorded_at: String,
    pub provenance: Provenance,
    pub source_ids: Vec<Id>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_entry_hash: Option<String>,
    pub replay_checksum: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Revision {
    pub revision_id: Id,
    pub case_space_id: Id,
    pub applied_entry_ids: Vec<Id>,
    pub applied_morphism_ids: Vec<Id>,
    pub checksum: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_revision_id: Option<Id>,
    pub created_at: String,
    pub source_ids: Vec<Id>,
    pub metadata: Map<String, Value>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Projection {
    pub projection_id: Id,
    pub audience: ProjectionAudience,
    pub revision_id: Id,
    pub represented_cell_ids: Vec<Id>,
    pub represented_relation_ids: Vec<Id>,
    pub omitted_cell_ids: Vec<Id>,
    pub omitted_relation_ids: Vec<Id>,
    pub information_loss: Vec<ProjectionLoss>,
    pub allowed_operations: Vec<String>,
    pub source_ids: Vec<Id>,
    pub warnings: Vec<ProjectionWarning>,
    pub metadata: Map<String, Value>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectionAudience {
    HumanReview,
    AiAgent,
    Audit,
    System,
    Migration,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectionLoss {
    pub description: String,
    pub represented_ids: Vec<Id>,
    pub omitted_ids: Vec<Id>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectionWarning {
    HiddenBlocker,
    HiddenUnreviewedInference,
    HiddenCloseInvariantFailure,
    InformationLoss,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewRecord {
    pub review_id: Id,
    pub target_ids: Vec<Id>,
    pub action: ReviewAction,
    pub outcome_review_status: ReviewStatus,
    pub reviewer_id: Id,
    pub reason: String,
    pub evidence_ids: Vec<Id>,
    pub source_ids: Vec<Id>,
    pub reviewed_at: String,
    pub provenance: Provenance,
    pub metadata: Map<String, Value>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewAction {
    Accept,
    Reject,
    Reopen,
    Waive,
    Defer,
    Supersede,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceBoundary {
    SourceBacked,
    Inferred,
    ReviewPromoted,
    Rejected,
    Contradicting,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ClosePolicy {
    pub policy_id: Id,
    pub required_goal_ids: Vec<Id>,
    pub required_projection_audiences: Vec<ProjectionAudience>,
    pub invariants: Vec<CloseInvariant>,
    pub metadata: Map<String, Value>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CloseInvariant {
    pub invariant_id: Id,
    pub invariant_type: CloseInvariantType,
    pub severity: Severity,
    pub description: String,
    pub required: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CloseInvariantType {
    NoHardObstructions,
    GoalsCovered,
    EvidenceAccepted,
    CompletionsReviewed,
    MorphismsReviewed,
    ProjectionsDiscloseLoss,
    BaseRevisionMatches,
    ReplayChecksumMatches,
    MigrationSourceRecorded,
    ValidationEvidenceNamed,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CloseCheck {
    pub check_id: Id,
    pub case_space_id: Id,
    pub revision_id: Id,
    pub close_policy_id: Id,
    pub closable: bool,
    pub invariant_results: Vec<CloseInvariantResult>,
    pub completion_candidate_ids: Vec<Id>,
    pub evidence_ids: Vec<Id>,
    pub source_ids: Vec<Id>,
    pub provenance: Provenance,
    pub metadata: Map<String, Value>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CloseInvariantResult {
    pub invariant_id: Id,
    pub passed: bool,
    pub severity: Severity,
    pub witness_ids: Vec<Id>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use higher_graphen_core::{Confidence, SourceKind, SourceRef};

    const NATIVE_EXAMPLE: &str =
        include_str!("../../../schemas/casegraphen/native.case.space.example.json");

    #[test]
    fn native_case_space_example_deserializes() {
        let space: CaseSpace =
            serde_json::from_str(NATIVE_EXAMPLE).expect("native case space example");

        assert_eq!(space.schema, NATIVE_CASE_SPACE_SCHEMA);
        assert_eq!(space.schema_version, NATIVE_CASE_SPACE_SCHEMA_VERSION);
        assert_eq!(space.case_cells.len(), 5);
        assert_eq!(space.case_relations.len(), 4);
        assert_eq!(space.morphism_log.len(), 1);
        assert_eq!(space.projections.len(), 2);
    }

    #[test]
    fn native_model_rejects_unknown_top_level_fields() {
        let mut value: Value =
            serde_json::from_str(NATIVE_EXAMPLE).expect("native case space example value");
        value["ready_cell_ids"] = Value::Array(Vec::new());

        assert!(serde_json::from_value::<CaseSpace>(value).is_err());
    }

    #[test]
    fn native_model_rejects_unknown_nested_fields() {
        let mut value: Value =
            serde_json::from_str(NATIVE_EXAMPLE).expect("native case space example value");
        value["case_cells"][0]["ready"] = Value::Bool(true);

        assert!(serde_json::from_value::<CaseSpace>(value).is_err());
    }

    #[test]
    fn review_and_evidence_boundaries_round_trip() {
        let review = ReviewRecord {
            review_id: id("review:accept-evidence"),
            target_ids: vec![id("evidence:source-backed-doc")],
            action: ReviewAction::Accept,
            outcome_review_status: ReviewStatus::Accepted,
            reviewer_id: id("reviewer:native-lead"),
            reason: "Source-backed evidence is sufficient.".to_owned(),
            evidence_ids: vec![id("evidence:source-backed-doc")],
            source_ids: vec![id("source:native-design")],
            reviewed_at: "2026-04-26T01:00:00Z".to_owned(),
            provenance: provenance(SourceKind::Human, ReviewStatus::Accepted),
            metadata: Map::new(),
        };
        let boundary = EvidenceBoundary::ReviewPromoted;

        let encoded_review = serde_json::to_string(&review).expect("serialize review");
        let encoded_boundary = serde_json::to_string(&boundary).expect("serialize boundary");

        assert_eq!(
            serde_json::from_str::<ReviewRecord>(&encoded_review).expect("deserialize review"),
            review
        );
        assert_eq!(
            serde_json::from_str::<EvidenceBoundary>(&encoded_boundary)
                .expect("deserialize evidence boundary"),
            boundary
        );
    }

    #[test]
    fn native_case_space_round_trips() {
        let space: CaseSpace =
            serde_json::from_str(NATIVE_EXAMPLE).expect("native case space example");
        let round_trip: CaseSpace =
            serde_json::from_str(&serde_json::to_string(&space).expect("serialize case space"))
                .expect("deserialize case space");

        assert_eq!(round_trip, space);
    }

    #[test]
    fn custom_extension_enums_require_non_empty_suffix() {
        assert_eq!(
            serde_json::from_value::<CaseCellType>(Value::String("custom:risk".to_owned()))
                .expect("custom cell type"),
            CaseCellType::Custom("risk".to_owned())
        );
        assert!(
            serde_json::from_value::<CaseMorphismType>(Value::String("custom:".to_owned()))
                .is_err()
        );
    }

    fn provenance(kind: SourceKind, review_status: ReviewStatus) -> Provenance {
        Provenance::new(
            SourceRef::new(kind),
            Confidence::new(1.0).expect("confidence"),
        )
        .with_review_status(review_status)
    }

    fn id(value: &str) -> Id {
        Id::new(value).expect("fixture id")
    }
}
