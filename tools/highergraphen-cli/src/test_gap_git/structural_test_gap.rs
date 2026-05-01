fn structural_model_for_changes(
    changes: &[GitChange],
    diff_evidence_id: &Id,
) -> Result<StructuralModel, String> {
    include!("structural_test_gap_body.rs")
}
