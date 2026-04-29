//! Report contracts for semantic proof certificate verification.

use crate::reports::{ProjectionViewSet, ReportEnvelope};
use higher_graphen_core::{Confidence, Id, ReviewStatus, Severity, SourceKind};
use serde::{Deserialize, Serialize};

/// Bounded semantic proof input schema identifier.
pub const SEMANTIC_PROOF_INPUT_SCHEMA: &str = "highergraphen.semantic_proof.input.v1";

/// Semantic proof report schema identifier.
pub const SEMANTIC_PROOF_REPORT_SCHEMA: &str = "highergraphen.semantic_proof.report.v1";

/// Bounded semantic proof report envelope.
pub type SemanticProofReport =
    ReportEnvelope<SemanticProofScenario, SemanticProofResult, ProjectionViewSet>;

/// Bounded semantic proof input document.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticProofInputDocument {
    /// Stable input schema identifier.
    pub schema: String,
    /// Source metadata for this proof bundle.
    pub source: SemanticProofSource,
    /// The theorem or proof obligation being evaluated.
    pub theorem: SemanticProofTheorem,
    /// Semantic cells referenced by proof obligations.
    #[serde(default)]
    pub semantic_cells: Vec<SemanticProofCell>,
    /// Semantic morphisms that must be proved or refuted.
    #[serde(default)]
    pub morphisms: Vec<SemanticProofMorphism>,
    /// Laws the theorem and morphisms must preserve.
    #[serde(default)]
    pub laws: Vec<SemanticProofLaw>,
    /// Machine-checkable proof certificates supplied by external backends.
    #[serde(default)]
    pub proof_certificates: Vec<SemanticProofCertificate>,
    /// Counterexample witnesses supplied by model checkers or symbolic execution.
    #[serde(default)]
    pub counterexamples: Vec<SemanticProofCounterexample>,
    /// Verification policy for accepting certificates.
    pub verification_policy: SemanticProofVerificationPolicy,
}

/// Source metadata for semantic proof documents.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticProofSource {
    /// Source category.
    pub kind: SourceKind,
    /// Optional stable URI.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
    /// Optional source title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Confidence for accepted supplied facts.
    pub confidence: Confidence,
    /// Adapter names that produced the input.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub adapters: Vec<String>,
}

/// Theorem or proof obligation under verification.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticProofTheorem {
    /// Stable theorem identifier.
    pub id: Id,
    /// Human-readable theorem summary.
    pub summary: String,
    /// Law IDs the theorem requires.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub law_ids: Vec<Id>,
    /// Morphism IDs covered by the theorem.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub morphism_ids: Vec<Id>,
}

/// Semantic cell extracted from typed IR, AST, schema, or proof IR.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticProofCell {
    /// Stable cell identifier.
    pub id: Id,
    /// Cell type, such as mir_block or rustdoc_item.
    pub cell_type: String,
    /// Human-readable label.
    pub label: String,
    /// Optional source IDs.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_ids: Vec<Id>,
    /// Optional confidence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<Confidence>,
}

/// Semantic morphism requiring proof.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticProofMorphism {
    /// Stable morphism identifier.
    pub id: Id,
    /// Morphism type, such as typed_signature_preservation.
    pub morphism_type: String,
    /// Source semantic endpoints.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_ids: Vec<Id>,
    /// Target semantic endpoints.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub target_ids: Vec<Id>,
    /// Laws this morphism must preserve.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub law_ids: Vec<Id>,
    /// Optional confidence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<Confidence>,
}

/// Semantic law used by the theorem or morphism.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticProofLaw {
    /// Stable law identifier.
    pub id: Id,
    /// Human-readable law summary.
    pub summary: String,
    /// IDs this law applies to.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub applies_to_ids: Vec<Id>,
    /// Optional confidence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<Confidence>,
}

/// Policy for accepting external proof certificates.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticProofVerificationPolicy {
    /// Accepted backend names.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub accepted_backends: Vec<String>,
    /// Whether input_hash is mandatory.
    #[serde(default)]
    pub require_input_hash: bool,
    /// Whether proof_hash is mandatory.
    #[serde(default)]
    pub require_proof_hash: bool,
    /// Whether certificate review_status must be accepted.
    #[serde(default)]
    pub require_accepted_review: bool,
}

/// Proof certificate supplied by an external proof backend.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticProofCertificate {
    /// Stable certificate identifier.
    pub id: Id,
    /// Certificate type, such as formal_proof or model_check.
    pub certificate_type: String,
    /// Backend name.
    pub backend: String,
    /// Backend version.
    pub backend_version: String,
    /// Theorem ID proved by the certificate.
    pub theorem_id: Id,
    /// Law IDs proved by the certificate.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub law_ids: Vec<Id>,
    /// Morphism IDs proved by the certificate.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub morphism_ids: Vec<Id>,
    /// Witness IDs used by the proof.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub witness_ids: Vec<Id>,
    /// Hash of the verified input.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_hash: Option<String>,
    /// Hash of the proof artifact.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof_hash: Option<String>,
    /// Confidence assigned by the adapter.
    pub confidence: Confidence,
    /// Review state of the certificate.
    pub review_status: ReviewStatus,
}

/// Counterexample witness supplied by a verifier.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticProofCounterexample {
    /// Stable counterexample identifier.
    pub id: Id,
    /// Counterexample type.
    pub counterexample_type: String,
    /// Theorem ID refuted by the counterexample.
    pub theorem_id: Id,
    /// Law IDs refuted by the counterexample.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub law_ids: Vec<Id>,
    /// Morphism IDs refuted by the counterexample.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub morphism_ids: Vec<Id>,
    /// Path or witness IDs.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub path_ids: Vec<Id>,
    /// Human-readable summary.
    pub summary: String,
    /// Severity assigned to the counterexample.
    pub severity: Severity,
    /// Confidence assigned by the adapter.
    pub confidence: Confidence,
    /// Review state of the counterexample.
    pub review_status: ReviewStatus,
}

/// Scenario included in the semantic proof report.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticProofScenario {
    /// Input schema.
    pub input_schema: String,
    /// Source metadata.
    pub source: SemanticProofSource,
    /// Theorem under verification.
    pub theorem: SemanticProofTheorem,
    /// Semantic cells.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub semantic_cells: Vec<SemanticProofCell>,
    /// Morphisms.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub morphisms: Vec<SemanticProofMorphism>,
    /// Laws.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub laws: Vec<SemanticProofLaw>,
    /// Proof certificates.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub proof_certificates: Vec<SemanticProofCertificate>,
    /// Counterexamples.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub counterexamples: Vec<SemanticProofCounterexample>,
    /// Verification policy.
    pub verification_policy: SemanticProofVerificationPolicy,
}

/// Semantic proof workflow status.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticProofStatus {
    /// Every theorem law and morphism obligation has an accepted certificate.
    Proved,
    /// A reviewed or supplied counterexample refutes at least one obligation.
    CounterexampleFound,
    /// Proof material is missing or rejected.
    InsufficientProof,
}

/// Accepted proof object emitted by the workflow.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticProofObject {
    /// Stable proof object ID.
    pub id: Id,
    /// Theorem IDs proved.
    pub theorem_ids: Vec<Id>,
    /// Law IDs proved.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub law_ids: Vec<Id>,
    /// Morphism IDs proved.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub morphism_ids: Vec<Id>,
    /// Certificate IDs used.
    pub certificate_ids: Vec<Id>,
    /// Witness IDs used.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub witness_ids: Vec<Id>,
    /// Human-readable summary.
    pub summary: String,
    /// Confidence.
    pub confidence: Confidence,
    /// Review state.
    pub review_status: ReviewStatus,
}

/// Verification issue emitted by the semantic proof workflow.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticProofIssue {
    /// Stable issue ID.
    pub id: Id,
    /// Issue type.
    pub issue_type: String,
    /// Target IDs.
    pub target_ids: Vec<Id>,
    /// Human-readable summary.
    pub summary: String,
    /// Severity.
    pub severity: Severity,
    /// Confidence.
    pub confidence: Confidence,
    /// Review state.
    pub review_status: ReviewStatus,
}

/// Semantic proof workflow result.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SemanticProofResult {
    /// Workflow status.
    pub status: SemanticProofStatus,
    /// Accepted certificate IDs.
    pub accepted_certificate_ids: Vec<Id>,
    /// Rejected certificate IDs.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rejected_certificate_ids: Vec<Id>,
    /// Proof objects emitted by accepted certificates.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub proof_objects: Vec<SemanticProofObject>,
    /// Counterexamples.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub counterexamples: Vec<SemanticProofCounterexample>,
    /// Issues preventing proof.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub issues: Vec<SemanticProofIssue>,
    /// Source IDs represented by the result.
    pub source_ids: Vec<Id>,
}
