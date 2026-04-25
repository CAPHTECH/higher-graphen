//! Deterministic Architecture Product direct database access smoke workflow.

use crate::error::{RuntimeError, RuntimeResult};
use crate::reports::{
    ArchitectureDirectDbAccessSmokeProjection, ArchitectureDirectDbAccessSmokeReport,
    ArchitectureDirectDbAccessSmokeResult, ArchitectureDirectDbAccessSmokeScenario,
    ArchitectureSmokeStatus, ProjectionAudience, ProjectionPurpose, ReportEnvelope, ReportMetadata,
};
use higher_graphen_completion::{
    detect_completion_candidates, CompletionCandidate, CompletionDetectionInput, CompletionRule,
    MissingType, SuggestedStructure,
};
use higher_graphen_core::{
    Confidence, Id, Provenance, ReviewStatus, Severity, SourceKind, SourceRef,
};
use higher_graphen_invariant::{
    CheckResult, CheckTargetKind, Invariant, InvariantScope, Violation,
};
use higher_graphen_obstruction::{
    Counterexample, Obstruction, ObstructionExplanation, ObstructionType, RequiredResolution,
};
use higher_graphen_projection::InformationLoss;
use higher_graphen_space::{
    Cell, ComplexType, InMemorySpaceStore, Incidence, IncidenceOrientation, Space,
};

const WORKFLOW_NAME: &str = "architecture_direct_db_access_smoke";
const REPORT_SCHEMA: &str = "highergraphen.architecture.direct_db_access_smoke.report.v1";
const REPORT_TYPE: &str = "architecture_direct_db_access_smoke";
const REPORT_VERSION: u32 = 1;

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

/// Runs the deterministic Architecture Product direct database access smoke workflow.
pub fn run_architecture_direct_db_access_smoke(
) -> RuntimeResult<ArchitectureDirectDbAccessSmokeReport> {
    let scenario = build_architecture_smoke_scenario()?;
    let check_result = direct_database_access_violation(&scenario.invariant)?;
    let obstruction = obstruction_for_violation(&check_result)?;
    let completion_candidates = propose_billing_status_api(&obstruction)?;
    ensure_unreviewed_candidate(&completion_candidates)?;

    let report_scenario = report_scenario(scenario);
    let result = report_result(check_result, obstruction, completion_candidates)?;
    let projection = report_projection(&result)?;

    Ok(ReportEnvelope {
        schema: REPORT_SCHEMA.to_owned(),
        report_type: REPORT_TYPE.to_owned(),
        report_version: REPORT_VERSION,
        metadata: ReportMetadata::architecture_direct_db_access_smoke(),
        scenario: report_scenario,
        result,
        projection,
    })
}

struct ArchitectureSmokeScenario {
    space: Space,
    cells: Vec<Cell>,
    incidences: Vec<Incidence>,
    invariant: Invariant,
}

fn build_architecture_smoke_scenario() -> RuntimeResult<ArchitectureSmokeScenario> {
    let mut store = InMemorySpaceStore::new();
    store.insert_space(Space::new(
        id(ARCHITECTURE_SPACE)?,
        "Architecture Product Smoke",
    ))?;

    let cells = insert_architecture_cells(&mut store)?;
    let incidences = insert_architecture_incidences(&mut store)?;
    let architecture_graph = store.construct_complex(
        id("complex:architecture-product-smoke")?,
        id(ARCHITECTURE_SPACE)?,
        "Architecture dependency and ownership graph",
        ComplexType::TypedGraph,
        cells.iter().map(|cell| cell.id.clone()),
        incidences.iter().map(|incidence| incidence.id.clone()),
    )?;
    debug_assert_eq!(architecture_graph.max_dimension, 0);

    let space = store
        .space(&id(ARCHITECTURE_SPACE)?)
        .ok_or_else(|| missing_inserted("space"))?
        .clone();

    Ok(ArchitectureSmokeScenario {
        space,
        cells,
        incidences,
        invariant: no_cross_context_database_access_invariant()?,
    })
}

fn insert_architecture_cells(store: &mut InMemorySpaceStore) -> RuntimeResult<Vec<Cell>> {
    let cells = vec![
        Cell::new(id(ORDER_SERVICE)?, id(ARCHITECTURE_SPACE)?, 0, "service")
            .with_label("Order Service")
            .with_context(id(ORDER_CONTEXT)?),
        Cell::new(id(BILLING_SERVICE)?, id(ARCHITECTURE_SPACE)?, 0, "service")
            .with_label("Billing Service")
            .with_context(id(BILLING_CONTEXT)?),
        Cell::new(id(BILLING_DB)?, id(ARCHITECTURE_SPACE)?, 0, "database")
            .with_label("Billing DB")
            .with_context(id(BILLING_CONTEXT)?),
    ];

    for cell in &cells {
        store.insert_cell(cell.clone())?;
    }

    Ok(cells)
}

fn insert_architecture_incidences(store: &mut InMemorySpaceStore) -> RuntimeResult<Vec<Incidence>> {
    let incidences = vec![
        Incidence::new(
            id(ORDER_READS_BILLING_DB)?,
            id(ARCHITECTURE_SPACE)?,
            id(ORDER_SERVICE)?,
            id(BILLING_DB)?,
            "reads_database",
            IncidenceOrientation::Directed,
        ),
        Incidence::new(
            id(BILLING_OWNS_BILLING_DB)?,
            id(ARCHITECTURE_SPACE)?,
            id(BILLING_SERVICE)?,
            id(BILLING_DB)?,
            "owns_database",
            IncidenceOrientation::Directed,
        ),
    ];

    for incidence in &incidences {
        store.insert_incidence(incidence.clone())?;
    }

    Ok(incidences)
}

fn no_cross_context_database_access_invariant() -> RuntimeResult<Invariant> {
    let mut invariant = Invariant::new(
        id(NO_CROSS_CONTEXT_DB_ACCESS)?,
        "No cross-context direct database access",
        InvariantScope::Contexts {
            space_id: id(ARCHITECTURE_SPACE)?,
            context_ids: vec![id(ORDER_CONTEXT)?, id(BILLING_CONTEXT)?],
        },
        Severity::Critical,
        provenance()?,
    );
    invariant.description = Some(
        "A component must not directly access a database owned by another context.".to_owned(),
    );
    Ok(invariant)
}

fn direct_database_access_violation(invariant: &Invariant) -> RuntimeResult<CheckResult> {
    let violation = Violation::new(
        "Order Service directly accesses Billing DB, which is owned by Billing Service.",
        Severity::Critical,
    )
    .with_location_cells(vec![id(ORDER_SERVICE)?, id(BILLING_DB)?])
    .with_location_contexts(vec![id(ORDER_CONTEXT)?, id(BILLING_CONTEXT)?]);

    Ok(CheckResult::violated(
        CheckTargetKind::Invariant,
        invariant.id.clone(),
        violation,
    ))
}

fn obstruction_for_violation(check_result: &CheckResult) -> RuntimeResult<Obstruction> {
    let violation = check_result
        .violation
        .as_ref()
        .ok_or_else(|| missing_violation("obstruction"))?;
    let explanation = ObstructionExplanation::new("Order Service directly accesses Billing DB")?
        .with_details(&violation.message)?;
    let counterexample = Counterexample::new("Direct cross-context database access is present")?
        .with_assignment("accessor", "Order Service")?
        .with_assignment("database", "Billing DB")?
        .with_assignment("owner", "Billing Service")?
        .with_path_cell(id(ORDER_SERVICE)?)
        .with_path_cell(id(BILLING_DB)?)
        .with_context(id(ORDER_CONTEXT)?)
        .with_context(id(BILLING_CONTEXT)?);
    let resolution = RequiredResolution::new(
        "Expose billing status through Billing Service and remove the direct Billing DB read.",
    )?
    .with_target_cell(id(ORDER_SERVICE)?)
    .with_target_cell(id(BILLING_SERVICE)?)
    .with_target_cell(id(BILLING_DB)?);

    Ok(Obstruction::new(
        id(DIRECT_DB_ACCESS_OBSTRUCTION)?,
        id(ARCHITECTURE_SPACE)?,
        ObstructionType::InvariantViolation,
        explanation,
        violation.severity,
        provenance()?,
    )
    .with_location_cell(id(ORDER_SERVICE)?)
    .with_location_cell(id(BILLING_DB)?)
    .with_location_context(id(ORDER_CONTEXT)?)
    .with_location_context(id(BILLING_CONTEXT)?)
    .with_counterexample(counterexample)
    .with_required_resolution(resolution))
}

fn propose_billing_status_api(
    obstruction: &Obstruction,
) -> RuntimeResult<Vec<CompletionCandidate>> {
    let suggested_api = SuggestedStructure::new(
        "api",
        "Billing Service should expose a billing status query API.",
    )?
    .with_structure_id(id(BILLING_STATUS_API_CELL)?)
    .with_related_ids(vec![
        id(BILLING_SERVICE)?,
        id(ORDER_SERVICE)?,
        id(BILLING_DB)?,
    ]);
    let rule = CompletionRule::new(
        id("rule:billing-status-api")?,
        id(BILLING_STATUS_API_CANDIDATE)?,
        MissingType::Cell,
        suggested_api,
        "The obstruction shows Order Service needs billing status without direct database access.",
        Confidence::new(0.9)?,
    )?
    .with_context_ids(vec![id(ARCHITECTURE_CONTEXT)?])
    .with_inferred_from(vec![obstruction.id.clone(), id(ORDER_READS_BILLING_DB)?]);
    let input = CompletionDetectionInput::new(id(ARCHITECTURE_SPACE)?, vec![rule])
        .with_context_ids(vec![id(ARCHITECTURE_CONTEXT)?]);

    Ok(detect_completion_candidates(input)?.candidates)
}

fn ensure_unreviewed_candidate(candidates: &[CompletionCandidate]) -> RuntimeResult<()> {
    match candidates {
        [candidate] if candidate.review_status == ReviewStatus::Unreviewed => Ok(()),
        [candidate] => Err(RuntimeError::workflow_construction(
            WORKFLOW_NAME,
            format!(
                "completion candidate {} has review status {:?}",
                candidate.id, candidate.review_status
            ),
        )),
        _ => Err(RuntimeError::workflow_construction(
            WORKFLOW_NAME,
            format!(
                "expected exactly one completion candidate, got {}",
                candidates.len()
            ),
        )),
    }
}

fn report_scenario(scenario: ArchitectureSmokeScenario) -> ArchitectureDirectDbAccessSmokeScenario {
    ArchitectureDirectDbAccessSmokeScenario {
        space_id: scenario.space.id,
        workflow_context_id: id_static(ARCHITECTURE_CONTEXT),
        context_ids: vec![id_static(ORDER_CONTEXT), id_static(BILLING_CONTEXT)],
        cells: scenario.cells,
        incidences: scenario.incidences,
        invariant_id: scenario.invariant.id,
        invariant_name: scenario.invariant.name,
    }
}

fn report_result(
    check_result: CheckResult,
    obstruction: Obstruction,
    completion_candidates: Vec<CompletionCandidate>,
) -> RuntimeResult<ArchitectureDirectDbAccessSmokeResult> {
    if !check_result.is_violated() {
        return Err(RuntimeError::workflow_construction(
            WORKFLOW_NAME,
            "deterministic smoke scenario must produce a violated check result",
        ));
    }

    Ok(ArchitectureDirectDbAccessSmokeResult {
        status: ArchitectureSmokeStatus::ViolationDetected,
        violated_invariant_id: id(NO_CROSS_CONTEXT_DB_ACCESS)?,
        check_result,
        obstructions: vec![obstruction],
        completion_candidates,
    })
}

fn report_projection(
    result: &ArchitectureDirectDbAccessSmokeResult,
) -> RuntimeResult<ArchitectureDirectDbAccessSmokeProjection> {
    let source_ids = projection_source_ids()?;
    let loss = InformationLoss::declared(
        "Projection summarizes the full space, invariant check, obstruction, and completion candidate.",
        source_ids.clone(),
    )?;

    Ok(ArchitectureDirectDbAccessSmokeProjection {
        audience: ProjectionAudience::Human,
        purpose: ProjectionPurpose::ArchitectureReview,
        summary:
            "Order Service directly reads Billing DB across Orders and Billing context boundaries."
                .to_owned(),
        recommended_actions: vec![
            "Route billing status access through Billing Service or a Billing Service API."
                .to_owned(),
            "Remove the direct Order Service read from Billing DB.".to_owned(),
        ],
        information_loss: vec![loss],
        source_ids: result_source_ids(result, source_ids),
    })
}

fn projection_source_ids() -> RuntimeResult<Vec<Id>> {
    Ok(vec![
        id(ARCHITECTURE_SPACE)?,
        id(ORDER_SERVICE)?,
        id(BILLING_SERVICE)?,
        id(BILLING_DB)?,
        id(ORDER_READS_BILLING_DB)?,
        id(BILLING_OWNS_BILLING_DB)?,
        id(NO_CROSS_CONTEXT_DB_ACCESS)?,
        id(DIRECT_DB_ACCESS_OBSTRUCTION)?,
        id(BILLING_STATUS_API_CANDIDATE)?,
    ])
}

fn result_source_ids(
    result: &ArchitectureDirectDbAccessSmokeResult,
    mut source_ids: Vec<Id>,
) -> Vec<Id> {
    source_ids.push(result.violated_invariant_id.clone());
    source_ids
}

fn provenance() -> RuntimeResult<Provenance> {
    Ok(Provenance::new(
        SourceRef::new(SourceKind::Document),
        Confidence::new(1.0)?,
    ))
}

fn id(value: &str) -> RuntimeResult<Id> {
    Ok(Id::new(value)?)
}

fn id_static(value: &str) -> Id {
    Id::new(value).expect("static runtime workflow id should be valid")
}

fn missing_inserted(target: &str) -> RuntimeError {
    RuntimeError::workflow_construction(
        WORKFLOW_NAME,
        format!("{target} was not available after insertion"),
    )
}

fn missing_violation(target: &str) -> RuntimeError {
    RuntimeError::workflow_construction(
        WORKFLOW_NAME,
        format!("{target} requires a violated check result"),
    )
}
