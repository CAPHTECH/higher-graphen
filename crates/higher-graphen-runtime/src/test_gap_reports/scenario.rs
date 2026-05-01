use higher_graphen_core::Id;
use serde::{Deserialize, Serialize};

use super::{
    TestGapChangeSet, TestGapCoverageType, TestGapDetectorContext, TestGapFactSource,
    TestGapLiftedStructure, TestGapObservedBranch, TestGapObservedChangedFile,
    TestGapObservedContext, TestGapObservedCoverage, TestGapObservedDependencyEdge,
    TestGapObservedEvidence, TestGapObservedHigherOrderCell, TestGapObservedHigherOrderIncidence,
    TestGapObservedInputLaw, TestGapObservedInputMorphism, TestGapObservedRequirement,
    TestGapObservedRiskSignal, TestGapObservedSymbol, TestGapObservedTest,
    TestGapObservedVerificationCell, TestGapRepository, TestGapSource,
};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestGapScenario {
    pub input_schema: String,
    pub source_boundary: TestGapSourceBoundary,
    pub source: TestGapSource,
    pub repository: TestGapRepository,
    pub change_set: TestGapChangeSet,
    pub changed_files: Vec<TestGapObservedChangedFile>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub symbols: Vec<TestGapObservedSymbol>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub branches: Vec<TestGapObservedBranch>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requirements: Vec<TestGapObservedRequirement>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tests: Vec<TestGapObservedTest>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub coverage: Vec<TestGapObservedCoverage>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependency_edges: Vec<TestGapObservedDependencyEdge>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub higher_order_cells: Vec<TestGapObservedHigherOrderCell>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub higher_order_incidences: Vec<TestGapObservedHigherOrderIncidence>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub morphisms: Vec<TestGapObservedInputMorphism>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub laws: Vec<TestGapObservedInputLaw>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub verification_cells: Vec<TestGapObservedVerificationCell>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub contexts: Vec<TestGapObservedContext>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<TestGapObservedEvidence>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub signals: Vec<TestGapObservedRiskSignal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detector_context: Option<TestGapDetectorContext>,
    pub lifted_structure: TestGapLiftedStructure,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestGapSourceBoundary {
    pub repository_id: Id,
    pub change_set_id: Id,
    pub base_ref: String,
    pub head_ref: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_commit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub head_commit: Option<String>,
    pub boundary: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub adapters: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub excluded_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub coverage_dimensions: Vec<TestGapCoverageType>,
    pub symbol_source: TestGapFactSource,
    pub branch_source: TestGapFactSource,
    pub test_mapping_source: TestGapFactSource,
    pub requirement_mapping_source: TestGapFactSource,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub information_loss: Vec<String>,
}
