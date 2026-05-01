use higher_graphen_core::{Confidence, Id, Severity, SourceKind};
use higher_graphen_structure::space::IncidenceOrientation;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{
    TestGapBranchType, TestGapChangeType, TestGapContextType, TestGapCoverageStatus,
    TestGapCoverageType, TestGapDependencyRelationType, TestGapEvidenceType,
    TestGapRequirementType, TestGapRiskSignalType, TestGapSymbolKind, TestGapTestType,
    TestGapVisibility,
};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestGapInputDocument {
    pub schema: String,
    pub source: TestGapSource,
    pub repository: TestGapRepository,
    pub change_set: TestGapChangeSet,
    pub changed_files: Vec<TestGapInputChangedFile>,
    #[serde(default)]
    pub symbols: Vec<TestGapInputSymbol>,
    #[serde(default)]
    pub branches: Vec<TestGapInputBranch>,
    #[serde(default)]
    pub requirements: Vec<TestGapInputRequirement>,
    #[serde(default)]
    pub tests: Vec<TestGapInputTest>,
    #[serde(default)]
    pub coverage: Vec<TestGapInputCoverage>,
    #[serde(default)]
    pub dependency_edges: Vec<TestGapInputDependencyEdge>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub higher_order_cells: Vec<TestGapHigherOrderCell>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub higher_order_incidences: Vec<TestGapHigherOrderIncidence>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub morphisms: Vec<TestGapInputMorphism>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub laws: Vec<TestGapInputLaw>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub verification_cells: Vec<TestGapVerificationCell>,
    #[serde(default)]
    pub contexts: Vec<TestGapInputContext>,
    #[serde(default)]
    pub evidence: Vec<TestGapInputEvidence>,
    #[serde(default)]
    pub signals: Vec<TestGapInputRiskSignal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detector_context: Option<TestGapDetectorContext>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestGapSource {
    pub kind: SourceKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub captured_at: Option<String>,
    pub confidence: Confidence,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub adapters: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestGapRepository {
    pub id: Id,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_branch: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestGapChangeSet {
    pub id: Id,
    pub base_ref: String,
    pub head_ref: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_commit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub head_commit: Option<String>,
    pub boundary: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub excluded_paths: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestGapInputChangedFile {
    pub id: Id,
    pub path: String,
    pub change_type: TestGapChangeType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    pub additions: u32,
    pub deletions: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub symbol_ids: Vec<Id>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub context_ids: Vec<Id>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_ids: Vec<Id>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestGapInputSymbol {
    pub id: Id,
    pub file_id: Id,
    pub name: String,
    pub kind: TestGapSymbolKind,
    pub visibility: TestGapVisibility,
    #[serde(default)]
    pub public_api: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_start: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_end: Option<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub branch_ids: Vec<Id>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requirement_ids: Vec<Id>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub context_ids: Vec<Id>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_ids: Vec<Id>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestGapInputBranch {
    pub id: Id,
    pub symbol_id: Id,
    pub branch_type: TestGapBranchType,
    pub summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub boundary_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub representative_value: Option<Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requirement_ids: Vec<Id>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_ids: Vec<Id>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestGapInputRequirement {
    pub id: Id,
    pub requirement_type: TestGapRequirementType,
    pub summary: String,
    #[serde(default)]
    pub in_scope: bool,
    #[serde(default)]
    pub bug_fix: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub implementation_ids: Vec<Id>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_ids: Vec<Id>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_verification: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestGapInputTest {
    pub id: Id,
    pub name: String,
    pub test_type: TestGapTestType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_id: Option<Id>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub target_ids: Vec<Id>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub branch_ids: Vec<Id>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requirement_ids: Vec<Id>,
    #[serde(default)]
    pub is_regression: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub context_ids: Vec<Id>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_ids: Vec<Id>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestGapInputCoverage {
    pub id: Id,
    pub coverage_type: TestGapCoverageType,
    pub target_id: Id,
    pub status: TestGapCoverageStatus,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub covered_by_test_ids: Vec<Id>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_ids: Vec<Id>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<Confidence>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestGapInputDependencyEdge {
    pub id: Id,
    pub from_id: Id,
    pub to_id: Id,
    pub relation_type: TestGapDependencyRelationType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orientation: Option<IncidenceOrientation>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_ids: Vec<Id>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<Confidence>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestGapHigherOrderCell {
    pub id: Id,
    pub cell_type: String,
    pub label: String,
    #[serde(default)]
    pub dimension: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub context_ids: Vec<Id>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_ids: Vec<Id>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<Confidence>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestGapHigherOrderIncidence {
    pub id: Id,
    pub from_id: Id,
    pub to_id: Id,
    pub relation_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orientation: Option<IncidenceOrientation>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_ids: Vec<Id>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<Confidence>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestGapInputMorphism {
    pub id: Id,
    pub morphism_type: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_ids: Vec<Id>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub target_ids: Vec<Id>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub law_ids: Vec<Id>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requirement_ids: Vec<Id>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_verification: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<Confidence>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestGapInputLaw {
    pub id: Id,
    pub summary: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub applies_to_ids: Vec<Id>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requirement_ids: Vec<Id>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_ids: Vec<Id>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_verification: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<Confidence>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestGapVerificationCell {
    pub id: Id,
    pub name: String,
    pub verification_type: String,
    pub test_type: TestGapTestType,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub target_ids: Vec<Id>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requirement_ids: Vec<Id>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub law_ids: Vec<Id>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub morphism_ids: Vec<Id>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_ids: Vec<Id>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<Confidence>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestGapInputContext {
    pub id: Id,
    pub name: String,
    pub context_type: TestGapContextType,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_ids: Vec<Id>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestGapInputEvidence {
    pub id: Id,
    pub evidence_type: TestGapEvidenceType,
    pub summary: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_ids: Vec<Id>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<Confidence>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestGapInputRiskSignal {
    pub id: Id,
    pub signal_type: TestGapRiskSignalType,
    pub summary: String,
    pub source_ids: Vec<Id>,
    pub severity: Severity,
    pub confidence: Confidence,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestGapDetectorContext {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_focus: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub excluded_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub test_kinds: Vec<TestGapTestType>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub declared_obligation_ids: Vec<Id>,
}
