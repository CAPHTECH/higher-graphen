#[path = "pr_review_git_support.rs"]
#[allow(dead_code)]
#[allow(clippy::duplicate_mod)]
mod pr_review_git_support;

use self::pr_review_git_support::*;
use crate::rust_test_semantics::extract_rust_test_semantics;
use higher_graphen_core::{Id, Severity, SourceKind};
use higher_graphen_runtime::{
    PrReviewTargetChangeType, TestGapChangeSet, TestGapChangeType, TestGapContextType,
    TestGapCoverageStatus, TestGapCoverageType, TestGapDependencyRelationType,
    TestGapDetectorContext, TestGapEvidenceType, TestGapHigherOrderCell,
    TestGapHigherOrderIncidence, TestGapInputChangedFile, TestGapInputContext,
    TestGapInputCoverage, TestGapInputDependencyEdge, TestGapInputDocument, TestGapInputEvidence,
    TestGapInputLaw, TestGapInputMorphism, TestGapInputRequirement, TestGapInputRiskSignal,
    TestGapInputSymbol, TestGapInputTest, TestGapRepository, TestGapRequirementType,
    TestGapRiskSignalType, TestGapSource, TestGapSymbolKind, TestGapTestType,
    TestGapVerificationCell, TestGapVisibility,
};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use syn::visit::Visit;

include!("test_gap_git/input.rs");
include!("test_gap_git/types.rs");
include!("test_gap_git/path_scan.rs");
include!("test_gap_git/symbols.rs");
include!("test_gap_git/structural_test_gap.rs");
include!("test_gap_git/structural_surfaces.rs");
include!("test_gap_git/semantic_models.rs");
include!("test_gap_git/binding_rules.rs");
include!("test_gap_git/rust_test_content.rs");
include!("test_gap_git/rust_semantics.rs");
include!("test_gap_git/semantic_helpers.rs");
include!("test_gap_git/requirements.rs");
include!("test_gap_git/tests_coverage.rs");
include!("test_gap_git/signals.rs");
include!("test_gap_git/path_helpers.rs");
