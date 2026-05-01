//! Bounded Architecture Product input lift workflow.

use crate::error::{RuntimeError, RuntimeResult};
use crate::reports::{
    AiProjectionRecord, AiProjectionRecordType, AiProjectionView, ArchitectureInputComponent,
    ArchitectureInputInference, ArchitectureInputLiftDocument, ArchitectureInputLiftProjection,
    ArchitectureInputLiftReport, ArchitectureInputLiftResult, ArchitectureInputLiftScenario,
    ArchitectureInputLiftStatus, ArchitectureInputRelation, AuditProjectionView,
    HumanReviewProjectionView, ProjectionAudience, ProjectionPurpose, ProjectionTrace,
    ProjectionViewSet, ReportEnvelope, ReportMetadata,
};
use higher_graphen_core::{Confidence, Id, Provenance, ReviewStatus, SourceRef};
use higher_graphen_interpretation::architecture::{
    architecture_input_lift_adapter, architecture_input_lift_component_type_mapping,
    architecture_input_lift_relation_morphism_type_mapping, architecture_interpretation_package,
    architecture_review_projection_template,
};
use higher_graphen_interpretation::{InterpretationPackage, ProjectionTemplate};
use higher_graphen_projection::InformationLoss;
use higher_graphen_reasoning::completion::{
    detect_completion_candidates, CompletionCandidate, CompletionDetectionInput, CompletionRule,
    SuggestedStructure,
};
use higher_graphen_structure::space::{Cell, ComplexType, InMemorySpaceStore, Incidence, Space};

const WORKFLOW_NAME: &str = "architecture_input_lift";
const INPUT_SCHEMA: &str = "highergraphen.architecture.input.v1";
const REPORT_SCHEMA: &str = "highergraphen.architecture.input_lift.report.v1";
const REPORT_TYPE: &str = "architecture_input_lift";
const REPORT_VERSION: u32 = 1;
const EXTRACTION_METHOD: &str = "architecture_input_lift.v1";

/// Runs the bounded Architecture Product input lift workflow.
pub fn run_architecture_input_lift(
    input: ArchitectureInputLiftDocument,
) -> RuntimeResult<ArchitectureInputLiftReport> {
    validate_input_schema(&input)?;
    validate_input_references(&input)?;
    let interpretation = architecture_interpretation_package()?;

    let lifted = lift_architecture_input(&input, &interpretation)?;
    ensure_inferences_are_not_accepted(&lifted.cells, &lifted.completion_candidates)?;

    let scenario = report_scenario(&input, &lifted);
    let result = report_result(&lifted);
    let projection = report_projection(&result, &lifted, &interpretation)?;

    Ok(ReportEnvelope {
        schema: REPORT_SCHEMA.to_owned(),
        report_type: REPORT_TYPE.to_owned(),
        report_version: REPORT_VERSION,
        metadata: ReportMetadata::architecture_input_lift(),
        scenario,
        result,
        projection,
    })
}

struct LiftedArchitecture {
    space: Space,
    cells: Vec<Cell>,
    incidences: Vec<Incidence>,
    completion_candidates: Vec<CompletionCandidate>,
}

fn lift_architecture_input(
    input: &ArchitectureInputLiftDocument,
    interpretation: &InterpretationPackage,
) -> RuntimeResult<LiftedArchitecture> {
    validate_input_against_interpretation_package(input, interpretation)?;

    let mut store = InMemorySpaceStore::new();
    let space = input_space(input);
    store.insert_space(space)?;

    let cells = insert_cells(&mut store, input)?;
    let incidences = insert_incidences(&mut store, input)?;
    construct_complex(&mut store, input, &cells, &incidences)?;
    let completion_candidates = completion_candidates(input)?;

    let space = store
        .space(&input.space.id)
        .ok_or_else(|| missing_inserted("space"))?
        .clone();

    Ok(LiftedArchitecture {
        space,
        cells,
        incidences,
        completion_candidates,
    })
}

fn validate_input_against_interpretation_package(
    input: &ArchitectureInputLiftDocument,
    interpretation: &InterpretationPackage,
) -> RuntimeResult<()> {
    architecture_input_lift_adapter(interpretation)?;
    for component in &input.components {
        architecture_input_lift_component_type_mapping(interpretation, &component.component_type)?;
    }
    for relation in &input.relations {
        architecture_input_lift_relation_morphism_type_mapping(
            interpretation,
            &relation.relation_type,
        )?;
    }
    Ok(())
}

fn input_space(input: &ArchitectureInputLiftDocument) -> Space {
    let space = Space::new(input.space.id.clone(), input.space.name.clone());
    match &input.space.description {
        Some(description) => space.with_description(description.clone()),
        None => space,
    }
}

fn insert_cells(
    store: &mut InMemorySpaceStore,
    input: &ArchitectureInputLiftDocument,
) -> RuntimeResult<Vec<Cell>> {
    let mut cells = Vec::with_capacity(input.components.len());
    for component in &input.components {
        let cell = component_cell(input, component)?;
        cells.push(store.insert_cell(cell)?);
    }
    Ok(cells)
}

fn component_cell(
    input: &ArchitectureInputLiftDocument,
    component: &ArchitectureInputComponent,
) -> RuntimeResult<Cell> {
    let confidence = component.confidence.unwrap_or(input.source.confidence);
    let provenance = fact_provenance(input, confidence, component.source_local_id.as_deref());
    Ok(Cell::new(
        component.id.clone(),
        input.space.id.clone(),
        0,
        component.component_type.clone(),
    )
    .with_label(component.label.clone())
    .with_context(component.context_id.clone())
    .with_provenance(provenance))
}

fn insert_incidences(
    store: &mut InMemorySpaceStore,
    input: &ArchitectureInputLiftDocument,
) -> RuntimeResult<Vec<Incidence>> {
    let mut incidences = Vec::with_capacity(input.relations.len());
    for relation in &input.relations {
        let incidence = relation_incidence(input, relation)?;
        incidences.push(store.insert_incidence(incidence)?);
    }
    Ok(incidences)
}

fn relation_incidence(
    input: &ArchitectureInputLiftDocument,
    relation: &ArchitectureInputRelation,
) -> RuntimeResult<Incidence> {
    let confidence = relation.confidence.unwrap_or(input.source.confidence);
    let provenance = fact_provenance(input, confidence, relation.source_local_id.as_deref());
    let mut incidence = Incidence::new(
        relation.id.clone(),
        input.space.id.clone(),
        relation.from_cell_id.clone(),
        relation.to_cell_id.clone(),
        relation.relation_type.clone(),
        relation.orientation,
    )
    .with_provenance(provenance);
    if let Some(weight) = relation.weight {
        incidence = incidence.with_weight(weight);
    }
    Ok(incidence)
}

fn construct_complex(
    store: &mut InMemorySpaceStore,
    input: &ArchitectureInputLiftDocument,
    cells: &[Cell],
    incidences: &[Incidence],
) -> RuntimeResult<()> {
    store.construct_complex(
        id(format!(
            "complex:architecture-input-lift:{}",
            input.space.id
        ))?,
        input.space.id.clone(),
        format!("{} lifted architecture graph", input.space.name),
        ComplexType::TypedGraph,
        cells.iter().map(|cell| cell.id.clone()),
        incidences.iter().map(|incidence| incidence.id.clone()),
    )?;
    Ok(())
}

fn completion_candidates(
    input: &ArchitectureInputLiftDocument,
) -> RuntimeResult<Vec<CompletionCandidate>> {
    let rules = input
        .inferred_structures
        .iter()
        .map(|inference| completion_rule(input, inference))
        .collect::<RuntimeResult<Vec<_>>>()?;
    let detection_input = CompletionDetectionInput::new(input.space.id.clone(), rules)
        .with_context_ids(
            input
                .contexts
                .iter()
                .map(|context| context.id.clone())
                .collect(),
        );
    Ok(detect_completion_candidates(detection_input)?.into_candidates())
}

fn completion_rule(
    input: &ArchitectureInputLiftDocument,
    inference: &ArchitectureInputInference,
) -> RuntimeResult<CompletionRule> {
    let mut suggested =
        SuggestedStructure::new(inference.structure_type.clone(), inference.summary.clone())?
            .with_related_ids(inference.related_ids.clone());
    if let Some(structure_id) = &inference.structure_id {
        suggested = suggested.with_structure_id(structure_id.clone());
    }
    let rule = CompletionRule::new(
        id(format!("rule:architecture-input-lift:{}", inference.id))?,
        inference.id.clone(),
        inference.missing_type,
        suggested,
        inference.rationale.clone(),
        inference.confidence,
    )?
    .with_context_ids(inference.context_ids.clone())
    .with_inferred_from(inference.inferred_from.clone());
    ensure_inferred_from_accepted(input, inference)?;
    Ok(rule)
}

fn fact_provenance(
    input: &ArchitectureInputLiftDocument,
    confidence: Confidence,
    source_local_id: Option<&str>,
) -> Provenance {
    let mut provenance = Provenance::new(source_ref(input, source_local_id), confidence)
        .with_review_status(ReviewStatus::Accepted);
    provenance.extraction_method = Some(EXTRACTION_METHOD.to_owned());
    provenance
}

fn source_ref(input: &ArchitectureInputLiftDocument, source_local_id: Option<&str>) -> SourceRef {
    SourceRef {
        kind: input.source.kind.clone(),
        uri: input.source.uri.clone(),
        title: input.source.title.clone(),
        captured_at: input.source.captured_at.clone(),
        source_local_id: source_local_id.map(ToOwned::to_owned),
    }
}

fn report_scenario(
    input: &ArchitectureInputLiftDocument,
    lifted: &LiftedArchitecture,
) -> ArchitectureInputLiftScenario {
    ArchitectureInputLiftScenario {
        input_schema: input.schema.clone(),
        source: input.source.clone(),
        space: lifted.space.clone(),
        contexts: input.contexts.clone(),
        cells: lifted.cells.clone(),
        incidences: lifted.incidences.clone(),
    }
}

fn report_result(lifted: &LiftedArchitecture) -> ArchitectureInputLiftResult {
    let accepted_fact_ids = accepted_fact_ids(&lifted.cells, &lifted.incidences);
    let inferred_structure_ids = lifted
        .completion_candidates
        .iter()
        .map(|candidate| candidate.id.clone())
        .collect();
    ArchitectureInputLiftResult {
        status: ArchitectureInputLiftStatus::Lifted,
        accepted_fact_ids,
        inferred_structure_ids,
        completion_candidates: lifted.completion_candidates.clone(),
    }
}

fn report_projection(
    result: &ArchitectureInputLiftResult,
    lifted: &LiftedArchitecture,
    interpretation: &InterpretationPackage,
) -> RuntimeResult<ArchitectureInputLiftProjection> {
    let template = architecture_review_projection_template(interpretation)?;
    let source_ids = projection_source_ids(result);
    let human_loss = InformationLoss::declared(
        "Projection summarizes accepted architecture facts and unreviewed inferred structures.",
        source_ids.clone(),
    )?;
    let ai_loss = InformationLoss::declared(
        "AI view preserves accepted fact and completion candidate records but omits the full lifted space payload.",
        source_ids.clone(),
    )?;
    let audit_loss = InformationLoss::declared(
        "Audit trace records represented source identifiers and view coverage but omits full object payloads.",
        source_ids.clone(),
    )?;
    let human_review = HumanReviewProjectionView {
        audience: projection_audience(template)?,
        purpose: projection_purpose(template)?,
        summary: format!(
            "Lifted {} accepted facts and preserved {} inferred structures as unreviewed candidates.",
            result.accepted_fact_ids.len(),
            result.inferred_structure_ids.len()
        ),
        recommended_actions: vec![
            "Review completion candidates before promoting any inferred structure.".to_owned(),
            "Extend input ingestion only after this bounded JSON path remains stable.".to_owned(),
        ],
        source_ids: source_ids.clone(),
        information_loss: vec![human_loss],
    };
    let ai_view = AiProjectionView {
        audience: ProjectionAudience::AiAgent,
        purpose: ProjectionPurpose::ArchitectureReview,
        records: ai_projection_records(lifted),
        source_ids: source_ids.clone(),
        information_loss: vec![ai_loss],
    };
    let audit_trace = AuditProjectionView {
        audience: ProjectionAudience::Audit,
        purpose: ProjectionPurpose::AuditTrace,
        source_ids,
        information_loss: vec![audit_loss],
        traces: audit_traces(ai_view.source_ids.clone()),
    };

    Ok(ProjectionViewSet {
        audience: human_review.audience,
        purpose: human_review.purpose,
        summary: human_review.summary.clone(),
        recommended_actions: human_review.recommended_actions.clone(),
        information_loss: human_review.information_loss.clone(),
        source_ids: human_review.source_ids.clone(),
        human_review,
        ai_view,
        audit_trace,
    })
}

fn projection_audience(template: &ProjectionTemplate) -> RuntimeResult<ProjectionAudience> {
    match template.audience.as_str() {
        "human" => Ok(ProjectionAudience::Human),
        audience => Err(validation_error(format!(
            "projection template {} uses unsupported audience {audience}",
            template.id
        ))),
    }
}

fn projection_purpose(template: &ProjectionTemplate) -> RuntimeResult<ProjectionPurpose> {
    match template.purpose.as_str() {
        "architecture_review" => Ok(ProjectionPurpose::ArchitectureReview),
        purpose => Err(validation_error(format!(
            "projection template {} uses unsupported purpose {purpose}",
            template.id
        ))),
    }
}

fn projection_source_ids(result: &ArchitectureInputLiftResult) -> Vec<Id> {
    let mut ids = Vec::new();
    for id in &result.accepted_fact_ids {
        push_unique(&mut ids, id.clone());
    }
    for candidate in &result.completion_candidates {
        push_unique(&mut ids, candidate.id.clone());
        for inferred_from in &candidate.inferred_from {
            push_unique(&mut ids, inferred_from.clone());
        }
    }
    ids
}

fn ai_projection_records(lifted: &LiftedArchitecture) -> Vec<AiProjectionRecord> {
    let cell_records = lifted.cells.iter().map(|cell| {
        let source_ids = cell_source_ids(cell);
        let provenance = cell.provenance.clone();
        AiProjectionRecord {
            id: cell.id.clone(),
            record_type: AiProjectionRecordType::Cell,
            summary: cell.label.clone().unwrap_or_else(|| cell.cell_type.clone()),
            source_ids,
            confidence: provenance.as_ref().map(|value| value.confidence),
            review_status: provenance.as_ref().map(|value| value.review_status),
            severity: None,
            provenance,
        }
    });
    let incidence_records = lifted.incidences.iter().map(|incidence| {
        let source_ids = incidence_source_ids(incidence);
        let provenance = incidence.provenance.clone();
        AiProjectionRecord {
            id: incidence.id.clone(),
            record_type: AiProjectionRecordType::Incidence,
            summary: format!(
                "{} {} {}",
                incidence.from_cell_id, incidence.relation_type, incidence.to_cell_id
            ),
            source_ids,
            confidence: provenance.as_ref().map(|value| value.confidence),
            review_status: provenance.as_ref().map(|value| value.review_status),
            severity: None,
            provenance,
        }
    });
    let candidate_records =
        lifted
            .completion_candidates
            .iter()
            .map(|candidate| AiProjectionRecord {
                id: candidate.id.clone(),
                record_type: AiProjectionRecordType::CompletionCandidate,
                summary: candidate.suggested_structure.summary.clone(),
                source_ids: completion_candidate_source_ids(candidate),
                confidence: Some(candidate.confidence),
                review_status: Some(candidate.review_status),
                severity: None,
                provenance: None,
            });

    cell_records
        .chain(incidence_records)
        .chain(candidate_records)
        .collect()
}

fn cell_source_ids(cell: &Cell) -> Vec<Id> {
    let mut ids = vec![cell.id.clone(), cell.space_id.clone()];
    for context_id in &cell.context_ids {
        push_unique(&mut ids, context_id.clone());
    }
    ids
}

fn incidence_source_ids(incidence: &Incidence) -> Vec<Id> {
    vec![
        incidence.id.clone(),
        incidence.space_id.clone(),
        incidence.from_cell_id.clone(),
        incidence.to_cell_id.clone(),
    ]
}

fn completion_candidate_source_ids(candidate: &CompletionCandidate) -> Vec<Id> {
    let mut ids = vec![candidate.id.clone(), candidate.space_id.clone()];
    if let Some(structure_id) = &candidate.suggested_structure.structure_id {
        push_unique(&mut ids, structure_id.clone());
    }
    for related_id in &candidate.suggested_structure.related_ids {
        push_unique(&mut ids, related_id.clone());
    }
    for inferred_from in &candidate.inferred_from {
        push_unique(&mut ids, inferred_from.clone());
    }
    ids
}

fn audit_traces(source_ids: Vec<Id>) -> Vec<ProjectionTrace> {
    source_ids
        .into_iter()
        .map(|source_id| ProjectionTrace {
            role: source_role(&source_id).to_owned(),
            source_id,
            represented_in: vec![
                "human_review".to_owned(),
                "ai_view".to_owned(),
                "audit_trace".to_owned(),
            ],
        })
        .collect()
}

fn source_role(source_id: &Id) -> &'static str {
    if source_id.as_str().starts_with("cell:") {
        "cell"
    } else if source_id.as_str().starts_with("incidence:") {
        "incidence"
    } else if source_id.as_str().starts_with("candidate:") {
        "completion_candidate"
    } else if source_id.as_str().starts_with("context:") {
        "context"
    } else if source_id.as_str().starts_with("space:") {
        "space"
    } else {
        "source"
    }
}

fn accepted_fact_ids(cells: &[Cell], incidences: &[Incidence]) -> Vec<Id> {
    let mut ids = Vec::with_capacity(cells.len() + incidences.len());
    ids.extend(cells.iter().map(|cell| cell.id.clone()));
    ids.extend(incidences.iter().map(|incidence| incidence.id.clone()));
    ids
}

fn validate_input_schema(input: &ArchitectureInputLiftDocument) -> RuntimeResult<()> {
    if input.schema == INPUT_SCHEMA {
        return Ok(());
    }
    Err(RuntimeError::unsupported_input_schema(
        input.schema.clone(),
        INPUT_SCHEMA,
    ))
}

fn validate_input_references(input: &ArchitectureInputLiftDocument) -> RuntimeResult<()> {
    ensure_unique_input_ids(input)?;
    ensure_component_contexts_declared(input)?;
    ensure_relation_endpoints_reference_components(input)?;
    ensure_inference_contexts_declared(input)?;
    ensure_inference_related_ids_accepted(input)
}

fn ensure_unique_input_ids(input: &ArchitectureInputLiftDocument) -> RuntimeResult<()> {
    let mut seen = Vec::new();
    ensure_unique_id(&mut seen, &input.space.id, "space")?;
    for context in &input.contexts {
        ensure_unique_id(&mut seen, &context.id, "context")?;
    }
    for component in &input.components {
        ensure_unique_id(&mut seen, &component.id, "component")?;
    }
    for relation in &input.relations {
        ensure_unique_id(&mut seen, &relation.id, "relation")?;
    }
    for inference in &input.inferred_structures {
        ensure_unique_id(&mut seen, &inference.id, "inference")?;
    }
    Ok(())
}

fn ensure_unique_id(
    seen: &mut Vec<(Id, &'static str)>,
    id: &Id,
    role: &'static str,
) -> RuntimeResult<()> {
    if let Some((_, existing_role)) = seen.iter().find(|(seen_id, _)| seen_id == id) {
        return Err(validation_error(format!(
            "{role} id {id} duplicates existing {existing_role} id"
        )));
    }
    seen.push((id.clone(), role));
    Ok(())
}

fn ensure_component_contexts_declared(input: &ArchitectureInputLiftDocument) -> RuntimeResult<()> {
    let context_ids = input
        .contexts
        .iter()
        .map(|context| context.id.clone())
        .collect::<Vec<_>>();
    for component in &input.components {
        if !context_ids.contains(&component.context_id) {
            return Err(validation_error(format!(
                "component {} references undeclared context {}",
                component.id, component.context_id
            )));
        }
    }
    Ok(())
}

fn ensure_relation_endpoints_reference_components(
    input: &ArchitectureInputLiftDocument,
) -> RuntimeResult<()> {
    let component_ids = input_component_ids(input);
    for relation in &input.relations {
        ensure_known_id(
            &component_ids,
            &relation.from_cell_id,
            "relation",
            &relation.id,
            "from component",
        )?;
        ensure_known_id(
            &component_ids,
            &relation.to_cell_id,
            "relation",
            &relation.id,
            "to component",
        )?;
    }
    Ok(())
}

fn ensure_inference_contexts_declared(input: &ArchitectureInputLiftDocument) -> RuntimeResult<()> {
    let context_ids = input
        .contexts
        .iter()
        .map(|context| context.id.clone())
        .collect::<Vec<_>>();
    for inference in &input.inferred_structures {
        for context_id in &inference.context_ids {
            ensure_known_id(
                &context_ids,
                context_id,
                "inference",
                &inference.id,
                "context",
            )?;
        }
    }
    Ok(())
}

fn ensure_inference_related_ids_accepted(
    input: &ArchitectureInputLiftDocument,
) -> RuntimeResult<()> {
    let accepted_ids = input_accepted_fact_ids(input);
    for inference in &input.inferred_structures {
        for related_id in &inference.related_ids {
            ensure_known_id(
                &accepted_ids,
                related_id,
                "inference",
                &inference.id,
                "related source",
            )?;
        }
    }
    Ok(())
}

fn ensure_inferred_from_accepted(
    input: &ArchitectureInputLiftDocument,
    inference: &ArchitectureInputInference,
) -> RuntimeResult<()> {
    let accepted_ids = input_accepted_fact_ids(input);
    for source_id in &inference.inferred_from {
        ensure_known_id(
            &accepted_ids,
            source_id,
            "inference",
            &inference.id,
            "inferred_from source",
        )?;
    }
    Ok(())
}

fn ensure_inferences_are_not_accepted(
    cells: &[Cell],
    candidates: &[CompletionCandidate],
) -> RuntimeResult<()> {
    let accepted_cell_ids = cells.iter().map(|cell| cell.id.clone()).collect::<Vec<_>>();
    for candidate in candidates {
        let structure_id = candidate.suggested_structure.structure_id.as_ref();
        if structure_id.is_some_and(|id| accepted_cell_ids.contains(id)) {
            return Err(validation_error(format!(
                "candidate {} proposes an already accepted cell",
                candidate.id
            )));
        }
    }
    Ok(())
}

fn input_accepted_fact_ids(input: &ArchitectureInputLiftDocument) -> Vec<Id> {
    let mut ids = Vec::with_capacity(input.components.len() + input.relations.len());
    ids.extend(
        input
            .components
            .iter()
            .map(|component| component.id.clone()),
    );
    ids.extend(input.relations.iter().map(|relation| relation.id.clone()));
    ids
}

fn input_component_ids(input: &ArchitectureInputLiftDocument) -> Vec<Id> {
    input
        .components
        .iter()
        .map(|component| component.id.clone())
        .collect()
}

fn ensure_known_id(
    known_ids: &[Id],
    referenced_id: &Id,
    owner_role: &str,
    owner_id: &Id,
    referenced_role: &str,
) -> RuntimeResult<()> {
    if known_ids.contains(referenced_id) {
        return Ok(());
    }
    Err(validation_error(format!(
        "{owner_role} {owner_id} references unknown {referenced_role} {referenced_id}"
    )))
}

fn push_unique(ids: &mut Vec<Id>, id: Id) {
    if !ids.contains(&id) {
        ids.push(id);
    }
}

fn id(value: impl Into<String>) -> RuntimeResult<Id> {
    Ok(Id::new(value)?)
}

fn missing_inserted(target: &str) -> RuntimeError {
    validation_error(format!("{target} was not available after insertion"))
}

fn validation_error(reason: impl Into<String>) -> RuntimeError {
    RuntimeError::workflow_construction(WORKFLOW_NAME, reason)
}
