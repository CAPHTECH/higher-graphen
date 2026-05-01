fn accepted_test_kinds_for_tests(tests: &[TestGapInputTest]) -> Vec<TestGapTestType> {
    let mut accepted = vec![TestGapTestType::Unit];
    for test in tests {
        if test.test_type != TestGapTestType::Unknown && !accepted.contains(&test.test_type) {
            accepted.push(test.test_type);
        }
    }
    accepted
}

fn expected_verification_label(accepted_test_kinds: &[TestGapTestType]) -> String {
    if accepted_test_kinds.contains(&TestGapTestType::Integration) {
        "unit_or_integration_test".to_owned()
    } else {
        "unit_test".to_owned()
    }
}

fn matching_requirement_ids(
    target_ids: &[Id],
    requirements: &[TestGapInputRequirement],
) -> Vec<Id> {
    requirements
        .iter()
        .filter(|requirement| {
            requirement
                .implementation_ids
                .iter()
                .any(|implementation_id| target_ids.contains(implementation_id))
        })
        .map(|requirement| requirement.id.clone())
        .collect()
}

fn test_gap_context_ids_for_path(path: &str) -> Result<Vec<Id>, String> {
    test_gap_context_descriptors_for_path(path).map(|descriptors| {
        descriptors
            .into_iter()
            .map(|(context_id, _, _)| context_id)
            .collect()
    })
}

fn test_gap_context_descriptors_for_path(
    path: &str,
) -> Result<Vec<(Id, String, TestGapContextType)>, String> {
    let mut contexts = vec![(
        id("context:repository")?,
        "Repository".to_owned(),
        TestGapContextType::Repository,
    )];
    if is_runtime_path(path) {
        contexts.push((
            id("context:runtime")?,
            "Runtime".to_owned(),
            TestGapContextType::Module,
        ));
    }
    if is_cli_path(path) {
        contexts.push((
            id("context:cli")?,
            "CLI".to_owned(),
            TestGapContextType::Module,
        ));
    }
    if is_schema_path(path) {
        contexts.push((
            id("context:schema")?,
            "Schema".to_owned(),
            TestGapContextType::RequirementScope,
        ));
    }
    if path.contains("/workflows/") {
        contexts.push((
            id("context:workflow-logic")?,
            "Workflow Logic".to_owned(),
            TestGapContextType::SymbolScope,
        ));
    }
    if is_test_path(path) {
        contexts.push((
            id("context:test-scope")?,
            "Test Scope".to_owned(),
            TestGapContextType::TestScope,
        ));
    }
    if path.starts_with("docs/") || path.starts_with("skills/") {
        contexts.push((
            id("context:agent-guidance")?,
            "Agent Guidance".to_owned(),
            TestGapContextType::ReviewFocus,
        ));
    }
    Ok(contexts)
}

fn test_gap_test_type_for_path(path: &str) -> TestGapTestType {
    if path.ends_with("_test.rs")
        || path.ends_with(".test.rs")
        || path.contains("/unit/")
        || path.contains("/unit_tests/")
    {
        TestGapTestType::Unit
    } else if path.contains("/tests/") {
        TestGapTestType::Integration
    } else {
        TestGapTestType::Unknown
    }
}

fn test_gap_test_type_for_observed_rust_test(path: &str) -> TestGapTestType {
    if is_test_path(path) {
        test_gap_test_type_for_path(path)
    } else {
        TestGapTestType::Unit
    }
}

fn is_rust_source_path(path: &str) -> bool {
    path.ends_with(".rs") && !path.starts_with("target/")
}

fn is_source_code_path(path: &str) -> bool {
    if is_test_path(path) || path.starts_with("target/") {
        return false;
    }
    matches!(
        std::path::Path::new(path)
            .extension()
            .and_then(|value| value.to_str()),
        Some(
            "rs" | "py"
                | "js"
                | "jsx"
                | "ts"
                | "tsx"
                | "go"
                | "java"
                | "kt"
                | "kts"
                | "swift"
                | "c"
                | "cc"
                | "cpp"
                | "h"
                | "hpp"
                | "cs"
                | "ex"
                | "exs"
        )
    )
}

fn is_highergraphen_structural_path(path: &str) -> bool {
    matches!(
        path,
        "tools/highergraphen-cli/src/main.rs"
            | "tools/highergraphen-cli/src/test_gap_git.rs"
            | "scripts/validate-json-contracts.py"
            | "crates/higher-graphen-runtime/src/lib.rs"
            | "crates/higher-graphen-runtime/src/reports.rs"
            | "crates/higher-graphen-runtime/src/test_gap_reports.rs"
            | "crates/higher-graphen-runtime/src/workflows/mod.rs"
            | "crates/higher-graphen-runtime/src/workflows/test_gap.rs"
            | "tools/highergraphen-cli/src/semantic_proof_artifact.rs"
            | "tools/highergraphen-cli/src/semantic_proof_backend.rs"
            | "tools/highergraphen-cli/src/semantic_proof_reinput.rs"
            | "tools/highergraphen-cli/src/test_semantics_gap.rs"
            | "tools/highergraphen-cli/src/test_semantics_interpretation.rs"
            | "tools/highergraphen-cli/src/test_semantics_review.rs"
            | "tools/highergraphen-cli/src/test_semantics_verification.rs"
            | "tools/highergraphen-cli/src/pr_review_git.rs"
            | "tools/highergraphen-cli/src/pr_review_git_support.rs"
            | "tools/highergraphen-cli/src/pr_review_structural.rs"
    )
}

fn is_test_gap_surface_path(path: &str) -> bool {
    path.contains("test_gap") || path.contains("test-gap")
}

fn is_semantic_proof_surface_path(path: &str) -> bool {
    path.contains("semantic_proof") || path.contains("semantic-proof")
}

fn is_test_semantics_surface_path(path: &str) -> bool {
    path.contains("test_semantics") || path.contains("test-semantics")
}

fn is_pr_review_surface_path(path: &str) -> bool {
    path.contains("pr_review") || path.contains("pr-review")
}

fn comparable_path_key(path: &str) -> String {
    let stem = std::path::Path::new(path)
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or(path)
        .trim_end_matches("_test")
        .trim_end_matches(".test")
        .trim_start_matches("test_")
        .to_owned();
    slug(&stem)
}

fn map_change_type(change_type: PrReviewTargetChangeType) -> TestGapChangeType {
    match change_type {
        PrReviewTargetChangeType::Added => TestGapChangeType::Added,
        PrReviewTargetChangeType::Modified => TestGapChangeType::Modified,
        PrReviewTargetChangeType::Deleted => TestGapChangeType::Deleted,
        PrReviewTargetChangeType::Renamed => TestGapChangeType::Renamed,
    }
}

trait TestGapSymbolPath {
    fn path(&self) -> &str;
}

impl TestGapSymbolPath for TestGapInputSymbol {
    fn path(&self) -> &str {
        self.path.as_deref().unwrap_or(self.name.as_str())
    }
}
