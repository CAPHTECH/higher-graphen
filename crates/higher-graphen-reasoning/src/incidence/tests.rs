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
fn incidence_severity_overrides_default_to_medium_when_unset() {
    let dangling_input = IncidenceConsistencyInput::new(id("space:advisory"), provenance())
        .with_cells([IncidenceCell::new(id("cell:a"))])
        .with_incidences([IncidenceRelation::new(
            id("incidence:a-missing"),
            id("cell:a"),
            id("cell:missing"),
        )]);
    let custom_dangling = IncidenceConsistencyInput::new(id("space:advisory"), provenance())
        .with_cells([IncidenceCell::new(id("cell:a"))])
        .with_incidences([IncidenceRelation::new(
            id("incidence:a-missing"),
            id("cell:a"),
            id("cell:missing"),
        )
        .with_dangling_severity(Severity::High)]);

    assert_eq!(
        IncidenceConsistencyEngine
            .check(dangling_input)
            .expect("default check succeeds")
            .obstructions[0]
            .severity,
        Severity::Medium
    );
    assert_eq!(
        IncidenceConsistencyEngine
            .check(custom_dangling)
            .expect("custom check succeeds")
            .obstructions[0]
            .severity,
        Severity::High
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
fn context_mismatch_and_region_severities_are_input_supplied() {
    let mismatch_input = IncidenceConsistencyInput::new(id("space:advisory"), provenance())
        .with_cells([
            IncidenceCell::new(id("cell:a")).with_contexts([id("context:a")]),
            IncidenceCell::new(id("cell:b")).with_contexts([id("context:b")]),
        ])
        .with_incidences([
            IncidenceRelation::new(id("incidence:a-b"), id("cell:a"), id("cell:b"))
                .with_context_mismatch_severity(Severity::Critical),
        ]);
    let region_input = IncidenceConsistencyInput::new(id("space:advisory"), provenance())
        .with_cells([IncidenceCell::new(id("cell:required"))])
        .with_required_regions([RequiredRegion::new(
            id("region:required"),
            [id("cell:required")],
            Vec::<Id>::new(),
        )
        .expect("required region")
        .with_severity(Severity::Low)]);

    assert_eq!(
        IncidenceConsistencyEngine
            .check(mismatch_input)
            .expect("mismatch check succeeds")
            .obstructions[0]
            .severity,
        Severity::Critical
    );
    assert_eq!(
        IncidenceConsistencyEngine
            .check(region_input)
            .expect("region check succeeds")
            .obstructions[0]
            .severity,
        Severity::Low
    );
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
fn cross_context_allowed_suppresses_only_context_mismatch() {
    let input = IncidenceConsistencyInput::new(id("space:advisory"), provenance())
        .with_cells([
            IncidenceCell::new(id("cell:a")).with_contexts([id("context:a")]),
            IncidenceCell::new(id("cell:required")),
        ])
        .with_incidences([
            IncidenceRelation::new(id("incidence:a-b"), id("cell:a"), id("cell:b"))
                .with_cross_context_allowed(),
            IncidenceRelation::new(
                id("incidence:a-required"),
                id("cell:a"),
                id("cell:required"),
            )
            .with_cross_context_allowed(),
        ])
        .with_required_regions([RequiredRegion::new(
            id("region:uncovered"),
            [id("cell:uncovered")],
            Vec::<Id>::new(),
        )
        .expect("required region")]);

    let report = IncidenceConsistencyEngine
        .check(input)
        .expect("incidence check succeeds");

    assert_eq!(report.obstructions.len(), 2);
    assert!(report
        .obstructions
        .iter()
        .any(|obstruction| obstruction.obstruction_type == ObstructionType::MissingMorphism));
    assert!(report
        .obstructions
        .iter()
        .any(|obstruction| obstruction.obstruction_type == ObstructionType::UncoveredRegion));
    assert!(report
        .obstructions
        .iter()
        .all(|obstruction| obstruction.obstruction_type != ObstructionType::ContextMismatch));
}

#[test]
fn incidence_input_json_round_trips_optional_severity_and_flag_fields() {
    let plain = IncidenceConsistencyInput::new(id("space:advisory"), provenance())
        .with_cells([IncidenceCell::new(id("cell:a"))])
        .with_incidences([IncidenceRelation::new(
            id("incidence:a-b"),
            id("cell:a"),
            id("cell:b"),
        )])
        .with_required_regions([RequiredRegion::new(
            id("region:required"),
            [id("cell:a")],
            Vec::<Id>::new(),
        )
        .expect("required region")]);
    let rich = IncidenceConsistencyInput::new(id("space:advisory"), provenance())
        .with_cells([IncidenceCell::new(id("cell:a"))])
        .with_incidences([
            IncidenceRelation::new(id("incidence:a-b"), id("cell:a"), id("cell:b"))
                .with_dangling_severity(Severity::High)
                .with_context_mismatch_severity(Severity::Low)
                .with_cross_context_allowed(),
        ])
        .with_required_regions([RequiredRegion::new(
            id("region:required"),
            [id("cell:a")],
            Vec::<Id>::new(),
        )
        .expect("required region")
        .with_severity(Severity::Critical)]);
    let plain_json = serde_json::to_value(&plain).expect("serialize plain input");
    let rich_json = serde_json::to_value(&rich).expect("serialize rich input");

    assert!(plain_json["incidences"][0]
        .get("dangling_severity")
        .is_none());
    assert!(plain_json["incidences"][0]
        .get("context_mismatch_severity")
        .is_none());
    assert!(plain_json["incidences"][0]
        .get("allow_cross_context")
        .is_none());
    assert!(plain_json["required_regions"][0].get("severity").is_none());
    assert_eq!(rich_json["incidences"][0]["dangling_severity"], "high");
    assert_eq!(
        serde_json::from_value::<IncidenceConsistencyInput>(plain_json).expect("plain round trip"),
        plain
    );
    assert_eq!(
        serde_json::from_value::<IncidenceConsistencyInput>(rich_json).expect("rich round trip"),
        rich
    );
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
