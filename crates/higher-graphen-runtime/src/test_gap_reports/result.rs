use higher_graphen_core::{Confidence, Id, ReviewStatus, Severity};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{
    TestGapMissingType, TestGapMorphismType, TestGapObstructionType, TestGapPreservationStatus,
    TestGapStatus, TestGapTestType,
};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestGapResult {
    pub status: TestGapStatus,
    pub accepted_fact_ids: Vec<Id>,
    pub evaluated_invariant_ids: Vec<Id>,
    pub morphism_summaries: Vec<TestGapMorphismSummary>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub proof_objects: Vec<TestGapProofObject>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub counterexamples: Vec<TestGapCounterexample>,
    pub obstructions: Vec<TestGapObstruction>,
    pub completion_candidates: Vec<TestGapCompletionCandidate>,
    pub source_ids: Vec<Id>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestGapProofObject {
    pub id: Id,
    pub proof_type: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub law_ids: Vec<Id>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub morphism_ids: Vec<Id>,
    pub verified_by_ids: Vec<Id>,
    pub witness_ids: Vec<Id>,
    pub summary: String,
    pub confidence: Confidence,
    pub review_status: ReviewStatus,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestGapCounterexample {
    pub id: Id,
    pub counterexample_type: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub law_ids: Vec<Id>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub morphism_ids: Vec<Id>,
    pub path_ids: Vec<Id>,
    pub summary: String,
    pub confidence: Confidence,
    pub review_status: ReviewStatus,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestGapMorphismSummary {
    pub id: Id,
    pub morphism_type: TestGapMorphismType,
    pub source_ids: Vec<Id>,
    pub target_ids: Vec<Id>,
    pub preservation_status: TestGapPreservationStatus,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub preserved: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub loss: Vec<String>,
    pub review_status: ReviewStatus,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestGapObstruction {
    pub id: Id,
    pub obstruction_type: TestGapObstructionType,
    pub title: String,
    pub target_ids: Vec<Id>,
    pub witness: Value,
    pub invariant_ids: Vec<Id>,
    pub evidence_ids: Vec<Id>,
    pub severity: Severity,
    pub confidence: Confidence,
    pub review_status: ReviewStatus,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestGapCompletionCandidate {
    pub id: Id,
    pub candidate_type: String,
    pub missing_type: TestGapMissingType,
    pub target_ids: Vec<Id>,
    pub obstruction_ids: Vec<Id>,
    pub suggested_test_shape: TestGapSuggestedTestShape,
    pub rationale: String,
    pub provenance: TestGapCandidateProvenance,
    pub severity: Severity,
    pub confidence: Confidence,
    pub review_status: ReviewStatus,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestGapSuggestedTestShape {
    pub test_name: String,
    pub test_kind: TestGapTestType,
    pub setup: String,
    pub inputs: Value,
    pub expected_behavior: String,
    pub assertions: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fixture_notes: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestGapCandidateProvenance {
    pub source_ids: Vec<Id>,
    pub extraction_method: String,
}
