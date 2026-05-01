use super::*;
use higher_graphen_core::{Confidence, SourceKind, SourceRef};
use higher_graphen_projection::{
    InformationLoss, OutputSchema, Projection, ProjectionAudience, ProjectionPurpose,
    ProjectionSelector,
};
use higher_graphen_structure::morphism::{Morphism, MorphismType};
use higher_graphen_structure::space::{Cell, Incidence, IncidenceOrientation, Space};
use std::collections::BTreeMap;

fn id(value: impl AsRef<str>) -> Id {
    Id::new(value.as_ref()).expect("test id should be valid")
}

fn provenance() -> Provenance {
    let source = SourceRef::new(SourceKind::Code);
    let confidence = Confidence::new(1.0).expect("test confidence should be valid");
    Provenance::new(source, confidence)
}

fn store_with_cells<const N: usize>(cell_ids: [&str; N]) -> InMemorySpaceStore {
    let mut store = InMemorySpaceStore::new();
    store
        .insert_space(Space::new(id("space:architecture"), "Architecture"))
        .expect("space should insert");
    for cell_id in cell_ids {
        store
            .insert_cell(Cell::new(
                id(cell_id),
                id("space:architecture"),
                0,
                "component",
            ))
            .expect("cell should insert");
    }
    store
}

fn insert_dependency(store: &mut InMemorySpaceStore, incidence_id: &str, from: &str, to: &str) {
    store
        .insert_incidence(Incidence::new(
            id(incidence_id),
            id("space:architecture"),
            id(from),
            id(to),
            "depends_on",
            IncidenceOrientation::Directed,
        ))
        .expect("incidence should insert");
}

#[test]
fn constructs_invariant_and_constraint_definitions() {
    let invariant = Invariant::new(
        id("inv:acyclic"),
        "Acyclic ownership",
        InvariantScope::Space {
            space_id: id("space:architecture"),
        },
        Severity::High,
        provenance(),
    );
    let constraint = Constraint::new(
        id("constraint:no-cross-context-db"),
        "No cross-context database access",
        ConstraintScope::Contexts {
            space_id: id("space:architecture"),
            context_ids: vec![id("context:billing")],
        },
        Severity::Critical,
        provenance(),
    );

    assert_eq!(invariant.description, None);
    assert_eq!(
        constraint.scope,
        ConstraintScope::Contexts {
            space_id: id("space:architecture"),
            context_ids: vec![id("context:billing")],
        }
    );
}

#[test]
fn builds_changed_cell_scoped_input() {
    let input = CheckInput::changed_cells(
        id("space:architecture"),
        vec![id("cell:service-a"), id("cell:service-b")],
    )
    .with_invariants(vec![id("inv:ownership")])
    .with_constraints(vec![id("constraint:dependency-direction")])
    .with_contexts(vec![id("context:runtime")])
    .with_related_morphisms(vec![id("morphism:migration")]);

    assert!(input.is_changed_cell_scoped());
    assert_eq!(input.changed_cell_ids.len(), 2);
    assert_eq!(input.invariant_ids, vec![id("inv:ownership")]);
    assert_eq!(input.related_morphism_ids, vec![id("morphism:migration")]);
}

#[test]
fn creates_satisfied_violated_and_unsupported_results() {
    let satisfied = CheckResult::satisfied(CheckTargetKind::Invariant, id("inv:shape"));
    let violation = Violation::new("context boundary crossed", Severity::High)
        .with_location_cells(vec![id("cell:repository")])
        .with_location_contexts(vec![id("context:billing")]);
    let violated = CheckResult::violated(
        CheckTargetKind::Constraint,
        id("constraint:boundary"),
        violation,
    );
    let unsupported = CheckResult::unsupported(
        CheckTargetKind::Invariant,
        id("inv:morphism-preservation"),
        " morphism summary missing ",
    );

    assert!(satisfied.is_satisfied());
    assert!(violated.is_violated());
    assert_eq!(
        violated.violation().map(|item| &item.message),
        Some(&"context boundary crossed".to_owned())
    );
    assert!(unsupported.is_unsupported());
    assert_eq!(
        unsupported.unsupported_reason(),
        Some("morphism summary missing")
    );
    assert!(satisfied.validate().is_ok());
    assert!(violated.validate().is_ok());
    assert!(unsupported.validate().is_ok());
}

#[test]
fn validates_status_specific_result_payloads() {
    let satisfied_with_violation = CheckResult {
        violation: Some(Violation::new("should not be present", Severity::Low)),
        ..CheckResult::satisfied(CheckTargetKind::Invariant, id("inv:shape"))
    };
    let violated_without_violation = CheckResult {
        violation: None,
        ..CheckResult::violated(
            CheckTargetKind::Constraint,
            id("constraint:shape"),
            Violation::new("missing", Severity::Medium),
        )
    };
    let unsupported_without_reason = CheckResult {
        unsupported_reason: None,
        ..CheckResult::unsupported(CheckTargetKind::Invariant, id("inv:future"), "missing")
    };
    let unsupported_blank_reason =
        CheckResult::unsupported(CheckTargetKind::Invariant, id("inv:future"), " ");

    assert!(satisfied_with_violation.validate().is_err());
    assert!(violated_without_violation.validate().is_err());
    assert!(unsupported_without_reason.validate().is_err());
    assert!(unsupported_blank_reason.validate().is_err());
}

#[test]
fn deserialization_rejects_status_specific_payload_mismatch() {
    let satisfied_with_violation = serde_json::json!({
        "target_kind": "invariant",
        "target_id": "inv:shape",
        "status": "satisfied",
        "violation": {
            "message": "should not be present",
            "severity": "low"
        }
    });
    let unsupported_without_reason = serde_json::json!({
        "target_kind": "invariant",
        "target_id": "inv:future",
        "status": "unsupported"
    });

    assert!(serde_json::from_value::<CheckResult>(satisfied_with_violation).is_err());
    assert!(serde_json::from_value::<CheckResult>(unsupported_without_reason).is_err());
}

#[test]
fn evaluator_satisfies_acyclic_required_path_and_safety_rules() {
    let mut store = store_with_cells(["cell:a", "cell:b", "cell:c"]);
    insert_dependency(&mut store, "incidence:a-b", "cell:a", "cell:b");
    insert_dependency(&mut store, "incidence:b-c", "cell:b", "cell:c");
    let input = CheckInput::new(id("space:architecture"));
    let kernel = EvaluatorKernel::new()
        .with_rule(EvaluatorRule::invariant(
            id("inv:acyclic"),
            Severity::High,
            EvaluatorCheck::Acyclicity(AcyclicityCheck::new().with_relation_type("depends_on")),
        ))
        .with_rule(EvaluatorRule::constraint(
            id("constraint:path-a-c"),
            Severity::Medium,
            EvaluatorCheck::RequiredPath(RequiredPathCheck::new(id("cell:a"), id("cell:c"))),
        ))
        .with_rule(EvaluatorRule::constraint(
            id("constraint:c-not-a"),
            Severity::Critical,
            EvaluatorCheck::ReachabilitySafety(ReachabilitySafetyCheck::new(
                [id("cell:c")],
                [id("cell:a")],
            )),
        ));

    let report = kernel
        .evaluate(&EvaluatorContext::new(&input, &store))
        .expect("evaluation should succeed");

    assert_eq!(report.results.len(), 3);
    assert!(report.all_satisfied());
}

#[test]
fn acyclicity_reports_cycle_locations() {
    let mut store = store_with_cells(["cell:a", "cell:b"]);
    insert_dependency(&mut store, "incidence:a-b", "cell:a", "cell:b");
    insert_dependency(&mut store, "incidence:b-a", "cell:b", "cell:a");
    let input = CheckInput::new(id("space:architecture"));
    let rule = EvaluatorRule::invariant(
        id("inv:acyclic"),
        Severity::High,
        EvaluatorCheck::Acyclicity(AcyclicityCheck::new()),
    );

    let result = rule
        .evaluate(&EvaluatorContext::new(&input, &store))
        .expect("evaluation should succeed");

    assert!(result.is_violated());
    let location_cell_ids = &result.violation().expect("violation").location_cell_ids;
    assert!(location_cell_ids.contains(&id("cell:a")));
    assert!(location_cell_ids.contains(&id("cell:b")));
}

#[test]
fn required_path_reports_missing_path() {
    let store = store_with_cells(["cell:a", "cell:c"]);
    let input = CheckInput::new(id("space:architecture"));
    let rule = EvaluatorRule::constraint(
        id("constraint:path-a-c"),
        Severity::Medium,
        EvaluatorCheck::RequiredPath(RequiredPathCheck::new(id("cell:a"), id("cell:c"))),
    );

    let result = rule
        .evaluate(&EvaluatorContext::new(&input, &store))
        .expect("evaluation should succeed");

    assert!(result.is_violated());
    assert_eq!(
        result.violation().expect("violation").location_cell_ids,
        vec![id("cell:a"), id("cell:c")]
    );
}

#[test]
fn reachability_safety_reports_forbidden_path_witness() {
    let mut store = store_with_cells(["cell:a", "cell:b", "cell:c"]);
    insert_dependency(&mut store, "incidence:a-b", "cell:a", "cell:b");
    insert_dependency(&mut store, "incidence:b-c", "cell:b", "cell:c");
    let input = CheckInput::new(id("space:architecture"));
    let rule = EvaluatorRule::constraint(
        id("constraint:no-a-c"),
        Severity::Critical,
        EvaluatorCheck::ReachabilitySafety(ReachabilitySafetyCheck::new(
            [id("cell:a")],
            [id("cell:c")],
        )),
    );

    let result = rule
        .evaluate(&EvaluatorContext::new(&input, &store))
        .expect("evaluation should succeed");

    assert!(result.is_violated());
    assert_eq!(
        result.violation().expect("violation").location_cell_ids,
        vec![id("cell:a"), id("cell:b"), id("cell:c")]
    );
}

#[test]
fn context_compatibility_checks_declared_membership_or_reports_unsupported() {
    let mut store = InMemorySpaceStore::new();
    store
        .insert_space(Space::new(id("space:architecture"), "Architecture"))
        .expect("space should insert");
    store
        .insert_cell(
            Cell::new(
                id("cell:repository"),
                id("space:architecture"),
                0,
                "component",
            )
            .with_context(id("context:billing")),
        )
        .expect("cell should insert");
    let input = CheckInput::changed_cells(id("space:architecture"), vec![id("cell:repository")]);
    let rule = EvaluatorRule::constraint(
        id("constraint:context"),
        Severity::High,
        EvaluatorCheck::ContextCompatibility(ContextCompatibilityCheck::new(
            Vec::<Id>::new(),
            [id("context:billing"), id("context:orders")],
        )),
    );

    let result = rule
        .evaluate(&EvaluatorContext::new(&input, &store))
        .expect("evaluation should succeed");

    assert!(result.is_violated());
    assert_eq!(
        result.violation().expect("violation").location_context_ids,
        vec![id("context:orders")]
    );

    let unsupported = EvaluatorRule::constraint(
        id("constraint:context-unsupported"),
        Severity::High,
        EvaluatorCheck::ContextCompatibility(ContextCompatibilityCheck::default()),
    )
    .evaluate(&EvaluatorContext::new(
        &CheckInput::new(id("space:architecture")),
        &store,
    ))
    .expect("evaluation should succeed");

    assert!(unsupported.is_unsupported());
}

#[test]
fn morphism_preservation_uses_supplied_report() {
    let store = store_with_cells(["cell:a"]);
    let input = CheckInput::new(id("space:architecture"));
    let morphism = Morphism {
        id: id("morphism:projection"),
        source_space_id: id("space:architecture"),
        target_space_id: id("space:summary"),
        name: "Projection".to_owned(),
        morphism_type: MorphismType::Projection,
        cell_mapping: BTreeMap::new(),
        relation_mapping: BTreeMap::new(),
        preserved_invariant_ids: vec![id("inv:preserved")],
        lost_structure: Vec::new(),
        distortion: Vec::new(),
        composable_with: Vec::new(),
        provenance: provenance(),
    };
    let rule = EvaluatorRule::invariant(
        id("inv:required"),
        Severity::High,
        EvaluatorCheck::MorphismPreservation(
            MorphismPreservationCheck::new(id("morphism:projection"))
                .with_invariants([id("inv:preserved"), id("inv:required")]),
        ),
    );

    let result = rule
        .evaluate(
            &EvaluatorContext::new(&input, &store).with_morphisms(std::slice::from_ref(&morphism)),
        )
        .expect("evaluation should succeed");

    assert!(result.is_violated());
    assert_eq!(
        result.violation().expect("violation").related_morphism_ids,
        vec![id("morphism:projection")]
    );

    let missing = EvaluatorRule::invariant(
        id("inv:required"),
        Severity::High,
        EvaluatorCheck::MorphismPreservation(MorphismPreservationCheck::new(id(
            "morphism:missing",
        ))),
    )
    .evaluate(&EvaluatorContext::new(&input, &store))
    .expect("evaluation should succeed");

    assert!(missing.is_unsupported());
}

#[test]
fn projection_loss_declaration_checks_declared_sources() {
    let store = store_with_cells(["cell:a"]);
    let input = CheckInput::new(id("space:architecture"));
    let projection = Projection::new(
        id("projection:review"),
        id("space:architecture"),
        "Review",
        ProjectionAudience::Architect,
        ProjectionPurpose::Report,
        ProjectionSelector::all(),
        OutputSchema::text(),
        [
            InformationLoss::declared("summarizes detail", [id("cell:a")])
                .expect("loss should be valid"),
        ],
    )
    .expect("projection should be valid");
    let satisfied = EvaluatorRule::constraint(
        id("constraint:loss"),
        Severity::Medium,
        EvaluatorCheck::ProjectionLossDeclared(
            ProjectionLossDeclarationCheck::new(id("projection:review"))
                .with_required_sources([id("cell:a")]),
        ),
    )
    .evaluate(
        &EvaluatorContext::new(&input, &store).with_projections(std::slice::from_ref(&projection)),
    )
    .expect("evaluation should succeed");

    assert!(satisfied.is_satisfied());

    let violated = EvaluatorRule::constraint(
        id("constraint:loss"),
        Severity::Medium,
        EvaluatorCheck::ProjectionLossDeclared(
            ProjectionLossDeclarationCheck::new(id("projection:review"))
                .with_required_sources([id("cell:b")]),
        ),
    )
    .evaluate(
        &EvaluatorContext::new(&input, &store).with_projections(std::slice::from_ref(&projection)),
    )
    .expect("evaluation should succeed");

    assert!(violated.is_violated());
}

#[test]
fn check_result_converts_violation_to_obstruction() {
    let result = CheckResult::violated(
        CheckTargetKind::Constraint,
        id("constraint:boundary"),
        Violation::new("boundary crossed", Severity::High)
            .with_location_cells(vec![id("cell:repository")])
            .with_location_contexts(vec![id("context:billing")])
            .with_related_morphisms(vec![id("morphism:projection")]),
    );

    let obstruction = result
        .to_obstruction(
            id("obstruction:boundary"),
            id("space:architecture"),
            provenance(),
        )
        .expect("handoff should succeed")
        .expect("violation should produce obstruction");

    assert_eq!(
        obstruction.obstruction_type,
        ObstructionType::ConstraintUnsatisfied
    );
    assert_eq!(obstruction.location_cell_ids, vec![id("cell:repository")]);
    assert_eq!(
        obstruction.location_context_ids,
        vec![id("context:billing")]
    );
    assert_eq!(
        obstruction.related_morphisms[0].morphism_id,
        id("morphism:projection")
    );
}

#[test]
fn kernel_respects_normalized_input_selection() {
    let store = store_with_cells(["cell:a"]);
    let input = CheckInput::new(id("space:architecture"))
        .with_invariants(vec![id("inv:selected"), id("inv:selected")]);
    let kernel = EvaluatorKernel::new()
        .with_rule(EvaluatorRule::invariant(
            id("inv:selected"),
            Severity::Low,
            EvaluatorCheck::Acyclicity(AcyclicityCheck::new()),
        ))
        .with_rule(EvaluatorRule::invariant(
            id("inv:skipped"),
            Severity::Low,
            EvaluatorCheck::Acyclicity(AcyclicityCheck::new()),
        ))
        .with_rule(EvaluatorRule::constraint(
            id("constraint:skipped"),
            Severity::Low,
            EvaluatorCheck::RequiredPath(RequiredPathCheck::new(id("cell:a"), id("cell:a"))),
        ));

    let report = kernel
        .evaluate(&EvaluatorContext::new(&input, &store))
        .expect("evaluation should succeed");

    assert_eq!(input.normalized().invariant_ids, vec![id("inv:selected")]);
    assert_eq!(report.results.len(), 1);
    assert_eq!(report.results[0].target_id(), &id("inv:selected"));
}
