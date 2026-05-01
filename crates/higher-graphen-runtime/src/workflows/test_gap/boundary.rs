use super::*;

pub(super) fn source_boundary(input: &TestGapInputDocument) -> TestGapSourceBoundary {
    let mut excluded_paths = input.change_set.excluded_paths.clone();
    if let Some(context) = &input.detector_context {
        for path in &context.excluded_paths {
            push_unique_string(&mut excluded_paths, path.clone());
        }
    }
    TestGapSourceBoundary {
        repository_id: input.repository.id.clone(),
        change_set_id: input.change_set.id.clone(),
        base_ref: input.change_set.base_ref.clone(),
        head_ref: input.change_set.head_ref.clone(),
        base_commit: input.change_set.base_commit.clone(),
        head_commit: input.change_set.head_commit.clone(),
        boundary: input.change_set.boundary.clone(),
        adapters: input.source.adapters.clone(),
        excluded_paths,
        coverage_dimensions: unique_coverage_dimensions(input),
        symbol_source: if input.symbols.is_empty() {
            TestGapFactSource::Unavailable
        } else {
            TestGapFactSource::AdapterSupplied
        },
        branch_source: if input.branches.is_empty() {
            TestGapFactSource::Unavailable
        } else {
            TestGapFactSource::AdapterSupplied
        },
        test_mapping_source: if input.tests.iter().any(|test| {
            !test.target_ids.is_empty()
                || !test.branch_ids.is_empty()
                || !test.requirement_ids.is_empty()
        }) {
            TestGapFactSource::AdapterSupplied
        } else {
            TestGapFactSource::Unavailable
        },
        requirement_mapping_source: if input
            .requirements
            .iter()
            .any(|requirement| !requirement.implementation_ids.is_empty())
        {
            TestGapFactSource::AdapterSupplied
        } else {
            TestGapFactSource::Unavailable
        },
        information_loss: source_boundary_information_loss(input),
    }
}

pub(super) fn source_boundary_information_loss(input: &TestGapInputDocument) -> Vec<String> {
    let mut loss = vec![
        "raw source bodies omitted".to_owned(),
        "full diffs summarized to changed files, symbols, and supplied branch metadata".to_owned(),
        "candidate suggestions are unreviewed detector inference".to_owned(),
    ];
    if input.coverage.is_empty() {
        loss.push("coverage data absent from bounded snapshot".to_owned());
    }
    if input.branches.is_empty() {
        loss.push("branch extraction unavailable in bounded snapshot".to_owned());
    }
    if input.higher_order_cells.iter().any(|cell| {
        cell.cell_type.starts_with("rust_") || cell.cell_type.starts_with("json_schema")
    }) {
        loss.push(
            "semantic cells are AST/schema structural observations; typed MIR-level equivalence and full behavior proofs are not embedded"
                .to_owned(),
        );
    }
    if input
        .tests
        .iter()
        .any(|test| !accepts_test_kind(input, test.test_type))
    {
        loss.push("some represented tests are outside the accepted verification policy".to_owned());
    }
    loss
}

pub(super) fn unique_coverage_dimensions(
    input: &TestGapInputDocument,
) -> Vec<crate::test_gap_reports::TestGapCoverageType> {
    let mut dimensions = Vec::new();
    for coverage in &input.coverage {
        if !dimensions.contains(&coverage.coverage_type) {
            dimensions.push(coverage.coverage_type);
        }
    }
    dimensions
}
