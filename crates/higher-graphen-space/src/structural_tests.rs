use super::*;

fn id(value: &str) -> Id {
    Id::new(value).expect("valid test id")
}

#[test]
fn structural_boundary_analysis_groups_subjects_with_boundary_like_roles() {
    let analysis = StructuralBoundaryAnalyzer::new()
        .with_observations([
            StructuralObservation::new(
                id("obs-boundary"),
                id("subject-b"),
                StructuralRole::Boundary,
            )
            .with_sources([id("source-2"), id("source-1"), id("source-2")]),
            StructuralObservation::new(
                id("obs-evidence"),
                id("subject-b"),
                StructuralRole::Evidence,
            )
            .with_source(id("source-3")),
            StructuralObservation::new(
                id("obs-composition"),
                id("subject-a"),
                StructuralRole::Composition,
            )
            .with_source(id("source-4")),
        ])
        .analyze();

    assert_eq!(analysis.signals.len(), 2);
    assert_eq!(analysis.signals[0].subject_id, id("subject-a"));
    assert_eq!(analysis.signals[0].roles, vec![StructuralRole::Composition]);
    assert_eq!(
        analysis.signals[0].observation_ids,
        vec![id("obs-composition")]
    );
    assert_eq!(analysis.signals[0].source_ids, vec![id("source-4")]);

    assert_eq!(analysis.signals[1].subject_id, id("subject-b"));
    assert_eq!(
        analysis.signals[1].roles,
        vec![StructuralRole::Boundary, StructuralRole::Evidence]
    );
    assert_eq!(
        analysis.signals[1].observation_ids,
        vec![id("obs-boundary"), id("obs-evidence")]
    );
    assert_eq!(
        analysis.signals[1].source_ids,
        vec![id("source-1"), id("source-2"), id("source-3")]
    );
}

#[test]
fn structural_boundary_analysis_ignores_subjects_without_boundary_like_roles() {
    let analysis = StructuralBoundaryAnalyzer::new()
        .with_observation(
            StructuralObservation::new(
                id("obs-contract"),
                id("subject-a"),
                StructuralRole::Contract,
            )
            .with_source(id("source-1")),
        )
        .with_observation(StructuralObservation::new(
            id("obs-projection"),
            id("subject-a"),
            StructuralRole::Projection,
        ))
        .with_observation(StructuralObservation::new(
            id("obs-incidence"),
            id("subject-b"),
            StructuralRole::Incidence,
        ))
        .analyze();

    assert_eq!(analysis.signals.len(), 1);
    assert_eq!(analysis.signals[0].subject_id, id("subject-b"));
    assert_eq!(analysis.signals[0].roles, vec![StructuralRole::Incidence]);
    assert_eq!(
        analysis.signals[0].observation_ids,
        vec![id("obs-incidence")]
    );
}
