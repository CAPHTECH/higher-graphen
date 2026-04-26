use super::{NativeCliError, NativeReasonSection};
use crate::{
    native_eval::evaluate_native_case,
    native_model::{
        CaseCell, CaseCellLifecycle, CaseCellType, CaseMorphism, CaseMorphismType, CaseSpace,
        MorphismLogEntry, ReviewAction, Revision, NATIVE_CASE_SPACE_SCHEMA,
        NATIVE_CASE_SPACE_SCHEMA_VERSION, NATIVE_MORPHISM_LOG_ENTRY_SCHEMA,
    },
    native_review::{check_native_close, NativeCloseCheckRequest},
    native_store::NativeCaseStore,
};
use higher_graphen_core::{Confidence, Id, Provenance, ReviewStatus, SourceKind, SourceRef};
use serde_json::{json, Map, Value};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

const REPORT_SCHEMA: &str = "highergraphen.case.native_cli.report.v1";
const REPORT_TYPE: &str = "native_cli_operation";
const REPORT_VERSION: u32 = 1;
const PROPOSAL_SCHEMA: &str = "highergraphen.case.native_cli.morphism_proposal.v1";
const PROPOSAL_DIR: &str = "native_morphism_proposals";

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
) -> Result<Value, NativeCliError> {
    let replay =
        NativeCaseStore::new(store.to_path_buf()).replay_current_case_space(case_space_id)?;
    let check = check_native_close(
        &replay.case_space,
        NativeCloseCheckRequest {
            close_policy_id: None,
            base_revision_id: base_revision_id.clone(),
            declared_projection_loss_ids: Vec::new(),
            validation_evidence_ids: validation_evidence_ids.to_vec(),
            source_ids: validation_evidence_ids.to_vec(),
        },
    )?;
    Ok(report(
        "casegraphen case close-check",
        json!({ "close_check": check }),
    ))
}

pub(super) fn case_topology(store: &Path, case_space_id: &Id) -> Result<Value, NativeCliError> {
    let replay =
        NativeCaseStore::new(store.to_path_buf()).replay_current_case_space(case_space_id)?;
    let topology = crate::topology::native_case_topology(&replay.case_space)?;
    Ok(report(
        "casegraphen case history topology",
        json!({ "topology": topology }),
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
    Ok(report(
        "casegraphen morphism check",
        json!({ "valid": true, "applicable": true, "morphism": morphism }),
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
    })?;
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
        metadata: Map::new(),
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
}

fn genesis_entry(input: GenesisEntryInput<'_>) -> Result<MorphismLogEntry, NativeCliError> {
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
        metadata: Map::new(),
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

fn proposal_value(case_space_id: &Id, morphism: &CaseMorphism) -> Value {
    json!({
        "schema": PROPOSAL_SCHEMA,
        "schema_version": 1,
        "case_space_id": case_space_id,
        "morphism": morphism
    })
}

fn read_proposal(
    store: &Path,
    case_space_id: &Id,
    morphism_id: &Id,
) -> Result<CaseMorphism, NativeCliError> {
    let path = proposal_path(store, case_space_id, morphism_id)?;
    let value = read_json(&path)?;
    if value["schema"] != json!(PROPOSAL_SCHEMA) {
        return Err(NativeCliError::invalid(format!(
            "{}: unsupported proposal schema",
            path.display()
        )));
    }
    if value["schema_version"] != json!(NATIVE_CASE_SPACE_SCHEMA_VERSION) {
        return Err(NativeCliError::invalid(format!(
            "{}: unsupported proposal schema version",
            path.display()
        )));
    }
    if value["case_space_id"] != json!(case_space_id) {
        return Err(NativeCliError::invalid(format!(
            "{}: proposal belongs to a different case space",
            path.display()
        )));
    }
    let morphism: CaseMorphism = serde_json::from_value(value["morphism"].clone())?;
    if morphism.morphism_id != *morphism_id {
        return Err(NativeCliError::invalid(format!(
            "{}: proposal morphism id mismatch",
            path.display()
        )));
    }
    Ok(morphism)
}

fn proposal_path(
    store: &Path,
    case_space_id: &Id,
    morphism_id: &Id,
) -> Result<PathBuf, NativeCliError> {
    Ok(store
        .join(PROPOSAL_DIR)
        .join(path_segment(case_space_id))
        .join(format!("{}.case_morphism.json", path_segment(morphism_id))))
}

fn read_case_space(path: &Path) -> Result<CaseSpace, NativeCliError> {
    serde_json::from_value(read_json(path)?).map_err(NativeCliError::from)
}

fn read_morphism(path: &Path) -> Result<CaseMorphism, NativeCliError> {
    serde_json::from_value(read_json(path)?).map_err(NativeCliError::from)
}

fn read_json(path: &Path) -> Result<Value, NativeCliError> {
    let text = fs::read_to_string(path).map_err(|source| NativeCliError::Io {
        path: path.to_owned(),
        source,
    })?;
    serde_json::from_str(&text).map_err(NativeCliError::from)
}

fn write_json(path: &Path, value: &Value) -> Result<(), NativeCliError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| NativeCliError::Io {
            path: parent.to_owned(),
            source,
        })?;
    }
    let text = serde_json::to_string_pretty(value)?;
    fs::write(path, format!("{text}\n")).map_err(|source| NativeCliError::Io {
        path: path.to_owned(),
        source,
    })
}

pub(super) fn report(command: &str, result: Value) -> Value {
    json!({
        "schema": REPORT_SCHEMA,
        "report_type": REPORT_TYPE,
        "report_version": REPORT_VERSION,
        "metadata": {
            "command": command,
            "tool_package": "tools/casegraphen",
            "core_packages": [
                "higher-graphen-core"
            ]
        },
        "input": {
            "command": command
        },
        "result": result,
        "projection": {
            "human_review": {
                "summary": "Native CaseGraphen CLI operation completed."
            },
            "ai_view": {
                "operation": command,
                "native_boundary": "CaseSpace plus MorphismLog state is replayed before derived reports are emitted."
            },
            "audit_trace": {
                "source_ids": [],
                "information_loss": [
                    "Native CLI operation reports include the operation result but not a full command-line argv transcript."
                ]
            }
        }
    })
}

fn known_ids(case_space: &CaseSpace) -> Vec<Id> {
    case_space
        .case_cells
        .iter()
        .map(|cell| cell.id.clone())
        .chain(
            case_space
                .case_relations
                .iter()
                .map(|relation| relation.id.clone()),
        )
        .chain(
            case_space
                .projections
                .iter()
                .map(|projection| projection.projection_id.clone()),
        )
        .chain(
            case_space
                .morphism_log
                .iter()
                .flat_map(|entry| [entry.entry_id.clone(), entry.morphism_id.clone()]),
        )
        .chain([case_space.revision.revision_id.clone()])
        .collect()
}

fn case_space_checksum(case_space: &CaseSpace) -> Result<String, NativeCliError> {
    let mut value = serde_json::to_value(case_space)?;
    if let Value::Object(object) = &mut value {
        if let Some(Value::Object(revision)) = object.get_mut("revision") {
            revision.insert("checksum".to_owned(), Value::String(String::new()));
        }
        if let Some(Value::Array(log)) = object.get_mut("morphism_log") {
            for entry in log {
                if let Value::Object(entry) = entry {
                    entry.insert("replay_checksum".to_owned(), Value::String(String::new()));
                }
            }
        }
    }
    let canonical = serde_json::to_string(&value)?;
    Ok(format!("fnv1a64:{:016x}", fnv1a64(canonical.as_bytes())))
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn provenance(kind: SourceKind, review_status: ReviewStatus) -> Provenance {
    Provenance::new(
        SourceRef::new(kind),
        Confidence::new(1.0).expect("valid confidence"),
    )
    .with_review_status(review_status)
}

fn timestamp() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    format!("unix:{seconds}")
}

fn path_segment(id: &Id) -> String {
    let mut segment = String::new();
    for byte in id.as_str().bytes() {
        match byte {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' => {
                segment.push(byte as char);
            }
            _ => segment.push_str(&format!("~{byte:02x}")),
        }
    }
    segment
}

fn relative_store_path(store: &Path, path: &Path) -> String {
    path.strip_prefix(store)
        .unwrap_or(path)
        .display()
        .to_string()
}

fn id_lossy(value: &str) -> Id {
    Id::new(value.to_owned()).expect("static id is valid")
}
