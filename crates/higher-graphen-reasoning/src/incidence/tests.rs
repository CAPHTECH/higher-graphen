use super::*;
use higher_graphen_core::{Confidence, SourceKind, SourceRef};

fn id(value: &str) -> Id {
    Id::new(value).expect("valid id")
}

fn provenance() -> Provenance {
    Provenance::new(
        SourceRef::new(SourceKind::Ai),
        Confidence::new(0.8).expect("valid confidence"),
    )
}

fn consistent_input() -> IncidenceConsistencyInput {
    IncidenceConsistencyInput::new(id("space:advisory"), provenance())
        .with_cells([
            IncidenceCell::new(id("cell:a")).with_contexts([id("context:shared")]),
            IncidenceCell::new(id("cell:b")).with_contexts([id("context:shared")]),
        ])
        .with_incidences([
            IncidenceRelation::new(id("incidence:a-b"), id("cell:a"), id("cell:b"))
                .with_contexts([id("context:shared")]),
        ])
        .with_required_regions([RequiredRegion::new(
            id("region:shared"),
            [id("cell:a")],
            [id("context:shared")],
        )
        .expect("required region")])
}

#[test]
fn consistent_incidence_view_yields_empty_obstructions() {
    let report = IncidenceConsistencyEngine
        .check(consistent_input())
        .expect("incidence check succeeds");

    assert!(report.obstructions.is_empty());
    assert_eq!(report.input_cell_count, 2);
    assert_eq!(report.input_incidence_count, 1);
}

#[test]
fn dangling_incidence_yields_missing_morphism_obstruction() {
    let input = IncidenceConsistencyInput::new(id("space:advisory"), provenance())
        .with_cells([IncidenceCell::new(id("cell:a"))])
        .with_incidences([IncidenceRelation::new(
            id("incidence:a-missing"),
            id("cell:a"),
            id("cell:missing"),
        )]);

    let report = IncidenceConsistencyEngine
        .check(input)
        .expect("incidence check succeeds");

    assert_eq!(report.obstructions.len(), 1);
    let obstruction = &report.obstructions[0];
    assert_eq!(
        obstruction.obstruction_type,
        ObstructionType::MissingMorphism
    );
    assert!(obstruction.location_cell_ids.contains(&id("cell:missing")));
    assert!(obstruction.has_counterexample());
    assert_eq!(
        obstruction
            .counterexample
            .as_ref()
            .and_then(|counterexample| counterexample.assignments.get("missing_cell_ids")),
        Some(&"cell:missing".to_owned())
    );
}

#[test]
fn dangling_incidence_can_use_custom_obstruction_type_and_resolution() {
    let resolution = RequiredResolution::new("Supply the advisory link")
        .expect("resolution")
        .with_target_cell(id("cell:missing"));
    let input = IncidenceConsistencyInput::new(id("space:advisory"), provenance())
        .with_cells([IncidenceCell::new(id("cell:a"))])
        .with_incidences([IncidenceRelation::new(
            id("incidence:custom"),
            id("cell:a"),
            id("cell:missing"),
        )
        .with_dangling_obstruction_type(
            ObstructionType::custom("advisory:missing_link").expect("custom type"),
        )
        .with_required_resolution(resolution.clone())]);

    let report = IncidenceConsistencyEngine
        .check(input)
        .expect("incidence check succeeds");

    assert!(report.obstructions[0].obstruction_type.is_custom());
    assert_eq!(report.obstructions[0].required_resolution, Some(resolution));
}

#[test]
fn uncovered_required_region_yields_uncovered_region_obstruction() {
    let input = IncidenceConsistencyInput::new(id("space:advisory"), provenance())
        .with_cells([IncidenceCell::new(id("cell:required"))])
        .with_required_regions([RequiredRegion::new(
            id("region:required"),
            [id("cell:required")],
            Vec::<Id>::new(),
        )
        .expect("required region")]);

    let report = IncidenceConsistencyEngine
        .check(input)
        .expect("incidence check succeeds");

    assert_eq!(report.obstructions.len(), 1);
    assert_eq!(
        report.obstructions[0].obstruction_type,
        ObstructionType::UncoveredRegion
    );
    assert!(report.obstructions[0]
        .location_cell_ids
        .contains(&id("cell:required")));
}

#[test]
fn incompatible_incidence_contexts_yield_context_mismatch_obstruction() {
    let input = IncidenceConsistencyInput::new(id("space:advisory"), provenance())
        .with_cells([
            IncidenceCell::new(id("cell:a")).with_contexts([id("context:a")]),
            IncidenceCell::new(id("cell:b")).with_contexts([id("context:b")]),
        ])
        .with_incidences([IncidenceRelation::new(
            id("incidence:a-b"),
            id("cell:a"),
            id("cell:b"),
        )]);

    let report = IncidenceConsistencyEngine
        .check(input)
        .expect("incidence check succeeds");

    assert_eq!(report.obstructions.len(), 1);
    assert_eq!(
        report.obstructions[0].obstruction_type,
        ObstructionType::ContextMismatch
    );
    assert!(report.obstructions[0]
        .location_context_ids
        .contains(&id("context:a")));
    assert!(report.obstructions[0]
        .location_context_ids
        .contains(&id("context:b")));
}

#[test]
fn incidence_report_json_round_trips_and_is_deterministic() {
    let input = IncidenceConsistencyInput::new(id("space:advisory"), provenance())
        .with_cells([IncidenceCell::new(id("cell:a"))])
        .with_incidences([IncidenceRelation::new(
            id("incidence:a-missing"),
            id("cell:a"),
            id("cell:missing"),
        )])
        .with_required_regions([RequiredRegion::new(
            id("region:uncovered"),
            [id("cell:other")],
            Vec::<Id>::new(),
        )
        .expect("required region")]);

    let first = IncidenceConsistencyEngine
        .check(input.clone())
        .expect("first check succeeds");
    let second = IncidenceConsistencyEngine
        .check(input)
        .expect("second check succeeds");
    let first_json = serde_json::to_string(&first).expect("report should serialize");
    let second_json = serde_json::to_string(&second).expect("report should serialize");
    let decoded: IncidenceConsistencyReport =
        serde_json::from_str(&first_json).expect("report should deserialize");

    assert_eq!(first_json, second_json);
    assert_eq!(decoded, first);
}

#[test]
fn incidence_report_exposes_only_counts_and_obstructions() {
    let report = IncidenceConsistencyEngine
        .check(consistent_input())
        .expect("incidence check succeeds");
    let value = serde_json::to_value(report).expect("report should serialize");

    assert!(value.get("cells").is_none());
    assert!(value.get("morphisms").is_none());
    assert!(value.get("complexes").is_none());
    assert_eq!(value["input_cell_count"], 2);
    assert_eq!(value["input_incidence_count"], 1);
}
