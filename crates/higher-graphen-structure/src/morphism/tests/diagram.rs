use super::*;

#[test]
fn diagram_commutativity_accepts_equivalent_explicit_paths() {
    let direct = fixture_morphism(
        "direct",
        "space/a",
        "space/c",
        [("cell/a1", "cell/c1")],
        [("rel/a1", "rel/c1")],
        ["invariant/shared"],
    );
    let first = fixture_morphism(
        "first",
        "space/a",
        "space/b",
        [("cell/a1", "cell/b1")],
        [("rel/a1", "rel/b1")],
        ["invariant/shared"],
    );
    let second = fixture_morphism(
        "second",
        "space/b",
        "space/c",
        [("cell/b1", "cell/c1")],
        [("rel/b1", "rel/c1")],
        ["invariant/shared"],
    );

    let report = check_diagram_commutativity(&[direct], &[first, second]);

    assert!(report.commutes);
    assert!(report.non_commutative_witnesses.is_empty());
    assert!(report.obstructions.is_empty());

    let roundtrip: DiagramCommutativityReport =
        serde_json::from_str(&serde_json::to_string(&report).expect("serialize"))
            .expect("deserialize");
    assert_eq!(roundtrip, report);
}

#[test]
fn diagram_commutativity_reports_mapping_disagreement() {
    let direct = fixture_morphism(
        "direct",
        "space/a",
        "space/c",
        [("cell/a1", "cell/c2")],
        [("rel/a1", "rel/c2")],
        ["invariant/shared"],
    );
    let first = fixture_morphism(
        "first",
        "space/a",
        "space/b",
        [("cell/a1", "cell/b1")],
        [("rel/a1", "rel/b1")],
        ["invariant/shared"],
    );
    let second = fixture_morphism(
        "second",
        "space/b",
        "space/c",
        [("cell/b1", "cell/c1")],
        [("rel/b1", "rel/c1")],
        ["invariant/shared"],
    );

    let report = check_diagram_commutativity(&[direct], &[first, second]);

    assert!(!report.commutes);
    assert!(report.obstructions.iter().any(|obstruction| {
        obstruction.obstruction_type == DiagramObstructionType::NonCommutativeDiagram
    }));
    assert_eq!(
        report.non_commutative_witnesses,
        vec![
            NonCommutativeWitness {
                element_kind: DiagramElementKind::Cell,
                source_element_id: id("cell/a1"),
                left_target_id: Some(id("cell/c2")),
                right_target_id: Some(id("cell/c1")),
            },
            NonCommutativeWitness {
                element_kind: DiagramElementKind::Relation,
                source_element_id: id("rel/a1"),
                left_target_id: Some(id("rel/c2")),
                right_target_id: Some(id("rel/c1")),
            },
        ]
    );
}

#[test]
fn diagram_commutativity_reports_incomplete_and_incompatible_paths() {
    let direct = fixture_morphism(
        "direct",
        "space/a",
        "space/c",
        [("cell/a1", "cell/c1")],
        [],
        ["invariant/shared"],
    );
    let first = fixture_morphism(
        "first",
        "space/a",
        "space/b",
        [("cell/a1", "cell/b1"), ("cell/a2", "cell/b2")],
        [],
        ["invariant/shared"],
    );
    let incompatible_second = fixture_morphism(
        "second",
        "space/x",
        "space/c",
        [("cell/b1", "cell/c1")],
        [],
        ["invariant/shared"],
    );

    let report = check_diagram_commutativity(&[direct], &[first, incompatible_second]);

    assert!(!report.commutes);
    assert!(report.obstructions.iter().any(|obstruction| {
        obstruction.obstruction_type == DiagramObstructionType::IncompatiblePath
    }));
    assert!(report.obstructions.iter().any(|obstruction| {
        obstruction.obstruction_type == DiagramObstructionType::IncompletePath
    }));
    assert_eq!(
        report.right_path.coverage.unmapped_cell_intermediate_ids,
        vec![id("cell/b2")]
    );
}

#[test]
fn diagram_check_batches_multiple_commutativity_requirements() {
    let direct = fixture_morphism(
        "direct",
        "space/a",
        "space/c",
        [("cell/a1", "cell/c1")],
        [],
        ["invariant/shared"],
    );
    let first = fixture_morphism(
        "first",
        "space/a",
        "space/b",
        [("cell/a1", "cell/b1")],
        [],
        ["invariant/shared"],
    );
    let second = fixture_morphism(
        "second",
        "space/b",
        "space/c",
        [("cell/b1", "cell/c1")],
        [],
        ["invariant/shared"],
    );
    let bad_direct = fixture_morphism(
        "bad-direct",
        "space/a",
        "space/c",
        [("cell/a1", "cell/c2")],
        [],
        ["invariant/shared"],
    );
    let requirements = vec![
        DiagramCommutativityRequirement::new(
            id("requirement/good"),
            vec![direct],
            vec![first.clone(), second.clone()],
        ),
        DiagramCommutativityRequirement::new(
            id("requirement/bad"),
            vec![bad_direct],
            vec![first, second],
        ),
    ];

    let report = check_diagram_requirements(id("diagram/batch"), &requirements);

    assert!(!report.commutes);
    assert_eq!(report.requirement_reports.len(), 2);
    assert!(report.requirement_reports[0].report.commutes);
    assert!(!report.requirement_reports[1].report.commutes);
}
