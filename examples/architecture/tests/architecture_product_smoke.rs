//! Smoke scenario for the Architecture Product MVP example.

use higher_graphen_completion::{
    detect_completion_candidates, CompletionDetectionInput, CompletionRule, MissingType,
    SuggestedStructure,
};
use higher_graphen_core::{
    Confidence, Id, Provenance, Result, ReviewStatus, Severity, SourceKind, SourceRef,
};
use higher_graphen_invariant::{
    CheckResult, CheckTargetKind, Invariant, InvariantScope, Violation,
};
use higher_graphen_obstruction::{
    Counterexample, Obstruction, ObstructionExplanation, ObstructionType, RequiredResolution,
};
use higher_graphen_space::{
    Cell, Complex, ComplexType, InMemorySpaceStore, Incidence, IncidenceOrientation, Space,
};

const ARCHITECTURE_SPACE: &str = "space:architecture-product-smoke";
const ARCHITECTURE_CONTEXT: &str = "context:architecture-review";
const ORDER_CONTEXT: &str = "context:orders";
const BILLING_CONTEXT: &str = "context:billing";
const ORDER_SERVICE: &str = "cell:order-service";
const BILLING_SERVICE: &str = "cell:billing-service";
const BILLING_DB: &str = "cell:billing-db";
const ORDER_READS_BILLING_DB: &str = "incidence:order-service-reads-billing-db";
const BILLING_OWNS_BILLING_DB: &str = "incidence:billing-service-owns-billing-db";
const NO_CROSS_CONTEXT_DB_ACCESS: &str = "invariant:no-cross-context-direct-database-access";
const DIRECT_DB_ACCESS_OBSTRUCTION: &str = "obstruction:order-service-direct-billing-db-access";
const BILLING_STATUS_API_CANDIDATE: &str = "candidate:billing-status-api";
const BILLING_STATUS_API_CELL: &str = "cell:billing-status-api";

#[test]
fn order_service_to_billing_db_produces_obstruction_and_unreviewed_candidate() -> Result<()> {
    let scenario = build_architecture_smoke_scenario()?;

    assert_eq!(
        scenario.space.cell_ids,
        vec![id(ORDER_SERVICE), id(BILLING_SERVICE), id(BILLING_DB),]
    );
    assert_eq!(scenario.architecture_graph.max_dimension, 0);
    assert_eq!(scenario.invariant.id, id(NO_CROSS_CONTEXT_DB_ACCESS));
    assert!(scenario.check_result.is_violated());

    let violation = scenario
        .check_result
        .violation()
        .expect("violated result should carry violation details");
    assert_eq!(
        violation.location_cell_ids,
        vec![id(ORDER_SERVICE), id(BILLING_DB)]
    );
    assert!(violation
        .message
        .contains("Order Service directly accesses Billing DB"));

    assert_eq!(
        scenario.obstruction.obstruction_type,
        ObstructionType::InvariantViolation
    );
    assert_eq!(
        scenario.obstruction.location_cell_ids,
        vec![id(ORDER_SERVICE), id(BILLING_DB)]
    );
    assert!(scenario.obstruction.has_counterexample());
    assert!(scenario.obstruction.requires_resolution());

    let candidates = scenario.completion_result.into_candidates();
    assert_eq!(candidates.len(), 1);
    let candidate = &candidates[0];
    assert_eq!(candidate.id, id(BILLING_STATUS_API_CANDIDATE));
    assert_eq!(candidate.review_status, ReviewStatus::Unreviewed);
    assert!(!candidate.review_status.has_review_action());
    assert_eq!(
        candidate.suggested_structure.structure_id,
        Some(id(BILLING_STATUS_API_CELL))
    );
    assert!(candidate
        .suggested_structure
        .summary
        .contains("billing status query API"));
    assert_eq!(
        candidate.inferred_from,
        vec![id(DIRECT_DB_ACCESS_OBSTRUCTION), id(ORDER_READS_BILLING_DB),]
    );

    Ok(())
}

struct ArchitectureSmokeScenario {
    space: Space,
    architecture_graph: Complex,
    invariant: Invariant,
    check_result: CheckResult,
    obstruction: Obstruction,
    completion_result: higher_graphen_completion::CompletionDetectionResult,
}

fn build_architecture_smoke_scenario() -> Result<ArchitectureSmokeScenario> {
    let mut store = InMemorySpaceStore::new();
    store.insert_space(Space::new(
        id(ARCHITECTURE_SPACE),
        "Architecture Product Smoke",
    ))?;
    insert_architecture_cells(&mut store)?;
    insert_architecture_incidences(&mut store)?;

    let space = store
        .space(&id(ARCHITECTURE_SPACE))
        .expect("space inserted before scenario assertions")
        .clone();
    let architecture_graph = store.construct_complex(
        id("complex:architecture-product-smoke"),
        id(ARCHITECTURE_SPACE),
        "Architecture dependency and ownership graph",
        ComplexType::TypedGraph,
        [id(ORDER_SERVICE), id(BILLING_SERVICE), id(BILLING_DB)],
        [id(ORDER_READS_BILLING_DB), id(BILLING_OWNS_BILLING_DB)],
    )?;

    let invariant = no_cross_context_database_access_invariant();
    let check_result = direct_database_access_violation(&invariant);
    let obstruction = obstruction_for_violation(&check_result)?;
    let completion_result = propose_billing_status_api(&obstruction)?;

    Ok(ArchitectureSmokeScenario {
        space,
        architecture_graph,
        invariant,
        check_result,
        obstruction,
        completion_result,
    })
}

fn insert_architecture_cells(store: &mut InMemorySpaceStore) -> Result<()> {
    store.insert_cell(
        Cell::new(id(ORDER_SERVICE), id(ARCHITECTURE_SPACE), 0, "service")
            .with_label("Order Service")
            .with_context(id(ORDER_CONTEXT)),
    )?;
    store.insert_cell(
        Cell::new(id(BILLING_SERVICE), id(ARCHITECTURE_SPACE), 0, "service")
            .with_label("Billing Service")
            .with_context(id(BILLING_CONTEXT)),
    )?;
    store.insert_cell(
        Cell::new(id(BILLING_DB), id(ARCHITECTURE_SPACE), 0, "database")
            .with_label("Billing DB")
            .with_context(id(BILLING_CONTEXT)),
    )?;
    Ok(())
}

fn insert_architecture_incidences(store: &mut InMemorySpaceStore) -> Result<()> {
    store.insert_incidence(Incidence::new(
        id(ORDER_READS_BILLING_DB),
        id(ARCHITECTURE_SPACE),
        id(ORDER_SERVICE),
        id(BILLING_DB),
        "reads_database",
        IncidenceOrientation::Directed,
    ))?;
    store.insert_incidence(Incidence::new(
        id(BILLING_OWNS_BILLING_DB),
        id(ARCHITECTURE_SPACE),
        id(BILLING_SERVICE),
        id(BILLING_DB),
        "owns_database",
        IncidenceOrientation::Directed,
    ))?;
    Ok(())
}

fn no_cross_context_database_access_invariant() -> Invariant {
    let mut invariant = Invariant::new(
        id(NO_CROSS_CONTEXT_DB_ACCESS),
        "No cross-context direct database access",
        InvariantScope::Contexts {
            space_id: id(ARCHITECTURE_SPACE),
            context_ids: vec![id(ORDER_CONTEXT), id(BILLING_CONTEXT)],
        },
        Severity::Critical,
        provenance(),
    );
    invariant.description = Some(
        "A component must not directly access a database owned by another context.".to_owned(),
    );
    invariant
}

fn direct_database_access_violation(invariant: &Invariant) -> CheckResult {
    let violation = Violation::new(
        "Order Service directly accesses Billing DB, which is owned by Billing Service.",
        Severity::Critical,
    )
    .with_location_cells(vec![id(ORDER_SERVICE), id(BILLING_DB)])
    .with_location_contexts(vec![id(ORDER_CONTEXT), id(BILLING_CONTEXT)]);

    CheckResult::violated(CheckTargetKind::Invariant, invariant.id.clone(), violation)
}

fn obstruction_for_violation(check_result: &CheckResult) -> Result<Obstruction> {
    let violation = check_result
        .violation()
        .expect("obstruction is only built from violated check results");
    let explanation = ObstructionExplanation::new("Order Service directly accesses Billing DB")?
        .with_details(&violation.message)?;
    let counterexample = Counterexample::new("Direct cross-context database access is present")?
        .with_assignment("accessor", "Order Service")?
        .with_assignment("database", "Billing DB")?
        .with_assignment("owner", "Billing Service")?
        .with_path_cell(id(ORDER_SERVICE))
        .with_path_cell(id(BILLING_DB))
        .with_context(id(ORDER_CONTEXT))
        .with_context(id(BILLING_CONTEXT));
    let resolution = RequiredResolution::new(
        "Expose billing status through Billing Service and remove the direct Billing DB read.",
    )?
    .with_target_cell(id(ORDER_SERVICE))
    .with_target_cell(id(BILLING_SERVICE))
    .with_target_cell(id(BILLING_DB));

    Ok(Obstruction::new(
        id(DIRECT_DB_ACCESS_OBSTRUCTION),
        id(ARCHITECTURE_SPACE),
        ObstructionType::InvariantViolation,
        explanation,
        violation.severity,
        provenance(),
    )
    .with_location_cell(id(ORDER_SERVICE))
    .with_location_cell(id(BILLING_DB))
    .with_location_context(id(ORDER_CONTEXT))
    .with_location_context(id(BILLING_CONTEXT))
    .with_counterexample(counterexample)
    .with_required_resolution(resolution))
}

fn propose_billing_status_api(
    obstruction: &Obstruction,
) -> Result<higher_graphen_completion::CompletionDetectionResult> {
    let suggested_api = SuggestedStructure::new(
        "api",
        "Billing Service should expose a billing status query API.",
    )?
    .with_structure_id(id(BILLING_STATUS_API_CELL))
    .with_related_ids(vec![id(BILLING_SERVICE), id(ORDER_SERVICE), id(BILLING_DB)]);
    let rule = CompletionRule::new(
        id("rule:billing-status-api"),
        id(BILLING_STATUS_API_CANDIDATE),
        MissingType::Cell,
        suggested_api,
        "The obstruction shows Order Service needs billing status without direct database access.",
        Confidence::new(0.9)?,
    )?
    .with_context_ids(vec![id(ARCHITECTURE_CONTEXT)])
    .with_inferred_from(vec![obstruction.id.clone(), id(ORDER_READS_BILLING_DB)]);
    let input = CompletionDetectionInput::new(id(ARCHITECTURE_SPACE), vec![rule])
        .with_context_ids(vec![id(ARCHITECTURE_CONTEXT)]);

    detect_completion_candidates(input)
}

fn provenance() -> Provenance {
    Provenance::new(
        SourceRef::new(SourceKind::Document),
        Confidence::new(1.0).expect("static confidence is in range"),
    )
}

fn id(value: &str) -> Id {
    Id::new(value).expect("static scenario id should be valid")
}
