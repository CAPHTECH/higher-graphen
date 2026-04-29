//! Missing unit test detector report shapes.
#![allow(missing_docs)]

use crate::reports::{ProjectionViewSet, ReportEnvelope};
use higher_graphen_core::{Confidence, Id, Provenance, ReviewStatus, Severity, SourceKind};
use higher_graphen_space::IncidenceOrientation;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub type TestGapReport = ReportEnvelope<TestGapScenario, TestGapResult, TestGapProjection>;

pub type TestGapProjection = ProjectionViewSet;

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

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestGapObservedChangedFile {
    #[serde(flatten)]
    pub record: TestGapInputChangedFile,
    pub review_status: ReviewStatus,
    pub confidence: Confidence,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestGapObservedSymbol {
    #[serde(flatten)]
    pub record: TestGapInputSymbol,
    pub review_status: ReviewStatus,
    pub confidence: Confidence,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestGapObservedBranch {
    #[serde(flatten)]
    pub record: TestGapInputBranch,
    pub review_status: ReviewStatus,
    pub confidence: Confidence,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestGapObservedRequirement {
    #[serde(flatten)]
    pub record: TestGapInputRequirement,
    pub review_status: ReviewStatus,
    pub confidence: Confidence,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestGapObservedTest {
    #[serde(flatten)]
    pub record: TestGapInputTest,
    pub review_status: ReviewStatus,
    pub confidence: Confidence,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestGapObservedCoverage {
    #[serde(flatten)]
    pub record: TestGapInputCoverage,
    pub review_status: ReviewStatus,
    #[serde(rename = "accepted_confidence")]
    pub confidence: Confidence,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestGapObservedDependencyEdge {
    #[serde(flatten)]
    pub record: TestGapInputDependencyEdge,
    pub review_status: ReviewStatus,
    pub confidence: Confidence,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestGapObservedContext {
    #[serde(flatten)]
    pub record: TestGapInputContext,
    pub review_status: ReviewStatus,
    pub confidence: Confidence,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestGapObservedEvidence {
    #[serde(flatten)]
    pub record: TestGapInputEvidence,
    pub review_status: ReviewStatus,
    #[serde(rename = "accepted_confidence")]
    pub confidence: Confidence,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestGapObservedRiskSignal {
    #[serde(flatten)]
    pub record: TestGapInputRiskSignal,
    pub review_status: ReviewStatus,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestGapLiftedStructure {
    pub space: TestGapLiftedSpace,
    pub structural_summary: TestGapStructuralSummary,
    pub contexts: Vec<TestGapLiftedContext>,
    pub cells: Vec<TestGapLiftedCell>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub incidences: Vec<TestGapLiftedIncidence>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestGapLiftedSpace {
    pub id: Id,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub cell_ids: Vec<Id>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub incidence_ids: Vec<Id>,
    pub context_ids: Vec<Id>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestGapStructuralSummary {
    pub accepted_cell_count: usize,
    pub accepted_incidence_count: usize,
    pub context_count: usize,
    pub branch_count: usize,
    pub requirement_count: usize,
    pub test_count: usize,
    pub coverage_record_count: usize,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestGapLiftedContext {
    pub id: Id,
    pub space_id: Id,
    pub name: String,
    pub context_type: String,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestGapLiftedCell {
    pub id: Id,
    pub space_id: Id,
    pub dimension: u32,
    pub cell_type: String,
    pub label: String,
    pub context_ids: Vec<Id>,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestGapLiftedIncidence {
    pub id: Id,
    pub space_id: Id,
    pub from_cell_id: Id,
    pub to_cell_id: Id,
    pub relation_type: String,
    pub orientation: IncidenceOrientation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<f64>,
    pub provenance: Provenance,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestGapResult {
    pub status: TestGapStatus,
    pub accepted_fact_ids: Vec<Id>,
    pub evaluated_invariant_ids: Vec<Id>,
    pub morphism_summaries: Vec<TestGapMorphismSummary>,
    pub obstructions: Vec<TestGapObstruction>,
    pub completion_candidates: Vec<TestGapCompletionCandidate>,
    pub source_ids: Vec<Id>,
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

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TestGapChangeType {
    Added,
    Modified,
    Deleted,
    Renamed,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TestGapSymbolKind {
    Function,
    Method,
    Type,
    Module,
    PublicApi,
    Unknown,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TestGapVisibility {
    Public,
    Crate,
    Protected,
    Private,
    Unknown,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TestGapBranchType {
    Branch,
    Boundary,
    Condition,
    ErrorPath,
    StateTransition,
    PatternArm,
    Unknown,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TestGapRequirementType {
    Requirement,
    BugFix,
    Issue,
    AcceptanceCriterion,
    AdrConstraint,
    Custom,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TestGapTestType {
    Unit,
    Property,
    Integration,
    Smoke,
    E2e,
    Manual,
    Unknown,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TestGapCoverageType {
    Line,
    Branch,
    Function,
    Condition,
    Mutation,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TestGapCoverageStatus {
    Covered,
    Partial,
    Uncovered,
    Unknown,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TestGapDependencyRelationType {
    Contains,
    ImplementsRequirement,
    HasBranch,
    CoveredByTest,
    ExercisesCondition,
    DependsOn,
    Supports,
    InContext,
    Custom,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TestGapContextType {
    Repository,
    Module,
    Package,
    SymbolScope,
    TestScope,
    Domain,
    RequirementScope,
    CoverageScope,
    ReviewFocus,
    Custom,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TestGapEvidenceType {
    DiffHunk,
    Coverage,
    TestResult,
    StaticAnalysis,
    RequirementLink,
    MutationResult,
    HumanNote,
    Custom,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TestGapRiskSignalType {
    TestGap,
    BoundaryChange,
    ErrorPathChange,
    BugFix,
    PublicApiChange,
    Custom,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TestGapFactSource {
    AdapterSupplied,
    DetectorInferred,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TestGapStatus {
    GapsDetected,
    NoGapsInSnapshot,
    UnsupportedInput,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TestGapMorphismType {
    RequirementToImplementation,
    ImplementationToTest,
    BeforeToAfter,
    CandidateToAcceptedTest,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TestGapPreservationStatus {
    Preserved,
    Partial,
    Lost,
    NotEvaluated,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TestGapObstructionType {
    MissingRequirementVerification,
    MissingPublicBehaviorUnitTest,
    MissingBranchUnitTest,
    MissingBoundaryCaseUnitTest,
    MissingErrorCaseUnitTest,
    MissingRegressionTest,
    StaleOrMismatchedTestMapping,
    InsufficientTestEvidence,
    ProjectionInformationLossMissing,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TestGapMissingType {
    UnitTest,
}
