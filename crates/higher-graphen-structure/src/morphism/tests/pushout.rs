use super::*;
use crate::space::{Cell, ComplexType, Incidence, IncidenceOrientation};

#[test]
fn explicit_pushout_constructs_clean_merged_structure() {
    let left = fixture_morphism(
        "left",
        "space/source",
        "space/left",
        [
            ("cell/source-a", "cell/left-a"),
            ("cell/source-b", "cell/left-b"),
        ],
        [("rel/source-a", "rel/left-a")],
        [],
    );
    let right = fixture_morphism(
        "right",
        "space/source",
        "space/right",
        [
            ("cell/source-a", "cell/right-a"),
            ("cell/source-b", "cell/right-b"),
        ],
        [("rel/source-a", "rel/right-a")],
        [],
    );
    let left_cells = vec![
        Cell::new(id("cell/left-private"), id("space/left"), 1, "edge")
            .with_boundary_cell(id("cell/left-a")),
        Cell::new(id("cell/left-a"), id("space/left"), 0, "vertex").with_label("shared-a"),
        Cell::new(id("cell/left-b"), id("space/left"), 0, "vertex").with_context(id("ctx/b")),
    ];
    let right_cells = vec![
        Cell::new(id("cell/right-b"), id("space/right"), 0, "vertex").with_context(id("ctx/b")),
        Cell::new(id("cell/right-a"), id("space/right"), 0, "vertex").with_label("shared-a"),
    ];
    let left_incidences = vec![Incidence::new(
        id("rel/left-a"),
        id("space/left"),
        id("cell/left-a"),
        id("cell/left-b"),
        "attaches",
        IncidenceOrientation::Directed,
    )];
    let right_incidences = vec![Incidence::new(
        id("rel/right-a"),
        id("space/right"),
        id("cell/right-a"),
        id("cell/right-b"),
        "attaches",
        IncidenceOrientation::Directed,
    )];

    let PushoutOutcome::Constructed {
        construction,
        report,
    } = construct_explicit_pushout(PushoutInputs {
        left: &left,
        right: &right,
        candidate_space_id: id("space/pushout"),
        candidate_space_name: "Pushout".to_owned(),
        complex_type: ComplexType::CellComplex,
        left_cells: &left_cells,
        right_cells: &right_cells,
        left_incidences: &left_incidences,
        right_incidences: &right_incidences,
    })
    else {
        panic!("clean pushout should construct");
    };

    let merged_a = id("space/pushout/pushout/cell/left:cell/left-a+right:cell/right-a");
    let merged_b = id("space/pushout/pushout/cell/left:cell/left-b+right:cell/right-b");
    let left_private = id("space/pushout/pushout/cell/left:cell/left-private");
    let merged_rel = id("space/pushout/pushout/incidence/left:rel/left-a+right:rel/right-a");

    assert_eq!(
        construction
            .cells
            .iter()
            .map(|cell| cell.id.clone())
            .collect::<Vec<_>>(),
        vec![merged_a.clone(), merged_b.clone(), left_private.clone()]
    );
    assert_eq!(construction.cells[0].space_id, id("space/pushout"));
    assert_eq!(construction.cells[2].dimension, 1);
    assert_eq!(construction.cells[2].boundary, vec![merged_a.clone()]);
    assert_eq!(construction.incidences.len(), 1);
    assert_eq!(construction.incidences[0].id, merged_rel);
    assert_eq!(construction.incidences[0].from_cell_id, merged_a.clone());
    assert_eq!(construction.incidences[0].to_cell_id, merged_b.clone());
    assert_eq!(construction.complex.max_dimension, 1);
    assert_eq!(
        construction.space.cell_ids,
        vec![merged_a.clone(), merged_b.clone(), left_private]
    );
    assert_eq!(
        construction.space.incidence_ids,
        construction.complex.incidence_ids
    );
    assert_eq!(
        construction.space.complex_ids,
        vec![id("space/pushout/pushout/complex")]
    );
    assert_eq!(construction.space.context_ids, vec![id("ctx/b")]);
    assert_eq!(construction.review_status, ReviewStatus::Unreviewed);
    assert_eq!(
        report.identified_cell_groups,
        vec![
            IdentifiedSourceGroup {
                source_element_id: id("cell/source-a"),
                left_target_id: id("cell/left-a"),
                right_target_id: id("cell/right-a"),
            },
            IdentifiedSourceGroup {
                source_element_id: id("cell/source-b"),
                left_target_id: id("cell/left-b"),
                right_target_id: id("cell/right-b"),
            },
        ]
    );
    assert!(report.obstructions.is_empty());
}

#[test]
fn explicit_pushout_blocks_incompatible_source_space() {
    let left = fixture_morphism("left", "space/a", "space/left", [], [], []);
    let right = fixture_morphism("right", "space/b", "space/right", [], [], []);

    let report = blocked_report(construct_explicit_pushout(empty_inputs(&left, &right)));

    assert_obstruction(&report, PushoutObstructionType::IncompatibleSourceSpace);
}

#[test]
fn explicit_pushout_blocks_incomplete_mapping() {
    let left = fixture_morphism(
        "left",
        "space/source",
        "space/left",
        [("cell/source-a", "cell/left-a")],
        [],
        [],
    );
    let right = fixture_morphism("right", "space/source", "space/right", [], [], []);
    let left_cells = vec![Cell::new(id("cell/left-a"), id("space/left"), 0, "vertex")];

    let report = blocked_report(construct_explicit_pushout(PushoutInputs {
        left: &left,
        right: &right,
        candidate_space_id: id("space/pushout"),
        candidate_space_name: "Pushout".to_owned(),
        complex_type: ComplexType::CellComplex,
        left_cells: &left_cells,
        right_cells: &[],
        left_incidences: &[],
        right_incidences: &[],
    }));

    assert_obstruction(&report, PushoutObstructionType::PushoutIncomplete);
}

#[test]
fn explicit_pushout_blocks_incompatible_identification() {
    let left = fixture_morphism(
        "left",
        "space/source",
        "space/left",
        [("cell/source-a", "cell/left-a")],
        [],
        [],
    );
    let right = fixture_morphism(
        "right",
        "space/source",
        "space/right",
        [("cell/source-a", "cell/right-a")],
        [],
        [],
    );
    let left_cells = vec![Cell::new(id("cell/left-a"), id("space/left"), 0, "vertex")];
    let right_cells = vec![Cell::new(id("cell/right-a"), id("space/right"), 1, "edge")];

    let report = blocked_report(construct_explicit_pushout(PushoutInputs {
        left: &left,
        right: &right,
        candidate_space_id: id("space/pushout"),
        candidate_space_name: "Pushout".to_owned(),
        complex_type: ComplexType::CellComplex,
        left_cells: &left_cells,
        right_cells: &right_cells,
        left_incidences: &[],
        right_incidences: &[],
    }));

    assert_obstruction(&report, PushoutObstructionType::IncompatibleIdentification);
}

#[test]
fn explicit_pushout_blocks_relation_endpoint_conflict() {
    let left = fixture_morphism(
        "left",
        "space/source",
        "space/left",
        [("cell/source-from", "cell/left-from")],
        [("rel/source-a", "rel/left-a")],
        [],
    );
    let right = fixture_morphism(
        "right",
        "space/source",
        "space/right",
        [("cell/source-from", "cell/right-from")],
        [("rel/source-a", "rel/right-a")],
        [],
    );
    let left_cells = vec![
        Cell::new(id("cell/left-from"), id("space/left"), 0, "vertex"),
        Cell::new(id("cell/left-to"), id("space/left"), 0, "vertex"),
    ];
    let right_cells = vec![
        Cell::new(id("cell/right-from"), id("space/right"), 0, "vertex"),
        Cell::new(id("cell/right-to"), id("space/right"), 0, "vertex"),
    ];
    let left_incidences = vec![Incidence::new(
        id("rel/left-a"),
        id("space/left"),
        id("cell/left-from"),
        id("cell/left-to"),
        "attaches",
        IncidenceOrientation::Directed,
    )];
    let right_incidences = vec![Incidence::new(
        id("rel/right-a"),
        id("space/right"),
        id("cell/right-from"),
        id("cell/right-to"),
        "attaches",
        IncidenceOrientation::Directed,
    )];

    let report = blocked_report(construct_explicit_pushout(PushoutInputs {
        left: &left,
        right: &right,
        candidate_space_id: id("space/pushout"),
        candidate_space_name: "Pushout".to_owned(),
        complex_type: ComplexType::CellComplex,
        left_cells: &left_cells,
        right_cells: &right_cells,
        left_incidences: &left_incidences,
        right_incidences: &right_incidences,
    }));

    assert_obstruction(&report, PushoutObstructionType::RelationEndpointConflict);
}

#[test]
fn explicit_pushout_constructs_with_ambiguous_identification_for_review() {
    let left = fixture_morphism(
        "left",
        "space/source",
        "space/left",
        [
            ("cell/source-a", "cell/left-a"),
            ("cell/source-b", "cell/left-a"),
        ],
        [],
        [],
    );
    let right = fixture_morphism(
        "right",
        "space/source",
        "space/right",
        [
            ("cell/source-a", "cell/right-a"),
            ("cell/source-b", "cell/right-b"),
        ],
        [],
        [],
    );
    let left_cells = vec![Cell::new(id("cell/left-a"), id("space/left"), 0, "vertex")];
    let right_cells = vec![
        Cell::new(id("cell/right-a"), id("space/right"), 0, "vertex"),
        Cell::new(id("cell/right-b"), id("space/right"), 0, "vertex"),
    ];

    let PushoutOutcome::Constructed {
        construction,
        report,
    } = construct_explicit_pushout(PushoutInputs {
        left: &left,
        right: &right,
        candidate_space_id: id("space/pushout"),
        candidate_space_name: "Pushout".to_owned(),
        complex_type: ComplexType::CellComplex,
        left_cells: &left_cells,
        right_cells: &right_cells,
        left_incidences: &[],
        right_incidences: &[],
    })
    else {
        panic!("ambiguous but well-defined pushout should construct");
    };

    assert_obstruction(&report, PushoutObstructionType::AmbiguousIdentification);
    assert_eq!(construction.review_status, ReviewStatus::Candidate);
    assert_eq!(construction.cells.len(), 1);
    assert!(!report.quotient_losses.is_empty());
}

#[test]
fn explicit_pushout_construction_is_deterministic() {
    let first = serde_json::to_string(&deterministic_outcome()).expect("serialize first outcome");
    let second = serde_json::to_string(&deterministic_outcome()).expect("serialize second outcome");

    assert_eq!(first, second);
}

#[test]
fn explicit_pushout_outcome_round_trips_json() {
    let outcome = deterministic_outcome();
    let encoded = serde_json::to_string(&outcome).expect("serialize outcome");
    let decoded: PushoutOutcome = serde_json::from_str(&encoded).expect("deserialize outcome");

    assert_eq!(decoded, outcome);
}

#[test]
fn explicit_pushout_canonical_ids_encode_reserved_separators_injectively() {
    let plus_id = singleton_left_cell_id("cell/a+b");
    let colon_id = singleton_left_cell_id("cell/a:b");
    let normal_id = singleton_left_cell_id("cell/normal");

    assert_eq!(plus_id, id("space/pushout/pushout/cell/left:cell/a%2Bb"));
    assert_eq!(colon_id, id("space/pushout/pushout/cell/left:cell/a%3Ab"));
    assert_ne!(plus_id, colon_id);
    assert_eq!(normal_id, id("space/pushout/pushout/cell/left:cell/normal"));
}

fn deterministic_outcome() -> PushoutOutcome {
    let left = fixture_morphism(
        "left",
        "space/source",
        "space/left",
        [("cell/source-a", "cell/left-a")],
        [("rel/source-a", "rel/left-a")],
        [],
    );
    let right = fixture_morphism(
        "right",
        "space/source",
        "space/right",
        [("cell/source-a", "cell/right-a")],
        [("rel/source-a", "rel/right-a")],
        [],
    );
    let left_cells = vec![
        Cell::new(id("cell/left-z"), id("space/left"), 0, "vertex"),
        Cell::new(id("cell/left-a"), id("space/left"), 0, "vertex"),
    ];
    let right_cells = vec![Cell::new(
        id("cell/right-a"),
        id("space/right"),
        0,
        "vertex",
    )];
    let left_incidences = vec![Incidence::new(
        id("rel/left-a"),
        id("space/left"),
        id("cell/left-a"),
        id("cell/left-a"),
        "attaches",
        IncidenceOrientation::Directed,
    )
    .with_weight(2.0)];
    let right_incidences = vec![Incidence::new(
        id("rel/right-a"),
        id("space/right"),
        id("cell/right-a"),
        id("cell/right-a"),
        "attaches",
        IncidenceOrientation::Directed,
    )
    .with_weight(2.0)];

    construct_explicit_pushout(PushoutInputs {
        left: &left,
        right: &right,
        candidate_space_id: id("space/pushout"),
        candidate_space_name: "Pushout".to_owned(),
        complex_type: ComplexType::CellComplex,
        left_cells: &left_cells,
        right_cells: &right_cells,
        left_incidences: &left_incidences,
        right_incidences: &right_incidences,
    })
}

fn singleton_left_cell_id(raw_cell_id: &str) -> Id {
    let left = fixture_morphism("left", "space/source", "space/left", [], [], []);
    let right = fixture_morphism("right", "space/source", "space/right", [], [], []);
    let left_cells = vec![Cell::new(id(raw_cell_id), id("space/left"), 0, "vertex")];

    let PushoutOutcome::Constructed { construction, .. } =
        construct_explicit_pushout(PushoutInputs {
            left: &left,
            right: &right,
            candidate_space_id: id("space/pushout"),
            candidate_space_name: "Pushout".to_owned(),
            complex_type: ComplexType::CellComplex,
            left_cells: &left_cells,
            right_cells: &[],
            left_incidences: &[],
            right_incidences: &[],
        })
    else {
        panic!("singleton pushout should construct");
    };

    construction.cells[0].id.clone()
}

fn empty_inputs<'a>(left: &'a Morphism, right: &'a Morphism) -> PushoutInputs<'a> {
    PushoutInputs {
        left,
        right,
        candidate_space_id: id("space/pushout"),
        candidate_space_name: "Pushout".to_owned(),
        complex_type: ComplexType::CellComplex,
        left_cells: &[],
        right_cells: &[],
        left_incidences: &[],
        right_incidences: &[],
    }
}

fn blocked_report(outcome: PushoutOutcome) -> ExplicitPushoutReport {
    let PushoutOutcome::Blocked { report } = outcome else {
        panic!("expected blocked pushout outcome");
    };
    report
}

fn assert_obstruction(report: &ExplicitPushoutReport, obstruction_type: PushoutObstructionType) {
    assert!(
        report
            .obstructions
            .iter()
            .any(|obstruction| obstruction.obstruction_type == obstruction_type),
        "expected obstruction {obstruction_type:?}, got {:?}",
        report.obstructions
    );
}
