use higher_graphen_core::{Confidence, ReviewStatus};
use serde::{Deserialize, Serialize};

use super::{
    TestGapHigherOrderCell, TestGapHigherOrderIncidence, TestGapInputBranch,
    TestGapInputChangedFile, TestGapInputContext, TestGapInputCoverage, TestGapInputDependencyEdge,
    TestGapInputEvidence, TestGapInputLaw, TestGapInputMorphism, TestGapInputRequirement,
    TestGapInputRiskSignal, TestGapInputSymbol, TestGapInputTest, TestGapVerificationCell,
};

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
pub struct TestGapObservedHigherOrderCell {
    #[serde(flatten)]
    pub record: TestGapHigherOrderCell,
    pub review_status: ReviewStatus,
    pub confidence: Confidence,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestGapObservedHigherOrderIncidence {
    #[serde(flatten)]
    pub record: TestGapHigherOrderIncidence,
    pub review_status: ReviewStatus,
    pub confidence: Confidence,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestGapObservedInputMorphism {
    #[serde(flatten)]
    pub record: TestGapInputMorphism,
    pub review_status: ReviewStatus,
    pub confidence: Confidence,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestGapObservedInputLaw {
    #[serde(flatten)]
    pub record: TestGapInputLaw,
    pub review_status: ReviewStatus,
    pub confidence: Confidence,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestGapObservedVerificationCell {
    #[serde(flatten)]
    pub record: TestGapVerificationCell,
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
