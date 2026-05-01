{
    push_structural_symbol(
        model,
        changes,
        diff_evidence_id,
        "tools/highergraphen-cli/src/test_semantics_interpretation.rs",
        "adapter:test-semantics:interpretation",
        "test-semantics interpretation adapter cell",
        "test_semantics_interpretation::interpret",
        TestGapSymbolKind::Module,
    )?;
    push_structural_symbol(
        model,
        changes,
        diff_evidence_id,
        "tools/highergraphen-cli/src/test_semantics_review.rs",
        "adapter:test-semantics:review",
        "test-semantics interpretation review adapter cell",
        "test_semantics_review::review",
        TestGapSymbolKind::Module,
    )?;
    push_structural_symbol(
        model,
        changes,
        diff_evidence_id,
        "tools/highergraphen-cli/src/test_semantics_verification.rs",
        "adapter:test-semantics:verification",
        "test-semantics verification adapter cell",
        "test_semantics_verification::verify",
        TestGapSymbolKind::Module,
    )?;
    push_structural_symbol(
        model,
        changes,
        diff_evidence_id,
        "tools/highergraphen-cli/src/test_semantics_gap.rs",
        "adapter:test-semantics:gap",
        "test-semantics expected-obligation gap adapter cell",
        "test_semantics_gap::detect",
        TestGapSymbolKind::Module,
    )?;

    push_law_symbol(
        model,
        changes,
        diff_evidence_id,
        &["tools/highergraphen-cli/src/test_semantics_interpretation.rs"],
        "law:test-semantics:interpretation-candidates-remain-unreviewed",
        "test-semantics interpretation emits candidate structure without accepted coverage",
    )?;
    push_law_symbol(
        model,
        changes,
        diff_evidence_id,
        &["tools/highergraphen-cli/src/test_semantics_review.rs"],
        "law:test-semantics:review-accept-does-not-promote-coverage",
        "accepting an interpretation candidate records review without promoting coverage or proof",
    )?;
    push_law_symbol(
        model,
        changes,
        diff_evidence_id,
        &["tools/highergraphen-cli/src/test_semantics_review.rs"],
        "law:test-semantics:review-reject-does-not-promote-coverage",
        "rejecting an interpretation candidate records review without promoting coverage or proof",
    )?;
    push_law_symbol(
        model,
        changes,
        diff_evidence_id,
        &["tools/highergraphen-cli/src/test_semantics_verification.rs"],
        "law:test-semantics:verify-positive-gates-promote-coverage",
        "accepted review, passed evidence, and semantic binding together promote verified coverage",
    )?;
    push_law_symbol(
        model,
        changes,
        diff_evidence_id,
        &["tools/highergraphen-cli/src/test_semantics_verification.rs"],
        "law:test-semantics:verify-rejected-review-fails-review-gate",
        "a rejected or unaccepted review prevents test-semantics verification",
    )?;
    push_law_symbol(
        model,
        changes,
        diff_evidence_id,
        &["tools/highergraphen-cli/src/test_semantics_verification.rs"],
        "law:test-semantics:verify-missing-evidence-fails-evidence-gate",
        "test-semantics verification requires passed execution evidence linked to the candidate",
    )?;
    push_law_symbol(
        model,
        changes,
        diff_evidence_id,
        &["tools/highergraphen-cli/src/test_semantics_verification.rs"],
        "law:test-semantics:verify-missing-binding-fails-semantic-binding-gate",
        "test-semantics verification requires a candidate law, morphism, or semantic target binding",
    )?;
    push_law_symbol(
        model,
        changes,
        diff_evidence_id,
        &["tools/highergraphen-cli/src/test_semantics_verification.rs"],
        "law:test-semantics:verify-does-not-create-proof-objects",
        "test-semantics verification creates proof obligations and semantic proof inputs but no proof objects",
    )?;
    push_law_symbol(
        model,
        changes,
        diff_evidence_id,
        &["tools/highergraphen-cli/src/test_semantics_gap.rs"],
        "law:test-semantics:gap-missing-obligation-emits-candidate",
        "test-semantics gap emits missing-test obstruction and candidate for uncovered expected obligations",
    )?;
    push_law_symbol(
        model,
        changes,
        diff_evidence_id,
        &["tools/highergraphen-cli/src/test_semantics_gap.rs"],
        "law:test-semantics:gap-no-gaps-when-all-obligations-covered",
        "test-semantics gap reports no gaps when every expected obligation is verified",
    )?;
    Ok(())
}
