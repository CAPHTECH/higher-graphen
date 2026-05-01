use serde::{Deserialize, Serialize};

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
