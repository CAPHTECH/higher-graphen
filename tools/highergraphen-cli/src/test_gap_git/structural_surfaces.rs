fn push_test_semantics_structural_model(
    model: &mut StructuralModel,
    changes: &[GitChange],
    diff_evidence_id: &Id,
) -> Result<(), String> {
    include!("test_semantics_surface_body.rs")
}

fn push_pr_review_structural_model(
    model: &mut StructuralModel,
    changes: &[GitChange],
    diff_evidence_id: &Id,
) -> Result<(), String> {
    push_structural_symbol(
        model,
        changes,
        diff_evidence_id,
        "tools/highergraphen-cli/src/pr_review_git.rs",
        "adapter:pr-review:git-input",
        "pr-review git input adapter cell",
        "pr_review_git::input_from_git",
        TestGapSymbolKind::Module,
    )?;
    push_structural_symbol(
        model,
        changes,
        diff_evidence_id,
        "tools/highergraphen-cli/src/pr_review_git_support.rs",
        "adapter:pr-review:git-support",
        "pr-review git parsing support cell",
        "pr_review_git_support diff parsing helpers",
        TestGapSymbolKind::Module,
    )?;
    push_structural_symbol(
        model,
        changes,
        diff_evidence_id,
        "tools/highergraphen-cli/src/pr_review_structural.rs",
        "adapter:pr-review:structural-boundary",
        "pr-review structural boundary analyzer adapter cell",
        "pr_review_structural::changed_structural_boundary",
        TestGapSymbolKind::Module,
    )?;

    push_law_symbol(
        model,
        changes,
        diff_evidence_id,
        &["tools/highergraphen-cli/src/pr_review_git.rs"],
        "law:pr-review:input-from-git-emits-bounded-snapshot",
        "pr-review input from-git emits a bounded provider-neutral review target snapshot",
    )?;
    push_law_symbol(
        model,
        changes,
        diff_evidence_id,
        &["tools/highergraphen-cli/src/pr_review_git_support.rs"],
        "law:pr-review:git-parser-handles-rename-and-quoted-paths",
        "pr-review git support parses rename, quoted path, and numstat edge cases deterministically",
    )?;
    push_law_symbol(
        model,
        changes,
        diff_evidence_id,
        &["tools/highergraphen-cli/src/pr_review_structural.rs"],
        "law:pr-review:structural-detects-boundary-incidence-composition",
        "pr-review structural analysis detects boundary, incidence, and composition observations",
    )?;
    push_law_symbol(
        model,
        changes,
        diff_evidence_id,
        &[
            "tools/highergraphen-cli/src/pr_review_git.rs",
            "tools/highergraphen-cli/src/pr_review_git_support.rs",
        ],
        "law:pr-review:recommendations-remain-unreviewed",
        "pr-review generated targets and candidates remain unreviewed suggestions",
    )?;
    Ok(())
}

fn push_semantic_proof_structural_model(
    model: &mut StructuralModel,
    changes: &[GitChange],
    diff_evidence_id: &Id,
) -> Result<(), String> {
    include!("semantic_proof_surface_body.rs")
}
