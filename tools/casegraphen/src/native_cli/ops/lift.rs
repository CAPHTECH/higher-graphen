use super::{
    io::{case_space_checksum, read_case_space},
    new_case_space, path_segment, report, retarget_latest_revision, source_boundary_value,
    NativeCliError,
};
use crate::{native_model::CaseSpace, native_store::NativeCaseStore};
use higher_graphen_core::Id;
use serde_json::{json, Map, Value};
use std::path::Path;

pub(in crate::native_cli) fn case_new(
    store: &Path,
    case_space_id: &Id,
    space_id: &Id,
    title: &str,
    revision_id: &Id,
) -> Result<Value, NativeCliError> {
    let case_space = new_case_space(case_space_id, space_id, title, revision_id)?;
    let record = NativeCaseStore::new(store.to_path_buf()).import_case_space(&case_space)?;
    Ok(report(
        "casegraphen space new",
        json!({ "record": record, "case_space": case_space }),
    ))
}

pub(in crate::native_cli) fn case_import(
    store: &Path,
    input: &Path,
    revision_id: &Id,
) -> Result<Value, NativeCliError> {
    let mut case_space = read_case_space(input)?;
    retarget_latest_revision(&mut case_space, revision_id)?;
    let record = NativeCaseStore::new(store.to_path_buf()).import_case_space(&case_space)?;
    Ok(report(
        "casegraphen lift native",
        json!({ "record": record, "case_space": case_space }),
    ))
}

pub(in crate::native_cli) fn lift_structured_source(
    store: &Path,
    input: &Path,
    revision_id: &Id,
    adapter: &str,
) -> Result<Value, NativeCliError> {
    let lift = read_lift_input(input, adapter)?;
    let case_space_id = Id::new(format!("case_space:{}", path_segment(&lift.source_id)))?;
    let mut case_space = new_case_space(
        &case_space_id,
        &lift.space_id,
        &format!("Lifted {}", lift.source_schema),
        revision_id,
    )?;
    let source_boundary = source_boundary_value(
        Id::new(format!("source_boundary:{}", path_segment(&case_space_id)))?,
        std::slice::from_ref(&lift.lift_source_id),
        &[adapter],
        "Structured source records are accepted as bounded lift input; generated records require review before they satisfy hard requirements.",
        "Lift adapters preserve source identifiers and declare unsupported source fields as information loss.",
        vec![json!({
            "source_schema": lift.source_schema,
            "input": input.display().to_string(),
            "note": "The first lift adapter records source identity and boundary metadata; full cell/relation materialization is handled by later morphism reducers."
        })],
    );
    annotate_lift_metadata(&mut case_space, &lift, adapter, input, source_boundary);
    refresh_lift_checksums(&mut case_space)?;
    let record = NativeCaseStore::new(store.to_path_buf()).import_case_space(&case_space)?;
    Ok(report(
        &format!("casegraphen lift {adapter}"),
        json!({
            "record": record,
            "case_space": case_space,
            "lift": {
                "adapter": adapter,
                "source_schema": lift.source_schema,
                "input": input.display().to_string()
            }
        }),
    ))
}

struct LiftInput {
    source_schema: String,
    source_id: Id,
    space_id: Id,
    lift_source_id: Id,
}

fn read_lift_input(input: &Path, adapter: &str) -> Result<LiftInput, NativeCliError> {
    let raw = std::fs::read_to_string(input).map_err(|source| NativeCliError::Io {
        path: input.to_path_buf(),
        source,
    })?;
    let value: Value = serde_json::from_str(&raw)?;
    let object = value
        .as_object()
        .ok_or_else(|| NativeCliError::invalid("lift input must be a JSON object"))?;
    let source_schema = object
        .get("schema")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_owned();
    let source_id = source_id_for_lift(adapter, object)?;
    let space_id = object
        .get("space_id")
        .and_then(Value::as_str)
        .ok_or_else(|| NativeCliError::invalid("lift input must contain space_id"))?;
    let lift_source_id = Id::new(format!("source:{}", path_segment(&source_id)))?;
    Ok(LiftInput {
        source_schema,
        source_id,
        space_id: Id::new(space_id.to_owned())?,
        lift_source_id,
    })
}

fn annotate_lift_metadata(
    case_space: &mut CaseSpace,
    lift: &LiftInput,
    adapter: &str,
    input: &Path,
    source_boundary: Value,
) {
    case_space
        .metadata
        .insert("source_boundary".to_owned(), source_boundary.clone());
    case_space.metadata.insert(
        "lift".to_owned(),
        json!({
            "adapter": adapter,
            "source_schema": lift.source_schema,
            "source_id": lift.source_id,
            "input": input.display().to_string()
        }),
    );
    if let Some(entry) = case_space.morphism_log.first_mut() {
        entry.source_ids = vec![lift.lift_source_id.clone()];
        entry.morphism.source_ids = vec![lift.lift_source_id.clone()];
        entry
            .morphism
            .metadata
            .insert("lift_semantics".to_owned(), json!(adapter));
        entry
            .morphism
            .metadata
            .insert("source_boundary".to_owned(), source_boundary);
        entry
            .morphism
            .metadata
            .insert("source_schema".to_owned(), json!(lift.source_schema));
        entry
            .morphism
            .metadata
            .insert("input".to_owned(), json!(input.display().to_string()));
    }
    if let Some(cell) = case_space.case_cells.first_mut() {
        cell.source_ids = vec![lift.lift_source_id.clone()];
        cell.metadata
            .insert("lifted_from".to_owned(), json!(lift.source_id));
        cell.metadata
            .insert("source_schema".to_owned(), json!(lift.source_schema));
    }
    case_space.revision.source_ids = vec![lift.lift_source_id.clone()];
}

fn refresh_lift_checksums(case_space: &mut CaseSpace) -> Result<(), NativeCliError> {
    case_space.revision.checksum.clear();
    if let Some(entry) = case_space.morphism_log.first_mut() {
        entry.replay_checksum.clear();
    }
    let checksum = case_space_checksum(case_space)?;
    case_space.revision.checksum = checksum.clone();
    if let Some(entry) = case_space.morphism_log.first_mut() {
        entry.replay_checksum = checksum;
    }
    Ok(())
}

fn source_id_for_lift(adapter: &str, object: &Map<String, Value>) -> Result<Id, NativeCliError> {
    let field = match adapter {
        "workflow" => "workflow_graph_id",
        "case-graph" => "case_graph_id",
        "native" => "case_space_id",
        _ => "id",
    };
    let raw = object
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| NativeCliError::invalid(format!("lift input must contain {field}")))?;
    Ok(Id::new(raw.to_owned())?)
}
