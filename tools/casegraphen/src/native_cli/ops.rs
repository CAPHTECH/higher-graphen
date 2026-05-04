use super::{
    path_helpers::{id_lossy, path_segment, relative_store_path},
    reporting::report,
    NativeCliError, NativeReasonSection,
};
use crate::{
    core_extension_bridge::{
        native_close_check_extensions, native_close_check_result, native_morphism_check_extensions,
        native_morphism_check_result,
    },
    native_eval::evaluate_native_case,
    native_model::{
        CaseCell, CaseCellLifecycle, CaseCellType, CaseMorphism, CaseMorphismType, CaseSpace,
        MorphismLogEntry, ProjectionAudience, ReviewAction, Revision, NATIVE_CASE_SPACE_SCHEMA,
        NATIVE_CASE_SPACE_SCHEMA_VERSION, NATIVE_MORPHISM_LOG_ENTRY_SCHEMA,
    },
    native_review::{check_native_close, NativeCloseCheckRequest, NativeOperationGate},
    native_store::NativeCaseStore,
    topology::TopologyReportOptions,
};
use higher_graphen_core::{Id, Provenance, ReviewStatus, SourceKind};
use serde_json::{json, Map, Value};
use std::path::Path;

mod io;
use io::{
    case_space_checksum, known_ids, provenance, proposal_path, proposal_value, read_case_space,
    read_morphism, read_proposal, timestamp, write_json,
};

pub(super) fn case_new(
    store: &Path,
    case_space_id: &Id,
    space_id: &Id,
    title: &str,
    revision_id: &Id,
) -> Result<Value, NativeCliError> {
    let case_space = new_case_space(case_space_id, space_id, title, revision_id)?;
    let record = NativeCaseStore::new(store.to_path_buf()).import_case_space(&case_space)?;
    Ok(report(
        "casegraphen case new",
        json!({ "record": record, "case_space": case_space }),
    ))
}

pub(super) fn case_import(
    store: &Path,
    input: &Path,
    revision_id: &Id,
) -> Result<Value, NativeCliError> {
    let mut case_space = read_case_space(input)?;
    retarget_latest_revision(&mut case_space, revision_id)?;
    let record = NativeCaseStore::new(store.to_path_buf()).import_case_space(&case_space)?;
    Ok(report(
        "casegraphen case import",
        json!({ "record": record, "case_space": case_space }),
    ))
}

pub(super) fn case_reason(
    store: &Path,
    case_space_id: &Id,
    section: NativeReasonSection,
) -> Result<Value, NativeCliError> {
    let replay =
        NativeCaseStore::new(store.to_path_buf()).replay_current_case_space(case_space_id)?;
    let evaluation = evaluate_native_case(&replay.case_space)?;
    let (command, result) = match section {
        NativeReasonSection::Reason => (
            "casegraphen case reason",
            json!({ "evaluation": evaluation }),
        ),
        NativeReasonSection::Frontier => (
            "casegraphen case frontier",
            json!({ "frontier_cell_ids": evaluation.frontier_cell_ids }),
        ),
        NativeReasonSection::Obstructions => (
            "casegraphen case obstructions",
            json!({ "obstructions": evaluation.obstructions }),
        ),
        NativeReasonSection::Completions => (
            "casegraphen case completions",
            json!({ "completion_candidates": evaluation.completion_candidates }),
        ),
        NativeReasonSection::Evidence => (
            "casegraphen case evidence",
            json!({ "evidence_findings": evaluation.evidence_findings }),
        ),
        NativeReasonSection::Project => (
            "casegraphen case project",
            json!({
                "projections": replay.case_space.projections,
                "projection_loss": evaluation.projection_loss,
            }),
        ),
    };
    Ok(report(command, result))
}

pub(super) fn case_close_check(
    store: &Path,
    case_space_id: &Id,
    base_revision_id: &Id,
    validation_evidence_ids: &[Id],
    gate_options: NativeCloseGateOptions,
) -> Result<Value, NativeCliError> {
    let replay =
        NativeCaseStore::new(store.to_path_buf()).replay_current_case_space(case_space_id)?;
    let operation_gate = close_operation_gate(&replay.case_space, gate_options)?;
    let check = check_native_close(
        &replay.case_space,
        NativeCloseCheckRequest {
            close_policy_id: operation_gate.close_policy_id.clone(),
            base_revision_id: base_revision_id.clone(),
            declared_projection_loss_ids: Vec::new(),
            validation_evidence_ids: validation_evidence_ids.to_vec(),
            source_ids: validation_evidence_ids.to_vec(),
            operation_gate: Some(operation_gate.gate),
        },
    )?;
    let core_extensions = native_close_check_extensions(&replay.case_space, &check);
    Ok(report(
        "casegraphen case close-check",
        native_close_check_result(check, core_extensions),
    ))
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct NativeCloseGateOptions {
    pub(super) close_policy_id: Option<Id>,
    pub(super) actor_id: Option<Id>,
    pub(super) capability_ids: Vec<Id>,
    pub(super) operation_scope_id: Option<Id>,
    pub(super) audience: Option<ProjectionAudience>,
    pub(super) source_boundary_id: Option<Id>,
}

struct ResolvedCloseGate {
    close_policy_id: Option<Id>,
    gate: NativeOperationGate,
}

pub(super) fn case_topology(
    store: &Path,
    case_space_id: &Id,
    topology_options: TopologyReportOptions,
) -> Result<Value, NativeCliError> {
    let replay =
        NativeCaseStore::new(store.to_path_buf()).replay_current_case_space(case_space_id)?;
    let topology = crate::topology::native_case_topology_with_history(
        &replay.case_space,
        &replay.history,
        topology_options,
    )?;
    Ok(report(
        "casegraphen case history topology",
        json!({ "topology": topology }),
    ))
}

pub(super) fn case_topology_diff(
    left_store: &Path,
    left_case_space_id: &Id,
    right_store: &Path,
    right_case_space_id: &Id,
    topology_options: TopologyReportOptions,
) -> Result<Value, NativeCliError> {
    let left_replay = NativeCaseStore::new(left_store.to_path_buf())
        .replay_current_case_space(left_case_space_id)?;
    let right_replay = NativeCaseStore::new(right_store.to_path_buf())
        .replay_current_case_space(right_case_space_id)?;
    let left_topology = crate::topology::native_case_topology_with_history(
        &left_replay.case_space,
        &left_replay.history,
        topology_options,
    )?;
    let right_topology = crate::topology::native_case_topology_with_history(
        &right_replay.case_space,
        &right_replay.history,
        topology_options,
    )?;
    let topology_diff = crate::topology::topology_diff(&left_topology, &right_topology);
    Ok(report(
        "casegraphen case history topology diff",
        json!({
            "left_case_space_id": left_case_space_id,
            "right_case_space_id": right_case_space_id,
            "topology_diff": topology_diff
        }),
    ))
}

pub(super) fn morphism_propose(
    store: &Path,
    case_space_id: &Id,
    input: &Path,
) -> Result<Value, NativeCliError> {
    let replay =
        NativeCaseStore::new(store.to_path_buf()).replay_current_case_space(case_space_id)?;
    let morphism = read_morphism(input)?;
    validate_candidate_morphism(&replay.case_space, &morphism)?;
    let proposal = proposal_value(case_space_id, &morphism);
    let path = proposal_path(store, case_space_id, &morphism.morphism_id)?;
    write_json(&path, &proposal)?;
    Ok(report(
        "casegraphen morphism propose",
        json!({
            "proposal_status": "checked",
            "proposal_path": relative_store_path(store, &path),
            "morphism": morphism
        }),
    ))
}

pub(super) fn morphism_check(
    store: &Path,
    case_space_id: &Id,
    morphism_id: &Id,
) -> Result<Value, NativeCliError> {
    let replay =
        NativeCaseStore::new(store.to_path_buf()).replay_current_case_space(case_space_id)?;
    let morphism = read_proposal(store, case_space_id, morphism_id)?;
    validate_candidate_morphism(&replay.case_space, &morphism)?;
    let core_extensions = native_morphism_check_extensions(&replay.case_space, &morphism);
    Ok(report(
        "casegraphen morphism check",
        native_morphism_check_result(morphism, core_extensions),
    ))
}

pub(super) fn morphism_apply(
    store: &Path,
    case_space_id: &Id,
    morphism_id: &Id,
    base_revision_id: &Id,
    reviewer_id: Option<&Id>,
    reason: Option<&str>,
) -> Result<Value, NativeCliError> {
    let store_api = NativeCaseStore::new(store.to_path_buf());
    let replay = store_api.replay_current_case_space(case_space_id)?;
    if &replay.current_revision_id != base_revision_id {
        return Err(NativeCliError::invalid(format!(
            "base revision {base_revision_id} is stale; current revision is {}",
            replay.current_revision_id
        )));
    }
    let mut morphism = read_proposal(store, case_space_id, morphism_id)?;
    validate_candidate_morphism(&replay.case_space, &morphism)?;
    morphism.review_status = ReviewStatus::Accepted;
    if let Some(reviewer_id) = reviewer_id {
        morphism
            .metadata
            .insert("reviewer_id".to_owned(), json!(reviewer_id));
    }
    if let Some(reason) = reason {
        if reason.trim().is_empty() {
            return Err(NativeCliError::invalid("review reason must not be empty"));
        }
        morphism
            .metadata
            .insert("review_reason".to_owned(), json!(reason.trim()));
    }
    let mut entry = entry_for_morphism(&replay.case_space, morphism.clone(), None)?;
    entry.replay_checksum = checksum_after_append(&replay.case_space, &entry)?;
    let record = store_api.append_morphism(case_space_id, entry.clone())?;
    Ok(report(
        "casegraphen morphism apply",
        json!({ "record": record, "entry": entry }),
    ))
}

pub(super) fn morphism_reject(
    store: &Path,
    case_space_id: &Id,
    morphism_id: &Id,
    reviewer_id: &Id,
    reason: &str,
    revision_id: &Id,
) -> Result<Value, NativeCliError> {
    let store_api = NativeCaseStore::new(store.to_path_buf());
    let replay = store_api.replay_current_case_space(case_space_id)?;
    let proposal = read_proposal(store, case_space_id, morphism_id)?;
    validate_candidate_morphism(&replay.case_space, &proposal)?;
    let review = review_morphism(
        &replay.case_space.revision.revision_id,
        revision_id,
        morphism_id,
        reviewer_id,
        reason,
    )?;
    let mut entry = entry_for_morphism(&replay.case_space, review, Some(reviewer_id.clone()))?;
    entry.replay_checksum = checksum_after_append(&replay.case_space, &entry)?;
    let record = store_api.append_morphism(case_space_id, entry.clone())?;
    Ok(report(
        "casegraphen morphism reject",
        json!({ "record": record, "entry": entry, "rejected_morphism": proposal }),
    ))
}

fn new_case_space(
    case_space_id: &Id,
    space_id: &Id,
    title: &str,
    revision_id: &Id,
) -> Result<CaseSpace, NativeCliError> {
    if title.trim().is_empty() {
        return Err(NativeCliError::invalid("case title must not be empty"));
    }
    let cell_id = Id::new("case:native-root".to_owned())?;
    let source_id = Id::new("source:native-cli".to_owned())?;
    let morphism_id = Id::new(format!("morphism:create:{}", path_segment(case_space_id)))?;
    let entry_id = Id::new(format!(
        "morphism_log_entry:create:{}",
        path_segment(case_space_id)
    ))?;
    let source_boundary = source_boundary_value(
        Id::new(format!(
            "source_boundary:{}",
            path_segment(case_space_id)
        ))?,
        std::slice::from_ref(&source_id),
        &["native.case.new.v1"],
        "native CLI source fields are accepted as explicit user input; inferred fields need review before close.",
        "case new records no inferred facts beyond the requested identifiers and title.",
        Vec::new(),
    );
    let now = timestamp();
    let provenance = provenance(SourceKind::Human, ReviewStatus::Accepted);
    let entry = genesis_entry(GenesisEntryInput {
        case_space_id,
        revision_id,
        cell_id: &cell_id,
        source_id: &source_id,
        morphism_id,
        entry_id,
        recorded_at: &now,
        provenance: &provenance,
        source_boundary: source_boundary.clone(),
    })?;
    let mut metadata = Map::new();
    metadata.insert("source_boundary".to_owned(), source_boundary);
    let mut case_space = CaseSpace {
        schema: NATIVE_CASE_SPACE_SCHEMA.to_owned(),
        schema_version: NATIVE_CASE_SPACE_SCHEMA_VERSION,
        case_space_id: case_space_id.clone(),
        space_id: space_id.clone(),
        case_cells: vec![root_case_cell(
            cell_id,
            space_id,
            title,
            &source_id,
            &provenance,
        )],
        case_relations: Vec::new(),
        morphism_log: vec![entry],
        projections: Vec::new(),
        revision: Revision {
            revision_id: revision_id.clone(),
            case_space_id: case_space_id.clone(),
            applied_entry_ids: Vec::new(),
            applied_morphism_ids: Vec::new(),
            checksum: String::new(),
            parent_revision_id: None,
            created_at: now,
            source_ids: vec![source_id],
            metadata: Map::new(),
        },
        close_policy_id: None,
        metadata,
    };
    case_space.revision.applied_entry_ids = vec![case_space.morphism_log[0].entry_id.clone()];
    case_space.revision.applied_morphism_ids = vec![case_space.morphism_log[0].morphism_id.clone()];
    let checksum = case_space_checksum(&case_space)?;
    case_space.revision.checksum = checksum.clone();
    case_space.morphism_log[0].replay_checksum = checksum;
    Ok(case_space)
}

struct GenesisEntryInput<'a> {
    case_space_id: &'a Id,
    revision_id: &'a Id,
    cell_id: &'a Id,
    source_id: &'a Id,
    morphism_id: Id,
    entry_id: Id,
    recorded_at: &'a str,
    provenance: &'a Provenance,
    source_boundary: Value,
}

fn genesis_entry(input: GenesisEntryInput<'_>) -> Result<MorphismLogEntry, NativeCliError> {
    let mut metadata = Map::new();
    metadata.insert(
        "lift_semantics".to_owned(),
        json!("native_cli_request_to_case_space"),
    );
    metadata.insert(
        "source_boundary_id".to_owned(),
        json!(Id::new(format!(
            "source_boundary:{}",
            path_segment(input.case_space_id)
        ))?),
    );
    metadata.insert("source_boundary".to_owned(), input.source_boundary);
    let morphism = CaseMorphism {
        morphism_id: input.morphism_id.clone(),
        morphism_type: CaseMorphismType::Create,
        source_revision_id: None,
        target_revision_id: input.revision_id.clone(),
        added_ids: vec![input.cell_id.clone()],
        updated_ids: Vec::new(),
        retired_ids: Vec::new(),
        preserved_ids: Vec::new(),
        violated_invariant_ids: Vec::new(),
        review_status: ReviewStatus::Accepted,
        evidence_ids: Vec::new(),
        source_ids: vec![input.source_id.clone()],
        metadata,
    };
    Ok(MorphismLogEntry {
        schema: NATIVE_MORPHISM_LOG_ENTRY_SCHEMA.to_owned(),
        schema_version: NATIVE_CASE_SPACE_SCHEMA_VERSION,
        case_space_id: input.case_space_id.clone(),
        sequence: 1,
        entry_id: input.entry_id,
        morphism_id: input.morphism_id,
        source_revision_id: None,
        target_revision_id: input.revision_id.clone(),
        morphism,
        actor_id: Id::new("actor:native-cli".to_owned())?,
        recorded_at: input.recorded_at.to_owned(),
        provenance: input.provenance.clone(),
        source_ids: vec![input.source_id.clone()],
        previous_entry_hash: None,
        replay_checksum: String::new(),
    })
}

fn root_case_cell(
    cell_id: Id,
    space_id: &Id,
    title: &str,
    source_id: &Id,
    provenance: &Provenance,
) -> CaseCell {
    CaseCell {
        id: cell_id,
        cell_type: CaseCellType::Case,
        space_id: space_id.clone(),
        title: title.trim().to_owned(),
        summary: None,
        lifecycle: CaseCellLifecycle::Active,
        source_ids: vec![source_id.clone()],
        structure_ids: Vec::new(),
        provenance: provenance.clone(),
        metadata: Map::new(),
    }
}

fn source_boundary_value(
    source_boundary_id: Id,
    included_sources: &[Id],
    adapters: &[&str],
    accepted_fact_policy: &str,
    inference_policy: &str,
    information_loss: Vec<Value>,
) -> Value {
    json!({
        "id": source_boundary_id,
        "included_sources": included_sources,
        "excluded_sources": [],
        "adapters": adapters,
        "accepted_fact_policy": accepted_fact_policy,
        "inference_policy": inference_policy,
        "information_loss": information_loss
    })
}

fn close_operation_gate(
    case_space: &CaseSpace,
    options: NativeCloseGateOptions,
) -> Result<ResolvedCloseGate, NativeCliError> {
    let source_boundary_id = match options.source_boundary_id {
        Some(id) => id,
        None => declared_source_boundary_id(case_space).ok_or_else(|| {
            NativeCliError::invalid(
                "case space does not declare a source boundary id for close-check",
            )
        })?,
    };
    let actor_id = options
        .actor_id
        .unwrap_or_else(|| id_lossy("actor:casegraphen-cli"));
    let capability_ids = if options.capability_ids.is_empty() {
        vec![id_lossy("capability:casegraphen-cli:close-check")]
    } else {
        options.capability_ids
    };
    Ok(ResolvedCloseGate {
        close_policy_id: options.close_policy_id,
        gate: NativeOperationGate {
            actor_id,
            operation: "close-check".to_owned(),
            operation_scope_id: options
                .operation_scope_id
                .unwrap_or_else(|| case_space.case_space_id.clone()),
            audience: options.audience.unwrap_or(ProjectionAudience::Audit),
            capability_ids,
            source_boundary_id,
        },
    })
}

fn declared_source_boundary_id(case_space: &CaseSpace) -> Option<Id> {
    case_space
        .metadata
        .get("source_boundary")
        .and_then(Value::as_object)
        .and_then(|boundary| boundary.get("id"))
        .and_then(Value::as_str)
        .and_then(|value| Id::new(value.to_owned()).ok())
        .or_else(|| {
            case_space
                .morphism_log
                .first()
                .and_then(|entry| entry.morphism.metadata.get("source_boundary_id"))
                .and_then(Value::as_str)
                .and_then(|value| Id::new(value.to_owned()).ok())
        })
}

fn retarget_latest_revision(
    case_space: &mut CaseSpace,
    revision_id: &Id,
) -> Result<(), NativeCliError> {
    let latest = case_space
        .morphism_log
        .last_mut()
        .ok_or_else(|| NativeCliError::invalid("case space morphism_log is empty"))?;
    latest.target_revision_id = revision_id.clone();
    latest.morphism.target_revision_id = revision_id.clone();
    case_space.revision.revision_id = revision_id.clone();
    for projection in &mut case_space.projections {
        projection.revision_id = revision_id.clone();
    }
    case_space.revision.checksum.clear();
    latest.replay_checksum.clear();
    let checksum = case_space_checksum(case_space)?;
    case_space.revision.checksum = checksum.clone();
    case_space
        .morphism_log
        .last_mut()
        .expect("latest checked")
        .replay_checksum = checksum;
    Ok(())
}

fn validate_candidate_morphism(
    case_space: &CaseSpace,
    morphism: &CaseMorphism,
) -> Result<(), NativeCliError> {
    if morphism.source_revision_id.as_ref() != Some(&case_space.revision.revision_id) {
        return Err(NativeCliError::invalid(format!(
            "morphism source_revision_id {:?} does not match current revision {}",
            morphism.source_revision_id, case_space.revision.revision_id
        )));
    }
    if morphism.target_revision_id == case_space.revision.revision_id {
        return Err(NativeCliError::invalid(
            "morphism target_revision_id must advance the revision",
        ));
    }
    if !morphism.added_ids.is_empty()
        || !morphism.updated_ids.is_empty()
        || !morphism.retired_ids.is_empty()
    {
        return Err(NativeCliError::invalid(
            "native morphism CLI currently accepts metadata-only morphisms",
        ));
    }
    let known = known_ids(case_space);
    for id in morphism.preserved_ids.iter().chain(&morphism.evidence_ids) {
        if !known.contains(id) {
            return Err(NativeCliError::invalid(format!(
                "unknown referenced id {id}"
            )));
        }
    }
    Ok(())
}

fn entry_for_morphism(
    case_space: &CaseSpace,
    morphism: CaseMorphism,
    actor_id: Option<Id>,
) -> Result<MorphismLogEntry, NativeCliError> {
    Ok(MorphismLogEntry {
        schema: NATIVE_MORPHISM_LOG_ENTRY_SCHEMA.to_owned(),
        schema_version: NATIVE_CASE_SPACE_SCHEMA_VERSION,
        case_space_id: case_space.case_space_id.clone(),
        sequence: case_space.morphism_log.len() as u64 + 1,
        entry_id: Id::new(format!(
            "morphism_log_entry:{}:{}",
            path_segment(&morphism.morphism_id),
            case_space.morphism_log.len() + 1
        ))?,
        morphism_id: morphism.morphism_id.clone(),
        source_revision_id: morphism.source_revision_id.clone(),
        target_revision_id: morphism.target_revision_id.clone(),
        actor_id: actor_id.unwrap_or_else(|| id_lossy("actor:native-cli")),
        recorded_at: timestamp(),
        provenance: provenance(SourceKind::Human, ReviewStatus::Accepted),
        source_ids: morphism.source_ids.clone(),
        previous_entry_hash: None,
        replay_checksum: String::new(),
        morphism,
    })
}

fn review_morphism(
    source_revision_id: &Id,
    target_revision_id: &Id,
    rejected_morphism_id: &Id,
    reviewer_id: &Id,
    reason: &str,
) -> Result<CaseMorphism, NativeCliError> {
    if target_revision_id == source_revision_id {
        return Err(NativeCliError::invalid(
            "review target_revision_id must advance the revision",
        ));
    }
    if reason.trim().is_empty() {
        return Err(NativeCliError::invalid("review reason must not be empty"));
    }
    let morphism_id = Id::new(format!(
        "morphism:review-reject:{}:{}",
        path_segment(rejected_morphism_id),
        path_segment(target_revision_id)
    ))?;
    let mut metadata = Map::new();
    metadata.insert("target_kind".to_owned(), json!("morphism"));
    metadata.insert("target_id".to_owned(), json!(rejected_morphism_id));
    metadata.insert("action".to_owned(), json!(ReviewAction::Reject));
    metadata.insert(
        "outcome_review_status".to_owned(),
        json!(ReviewStatus::Rejected),
    );
    metadata.insert("reviewer_id".to_owned(), json!(reviewer_id));
    metadata.insert("reason".to_owned(), json!(reason.trim()));
    Ok(CaseMorphism {
        morphism_id,
        morphism_type: CaseMorphismType::Review,
        source_revision_id: Some(source_revision_id.clone()),
        target_revision_id: target_revision_id.clone(),
        added_ids: Vec::new(),
        updated_ids: Vec::new(),
        retired_ids: Vec::new(),
        preserved_ids: Vec::new(),
        violated_invariant_ids: Vec::new(),
        review_status: ReviewStatus::Accepted,
        evidence_ids: Vec::new(),
        source_ids: vec![rejected_morphism_id.clone()],
        metadata,
    })
}

fn checksum_after_append(
    case_space: &CaseSpace,
    entry: &MorphismLogEntry,
) -> Result<String, NativeCliError> {
    let mut next = case_space.clone();
    next.morphism_log.push(entry.clone());
    next.revision = Revision {
        revision_id: entry.target_revision_id.clone(),
        case_space_id: case_space.case_space_id.clone(),
        applied_entry_ids: vec![entry.entry_id.clone()],
        applied_morphism_ids: vec![entry.morphism_id.clone()],
        checksum: String::new(),
        parent_revision_id: entry.source_revision_id.clone(),
        created_at: entry.recorded_at.clone(),
        source_ids: entry.source_ids.clone(),
        metadata: Map::new(),
    };
    case_space_checksum(&next)
}

